import { invoke } from "@tauri-apps/api/core";
import { PlatformInfo } from "../types";
import { installUpdate, openReleasePage, UpdateInfo } from "../services/update";

interface RenderObject {
	toast: HTMLElement,
	message: string,
	buttons: Array<{ className: string; text: string }>
}

export async function showUpdateToast(updateInfo: UpdateInfo) {
	const platformInfo: PlatformInfo = await invoke("get_platform_info");
	const updateSupported = platformInfo.update_supported;

	const toast = document.createElement("div");
	toast.className = "update-toast";

	if (!updateSupported) {
		renderUpdateToast({
			toast,
			message: `Новое обновление v${updateInfo.version}`,
			buttons: [
				{ className: "btn-secondary update-btn-later", text: "Позже" },
				{ className: "btn-primary update-btn-open", text: "Открыть" },
			],
		});
	} else {
		renderUpdateToast({
			toast,
			message: `Обновиться до ${updateInfo.version}?`,
			buttons: [
				{ className: "btn-secondary update-btn-later", text: "Позже" },
				{ className: "btn-success update-btn-yes", text: "Да" },
			],
		});
	}

	document.body.appendChild(toast);

	setTimeout(() => {
		toast.classList.add("show");
	}, 100);

	const laterBtn = toast.querySelector(".update-btn-later") as HTMLButtonElement;

	laterBtn.addEventListener("click", () => {
		hideUpdateToast(toast);
	});

	if (!updateSupported) {
		const openBtn = toast.querySelector(".update-btn-open") as HTMLButtonElement;

		openBtn.addEventListener("click", async () => {
			openBtn.disabled = true;
			openBtn.textContent = "Открываем...";

			const ok = await openReleasePage(updateInfo);
			if (!ok) {
				openBtn.disabled = false;
				openBtn.textContent = "Открыть"
			}
			hideUpdateToast(toast);
		});
	} else {
		const yesBtn = toast.querySelector(".update-btn-yes") as HTMLButtonElement;

		yesBtn.addEventListener("click", async () => {
			yesBtn.disabled = true;
			yesBtn.textContent = "Устанавливаем...";

			const ok = await installUpdate();
			if (!ok) {
				yesBtn.disabled = false;
				yesBtn.textContent = "Да"
			}
			hideUpdateToast(toast);
		});
	}
}

function renderUpdateToast(object: RenderObject) {
	object.toast.innerHTML = `
			<div class="update-toast-content">
				<div class="update-toast-text">
					Обновиться до ${object.message}?
				</div>
				<div class="update-toast-buttons">
					${object.buttons
						.map(button => `<button class="${button.className}">${button.text}</button>`)
						.join("")}
				</div>
			</div>
		`;
}

export function showToast(text: string, className: string) {
  const existingToasts = document.querySelectorAll('.toast');
  if (existingToasts.length >= 3) {
    const oldestToast = existingToasts[0] as HTMLElement;
    oldestToast.remove();
  }

  const toast = document.createElement("div");
  toast.className = `toast ${className}`;
  toast.textContent = text;

  document.body.appendChild(toast);

  updateToastPositions();

  requestAnimationFrame(() => {
    toast.classList.add("show");
  });

  setTimeout(() => {
    toast.classList.remove("show");
    toast.classList.add("hide");
    setTimeout(() => {
      toast.remove();
      updateToastPositions();
    }, 300);
  }, 5000);
}
  
function hideUpdateToast(toast: HTMLElement) {
	toast.classList.remove("show");
	setTimeout(() => {
		if (toast.parentNode) {
			toast.parentNode.removeChild(toast);
		}
	}, 300);
}

function updateToastPositions() {
  const toasts = document.querySelectorAll('.toast');
  toasts.forEach((toast, index) => {
    (toast as HTMLElement).style.top = `${20 + (index * 60)}px`;
  });
}
