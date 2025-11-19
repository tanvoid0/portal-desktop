export interface UpdateInfo {
	version: string;
	date?: string;
	body?: string;
	available: boolean;
}

export interface UpdateStatus {
	checking: boolean;
	available: boolean;
	installing: boolean;
	error: string | null;
	info: UpdateInfo | null;
}

