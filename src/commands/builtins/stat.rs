use crate::commands::system::{BuiltinCommand, ParsedArgs};
use crate::shell::state::LunaState;
use shellframe::Context;
use shellframe::Output;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::time::UNIX_EPOCH;

pub struct StatCommand;

impl BuiltinCommand for StatCommand {
    fn name(&self) -> &'static str {
        "stat"
    }
    fn desc(&self) -> &'static str {
        "Display file or file system status"
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
                "stat: missing operand\n".into(),
            ));
        }

        let mut out = String::new();
        for file in &args.positionals {
            match fs::metadata(file) {
                Ok(m) => {
                    let size = m.size();
                    let blocks = m.blocks();
                    let ino = m.ino();
                    let mode = format!("{:o}", m.mode() & 0o777); // Extract unix permissions mode
                    let uid = m.uid();
                    let gid = m.gid();
                    let atime = m
                        .accessed()
                        .unwrap_or(UNIX_EPOCH)
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let mtime = m
                        .modified()
                        .unwrap_or(UNIX_EPOCH)
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    out.push_str(&format!(
                        "  <color_primary>File:</color_primary> <color_text>{}</color_text>\n  \
                         <color_primary>Size:</color_primary> <color_text>{}</color_text>\t<color_primary>Blocks:</color_primary> <color_text>{}</color_text>\n  \
                         <color_primary>Inode:</color_primary> <color_text>{}</color_text>\n  \
                         <color_primary>Access:</color_primary> <color_secondary>(0{})</color_secondary>\t<color_primary>Uid:</color_primary> <color_text>{}</color_text>\t<color_primary>Gid:</color_primary> <color_text>{}</color_text>\n  \
                         <color_primary>Access:</color_primary> <color_warn>{}</color_warn>\n  \
                         <color_primary>Modify:</color_primary> <color_warn>{}</color_warn>\n", 
                        file, size, blocks, ino, mode, uid, gid, atime, mtime));
                }
                Err(e) => {
                    out.push_str(&format!(
                        "<color_error>stat: cannot stat '{}': {}</color_error>\n",
                        file, e
                    ));
                }
            }
        }
        Ok(Output::success(out))
    }
}
