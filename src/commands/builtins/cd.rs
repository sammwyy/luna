use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::config::LunaConfig;
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::env;

pub struct CdCommand;

impl BuiltinCommand for CdCommand {
    fn name(&self) -> &'static str {
        "cd"
    }

    fn desc(&self) -> &'static str {
        "Change the current directory"
    }

    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "logical",
                short: Some('L'),
                desc: "use logical path structure",
                flag_type: FlagType::Bool,
                required: false,
            },
            FlagDef {
                name: "physical",
                short: Some('P'),
                desc: "use physical path structure",
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
        let default_dir = if ctx.state.config.cd_home_default() {
            env::var("HOME").unwrap_or_else(|_| "/".to_string())
        } else {
            "/".to_string()
        };

        let arg = args.positionals.first().cloned();

        let target = match arg.as_deref() {
            Some("-") => {
                let prev = ctx.state.prev_cwd.clone();
                println!("{}", prev); // Bash prints the new directory when using 'cd -'
                prev
            }
            Some(t) => t.to_string(),
            None => default_dir,
        };

        let old_cwd = ctx.get_cwd().to_string();

        if env::set_current_dir(&target).is_ok() {
            if let Ok(new_cwd) = env::current_dir() {
                let new_cwd_str = new_cwd.to_string_lossy().to_string();
                ctx.set_cwd(new_cwd_str.clone());
                ctx.state.prev_cwd = old_cwd;
            }
            Ok(Output::success("".into()))
        } else {
            Ok(Output::error(
                1,
                "".into(),
                format!(
                    "<color_error>cd: {}: No such directory</color_error>\n",
                    target
                ),
            ))
        }
    }

    fn dry_run(&self, ctx_config: &LunaConfig, args: &ParsedArgs) -> Result<(), String> {
        let default_dir = if ctx_config.cd_home_default() {
            env::var("HOME").unwrap_or_else(|_| "/".to_string())
        } else {
            "/".to_string()
        };

        let arg = args.positionals.first().cloned();
        let target = match arg.as_deref() {
            Some("-") => return Ok(()), // Assume it works for dry run
            // We don't have access to LunaState here easily, so we just return Ok
            Some(t) => t.to_string(),
            None => default_dir,
        };

        if std::path::Path::new(&target).exists() {
            Ok(())
        } else {
            Err(format!("cd: {}: No such directory", target))
        }
    }
}
