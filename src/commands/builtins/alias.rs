use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;

pub struct AliasCommand;

impl BuiltinCommand for AliasCommand {
    fn name(&self) -> &'static str {
        "alias"
    }
    fn desc(&self) -> &'static str {
        "Create or list aliases"
    }
    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        if let Some(arg) = args.positionals.first() {
            if let Some(eq) = arg.find('=') {
                let name = arg[..eq].to_string();
                let val = arg[eq + 1..].to_string();
                ctx.state.aliases.insert(name, val);
                return Ok(Output::success("".into()));
            }
        }
        let mut out = String::new();
        let mut keys: Vec<_> = ctx.state.aliases.keys().collect();
        keys.sort();
        for k in keys {
            let v = &ctx.state.aliases[k];
            out.push_str(&format!(
                "alias <color_primary>{}</color_primary>='<color_secondary>{}</color_secondary>'\n",
                k, v
            ));
        }
        Ok(Output::success(out))
    }
}
