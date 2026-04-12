use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::{Context, Output};

pub struct BackCommand;

impl BuiltinCommand for BackCommand {
    fn name(&self) -> &'static str {
        "back"
    }

    fn desc(&self) -> &'static str {
        "Go to the previous directory in history"
    }

    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        _args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        if ctx.state.dir_index > 0 {
            ctx.state.dir_index -= 1;
            let target = ctx.state.dir_history[ctx.state.dir_index].clone();
            ctx.set_cwd(target.clone());
            Ok(Output::success(format!("{}\n", target)))
        } else {
            Ok(Output::error(
                1,
                String::new(),
                "back: no previous directory\n".to_string(),
            ))
        }
    }
}
