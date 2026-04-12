use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;

pub struct WhichCommand;

impl BuiltinCommand for WhichCommand {
    fn name(&self) -> &'static str {
        "which"
    }

    fn desc(&self) -> &'static str {
        "Locate a command"
    }

    fn flags(&self) -> Vec<FlagDef> {
        vec![FlagDef {
            name: "all",
            short: Some('a'),
            desc: "print all matching executables in PATH, not just the first",
            flag_type: FlagType::Bool,
            required: false,
        }]
    }

    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        if args.positionals.is_empty() {
            return Ok(Output::error(
                1,
                "".into(),
                "which: missing argument\n".into(),
            ));
        }

        let show_all = args.get_bool("all");
        let mut out = String::new();
        let mut exit_code = 0;

        for name in &args.positionals {
            let mut found = false;

            if _ctx.state.aliases.contains_key(name) {
                let target = &_ctx.state.aliases[name];
                out.push_str(&format!("<color_primary>{}</color_primary>: alias to <color_secondary>{}</color_secondary>\n", name, target));
                found = true;
                if !show_all {
                    continue;
                }
            }

            if _ctx.state.builtins.contains(name) {
                out.push_str(&format!(
                    "<color_primary>{}</color_primary>: luna built-in command\n",
                    name
                ));
                found = true;
                if !show_all {
                    continue;
                }
            }

            if show_all {
                if let Ok(paths) = which::which_all(name) {
                    for path in paths {
                        out.push_str(&format!(
                            "<color_secondary>{}</color_secondary>\n",
                            path.display()
                        ));
                        found = true;
                    }
                }
            } else if let Ok(path) = which::which(name) {
                out.push_str(&format!(
                    "<color_secondary>{}</color_secondary>\n",
                    path.display()
                ));
                found = true;
            }

            if !found {
                out.push_str(&format!("<color_error>{}: not found</color_error>\n", name));
                exit_code = 1;
            }
        }
        Ok(Output::new(exit_code, out, "".into()))
    }
}
