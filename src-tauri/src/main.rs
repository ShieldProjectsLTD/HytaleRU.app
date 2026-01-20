#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::path::PathBuf;
use std::fs;
mod updater;
mod localization;
mod gamepath;

use updater::check_for_updates;
use hytaleru_lib::save_to_config;
use hytaleru_lib::load_from_config;
use hytaleru_lib::remove_config;

#[tauri::command]
fn get_current_game_path() -> Result<String, String> {
    let game = crate::gamepath::hytale_game_dir()?;
    Ok(game.display().to_string())
}

#[tauri::command]
fn save_custom_path(path: String) -> Result<(), String> {
    use std::path::PathBuf;

    let path_buf = PathBuf::from(&path);

    // 1. Быстрая проверка
    if !path_buf.exists() {
        return Err("Указанный путь не существует".into());
    }

    // 2. Ищем корень Hytale
    let root = crate::gamepath::get_hytale_root_from_path(&path_buf);

    if !root.ends_with("Hytale") {
        return Err("Это не корневая папка Hytale".into());
    }

    // 3. Проверяем путь к игре
    let game_path = root.join("install/release/package/game/latest");

    if !game_path.exists() {
        return Err("Папка Hytale не найдена".into());
    }

    // 4. Проверяем exe
    let exe = game_path.join("Client/HytaleClient.exe");

    if !exe.exists() {
        return Err("Файл HytaleClient.exe не найден".into());
    }

    crate::save_to_config(&root.display().to_string())
}

#[tauri::command]
fn validate_custom_path(path: String) -> Result<String, String> {
    let path_buf = PathBuf::from(&path);

    if !path_buf.exists() {
        return Err("Путь не существует".into());
    }

    // Находим корень Hytale
    let root = crate::gamepath::get_hytale_root_from_path(&path_buf);
    if !root.ends_with("Hytale") {
        return Err("Это не корневая папка Hytale".into());
    }

    let game_path = root.join("install/release/package/game/latest");
    if !game_path.exists() {
        return Err("Папка Hytale не найдена".into());
    }

    let exe_path = game_path.join("Client/HytaleClient.exe");
    if !exe_path.exists() {
        return Err("Файл HytaleClient.exe не найден".into());
    }

    // Сохраняем путь в path.txt рядом с приложением
    if let Ok(exe_dir) = std::env::current_exe() {
        let app_dir = exe_dir.parent().unwrap_or(&exe_dir);
        let path_file = app_dir.join("path.txt");
        let _ = fs::write(path_file, root.display().to_string());
    }

    Ok(root.display().to_string())
}

#[tauri::command]
fn get_saved_path() -> Result<Option<String>, String> {
    if let Ok(exe_dir) = std::env::current_exe() {
        let app_dir = exe_dir.parent().unwrap_or(&exe_dir);
        let path_file = app_dir.join("path.txt");
        if path_file.exists() {
            let s = fs::read_to_string(path_file).map_err(|e| e.to_string())?;
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() {
                return Ok(None);
            }
            return Ok(Some(trimmed));
        }
    }
    Ok(None)
}

#[tauri::command]
fn check_ru_installed(path: String) -> Result<bool, String> {
    let root = PathBuf::from(&path);
    let ru_file = root.join("install/release/package/game/latest/Client/Data/Shared/Language/ru-RU/client.lang");
    Ok(ru_file.exists())
}


fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Проверка обновлений при запуске
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                
                if let Err(e) = check_for_updates(app_handle).await {
                    eprintln!("Ошибка при проверке обновлений: {}", e);
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            localization::install_ru_cmd,
            localization::restore_original_cmd,
            localization::check_ru_exists,
            localization::remove_ru_cmd,

            get_current_game_path,
            save_custom_path,
            validate_custom_path,
            check_ru_installed,
            get_saved_path,

            check_for_updates
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
