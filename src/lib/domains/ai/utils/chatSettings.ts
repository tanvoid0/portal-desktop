const STORAGE_KEY = 'portal.ai.chatSettings';

/**
 * Sampling knobs mirror the LM Studio panel. `null` means "leave it to the
 * model" — the field is omitted from the request instead of sent as a default.
 */
export interface ChatSettings {
  systemPrompt: string;
  temperature: number;
  /** When false, `maxTokens` is not sent and the model runs to its own limit. */
  limitResponseLength: boolean;
  maxTokens: number;
  stopStrings: string[];
  topK: number | null;
  topP: number | null;
  minP: number | null;
  repeatPenalty: number | null;
  presencePenalty: number | null;
}

export const DEFAULT_CHAT_SETTINGS: ChatSettings = {
  systemPrompt: '',
  temperature: 0.7,
  limitResponseLength: true,
  maxTokens: 2048,
  stopStrings: [],
  topK: null,
  topP: null,
  minP: null,
  repeatPenalty: null,
  presencePenalty: null,
};

/**
 * Provider-specific sampling fields, merged into the OpenAI-compatible body by
 * the Rust provider. Omits anything the user left off.
 */
export function samplingExtras(settings: ChatSettings): Record<string, unknown> | undefined {
  const extras: Record<string, unknown> = {};
  if (settings.topK != null) extras.top_k = settings.topK;
  if (settings.topP != null) extras.top_p = settings.topP;
  if (settings.minP != null) extras.min_p = settings.minP;
  if (settings.repeatPenalty != null) extras.repeat_penalty = settings.repeatPenalty;
  if (settings.presencePenalty != null) extras.presence_penalty = settings.presencePenalty;
  if (settings.stopStrings.length) extras.stop = settings.stopStrings;
  return Object.keys(extras).length ? extras : undefined;
}

export function loadChatSettings(): ChatSettings {
  if (typeof localStorage === 'undefined') return { ...DEFAULT_CHAT_SETTINGS };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULT_CHAT_SETTINGS };
    const parsed = JSON.parse(raw) as Partial<ChatSettings>;
    return { ...DEFAULT_CHAT_SETTINGS, ...parsed };
  } catch {
    return { ...DEFAULT_CHAT_SETTINGS };
  }
}

export function saveChatSettings(settings: ChatSettings): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch {
    // ignore quota / private mode
  }
}
