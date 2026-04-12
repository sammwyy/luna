use crate::commands::system::{BuiltinCommand, FlagDef, FlagType, ParsedArgs};
use crate::shell::config::LunaConfig;
use crate::shell::state::LunaState;
use shellframe::{Context, Output};
use std::path::Path;
use viuer::{print_from_file, Config};

pub struct ViewCommand;

impl BuiltinCommand for ViewCommand {
    fn name(&self) -> &'static str {
        "view"
    }

    fn desc(&self) -> &'static str {
        "View an image in the terminal"
    }

    fn flags(&self) -> Vec<FlagDef> {
        vec![
            FlagDef {
                name: "width",
                short: Some('w'),
                desc: "Set the width of the image",
                flag_type: FlagType::Integer,
                required: false,
            },
            FlagDef {
                name: "height",
                short: Some('h'),
                desc: "Set the height of the image",
                flag_type: FlagType::Integer,
                required: false,
            },
        ]
    }

    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        let path_str = match args.positionals.first() {
            Some(p) => p,
            None => {
                return Ok(Output::error(
                    1,
                    "".into(),
                    "Usage: view <path_to_image>\n".into(),
                ));
            }
        };

        let width = args.get_int("width").map(|w| w as u32);
        let height = args.get_int("height").map(|h| h as u32);

        let conf = Config {
            width,
            height,
            absolute_offset: false,
            ..Default::default()
        };

        println!();
        let result = print_from_file(path_str, &conf);
        println!();

        match result {
            Ok(_) => Ok(Output::success("".into())),
            Err(e) => Ok(Output::error(
                1,
                "".into(),
                format!("view: {}: {}\n", path_str, e),
            )),
        }
    }

    fn dry_run(&self, _ctx_config: &LunaConfig, args: &ParsedArgs) -> Result<(), String> {
        if let Some(path) = args.positionals.first() {
            if Path::new(path).exists() {
                Ok(())
            } else {
                Err(format!("view: {}: No such file", path))
            }
        } else {
            Err("view: missing path".to_string())
        }
    }
}
