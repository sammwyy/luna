use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;

pub struct SedCommand;

impl BuiltinCommand for SedCommand {
    fn name(&self) -> &'static str {
        "sed"
    }
    fn desc(&self) -> &'static str {
        "Stream editor for filtering and transforming text"
    }
    fn flags(&self) -> Vec<FlagDef> {
        vec![FlagDef {
            name: "expression",
            short: Some('e'),
            desc: "add the script to the commands to be executed",
            flag_type: FlagType::String,
            required: false,
        }]
    }
    // simple s/old/new/g support
    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        stdin: &str,
    ) -> anyhow::Result<Output> {
        let script = match args.get_string("expression") {
            Some(s) => s,
            None => {
                if args.positionals.is_empty() {
                    return Ok(Output::error(1, "".into(), "sed: missing script\n".into()));
                }
                args.positionals[0].clone()
            }
        };

        let files = if args.get_string("expression").is_some() {
            &args.positionals[..]
        } else {
            &args.positionals[1..]
        };

        let text = if files.is_empty() {
            stdin.to_string()
        } else {
            let mut buf = String::new();
            for f in files {
                buf.push_str(&fs::read_to_string(f).unwrap_or_default());
            }
            buf
        };

        if !script.starts_with("s/") {
            return Ok(Output::error(
                1,
                "".into(),
                "sed: unsupported script format, use s/old/new/\n".into(),
            ));
        }

        let parts: Vec<&str> = script.split('/').collect();
        if parts.len() < 3 {
            return Ok(Output::error(
                1,
                "".into(),
                "sed: bad script format\n".into(),
            ));
        }

        let search = parts[1];
        let replace = parts[2];
        let global = parts.get(3).unwrap_or(&"").contains("g");

        let mut out = String::new();
        for line in text.lines() {
            if global {
                out.push_str(&line.replace(search, replace));
            } else {
                out.push_str(&line.replacen(search, replace, 1));
            }
            out.push('\n');
        }

        Ok(Output::success(out))
    }
}
