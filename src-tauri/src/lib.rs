use std::fs;
use std::path::PathBuf;

pub fn resolve_assets_dir() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("HYTALERU_ASSETS_DIR") {
        let dir = PathBuf::from(path);
        if dir.exists() {
            return Ok(dir);
        }
    }

    if cfg!(debug_assertions) {
        if let Ok(current) = std::env::current_dir() {
            let direct = current.join("src-tauri").join("assets");
            if direct.exists() {
                return Ok(direct);
            }

            let nested = current.join("assets");
            if nested.exists() {
                return Ok(nested);
            }
        }
    }

    let exe = std::env::current_exe().map_err(|_| "Не удалось определить путь приложения")?;
    let mut current = exe
        .parent()
        .ok_or("Не удалось определить директорию приложения")?
        .to_path_buf();

    for _ in 0..8 {
        let direct_assets = current.join("assets");
        if direct_assets.exists() {
            return Ok(direct_assets);
        }

        let nested_assets = current.join("src-tauri").join("assets");
        if nested_assets.exists() {
            return Ok(nested_assets);
        }

        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    Err("Не удалось найти директорию assets".to_string())
}

pub fn get_config_path() -> Result<PathBuf, String> {
    let app_data = dirs::config_dir().ok_or("Cannot find config dir")?;
    let config_dir = app_data.join("HytaleRuLoader");
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    Ok(config_dir.join("config.txt"))
}

pub fn save_to_config(value: &str) -> Result<(), String> {
    let path = get_config_path()?;
    fs::write(path, value).map_err(|e| e.to_string())
}

pub fn load_from_config() -> Result<Option<String>, String> {
    let path = get_config_path()?;

    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let trimmed = content.trim().to_string();

    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed))
    }
}

pub fn remove_config() {
    if let Ok(path) = get_config_path() {
        let _ = fs::remove_file(path);
    }
}
