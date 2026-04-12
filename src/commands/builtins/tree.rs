use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::config::LunaConfig;
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;
use std::path::PathBuf;

pub struct TreeCommand;

impl BuiltinCommand for TreeCommand {
    fn name(&self) -> &'static str {
        "tree"
    }
    fn desc(&self) -> &'static str {
        "List contents of directories in a tree-like format"
    }
    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        let target = if args.positionals.is_empty() {
            ctx.get_cwd().to_string()
        } else {
            args.positionals[0].clone()
        };

        let mut out = String::new();
        let target_path = PathBuf::from(&target);
        out.push_str(&format!(
            "<color_primary><bold>{}</bold></color_primary>\n",
            target_path
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or(target.into())
        ));

        fn visit_dirs(dir: &PathBuf, prefix: &str, out: &mut String, depth: usize) {
            if depth > 10 {
                return;
            } // Safety depth

            if let Ok(entries) = fs::read_dir(dir) {
                let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
                // Sort: dirs first
                entries.sort_by(|a, b| {
                    let adir = a.path().is_dir();
                    let bdir = b.path().is_dir();
                    if adir != bdir {
                        return bdir.cmp(&adir);
                    }
                    a.file_name().cmp(&b.file_name())
                });

                let count = entries.len();
                for (i, entry) in entries.iter().enumerate() {
                    let is_last = i == count - 1;
                    let marker = if is_last { "└── " } else { "├── " };

                    let name = entry.file_name().to_string_lossy().into_owned();
                    let is_dir = entry.path().is_dir();

                    let styled_name = if is_dir {
                        format!("<color_secondary>{name}/</color_secondary>")
                    } else {
                        format!("<color_text>{name}</color_text>")
                    };

                    out.push_str(&format!(
                        "<color_border>{}</color_border><color_border>{}</color_border>{}\n",
                        prefix, marker, styled_name
                    ));

                    if is_dir {
                        let next_prefix = if is_last {
                            format!("{}    ", prefix)
                        } else {
                            format!("{}│   ", prefix)
                        };
                        visit_dirs(&entry.path(), &next_prefix, out, depth + 1);
                    }
                }
            }
        }

        visit_dirs(&target_path, "", &mut out, 0);
        Ok(Output::success(out))
    }

    fn dry_run(&self, _config: &LunaConfig, args: &ParsedArgs) -> Result<(), String> {
        let target = args
            .positionals
            .first()
            .cloned()
            .unwrap_or_else(|| ".".to_string());
        if std::path::Path::new(&target).exists() {
            Ok(())
        } else {
            Err(format!("tree: {}: No such directory", target))
        }
    }
}
