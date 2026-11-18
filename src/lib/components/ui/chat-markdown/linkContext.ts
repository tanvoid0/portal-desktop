import { getContext, setContext } from 'svelte';

const KEY = Symbol('chatLinkHandler');

/** Return true when the host handled the link (ChatMarkdown then skips the default navigation). */
export type ChatLinkHandler = (href: string) => boolean;

export const provideChatLinkHandler = (h: ChatLinkHandler) => setContext(KEY, h);
export const useChatLinkHandler = () => getContext<ChatLinkHandler | undefined>(KEY);

const RUN_KEY = Symbol('chatRunHandler');

/** Run a shell code block from the chat. Hosts without a terminal provide none, and the Run button is dropped. */
export type ChatRunHandler = (command: string) => void;

export const provideChatRunHandler = (h: ChatRunHandler) => setContext(RUN_KEY, h);
export const useChatRunHandler = () => getContext<ChatRunHandler | undefined>(RUN_KEY);
