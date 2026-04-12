/// Plugin engine for luna.
///
/// Plugins have full access to Lua stdlib and luna.* API.
/// They can register hooks for various shell events.
use crate::lua::api as luna_api;
use crate::shell::context::ShellContext;
use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use mlua::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// ─── Shared state accessible by plugin Lua closures ───────────────────────────

#[derive(Default, Clone)]
pub struct SharedState {
    pub env_vars: Arc<Mutex<IndexMap<String, String>>>,
    pub plugin_vars: Arc<Mutex<HashMap<String, String>>>,
    pub aliases: Arc<Mutex<HashMap<String, String>>>,
}

// ─── Event names ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HookEvent {
    PreCommand,
    PostCommand,
    DirectoryChange,
    Prompt,
    Unknown(String),
}

impl HookEvent {
    fn from_str(s: &str) -> Self {
        match s {
            "on_pre_command" => HookEvent::PreCommand,
            "on_post_command" => HookEvent::PostCommand,
            "on_directory_change" => HookEvent::DirectoryChange,
            "on_prompt" => HookEvent::Prompt,
            other => HookEvent::Unknown(other.to_string()),
        }
    }
}

// ─── Plugin engine ────────────────────────────────────────────────────────────

pub struct PluginEngine {
    plugins: Vec<LoadedPlugin>,
    pub shared: SharedState,
}

struct LoadedPlugin {
    name: String,
    hooks: HashMap<HookEvent, Vec<LuaRegistryKey>>,
    _lua: Lua, // Keep Lua state alive
}

impl PluginEngine {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            shared: SharedState::default(),
        }
    }

    pub fn load_dir(&mut self, dir: &PathBuf) {
        if !dir.exists() {
            return;
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("luna: plugins: cannot read {dir:?}: {e}");
                return;
            }
        };

        let mut paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "lua").unwrap_or(false))
            .collect();
        paths.sort();

        for path in paths {
            if let Err(e) = self.load_plugin(&path) {
                eprintln!("luna: plugin {path:?}: {e}");
            }
        }
    }

    pub fn load_plugin(&mut self, path: &PathBuf) -> Result<()> {
        let source =
            std::fs::read_to_string(path).map_err(|e| anyhow!("cannot read {path:?}: {e}"))?;

        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let lua = Lua::new_with(
            LuaStdLib::COROUTINE
                | LuaStdLib::IO
                | LuaStdLib::MATH
                | LuaStdLib::OS
                | LuaStdLib::PACKAGE
                | LuaStdLib::STRING
                | LuaStdLib::TABLE
                | LuaStdLib::UTF8,
            LuaOptions::default(),
        )
        .map_err(|e| anyhow!("{e}"))?;

        let plugin_hooks: Arc<Mutex<HashMap<HookEvent, Vec<LuaRegistryKey>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Register globals
        luna_api::register_globals(&lua).map_err(|e| anyhow!("{e}"))?;

        // Build luna table
        let luna = lua.create_table().map_err(|e| anyhow!("{e}"))?;
        luna_api::register_text_api(&lua, &luna).map_err(|e| anyhow!("{e}"))?;
        luna_api::register_ansi_api(&lua, &luna).map_err(|e| anyhow!("{e}"))?;
        luna_api::register_plugin_exec_api(&lua, &luna, self.shared.env_vars.clone())
            .map_err(|e| anyhow!("{e}"))?;
        luna_api::register_plugin_env_api(&lua, &luna, self.shared.env_vars.clone())
            .map_err(|e| anyhow!("{e}"))?;
        luna_api::register_plugin_vars_api(&lua, &luna, self.shared.plugin_vars.clone())
            .map_err(|e| anyhow!("{e}"))?;
        luna_api::register_plugin_alias_api(&lua, &luna, self.shared.aliases.clone())
            .map_err(|e| anyhow!("{e}"))?;

        // luna.hook
        {
            let hooks_clone = plugin_hooks.clone();
            let hook_fn = lua
                .create_function(move |lua_ctx, (event, func): (String, LuaFunction)| {
                    let ev = HookEvent::from_str(&event);
                    let key = lua_ctx.create_registry_value(func)?;
                    hooks_clone.lock().unwrap().entry(ev).or_default().push(key);
                    Ok(())
                })
                .map_err(|e| anyhow!("{e}"))?;
            luna.set("hook", hook_fn).map_err(|e| anyhow!("{e}"))?;
        }

        lua.globals()
            .set("luna", luna)
            .map_err(|e| anyhow!("{e}"))?;

        lua.load(&source)
            .set_name(&format!("plugin:{name}"))
            .exec()
            .map_err(|e| anyhow!("error in plugin '{name}': {e}"))?;

        let hooks = plugin_hooks
            .lock()
            .unwrap()
            .drain()
            .collect::<HashMap<_, _>>();

        self.plugins.push(LoadedPlugin {
            name,
            hooks,
            _lua: lua,
        });
        Ok(())
    }

    // ─── Hook firing ─────────────────────────────────────────────────────────

    pub fn fire_pre_command(&self, cmd: &str) {
        self.fire_event(&HookEvent::PreCommand, |_, func| {
            let _ = func.call::<()>(cmd.to_string());
        });
    }

    pub fn fire_post_command(&self, cmd: &str, code: i32, ms: u128) {
        self.fire_event(&HookEvent::PostCommand, |_, func| {
            let _ = func.call::<()>((cmd.to_string(), code, ms as i64));
        });
    }

    pub fn fire_dir_change(&self, old: &str, new: &str) {
        self.fire_event(&HookEvent::DirectoryChange, |_, func| {
            let _ = func.call::<()>((old.to_string(), new.to_string()));
        });
    }

    pub fn fire_prompt(&self, context: &ShellContext) {
        self.fire_event(&HookEvent::Prompt, |lua, func| {
            if let Ok(ctx_table) = luna_api::convert_context(lua, context) {
                let _ = lua.globals().set("ctx", ctx_table);
            }
            let _ = func.call::<()>(());
        });
    }

    fn fire_event<F>(&self, event: &HookEvent, mut call: F)
    where
        F: FnMut(&Lua, LuaFunction),
    {
        for plugin in &self.plugins {
            if let Some(keys) = plugin.hooks.get(event) {
                for key in keys {
                    if let Ok(func) = plugin._lua.registry_value::<LuaFunction>(key) {
                        call(&plugin._lua, func);
                    }
                }
            }
        }
    }

    pub fn sync_env_from(&self, env: &IndexMap<String, String>) {
        let mut guard = self.shared.env_vars.lock().unwrap();
        *guard = env.clone();
    }

    pub fn drain_env_into(&self, env: &mut IndexMap<String, String>) {
        let guard = self.shared.env_vars.lock().unwrap();
        for (k, v) in guard.iter() {
            env.insert(k.clone(), v.clone());
        }
    }

    pub fn sync_aliases_from(&self, aliases: &HashMap<String, String>) {
        let mut guard = self.shared.aliases.lock().unwrap();
        *guard = aliases.clone();
    }

    pub fn drain_aliases_into(&self, aliases: &mut HashMap<String, String>) {
        let guard = self.shared.aliases.lock().unwrap();
        *aliases = guard.clone();
    }

    pub fn plugin_vars_snapshot(&self) -> HashMap<String, String> {
        self.shared.plugin_vars.lock().unwrap().clone()
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    pub fn plugin_names(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name.as_str()).collect()
    }
}
