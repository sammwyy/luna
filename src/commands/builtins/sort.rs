use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;

pub struct SortCommand;

impl BuiltinCommand for SortCommand {
    fn name(&self) -> &'static str {
        "sort"
    }
    fn desc(&self) -> &'static str {
        "Sort lines of text files"
    }
    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "reverse",
                short: Some('r'),
                desc: "reverse the result of comparisons",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "unique",
                short: Some('u'),
                desc: "output only the first of an equal run",
                flag_type: FlagType::Bool,
                required: false,
            },
        ]
    }
    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        stdin: &str,
    ) -> anyhow::Result<Output> {
        let text = if args.positionals.is_empty() {
            stdin.to_string()
        } else {
            let mut buf = String::new();
            for f in &args.positionals {
                buf.push_str(&fs::read_to_string(f).unwrap_or_default());
            }
            buf
        };

        let mut lines: Vec<&str> = text.lines().collect();
        lines.sort();
        if args.get_bool("reverse") {
            lines.reverse();
        }
        if args.get_bool("unique") {
            lines.dedup();
        }

        Ok(Output::success(lines.join("\n") + "\n"))
    }
}
