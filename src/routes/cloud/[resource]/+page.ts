import { error } from '@sveltejs/kit';
import { getResourceDescriptor } from '$lib/domains/cloud/core/resourceDescriptors';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ params }) => {
  const descriptor = getResourceDescriptor(params.resource);
  if (!descriptor) {
    error(404, `Unknown cloud resource "${params.resource}"`);
  }
  return { descriptor };
};
