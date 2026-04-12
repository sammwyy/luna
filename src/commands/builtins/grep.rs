use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;

pub struct GrepCommand;

impl BuiltinCommand for GrepCommand {
    fn name(&self) -> &'static str {
        "grep"
    }
    fn desc(&self) -> &'static str {
        "Print lines that match patterns"
    }
    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "ignore-case",
                short: Some('i'),
                desc: "ignore case distinctions",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "invert-match",
                short: Some('v'),
                desc: "select non-matching lines",
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
        if args.positionals.is_empty() {
            return Ok(Output::error(
                1,
                "".into(),
                "grep: missing pattern\n".into(),
            ));
        }

        let pattern = args.positionals[0].clone();
        let ignore_case = args.get_bool("ignore-case");
        let invert_match = args.get_bool("invert-match");

        let search_str = if ignore_case {
            pattern.to_lowercase()
        } else {
            pattern
        };

        let text = if args.positionals.len() < 2 {
            stdin.to_string()
        } else {
            let mut buf = String::new();
            for f in &args.positionals[1..] {
                buf.push_str(&fs::read_to_string(f).unwrap_or_default());
            }
            buf
        };

        let mut out = String::new();
        for line in text.lines() {
            let target = if ignore_case {
                line.to_lowercase()
            } else {
                line.to_string()
            };
            let matched = target.contains(&search_str);
            if matched ^ invert_match {
                out.push_str(line);
                out.push('\n');
            }
        }

        Ok(Output::success(out))
    }
}
