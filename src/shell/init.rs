use crate::setup;
use crate::shell::Luna;
use std::env;
use std::fs;
use std::path::PathBuf;

impl Luna {
    pub fn load_rc(&mut self) {
        let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
        let bashrc = PathBuf::from(&home).join(".bashrc");
        let lunarc = PathBuf::from(&home).join(".lunarc");
        let lunarc_dir = setup::luna_dir().join(".lunarc");

        let config = self.shell.context.state.config.clone();

        if config.should_run_bashrc() && bashrc.exists() {
            if let Ok(content) = fs::read_to_string(&bashrc) {
                self.eval_rc(content);
            }
        }

        if config.should_run_lunarc() {
            if lunarc.exists() {
                if let Ok(content) = fs::read_to_string(&lunarc) {
                    self.eval_rc(content);
                }
            } else if lunarc_dir.exists() {
                if let Ok(content) = fs::read_to_string(&lunarc_dir) {
                    self.eval_rc(content);
                }
            }
        }
    }

    pub fn eval_rc(&mut self, content: String) {
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                if trimmed.starts_with("alias ") {
                    if let Some(eq) = trimmed.find('=') {
                        let name = trimmed[6..eq].trim().to_string();
                        let mut val = trimmed[eq + 1..].trim().to_string();
                        if val.starts_with('"') && val.ends_with('"') {
                            val = val[1..val.len() - 1].to_string();
                        } else if val.starts_with('\'') && val.ends_with('\'') {
                            val = val[1..val.len() - 1].to_string();
                        }
                        self.shell.context.state.aliases.insert(name, val);
                        continue;
                    }
                }
                let _ = self.shell.execute(trimmed);
            }
        }
    }
}
