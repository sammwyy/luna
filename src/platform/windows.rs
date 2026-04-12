use super::{FileMetadata, Platform};
use std::fs;
use std::os::windows::fs::MetadataExt;

pub struct WindowsPlatform;

impl Platform for WindowsPlatform {
    fn get_hostname() -> String {
        std::env::var("COMPUTERNAME").unwrap_or_else(|_| "localhost".to_string())
    }

    fn get_timezone_offset() -> i64 {
        unsafe {
            extern "C" {
                fn _tzset();
                fn _get_timezone(offset: *mut i32) -> i32;
            }
            _tzset();
            let mut offset = 0i32;
            _get_timezone(&mut offset);
            -(offset as i64)
        }
    }

    fn get_file_metadata(m: &fs::Metadata) -> FileMetadata {
        FileMetadata {
            size: m.len(),
            blocks: 0,
            ino: 0,
            mode: format!("{:X}", m.file_attributes()),
            uid: 0,
            gid: 0,
        }
    }

    fn setup_terminal() -> anyhow::Result<()> {
        // Enable Virtual Terminal Processing (ANSI escape codes) on Windows.
        // Without this flag, cmd.exe / Windows Terminal / conhost will print
        // raw ANSI bytes instead of interpreting them.
        //
        // We talk directly to the Windows Console API via FFI to avoid an
        // extra crate.  The constants come from <consoleapi.h>.
        unsafe {
            // ----- type aliases -------------------------------------------------
            type HANDLE = *mut std::ffi::c_void;
            type BOOL   = i32;
            type DWORD  = u32;

            const ENABLE_VIRTUAL_TERMINAL_PROCESSING: DWORD = 0x0004;
            const STD_OUTPUT_HANDLE: DWORD = 0xFFFFFFF5_u32; // (-11i32) cast

            extern "system" {
                fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
                fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: *mut DWORD) -> BOOL;
                fn SetConsoleMode(hConsoleHandle: HANDLE, dwMode: DWORD) -> BOOL;
                fn SetConsoleOutputCP(wCodePageID: DWORD) -> BOOL;
                fn SetConsoleCP(wCodePageID: DWORD) -> BOOL;
            }

            // UTF-8 (code page 65001) — ensures prompt glyphs / symbols render
            // correctly even in environments that default to e.g. CP-1252.
            SetConsoleOutputCP(65001);
            SetConsoleCP(65001);

            // Enable ANSI / VT100 on the stdout handle.
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            if !handle.is_null() && handle as isize != -1 {
                let mut mode: DWORD = 0;
                if GetConsoleMode(handle, &mut mode) != 0 {
                    if mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING == 0 {
                        SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
                    }
                }
            }
        }

        Ok(())
    }
}

pub use WindowsPlatform as CurrentPlatform;
