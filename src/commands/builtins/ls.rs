use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::renderer::table::Table;
use crate::shell::config::LunaConfig;
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;
use std::time::SystemTime;

pub struct LsCommand;

impl BuiltinCommand for LsCommand {
    fn name(&self) -> &'static str {
        "ls"
    }
    fn desc(&self) -> &'static str {
        "List directory contents"
    }
    fn flags(&self) -> Vec<FlagDef> {
        vec![FlagDef {
            name: "all",
            short: Some('a'),
            desc: "Do not ignore entries starting with .",
            flag_type: FlagType::Bool,
            required: false,
        }]
    }
    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        let target = match args.positionals.first() {
            Some(p) => p.clone(),
            None => ctx.get_cwd().to_string(),
        };

        let show_all = args.get_bool("all");
        let mut table = Table::new(vec![
            "<color_primary>Name</color_primary>".into(),
            "<color_primary>Type</color_primary>".into(),
            "<color_primary>Size</color_primary>".into(),
            "<color_primary>Modified</color_primary>".into(),
        ]);

        match fs::read_dir(&target) {
            Ok(entries) => {
                let mut entries_vec: Vec<_> = entries.flatten().collect();
                // Sort: dirs first, then files
                entries_vec.sort_by(|a, b| {
                    let adir = a.metadata().map(|m| m.is_dir()).unwrap_or(false);
                    let bdir = b.metadata().map(|m| m.is_dir()).unwrap_or(false);
                    if adir != bdir {
                        return bdir.cmp(&adir);
                    }
                    a.file_name().cmp(&b.file_name())
                });

                for entry in entries_vec {
                    let metadata = match entry.metadata() {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    let mut name = entry.file_name().to_string_lossy().to_string();
                    if !show_all && name.starts_with('.') {
                        continue;
                    }

                    let is_dir = metadata.is_dir();
                    let ftype = if is_dir {
                        name = format!("<color_secondary>{name}/</color_secondary>");
                        "Dir"
                    } else {
                        "File"
                    };

                    let size = metadata.len();
                    let modified_time = metadata.modified().unwrap_or(SystemTime::now());
                    let now = SystemTime::now();
                    let diff = now
                        .duration_since(modified_time)
                        .unwrap_or_default()
                        .as_secs();

                    // Relative date format
                    let modified_str = if diff > 31536000 {
                        format!("{} years ago", diff / 31536000)
                    } else if diff > 2592000 {
                        format!("{} months ago", diff / 2592000)
                    } else if diff > 86400 {
                        format!("{} days ago", diff / 86400)
                    } else if diff > 3600 {
                        format!("{} hours ago", diff / 3600)
                    } else if diff > 60 {
                        format!("{} mins ago", diff / 60)
                    } else {
                        format!("Just now")
                    };

                    table.add_row(vec![
                        name,
                        ftype.to_string(),
                        if is_dir {
                            "-".to_string()
                        } else {
                            format!("{} B", size)
                        },
                        modified_str,
                    ]);
                }
                if ctx.state.config.ls_render_table() {
                    Ok(Output::success(table.render()))
                } else {
                    let mut out = String::new();
                    for (idx, row) in table.rows.iter().enumerate() {
                        if idx > 0 {
                            out.push_str("  ");
                        }
                        out.push_str(&row[0]);
                    }
                    out.push('\n');
                    Ok(Output::success(out))
                }
            }
            Err(e) => Ok(Output::error(
                1,
                "".into(),
                format!("ls: {}: {}\n", target, e),
            )),
        }
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
            Err(format!("ls: {}: No such file or directory", target))
        }
    }
}
