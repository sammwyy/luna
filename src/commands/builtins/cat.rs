use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::config::LunaConfig;
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::util::as_24_bit_terminal_escaped;

pub struct CatCommand;

impl BuiltinCommand for CatCommand {
    fn name(&self) -> &'static str {
        "cat"
    }
    fn desc(&self) -> &'static str {
        "Read files and print to stdout with syntax highlight"
    }
    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "number",
                short: Some('n'),
                desc: "Number all output lines",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "number-nonblank",
                short: Some('b'),
                desc: "Number nonempty output lines, overrides -n",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "squeeze-blank",
                short: Some('s'),
                desc: "Squeeze multiple adjacent empty lines",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "show-ends",
                short: Some('E'),
                desc: "Display $ at end of each line",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "show-tabs",
                short: Some('T'),
                desc: "Display TAB characters as ^I",
                flag_type: FlagType::Bool,
                required: false,
            },
        ]
    }

    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        let mut out = String::new();
        let ps = &ctx.state.syntax_set;
        let ts = &ctx.state.theme_set;
        let theme = &ts.themes["base16-ocean.dark"];

        let number = args.get_bool("number");
        let number_nonblank = args.get_bool("number-nonblank");
        let squeeze_blank = args.get_bool("squeeze-blank");
        let show_ends = args.get_bool("show-ends");
        let show_tabs = args.get_bool("show-tabs");

        let mut line_count = 1;
        let mut last_was_blank = false;

        for file in args.positionals {
            let path = Path::new(&file);
            if !path.exists() {
                return Ok(Output::error(
                    1,
                    out,
                    format!("cat: {}: No such file\n", file),
                ));
            }
            if path.is_dir() {
                return Ok(Output::error(
                    1,
                    out,
                    format!("cat: {}: Is a directory\n", file),
                ));
            }

            match fs::read_to_string(&file) {
                Ok(content) => {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let highlight_enabled = ctx.state.config.cat_highlight();
                    let allowed_exts = ctx.state.config.cat_highlight_exts();

                    if highlight_enabled && allowed_exts.contains(&ext.to_string()) {
                        let syntax = ps
                            .find_syntax_by_extension(ext)
                            .unwrap_or_else(|| ps.find_syntax_plain_text());

                        let mut h = HighlightLines::new(syntax, theme);
                        for line in content.lines() {
                            let is_blank = line.trim().is_empty();
                            if squeeze_blank && is_blank && last_was_blank {
                                continue;
                            }

                            if number_nonblank {
                                if !is_blank {
                                    out.push_str(&format!("{:6}\t", line_count));
                                    line_count += 1;
                                }
                            } else if number {
                                out.push_str(&format!("{:6}\t", line_count));
                                line_count += 1;
                            }

                            let mut processed_line = line.to_string();
                            if show_tabs {
                                processed_line = processed_line.replace('\t', "^I");
                            }
                            if show_ends {
                                processed_line.push('$');
                            }

                            let ranges = h.highlight_line(&processed_line, ps).unwrap();
                            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                            out.push_str(&escaped);
                            out.push_str("\x1b[0m\n");
                            last_was_blank = is_blank;
                        }
                    } else {
                        for line in content.lines() {
                            let is_blank = line.trim().is_empty();
                            if squeeze_blank && is_blank && last_was_blank {
                                continue;
                            }

                            if number_nonblank {
                                if !is_blank {
                                    out.push_str(&format!("{:6}\t", line_count));
                                    line_count += 1;
                                }
                            } else if number {
                                out.push_str(&format!("{:6}\t", line_count));
                                line_count += 1;
                            }

                            if show_tabs {
                                out.push_str(&line.replace('\t', "^I"));
                            } else {
                                out.push_str(line);
                            }

                            if show_ends {
                                out.push('$');
                            }
                            out.push('\n');
                            last_was_blank = is_blank;
                        }
                    }
                }
                Err(e) => return Ok(Output::error(1, out, format!("cat: {}: {}\n", file, e))),
            }
        }
        Ok(Output::success(out))
    }

    fn dry_run(&self, _config: &LunaConfig, args: &ParsedArgs) -> Result<(), String> {
        for file in &args.positionals {
            let path = Path::new(file);
            if !path.exists() {
                return Err(format!("cat: {}: No such file", file));
            }
        }
        Ok(())
    }
}
