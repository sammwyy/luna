use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;
use std::path::PathBuf;

pub struct CpCommand;

impl BuiltinCommand for CpCommand {
    fn name(&self) -> &'static str {
        "cp"
    }

    fn desc(&self) -> &'static str {
        "Copy files"
    }

    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "recursive",
                short: Some('r'),
                desc: "copy directories recursively",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "force",
                short: Some('f'),
                desc: "if an existing destination file cannot be opened, remove it and try again",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "verbose",
                short: Some('v'),
                desc: "explain what is being done",
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
        let recursive = args.get_bool("recursive");
        let force = args.get_bool("force");
        let verbose = args.get_bool("verbose");
        let mut out = String::new();

        if args.positionals.len() < 2 {
            return Ok(Output::error(
                1,
                "".into(),
                "cp: missing destination operand\n".into(),
            ));
        }
        let dst_arg = args.positionals.last().unwrap();
        let srcs = &args.positionals[..args.positionals.len() - 1];
        let dst = PathBuf::from(ctx.get_cwd()).join(dst_arg);

        for src_arg in srcs {
            let src = PathBuf::from(ctx.get_cwd()).join(src_arg);

            if !src.exists() {
                return Ok(Output::error(
                    1,
                    out,
                    format!("cp: cannot stat '{}': No such file or directory\n", src_arg),
                ));
            }

            let final_dst = if dst.is_dir() {
                dst.join(src.file_name().unwrap())
            } else {
                dst.clone()
            };

            if src.is_dir() {
                if !recursive {
                    return Ok(Output::error(
                        1,
                        out,
                        format!("cp: -r not specified; omitting directory '{}'\n", src_arg),
                    ));
                }

                if let Err(e) = self.copy_dir(&src, &final_dst, force, verbose, &mut out) {
                    return Ok(Output::error(
                        1,
                        out,
                        format!("cp: cannot copy directory '{}': {}\n", src_arg, e),
                    ));
                }
            } else {
                if final_dst.exists() && force {
                    let _ = fs::remove_file(&final_dst);
                }

                if let Err(e) = fs::copy(&src, &final_dst) {
                    return Ok(Output::error(
                        1,
                        out,
                        format!("cp: cannot copy '{}': {}\n", src_arg, e),
                    ));
                } else if verbose {
                    out.push_str(&format!("'{}' -> '{}'\n", src_arg, final_dst.display()));
                }
            }
        }
        Ok(Output::success(out))
    }
}

impl CpCommand {
    fn copy_dir(
        &self,
        src: &PathBuf,
        dst: &PathBuf,
        force: bool,
        verbose: bool,
        out: &mut String,
    ) -> anyhow::Result<()> {
        if !dst.exists() {
            fs::create_dir_all(dst)?;
        }

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if file_type.is_dir() {
                self.copy_dir(&src_path, &dst_path, force, verbose, out)?;
            } else {
                if dst_path.exists() && force {
                    let _ = fs::remove_file(&dst_path);
                }
                fs::copy(&src_path, &dst_path)?;
                if verbose {
                    out.push_str(&format!(
                        "'{}' -> '{}'\n",
                        src_path.display(),
                        dst_path.display()
                    ));
                }
            }
        }
        Ok(())
    }
}
