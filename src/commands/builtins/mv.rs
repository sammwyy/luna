use crate::commands::system::{BuiltinCommand, ParsedArgs};
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
    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        if args.positionals.len() < 2 {
            return Ok(Output::error(
                1,
                "".into(),
                "mv: missing destination operand\n".into(),
            ));
        }
        let dst_arg = args.positionals.last().unwrap();
        let src = PathBuf::from(ctx.get_cwd()).join(&args.positionals[0]);
        let dst_base = PathBuf::from(ctx.get_cwd()).join(dst_arg);

        let dst = if dst_base.is_dir() {
            dst_base.join(src.file_name().unwrap())
        } else {
            dst_base
        };

        if let Err(e) = fs::rename(&src, &dst) {
            return Ok(Output::error(
                1,
                "".into(),
                format!("mv: cannot move to '{}': {}\n", dst_arg, e),
            ));
        }

        Ok(Output::success("".into()))
    }
}
