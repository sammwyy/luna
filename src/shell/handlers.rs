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

        let result = if context.capture || !_stdin.is_empty() {
            if !_stdin.is_empty() {
                cmd.stdin(std::process::Stdio::piped());
            }
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
            child.wait_with_output()
        } else {
            // TUI and Direct Execution mode
            // Ignore signals in the parent so they only affect the child process
            let old_int = unsafe { libc::signal(libc::SIGINT, libc::SIG_IGN) };
            let old_quit = unsafe { libc::signal(libc::SIGQUIT, libc::SIG_IGN) };
            let old_tstp = unsafe { libc::signal(libc::SIGTSTP, libc::SIG_IGN) };
            let old_ttin = unsafe { libc::signal(libc::SIGTTIN, libc::SIG_IGN) };
            let old_ttou = unsafe { libc::signal(libc::SIGTTOU, libc::SIG_IGN) };

            let status = cmd.status();

            // Restore signal handlers
            unsafe {
                libc::signal(libc::SIGINT, old_int);
                libc::signal(libc::SIGQUIT, old_quit);
                libc::signal(libc::SIGTSTP, old_tstp);
                libc::signal(libc::SIGTTIN, old_ttin);
                libc::signal(libc::SIGTTOU, old_ttou);
            }

            match status {
                Ok(s) => {
                    return Ok(Output::new(s.code().unwrap_or(0), "".into(), "".into()));
                }
                Err(e) => {
                    let msg = format!("luna: {name}: {e}\n");
                    return Ok(Output::error(127, "".into(), msg));
                }
            }
        };

        match result {
            Ok(output) => Ok(Output::new(
                output.status.code().unwrap_or(0),
                String::from_utf8_lossy(&output.stdout).to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            )),
            Err(e) => Ok(Output::error(
                127,
                "".into(),
                format!("luna: {name}: {e}\n"),
            )),
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
