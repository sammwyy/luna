use crate::commands::system::{BuiltinCommand, FlagDef, ParsedArgs};
use crate::renderer::markup;
use crate::shell::config::LunaConfig;
use crate::shell::state::LunaState;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use shellframe::{Context, Output};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub struct CdiCommand;

impl BuiltinCommand for CdiCommand {
    fn name(&self) -> &'static str {
        "cdi"
    }

    fn desc(&self) -> &'static str {
        "Interactive directory navigation"
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
        let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());

        let target = args
            .positionals
            .first()
            .cloned()
            .unwrap_or_else(|| home.clone());

        let start_dir = PathBuf::from(target);
        if !start_dir.exists() || !start_dir.is_dir() {
            return Ok(Output::error(
                1,
                "".into(),
                format!(
                    "<color_error>cdi: {}: No such directory</color_error>\n",
                    start_dir.display()
                ),
            ));
        }

        let abs_start = fs::canonicalize(start_dir).unwrap_or_else(|_| PathBuf::from("."));

        if let Some(final_path) = self.interactive_nav(&abs_start)? {
            if env::set_current_dir(&final_path).is_ok() {
                if let Ok(new_cwd) = env::current_dir() {
                    ctx.set_cwd(new_cwd.to_string_lossy().to_string());
                }
                Ok(Output::success("".into()))
            } else {
                Ok(Output::error(
                    1,
                    "".into(),
                    format!(
                        "<color_error>cdi: Failed to change directory to {}</color_error>\n",
                        final_path.display()
                    ),
                ))
            }
        } else {
            Ok(Output::success("".into()))
        }
    }

    fn dry_run(&self, _ctx_config: &LunaConfig, args: &ParsedArgs) -> Result<(), String> {
        let target = args.positionals.first().map(|s| s.as_str()).unwrap_or(".");
        if Path::new(target).exists() {
            Ok(())
        } else {
            Err(format!("cdi: {}: No such directory", target))
        }
    }
}

impl CdiCommand {
    fn interactive_nav(&self, start_dir: &Path) -> io::Result<Option<PathBuf>> {
        let mut current_dir = start_dir.to_path_buf();
        let mut selected = 0;
        let mut last_rendered_lines = 0;
        let max_visible = 15;

        enable_raw_mode()?;
        let mut stdout = io::stdout();

        let result = loop {
            let entries = self.get_entries(&current_dir);
            if selected >= entries.len() {
                selected = entries.len().saturating_sub(1);
            }

            // Clean previous render
            if last_rendered_lines > 0 {
                for _ in 0..last_rendered_lines {
                    execute!(
                        stdout,
                        cursor::MoveUp(1),
                        terminal::Clear(terminal::ClearType::CurrentLine)
                    )?;
                }
            }

            // Compute scrolling
            let start_idx = if selected >= max_visible {
                selected - max_visible + 1
            } else {
                0
            };
            let end_idx = (start_idx + max_visible).min(entries.len());

            // Header
            let mut output = format!("\r<color_primary><bold>Navigating:</bold></color_primary> <color_secondary>{}</color_secondary>\n", current_dir.display());
            let mut line_count = 1;

            if start_idx > 0 {
                output.push_str("\r  <gray>... (more above)</gray>\n");
                line_count += 1;
            }

            // List
            for i in start_idx..end_idx {
                let entry = &entries[i];
                let prefix = if i == selected { "> " } else { "  " };
                let style = if i == selected {
                    "<bold><color_secondary>"
                } else {
                    "<color_text>"
                };
                let close = if i == selected {
                    "</color_secondary></bold>"
                } else {
                    "</color_text>"
                };

                output.push_str(&format!("\r{}{}{}{}\n", prefix, style, entry, close));
                line_count += 1;
            }

            if end_idx < entries.len() {
                output.push_str("\r  <gray>... (more below)</gray>\n");
                line_count += 1;
            }

            // Preview
            output.push_str("\r<gray>─────────────────────────────────</gray>\n");
            line_count += 1;

            let selected_entry = &entries[selected];
            let preview_path = if selected_entry == "." {
                current_dir.clone()
            } else if selected_entry == ".." {
                current_dir.parent().unwrap_or(&current_dir).to_path_buf()
            } else {
                current_dir.join(selected_entry)
            };

            let preview = self.get_preview(&preview_path);
            output.push_str(&preview);
            line_count += preview.lines().count();

            // Hints
            output.push_str("\r<gray>─────────────────────────────────</gray>\n");
            output.push_str("\r<gray>↑/↓: navigate • Enter: select/open • Esc: cancel</gray>\n");
            line_count += 2;

            let ansi = markup::render_ansi(&output);
            write!(stdout, "{}", ansi)?;
            stdout.flush()?;
            last_rendered_lines = line_count;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }

                match key.code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        } else {
                            selected = entries.len() - 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected < entries.len() - 1 {
                            selected += 1;
                        } else {
                            selected = 0;
                        }
                    }
                    KeyCode::Enter => {
                        let choice = &entries[selected];
                        if choice == "." {
                            break Ok(Some(current_dir));
                        } else if choice == ".." {
                            if let Some(parent) = current_dir.parent() {
                                current_dir = parent.to_path_buf();
                                selected = 0;
                            }
                        } else {
                            let new_path = current_dir.join(choice);
                            if new_path.is_dir() {
                                current_dir = new_path;
                                selected = 0;
                            }
                        }
                    }
                    KeyCode::Esc => {
                        break Ok(None);
                    }
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        break Ok(None);
                    }
                    _ => {}
                }
            }
        };

        // Final cleanup
        if last_rendered_lines > 0 {
            for _ in 0..last_rendered_lines {
                execute!(
                    stdout,
                    cursor::MoveUp(1),
                    terminal::Clear(terminal::ClearType::CurrentLine)
                )?;
            }
        }
        stdout.flush()?;

        disable_raw_mode()?;
        result
    }

    fn get_entries(&self, path: &Path) -> Vec<String> {
        let mut entries = vec!["..".to_string(), ".".to_string()];
        if let Ok(read_dir) = fs::read_dir(path) {
            let mut dirs: Vec<String> = read_dir
                .flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            dirs.sort();
            entries.extend(dirs);
        }
        entries
    }

    fn get_preview(&self, path: &Path) -> String {
        if !path.exists() {
            return "\r  <color_error>(Does not exist)</color_error>\n".to_string();
        }

        if path.is_dir() {
            match fs::read_dir(path) {
                Ok(entries) => {
                    let mut items = Vec::new();
                    for entry in entries.flatten().take(5) {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let is_dir = entry.path().is_dir();
                        if is_dir {
                            items.push(format!("<color_secondary>{}/</color_secondary>", name));
                        } else {
                            items.push(format!("<color_text>{}</color_text>", name));
                        }
                    }

                    if items.is_empty() {
                        "\r  <gray>(Empty directory)</gray>\n".to_string()
                    } else {
                        let mut res = String::new();
                        for item in items {
                            res.push_str(&format!("\r  {}\n", item));
                        }
                        res
                    }
                }
                Err(e) => format!("\r  <color_error>Error: {}</color_error>\n", e),
            }
        } else {
            "\r  <gray>(Not a directory)</gray>\n".to_string()
        }
    }
}
