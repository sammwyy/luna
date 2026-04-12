use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;
use std::path::PathBuf;

pub struct RmCommand;

impl BuiltinCommand for RmCommand {
    fn name(&self) -> &'static str {
        "rm"
    }
    fn desc(&self) -> &'static str {
        "Remove files or directories"
    }
    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "recursive",
                short: Some('r'),
                desc: "Remove directories and their contents recursively",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "force",
                short: Some('f'),
                desc: "Ignore nonexistent files and arguments, never prompt",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "verbose",
                short: Some('v'),
                desc: "Explain what is being done",
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
        let recursive = args.get_bool("recursive");
        let force = args.get_bool("force");
        let verbose = args.get_bool("verbose");
        let mut out = String::new();

        if args.positionals.is_empty() {
            if force {
                return Ok(Output::success("".into()));
            }
            return Ok(Output::error(1, "".into(), "rm: missing operand\n".into()));
        }

        for target in &args.positionals {
            let path = PathBuf::from(ctx.get_cwd()).join(target);

            if !path.exists() {
                if force {
                    continue;
                } else {
                    return Ok(Output::error(
                        1,
                        out,
                        format!(
                            "rm: cannot remove '{}': No such file or directory\n",
                            target
                        ),
                    ));
                }
            }

            if path.is_dir() {
                if recursive {
                    if let Err(e) = fs::remove_dir_all(&path) {
                        if !force {
                            return Ok(Output::error(
                                1,
                                out,
                                format!("rm: cannot remove '{}': {}\n", target, e),
                            ));
                        }
                    } else if verbose {
                        out.push_str(&format!("removed directory '{}'\n", target));
                    }
                } else {
                    if !force {
                        return Ok(Output::error(
                            1,
                            out,
                            format!("rm: cannot remove '{}': Is a directory\n", target),
                        ));
                    }
                }
            } else {
                if let Err(e) = fs::remove_file(&path) {
                    if !force {
                        return Ok(Output::error(
                            1,
                            out,
                            format!("rm: cannot remove '{}': {}\n", target, e),
                        ));
                    }
                } else if verbose {
                    out.push_str(&format!("removed '{}'\n", target));
                }
            }
        }
        Ok(Output::success(out))
    }
}
