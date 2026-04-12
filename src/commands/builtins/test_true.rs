use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;

pub struct TrueCommand;

impl BuiltinCommand for TrueCommand {
    fn name(&self) -> &'static str {
        "true"
    }
    fn desc(&self) -> &'static str {
        "Do nothing, successfully"
    }
    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        _args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        Ok(Output::success("".into()))
    }
}
