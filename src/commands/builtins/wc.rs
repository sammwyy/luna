use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;

pub struct WcCommand;

impl BuiltinCommand for WcCommand {
    fn name(&self) -> &'static str {
        "wc"
    }

    fn desc(&self) -> &'static str {
        "Print newline, word, and byte counts"
    }

    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "lines",
                short: Some('l'),
                desc: "print the newline counts",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "words",
                short: Some('w'),
                desc: "print the word counts",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "bytes",
                short: Some('c'),
                desc: "print the byte counts",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "chars",
                short: Some('m'),
                desc: "print the character counts",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "max-line-length",
                short: Some('L'),
                desc: "print the maximum display width",
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
        let mut count_lines = args.get_bool("lines");
        let mut count_words = args.get_bool("words");
        let mut count_bytes = args.get_bool("bytes");
        let mut count_chars = args.get_bool("chars");
        let mut count_max_line = args.get_bool("max-line-length");

        if !count_lines && !count_words && !count_bytes && !count_chars && !count_max_line {
            count_lines = true;
            count_words = true;
            count_bytes = true;
        }

        let text = if args.positionals.is_empty() {
            stdin.to_string()
        } else {
            let mut buf = String::new();
            for f in &args.positionals {
                buf.push_str(&fs::read_to_string(f).unwrap_or_default());
            }
            buf
        };

        let lines = text.lines().count();
        let words = text.split_whitespace().count();
        let bytes = text.len();
        let chars = text.chars().count();
        let max_line = text.lines().map(|l| l.len()).max().unwrap_or(0);

        let mut out = String::new();
        if count_lines {
            out.push_str(&format!("{:>8}", lines));
        }
        if count_words {
            out.push_str(&format!("{:>8}", words));
        }
        if count_chars {
            out.push_str(&format!("{:>8}", chars));
        } else if count_bytes {
            out.push_str(&format!("{:>8}", bytes));
        }
        if count_max_line {
            out.push_str(&format!("{:>8}", max_line));
        }
        out.push('\n');

        Ok(Output::success(out))
    }
}
