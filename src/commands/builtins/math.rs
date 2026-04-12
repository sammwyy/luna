use crate::commands::system::{BuiltinCommand, FlagDef, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;

pub struct MathCommand;

impl BuiltinCommand for MathCommand {
    fn name(&self) -> &'static str {
        "math"
    }

    fn desc(&self) -> &'static str {
        "Evaluate mathematical expressions"
    }

    fn flags(&self) -> Vec<FlagDef> {
        vec![]
    }

    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        stdin: &str,
    ) -> anyhow::Result<Output> {
        let expression = if args.positionals.is_empty() {
            if stdin.is_empty() {
                return Ok(Output::error(
                    1,
                    "".into(),
                    "math: missing expression\n".into(),
                ));
            }
            stdin.to_string()
        } else {
            args.positionals.join(" ")
        };

        match meval::eval_str(&expression) {
            Ok(result) => Ok(Output::success(format!("{}\n", result))),
            Err(e) => Ok(Output::error(
                1,
                "".into(),
                format!(
                    "math: error evaluating expression '{}': {}\n",
                    expression, e
                ),
            )),
        }
    }
}
