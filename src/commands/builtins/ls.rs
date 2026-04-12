use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::renderer::table::Table;
use crate::shell::config::LunaConfig;
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use users::{get_group_by_gid, get_user_by_uid};

pub struct LsCommand;

impl BuiltinCommand for LsCommand {
    fn name(&self) -> &'static str {
        "ls"
    }
    fn desc(&self) -> &'static str {
        "List directory contents"
    }
    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "all",
                short: Some('a'),
                desc: "Do not ignore entries starting with .",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "long",
                short: Some('l'),
                desc: "Use a long listing format",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "human-readable",
                short: Some('h'),
                desc: "With -l, print sizes like 1K 234M 2G etc.",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "recursive",
                short: Some('r'),
                desc: "List subdirectories recursively (1 level, tree-like)",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "sort-size",
                short: Some('S'),
                desc: "Sort by file size, largest first",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "sort-time",
                short: Some('t'),
                desc: "Sort by modification time, newest first",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "reverse",
                short: Some('v'),
                desc: "Reverse order while sorting",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "directory",
                short: Some('d'),
                desc: "List directories themselves, not their contents",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "one-line",
                short: Some('1'),
                desc: "List one file per line",
                flag_type: FlagType::Bool,
                required: false,
            },
        ]
    }

    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        let targets = if args.positionals.is_empty() {
            vec![".".to_string()]
        } else {
            args.positionals.clone()
        };

        let show_all = args.get_bool("all");
        let long_format = args.get_bool("long");
        let human_readable = args.get_bool("human-readable");
        let recursive = args.get_bool("recursive");
        let sort_size = args.get_bool("sort-size");
        let sort_time = args.get_bool("sort-time");
        let reverse = args.get_bool("reverse");
        let directory_only = args.get_bool("directory");
        let one_line = args.get_bool("one-line");

        let mut final_output = String::new();

        for target in targets.iter() {
            if targets.len() > 1 || recursive {
                if !final_output.is_empty() {
                    final_output.push('\n');
                }
                final_output.push_str(&format!("{target}:\n"));
            }

            match self.list_dir(
                target,
                show_all,
                long_format,
                human_readable,
                recursive,
                sort_size,
                sort_time,
                reverse,
                directory_only,
                one_line,
                ctx,
            ) {
                Ok(out) => final_output.push_str(&out),
                Err(e) => {
                    final_output.push_str(&format!("ls: {}: {}\n", target, e));
                }
            }
        }

        Ok(Output::success(final_output))
    }

    fn dry_run(&self, _config: &LunaConfig, _args: &ParsedArgs) -> Result<(), String> {
        Ok(())
    }
}

impl LsCommand {
    fn format_size(&self, size: u64, human: bool) -> String {
        if !human {
            return format!("{} B", size);
        }

        if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} K", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            format!("{:.1} M", size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} G", size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    fn format_permissions(&self, metadata: &std::fs::Metadata) -> String {
        #[cfg(unix)]
        {
            let mode = metadata.permissions().mode();
            let ftype = if metadata.is_dir() {
                "d"
            } else if metadata.is_symlink() {
                "l"
            } else {
                "-"
            };

            let r1 = if mode & 0o400 != 0 { "r" } else { "-" };
            let w1 = if mode & 0o200 != 0 { "w" } else { "-" };
            let x1 = if mode & 0o100 != 0 { "x" } else { "-" };

            let r2 = if mode & 0o040 != 0 { "r" } else { "-" };
            let w2 = if mode & 0o020 != 0 { "w" } else { "-" };
            let x2 = if mode & 0o010 != 0 { "x" } else { "-" };

            let r3 = if mode & 0o004 != 0 { "r" } else { "-" };
            let w3 = if mode & 0o002 != 0 { "w" } else { "-" };
            let x3 = if mode & 0o001 != 0 { "x" } else { "-" };

            format!(
                "{ftype}<color_error>{r1}{w1}{x1}</color_error><color_warn>{r2}{w2}{x2}</color_warn><color_primary>{r3}{w3}{x3}</color_primary>"
            )
        }
        #[cfg(not(unix))]
        {
            if metadata.is_dir() {
                "d---------".to_string()
            } else {
                "----------".to_string()
            }
        }
    }

    fn format_owner(&self, uid: u32) -> String {
        #[cfg(unix)]
        {
            if let Some(user) = get_user_by_uid(uid) {
                user.name().to_string_lossy().to_string()
            } else {
                uid.to_string()
            }
        }
        #[cfg(not(unix))]
        {
            uid.to_string()
        }
    }

    fn format_group(&self, gid: u32) -> String {
        #[cfg(unix)]
        {
            if let Some(group) = get_group_by_gid(gid) {
                group.name().to_string_lossy().to_string()
            } else {
                gid.to_string()
            }
        }
        #[cfg(not(unix))]
        {
            gid.to_string()
        }
    }

    fn list_dir(
        &self,
        target: &str,
        show_all: bool,
        long_format: bool,
        human_readable: bool,
        recursive: bool,
        sort_size: bool,
        sort_time: bool,
        reverse: bool,
        directory_only: bool,
        one_line: bool,
        ctx: &Context<LunaState>,
    ) -> anyhow::Result<String> {
        let mut headers = vec!["<color_primary>Name</color_primary>".into()];
        if long_format {
            headers.push("<color_primary>Permissions</color_primary>".into());
            headers.push("<color_primary>User</color_primary>".into());
            headers.push("<color_primary>Group</color_primary>".into());
        }
        headers.push("<color_primary>Type</color_primary>".into());
        headers.push("<color_primary>Size</color_primary>".into());
        headers.push("<color_primary>Modified</color_primary>".into());

        let mut table = Table::new(headers);
        table.alternating_rows = ctx.state.config.ls_alternating_rows();

        let path = Path::new(target);
        if directory_only {
            self.add_entry_to_table(&mut table, path, long_format, human_readable, ctx, None)?;
        } else if path.is_file() {
            self.add_entry_to_table(&mut table, path, long_format, human_readable, ctx, None)?;
        } else {
            let entries = fs::read_dir(target)?;
            let mut entries_vec: Vec<_> = entries.flatten().collect();

            // Sorting logic
            entries_vec.sort_by(|a, b| {
                let res = if sort_size {
                    let asize = a.metadata().map(|m| m.len()).unwrap_or(0);
                    let bsize = b.metadata().map(|m| m.len()).unwrap_or(0);
                    bsize.cmp(&asize)
                } else if sort_time {
                    let atime = a
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(SystemTime::UNIX_EPOCH);
                    let btime = b
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(SystemTime::UNIX_EPOCH);
                    btime.cmp(&atime)
                } else {
                    let adir = a.metadata().map(|m| m.is_dir()).unwrap_or(false);
                    let bdir = b.metadata().map(|m| m.is_dir()).unwrap_or(false);
                    if adir != bdir {
                        bdir.cmp(&adir)
                    } else {
                        a.file_name().cmp(&b.file_name())
                    }
                };

                if reverse {
                    res.reverse()
                } else {
                    res
                }
            });

            for entry in entries_vec {
                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let name = entry.file_name().to_string_lossy().to_string();
                if !show_all && name.starts_with('.') {
                    continue;
                }

                self.add_entry_to_table(
                    &mut table,
                    &entry.path(),
                    long_format,
                    human_readable,
                    ctx,
                    None,
                )?;

                if recursive && metadata.is_dir() {
                    if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                        let mut sub_vec: Vec<fs::DirEntry> = sub_entries.flatten().collect();
                        sub_vec.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

                        for sub_entry in sub_vec {
                            let sub_name = sub_entry.file_name().to_string_lossy().to_string();
                            if !show_all && sub_name.starts_with('.') {
                                continue;
                            }
                            self.add_entry_to_table(
                                &mut table,
                                &sub_entry.path(),
                                long_format,
                                human_readable,
                                ctx,
                                Some(" └── "),
                            )?;
                        }
                    }
                }
            }

            let output = if long_format || ctx.state.config.ls_render_table() {
                table.render()
            } else {
                let mut out = String::new();
                for (idx, row) in table.rows.iter().enumerate() {
                    if idx > 0 {
                        if one_line {
                            out.push('\n');
                        } else {
                            out.push_str("  ");
                        }
                    }
                    out.push_str(&row[0]);
                }
                out.push('\n');
                out
            };

            return Ok(output);
        }

        Ok(table.render())
    }

    fn add_entry_to_table(
        &self,
        table: &mut Table,
        path: &Path,
        long: bool,
        human_readable: bool,
        _ctx: &Context<LunaState>,
        prefix: Option<&str>,
    ) -> anyhow::Result<()> {
        let metadata = path.metadata()?;
        let is_dir = metadata.is_dir();
        let base_name = path
            .file_name()
            .unwrap_or_else(|| path.as_os_str())
            .to_string_lossy();

        let mut name = if is_dir {
            format!("<color_secondary>{base_name}/</color_secondary>")
        } else {
            base_name.to_string()
        };

        if let Some(p) = prefix {
            name = format!("<color_border>{p}</color_border>{name}");
        }

        let ftype = if is_dir { "Dir" } else { "File" };
        let size = metadata.len();
        let modified_time = metadata.modified().unwrap_or(SystemTime::now());
        let now = SystemTime::now();
        let diff = now
            .duration_since(modified_time)
            .unwrap_or_default()
            .as_secs();

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
            "Just now".to_string()
        };

        let mut row = vec![name];

        if long {
            row.push(self.format_permissions(&metadata));
            #[cfg(unix)]
            {
                row.push(self.format_owner(metadata.uid()));
                row.push(self.format_group(metadata.gid()));
            }
            #[cfg(not(unix))]
            {
                row.push("-".to_string());
                row.push("-".to_string());
            }
        }

        row.push(ftype.to_string());
        row.push(if is_dir {
            "-".to_string()
        } else {
            self.format_size(size, human_readable)
        });
        row.push(modified_str);

        table.add_row(row);
        Ok(())
    }
}
