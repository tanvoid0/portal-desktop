/**
 * Legacy project store — active project selection only.
 * Server data lives in TanStack Query (see projectQueries.ts).
 */

import { writable } from "svelte/store";

interface ProjectSessionState {
  activeProjectId: string | null;
}

const initialState: ProjectSessionState = {
  activeProjectId: null,
};

function createProjectStore() {
  const { subscribe, set, update } = writable<ProjectSessionState>(initialState);

  return {
    subscribe,

    setActiveProject: (id: string | null) => {
      update((state) => ({ ...state, activeProjectId: id }));
    },

    reset: () => set(initialState),
  };
}

export const projectStore = createProjectStore();
