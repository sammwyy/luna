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
            FlagDef {
                name: "numeric-sort",
                short: Some('n'),
                desc: "compare according to string numerical value",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "ignore-case",
                short: Some('f'),
                desc: "fold lower case to upper case characters",
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

        let numeric = args.get_bool("numeric-sort");
        let ignore_case = args.get_bool("ignore-case");

        let mut lines: Vec<&str> = text.lines().collect();

        lines.sort_by(|a, b| {
            let (a, b) = if ignore_case {
                (a.to_lowercase(), b.to_lowercase())
            } else {
                (a.to_string(), b.to_string())
            };

            if numeric {
                let a_num = a.trim().parse::<f64>().unwrap_or(0.0);
                let b_num = b.trim().parse::<f64>().unwrap_or(0.0);
                a_num.partial_cmp(&b_num).unwrap_or(a.cmp(&b))
            } else {
                a.cmp(&b)
            }
        });

        if args.get_bool("reverse") {
            lines.reverse();
        }
        if args.get_bool("unique") {
            lines.dedup();
        }

        Ok(Output::success(lines.join("\n") + "\n"))
    }
}
