// ORIGINAL fonts
const FONT_ORIG_JSON: &[u8] =
    include_bytes!("../assets/Fonts/original/Lexend-Bold.json");
const FONT_ORIG_PNG: &[u8] =
    include_bytes!("../assets/Fonts/original/Lexend-Bold.png");
const FONT_ORIG_TTF: &[u8] =
    include_bytes!("../assets/Fonts/original/Lexend-Bold.ttf");

// RU fonts
const FONT_RU_JSON: &[u8] =
    include_bytes!("../assets/Fonts/withRU/Lexend-Bold.json");
const FONT_RU_PNG: &[u8] =
    include_bytes!("../assets/Fonts/withRU/Lexend-Bold.png");
const FONT_RU_TTF: &[u8] =
    include_bytes!("../assets/Fonts/withRU/Lexend-Bold.ttf");

// RU language
const LANG_CLIENT: &[u8] =
    include_bytes!("../assets/Language/ru-RU/client.lang");
const LANG_META: &[u8] =
    include_bytes!("../assets/Language/ru-RU/meta.lang");
const LANG_SERVER: &[u8] =
    include_bytes!("../assets/Language/ru-RU/server.lang");
const LANG_WORDLISTS: &[u8] =
    include_bytes!("../assets/Language/ru-RU/wordlists.lang");

use std::fs;
use std::path::PathBuf;
// use tauri::Manager;


fn get_config_path() -> Result<PathBuf, String> {
    let app_data = dirs::config_dir().ok_or("Cannot find config dir")?;
    let config_dir = app_data.join("HytaleRuLoader");
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    Ok(config_dir.join("config.txt"))
}

#[tauri::command]
fn save_custom_path(path: String) -> Result<(), String> {
    let config_path = get_config_path()?;
    
    if !path.contains("Hytale") {
        return Err("Путь должен содержать папку Hytale".to_string());
    }
    
    let path_buf = PathBuf::from(&path);
    let hytale_root = get_hytale_root_from_path(&path_buf);
    
    let game_path = hytale_root.join("install/release/package/game/latest");
    if !game_path.exists() {
        return Err(format!("Путь к игре не найден: {}", game_path.display()));
    }
    
    let exe_path = game_path.join("Client/HytaleClient.exe");
    if !exe_path.exists() {
        return Err("Файл HytaleClient.exe не найден".to_string());
    }
    
    fs::write(config_path, hytale_root.display().to_string()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn load_custom_path() -> Result<Option<String>, String> {
    let config_path = get_config_path()?;
    if config_path.exists() {
        let path = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        let trimmed_path = path.trim().to_string();
        
        if PathBuf::from(&trimmed_path).exists() {
            Ok(Some(trimmed_path))
        } else {
            fs::remove_file(&config_path).ok();
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn get_hytale_root_from_path(path: &PathBuf) -> PathBuf {
    let mut current = path.clone();
    
    while let Some(parent) = current.parent() {
        if current.ends_with("Hytale") {
            return current;
        }
        current = parent.to_path_buf();
    }
    
    path.clone()
}

fn hytale_game_dir() -> Result<PathBuf, String> {
    // Сначала пробуем пользовательский путь
    if let Ok(Some(custom_path)) = load_custom_path() {
        let custom_root = PathBuf::from(&custom_path);
        
        // Проверяем, что это корневая папка Hytale
        if !custom_root.ends_with("Hytale") {
            return Err("Сохраненный путь должен быть корневой папкой Hytale".to_string());
        }
        
        // Строим полный путь к игре
        let game_path = custom_root.join("install/release/package/game/latest");
        
        // Проверяем существование
        if game_path.exists() {
            return Ok(game_path);
        } else {
            // Путь не существует, удаляем из конфига
            let config_path = get_config_path()?;
            fs::remove_file(config_path).ok();
        }
    }
    
    // Используем стандартный путь по умолчанию
    get_default_game_dir()
}

fn get_default_game_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").map_err(|_| "Cannot find APPDATA")?;
        let hytale_root = PathBuf::from(appdata).join("Hytale");
        let game_path = hytale_root.join("install/release/package/game/latest");
        
        // Проверяем существование
        if game_path.exists() {
            Ok(game_path)
        } else {
            Err("Стандартный путь к игре не найден".to_string())
        }
    }

    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or("Cannot find home dir")?;
        let hytale_root = home.join(".config/Hytale");
        let game_path = hytale_root.join("install/release/package/game/latest");
        
        if game_path.exists() {
            Ok(game_path)
        } else {
            Err("Стандартный путь к игре не найден".to_string())
        }
    }

    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("Cannot find home dir")?;
        let hytale_root = home.join("Library/Application Support/Hytale");
        let game_path = hytale_root.join("install/release/package/game/latest");
        
        if game_path.exists() {
            Ok(game_path)
        } else {
            Err("Стандартный путь к игре не найден".to_string())
        }
    }
}

#[tauri::command]
fn get_hytale_root_path() -> Result<String, String> {
    // Получаем путь к игре
    let game_dir = hytale_game_dir()?;
    
    // Находим корневую папку Hytale
    let root_path = get_hytale_root_from_path(&game_dir);
    
    Ok(root_path.display().to_string())
}

// Функция проверки существования игры
#[tauri::command]
fn validate_game_path(path: String) -> Result<bool, String> {
    let path_buf = PathBuf::from(&path);
    
    // Проверяем, что путь содержит Hytale
    if !path.contains("Hytale") {
        return Ok(false);
    }
    
    // Находим корневую папку
    let hytale_root = get_hytale_root_from_path(&path_buf);
    
    // Проверяем путь к игре
    let game_path = hytale_root.join("install/release/package/game/latest");
    if !game_path.exists() {
        return Ok(false);
    }
    
    // Проверяем exe файл
    let exe_path = game_path.join("Client/HytaleClient.exe");
    Ok(exe_path.exists())
}



fn restore_original(game: &PathBuf) -> Result<(), String> {
    let fonts = game.join("Client/Data/Shared/Fonts");

    fs::write(fonts.join("Lexend-Bold.json"), FONT_ORIG_JSON)
        .map_err(|e| e.to_string())?;
    fs::write(fonts.join("Lexend-Bold.png"), FONT_ORIG_PNG)
        .map_err(|e| e.to_string())?;
    fs::write(fonts.join("Lexend-Bold.ttf"), FONT_ORIG_TTF)
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn install_ru(game: &PathBuf) -> Result<(), String> {
    let exe_path = game.join("Client/HytaleClient.exe");
    if !exe_path.exists() {
        return Err("HytaleClient.exe не найден. Проверьте путь к игре.".to_string());
    }

    let fonts = game.join("Client/Data/Shared/Fonts");
    let lang  = game.join("Client/Data/Shared/Language/ru-RU");

    // гарантируем папки
    fs::create_dir_all(&fonts).map_err(|e| e.to_string())?;
    fs::create_dir_all(&lang).map_err(|e| e.to_string())?;

    // шрифты (RU)
    fs::write(fonts.join("Lexend-Bold.json"), FONT_RU_JSON)
        .map_err(|e| e.to_string())?;
    fs::write(fonts.join("Lexend-Bold.png"), FONT_RU_PNG)
        .map_err(|e| e.to_string())?;
    fs::write(fonts.join("Lexend-Bold.ttf"), FONT_RU_TTF)
        .map_err(|e| e.to_string())?;

    // язык
    fs::write(lang.join("client.lang"), LANG_CLIENT)
        .map_err(|e| e.to_string())?;
    fs::write(lang.join("meta.lang"), LANG_META)
        .map_err(|e| e.to_string())?;
    fs::write(lang.join("server.lang"), LANG_SERVER)
        .map_err(|e| e.to_string())?;
    fs::write(lang.join("wordlists.lang"), LANG_WORDLISTS)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn restore_original_cmd() -> Result<(), String> {
    let game = hytale_game_dir()?;
    restore_original(&game)
}

#[tauri::command]
fn install_ru_cmd() -> Result<(), String> {
    let game = hytale_game_dir()?;
    install_ru(&game)
}

#[tauri::command]
fn check_ru_exists() -> Result<bool, String> {
    let game = hytale_game_dir()?;
    let ru_folder = game.join("Client/Data/Shared/Language/ru-RU");
    Ok(ru_folder.exists())
}

#[tauri::command]
fn remove_ru_cmd() -> Result<(), String> {
    let game = hytale_game_dir()?;
    let ru_folder = game.join("Client/Data/Shared/Language/ru-RU");
    if ru_folder.exists() {
        fs::remove_dir_all(&ru_folder).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_current_game_path() -> Result<String, String> {
    let game = hytale_game_dir()?;
    Ok(game.display().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            install_ru_cmd,
            restore_original_cmd,
            check_ru_exists,
            remove_ru_cmd,
            save_custom_path,
            load_custom_path,
            get_current_game_path,
            get_hytale_root_path,
            validate_game_path  
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
