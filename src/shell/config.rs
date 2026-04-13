use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::setup::themes_dir;

// Features config
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct CorrectorConfig {
    pub enabled: Option<bool>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub builtins: Option<bool>,
    pub system: Option<bool>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct LinterErrorsConfig {
    pub enabled: Option<bool>,
    pub layout: Option<String>,
    pub commands: Option<bool>,
    pub flags: Option<bool>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct LinterCommandsConfig {
    pub enabled: Option<bool>,
    pub commands: Option<bool>,
    pub flags: Option<bool>,
    pub strings: Option<bool>,
    pub number: Option<bool>,
    pub boolean: Option<bool>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct LinterConfig {
    pub errors: Option<LinterErrorsConfig>,
    pub commands: Option<LinterCommandsConfig>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SuggestionsConfig {
    pub enabled: Option<bool>,
    pub commands: Option<bool>,
    pub system: Option<bool>,

    pub short_flags: Option<bool>,
    pub long_flags: Option<bool>,
    pub max_items: Option<usize>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct TabCompleteConfig {
    pub enabled: Option<bool>,
    pub files: Option<bool>,
    pub commands: Option<bool>,
    pub flags: Option<bool>,
}

// Commands config
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct CatConfig {
    pub highlight: Option<bool>,
    pub highlight_exts: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct LsConfig {
    pub render_table: Option<bool>,
    pub alternating_rows: Option<bool>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct CdConfig {
    pub home_default: Option<bool>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct HeadConfig {
    pub lines: Option<usize>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct TailConfig {
    pub lines: Option<usize>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct BuiltinConfig {
    pub enabled: Option<bool>,
    pub blocked: Option<Vec<String>>,
    pub allowed: Option<Vec<String>>,

    pub cat: Option<CatConfig>,
    pub ls: Option<LsConfig>,
    pub cd: Option<CdConfig>,
    pub head: Option<HeadConfig>,
    pub tail: Option<TailConfig>,
}

// Main config
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct LunaConfig {
    pub inherit_system_env: Option<bool>,
    pub run_bashrc: Option<bool>,
    pub run_lunarc: Option<bool>,

    pub theme: Option<String>,
    pub newline: Option<bool>,
    pub universal_multi_file_parsing: Option<bool>,

    pub corrector: Option<CorrectorConfig>,
    pub linter: Option<LinterConfig>,
    pub tabcomplete: Option<TabCompleteConfig>,
    pub suggestions: Option<SuggestionsConfig>,
    pub builtin: Option<BuiltinConfig>,
}

impl LunaConfig {
    pub fn load(config_path: PathBuf) -> Self {
        if let Ok(content) = fs::read_to_string(config_path) {
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self, config_path: &PathBuf) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    pub fn should_add_newline(&self) -> bool {
        self.newline.unwrap_or(true)
    }

    pub fn should_inherit_system_env(&self) -> bool {
        self.inherit_system_env.unwrap_or(true)
    }

    pub fn should_run_bashrc(&self) -> bool {
        self.run_bashrc.unwrap_or(false)
    }

    pub fn should_run_lunarc(&self) -> bool {
        self.run_lunarc.unwrap_or(true)
    }

    pub fn is_builtin_enabled(&self, name: &str) -> bool {
        if let Some(config) = &self.builtin {
            if !config.enabled.unwrap_or(true) {
                return false;
            }
            if let Some(blocked) = &config.blocked {
                if blocked.contains(&name.to_string()) {
                    return false;
                }
            }
            if let Some(allowed) = &config.allowed {
                if !allowed.is_empty() && !allowed.contains(&name.to_string()) {
                    return false;
                }
            }
        }
        true
    }

    pub fn universal_multi_file_parsing(&self) -> bool {
        self.universal_multi_file_parsing.unwrap_or(false)
    }

    // --- Accessors for corrector ---

    pub fn corrector_enabled(&self) -> bool {
        self.corrector
            .as_ref()
            .and_then(|c| c.enabled)
            .unwrap_or(true)
    }

    pub fn corrector_min_length(&self) -> usize {
        self.corrector
            .as_ref()
            .and_then(|c| c.min_length)
            .unwrap_or(0)
    }

    pub fn corrector_max_length(&self) -> usize {
        self.corrector
            .as_ref()
            .and_then(|c| c.max_length)
            .unwrap_or(999)
    }

    pub fn corrector_builtins(&self) -> bool {
        self.corrector
            .as_ref()
            .and_then(|c| c.builtins)
            .unwrap_or(true)
    }

    pub fn corrector_system(&self) -> bool {
        self.corrector
            .as_ref()
            .and_then(|c| c.system)
            .unwrap_or(true)
    }

    // --- Accessors for linter ---

    pub fn linter_errors_enabled(&self) -> bool {
        self.linter
            .as_ref()
            .and_then(|l| l.errors.as_ref())
            .and_then(|e| e.enabled)
            .unwrap_or(true)
    }

    pub fn linter_errors_layout(&self) -> String {
        self.linter
            .as_ref()
            .and_then(|l| l.errors.as_ref())
            .and_then(|e| e.layout.clone())
            .unwrap_or("right".to_string())
    }

    pub fn linter_errors_commands(&self) -> bool {
        self.linter
            .as_ref()
            .and_then(|l| l.errors.as_ref())
            .and_then(|e| e.commands)
            .unwrap_or(true)
    }

    pub fn linter_errors_flags(&self) -> bool {
        self.linter
            .as_ref()
            .and_then(|l| l.errors.as_ref())
            .and_then(|e| e.flags)
            .unwrap_or(true)
    }

    pub fn linter_commands_enabled(&self) -> bool {
        self.linter
            .as_ref()
            .and_then(|l| l.commands.as_ref())
            .and_then(|c| c.enabled)
            .unwrap_or(true)
    }

    pub fn linter_commands_commands(&self) -> bool {
        self.linter
            .as_ref()
            .and_then(|l| l.commands.as_ref())
            .and_then(|c| c.commands)
            .unwrap_or(true)
    }

    pub fn linter_commands_flags(&self) -> bool {
        self.linter
            .as_ref()
            .and_then(|l| l.commands.as_ref())
            .and_then(|c| c.flags)
            .unwrap_or(true)
    }

    pub fn linter_commands_strings(&self) -> bool {
        self.linter
            .as_ref()
            .and_then(|l| l.commands.as_ref())
            .and_then(|c| c.strings)
            .unwrap_or(true)
    }

    pub fn linter_commands_number(&self) -> bool {
        self.linter
            .as_ref()
            .and_then(|l| l.commands.as_ref())
            .and_then(|c| c.number)
            .unwrap_or(true)
    }

    pub fn linter_commands_boolean(&self) -> bool {
        self.linter
            .as_ref()
            .and_then(|l| l.commands.as_ref())
            .and_then(|c| c.boolean)
            .unwrap_or(true)
    }

    // --- Accessors for tabcomplete ---

    pub fn tabcomplete_enabled(&self) -> bool {
        self.tabcomplete
            .as_ref()
            .and_then(|t| t.enabled)
            .unwrap_or(true)
    }

    pub fn tabcomplete_files(&self) -> bool {
        self.tabcomplete
            .as_ref()
            .and_then(|t| t.files)
            .unwrap_or(true)
    }

    pub fn tabcomplete_commands(&self) -> bool {
        self.tabcomplete
            .as_ref()
            .and_then(|t| t.commands)
            .unwrap_or(true)
    }

    // --- Accessors for suggestions ---

    pub fn suggestions_enabled(&self) -> bool {
        self.suggestions
            .as_ref()
            .and_then(|s| s.enabled)
            .unwrap_or(true)
    }

    pub fn suggestions_commands(&self) -> bool {
        self.suggestions
            .as_ref()
            .and_then(|s| s.commands)
            .unwrap_or(true)
    }

    pub fn suggestions_system(&self) -> bool {
        self.suggestions
            .as_ref()
            .and_then(|s| s.system)
            .unwrap_or(true)
    }

    pub fn suggestions_short_flags(&self) -> bool {
        self.suggestions
            .as_ref()
            .and_then(|s| s.short_flags)
            .unwrap_or(true)
    }

    pub fn suggestions_long_flags(&self) -> bool {
        self.suggestions
            .as_ref()
            .and_then(|s| s.long_flags)
            .unwrap_or(true)
    }

    pub fn suggestions_max_items(&self) -> usize {
        self.suggestions
            .as_ref()
            .and_then(|s| s.max_items)
            .unwrap_or(4)
    }

    pub fn tabcomplete_flags(&self) -> bool {
        self.tabcomplete
            .as_ref()
            .and_then(|t| t.flags)
            .unwrap_or(true)
    }

    // --- Accessors for builtins ---

    pub fn cat_highlight(&self) -> bool {
        self.builtin
            .as_ref()
            .and_then(|b| b.cat.as_ref())
            .and_then(|c| c.highlight)
            .unwrap_or(true)
    }

    pub fn cat_highlight_exts(&self) -> Vec<String> {
        self.builtin
            .as_ref()
            .and_then(|b| b.cat.as_ref())
            .and_then(|c| c.highlight_exts.clone())
            .unwrap_or_else(|| {
                vec![
                    "rs", "py", "js", "ts", "lua", "toml", "c", "cpp", "h", "hpp",
                ]
                .into_iter()
                .map(|s| s.to_string())
                .collect()
            })
    }

    pub fn ls_render_table(&self) -> bool {
        self.builtin
            .as_ref()
            .and_then(|b| b.ls.as_ref())
            .and_then(|l| l.render_table)
            .unwrap_or(true)
    }

    pub fn ls_alternating_rows(&self) -> bool {
        self.builtin
            .as_ref()
            .and_then(|b| b.ls.as_ref())
            .and_then(|l| l.alternating_rows)
            .unwrap_or(true)
    }

    pub fn cd_home_default(&self) -> bool {
        self.builtin
            .as_ref()
            .and_then(|b| b.cd.as_ref())
            .and_then(|c| c.home_default)
            .unwrap_or(true)
    }

    pub fn head_lines(&self) -> usize {
        self.builtin
            .as_ref()
            .and_then(|b| b.head.as_ref())
            .and_then(|h| h.lines)
            .unwrap_or(10)
    }

    pub fn tail_lines(&self) -> usize {
        self.builtin
            .as_ref()
            .and_then(|b| b.tail.as_ref())
            .and_then(|t| t.lines)
            .unwrap_or(10)
    }

    pub fn resolve_theme_path(&self) -> Option<PathBuf> {
        let theme = self.theme.as_deref()?;
        let path = if Path::new(theme).is_absolute() {
            PathBuf::from(theme)
        } else {
            themes_dir().join(theme)
        };

        if path.exists() {
            return Some(path);
        }

        if !theme.ends_with(".lua") {
            let lua_path = path.with_extension("lua");
            if lua_path.exists() {
                return Some(lua_path);
            }
        }

        Some(path)
    }
}
