use std::path::{Path, PathBuf};

use crate::{
    load_from_config,
    remove_config,
};

pub fn hytale_game_dir() -> Result<PathBuf, String> {
    if let Ok(Some(custom_root)) = load_from_config() {
        let root = PathBuf::from(&custom_root);

        if root.ends_with("Hytale") {
            let game = build_game_path(&root);
            if game.exists() {
                return Ok(game);
            }
        }

        remove_config();
    }

    get_default_game_dir()
}

fn build_game_path(root: &Path) -> PathBuf {
    root.join("install/release/package/game/latest")
}

pub fn get_default_game_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").map_err(|_| "No APPDATA")?;
        let root = PathBuf::from(appdata).join("Hytale");
        let game = build_game_path(&root);

        game.exists()
            .then(|| game)
            .ok_or("Default Hytale path not found".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or("No home dir")?;
        let root = home.join(".config/Hytale");
        let game = build_game_path(&root);

        game.exists()
            .then(|| game)
            .ok_or("Default Hytale path not found".to_string())
    }

    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("No home dir")?;
        let root = home.join("Library/Application Support/Hytale");
        let game = build_game_path(&root);

        game.exists()
            .then(|| game)
            .ok_or("Default Hytale path not found".to_string())
    }
}

pub fn get_hytale_root_from_path(path: &PathBuf) -> PathBuf {
    let mut current = path.clone();

    while let Some(parent) = current.parent() {
        if current.file_name().is_some_and(|n| n == "Hytale") {
            return current;
        }
        current = parent.to_path_buf();
    }

    path.clone()
}
