use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::setup;
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;

pub struct HistoryCommand;

impl BuiltinCommand for HistoryCommand {
    fn name(&self) -> &'static str {
        "history"
    }
    fn desc(&self) -> &'static str {
        "Show command history"
    }
    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        _args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        let hist_file = setup::history_file();
        match fs::read_to_string(&hist_file) {
            Ok(content) => {
                let mut out = String::new();
                for (i, line) in content.lines().enumerate() {
                    out.push_str(&format!(
                        "<color_border>{:5}</color_border>  <color_text>{}</color_text>\n",
                        i + 1,
                        line
                    ));
                }
                Ok(Output::success(out))
            }
            Err(_) => Ok(Output::success("".into())),
        }
    }
}
