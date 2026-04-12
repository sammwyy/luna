use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;

pub struct UniqCommand;

impl BuiltinCommand for UniqCommand {
    fn name(&self) -> &'static str {
        "uniq"
    }
    fn desc(&self) -> &'static str {
        "Report or omit repeated lines"
    }
    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "count",
                short: Some('c'),
                desc: "prefix lines by the number of occurrences",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "repeated",
                short: Some('d'),
                desc: "only print duplicate lines, one for each group",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "unique",
                short: Some('u'),
                desc: "only print unique lines",
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

        let count = args.get_bool("count");
        let duplicates_only = args.get_bool("repeated");
        let unique_only = args.get_bool("unique");

        let mut out = String::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let current = lines[i];
            let mut run_count = 1usize;
            while i + run_count < lines.len() && lines[i + run_count] == current {
                run_count += 1;
            }

            let should_print = match (duplicates_only, unique_only) {
                (true, _) => run_count > 1,
                (_, true) => run_count == 1,
                _ => true,
            };

            if should_print {
                if count {
                    out.push_str(&format!("{:>7} {}\n", run_count, current));
                } else {
                    out.push_str(current);
                    out.push('\n');
                }
            }
            i += run_count;
        }

        Ok(Output::success(out))
    }
}
