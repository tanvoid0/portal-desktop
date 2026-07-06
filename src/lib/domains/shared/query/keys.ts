export const queryKeys = {
  dashboard: {
    overview: ["dashboard", "overview"] as const,
  },
  projects: {
    all: ["projects"] as const,
    detail: (id: number | string) => ["projects", id] as const,
  },
  tasks: {
    all: ["tasks"] as const,
    detail: (id: string) => ["tasks", id] as const,
  },
  cloud: {
    resources: (type: string, namespace: string) =>
      ["cloud", type, namespace] as const,
  },
  sdk: {
    managers: ["sdk", "managers"] as const,
    versions: (name: string) => ["sdk", name, "versions"] as const,
  },
};
