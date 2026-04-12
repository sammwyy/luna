use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;

pub struct ExitCommand;

impl BuiltinCommand for ExitCommand {
    fn name(&self) -> &'static str {
        "exit"
    }
    fn desc(&self) -> &'static str {
        "Exit the shell"
    }
    fn flags(&self) -> Vec<FlagDef> {
        vec![FlagDef {
            name: "code",
            short: Some('c'),
            desc: "Exit status code",
            flag_type: FlagType::Integer,
            required: false,
        }]
    }
    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        let code = args.get_int("code").unwrap_or(0);
        std::process::exit(code as i32);
    }
}
