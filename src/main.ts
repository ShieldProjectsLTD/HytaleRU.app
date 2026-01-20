import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

const actionBtn = document.getElementById("action-btn") as HTMLButtonElement;
const folderBtn = document.getElementById("folder-btn") as HTMLButtonElement;
const statusText = document.getElementById("p-status") as HTMLParagraphElement;
const gamePathText = document.getElementById("p-gamepath") as HTMLParagraphElement;

let validPath: string | null = null;
let ruInstalled = false;

async function init() {
  try {
    const saved = await invoke<string | null>("get_saved_path");
    if (saved) {
      await validateAndSetPath(saved);
    } else {
      statusText.textContent = "Ищем игру...";
      updateUIStatus();
      
      // Пытаемся найти игру автоматически
      try {
        const found = await invoke<string | null>("find_game_automatically");
        if (found) {
          await validateAndSetPath(found);
        } else {
          statusText.textContent = "Игра не найдена. Выберите папку вручную";
          updateUIStatus();
        }
      } catch (err) {
        console.error("Ошибка при автоматическом поиске:", err);
        statusText.textContent = "Игра не найдена. Выберите папку вручную";
        updateUIStatus();
      }
    }
  } catch (err) {
    console.error("Ошибка при загрузке сохраненного пути:", err);
    statusText.textContent = "Выберите папку Hytale";
    updateUIStatus();
  }
}

async function validateAndSetPath(path: string) {
  const root = cutToHytaleRoot(path);

  try {
    validPath = await invoke<string>("validate_custom_path", { path: root });

    ruInstalled = await invoke<boolean>("check_ru_installed", { path: validPath });

    statusText.textContent = ruInstalled ? "Русский язык установлен ✓" : "Русский язык не установлен";
  } catch (err) {
    validPath = null;
    ruInstalled = false;
    statusText.textContent = String(err);
  }

  updateGamePathDisplay();
  updateUIStatus();
}

function cutToHytaleRoot(path: string): string {
  const parts = path.split(/[\\/]/);
  const idx = parts.findIndex(p => p === "Hytale");
  if (idx === -1) return path;
  return parts.slice(0, idx + 1).join(path.includes("\\") ? "\\" : "/");
}

function updateGamePathDisplay() {
  gamePathText.textContent = validPath ? `Путь: ${validPath}` : "Путь: не выбран";
}

function updateUIStatus() {
  if (!validPath) {
    actionBtn.disabled = true;
    actionBtn.textContent = "Выберите папку Hytale";
    statusText.style.color = "red";
    // Высвечиваем кнопку "Указать игру" когда игра не найдена
    folderBtn.style.opacity = "1.0";
  } else {
    actionBtn.disabled = false;
    actionBtn.textContent = ruInstalled ? "Удалить русский язык" : "Установить русский язык";
    statusText.style.color = "limegreen";
    // Возвращаем обычную прозрачность когда игра найдена
    folderBtn.style.opacity = "0.4";
  }
}

async function selectGamePath() {
  try {
    const selected = await open({ directory: true, multiple: false, title: "Выберите корневую папку Hytale" });
    if (typeof selected === "string") {
      await validateAndSetPath(selected);
    }
  } catch (err) {
    console.error(err);
  }
}

folderBtn.addEventListener("click", selectGamePath);
actionBtn.addEventListener("click", async () => {
  if (!validPath) return;
  actionBtn.disabled = true;

  try {
    if (ruInstalled) {
      await invoke("remove_ru_cmd");
      await invoke("restore_original_cmd");
      statusText.textContent = "Русский язык удалён";
    } else {
      await invoke("install_ru_cmd");
      statusText.textContent = "Русский язык установлен";
    }

    // После действия проверяем ещё раз
    ruInstalled = await invoke<boolean>("check_ru_installed", { path: validPath });
    updateUIStatus();
  } catch (err) {
    statusText.textContent = `Ошибка: ${err}`;
  } finally {
    actionBtn.disabled = false;
  }
});

document.addEventListener("DOMContentLoaded", init);