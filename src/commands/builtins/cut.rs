use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;

pub struct CutCommand;

impl BuiltinCommand for CutCommand {
    fn name(&self) -> &'static str {
        "cut"
    }
    fn desc(&self) -> &'static str {
        "Remove sections from each line of files"
    }
    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "delimiter",
                short: Some('d'),
                desc: "use DELIM instead of TAB for field delimiter",
                flag_type: FlagType::String,
                required: false,
            },
            FlagDef {
                name: "fields",
                short: Some('f'),
                desc: "select only these fields",
                flag_type: FlagType::String,
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
        let delim = args
            .get_string("delimiter")
            .unwrap_or_else(|| "\t".to_string());
        let fields_str = args.get_string("fields").unwrap_or_else(|| "1".to_string());
        let field_idx: usize = fields_str.parse::<usize>().unwrap_or(1).saturating_sub(1); // 1-based to 0-based

        let text = if args.positionals.is_empty() {
            stdin.to_string()
        } else {
            let mut buf = String::new();
            for f in &args.positionals {
                buf.push_str(&fs::read_to_string(f).unwrap_or_default());
            }
            buf
        };

        let mut out = String::new();
        for line in text.lines() {
            let parts: Vec<&str> = line.split(&delim).collect();
            if let Some(field) = parts.get(field_idx) {
                out.push_str(field);
                out.push('\n');
            }
        }

        Ok(Output::success(out))
    }
}
