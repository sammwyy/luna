use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;

pub struct TailCommand;

impl BuiltinCommand for TailCommand {
    fn name(&self) -> &'static str {
        "tail"
    }
    fn desc(&self) -> &'static str {
        "Print the last N lines of data"
    }
    fn flags(&self) -> Vec<FlagDef> {
        vec![FlagDef {
            name: "lines",
            short: Some('n'),
            desc: "print the last K lines",
            flag_type: FlagType::Integer,
            required: false,
        }]
    }
    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        stdin: &str,
    ) -> anyhow::Result<Output> {
        let n = args
            .get_int("lines")
            .unwrap_or(ctx.state.config.tail_lines() as i64) as usize;

        let text = if args.positionals.is_empty() {
            stdin.to_string()
        } else {
            let mut buf = String::new();
            for f in &args.positionals {
                buf.push_str(&fs::read_to_string(f).unwrap_or_default());
            }
            buf
        };

        let lines: Vec<_> = text.lines().collect();
        let skip = if lines.len() > n { lines.len() - n } else { 0 };
        let out: Vec<&str> = lines[skip..].iter().copied().collect();
        Ok(Output::success(out.join("\n") + "\n"))
    }
}
