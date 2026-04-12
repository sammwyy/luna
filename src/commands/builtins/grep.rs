use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;

pub struct GrepCommand;

impl BuiltinCommand for GrepCommand {
    fn name(&self) -> &'static str {
        "grep"
    }
    fn desc(&self) -> &'static str {
        "Print lines that match patterns"
    }
    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "ignore-case",
                short: Some('i'),
                desc: "ignore case distinctions",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "invert-match",
                short: Some('v'),
                desc: "select non-matching lines",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "line-number",
                short: Some('n'),
                desc: "prefix each line of output with the 1-based line number",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "count",
                short: Some('c'),
                desc: "print only a count of matching lines per file",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "files-with-matches",
                short: Some('l'),
                desc: "print only names of files with matches",
                flag_type: FlagType::Bool,
                required: false,
            },
        ]
    }
    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        stdin: &str,
    ) -> anyhow::Result<Output> {
        if args.positionals.is_empty() {
            return Ok(Output::error(
                1,
                "".into(),
                "grep: missing pattern\n".into(),
            ));
        }

        let pattern = args.positionals[0].clone();
        let ignore_case = args.get_bool("ignore-case");
        let invert_match = args.get_bool("invert-match");
        let line_number = args.get_bool("line-number");
        let count_only = args.get_bool("count");
        let list_files = args.get_bool("files-with-matches");

        let search_str = if ignore_case {
            pattern.to_lowercase()
        } else {
            pattern
        };

        let files = if args.positionals.len() < 2 {
            vec!["(stdin)".to_string()]
        } else {
            args.positionals[1..].to_vec()
        };

        let mut out = String::new();
        let multiple_files = files.len() > 1;

        for file in files {
            let content = if file == "(stdin)" {
                stdin.to_string()
            } else {
                match fs::read_to_string(&file) {
                    Ok(c) => c,
                    Err(_) => continue,
                }
            };

            let mut match_count = 0;
            let mut file_has_match = false;

            for (idx, line) in content.lines().enumerate() {
                let target = if ignore_case {
                    line.to_lowercase()
                } else {
                    line.to_string()
                };
                let matched = target.contains(&search_str);
                if matched ^ invert_match {
                    match_count += 1;
                    file_has_match = true;

                    if !count_only && !list_files {
                        if multiple_files && file != "(stdin)" {
                            out.push_str(&format!("{}:", file));
                        }
                        if line_number {
                            out.push_str(&format!("{}:", idx + 1));
                        }
                        out.push_str(line);
                        out.push('\n');
                    }
                }
            }

            if list_files && file_has_match {
                out.push_str(&file);
                out.push('\n');
            } else if count_only {
                if multiple_files && file != "(stdin)" {
                    out.push_str(&format!("{}:", file));
                }
                out.push_str(&format!("{}\n", match_count));
            }
        }

        Ok(Output::success(out))
    }
}
