use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
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

    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "logical",
                short: Some('L'),
                desc: "use PWD from environment, even if it contains symlinks",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "physical",
                short: Some('P'),
                desc: "avoid all symlinks",
                flag_type: FlagType::Bool,
                required: false,
            },
        ]
    }

    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        _args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        // Luna currently doesn't track physical vs logical differently in Context
        // but we accept the flags for compatibility.
        Ok(Output::success(format!("{}\n", ctx.get_cwd())))
    }
}
