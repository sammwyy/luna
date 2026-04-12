use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;

pub struct FileCommand;

impl BuiltinCommand for FileCommand {
    fn name(&self) -> &'static str {
        "file"
    }
    fn desc(&self) -> &'static str {
        "Determine file type"
    }
    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        if args.positionals.is_empty() {
            return Ok(Output::error(
                1,
                "".into(),
                "file: missing operand\n".into(),
            ));
        }

        let mut out = String::new();
        for fname in &args.positionals {
            match fs::metadata(fname) {
                Ok(m) => {
                    if m.is_dir() {
                        out.push_str(&format!("{}: directory\n", fname));
                    } else {
                        out.push_str(&format!("{}: regular file\n", fname));
                    }
                }
                Err(_) => {
                    out.push_str(&format!(
                        "{}: cannot open (No such file or directory)\n",
                        fname
                    ));
                }
            }
        }
        Ok(Output::success(out))
    }
}
