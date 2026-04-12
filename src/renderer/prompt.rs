use crate::lua::ThemeEngine;
use crate::renderer::markup;
use crate::shell::context::ShellContext;

pub fn render_prompt(theme: &Option<ThemeEngine>, context: &ShellContext) -> String {
    if let Some(eng) = theme {
        if let Some(rendered) = eng.render_prompt(context) {
            return markup::render_ansi(&rendered);
        }
        return "\x1b[33m[theme error]\x1b[0m > ".to_string();
    }
    "\x1b[33m[no theme]\x1b[0m > ".to_string()
}
