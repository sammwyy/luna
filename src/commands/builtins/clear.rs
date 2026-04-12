use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;

pub struct ClearCommand;

impl BuiltinCommand for ClearCommand {
    fn name(&self) -> &'static str {
        "clear"
    }
    fn desc(&self) -> &'static str {
        "Clear the terminal screen"
    }
    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        _args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        Ok(Output::success("\x1b[H\x1b[2J\x1b[3J".into()))
    }
}
