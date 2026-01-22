use tauri_plugin_updater::UpdaterExt;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub date: Option<String>,
    pub body: Option<String>,
}

#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater_builder().build().map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(Some(update)) => {
            println!("Доступно обновление: {}", update.version);
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
        Err(e) => Err(format!("Ошибка проверки обновлений: {}", e))
    }
}

#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    let updater = app.updater_builder().build().map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(Some(update)) => {
            println!("Устанавливаем обновление: {}", update.version);

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