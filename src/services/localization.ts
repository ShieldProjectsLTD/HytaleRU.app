import { invoke } from "@tauri-apps/api/core";
import { LocalizationUpdateInfo } from "../types";

export type LocalizationUpdateResult =
  | { ok: true; langVersion: string | null; updateAvailable: boolean; updated: boolean }
  | { ok: false };

export async function checkLocalizationUpdates(): Promise<LocalizationUpdateResult> {
  try {
    const info: LocalizationUpdateInfo | null = await invoke("check_localization_updates");
    const currentVersion = info?.current_version ?? null;

    if (!info || !info.update_available) {
      return { ok: true, langVersion: currentVersion, updateAvailable: false, updated: false };
    }

		try {
			const updated = await invoke<boolean>("auto_update_localization");
			const langVersion = updated ? info.latest_version : currentVersion;
			return { ok: true, langVersion, updateAvailable: true, updated };
		} catch (error) {
			console.error("Ошибка авто‑обновления локализации:", error);
			return { ok: true, langVersion: currentVersion, updateAvailable: true, updated: false};
		}

  } catch (error) {
    console.error("Ошибка проверки локализации:", error);
    return { ok: false };
  }
}