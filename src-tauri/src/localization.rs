use std::fs;
use std::path::{Path, PathBuf};
use crate::gamepath::hytale_game_dir;
use hytaleru_lib::resolve_assets_dir;

fn restore_original(game: &PathBuf) -> Result<(), String> {
    let assets_dir = resolve_assets_dir()?;
    let original_fonts = assets_dir.join("Fonts").join("original");
    let fonts = game.join("Client/Data/Shared/Fonts");

    if !original_fonts.exists() {
        return Err("Оригинальные шрифты не найдены".to_string());
    }

    fs::create_dir_all(&fonts).map_err(|e| e.to_string())?;
    copy_dir_recursive(&original_fonts, &fonts)?;

    Ok(())
}

fn install_ru(game: &PathBuf) -> Result<(), String> {
    let assets_dir = resolve_assets_dir()?;
    let exe_path = game.join("Client/HytaleClient.exe");
    if !exe_path.exists() {
        return Err("HytaleClient.exe не найден. Проверьте путь к игре.".to_string());
    }

    let fonts = game.join("Client/Data/Shared/Fonts");
    let lang  = game.join("Client/Data/Shared/Language/ru-RU");
    let ru_fonts = assets_dir.join("Fonts").join("withRU");
    let ru_lang = assets_dir.join("Language").join("ru-RU");

    if !ru_fonts.exists() {
        return Err("Папка Fonts/withRU не найдена".to_string());
    }

    if !ru_lang.exists() {
        return Err("Папка Language/ru-RU не найдена".to_string());
    }

    // гарантируем папки
    fs::create_dir_all(&fonts).map_err(|e| e.to_string())?;
    copy_dir_recursive(&ru_fonts, &fonts)?;

    if lang.exists() {
        fs::remove_dir_all(&lang).map_err(|e| e.to_string())?;
    }
    copy_dir_recursive(&ru_lang, &lang)?;

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;

    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if entry_path.is_dir() {
            copy_dir_recursive(&entry_path, &dst_path)?;
        } else {
            fs::copy(&entry_path, &dst_path).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn check_ru_exists() -> Result<bool, String> {
    let game = hytale_game_dir()?;
    let ru_folder = game.join("Client/Data/Shared/Language/ru-RU");
    Ok(ru_folder.exists())
}

#[tauri::command]
pub fn install_ru_cmd() -> Result<(), String> {
    let game = hytale_game_dir()?;
    install_ru(&game)
}

#[tauri::command]
pub fn remove_ru_cmd() -> Result<(), String> {
    let game = hytale_game_dir()?;
    let ru_folder = game.join("Client/Data/Shared/Language/ru-RU");
    if ru_folder.exists() {
        fs::remove_dir_all(&ru_folder).map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[tauri::command]
pub fn restore_original_cmd() -> Result<(), String> {
    let game = hytale_game_dir()?;
    restore_original(&game)
}
