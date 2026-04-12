use std::env;
use std::fs;
use std::path::PathBuf;

pub fn luna_dir() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
    let dir = PathBuf::from(home).join(".luna");
    dir
}

pub fn history_file() -> PathBuf {
    luna_dir().join(".luna_history")
}

pub fn config_file() -> PathBuf {
    luna_dir().join("config.toml")
}

pub fn themes_dir() -> PathBuf {
    luna_dir().join("themes")
}

pub fn plugins_dir() -> PathBuf {
    luna_dir().join("plugins")
}

// ─── Setup Helper ────────────────────────────────────────────────────────────

struct SetupHelper {
    root: PathBuf,
}

impl SetupHelper {
    fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn setup(&self, relative_path: &str, content: &str, force: bool) -> &Self {
        let target = self.root.join(relative_path);
        if force || !target.exists() {
            // Ensure parent directory exists
            if let Some(parent) = target.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(target, content);
        }
        self
    }
}

/// Setup the user directory (~/.luna) with default config and themes if missing.
pub fn setup_user_dir(force: bool) {
    let helper = SetupHelper::new(luna_dir());

    helper
        .setup("config.toml", include_str!("../assets/config.toml"), force)
        .setup(
            "themes/default.lua",
            include_str!("../assets/themes/default.lua"),
            force,
        )
        .setup(
            "plugins/git.lua",
            include_str!("../assets/plugins/git.lua"),
            force,
        )
        .setup(
            "plugins/runtime_version.lua",
            include_str!("../assets/plugins/runtime_version.lua"),
            force,
        )
        .setup(
            "plugins/windows-aliases.lua",
            include_str!("../assets/plugins/windows-aliases.lua"),
            force,
        )
        .setup(
            "plugins/autoenv.lua",
            include_str!("../assets/plugins/autoenv.lua"),
            force,
        )
        .setup(
            "plugins/autoaliases.lua",
            include_str!("../assets/plugins/autoaliases.lua"),
            force,
        );
}
