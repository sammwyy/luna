use crate::commands::system::Registry;
use crate::shell::config::LunaConfig;
use std::collections::HashMap;
use std::sync::Arc;

pub struct SyntaxHighlighter {
    pub registry: Arc<Registry>,
    pub config: LunaConfig,
    pub aliases: HashMap<String, String>,
}

impl SyntaxHighlighter {
    pub fn new(
        registry: Arc<Registry>,
        config: LunaConfig,
        aliases: HashMap<String, String>,
    ) -> Self {
        Self {
            registry,
            config,
            aliases,
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

    pub fn highlight(&self, line: &str) -> String {
        if line.is_empty() {
            return line.to_string();
        }

        if !self.config.linter_commands_enabled() {
            return line.to_string();
        }

        let mut out = String::new();
        let mut i = 0;
        let chars: Vec<char> = line.chars().collect();
        let mut first_word = true;

        // ANSI Color codes
        const C_CMD: &str = "\x1b[1;36m";
        const C_FLAG: &str = "\x1b[38;5;208m";
        const C_STR: &str = "\x1b[32m";
        const C_NUM: &str = "\x1b[33m";
        const C_BOOL: &str = "\x1b[35m";
        const C_OP: &str = "\x1b[1;33m";
        const C_ERR: &str = "\x1b[31m";
        const C_RESET: &str = "\x1b[0m";

        let operators = ["&&", "||", "|", ";", ">>", ">", "<"];

        while i < chars.len() {
            if chars[i].is_whitespace() {
                out.push(chars[i]);
                i += 1;
                continue;
            }

            let mut found_op = None;
            for op in &operators {
                if line[i..].starts_with(op) {
                    found_op = Some(op);
                    break;
                }
            }

            if let Some(op) = found_op {
                out.push_str(C_OP);
                out.push_str(op);
                out.push_str(C_RESET);
                i += op.len();
                first_word = true;
                continue;
            }

            if self.config.linter_commands_strings() && (chars[i] == '"' || chars[i] == '\'') {
                let quote = chars[i];
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != quote {
                    i += 1;
                }
                if i < chars.len() {
                    out.push_str(C_STR);
                    for j in start..=i {
                        out.push(chars[j]);
                    }
                    out.push_str(C_RESET);
                    i += 1;
                } else {
                    out.push_str(C_ERR);
                    for j in start..chars.len() {
                        out.push(chars[j]);
                    }
                    out.push_str(C_RESET);
                    i = chars.len();
                }
                first_word = false;
                continue;
            }

            let word_start = i;
            while i < chars.len()
                && !chars[i].is_whitespace()
                && !operators.iter().any(|op| line[i..].starts_with(op))
                && chars[i] != '"'
                && chars[i] != '\''
            {
                i += 1;
            }
            let word: String = chars[word_start..i].iter().collect();

            if first_word {
                if self.config.linter_commands_commands() {
                    if self.cmd_exists(&word) || word.is_empty() {
                        out.push_str(C_CMD);
                    } else {
                        out.push_str(C_ERR);
                    }
                }
                out.push_str(&word);
                out.push_str(C_RESET);
                first_word = false;
            } else if self.config.linter_commands_flags() && word.starts_with('-') {
                let line_upto = &line[..i];
                let line_words: Vec<&str> = line_upto.split_whitespace().collect();
                let mut cmd_name = None;
                for w in line_words.iter().rev() {
                    if self.cmd_exists(w) && !w.starts_with('-') {
                        cmd_name = Some(*w);
                        break;
                    }
                }

                let mut invalid = false;
                if let Some(name) = cmd_name {
                    if let Some(cmd) = self.registry.commands.get(name) {
                        let flags = cmd.flags();
                        let flag_stripped = if word.starts_with("--") {
                            &word[2..]
                        } else {
                            &word[1..]
                        };
                        if word.starts_with("--") {
                            if !flags.iter().any(|f| f.name == flag_stripped) {
                                invalid = true;
                            }
                        } else {
                            for c in flag_stripped.chars() {
                                if !flags.iter().any(|f| f.short == Some(c)) {
                                    invalid = true;
                                    break;
                                }
                            }
                        }
                    }
                }
                if invalid {
                    out.push_str(C_ERR);
                } else {
                    out.push_str(C_FLAG);
                }
                out.push_str(&word);
                out.push_str(C_RESET);
            } else if self.config.linter_commands_boolean() && (word == "true" || word == "false") {
                out.push_str(C_BOOL);
                out.push_str(&word);
                out.push_str(C_RESET);
            } else if self.config.linter_commands_number()
                && word.chars().all(|c| c.is_ascii_digit() || c == '.')
            {
                out.push_str(C_NUM);
                out.push_str(&word);
                out.push_str(C_RESET);
            } else {
                out.push_str(&word);
            }
        }
        out
    }
}
