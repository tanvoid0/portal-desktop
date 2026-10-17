import { invokeClient } from '$lib/utils/invokeClient';
import type { ProjectModule } from '$lib/domains/projects/types';

export const fetchAllModules = () => invokeClient.post<ProjectModule[]>('get_all_project_modules');

/** Rescans every project's folders; slow on a big list, so call it in the background. */
export const refreshAllModules = () =>
  invokeClient.post<ProjectModule[]>('refresh_all_project_modules');

export const refreshProjectModules = (projectId: number) =>
  invokeClient.post<ProjectModule[]>('refresh_project_modules', { projectId });
