use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;

pub struct HeadCommand;

impl BuiltinCommand for HeadCommand {
    fn name(&self) -> &'static str {
        "head"
    }

    fn desc(&self) -> &'static str {
        "Print the first N lines of data"
    }

    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "lines",
                short: Some('n'),
                desc: "print the first K lines",
                flag_type: FlagType::Integer,
                required: false,
            },
            FlagDef {
                name: "bytes",
                short: Some('c'),
                desc: "print the first K bytes",
                flag_type: FlagType::Integer,
                required: false,
            },
        ]
    }

    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        stdin: &str,
    ) -> anyhow::Result<Output> {
        let n = args.get_int("lines");
        let c = args.get_int("bytes");

        let text = if args.positionals.is_empty() {
            stdin.to_string()
        } else {
            let mut buf = String::new();
            for f in &args.positionals {
                buf.push_str(&fs::read_to_string(f).unwrap_or_default());
            }
            buf
        };

        if let Some(bytes) = c {
            let bytes = bytes as usize;
            let end = if text.len() > bytes {
                bytes
            } else {
                text.len()
            };
            return Ok(Output::success(text[..end].to_string()));
        }

        let lines_count = n.unwrap_or(ctx.state.config.head_lines() as i64) as usize;
        let out: Vec<_> = text.lines().take(lines_count).collect();
        Ok(Output::success(out.join("\n") + "\n"))
    }
}
