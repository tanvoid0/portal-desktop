import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

// The tab hub was split into three standalone routes; keep old links working.
const TAB_ROUTES: Record<string, string> = {
  languages: '/settings/languages',
  'package-managers': '/settings/package-managers',
  frameworks: '/settings/frameworks',
};

export const load: PageLoad = ({ url }) => {
  throw redirect(307, TAB_ROUTES[url.searchParams.get('tab') ?? ''] ?? '/settings/languages');
};
