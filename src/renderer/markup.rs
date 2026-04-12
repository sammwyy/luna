/// Rich text parser for luna.
///
/// Supported syntax:
///   <red>text</red>           — named color (foreground)
///   <#ff4444>text</#ff4444>   — hex foreground color
///   <#ff4444>text</color>     — </color> closes any open color
///   <bg:red>text</bg:red>     — named background color
///   <bg:#ff4444>text          — hex background color
///   <reset>                   — full reset
///   <bold>text</bold>
///   <italic>text</italic>
///   <underline>text</underline>
///   <strike>text</strike>
///   <gradient from=#rrggbb to=#rrggbb>text</gradient>
///
/// Colors can be left open — they just stay active.
/// </color> pops the last pushed foreground color off the stack
/// (reverts to the previous one).

// ─── ANSI helpers ────────────────────────────────────────────────────────────

fn ansi_fg_rgb(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{};{};{}m", r, g, b)
}

fn ansi_bg_rgb(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[48;2;{};{};{}m", r, g, b)
}

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_BOLD: &str = "\x1b[1m";
const ANSI_ITALIC: &str = "\x1b[3m";
const ANSI_UNDERLINE: &str = "\x1b[4m";
const ANSI_STRIKE: &str = "\x1b[9m";
const ANSI_BOLD_OFF: &str = "\x1b[22m";
const ANSI_ITALIC_OFF: &str = "\x1b[23m";
const ANSI_UNDERLINE_OFF: &str = "\x1b[24m";
const ANSI_STRIKE_OFF: &str = "\x1b[29m";

// ─── Named colour palette ─────────────────────────────────────────────────────

fn named_color(name: &str) -> Option<(u8, u8, u8)> {
    match name.to_ascii_lowercase().as_str() {
        "black" => Some((0, 0, 0)),
        "red" => Some((205, 49, 49)),
        "green" => Some((13, 188, 121)),
        "yellow" => Some((229, 229, 16)),
        "blue" => Some((36, 114, 200)),
        "magenta" => Some((188, 63, 188)),
        "cyan" => Some((17, 168, 205)),
        "white" => Some((229, 229, 229)),
        "orange" => Some((255, 165, 0)),
        "pink" => Some((255, 105, 180)),
        "purple" => Some((148, 0, 211)),
        "gray" | "grey" => Some((128, 128, 128)),
        "brightred" => Some((241, 76, 76)),
        "brightgreen" => Some((35, 209, 139)),
        "brightyellow" => Some((245, 245, 67)),
        "brightblue" => Some((59, 142, 234)),
        "brightmagenta" => Some((214, 112, 214)),
        "brightcyan" => Some((41, 184, 219)),
        "brightwhite" => Some((255, 255, 255)),
        _ => None,
    }
}

// ─── Hex parsing ─────────────────────────────────────────────────────────────

fn parse_hex_color(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.trim_start_matches('#');
    let s = if s.len() == 3 {
        // Expand short form: abc → aabbcc
        format!(
            "{}{}{}{}{}{}",
            &s[0..1],
            &s[0..1],
            &s[1..2],
            &s[1..2],
            &s[2..3],
            &s[2..3],
        )
    } else {
        s.to_string()
    };
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some((r, g, b))
}

// ─── Gradient ────────────────────────────────────────────────────────────────

fn lerp_color(from: (u8, u8, u8), to: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    let r = (from.0 as f64 + (to.0 as f64 - from.0 as f64) * t).round() as u8;
    let g = (from.1 as f64 + (to.1 as f64 - from.1 as f64) * t).round() as u8;
    let b = (from.2 as f64 + (to.2 as f64 - from.2 as f64) * t).round() as u8;
    (r, g, b)
}

fn render_gradient(text: &str, from: (u8, u8, u8), to: (u8, u8, u8)) -> String {
    // Count visible (non-whitespace-only) chars for step calculation.
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len().max(1);
    let mut out = String::new();
    for (i, ch) in chars.iter().enumerate() {
        let t = if len == 1 {
            0.0
        } else {
            i as f64 / (len - 1) as f64
        };
        let (r, g, b) = lerp_color(from, to, t);
        out.push_str(&ansi_fg_rgb(r, g, b));
        out.push(*ch);
    }
    out
}

// ─── Tag representation ───────────────────────────────────────────────────────

#[derive(Debug)]
enum Token {
    Text(String),
    OpenTag(Tag),
    CloseTag(CloseKind),
}

#[derive(Debug)]
enum CloseKind {
    Named(String), // </red>, </bold>, </gradient>, </#ff4444>
    Color,         // </color>  — closes any open fg colour
    Bg,            // </bg:red>
}

#[derive(Debug)]
enum Tag {
    Fg(u8, u8, u8),
    Bg(u8, u8, u8),
    Reset,
    Bold,
    Italic,
    Underline,
    Strike,
    Gradient {
        from: (u8, u8, u8),
        to: (u8, u8, u8),
    },
}

// ─── Tokeniser ────────────────────────────────────────────────────────────────

fn tokenise(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut buf = String::new();

    while i < chars.len() {
        if chars[i] == '<' {
            // find closing '>'
            if let Some(end) = chars[i..].iter().position(|&c| c == '>') {
                let end = i + end;
                let inner: String = chars[i + 1..end].iter().collect();

                if !buf.is_empty() {
                    tokens.push(Token::Text(std::mem::take(&mut buf)));
                }

                if let Some(tok) = parse_tag(&inner) {
                    tokens.push(tok);
                } else {
                    // Not a recognised tag — treat as literal text.
                    buf.push('<');
                    buf.push_str(&inner);
                    buf.push('>');
                }
                i = end + 1;
                continue;
            }
        }
        buf.push(chars[i]);
        i += 1;
    }
    if !buf.is_empty() {
        tokens.push(Token::Text(buf));
    }
    tokens
}

fn parse_tag(inner: &str) -> Option<Token> {
    let inner = inner.trim();

    // ── Closing tags ──────────────────────────────────────────────────────────
    if inner.starts_with('/') {
        let name = inner[1..].trim();
        if name.eq_ignore_ascii_case("color") || name.eq_ignore_ascii_case("colour") {
            return Some(Token::CloseTag(CloseKind::Color));
        }
        if name.starts_with("bg:") || name.starts_with("bg/") || name.eq_ignore_ascii_case("bg") {
            return Some(Token::CloseTag(CloseKind::Bg));
        }
        return Some(Token::CloseTag(CloseKind::Named(name.to_string())));
    }

    // ── Self-closing / open tags ──────────────────────────────────────────────

    // <reset>
    if inner.eq_ignore_ascii_case("reset") {
        return Some(Token::OpenTag(Tag::Reset));
    }

    // <bold>, <italic>, <underline>, <strike>
    match inner.to_ascii_lowercase().as_str() {
        "bold" => return Some(Token::OpenTag(Tag::Bold)),
        "italic" => return Some(Token::OpenTag(Tag::Italic)),
        "underline" => return Some(Token::OpenTag(Tag::Underline)),
        "strike" | "strikethrough" | "stroke" => return Some(Token::OpenTag(Tag::Strike)),
        _ => {}
    }

    // <gradient from=#aabbcc to=#ddeeff>
    if inner.to_ascii_lowercase().starts_with("gradient") {
        let attrs = &inner["gradient".len()..].trim();
        let from_col = extract_attr(attrs, "from")?;
        let to_col = extract_attr(attrs, "to")?;
        let from = resolve_color(&from_col)?;
        let to = resolve_color(&to_col)?;
        return Some(Token::OpenTag(Tag::Gradient { from, to }));
    }

    // <bg:red> or <bg:#ff4444>
    if inner.to_ascii_lowercase().starts_with("bg:")
        || inner.to_ascii_lowercase().starts_with("bg/")
    {
        let color_str = inner[3..].trim();
        let (r, g, b) = resolve_color(color_str)?;
        return Some(Token::OpenTag(Tag::Bg(r, g, b)));
    }

    // <red>, <#ff4444>  — foreground colour
    let (r, g, b) = resolve_color(inner)?;
    Some(Token::OpenTag(Tag::Fg(r, g, b)))
}

use std::collections::HashMap;
use std::sync::Mutex;

static THEME_VARS_MUTEX: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

pub fn update_theme_vars(vars: HashMap<String, String>) {
    let mut guard = THEME_VARS_MUTEX.lock().unwrap();
    *guard = Some(vars);
}

fn get_theme_var(name: &str) -> Option<String> {
    let guard = THEME_VARS_MUTEX.lock().unwrap();
    guard.as_ref()?.get(name).cloned()
}

fn resolve_color(s: &str) -> Option<(u8, u8, u8)> {
    if s.starts_with('#') {
        parse_hex_color(s)
    } else {
        // 1. Try exact theme var match (e.g. "color_primary")
        if let Some(val) = get_theme_var(s) {
            if let Some(c) = parse_hex_color(&val) {
                return Some(c);
            }
            if let Some(c) = named_color(&val) {
                return Some(c);
            }
        }

        // 2. Try with "color_" prefix if not already there
        if !s.starts_with("color_") {
            let prefixed = format!("color_{}", s);
            if let Some(val) = get_theme_var(&prefixed) {
                if let Some(c) = parse_hex_color(&val) {
                    return Some(c);
                }
                if let Some(c) = named_color(&val) {
                    return Some(c);
                }
            }
        }

        // 3. Fallback to standard named colors
        named_color(s)
    }
}

/// Extract `key=value` from an attribute string.
fn extract_attr(attrs: &str, key: &str) -> Option<String> {
    let pattern = format!("{}=", key);
    let pos = attrs.to_ascii_lowercase().find(&pattern)?;
    let rest = &attrs[pos + pattern.len()..];
    // value ends at whitespace or end of string
    let val: String = rest.chars().take_while(|c| !c.is_whitespace()).collect();
    Some(val)
}

// ─── Renderer ─────────────────────────────────────────────────────────────────

/// State tracked while walking the token stream.
#[derive(Default)]
struct RenderState {
    /// Stack of active foreground colours (RGB).  None means "default".
    fg_stack: Vec<Option<(u8, u8, u8)>>,
    /// Stack of active background colours.
    bg_stack: Vec<Option<(u8, u8, u8)>>,
    bold: bool,
    italic: bool,
    underline: bool,
    strike: bool,
    /// Are we currently inside a <gradient>…</gradient> block?
    gradient: Option<((u8, u8, u8), (u8, u8, u8))>,
    /// Buffer collecting text inside gradient.
    grad_buf: String,
}

impl RenderState {
    fn reapply_fg(&self) -> String {
        match self.fg_stack.last() {
            Some(Some((r, g, b))) => ansi_fg_rgb(*r, *g, *b),
            Some(None) | None => "\x1b[39m".to_string(), // default fg
        }
    }

    fn reapply_bg(&self) -> String {
        match self.bg_stack.last() {
            Some(Some((r, g, b))) => ansi_bg_rgb(*r, *g, *b),
            Some(None) | None => "\x1b[49m".to_string(), // default bg
        }
    }
}

/// Convert luna rich-text markup to ANSI escape sequences.
pub fn render_ansi(input: &str) -> String {
    let tokens = tokenise(input);
    let mut out = String::new();
    let mut st = RenderState::default();

    for tok in tokens {
        match tok {
            Token::Text(text) => {
                if let Some((from, to)) = st.gradient {
                    st.grad_buf.push_str(&text);
                    // We accumulate and will render on </gradient>
                    let _ = (from, to); // suppress unused warning
                } else {
                    out.push_str(&text);
                }
            }

            Token::OpenTag(tag) => match tag {
                Tag::Reset => {
                    // Flush gradient if open
                    if let Some((from, to)) = st.gradient.take() {
                        out.push_str(&render_gradient(&st.grad_buf, from, to));
                        st.grad_buf.clear();
                    }
                    out.push_str(ANSI_RESET);
                    st.fg_stack.clear();
                    st.bg_stack.clear();
                    st.bold = false;
                    st.italic = false;
                    st.underline = false;
                    st.strike = false;
                }
                Tag::Fg(r, g, b) => {
                    st.fg_stack.push(Some((r, g, b)));
                    out.push_str(&ansi_fg_rgb(r, g, b));
                }
                Tag::Bg(r, g, b) => {
                    st.bg_stack.push(Some((r, g, b)));
                    out.push_str(&ansi_bg_rgb(r, g, b));
                }
                Tag::Bold => {
                    st.bold = true;
                    out.push_str(ANSI_BOLD);
                }
                Tag::Italic => {
                    st.italic = true;
                    out.push_str(ANSI_ITALIC);
                }
                Tag::Underline => {
                    st.underline = true;
                    out.push_str(ANSI_UNDERLINE);
                }
                Tag::Strike => {
                    st.strike = true;
                    out.push_str(ANSI_STRIKE);
                }
                Tag::Gradient { from, to } => {
                    st.gradient = Some((from, to));
                    st.grad_buf.clear();
                }
            },

            Token::CloseTag(kind) => {
                match kind {
                    CloseKind::Color => {
                        st.fg_stack.pop();
                        out.push_str(&st.reapply_fg());
                    }
                    CloseKind::Named(n) => {
                        let lower = n.to_ascii_lowercase();
                        match lower.as_str() {
                            // ── Attribute closes ──────────────────────────────
                            "bold" => {
                                st.bold = false;
                                out.push_str(ANSI_BOLD_OFF);
                            }
                            "italic" => {
                                st.italic = false;
                                out.push_str(ANSI_ITALIC_OFF);
                            }
                            "underline" => {
                                st.underline = false;
                                out.push_str(ANSI_UNDERLINE_OFF);
                            }
                            "strike" | "strikethrough" | "stroke" => {
                                st.strike = false;
                                out.push_str(ANSI_STRIKE_OFF);
                            }
                            "gradient" => {
                                if let Some((from, to)) = st.gradient.take() {
                                    out.push_str(&render_gradient(&st.grad_buf, from, to));
                                    st.grad_buf.clear();
                                    out.push_str(&st.reapply_fg());
                                }
                            }
                            // ── Colour closes  ────────────────────────────────
                            _ => {
                                // Pop the matching fg colour (if any), else pop top
                                if st.fg_stack.last().map_or(false, |c| {
                                    if let Some(col) = c {
                                        if let Some(nc) = named_color(&n) {
                                            nc == *col
                                        } else if let Some(hc) = parse_hex_color(&n) {
                                            hc == *col
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                }) {
                                    st.fg_stack.pop();
                                } else {
                                    st.fg_stack.pop();
                                }
                                out.push_str(&st.reapply_fg());
                            }
                        }
                    }
                    CloseKind::Bg => {
                        st.bg_stack.pop();
                        out.push_str(&st.reapply_bg());
                    }
                }
            }
        }
    }

    // Flush any unclosed gradient
    if let Some((from, to)) = st.gradient.take() {
        out.push_str(&render_gradient(&st.grad_buf, from, to));
    }

    out
}

/// Strip all luna rich-text tags (returns plain text).
pub fn strip_ansi(input: &str) -> String {
    let tokens = tokenise(input);
    let mut out = String::new();
    let mut in_gradient = false;
    let mut grad_buf = String::new();

    for tok in tokens {
        match tok {
            Token::Text(t) => {
                if in_gradient {
                    grad_buf.push_str(&t);
                } else {
                    out.push_str(&t);
                }
            }
            Token::OpenTag(Tag::Gradient { .. }) => {
                in_gradient = true;
                grad_buf.clear();
            }
            Token::CloseTag(CloseKind::Named(ref n)) if n.eq_ignore_ascii_case("gradient") => {
                in_gradient = false;
                out.push_str(&grad_buf);
                grad_buf.clear();
            }
            _ => {} // ignore all formatting tags
        }
    }

    if in_gradient {
        out.push_str(&grad_buf);
    }
    out
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_named_color() {
        let out = render_ansi("<red>hello</red> world");
        assert!(out.contains("hello"));
        assert!(out.contains("\x1b[38;2;"));
    }

    #[test]
    fn test_hex_color() {
        let out = render_ansi("<#ff0000>hello</color>");
        assert!(out.contains("hello"));
    }

    #[test]
    fn test_gradient() {
        let out = render_ansi("<gradient from=#ff0000 to=#0000ff>Hello</gradient>");
        // Should have multiple ANSI codes
        assert!(out.contains("\x1b[38;2;"));
    }

    #[test]
    fn test_strip() {
        let plain =
            strip_ansi("<red>hello</red> <gradient from=#ff0000 to=#0000ff>world</gradient>");
        assert_eq!(plain, "hello world");
    }

    #[test]
    fn test_bold_italic() {
        let out = render_ansi("<bold><italic>wow</italic></bold>");
        assert!(out.contains(ANSI_BOLD));
        assert!(out.contains(ANSI_ITALIC));
    }
}
