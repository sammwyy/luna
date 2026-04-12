use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;
use std::path::PathBuf;

pub struct FindCommand;

impl BuiltinCommand for FindCommand {
    fn name(&self) -> &'static str {
        "find"
    }
    fn desc(&self) -> &'static str {
        "Search for files in a directory hierarchy"
    }
    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        let start_path = if args.positionals.is_empty() {
            ctx.get_cwd().to_string()
        } else {
            args.positionals[0].clone()
        };

        let name_filter = args
            .positionals
            .iter()
            .position(|x| x == "-name")
            .and_then(|i| args.positionals.get(i + 1));

        let mut out = String::new();

        fn visit_dirs(dir: &PathBuf, out: &mut String, name_filter: Option<&String>) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();

                    let matches = match name_filter {
                        Some(f) => entry.file_name().to_string_lossy() == f.as_str(),
                        None => true,
                    };

                    if matches {
                        out.push_str(&format!("{}\n", path.display()));
                    }

                    if path.is_dir() {
                        visit_dirs(&path, out, name_filter);
                    }
                }
            }
        }

        // Root match
        if let Some(f) = name_filter {
            if PathBuf::from(&start_path).file_name().unwrap_or_default() == f.as_str() {
                out.push_str(&format!("{}\n", start_path));
            }
        } else {
            out.push_str(&format!("{}\n", start_path));
        }

        visit_dirs(&PathBuf::from(&start_path), &mut out, name_filter);
        Ok(Output::success(out))
    }
}
