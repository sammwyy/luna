use crate::commands::system::{FlagType, Registry};
use crate::renderer::highlight::SyntaxHighlighter;
use crate::shell::config::LunaConfig;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Context;
use rustyline::Helper;
use std::collections::HashMap;
use std::sync::Arc;

pub struct LunaHelper {
    pub registry: Arc<Registry>,
    pub file_completer: FilenameCompleter,
    pub config: LunaConfig,
    pub aliases: HashMap<String, String>,
    pub highlighter: SyntaxHighlighter,
    pub last_command: String,
    pub available_lines: usize,
}

impl LunaHelper {
    pub fn new(
        registry: Arc<Registry>,
        config: LunaConfig,
        aliases: HashMap<String, String>,
    ) -> Self {
        let highlighter = SyntaxHighlighter::new(registry.clone(), config.clone(), aliases.clone());
        Self {
            registry,
            file_completer: FilenameCompleter::new(),
            config,
            aliases,
            highlighter,
            last_command: String::new(),
            available_lines: 100, // Def safe value
        }
    }

    fn cmd_exists(&self, name: &str) -> bool {
        if self.registry.commands.contains_key(name) {
            return true;
        }
        if self.aliases.contains_key(name) {
            return true;
        }
        if name.starts_with("./") || name.starts_with('/') {
            return std::path::Path::new(name).exists();
        }
        if which::which(name).is_ok() {
            return true;
        }
        false
    }

    fn get_suggestions(&self, line: &str) -> Vec<String> {
        let words: Vec<&str> = line.split_whitespace().collect();
        let is_completing_word = !line.is_empty() && !line.ends_with(char::is_whitespace);

        let mut suggestions = Vec::new();

        if (words.len() <= 1 && is_completing_word) || words.is_empty() {
            if self.config.suggestions_commands() {
                let search = words.first().unwrap_or(&"");
                // Builtins
                for name in self.registry.commands.keys() {
                    if name.starts_with(search) {
                        suggestions.push(name.clone());
                    }
                }
                // Aliases
                for name in self.aliases.keys() {
                    if name.starts_with(search) {
                        suggestions.push(name.clone());
                    }
                }
            }
            if self.config.suggestions_system() {
                let search = words.first().unwrap_or(&"");
                if let Ok(path_env) = std::env::var("PATH") {
                    for dir in path_env.split(':') {
                        if let Ok(entries) = std::fs::read_dir(dir) {
                            for entry in entries.flatten() {
                                if let Ok(name) = entry.file_name().into_string() {
                                    if name.starts_with(search) {
                                        suggestions.push(name);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            suggestions.sort();
            suggestions.dedup();
            suggestions = suggestions
                .into_iter()
                .take(self.config.suggestions_max_items())
                .collect();
        } else if !words.is_empty() {
            // Flags or subcommands
            let cmd_name = words[0];
            if let Some(cmd) = self.registry.commands.get(cmd_name) {
                let last_word = if is_completing_word {
                    words.last().unwrap_or(&"")
                } else {
                    ""
                };

                if last_word.starts_with("--") {
                    if self.config.suggestions_long_flags() {
                        let search = &last_word[2..];
                        for flag in cmd.flags() {
                            if flag.name.starts_with(search) {
                                suggestions.push(format!("--{}", flag.name));
                            }
                        }
                    }
                } else if last_word.starts_with('-') {
                    if self.config.suggestions_short_flags() {
                        let search = &last_word[1..];
                        for flag in cmd.flags() {
                            if let Some(s) = flag.short {
                                if search.is_empty() || s.to_string().starts_with(search) {
                                    suggestions.push(format!("-{}", s));
                                }
                            }
                        }
                    }
                }
            }

            suggestions.sort();
            suggestions.dedup();
            // Flag suggestions usually don't need a small limit but we can take 4 or 6.
            suggestions = suggestions.into_iter().take(6).collect();
        }

        suggestions
    }

    fn get_overlay_components(
        &self,
        line: &str,
    ) -> Vec<Box<dyn crate::renderer::overlay::OverlayComponent>> {
        use crate::renderer::overlay::{SuggestionBox, Tip};
        let mut components: Vec<Box<dyn crate::renderer::overlay::OverlayComponent>> = Vec::new();

        // 1. !! expansion tip
        if line.contains("!!") {
            if self.last_command.is_empty() {
                components.push(Box::new(Tip {
                    text: "no previous command".into(),
                    color_tag: "color_error".into(),
                }));
            } else {
                let expanded = line.replace("!!", &self.last_command);
                components.push(Box::new(Tip {
                    text: format!("expands to: {}", expanded),
                    color_tag: "color_secondary".into(),
                }));
            }
        }

        // 2. Suggestion Box
        if self.config.suggestions_enabled() {
            let free_lines = self.available_lines.saturating_sub(1);
            if free_lines >= 3 {
                let mut suggestions = self.get_suggestions(line);
                let max_items = free_lines - 2;
                if suggestions.len() > max_items {
                    suggestions.truncate(max_items);
                }

                if !suggestions.is_empty() {
                    components.push(Box::new(SuggestionBox { items: suggestions }));
                }
            }
        }

        // 3. Command/Flag Linter Tips
        let operators = ["&&", "||", "|", ";", ">>", ">", "<"];
        let mut last_cmd_start = 0;
        for op in &operators {
            if let Some(pos) = line.rfind(op) {
                if pos + op.len() > last_cmd_start {
                    last_cmd_start = pos + op.len();
                }
            }
        }

        let current_segment = &line[last_cmd_start..].trim_start();
        if !current_segment.is_empty() {
            let words: Vec<String> = current_segment
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            if !words.is_empty() {
                let cmd_name = &words[0];

                // Check if command exists
                if self.config.linter_errors_commands() && !self.cmd_exists(cmd_name) {
                    let builtins: Vec<String> = self.registry.commands.keys().cloned().collect();
                    let aliases: Vec<String> = self.aliases.keys().cloned().collect();

                    if let Some(suggested) = crate::shell::utils::suggest_commands(
                        cmd_name,
                        &builtins,
                        &aliases,
                        self.config.corrector_builtins(),
                        self.config.corrector_system(),
                    )
                    .first()
                    {
                        components.push(Box::new(Tip {
                            text: format!("did you mean: {}", suggested),
                            color_tag: "color_warn".into(),
                        }));
                    } else {
                        components.push(Box::new(Tip {
                            text: "command not found".into(),
                            color_tag: "color_error".into(),
                        }));
                    }
                } else if self.config.linter_errors_flags() {
                    // Check flags via dry_run
                    if let Some(cmd) = self.registry.commands.get(cmd_name) {
                        let args_slice = &words[1..];
                        match cmd.parse_args(args_slice) {
                            Ok(parsed) => {
                                if let Err(e) = cmd.dry_run(&self.config, &parsed) {
                                    components.push(Box::new(Tip {
                                        text: e,
                                        color_tag: "color_error".into(),
                                    }));
                                }
                            }
                            Err(e) => {
                                components.push(Box::new(Tip {
                                    text: e,
                                    color_tag: "color_error".into(),
                                }));
                            }
                        }
                    }
                }
            }
        }

        components
    }
}

impl Helper for LunaHelper {}

impl Completer for LunaHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        if !self.config.tabcomplete_enabled() {
            return Ok((0, Vec::new()));
        }

        let (words, last_word) = if line[..pos].ends_with(char::is_whitespace) {
            let words: Vec<&str> = line[..pos].split_whitespace().collect();
            (words, "")
        } else {
            let mut words: Vec<&str> = line[..pos].split_whitespace().collect();
            let last = words.pop().unwrap_or("");
            (words, last)
        };

        let mut candidates = Vec::new();
        let starts_path = last_word.starts_with("./")
            || last_word.starts_with('/')
            || last_word.starts_with("../");

        // 1. Command completion
        let is_command_start = words.is_empty() || {
            let last_before = words.last().cloned().unwrap_or("");
            last_before == "|" || last_before == "&&" || last_before == "||" || last_before == ";"
        };

        if self.config.tabcomplete_commands()
            && is_command_start
            && !starts_path
            && !last_word.is_empty()
        {
            // Builtins
            for cmd_name in self.registry.commands.keys() {
                if cmd_name.starts_with(last_word) {
                    candidates.push(Pair {
                        display: cmd_name.to_string(),
                        replacement: cmd_name.to_string(),
                    });
                }
            }

            // Executables in PATH
            if let Ok(path_env) = std::env::var("PATH") {
                for dir in path_env.split(':') {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.flatten() {
                            if let Ok(name) = entry.file_name().into_string() {
                                if name.starts_with(last_word) {
                                    candidates.push(Pair {
                                        display: name.clone(),
                                        replacement: name,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Flag completion
        if self.config.tabcomplete_flags() && !words.is_empty() && last_word.starts_with('-') {
            let mut current_cmd = None;
            for w in words.iter().rev() {
                if self.cmd_exists(w) && !w.starts_with('-') {
                    current_cmd = Some(*w);
                    break;
                }
            }

            if let Some(cmd_name) = current_cmd {
                if let Some(cmd) = self.registry.commands.get(cmd_name) {
                    let prefix = if last_word.starts_with("--") {
                        &last_word[2..]
                    } else {
                        &last_word[1..]
                    };
                    for flag in &cmd.flags() {
                        if flag.name.starts_with(prefix) {
                            let replacement = format!("--{} ", flag.name);
                            let mut desc = flag.desc.to_string();
                            if let FlagType::Enum(opts) = &flag.flag_type {
                                desc.push_str(&format!(" {:?}", opts));
                            }
                            candidates.push(Pair {
                                display: format!("--{} ({})", flag.name, desc),
                                replacement,
                            });
                        }
                    }
                }
            }
        }

        if !candidates.is_empty() {
            candidates.sort_by(|a, b| a.display.cmp(&b.display));
            candidates.dedup_by(|a, b| a.display == b.display);
            return Ok((pos - last_word.len(), candidates));
        }

        // 3. Fallback to file completion
        if self.config.tabcomplete_files() {
            self.file_completer.complete(line, pos, ctx)
        } else {
            Ok((0, Vec::new()))
        }
    }
}

impl Hinter for LunaHelper {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        let mut manager = crate::renderer::overlay::OverlayManager::new();

        if self.config.linter_errors_enabled() && !line.is_empty() {
            let comps = self.get_overlay_components(line);
            for comp in comps {
                manager.add(comp);
            }
        }

        Some(manager.render_all(line))
    }
}

use std::borrow::Cow;
impl Highlighter for LunaHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let out = self.highlighter.highlight(line);
        Cow::Owned(out)
    }
    fn highlight_char(&self, _line: &str, _pos: usize, _final: bool) -> bool {
        true
    }
}
impl Validator for LunaHelper {
    fn validate(&self, _ctx: &mut ValidationContext<'_>) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}
