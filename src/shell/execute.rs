use crate::lua::ThemeEngine;
use crate::renderer;
use crate::renderer::markup;
use crate::shell::context::ShellContext;
use crate::shell::utils;
use crate::shell::Luna;
use std::time::Instant;

impl Luna {
    pub fn execute_line(&mut self, line: &str) {
        let expanded = self.expand_aliases(line);

        self.plugins.sync_env_from(&self.shell.context.env);
        self.plugins
            .sync_aliases_from(&self.shell.context.state.aliases);
        self.plugins.fire_pre_command(&expanded);
        self.plugins.drain_env_into(&mut self.shell.context.env);
        self.plugins
            .drain_aliases_into(&mut self.shell.context.state.aliases);

        let t0 = Instant::now();
        let exec_result = self.shell.execute(&expanded);
        let elapsed = t0.elapsed().as_millis();

        let old_cwd = self.shell.context.state.prev_cwd.clone();
        let new_cwd = self.shell.context.get_cwd().to_string();

        match exec_result {
            Ok(out) => {
                self.shell.context.state.last_exit_code = out.exit_code;
                self.shell.context.state.last_duration_ms = elapsed;

                // Trigger correction if command not found
                if out.exit_code == 127 {
                    if let Some(corrected) = self.try_correct_command(&expanded) {
                        return self.execute_line(&corrected);
                    }
                }

                renderer::print_stdout(&out.stdout);
                renderer::print_stderr(&out.stderr);
            }
            Err(e) => {
                let err_str = e.to_string();
                self.shell.context.state.last_exit_code = 1;
                self.shell.context.state.last_duration_ms = elapsed;

                let context_snap = self.build_context_snapshot();
                let err_msg = renderer::error::render_error(&self.theme, &context_snap, &err_str);
                eprint!("{err_msg}");
            }
        }

        let code = self.shell.context.state.last_exit_code;
        let ms = self.shell.context.state.last_duration_ms;
        self.plugins.sync_env_from(&self.shell.context.env);
        self.plugins
            .sync_aliases_from(&self.shell.context.state.aliases);
        self.plugins.fire_post_command(&expanded, code, ms);
        self.plugins.drain_env_into(&mut self.shell.context.env);
        self.plugins
            .drain_aliases_into(&mut self.shell.context.state.aliases);

        if old_cwd != new_cwd {
            // Update history
            {
                let state = &mut self.shell.context.state;
                let is_navigating = state.dir_history.get(state.dir_index) == Some(&new_cwd);

                if !is_navigating {
                    // Normal cd, truncate future history and append new path
                    state.dir_history.truncate(state.dir_index + 1);
                    state.dir_history.push(new_cwd.clone());
                    state.dir_index = state.dir_history.len() - 1;
                }
            }

            self.plugins.sync_env_from(&self.shell.context.env);
            self.plugins
                .sync_aliases_from(&self.shell.context.state.aliases);
            self.plugins.fire_dir_change(&old_cwd, &new_cwd);
            self.plugins.drain_env_into(&mut self.shell.context.env);
            self.plugins
                .drain_aliases_into(&mut self.shell.context.state.aliases);
            self.shell.context.state.prev_cwd = new_cwd;
        }

        if self.shell.context.state.config.should_add_newline() {
            println!();
        }
    }

    pub fn expand_aliases(&self, line: &str) -> String {
        utils::expand_aliases(line, &self.shell.context.state.aliases)
    }

    pub fn build_context_snapshot(&self) -> ShellContext {
        ShellContext::new(
            self.shell.context.state.last_exit_code,
            self.shell.context.state.last_duration_ms,
            self.shell.context.get_cwd(),
            &self.shell.context.env,
            &self.plugins.plugin_vars_snapshot(),
        )
    }

    pub fn build_prompt(&self, context: &ShellContext) -> String {
        renderer::prompt::render_prompt(&self.theme, context)
    }

    pub fn reload_theme(&mut self) -> anyhow::Result<()> {
        if let Some(path) = self.shell.context.state.config.resolve_theme_path() {
            let mut engine = ThemeEngine::new()?;
            engine.load_file(&path)?;
            self.theme = Some(engine);
            self.update_theme_vars();
        } else {
            self.theme = None;
        }
        Ok(())
    }

    pub fn update_theme_vars(&mut self) {
        if let Some(theme) = &self.theme {
            let vars = theme.extract_theme_vars();

            // Sync to plugins
            {
                let mut guard = self.plugins.shared.plugin_vars.lock().unwrap();
                for (k, v) in &vars {
                    guard.insert(k.clone(), v.clone());
                }
            }

            // Sync to shell state (for builtins)
            self.shell.context.state.theme_vars = vars.clone();

            // Sync to global text engine (for tags resolver)
            markup::update_theme_vars(vars);
        }
    }
}
