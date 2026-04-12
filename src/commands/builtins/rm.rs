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
        vec![FlagDef {
            name: "recursive",
            short: Some('r'),
            desc: "Remove directories and their contents recursively",
            flag_type: FlagType::Bool,
            required: false,
        }]
    }
    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        let recursive = args.get_bool("recursive");

        if args.positionals.is_empty() {
            return Ok(Output::error(1, "".into(), "rm: missing operand\n".into()));
        }

        for target in &args.positionals {
            let path = PathBuf::from(ctx.get_cwd()).join(target);
            if path.is_dir() {
                if recursive {
                    if let Err(e) = fs::remove_dir_all(&path) {
                        return Ok(Output::error(
                            1,
                            "".into(),
                            format!("rm: cannot remove '{}': {}\n", target, e),
                        ));
                    }
                } else {
                    return Ok(Output::error(
                        1,
                        "".into(),
                        format!("rm: cannot remove '{}': Is a directory\n", target),
                    ));
                }
            } else {
                if let Err(e) = fs::remove_file(&path) {
                    return Ok(Output::error(
                        1,
                        "".into(),
                        format!("rm: cannot remove '{}': {}\n", target, e),
                    ));
                }
            }
        }
        Ok(Output::success("".into()))
    }
}
