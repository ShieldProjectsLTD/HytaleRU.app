export interface PlatformInfo {
	platform: string;
	update_supported: boolean;
}

export interface LocalizationUpdateInfo {
	current_version: string | null;
	latest_version: string;
	update_available: boolean;
	download_url: string | null;
	changelog: string | null;
}