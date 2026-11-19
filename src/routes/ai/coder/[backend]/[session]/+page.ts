import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

// /ai/coder/<backend>/<session> — a stable external link; the workspace keeps ?session= in sync.
export const load: PageLoad = ({ params, url }) => {
  const q = new URLSearchParams(url.search);
  q.set('session', params.session);
  throw redirect(307, `/ai/coder/${params.backend}?${q}`);
};
