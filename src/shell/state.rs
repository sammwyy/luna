use crate::shell::config::LunaConfig;
use std::collections::HashMap;

pub struct LunaState {
    pub config: LunaConfig,
    pub aliases: HashMap<String, String>,

    /// Exit code reported by the last command (0 = success).
    pub last_exit_code: i32,
    /// Wall-clock time the last command took to run, in milliseconds.
    pub last_duration_ms: u128,

    /// The CWD as of the previous command cycle (used to detect directory changes).
    pub prev_cwd: String,

    /// Directory navigation history
    pub dir_history: Vec<String>,
    pub dir_index: usize,

    /// List of registered built-in command names.
    pub builtins: Vec<String>,

    /// Theme colors
    pub theme_vars: HashMap<String, String>,

    /// Signal to reload theme
    pub theme_dirty: bool,

    /// Syntax highlighting resources
    pub syntax_set: syntect::parsing::SyntaxSet,
    pub theme_set: syntect::highlighting::ThemeSet,
}

impl LunaState {
    pub fn new(config: LunaConfig, initial_cwd: &str) -> Self {
        Self {
            config,
            aliases: HashMap::new(),
            last_exit_code: 0,
            last_duration_ms: 0,
            prev_cwd: initial_cwd.to_string(),
            dir_history: vec![initial_cwd.to_string()],
            dir_index: 0,
            builtins: Vec::new(),
            theme_vars: HashMap::new(),
            theme_dirty: false,
            syntax_set: syntect::parsing::SyntaxSet::load_defaults_newlines(),
            theme_set: syntect::highlighting::ThemeSet::load_defaults(),
        }
    }
}
