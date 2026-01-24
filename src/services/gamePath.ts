import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";

export type ValidatePathResult =
  | { ok: true; validPath: string; ruInstalled: boolean; warning?: "check_failed" }
  | { ok: false; reason: "not_found" };

export function cutToHytaleRoot(path: string): string {
  const parts = path.split(/[\\/]/);
  const idx = parts.findIndex(p => p === "Hytale");
  if (idx === -1) return path;
  return parts.slice(0, idx + 1).join(path.includes("\\") ? "\\" : "/");
}

export async function selectGamePath(): Promise<string | null> {
  try {
    const selected = await open({ 
      directory: true, 
      multiple: false, 
      title: "Выберите корневую папку Hytale",
    });
    if (typeof selected === "string") {
      return selected;
    }
  } catch (error) {
    console.error(error);
  }
  return null;
}

export async function openGamePath(path: string): Promise<boolean> {
  try {
    await openPath(path);
		return true;
  } catch (error) {
    console.error("Ошибка при открытии проводника:", error);
		return false;
	}
}

export async function validatePath(root: string): Promise<ValidatePathResult> {
	try {
		const validPath = await invoke<string>("validate_custom_path", { path: root });
		try {
			const ruInstalled = await checkRUInstalled(validPath);
			return { ok: true, validPath, ruInstalled };
		} catch (error) {
			console.error("Error checking RU lang on game dir: ", error);
			return { ok: true, validPath, ruInstalled: false, warning: "check_failed" };
		}
	} catch (error) {
		return { ok: false, reason: "not_found" };
	}
}

async function checkRUInstalled(validPath: string): Promise<boolean> {
	return invoke<boolean>("check_ru_installed", { path: validPath });
}