use crate::shell::state::LunaState;

use shellframe::{Output, RedirectMode};
use std::process::Command;

pub fn setup_shell_handlers(shell: &mut shellframe::Shell<LunaState>) {
    // System command hook
    shell.set_hook(|name, args, context, _stdin| {
        let mut cmd = Command::new(name);
        cmd.args(args);
        cmd.envs(&context.env);
        cmd.current_dir(context.get_cwd());

        if !_stdin.is_empty() {
            cmd.stdin(std::process::Stdio::piped());
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    let msg = format!("luna: {name}: {e}\n");
                    return Ok(Output::error(127, "".into(), msg));
                }
            };
            if let Some(mut child_stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = child_stdin.write_all(_stdin.as_bytes());
            }
            let output = match child.wait_with_output() {
                Ok(out) => out,
                Err(e) => {
                    return Ok(Output::error(
                        127,
                        "".into(),
                        format!("luna: {name}: {e}\n"),
                    ))
                }
            };
            Ok(Output::new(
                output.status.code().unwrap_or(0),
                String::from_utf8_lossy(&output.stdout).to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        } else {
            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    let msg = format!("luna: {name}: {e}\n");
                    return Ok(Output::error(127, "".into(), msg));
                }
            };
            let status = match child.wait() {
                Ok(s) => s,
                Err(e) => {
                    return Ok(Output::error(
                        127,
                        "".into(),
                        format!("luna: {name}: {e}\n"),
                    ))
                }
            };
            Ok(Output::new(
                status.code().unwrap_or(0),
                String::new(),
                String::new(),
            ))
        }
    });

    // Redirection handler
    shell.set_redirect_handler(|sh, expr, file, mode, stdin| {
        use std::fs::File;
        use std::io::{Read, Write};
        match mode {
            RedirectMode::Input => {
                let mut content = String::new();
                if let Ok(mut f) = File::open(file) {
                    let _ = f.read_to_string(&mut content);
                }
                sh.eval(expr, &content)
            }
            RedirectMode::Overwrite | RedirectMode::Append => {
                let output = sh.eval(expr, stdin)?;
                let mut f = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(*mode == RedirectMode::Append)
                    .truncate(*mode == RedirectMode::Overwrite)
                    .open(file)?;
                f.write_all(output.stdout.as_bytes())?;
                Ok(Output::new(output.exit_code, String::new(), output.stderr))
            }
        }
    });
}
