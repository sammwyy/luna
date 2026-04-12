use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::path::PathBuf;

pub struct TouchCommand;

impl BuiltinCommand for TouchCommand {
    fn name(&self) -> &'static str {
        "touch"
    }

    fn desc(&self) -> &'static str {
        "Change file access and modification times (create if not exist)"
    }

    fn flags(&self) -> Vec<FlagDef> {
        vec![FlagDef {
            name: "no-create",
            short: Some('c'),
            desc: "do not create any files",
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
        let no_create = args.get_bool("no-create");

        for arg in &args.positionals {
            let path = PathBuf::from(ctx.get_cwd()).join(arg);

            if no_create && !path.exists() {
                continue;
            }

            // For touch, just writing empty content to recreate or open/append
            // To properly update atime/mtime without filetime crate, just touch it by OpenOptions
            let res = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&path);

            if let Err(e) = res {
                return Ok(Output::error(
                    1,
                    "".into(),
                    format!("touch: cannot touch '{}': {}\n", arg, e),
                ));
            }
        }
        Ok(Output::success("".into()))
    }
}
