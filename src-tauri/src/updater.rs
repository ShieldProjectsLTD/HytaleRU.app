use tauri_plugin_updater::UpdaterExt;

#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<(), String> {
    let updater = app.updater_builder().build().map_err(|e| e.to_string())?;
    
    match updater.check().await {
        Ok(Some(update)) => {
            println!("Доступно обновление: {}", update.version);
            
            // Автоматическая установка обновления
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
        Ok(None) => {
            println!("Обновлений не найдено");
            Ok(())
        },
        Err(e) => Err(format!("Ошибка проверки обновлений: {}", e))
    }
}