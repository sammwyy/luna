use crate::renderer::markup;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write};

pub struct MenuOption {
    pub label: String,
    pub command: Option<String>,
}

pub fn show_correction_menu(options: Vec<MenuOption>) -> io::Result<Option<String>> {
    let mut selected = 0;

    enable_raw_mode()?;

    let result = loop {
        // Render menu
        let mut menu_output = String::new();
        menu_output.push_str("\r\n<#E5E510>Command not found. Did you mean?</#E5E510>\n");

        for (i, opt) in options.iter().enumerate() {
            let prefix = if i == selected { "> " } else { "  " };
            let style = if i == selected { "<bold>" } else { "" };
            let close = if i == selected { "</bold>" } else { "" };
            menu_output.push_str(&format!("\r{}{}{}{}\n", prefix, style, opt.label, close));
        }

        let ansi = markup::render_ansi(&menu_output);
        print!("{}", ansi);
        io::stdout().flush()?;

        // Move cursor back up to redraw next time
        let lines_to_move_up = options.len() + 2;
        print!("\x1b[{}A", lines_to_move_up);

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected < options.len() - 1 {
                        selected += 1;
                    }
                }
                KeyCode::Enter => {
                    break Ok(options[selected].command.clone());
                }
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('n') | KeyCode::Char('1') => {
                    break Ok(None);
                }
                _ => {}
            }
        }
    };

    // Clean up menu lines
    for _ in 0..options.len() + 2 {
        print!("\r\x1b[J\n");
    }
    print!("\x1b[{}A", options.len() + 2);
    io::stdout().flush()?;

    disable_raw_mode()?;
    result
}
