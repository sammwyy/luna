use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::config::LunaConfig;
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;
use std::path::{Path, PathBuf};

pub struct MkdirCommand;

impl BuiltinCommand for MkdirCommand {
    fn name(&self) -> &'static str {
        "mkdir"
    }
    fn desc(&self) -> &'static str {
        "Create directories"
    }
    fn flags(&self) -> Vec<FlagDef> {
        vec![FlagDef {
            name: "parents",
            short: Some('p'),
            desc: "Create parent directories as needed",
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
        let parents = args.get_bool("parents");

        if args.positionals.is_empty() {
            return Ok(Output::error(
                1,
                "".into(),
                "mkdir: missing operand\n".into(),
            ));
        }

        for dir in &args.positionals {
            let path = PathBuf::from(ctx.get_cwd()).join(dir);
            let res = if parents {
                fs::create_dir_all(path)
            } else {
                fs::create_dir(path)
            };
            if let Err(e) = res {
                return Ok(Output::error(
                    1,
                    "".into(),
                    format!("mkdir: cannot create directory '{}': {}\n", dir, e),
                ));
            }
        }
        Ok(Output::success("".into()))
    }

    fn dry_run(&self, _config: &LunaConfig, args: &ParsedArgs) -> Result<(), String> {
        if args.positionals.is_empty() {
            return Err("mkdir: missing operand".to_string());
        }

        let parents = args.get_bool("parents");
        for dir in &args.positionals {
            let path = Path::new(dir);
            if parents {
                // With -p, we don't care if parents exist.
                // We only care if some component of the path is an existing FILE that is not a dir
                let mut current = PathBuf::new();
                for component in path.components() {
                    current.push(component);
                    if current.exists() && !current.is_dir() {
                        return Err(format!(
                            "mkdir: cannot create directory '{}': File exists",
                            dir
                        ));
                    }
                }
            } else {
                // Without -p, parent must exist and be a dir
                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() && (!parent.exists() || !parent.is_dir()) {
                        return Err(format!(
                            "mkdir: cannot create directory '{}': No such file or directory",
                            dir
                        ));
                    }
                }
                if path.exists() {
                    return Err(format!(
                        "mkdir: cannot create directory '{}': File exists",
                        dir
                    ));
                }
            }
        }
        Ok(())
    }
}
