export const AI_CODER_PATH = '/ai/coder';

import { AI_CHAT_PATH } from '$lib/config/ai-nav';
import { AGENTS_BASE } from '$lib/config/agents-tabs';

export { AI_CHAT_PATH };

export const AI_TABS = [
  {
    id: 'home',
    label: 'Home',
    icon: 'home',
    url: AI_CHAT_PATH,
  },
  {
    id: 'code',
    label: 'Code',
    icon: 'code',
    url: AI_CODER_PATH,
  },
  {
    id: 'agents',
    label: 'Agents',
    icon: 'robot',
    url: AGENTS_BASE,
  },
] as const;

export type AiTabId = (typeof AI_TABS)[number]['id'];

export function getActiveAiTab(pathname: string): AiTabId {
  if (pathname === AI_CODER_PATH || pathname.startsWith(`${AI_CODER_PATH}/`)) {
    return 'code';
  }
  if (pathname === AGENTS_BASE || pathname.startsWith(`${AGENTS_BASE}/`)) {
    return 'agents';
  }
  return 'home';
}

export function isAiCoderRoute(pathname: string): boolean {
  return pathname === AI_CODER_PATH || pathname.startsWith(`${AI_CODER_PATH}/`);
}

export function isAiSectionRoute(pathname: string): boolean {
  return pathname === '/ai' || pathname.startsWith('/ai/');
}

export function getAiTabBreadcrumb(pathname: string): {
  label: string;
  href: string;
  icon: string;
} {
  if (isAiCoderRoute(pathname)) {
    return { label: 'Code', href: AI_CODER_PATH, icon: 'code' };
  }
  if (pathname === AGENTS_BASE || pathname.startsWith(`${AGENTS_BASE}/`)) {
    return { label: 'Agents', href: AGENTS_BASE, icon: 'robot' };
  }
  if (pathname === '/ai/logs') {
    return { label: 'AI Logs', href: '/ai/logs', icon: 'file-text' };
  }
  return { label: 'Home', href: AI_CHAT_PATH, icon: 'message-circle' };
}
