use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::{Context, Output};

pub struct NextCommand;

impl BuiltinCommand for NextCommand {
    fn name(&self) -> &'static str {
        "next"
    }

    fn desc(&self) -> &'static str {
        "Go to the next directory in history"
    }

    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        _args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        if ctx.state.dir_index + 1 < ctx.state.dir_history.len() {
            ctx.state.dir_index += 1;
            let target = ctx.state.dir_history[ctx.state.dir_index].clone();
            ctx.set_cwd(target.clone());
            Ok(Output::success(format!("{}\n", target)))
        } else {
            Ok(Output::error(
                1,
                String::new(),
                "next: no next directory\n".to_string(),
            ))
        }
    }
}
