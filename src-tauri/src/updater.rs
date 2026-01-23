use tauri_plugin_updater::UpdaterExt;
use serde::{Serialize, Deserialize};
use tauri_plugin_opener::OpenerExt;

#[derive(Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub date: Option<String>,
    pub body: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PlatformInfo {
    pub platform: String,
    pub update_supported: bool,
}


#[tauri::command]
pub fn get_platform_info() -> PlatformInfo {
    let platform = std::env::consts::OS.to_string();
    let update_supported = !cfg!(target_os = "linux");

    PlatformInfo {
        platform,
        update_supported,
    }
}

#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater_builder().build().map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(Some(update)) => {
            println!("Доступно обновление: v{}", update.version);
            Ok(Some(UpdateInfo {
                version: update.version,
                date: update.date.map(|d| d.to_string()),
                body: update.body,
            }))
        },
        Ok(None) => {
            println!("Обновлений не найдено");
            Ok(None)
        },
        Err(e) => Err(format!("Ошибка проверки обновлений: v{}", e))
    }
}

#[tauri::command]
pub async fn open_release_page(app: tauri::AppHandle, version: String) -> Result<(), String> {
    let release_url = format!("https://github.com/ShieldProjectsLTD/HytaleRU.app/releases/tag/v{}", version);

    app.opener()
        .open_url(release_url, None::<&str>)
        .map_err(|e| format!("Не удалось открыть страницу релиза: {}", e))?;

    Ok(())
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    let updater = app.updater_builder().build().map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(Some(update)) => {
            println!("Устанавливаем обновление: v{}", update.version);

            match update.download_and_install(|chunk_length, content_length| {
                if let Some(total) = content_length {
                    let progress = (chunk_length as f64 / total as f64) * 100.0;
                    println!("Загружено: {:.2}%", progress);
                }
            }, || {
                println!("Загрузка завершена, устанавливаем...");
            }).await {
                Ok(_) => {
                    println!("Обновление установлено успешно!");
                    Ok(())
                },
                Err(e) => Err(format!("Ошибка установки: {}", e))
            }
        },
        Ok(None) => Err("Обновление не найдено".to_string()),
        Err(e) => Err(format!("Ошибка проверки обновлений: {}", e))
    }
}

#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn install_update(_app: tauri::AppHandle) -> Result<(), String> {
    Err("Автоматические обновления на Linux не поддерживаются. Пожалуйста, скачайте обновление вручную со страницы релизов.".to_string())
}