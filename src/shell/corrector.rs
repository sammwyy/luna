use crate::renderer::menu::{show_correction_menu, MenuOption};
use crate::shell::utils::suggest_commands;
use crate::shell::Luna;

impl Luna {
    pub fn try_correct_command(&mut self, line: &str) -> Option<String> {
        let config = &self.shell.context.state.config;

        // Check if enabled
        if !config.corrector_enabled() {
            return None;
        }

        // Operators we care about
        let operators = ["|", "&&", "||", ";"];

        // Simple tokenization by operators
        let mut segments = Vec::new();
        let mut current = String::new();

        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let mut found_op = None;
            for op in operators {
                if line[i..].starts_with(op) {
                    found_op = Some(op);
                    break;
                }
            }

            if let Some(op) = found_op {
                if !current.trim().is_empty() {
                    segments.push(current.trim().to_string());
                }
                segments.push(op.to_string());
                current = String::new();
                i += op.len();
            } else {
                current.push(chars[i]);
                i += 1;
            }
        }
        if !current.trim().is_empty() {
            segments.push(current.trim().to_string());
        }

        // Identify which segments are commands and which are wrong
        let builtins = self.shell.context.state.builtins.clone();
        let aliases: Vec<String> = self.shell.context.state.aliases.keys().cloned().collect();

        let mut errors = Vec::new(); // (segment_index, suggestions)

        for (idx, seg) in segments.iter().enumerate() {
            if operators.contains(&seg.as_str()) {
                continue;
            }

            let parts: Vec<&str> = seg.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            let cmd_name = parts[0];

            if !self.cmd_exists(cmd_name) {
                // Check length limits
                let len = cmd_name.len();
                if len < config.corrector_min_length() || len > config.corrector_max_length() {
                    continue;
                }

                let suggestions = suggest_commands(
                    cmd_name,
                    &builtins,
                    &aliases,
                    config.corrector_builtins(),
                    config.corrector_system(),
                );

                if !suggestions.is_empty() {
                    errors.push((idx, suggestions));
                }
            }
        }

        if errors.is_empty() {
            return None;
        }

        // Number of operators/commands
        let num_commands = segments
            .iter()
            .filter(|s| !operators.contains(&s.as_str()))
            .count();
        let num_ops = segments.len() - num_commands;

        // Character limit per segment based on user rules
        let char_limit = if num_ops == 0 {
            32
        } else if num_ops == 1 {
            16
        } else if num_ops < 5 {
            8
        } else {
            0
        };

        let mut options = Vec::new();
        options.push(MenuOption {
            label: "Cancel".to_string(),
            command: None,
        });

        // Focus on the first error found
        let (err_idx, suggestions) = &errors[0];

        for suggested in suggestions {
            let mut label = String::new();
            let mut full_cmd_line = String::new();

            for (idx, seg) in segments.iter().enumerate() {
                if idx > 0 {
                    label.push_str(" ");
                    full_cmd_line.push_str(" ");
                }

                if operators.contains(&seg.as_str()) {
                    label.push_str(&format!("<#E5E510>{}</#E5E510>", seg));
                    full_cmd_line.push_str(seg);
                } else if idx == *err_idx {
                    // This is the segment we are correcting
                    let parts: Vec<&str> = seg.split_whitespace().collect();
                    let args = if parts.len() > 1 {
                        &seg[parts[0].len()..].trim()
                    } else {
                        ""
                    };

                    let limit = char_limit * 2;
                    let label_args = if args.len() > limit && limit > 0 {
                        format!("{}...", &args[..limit])
                    } else if limit == 0 {
                        "".to_string()
                    } else {
                        args.to_string()
                    };

                    label.push_str(&format!(
                        "<color_primary>{}</color_primary> <#888888>{}</#888888>",
                        suggested, label_args
                    ));

                    if args.is_empty() {
                        full_cmd_line.push_str(suggested);
                    } else {
                        full_cmd_line.push_str(&format!("{} {}", suggested, args));
                    }
                } else {
                    // Other segments
                    let parts: Vec<&str> = seg.split_whitespace().collect();
                    let cmd = parts[0];
                    let args = if parts.len() > 1 {
                        &seg[cmd.len()..].trim()
                    } else {
                        ""
                    };

                    let label_args = if args.len() > char_limit && char_limit > 0 {
                        format!("{}...", &args[..char_limit])
                    } else if char_limit == 0 {
                        "".to_string()
                    } else {
                        args.to_string()
                    };

                    label.push_str(&format!(
                        "<color_secondary>{}</color_secondary> <#888888>{}</#888888>",
                        cmd, label_args
                    ));
                    full_cmd_line.push_str(seg);
                }
            }

            options.push(MenuOption {
                label,
                command: Some(full_cmd_line),
            });
        }

        match show_correction_menu(options) {
            Ok(Some(corrected)) => Some(corrected),
            _ => None,
        }
    }

    fn cmd_exists(&self, name: &str) -> bool {
        if self
            .shell
            .context
            .state
            .builtins
            .contains(&name.to_string())
        {
            return true;
        }
        if self.shell.context.state.aliases.contains_key(name) {
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
