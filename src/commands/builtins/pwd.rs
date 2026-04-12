use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;

pub struct PwdCommand;

impl BuiltinCommand for PwdCommand {
    fn name(&self) -> &'static str {
        "pwd"
    }
    fn desc(&self) -> &'static str {
        "Print working directory"
    }
    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        _args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        Ok(Output::success(format!("{}\n", ctx.get_cwd())))
    }
}
