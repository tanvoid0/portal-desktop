import { getContext, setContext } from 'svelte';

const KEY = Symbol('chatLinkHandler');

/** Return true when the host handled the link (ChatMarkdown then skips the default navigation). */
export type ChatLinkHandler = (href: string) => boolean;

export const provideChatLinkHandler = (h: ChatLinkHandler) => setContext(KEY, h);
export const useChatLinkHandler = () => getContext<ChatLinkHandler | undefined>(KEY);
