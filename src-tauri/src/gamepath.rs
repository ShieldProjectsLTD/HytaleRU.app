use std::path::{Path, PathBuf};
use std::fs;

use crate::{load_from_config, remove_config};

pub fn get_path_file() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|_| "Не удалось получить путь к исполняемому файлу")?;
    
    let mut check_dir = exe_path.parent()
        .ok_or("Не удалось получить папку исполняемого файла")?
        .to_path_buf();
    
    for _ in 0..10 {
        let src_tauri = check_dir.join("src-tauri");
        if src_tauri.exists() && src_tauri.is_dir() {
            return Ok(check_dir.join("path.txt"));
        }
        
        if let Some(parent) = check_dir.parent() {
            check_dir = parent.to_path_buf();
        } else {
            break;
        }
    }
    
    let app_dir = exe_path.parent()
        .ok_or("Не удалось получить папку приложения")?;
    Ok(app_dir.join("path.txt"))
}

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
        if let Ok(appdata) = std::env::var("APPDATA") {
            let root = PathBuf::from(appdata).join("Hytale");
            let game = build_game_path(&root);
            if game.exists() {
                return Ok(game);
            }
        }
        
        scan_all_drives()
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

#[cfg(target_os = "windows")]
fn scan_all_drives() -> Result<PathBuf, String> {
    if let Ok(system_drive) = std::env::var("SystemDrive") {
        let drive_path = PathBuf::from(&system_drive);
        if let Ok(found) = search_hytale_on_drive(&drive_path) {
            return Ok(found);
        }
    }
    
    for drive_letter in b'A'..=b'Z' {
        let drive = format!("{}:\\", drive_letter as char);
        let drive_path = PathBuf::from(&drive);
        
        if let Ok(system_drive) = std::env::var("SystemDrive") {
            if drive_path.starts_with(&system_drive) {
                continue;
            }
        }
        
        if !drive_path.exists() {
            continue;
        }
        
        if let Ok(found) = search_hytale_on_drive(&drive_path) {
            return Ok(found);
        }
    }
    
    Err("Hytale не найден ни на одном диске".to_string())
}

#[cfg(target_os = "windows")]
fn search_hytale_on_drive(drive: &Path) -> Result<PathBuf, String> {
    let possible_paths = vec![
        drive.join("Users").join(std::env::var("USERNAME").unwrap_or_default()).join("AppData").join("Roaming").join("Hytale"),
        drive.join("ProgramData").join("Hytale"),
        drive.join("Games").join("Hytale"),
        drive.join("Hytale"),
        
        drive.join("Program Files").join("Hytale"),
        drive.join("Program Files (x86)").join("Hytale"),
    ];
    
    for root in possible_paths {
        if root.exists() && root.ends_with("Hytale") {
            let game = build_game_path(&root);
            if game.exists() {
                return Ok(game);
            }
        }
    }
    
    let users_dir = drive.join("Users");
    if users_dir.exists() {
        if let Ok(entries) = fs::read_dir(&users_dir) {
            for entry in entries.flatten() {
                let user_path = entry.path();
                if user_path.is_dir() {
                    let hytale_path = user_path.join("AppData").join("Roaming").join("Hytale");
                    if hytale_path.exists() {
                        let game = build_game_path(&hytale_path);
                        if game.exists() {
                            return Ok(game);
                        }
                    }
                }
            }
        }
    }
    
    Err("Hytale не найден на этом диске".to_string())
}
