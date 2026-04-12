#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use self::unix::*;
#[cfg(windows)]
pub use self::windows::*;

use std::fs;

pub struct FileMetadata {
    pub size: u64,
    pub blocks: u64,
    pub ino: u64,
    pub mode: String,
    pub uid: u32,
    pub gid: u32,
}

pub trait Platform {
    fn get_hostname() -> String;
    fn get_timezone_offset() -> i64;
    fn get_file_metadata(m: &fs::Metadata) -> FileMetadata;
    fn setup_terminal() -> anyhow::Result<()>;
}
