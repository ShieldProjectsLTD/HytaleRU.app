import { invoke } from "@tauri-apps/api/core";
import { showToast, showUpdateToast } from "../ui/toast";
import { relaunch } from "@tauri-apps/plugin-process";

export interface UpdateInfo {
	version: string;
	date?: string;
	body?: string;
}

export async function checkForUpdates() {
	try {
		const updateInfo: UpdateInfo | null = await invoke("check_for_updates");

		if (updateInfo) {
			await showUpdateToast(updateInfo);
		}
	} catch (err) {
		console.error("Ошибка проверки обновлений:", err);
	}
}

export async function installUpdate(): Promise<boolean> {
	try {
		await invoke("install_update");
		showToast("Обновление установлено! Перезапустите приложение.", "status-success");

		setTimeout(async () => {
			await relaunch();
		}, 2000);
		return true;
	} catch (error) {
		console.error("Error installing update: ", error);
		showToast("Ошибка установки обновления", "status-error");
		return false;
	}
}

export async function openReleasePage(updateInfo: UpdateInfo): Promise<boolean> {
	try {
		await invoke("open_release_page", { version: updateInfo.version })
		showToast("Открыта страница релиза", "status-success")
		return true;
	} catch (error) {
		console.error("Error open release page: ", error);
		showToast("Не удалось открыть", "status-error");
		return false;
	}
}