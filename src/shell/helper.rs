use crate::commands::system::{FlagType, Registry};
use crate::renderer::highlight::SyntaxHighlighter;
use crate::shell::config::LunaConfig;
use crate::shell::utils::suggest_commands;
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
        if !self.config.linter_errors_enabled() || line.trim().is_empty() {
            return None;
        }

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
        if current_segment.is_empty() {
            return None;
        }

        let words: Vec<String> = current_segment
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        if words.is_empty() {
            return None;
        }

        let cmd_name = &words[0];
        let layout = self.config.linter_errors_layout();
        let prefix = if layout == "down" { "\r\n" } else { "   " };

        // 1. Check if command exists
        if self.config.linter_errors_commands() && !self.cmd_exists(cmd_name) {
            let builtins: Vec<String> = self.registry.commands.keys().cloned().collect();
            let aliases: Vec<String> = self.aliases.keys().cloned().collect();

            if let Some(suggested) = suggest_commands(
                cmd_name,
                &builtins,
                &aliases,
                self.config.corrector_builtins(),
                self.config.corrector_system(),
            )
            .first()
            {
                return Some(format!(
                    "{}\x1b[2;33m(did you mean: {})\x1b[0m",
                    prefix, suggested
                ));
            }
            return Some(format!("{}\x1b[2;31m(command not found)\x1b[0m", prefix));
        }

        // 2. If it's a builtin, run dry_run
        if self.config.linter_errors_flags() {
            if let Some(cmd) = self.registry.commands.get(cmd_name) {
                let args_slice = &words[1..];
                match cmd.parse_args(args_slice) {
                    Ok(parsed) => {
                        if let Err(e) = cmd.dry_run(&self.config, &parsed) {
                            return Some(format!("{}\x1b[2;31m({})\x1b[0m", prefix, e));
                        }
                    }
                    Err(e) => {
                        return Some(format!("{}\x1b[2;31m({})\x1b[0m", prefix, e));
                    }
                }
            }
        }

        None
    }
}

use std::borrow::Cow;
impl Highlighter for LunaHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Owned(self.highlighter.highlight(line))
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
