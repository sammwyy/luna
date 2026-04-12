use super::{FileMetadata, Platform};
use std::fs;
use std::os::unix::fs::MetadataExt;

pub struct UnixPlatform;

impl Platform for UnixPlatform {
    fn get_hostname() -> String {
        std::fs::read_to_string("/proc/sys/kernel/hostname")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "localhost".to_string())
    }

    fn get_timezone_offset() -> i64 {
        unsafe {
            extern "C" {
                static timezone: i64;
                fn tzset();
            }
            tzset();
            -timezone
        }
    }

    fn get_file_metadata(m: &fs::Metadata) -> FileMetadata {
        FileMetadata {
            size: m.len(),
            blocks: m.blocks(),
            ino: m.ino(),
            mode: format!("{:o}", m.mode() & 0o777),
            uid: m.uid(),
            gid: m.gid(),
        }
    }

    fn setup_terminal() -> anyhow::Result<()> {
        Ok(())
    }
}

pub use UnixPlatform as CurrentPlatform;
