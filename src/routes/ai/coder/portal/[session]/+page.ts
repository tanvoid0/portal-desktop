import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

// /ai/coder/portal/<session> — Portal's own coder keeps its focused thread in ?id=.
export const load: PageLoad = ({ params, url }) => {
  const q = new URLSearchParams(url.search);
  q.set('id', params.session);
  throw redirect(307, `/ai/coder?${q}`);
};
