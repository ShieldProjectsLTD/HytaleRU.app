import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";

const actionBtn = document.getElementById("action-btn") as HTMLButtonElement;
const folderBtn = document.getElementById("folder-btn") as HTMLButtonElement;
const gamePathTooltip = document.getElementById("gamepath-tooltip") as HTMLSpanElement;
const gamePathIconBtn = document.getElementById("gamepath-icon-btn") as HTMLButtonElement;
const toastContainer = document.getElementById("toast-container") as HTMLDivElement;

let validPath: string | null = null;
let ruInstalled = false;

async function init() {
  try {
    const saved = await invoke<string | null>("get_saved_path");
    if (saved) {
      await validateAndSetPath(saved);
    } else {
      updateUIStatus();
      
      // Пытаемся найти игру автоматически
      try {
        const found = await invoke<string | null>("find_game_automatically");
        if (found) {
          await validateAndSetPath(found);
        } else {
          showToast("Путь не найден", "status-error");
          updateUIStatus();
        }
      } catch (err) {
        console.error("Ошибка при автоматическом поиске:", err);
        showToast("Путь не найден", "status-error");
        updateUIStatus();
      }
    }
  } catch (err) {
    console.error("Ошибка при загрузке сохраненного пути:", err);
    showToast("Путь не найден", "status-error");
    updateUIStatus();
  }
}

async function validateAndSetPath(path: string) {
  const root = cutToHytaleRoot(path);

  try {
    validPath = await invoke<string>("validate_custom_path", { path: root });

    ruInstalled = await invoke<boolean>("check_ru_installed", { path: validPath });

    if (ruInstalled) {
      showToast("Русский язык установлен", "status-success");
    } else {
      showToast("Русский язык не установлен", "status-neutral");
    }
  } catch (err) {
    validPath = null;
    ruInstalled = false;
    showToast("Путь не найден", "status-error");
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
  gamePathTooltip.textContent = validPath || "Путь не выбран";
  const gamePathContainer = document.getElementById("gamepath-container") as HTMLDivElement;
  if (validPath) {
    gamePathContainer.style.display = "flex";
  } else {
    gamePathContainer.style.display = "none";
  }
}

function showToast(text: string, className: string) {
  const toast = document.createElement("div");
  toast.className = `toast ${className}`;
  toast.textContent = text;
  
  toastContainer.appendChild(toast);
  
  // Анимация появления
  requestAnimationFrame(() => {
    toast.classList.add("show");
  });
  
  // Автоматическое скрытие через 5 секунд
  setTimeout(() => {
    toast.classList.remove("show");
    toast.classList.add("hide");
    setTimeout(() => {
      toast.remove();
    }, 300);
  }, 5000);
}

function updateUIStatus() {
  if (!validPath) {
    actionBtn.disabled = true;
    actionBtn.textContent = "Выберите папку Hytale";
    folderBtn.style.opacity = "1.0";
  } else {
    actionBtn.disabled = false;
    actionBtn.textContent = ruInstalled ? "Удалить русский язык" : "Установить русский язык";
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

async function openGamePath() {
  if (!validPath) return;
  try {
    await openPath(validPath);
  } catch (err) {
    console.error("Ошибка при открытии проводника:", err);
    showToast("Не удалось открыть проводник", "status-error");
  }
}

gamePathIconBtn.addEventListener("click", openGamePath);
folderBtn.addEventListener("click", selectGamePath);
actionBtn.addEventListener("click", async () => {
  if (!validPath) return;
  actionBtn.disabled = true;

  try {
    if (ruInstalled) {
      await invoke("remove_ru_cmd");
      await invoke("restore_original_cmd");
      showToast("Удалён русский язык", "status-neutral");
    } else {
      await invoke("install_ru_cmd");
      showToast("Русский язык установлен", "status-success");
    }

    // После действия проверяем ещё раз
    ruInstalled = await invoke<boolean>("check_ru_installed", { path: validPath });
    updateUIStatus();
  } catch (err) {
    showToast(`Ошибка: ${err}`, "status-error");
  } finally {
    actionBtn.disabled = false;
  }
});

document.addEventListener("DOMContentLoaded", init);