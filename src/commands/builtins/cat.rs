use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::config::LunaConfig;
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::util::as_24_bit_terminal_escaped;

pub struct CatCommand;

impl BuiltinCommand for CatCommand {
    fn name(&self) -> &'static str {
        "cat"
    }
    fn desc(&self) -> &'static str {
        "Read files and print to stdout with syntax highlight"
    }
    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        let mut out = String::new();
        let ps = &ctx.state.syntax_set;
        let ts = &ctx.state.theme_set;
        let theme = &ts.themes["base16-ocean.dark"];

        for file in args.positionals {
            let path = Path::new(&file);
            if !path.exists() {
                return Ok(Output::error(
                    1,
                    out,
                    format!("cat: {}: No such file\n", file),
                ));
            }
            if path.is_dir() {
                return Ok(Output::error(
                    1,
                    out,
                    format!("cat: {}: Is a directory\n", file),
                ));
            }

            match fs::read_to_string(&file) {
                Ok(content) => {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let highlight_enabled = ctx.state.config.cat_highlight();
                    let allowed_exts = ctx.state.config.cat_highlight_exts();

                    if highlight_enabled && allowed_exts.contains(&ext.to_string()) {
                        let syntax = ps
                            .find_syntax_by_extension(ext)
                            .unwrap_or_else(|| ps.find_syntax_plain_text());

                        let mut h = HighlightLines::new(syntax, theme);
                        for line in content.lines() {
                            let ranges = h.highlight_line(line, ps).unwrap();
                            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                            out.push_str(&escaped);
                            out.push_str("\x1b[0m\n");
                        }
                    } else {
                        out.push_str(&content);
                        if !content.ends_with('\n') {
                            out.push('\n');
                        }
                    }
                }
                Err(e) => return Ok(Output::error(1, out, format!("cat: {}: {}\n", file, e))),
            }
        }
        Ok(Output::success(out))
    }

    fn dry_run(&self, _config: &LunaConfig, args: &ParsedArgs) -> Result<(), String> {
        for file in &args.positionals {
            let path = Path::new(file);
            if !path.exists() {
                return Err(format!("cat: {}: No such file", file));
            }
        }
        Ok(())
    }
}
