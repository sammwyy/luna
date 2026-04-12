use crate::commands::system::{BuiltinCommand, FlagDef, ParsedArgs};
use crate::setup::{config_file, themes_dir};
use crate::shell::state::LunaState;
use shellframe::{Context, Output};
use std::fs;
use std::path::Path;

pub struct ThemeCommand;

impl BuiltinCommand for ThemeCommand {
    fn name(&self) -> &'static str {
        "theme"
    }
    fn desc(&self) -> &'static str {
        "Manage luna themes"
    }
    fn flags(&self) -> Vec<FlagDef> {
        vec![]
    }

    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        let sub = match args.positionals.get(0) {
            Some(s) => s.as_str(),
            None => {
                return Ok(Output::success(
                    "<color_primary><bold>luna theme manager</bold></color_primary>\n\n\
                      Usage:\n  \
                      theme list                    - List all themes in ~/.luna/themes\n  \
                      theme set <id>                - Set the active theme\n  \
                      theme install <path>          - Install a .lua theme file\n  \
                      theme remove <id>             - Remove a theme\n"
                        .to_string(),
                ));
            }
        };

        match sub {
            "list" => {
                let mut out = String::new();
                out.push_str("<color_primary><bold>Available themes:</bold></color_primary>\n");
                let dir = themes_dir();
                let current = ctx.state.config.theme.as_deref().unwrap_or("default.lua");

                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().into_owned();
                        if name.ends_with(".lua") {
                            let id = name.trim_end_matches(".lua");
                            let is_active = (name == current) || (id == current);

                            if is_active {
                                out.push_str(&format!(" <color_secondary>*</color_secondary> <color_secondary><bold>{}</bold></color_secondary> (active)\n", id));
                            } else {
                                out.push_str(&format!("   <color_text>{}</color_text>\n", id));
                            }
                        }
                    }
                }
                Ok(Output::success(out))
            }
            "set" => {
                let id = match args.positionals.get(1) {
                    Some(id) => id,
                    None => {
                        return Ok(Output::error(
                            1,
                            "".into(),
                            "Usage: theme set <id>\n".into(),
                        ))
                    }
                };

                let mut theme_name = id.clone();
                if !theme_name.ends_with(".lua") {
                    theme_name.push_str(".lua");
                }

                let theme_path = themes_dir().join(&theme_name);
                if !theme_path.exists() {
                    return Ok(Output::error(
                        1,
                        "".into(),
                        format!("Theme '{}' not found in ~/.luna/themes/\n", id),
                    ));
                }

                ctx.state.config.theme = Some(theme_name);
                let cfg_path = config_file();
                ctx.state.config.save(&cfg_path)?;
                ctx.state.theme_dirty = true;

                Ok(Output::success(format!(
                    "Theme set to <color_primary>{}</color_primary>. Reloading engine...\n",
                    id
                )))
            }
            "install" => {
                let path_str = match args.positionals.get(1) {
                    Some(p) => p,
                    None => {
                        return Ok(Output::error(
                            1,
                            "".into(),
                            "Usage: theme install <path_to_lua_file>\n".into(),
                        ))
                    }
                };
                let source = Path::new(path_str);
                if !source.exists() {
                    return Ok(Output::error(
                        1,
                        "".into(),
                        format!("File not found: {}\n", path_str),
                    ));
                }
                let name = source.file_name().unwrap().to_string_lossy();
                let target = themes_dir().join(name.as_ref());
                fs::copy(source, target)?;
                Ok(Output::success(format!(
                    "Theme <color_secondary>{}</color_secondary> installed to ~/.luna/themes/\n",
                    name
                )))
            }
            "remove" => {
                let id = match args.positionals.get(1) {
                    Some(id) => id,
                    None => {
                        return Ok(Output::error(
                            1,
                            "".into(),
                            "Usage: theme remove <id>\n".into(),
                        ))
                    }
                };
                if id == "default" {
                    return Ok(Output::error(
                        1,
                        "".into(),
                        "Cannot remove the default theme.\n".into(),
                    ));
                }

                let mut theme_name = id.clone();
                if !theme_name.ends_with(".lua") {
                    theme_name.push_str(".lua");
                }
                let target = themes_dir().join(&theme_name);
                if target.exists() {
                    fs::remove_file(target)?;
                    Ok(Output::success(format!(
                        "Theme <color_error>{}</color_error> removed.\n",
                        id
                    )))
                } else {
                    Ok(Output::error(
                        1,
                        "".into(),
                        format!("Theme '{}' not found.\n", id),
                    ))
                }
            }
            _ => Ok(Output::error(
                1,
                "".into(),
                format!("Unknown subcommand: {}\n", sub),
            )),
        }
    }
}
