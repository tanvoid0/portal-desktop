import { redirect } from '@sveltejs/kit';
import { agentTabPath } from '$lib/config/agents-tabs';
import type { PageLoad } from './$types';

// The Agents tab merged into Code, one engine tab per backend (#725).
export const load: PageLoad = ({ url }) => {
  throw redirect(307, `${agentTabPath()}${url.search}`);
};
