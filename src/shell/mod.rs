pub mod config;
pub mod context;
pub mod corrector;
mod execute;
pub mod handlers;
pub mod helper;
mod init;
pub mod state;
pub mod utils;

use crate::lua::{PluginEngine, ThemeEngine};
use crate::setup;
use anyhow::Result;
use rustyline::Config as RlConfig;
use shellframe::Shell;
use state::LunaState;

pub struct Luna {
    pub shell: Shell<LunaState>,
    pub theme: Option<ThemeEngine>,
    pub plugins: PluginEngine,
}

impl Luna {
    pub fn new(shell: Shell<LunaState>, theme: Option<ThemeEngine>, plugins: PluginEngine) -> Self {
        Self {
            shell,
            theme,
            plugins,
        }
    }

    /// Initialize a new Luna instance with default configuration.
    pub fn init() -> Result<Self> {
        let config_file = setup::config_file();
        let config = config::LunaConfig::load(config_file);

        let cwd = std::env::current_dir()?.to_string_lossy().to_string();
        let mut ctx = shellframe::Context::new(
            cwd.clone(),
            indexmap::IndexMap::new(),
            state::LunaState::new(config.clone(), &cwd),
        );

        if config.should_inherit_system_env() {
            ctx.inherit_system_env();
        }

        let mut shell_inner = Shell::new(ctx);

        // Lua theme
        let mut theme_engine: Option<ThemeEngine> = None;
        if let Some(theme_path) = config.resolve_theme_path() {
            match ThemeEngine::new() {
                Ok(mut eng) => {
                    if let Err(e) = eng.load_file(&theme_path) {
                        eprintln!("luna: theme error: {e}");
                    } else {
                        theme_engine = Some(eng);
                    }
                }
                Err(e) => eprintln!("luna: theme VM error: {e}"),
            }
        }

        // Plugin engine
        let plugins_dir = setup::plugins_dir();
        let mut plugin_engine = PluginEngine::new();
        plugin_engine.load_dir(&plugins_dir);

        plugin_engine.drain_aliases_into(&mut shell_inner.context.state.aliases);

        if let Ok(exe) = std::env::current_exe() {
            shell_inner
                .context
                .state
                .aliases
                .insert("luna".to_string(), exe.to_string_lossy().to_string());
        }

        Ok(Self::new(shell_inner, theme_engine, plugin_engine))
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup internal shell handlers (hooks, redirects, etc.)
        handlers::setup_shell_handlers(&mut self.shell);

        // Load initialization scripts (.lunarc, .bashrc)
        self.load_rc();

        // Register all built-in commands
        let registry = crate::commands::register_all(&mut self.shell);
        self.shell.context.state.builtins = registry.commands.keys().cloned().collect();

        // Initial theme vars sync
        self.update_theme_vars();

        // Initialize Rustyline editor
        let mut rl = rustyline::Editor::<helper::LunaHelper, rustyline::history::DefaultHistory>::with_config(
            RlConfig::builder().auto_add_history(true).build(),
        )?;

        rl.set_helper(Some(helper::LunaHelper::new(
            registry,
            self.shell.context.state.config.clone(),
            self.shell.context.state.aliases.clone(),
        )));

        // Load history
        let hist_file = setup::history_file();
        let _ = rl.load_history(&hist_file);

        loop {
            // Check if theme needs reloading
            if self.shell.context.state.theme_dirty {
                let _ = self.reload_theme();
                self.shell.context.state.theme_dirty = false;
            }

            // Sync environment to plugins and fire prompt event
            let context = self.build_context_snapshot();
            self.plugins.sync_env_from(&self.shell.context.env);
            self.plugins.fire_prompt(&context);
            self.plugins.drain_env_into(&mut self.shell.context.env);

            // Re-build context after plugin modifications for the prompt
            let final_context = self.build_context_snapshot();
            let prompt = self.build_prompt(&final_context);

            // Sync last command to helper for !! expansion preview
            if let Some(h) = rl.helper_mut() {
                h.last_command = self.shell.context.state.last_command.clone();
                if let Ok((cols, rows)) = crossterm::terminal::size() {
                    if let Ok((_, cur_y)) = crossterm::cursor::position() {
                        // Calculate prompt lines (minimum lines + wrapped lines)
                        let raw_prompt = crate::renderer::markup::strip_ansi(&prompt);
                        let prompt_lines = raw_prompt.lines().count();
                        let prompt_wrap = (raw_prompt.len() as u16) / cols;
                        let total_prompt_h = (prompt_lines as u16 + prompt_wrap).saturating_sub(1);

                        let prompt_y = cur_y + total_prompt_h;

                        h.available_lines = if rows > prompt_y {
                            (rows - prompt_y) as usize
                        } else {
                            0
                        };
                    }
                }
            }

            match rl.readline(&prompt) {
                Ok(line) => {
                    use crossterm::cursor::MoveToColumn;
                    use crossterm::terminal::{Clear, ClearType};
                    use std::io::Write;
                    print!("{}{}", MoveToColumn(0), Clear(ClearType::FromCursorDown));
                    let _ = std::io::stdout().flush();

                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Add to history and save
                    let _ = rl.save_history(&hist_file);

                    // Execute the command line
                    self.execute_line(trimmed);
                }
                Err(_) => break, // Exit on EOF or Ctrl+C
            }
        }

        // Final history save
        let _ = rl.save_history(&hist_file);
        Ok(())
    }
}
