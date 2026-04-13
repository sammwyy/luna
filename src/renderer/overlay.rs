use crate::renderer::markup;
use crossterm::{cursor, terminal};

pub struct OverlayContext {
    pub _term_width: u16,
    pub _term_height: u16,
    pub _cursor_x: u16,
    pub _cursor_y: u16,
    pub _prompt_height: usize,
}

impl OverlayContext {
    pub fn new(line: &str) -> Self {
        let (term_width, term_height) = terminal::size().unwrap_or((80, 23));
        let (cursor_x, cursor_y) = cursor::position().unwrap_or((0, 0));
        let prompt_height = if line.contains('\n') {
            line.split('\n').count()
        } else {
            1
        };

        Self {
            _term_width: term_width,
            _term_height: term_height,
            _cursor_x: cursor_x,
            _cursor_y: cursor_y,
            _prompt_height: prompt_height,
        }
    }
}

pub enum OverlayPosition {
    Inline,
    Block,
}

pub trait OverlayComponent {
    fn render(&self, ctx: &OverlayContext) -> String;
    fn position(&self) -> OverlayPosition;
}

pub struct SuggestionBox {
    pub items: Vec<String>,
}

impl OverlayComponent for SuggestionBox {
    fn position(&self) -> OverlayPosition {
        OverlayPosition::Block
    }

    fn render(&self, _ctx: &OverlayContext) -> String {
        if self.items.is_empty() {
            return String::new();
        }

        let max_item_len = self.items.iter().map(|s| s.len()).max().unwrap_or(0);
        let width = max_item_len + 4;

        let mut out = String::new();
        out.push_str("\r\n");

        let mut box_markup = String::new();

        // Top line
        box_markup.push_str("<color_border>┌");
        for _ in 0..width {
            box_markup.push('─');
        }
        box_markup.push_str("┐</color_border>\r\n");

        for item in &self.items {
            box_markup.push_str("<color_border>│</color_border> ");
            box_markup.push_str("<color_secondary>");
            box_markup.push_str(item);
            box_markup.push_str("</color_secondary>");
            let padding = width - item.len() - 1;
            for _ in 0..padding {
                box_markup.push(' ');
            }
            box_markup.push_str("<color_border>│</color_border>\r\n");
        }

        // Bottom line
        box_markup.push_str("<color_border>└");
        for _ in 0..width {
            box_markup.push('─');
        }
        box_markup.push_str("┘</color_border>");

        out.push_str(&markup::render_ansi(&box_markup));
        out
    }
}

pub struct Tip {
    pub text: String,
    pub color_tag: String,
}

impl OverlayComponent for Tip {
    fn position(&self) -> OverlayPosition {
        OverlayPosition::Inline
    }

    fn render(&self, _ctx: &OverlayContext) -> String {
        if self.text.is_empty() {
            return String::new();
        }
        format!(
            "   \x1b[2m{}\x1b[0m",
            markup::render_ansi(&format!(
                "<{}>({})</{}>",
                self.color_tag, self.text, self.color_tag
            ))
        )
    }
}

pub struct OverlayManager {
    pub components: Vec<Box<dyn OverlayComponent>>,
}

impl OverlayManager {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn add(&mut self, component: Box<dyn OverlayComponent>) {
        self.components.push(component);
    }

    pub fn render_all(&self, line: &str) -> String {
        let ctx = OverlayContext::new(line);
        let mut out = String::new();

        // 1. Render all inline components normally
        for comp in &self.components {
            if let OverlayPosition::Inline = comp.position() {
                out.push_str(&comp.render(&ctx));
            }
        }

        // 2. Render all block components inside a zero-width ghost block
        let mut block_out = String::new();
        for comp in &self.components {
            if let OverlayPosition::Block = comp.position() {
                block_out.push_str(&comp.render(&ctx));
            }
        }

        // Always save, clear, draw blocks, restore to prevent hint ghosting
        out.push_str("\x1b[s\x1b[J");
        out.push_str(&block_out);
        out.push_str("\x1b[u");

        out
    }
}
