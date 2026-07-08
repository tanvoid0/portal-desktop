const STORAGE_KEY = "portal.ai.chatCatalog";

export interface ChatCatalogPrefs {
  backendProvider: string;
  model: string;
}

export function loadChatCatalogPrefs(): Partial<ChatCatalogPrefs> | null {
  if (typeof localStorage === "undefined") return null;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<ChatCatalogPrefs>;
    if (!parsed || typeof parsed !== "object") return null;
    return parsed;
  } catch {
    return null;
  }
}

export function saveChatCatalogPrefs(prefs: ChatCatalogPrefs): void {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs));
  } catch {
    // ignore quota / private mode
  }
}
