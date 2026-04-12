use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;

pub struct FalseCommand;

impl BuiltinCommand for FalseCommand {
    fn name(&self) -> &'static str {
        "false"
    }
    fn desc(&self) -> &'static str {
        "Do nothing, unsuccessfully"
    }
    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        _args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        Ok(Output::error(1, "".into(), "".into()))
    }
}
