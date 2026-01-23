import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { showToast } from "./ui/toast";
import { checkLocalizationUpdates } from "./services/localization";
import { checkForUpdates } from "./services/update";
import { cutToHytaleRoot, openGamePath, selectGamePath, validatePath } from "./services/gamePath";

const actionBtn = document.getElementById("action-btn")                 as HTMLButtonElement;
const folderBtn = document.getElementById("folder-btn")                 as HTMLButtonElement;
const gamePathIconBtn = document.getElementById("gamepath-icon-btn")    as HTMLButtonElement;
const gamePathTooltip = document.getElementById("gamepath-tooltip")     as HTMLSpanElement;
const versionDisplay = document.getElementById("version-display")       as HTMLDivElement;
const gamePathContainer = document.getElementById("gamepath-container") as HTMLDivElement;

let validPath: string | null = null;
let appVersion: string | null = null;
let ruInstalled: boolean | null = null;
let langVersion: string | null = null;

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
    appVersion = await getVersion();
    updateVersionDisplay();
  } catch (err) {
    console.error("Ошибка при получении версии:", err);
    updateVersionDisplay();
  }

  setTimeout(() => {
    checkForUpdates();
    refreshLocalization();
  }, 2000);
}

async function validateAndSetPath(path: string) {
  const root = cutToHytaleRoot(path);

	const result = await validatePath(root);
  if (result.ok) {
		validPath = result.validPath;
		ruInstalled = result.ruInstalled;
		if (result.warning === "check_failed") {
			showToast("Не удалось проверить RU язык", "status-error");
		} else if (ruInstalled) {
			showToast("Русский язык установлен", "status-success");
		} else {
			showToast("Русский язык не установлен", "status-neutral");
		}
	} else {
		validPath = null;
		ruInstalled = false;
		showToast("Путь не найден", "status-error");
	}

  updateGamePathDisplay();
  updateUIStatus();
}

function updateGamePathDisplay() {
  gamePathTooltip.textContent = validPath || "Путь не выбран";
  if (validPath) {
    gamePathContainer.style.display = "flex";
  } else {
    gamePathContainer.style.display = "none";
  }
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

async function refreshLocalization() {
  const result = await checkLocalizationUpdates();
  if (!result.ok) {
    return;
  }
  langVersion = result.langVersion;
  updateVersionDisplay();
  if (result.updated) {
    showToast("Локализация обновлена", "status-success");
  }
}

gamePathIconBtn.addEventListener("click", () => {
  if (!validPath) {
		return;
	}
  openGamePath(validPath);
});
folderBtn.addEventListener("click", async () => {
  const selected = await selectGamePath();
	if (!selected) {
		return;
	}
	await validateAndSetPath(selected);
});
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

function updateVersionDisplay() {
  const app = appVersion ? `v${appVersion}` : "v?.?.?";
  const lang = langVersion ? `v${langVersion}` : "v?.?.?";
  versionDisplay.textContent = `${app} | langRu: ${lang}`;
}

document.addEventListener("DOMContentLoaded", init);