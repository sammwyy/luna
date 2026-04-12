pub mod builtins;
pub mod system;

use crate::shell::state::LunaState;
use shellframe::{Context, Output, Shell};
use std::sync::Arc;
use system::{BuiltinCommand, ParsedArgs, Registry};

pub struct HelpCommand {
    help_main: String,
    help_details: std::collections::HashMap<String, String>,
}

impl BuiltinCommand for HelpCommand {
    fn name(&self) -> &'static str {
        "help"
    }
    fn desc(&self) -> &'static str {
        "Show help for commands"
    }
    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        if let Some(cmd) = args.positionals.first() {
            if let Some(detail) = self.help_details.get(cmd) {
                Ok(Output::success(format!("{}\n", detail)))
            } else {
                Ok(Output::error(
                    1,
                    "".into(),
                    format!("help: no such command: {}\n", cmd),
                ))
            }
        } else {
            Ok(Output::success(format!("{}\n", self.help_main)))
        }
    }
}

pub fn register_all(shell: &mut Shell<LunaState>) -> Arc<Registry> {
    let mut registry = Registry::new();

    let config = shell.context.state.config.clone();

    let all_builtins: Vec<Arc<dyn BuiltinCommand>> = vec![
        Arc::new(builtins::alias::AliasCommand),
        Arc::new(builtins::back::BackCommand),
        Arc::new(builtins::cat::CatCommand),
        Arc::new(builtins::cd::CdCommand),
        Arc::new(builtins::clear::ClearCommand),
        Arc::new(builtins::cp::CpCommand),
        Arc::new(builtins::cut::CutCommand),
        Arc::new(builtins::echo::EchoCommand),
        Arc::new(builtins::env::EnvCommand),
        Arc::new(builtins::exit::ExitCommand),
        Arc::new(builtins::file::FileCommand),
        Arc::new(builtins::find::FindCommand),
        Arc::new(builtins::grep::GrepCommand),
        Arc::new(builtins::head::HeadCommand),
        Arc::new(builtins::history::HistoryCommand),
        Arc::new(builtins::jq::JqCommand),
        Arc::new(builtins::ls::LsCommand),
        Arc::new(builtins::mkdir::MkdirCommand),
        Arc::new(builtins::has::HasCommand),
        Arc::new(builtins::mv::MvCommand),
        Arc::new(builtins::next::NextCommand),
        Arc::new(builtins::pwd::PwdCommand),
        Arc::new(builtins::rm::RmCommand),
        Arc::new(builtins::sed::SedCommand),
        Arc::new(builtins::sort::SortCommand),
        Arc::new(builtins::stat::StatCommand),
        Arc::new(builtins::tail::TailCommand),
        Arc::new(builtins::test_false::FalseCommand),
        Arc::new(builtins::test_true::TrueCommand),
        Arc::new(builtins::touch::TouchCommand),
        Arc::new(builtins::tree::TreeCommand),
        Arc::new(builtins::theme::ThemeCommand),
        Arc::new(builtins::uniq::UniqCommand),
        Arc::new(builtins::wc::WcCommand),
        Arc::new(builtins::which::WhichCommand),
    ];

    for builtin in all_builtins {
        if config.is_builtin_enabled(builtin.name()) {
            registry.register(builtin);
        }
    }

    let help_main = {
        let mut text = String::new();
        text.push_str("<color_primary><bold>luna built-in commands:</bold></color_primary>\n");
        for (name, cmd) in &registry.commands {
            text.push_str(&format!(
                "  <color_secondary>{:<10}</color_secondary> <color_text>{}</color_text>\n",
                name,
                cmd.desc()
            ));
        }
        text
    };

    let mut help_details = std::collections::HashMap::new();
    for (name, cmd) in &registry.commands {
        let mut detail = format!("<color_primary>Usage:</color_primary> <color_secondary>{}</color_secondary>\n\n<color_text>{}</color_text>", name, cmd.desc());
        let flags = cmd.flags();
        if !flags.is_empty() {
            detail.push_str("\n\n<color_primary><bold>Flags:</bold></color_primary>\n");
            for f in &flags {
                let short = f.short.map_or("".to_string(), |c| format!("-{}, ", c));
                let ftype = match &f.flag_type {
                    system::FlagType::Bool => "BOOL".to_string(),
                    system::FlagType::String => "STRING".to_string(),
                    system::FlagType::Integer => "INTEGER".to_string(),
                    system::FlagType::Enum(v) => format!("ENUM {:?}", v),
                };
                detail.push_str(&format!(
                    "  {}--{} <<#9ca3af>{}</#9ca3af>>    <color_text>{}</color_text>\n",
                    short, f.name, ftype, f.desc
                ));
            }
        }
        help_details.insert(name.clone(), detail);
    }

    registry.register(Arc::new(HelpCommand {
        help_main,
        help_details,
    }));

    let arc_registry = Arc::new(registry);

    for (name, cmd) in arc_registry.commands.iter() {
        let cmd_clone = cmd.clone();
        shell.register_builtin(name, move |args, ctx, stdin| {
            let mut parsed = match cmd_clone.parse_args(args) {
                Ok(p) => p,
                Err(e) => {
                    return Ok(Output::error(
                        1,
                        "".into(),
                        format!("{}: {}\n", cmd_clone.name(), e),
                    ))
                }
            };

            let cmd_name = cmd_clone.name();
            let always_expand = [
                "mkdir", "rmdir", "touch", "cat", "cp", "mv", "rm", "file", "ls",
            ];

            if ctx.state.config.universal_multi_file_parsing() || always_expand.contains(&cmd_name)
            {
                parsed.positionals =
                    crate::shell::utils::expand_paths(&parsed.positionals, ctx.get_cwd());
            }

            cmd_clone.run(ctx, parsed, stdin)
        });
    }

    arc_registry
}
