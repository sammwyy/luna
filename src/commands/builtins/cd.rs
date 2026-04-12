use crate::commands::system::{BuiltinCommand, ParsedArgs};
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

        let target = args
            .positionals
            .first()
            .cloned()
            .unwrap_or_else(|| default_dir);

        if env::set_current_dir(&target).is_ok() {
            if let Ok(new_cwd) = env::current_dir() {
                ctx.set_cwd(new_cwd.to_string_lossy().to_string());
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

        let target = args
            .positionals
            .first()
            .cloned()
            .unwrap_or_else(|| default_dir);

        if std::path::Path::new(&target).exists() {
            Ok(())
        } else {
            Err(format!("cd: {}: No such directory", target))
        }
    }
}
