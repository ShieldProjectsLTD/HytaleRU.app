use hytaleru_lib::resolve_assets_dir;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const GITHUB_API_URL: &str = "https://api.github.com/repos/zzentq/HytaleRussianTranslation/releases/latest";
const USER_AGENT: &str = "HytaleRU-App";
const SHARED_PREFIX: &str = "install/release/package/game/latest/Client/Data/Shared/";
const MAX_ARCHIVE_SIZE: u64 = 50 * 1024 * 1024;
const MAX_ENTRY_SIZE: u64 = 10 * 1024 * 1024;
const MAX_MANIFEST_SIZE: u64 = 256 * 1024;

#[derive(Serialize, Deserialize, Debug)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: Option<String>,
    pub published_at: String,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
    pub download_count: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalizationUpdateInfo {
    pub current_version: Option<String>,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: Option<String>,
    pub changelog: Option<String>,
}

#[tauri::command]
pub async fn check_localization_updates() -> Result<Option<LocalizationUpdateInfo>, String> {
    let release = fetch_latest_release().await?;
    let asset = select_zip_asset(&release)?;
    let current_version = get_current_localization_version()?;
    let latest_version = normalize_version(&release.tag_name);
    let update_available = is_update_available(&current_version, &latest_version);
    if !update_available {
        println!("Обновлений локализации не найдено");
    }

    Ok(Some(LocalizationUpdateInfo {
        current_version,
        latest_version,
        update_available,
        download_url: Some(asset.browser_download_url.clone()),
        changelog: release.body,
    }))
}

#[tauri::command]
pub async fn auto_update_localization() -> Result<bool, String> {
    let release = fetch_latest_release().await?;
    let asset = select_zip_asset(&release)?;
    let current_version = get_current_localization_version()?;
    let latest_version = normalize_version(&release.tag_name);

    if !is_update_available(&current_version, &latest_version) {
        println!("Обновлений локализации не найдено");
        return Ok(false);
    }

    let zip_path = download_zip(&asset.browser_download_url, asset.size).await?;
    let result = install_localization_update(&zip_path, &latest_version);
    let _ = fs::remove_file(&zip_path);
    result?;

    Ok(true)
}

#[tauri::command]
pub async fn download_localization_update(
    version: String,
    download_url: String,
) -> Result<(), String> {
    let release = fetch_latest_release().await?;
    let asset = select_zip_asset(&release)?;
    let latest_version = normalize_version(&release.tag_name);
    let requested_version = normalize_version(&version);

    if requested_version != latest_version {
        return Err("Запрошенная версия не совпадает с последним релизом".to_string());
    }

    if download_url != asset.browser_download_url {
        return Err("Ссылка на загрузку не совпадает с последним релизом".to_string());
    }

    let zip_path = download_zip(&asset.browser_download_url, asset.size).await?;
    let result = install_localization_update(&zip_path, &latest_version);
    let _ = fs::remove_file(&zip_path);
    result
}

fn get_current_localization_version() -> Result<Option<String>, String> {
    let assets_dir = resolve_assets_dir()?;
    let manifest_file = assets_dir.join("manifest.json");

    if !manifest_file.exists() {
        return Ok(None);
    }

    let manifest_content = fs::read_to_string(&manifest_file)
        .map_err(|e| format!("Ошибка чтения manifest.json: {}", e))?;

    let manifest: serde_json::Value = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("Ошибка парсинга manifest.json: {}", e))?;

    match manifest.get("Version").and_then(|v| v.as_str()) {
        Some(version) => Ok(Some(version.to_string())),
        None => Err("Поле Version не найдено в manifest.json".to_string()),
    }
}

async fn fetch_latest_release() -> Result<GitHubRelease, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| format!("Ошибка инициализации клиента: {}", e))?;

    let response = client
        .get(GITHUB_API_URL)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("Ошибка запроса к GitHub API: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API вернул статус: {}", response.status()));
    }

    response
        .json::<GitHubRelease>()
        .await
        .map_err(|e| format!("Ошибка парсинга JSON: {}", e))
}

fn select_zip_asset(release: &GitHubRelease) -> Result<&GitHubAsset, String> {
    release
        .assets
        .iter()
        .find(|asset| asset.name.ends_with(".zip") && asset.name.contains("Hytale-Russian"))
        .ok_or("ZIP файл релиза не найден".to_string())
}

async fn download_zip(download_url: &str, asset_size: u64) -> Result<PathBuf, String> {
    if asset_size > MAX_ARCHIVE_SIZE {
        return Err("Архив слишком большой".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Ошибка инициализации клиента: {}", e))?;

    let response = client
        .get(download_url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("Ошибка скачивания архива: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Ошибка скачивания, статус: {}", response.status()));
    }

    if let Some(size) = response.content_length() {
        if size > MAX_ARCHIVE_SIZE {
            return Err("Архив слишком большой".to_string());
        }
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Ошибка чтения архива: {}", e))?;

    if bytes.len() as u64 > MAX_ARCHIVE_SIZE {
        return Err("Архив слишком большой".to_string());
    }

    let zip_path = create_temp_zip_path()?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&zip_path)
        .map_err(|e| format!("Ошибка создания временного файла: {}", e))?;

    file.write_all(&bytes)
        .map_err(|e| format!("Ошибка записи архива: {}", e))?;

    Ok(zip_path)
}

fn create_temp_zip_path() -> Result<PathBuf, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "Ошибка времени системы")?
        .as_millis();
    let filename = format!("hytale_ru_update_{}.zip", now);
    Ok(std::env::temp_dir().join(filename))
}

fn install_localization_update(zip_path: &Path, latest_version: &str) -> Result<(), String> {
    use zip::ZipArchive;

    let assets_dir = resolve_assets_dir()?;
    ensure_original_fonts(&assets_dir)?;
    let staging_dir = assets_dir.join(".update_tmp");
    prepare_dir(&staging_dir)?;
    let _guard = StagingGuard(staging_dir.clone());

    let staging_fonts = staging_dir.join("Fonts").join("withRU");
    let staging_lang = staging_dir.join("Language").join("ru-RU");
    fs::create_dir_all(&staging_fonts)
        .map_err(|e| format!("Ошибка создания директории: {}", e))?;
    fs::create_dir_all(&staging_lang)
        .map_err(|e| format!("Ошибка создания директории: {}", e))?;

    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Ошибка открытия ZIP файла: {}", e))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Ошибка чтения ZIP архива: {}", e))?;

    let mut manifest_bytes: Option<Vec<u8>> = None;
    let mut found_fonts = false;
    let mut found_lang = false;
    let mut total_size: u64 = 0;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Ошибка чтения файла из архива: {}", e))?;

        if entry.is_dir() || is_symlink(&entry) {
            continue;
        }

        if entry.size() > MAX_ENTRY_SIZE {
            return Err("Файл в архиве слишком большой".to_string());
        }

        total_size = total_size.saturating_add(entry.size());
        if total_size > MAX_ARCHIVE_SIZE {
            return Err("Архив слишком большой".to_string());
        }

        let safe_name = entry
            .enclosed_name()
            .ok_or("Небезопасный путь в архиве")?;
        let name = safe_name
            .to_string_lossy()
            .replace('\\', "/");

        if is_manifest_path(&name) {
            let bytes = read_zip_entry(&mut entry, MAX_MANIFEST_SIZE)?;
            validate_manifest(&bytes)?;
            manifest_bytes = Some(bytes);
            continue;
        }

        if let Some(relative) = extract_shared_relative(&name) {
            if let Some(fonts_rel) = relative.strip_prefix("Fonts/") {
                let out_path = safe_join(&staging_fonts, Path::new(fonts_rel))?;
                write_zip_entry(&mut entry, &out_path)?;
                found_fonts = true;
                continue;
            }

            if let Some(lang_rel) = relative
                .strip_prefix("Language/ru-RU/")
                .or_else(|| relative.strip_prefix("Language/ru_RU/"))
            {
                let out_path = safe_join(&staging_lang, Path::new(lang_rel))?;
                write_zip_entry(&mut entry, &out_path)?;
                found_lang = true;
            }
        }
    }

    let manifest = manifest_bytes.ok_or("manifest.json не найден в архиве")?;
    let manifest_version = extract_manifest_version(&manifest)?;
    if normalize_version(&manifest_version) != normalize_version(latest_version) {
        return Err("Версия manifest.json не совпадает с релизом".to_string());
    }

    if !found_fonts {
        return Err("В архиве нет файлов Fonts".to_string());
    }

    if !found_lang {
        return Err("В архиве нет файлов Language/ru-RU".to_string());
    }

    let assets_fonts = assets_dir.join("Fonts").join("withRU");
    let assets_lang = assets_dir.join("Language").join("ru-RU");

    replace_dir(&staging_fonts, &assets_fonts)?;
    replace_dir(&staging_lang, &assets_lang)?;
    write_atomic(&assets_dir.join("manifest.json"), &manifest)?;
    Ok(())
}

fn ensure_original_fonts(assets_dir: &Path) -> Result<(), String> {
    let original_dir = assets_dir.join("Fonts").join("original");
    let required = ["Lexend-Bold.json", "Lexend-Bold.png", "Lexend-Bold.ttf"];

    let has_all = required
        .iter()
        .all(|name| original_dir.join(name).exists());

    if has_all {
        return Ok(());
    }

    let game_dir = crate::gamepath::hytale_game_dir()?;
    let game_fonts = game_dir.join("Client/Data/Shared/Fonts");
    if !game_fonts.exists() {
        return Err("Папка Fonts в игре не найдена".to_string());
    }

    prepare_dir(&original_dir)?;
    copy_dir_recursive(&game_fonts, &original_dir)?;
    Ok(())
}

fn extract_shared_relative(path: &str) -> Option<&str> {
    path.find(SHARED_PREFIX)
        .map(|pos| &path[(pos + SHARED_PREFIX.len())..])
}

fn is_manifest_path(path: &str) -> bool {
    path == "manifest.json" || path.ends_with("/manifest.json")
}

fn validate_manifest(bytes: &[u8]) -> Result<(), String> {
    let manifest: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| format!("Ошибка парсинга manifest.json: {}", e))?;

    if manifest.get("Version").and_then(|v| v.as_str()).is_none() {
        return Err("Поле Version не найдено в manifest.json".to_string());
    }

    Ok(())
}

fn extract_manifest_version(bytes: &[u8]) -> Result<String, String> {
    let manifest: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| format!("Ошибка парсинга manifest.json: {}", e))?;

    manifest
        .get("Version")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
        .ok_or("Поле Version не найдено в manifest.json".to_string())
}

fn read_zip_entry(entry: &mut zip::read::ZipFile, limit: u64) -> Result<Vec<u8>, String> {
    if entry.size() > limit {
        return Err("Файл в архиве слишком большой".to_string());
    }

    let mut buffer = Vec::with_capacity(entry.size() as usize);
    entry
        .read_to_end(&mut buffer)
        .map_err(|e| format!("Ошибка чтения файла из архива: {}", e))?;
    Ok(buffer)
}

fn write_zip_entry(entry: &mut zip::read::ZipFile, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Ошибка создания директории: {}", e))?;
    }

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(|e| format!("Ошибка создания файла: {}", e))?;

    std::io::copy(entry, &mut file)
        .map_err(|e| format!("Ошибка копирования файла: {}", e))?;
    Ok(())
}

fn prepare_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path)
            .map_err(|e| format!("Ошибка очистки директории: {}", e))?;
    }
    fs::create_dir_all(path)
        .map_err(|e| format!("Ошибка создания директории: {}", e))?;
    Ok(())
}

fn replace_dir(src: &Path, dst: &Path) -> Result<(), String> {
    if dst.exists() {
        fs::remove_dir_all(dst)
            .map_err(|e| format!("Ошибка удаления директории: {}", e))?;
    }

    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Ошибка создания директории: {}", e))?;
    }

    if fs::rename(src, dst).is_err() {
        copy_dir_recursive(src, dst)?;
        fs::remove_dir_all(src)
            .map_err(|e| format!("Ошибка удаления временной директории: {}", e))?;
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst)
        .map_err(|e| format!("Ошибка создания директории: {}", e))?;

    for entry in fs::read_dir(src)
        .map_err(|e| format!("Ошибка чтения директории: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Ошибка чтения записи: {}", e))?;
        let entry_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if entry_path.is_dir() {
            copy_dir_recursive(&entry_path, &dst_path)?;
        } else {
            fs::copy(&entry_path, &dst_path)
                .map_err(|e| format!("Ошибка копирования файла: {}", e))?;
        }
    }

    Ok(())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temp_path = path.with_extension(format!(
        "tmp_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "Ошибка времени системы")?
            .as_millis()
    ));

    if let Some(parent) = temp_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Ошибка создания директории: {}", e))?;
    }

    fs::write(&temp_path, bytes)
        .map_err(|e| format!("Ошибка записи файла: {}", e))?;
    fs::rename(&temp_path, path)
        .map_err(|e| format!("Ошибка сохранения файла: {}", e))?;
    Ok(())
}

fn safe_join(base: &Path, relative: &Path) -> Result<PathBuf, String> {
    if relative.components().any(|c| {
        matches!(
            c,
            std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_)
        )
    }) {
        return Err("Небезопасный путь в архиве".to_string());
    }
    Ok(base.join(relative))
}

fn is_symlink(entry: &zip::read::ZipFile) -> bool {
    if let Some(mode) = entry.unix_mode() {
        (mode & 0o170000) == 0o120000
    } else {
        false
    }
}

fn normalize_version(version: &str) -> String {
    version.trim().trim_start_matches('v').to_string()
}

fn is_update_available(current: &Option<String>, latest: &str) -> bool {
    match current.as_ref().map(|v| normalize_version(v)) {
        Some(current_version) => compare_versions(&current_version, latest) == Ordering::Less,
        None => true,
    }
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    let left_parts = parse_version(left);
    let right_parts = parse_version(right);
    let max_len = left_parts.len().max(right_parts.len());

    for i in 0..max_len {
        let l = *left_parts.get(i).unwrap_or(&0);
        let r = *right_parts.get(i).unwrap_or(&0);
        match l.cmp(&r) {
            Ordering::Equal => continue,
            other => return other,
        }
    }

    Ordering::Equal
}

fn parse_version(value: &str) -> Vec<u64> {
    value
        .split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}

struct StagingGuard(PathBuf);

impl Drop for StagingGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
