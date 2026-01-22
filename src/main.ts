import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import { relaunch } from "@tauri-apps/plugin-process";

const actionBtn = document.getElementById("action-btn") as HTMLButtonElement;
const folderBtn = document.getElementById("folder-btn") as HTMLButtonElement;
const gamePathTooltip = document.getElementById("gamepath-tooltip") as HTMLSpanElement;
const gamePathIconBtn = document.getElementById("gamepath-icon-btn") as HTMLButtonElement;
const toastContainer = document.getElementById("toast-container") as HTMLDivElement;

let validPath: string | null = null;
let ruInstalled = false;

interface UpdateInfo {
  version: string;
  date?: string;
  body?: string;
}

async function init() {
  try {
    const saved = await invoke<string | null>("get_saved_path");
    if (saved) {
      await validateAndSetPath(saved);
    } else {
      updateUIStatus();

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

  try {
    const window = getCurrentWindow();
    await window.show();
  } catch (err) {
    console.error("Ошибка при отображении окна:", err);
  }

  // Проверяем обновления после показа окна
  setTimeout(() => {
    checkForUpdates();
  }, 2000); // Ждём 2 секунды после запуска
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
  
  requestAnimationFrame(() => {
    toast.classList.add("show");
  });
  
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
    const selected = await open({ 
      directory: true, 
      multiple: false, 
      title: "Выберите корневую папку Hytale",
    });
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

    ruInstalled = await invoke<boolean>("check_ru_installed", { path: validPath });
    updateUIStatus();
  } catch (err) {
    showToast(`Ошибка: ${err}`, "status-error");
  } finally {
    actionBtn.disabled = false;
  }
});

async function checkForUpdates() {
  try {
    const updateInfo: UpdateInfo | null = await invoke("check_for_updates");

    if (updateInfo) {
      showUpdateToast(updateInfo);
    }
  } catch (err) {
    console.error("Ошибка проверки обновлений:", err);
  }
}

function showUpdateToast(updateInfo: UpdateInfo) {
  const toast = document.createElement("div");
  toast.className = "update-toast";
  toast.innerHTML = `
    <div class="update-toast-content">
      <div class="update-toast-text">
        Обновиться до ${updateInfo.version}?
      </div>
      <div class="update-toast-buttons">
        <button class="btn-secondary update-btn-later">Позже</button>
        <button class="btn-success update-btn-yes">Да</button>
      </div>
    </div>
  `;

  document.body.appendChild(toast);

  // Анимация появления
  setTimeout(() => {
    toast.classList.add("show");
  }, 100);

  // Обработчики кнопок
  const laterBtn = toast.querySelector(".update-btn-later") as HTMLButtonElement;
  const yesBtn = toast.querySelector(".update-btn-yes") as HTMLButtonElement;

  laterBtn.addEventListener("click", () => {
    hideUpdateToast(toast);
  });

  yesBtn.addEventListener("click", async () => {
    yesBtn.disabled = true;
    yesBtn.textContent = "Устанавливаем...";

    try {
      await invoke("install_update");
      showToast("Обновление установлено! Перезапустите приложение.", "status-success");

      // Перезапуск приложения
      setTimeout(async () => {
        await relaunch();
      }, 2000);

    } catch (err) {
      console.error("Ошибка установки обновления:", err);
      showToast("Ошибка установки обновления", "status-error");
      yesBtn.disabled = false;
      yesBtn.textContent = "Да";
    }

    hideUpdateToast(toast);
  });
}

function hideUpdateToast(toast: HTMLElement) {
  toast.classList.remove("show");
  setTimeout(() => {
    if (toast.parentNode) {
      toast.parentNode.removeChild(toast);
    }
  }, 300);
}

document.addEventListener("DOMContentLoaded", init);