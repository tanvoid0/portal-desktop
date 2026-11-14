import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

// A session's detail view is now the focused session in the Agents workspace (#546).
export const load: PageLoad = ({ params }) => {
  throw redirect(307, `/ai/agents?session=${params.id}`);
};
