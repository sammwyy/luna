use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;

pub struct WhichCommand;

impl BuiltinCommand for WhichCommand {
    fn name(&self) -> &'static str {
        "which"
    }
    fn desc(&self) -> &'static str {
        "Locate a command"
    }
    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        if args.positionals.is_empty() {
            return Ok(Output::error(
                1,
                "".into(),
                "which: missing argument\n".into(),
            ));
        }

        let mut out = String::new();
        let mut exit_code = 0;

        for name in &args.positionals {
            if _ctx.state.aliases.contains_key(name) {
                let target = &_ctx.state.aliases[name];
                out.push_str(&format!("<color_primary>{}</color_primary>: alias to <color_secondary>{}</color_secondary>\n", name, target));
            } else if _ctx.state.builtins.contains(name) {
                out.push_str(&format!(
                    "<color_primary>{}</color_primary>: luna built-in command\n",
                    name
                ));
            } else if let Ok(path) = which::which(name) {
                out.push_str(&format!(
                    "<color_secondary>{}</color_secondary>\n",
                    path.display()
                ));
            } else {
                out.push_str(&format!("<color_error>{}: not found</color_error>\n", name));
                exit_code = 1;
            }
        }
        Ok(Output::new(exit_code, out, "".into()))
    }
}
