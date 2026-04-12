use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;

pub struct EnvCommand;

impl BuiltinCommand for EnvCommand {
    fn name(&self) -> &'static str {
        "env"
    }
    fn desc(&self) -> &'static str {
        "List environment variables"
    }
    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        _args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        let mut out = String::new();
        let mut keys: Vec<_> = ctx.env.keys().collect();
        keys.sort();
        for k in keys {
            let v = &ctx.env[k];
            out.push_str(&format!(
                "<color_primary>{}</color_primary>=<color_text>{}</color_text>\n",
                k, v
            ));
        }
        Ok(Output::success(out))
    }
}
