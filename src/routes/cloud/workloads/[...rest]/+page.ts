import { redirect } from '@sveltejs/kit';
import { getResourceDescriptor } from '$lib/domains/cloud/core/resourceDescriptors';
import type { PageLoad } from './$types';

// The workload list pages moved to /cloud/<resource>. Detail routes
// (/cloud/workloads/pods/[pod] and friends) are more specific, so they still match
// their own pages and never reach this one.
export const load: PageLoad = ({ params, url }) => {
  const descriptor = getResourceDescriptor(params.rest);
  redirect(307, descriptor ? `/cloud/${descriptor.slug}${url.search}` : '/cloud/workloads');
};
