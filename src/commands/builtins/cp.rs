use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;
use std::path::PathBuf;

pub struct CpCommand;

impl BuiltinCommand for CpCommand {
    fn name(&self) -> &'static str {
        "cp"
    }
    fn desc(&self) -> &'static str {
        "Copy files"
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
                "cp: missing destination operand\n".into(),
            ));
        }
        let dst_arg = args.positionals.last().unwrap();
        let srcs = &args.positionals[..args.positionals.len() - 1];
        let dst = PathBuf::from(ctx.get_cwd()).join(dst_arg);

        for src_arg in srcs {
            let src = PathBuf::from(ctx.get_cwd()).join(src_arg);
            let final_dst = if dst.is_dir() {
                dst.join(src.file_name().unwrap())
            } else {
                dst.clone()
            };
            if let Err(e) = fs::copy(&src, &final_dst) {
                return Ok(Output::error(
                    1,
                    "".into(),
                    format!("cp: cannot copy '{}': {}\n", src_arg, e),
                ));
            }
        }
        Ok(Output::success("".into()))
    }
}
