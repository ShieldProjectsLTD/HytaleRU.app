use std::fs;
use std::path::PathBuf;

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
