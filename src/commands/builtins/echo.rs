use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;

pub struct EchoCommand;

impl BuiltinCommand for EchoCommand {
    fn name(&self) -> &'static str {
        "echo"
    }
    fn desc(&self) -> &'static str {
        "Print arguments"
    }
    fn flags(&self) -> Vec<FlagDef> {
        vec![FlagDef {
            name: "no-newline",
            short: Some('n'),
            desc: "Do not print the trailing newline",
            flag_type: FlagType::Bool,
            required: false,
        }]
    }
    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        let space_separated = args.positionals.join(" ");
        if args.get_bool("no-newline") {
            Ok(Output::success(space_separated))
        } else {
            Ok(Output::success(format!("{}\n", space_separated)))
        }
    }
}
