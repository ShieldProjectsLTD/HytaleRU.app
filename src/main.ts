import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

const actionBtn = document.getElementById("action-btn") as HTMLButtonElement;
const folderBtn = document.getElementById("folder-btn") as HTMLButtonElement;
const statusText = document.getElementById("p-status") as HTMLParagraphElement;
const gamePathText = document.getElementById("p-gamepath") as HTMLParagraphElement;

let ruInstalled = false;


async function selectGamePath() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Выберите корневую папку Hytale"
    });
    
    if (selected && typeof selected === 'string') {
      const isValid = await invoke<boolean>("validate_game_path", {
        path: selected
      });
      
      if (isValid) {
        await invoke("save_custom_path", { path: selected });
        await checkStatus();
      } else {
        alert("В выбранной папке не найдена игра Hytale. Обычно она в AppData/Roaming/ (%AppData%)");
      }
    }
  } catch (error) {
    console.error("Ошибка выбора пути:", error);
    statusText.textContent = `Ошибка: ${error}`;
  }
}

async function updateGamePathDisplay() {
  try {
    const hytaleRootPath = await invoke<string>("get_hytale_root_path");
    gamePathText.textContent = `Путь: ${hytaleRootPath}`;
  } catch (e) {
    gamePathText.textContent = `Путь: не найден`;
    console.error(e);
  }
}

async function checkStatus() {
  try {
    await updateGamePathDisplay();
    
    ruInstalled = await invoke<boolean>("check_ru_exists");
    
    if (ruInstalled) {
      actionBtn.textContent = "Удалить русский язык";
      statusText.textContent = "Русский язык установлен ✓";
    } else {
      actionBtn.textContent = "Установить русский язык";
      statusText.textContent = "Русский язык не установлен";
    }
    
    actionBtn.disabled = false;
  } catch (err) {
    actionBtn.textContent = "Игра не найдена";
    statusText.textContent = "Не удалось найти Hytale. Укажите путь вручную.";
    gamePathText.textContent = "Путь: не найден";
    actionBtn.disabled = true;
  }
}

async function handleAction() {
  actionBtn.disabled = true;
  statusText.textContent = ruInstalled ? "Удаление..." : "Установка...";
  
  try {
    if (ruInstalled) {
      await invoke("remove_ru_cmd");
      await invoke("restore_original_cmd");
      statusText.textContent = "Русский язык удалён";
    } else {
      await invoke("install_ru_cmd");
      statusText.textContent = "Русский язык установлен!";
    }
    
    await checkStatus();
  } catch (err) {
    statusText.textContent = `Ошибка: ${err}`;
    actionBtn.disabled = false;
  }
}

actionBtn.addEventListener("click", handleAction);
folderBtn.addEventListener("click", selectGamePath);

document.addEventListener('DOMContentLoaded', () => {
  gamePathText.textContent = '';
  statusText.textContent = 'Проверка...';
  
  checkStatus();
});