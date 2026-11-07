import { redirect } from '@sveltejs/kit';

// Old tab deep links (/sdk?tab=managers) now live at their own routes.
const TAB_ROUTES: Record<string, string> = {
  managers: '/sdk/managers',
  services: '/sdk/services',
  versions: '/sdk/versions',
};

export const ssr = false;

export const load = ({ url }) => {
  const target = TAB_ROUTES[url.searchParams.get('tab') ?? ''];
  if (target) redirect(307, target);
};
