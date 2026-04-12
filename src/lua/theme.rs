/// Sandboxed Lua theme engine.
///
/// Themes must define a global `run()` function.
/// They have access to:
///   • `ctx` — Global table with shell state (cwd, user, hostname, etc.)
///   • `print(str)` — Add text to the prompt buffer
///   • `luna.text.*` — Port of the Rust rich-text engine
///   • `luna.ansi.*` — Raw ANSI constants
use crate::lua::api as luna_api;
use crate::shell::context::ShellContext;
use anyhow::{anyhow, Result};
use mlua::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct ThemeEngine {
    lua: Lua,
    buffer: Arc<Mutex<String>>,
    loaded: bool,
}

impl ThemeEngine {
    pub fn new() -> Result<Self> {
        let lua = Lua::new_with(
            LuaStdLib::STRING | LuaStdLib::MATH | LuaStdLib::TABLE | LuaStdLib::UTF8,
            LuaOptions::default(),
        );

        // Map mlua error to anyhow string to avoid Send/Sync issues with internal mlua errors
        let lua = lua.map_err(|e| anyhow!("{e}"))?;

        let buffer = Arc::new(Mutex::new(String::new()));

        // Global functions
        {
            let globals = lua.globals();

            // print(s) -> appends to local buffer
            let b_clone = buffer.clone();
            let print_fn = lua.create_function(move |_, s: String| {
                let mut b = b_clone.lock().unwrap();
                b.push_str(&s);
                Ok(())
            });
            let print_fn = print_fn.map_err(|e| anyhow!("{e}"))?;
            globals.set("print", print_fn).map_err(|e| anyhow!("{e}"))?;

            // luna module
            let luna = lua.create_table().map_err(|e| anyhow!("{e}"))?;
            luna_api::register_text_api(&lua, &luna).map_err(|e| anyhow!("{e}"))?;
            luna_api::register_ansi_api(&lua, &luna).map_err(|e| anyhow!("{e}"))?;
            globals.set("luna", luna).map_err(|e| anyhow!("{e}"))?;

            // Nuke dangerous globals
            for name in &[
                "load",
                "loadfile",
                "dofile",
                "require",
                "collectgarbage",
                "rawget",
                "rawset",
                "rawequal",
                "module",
                "package",
                "os",
                "io",
                "debug",
                "coroutine",
            ] {
                globals
                    .set(*name, LuaValue::Nil)
                    .map_err(|e| anyhow!("{e}"))?;
            }
        }

        Ok(Self {
            lua,
            buffer,
            loaded: false,
        })
    }

    pub fn load_file(&mut self, path: &PathBuf) -> Result<()> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("theme: cannot read {path:?}: {e}"))?;
        self.load_source(&source, path.to_string_lossy().as_ref())
    }

    pub fn load_source(&mut self, source: &str, name: &str) -> Result<()> {
        self.lua
            .load(source)
            .set_name(name)
            .exec()
            .map_err(|e| anyhow!("theme: error in '{name}': {e}"))?;
        self.loaded = true;
        Ok(())
    }

    /// Render the prompt by calling `run()` in Lua.
    pub fn render_prompt(&self, context: &ShellContext) -> Option<String> {
        if !self.loaded {
            return None;
        }

        // Reset buffer
        {
            let mut b = self.buffer.lock().unwrap();
            b.clear();
        }

        let globals = self.lua.globals();

        // Set the global `ctx` table
        if let Ok(ctx_table) = luna_api::convert_context(&self.lua, context) {
            let _ = globals.set("ctx", ctx_table);
        }

        // Call run()
        let run: LuaFunction = match globals.get("run") {
            Ok(f) => f,
            Err(_) => return None,
        };
        let result: LuaValue = run.call(()).ok().unwrap_or(LuaValue::Nil);

        let mut final_prompt = self.buffer.lock().unwrap().clone();

        // If run() returns a string, append it
        if let LuaValue::String(s) = result {
            if let Ok(s_str) = s.to_str() {
                final_prompt.push_str(&s_str);
            }
        }

        if final_prompt.is_empty() {
            None
        } else {
            Some(final_prompt)
        }
    }

    /// Render an error message if the theme defines `on_error(msg)`.
    pub fn render_error(&self, context: &ShellContext, message: &str) -> Option<String> {
        if !self.loaded {
            return None;
        }

        // Reset buffer
        {
            let mut b = self.buffer.lock().unwrap();
            b.clear();
        }

        let globals = self.lua.globals();

        // Set the global `ctx` table
        if let Ok(ctx_table) = luna_api::convert_context(&self.lua, context) {
            let _ = globals.set("ctx", ctx_table);
        }

        // Call on_error(msg)
        let on_error: LuaFunction = globals.get("on_error").ok()?;
        let _: () = on_error.call(message.to_string()).ok()?;

        let final_error = self.buffer.lock().unwrap().clone();
        if final_error.is_empty() {
            None
        } else {
            Some(final_error)
        }
    }

    /// Extract theme variables (primary, secondary, etc.) to use in shell builtins.
    pub fn extract_theme_vars(&self) -> std::collections::HashMap<String, String> {
        use std::collections::HashMap;
        let mut vars = HashMap::new();
        if !self.loaded {
            return vars;
        }

        let globals = self.lua.globals();
        let expected = ["primary", "secondary", "warn", "error", "text", "border"];

        for name in &expected {
            if let Ok(val) = globals.get::<String>(*name) {
                vars.insert(format!("color_{}", name), val);
            }
        }

        vars
    }
}
