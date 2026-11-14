import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

// The Run/Sessions sub-tabs are gone (#546) — one Agents workspace now hosts
// both. `?cwd=` still prefills the launcher, just read on the merged page.
export const load: PageLoad = ({ url }) => {
  throw redirect(307, `/ai/agents${url.search}`);
};
