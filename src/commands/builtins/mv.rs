use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;
use std::path::PathBuf;

pub struct MvCommand;

impl BuiltinCommand for MvCommand {
    fn name(&self) -> &'static str {
        "mv"
    }

    fn desc(&self) -> &'static str {
        "Move or rename files"
    }

    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "force",
                short: Some('f'),
                desc: "do not prompt before overwriting",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "verbose",
                short: Some('v'),
                desc: "explain what is being done",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "no-clobber",
                short: Some('n'),
                desc: "do not overwrite an existing file",
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
        let force = args.get_bool("force");
        let verbose = args.get_bool("verbose");
        let no_clobber = args.get_bool("no-clobber");
        let mut out = String::new();

        if args.positionals.len() < 2 {
            return Ok(Output::error(
                1,
                "".into(),
                "mv: missing destination operand\n".into(),
            ));
        }

        let dst_arg = args.positionals.last().unwrap();
        let srcs = &args.positionals[..args.positionals.len() - 1];
        let dst_base = PathBuf::from(ctx.get_cwd()).join(dst_arg);

        for src_arg in srcs {
            let src = PathBuf::from(ctx.get_cwd()).join(src_arg);

            if !src.exists() {
                return Ok(Output::error(
                    1,
                    out,
                    format!("mv: cannot stat '{}': No such file or directory\n", src_arg),
                ));
            }

            let dst = if dst_base.is_dir() {
                dst_base.join(src.file_name().unwrap())
            } else {
                dst_base.clone()
            };

            if dst.exists() {
                if no_clobber {
                    continue;
                }
                if !force {
                    // In a real shell we might prompt, but here we just proceed or error.
                    // Bash -f suppresses prompt. Since we don't have prompt yet, we just overwrite.
                }
            }

            if let Err(e) = fs::rename(&src, &dst) {
                return Ok(Output::error(
                    1,
                    out,
                    format!(
                        "mv: cannot move '{}' to '{}': {}\n",
                        src_arg,
                        dst.display(),
                        e
                    ),
                ));
            } else if verbose {
                out.push_str(&format!("'{}' -> '{}'\n", src_arg, dst.display()));
            }
        }

        Ok(Output::success(out))
    }
}
