use crate::lua::ThemeEngine;
use crate::renderer::markup;
use crate::shell::context::ShellContext;

pub fn render_error(theme: &Option<ThemeEngine>, context: &ShellContext, err: &str) -> String {
    if let Some(eng) = theme {
        if let Some(rendered) = eng.render_error(context, err) {
            let ansi = markup::render_ansi(&rendered);
            if ansi.ends_with('\n') {
                return ansi;
            } else {
                return format!("{}\n", ansi);
            }
        }
    }
    format!("\x1b[31merror:\x1b[0m {err}\n")
}
