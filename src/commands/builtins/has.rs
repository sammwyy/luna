use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::process::Command;

pub struct HasCommand;

impl BuiltinCommand for HasCommand {
    fn name(&self) -> &'static str {
        "has"
    }

    fn desc(&self) -> &'static str {
        "Check if programs are installed and show their versions"
    }

    fn run(
        &self,
        _ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        _stdin: &str,
    ) -> anyhow::Result<Output> {
        if args.positionals.is_empty() {
            return Ok(Output::error(
                1,
                "".into(),
                "has: missing argument\n".into(),
            ));
        }

        let mut out = String::new();
        let mut exit_code = 0;

        for name in &args.positionals {
            let mut found = false;
            let mut version_str = String::new();

            if _ctx.state.aliases.contains_key(name)
                || _ctx.state.builtins.contains(name)
                || which::which(name).is_ok()
            {
                found = true;

                let output = if cfg!(windows) {
                    let mut out = Command::new("cmd")
                        .args(["/C", name, "--version"])
                        .output()
                        .ok();
                    if out.is_none() || !out.as_ref().unwrap().status.success() {
                        out = Command::new("cmd").args(["/C", name, "-v"]).output().ok();
                    }
                    out
                } else {
                    let mut out = Command::new(name).arg("--version").output().ok();
                    if out.is_none() || !out.as_ref().unwrap().status.success() {
                        out = Command::new(name).arg("-v").output().ok();
                    }
                    out
                };

                if let Some(out) = output {
                    let text = String::from_utf8_lossy(&out.stdout);
                    let err_text = String::from_utf8_lossy(&out.stderr);
                    if let Some(v) = extract_semver(&text).or_else(|| extract_semver(&err_text)) {
                        version_str = format!(" (v{})", v);
                    }
                }
            }

            if found {
                out.push_str(&format!(
                    "  <#22c55e>✓</#22c55e> {}<color_secondary>{}</color_secondary>\n",
                    name, version_str
                ));
            } else {
                exit_code = 1;
                out.push_str(&format!(
                    "  <color_error>✗</color_error> {} <color_secondary>(✗)</color_secondary>\n",
                    name
                ));
            }
        }
        Ok(Output::new(exit_code, out, "".into()))
    }
}

fn extract_semver(text: &str) -> Option<String> {
    let mut i = 0;
    let chars: Vec<char> = text.chars().collect();
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let start = i;
            let mut dot_count = 0;
            let mut end = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                if chars[i] == '.' {
                    dot_count += 1;
                }
                end = i;
                i += 1;
            }
            if dot_count > 0 && chars[end].is_ascii_digit() {
                return Some(chars[start..=end].iter().collect());
            }
        } else {
            i += 1;
        }
    }
    None
}
