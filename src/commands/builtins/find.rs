use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;
use std::path::PathBuf;

pub struct FindCommand;

enum MatchType {
    Exact(String),
    Contains(String),
    StartsWith(String),
    EndsWith(String),
}

fn parse_pattern(p: &str) -> MatchType {
    if p.starts_with('*') && p.ends_with('*') && p.len() >= 2 {
        MatchType::Contains(p[1..p.len() - 1].to_string())
    } else if p.starts_with('*') {
        MatchType::EndsWith(p[1..].to_string())
    } else if p.ends_with('*') {
        MatchType::StartsWith(p[..p.len() - 1].to_string())
    } else {
        MatchType::Exact(p.to_string())
    }
}

fn does_match(name: &str, m: &MatchType) -> bool {
    // If target pattern is effectively empty, it matches everything (e.g. single '*')
    let s = match m {
        MatchType::Contains(s) => s,
        MatchType::StartsWith(s) => s,
        MatchType::EndsWith(s) => s,
        MatchType::Exact(s) => s,
    };
    if s.is_empty() {
        return true;
    }

    match m {
        MatchType::Contains(s) => name.contains(s),
        MatchType::StartsWith(s) => name.starts_with(s),
        MatchType::EndsWith(s) => name.ends_with(s),
        MatchType::Exact(s) => name == s, // Changed to Exact matching
    }
}

fn format_match(path: &PathBuf, match_type: Option<&MatchType>) -> String {
    let parent = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let match_text = match match_type {
        Some(MatchType::Contains(s)) => s.as_str(),
        Some(MatchType::StartsWith(s)) => s.as_str(),
        Some(MatchType::EndsWith(s)) => s.as_str(),
        Some(MatchType::Exact(s)) => s.as_str(),
        None => "",
    };

    let final_name = if let Some((stem, ext)) = name.rsplit_once('.') {
        if !match_text.is_empty() {
            if match_text.contains('.') {
                name.replace(
                    match_text,
                    &format!("<color_primary>{}</color_primary>", match_text),
                )
            } else {
                let colored_stem = stem.replace(
                    match_text,
                    &format!("<color_primary>{}</color_primary>", match_text),
                );
                format!("{}<color_warning>.{}</color_warning>", colored_stem, ext)
            }
        } else {
            format!("{}<color_warning>.{}</color_warning>", stem, ext)
        }
    } else {
        if !match_text.is_empty() {
            name.replace(
                match_text,
                &format!("<color_primary>{}</color_primary>", match_text),
            )
        } else {
            name.clone()
        }
    };

    if parent.is_empty() {
        final_name
    } else {
        let parent_display = if parent.ends_with(std::path::MAIN_SEPARATOR)
            || parent.ends_with('/')
            || parent.ends_with('\\')
        {
            parent.clone()
        } else {
            format!("{}\\", parent)
        };
        format!(
            "<color_secondary>{}</color_secondary>{}",
            parent_display, final_name
        )
    }
}

impl BuiltinCommand for FindCommand {
    fn name(&self) -> &'static str {
        "find"
    }

    fn desc(&self) -> &'static str {
        "Search for files in a directory hierarchy"
    }

    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "ext",
                short: Some('e'),
                desc: "Filter by comma-separated extensions (e.g. rs,txt)",
                flag_type: FlagType::String,
                required: false,
            },
            FlagDef {
                name: "files",
                short: Some('f'),
                desc: "Show only files",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "directories",
                short: Some('d'),
                desc: "Show only directories",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "recursive",
                short: Some('r'),
                desc: "Search recursively",
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
        let (start_path, pattern_str) = if args.positionals.is_empty() {
            (ctx.get_cwd().to_string(), None)
        } else if args.positionals.len() == 1 {
            let p = &args.positionals[0];
            if PathBuf::from(p).is_dir() {
                (p.clone(), None)
            } else {
                (ctx.get_cwd().to_string(), Some(p.clone()))
            }
        } else {
            (
                args.positionals[0].clone(),
                Some(args.positionals[1].clone()),
            )
        };

        let match_type = pattern_str.map(|s| parse_pattern(&s));

        let exts_str = args.get_string("ext").unwrap_or_default();
        let exts: Vec<&str> = if exts_str.is_empty() {
            Vec::new()
        } else {
            exts_str.split(',').collect()
        };

        let files_only = args.get_bool("files");
        let dirs_only = args.get_bool("directories");
        let recursive = args.get_bool("recursive");

        struct FindStats {
            files: usize,
            dirs: usize,
        }
        let mut stats = FindStats { files: 0, dirs: 0 };

        fn visit_dirs(
            dir: &PathBuf,
            pattern: Option<&MatchType>,
            exts: &Vec<&str>,
            files_only: bool,
            dirs_only: bool,
            recursive: bool,
            stats: &mut FindStats,
        ) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();

                    let mut is_match = true;

                    if let Some(p) = pattern {
                        if !does_match(&name, p) {
                            is_match = false;
                        }
                    }

                    if is_match && !exts.is_empty() && path.is_file() {
                        if let Some(e) = path.extension() {
                            let ext_str = e.to_string_lossy().to_string();
                            if !exts.contains(&ext_str.as_str()) {
                                is_match = false;
                            }
                        } else {
                            is_match = false;
                        }
                    }

                    if is_match {
                        let is_dir = path.is_dir();
                        let is_file = path.is_file();

                        let should_print = if files_only && !dirs_only {
                            is_file
                        } else if dirs_only && !files_only {
                            is_dir
                        } else {
                            true
                        };

                        if should_print {
                            if is_file {
                                stats.files += 1;
                            } else if is_dir {
                                stats.dirs += 1;
                            }
                            let msg = format!("{}\n", format_match(&path, pattern));
                            crate::renderer::print_stdout(&msg);
                        }
                    }

                    if recursive && path.is_dir() {
                        visit_dirs(
                            &path, pattern, exts, files_only, dirs_only, recursive, stats,
                        );
                    }
                }
            }
        }

        visit_dirs(
            &PathBuf::from(&start_path),
            match_type.as_ref(),
            &exts,
            files_only,
            dirs_only,
            recursive,
            &mut stats,
        );

        let stats_msg = format!(
            "\n<color_secondary>Found <color_primary>{}</color_primary> files and <color_primary>{}</color_primary> directories.</color_secondary>\n",
            stats.files, stats.dirs
        );
        crate::renderer::print_stdout(&stats_msg);

        Ok(Output::success("".into()))
    }
}
