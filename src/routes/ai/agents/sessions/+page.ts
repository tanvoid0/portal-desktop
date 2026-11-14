import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

// The sessions list is now the Agents workspace's sidebar (#546).
export const load: PageLoad = () => {
  throw redirect(307, '/ai/agents');
};
