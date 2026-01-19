use std::fs;
use std::path::PathBuf;
use crate::gamepath::hytale_game_dir;

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
