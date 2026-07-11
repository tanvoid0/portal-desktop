import type { Deployment, DockerContainer } from "../types";
import {
  containerStatusGroup,
  isContainerRunning,
  shortImageName,
} from "./format";

export type WorkloadGroupKind = "portal" | "compose" | "image" | "standalone";
export type GroupByMode = "stack" | "image" | "network" | "flat";
export type WorkloadSection = "portal" | "compose" | "standalone" | "image";

export interface WorkloadGroup {
  id: string;
  name: string;
  kind: WorkloadGroupKind;
  section: WorkloadSection;
  containers: DockerContainer[];
  deployment?: Deployment;
  images: string[];
  networks: string[];
  subtitle?: string;
  projectPath?: string;
}

export interface WorkloadGroupRollup {
  total: number;
  running: number;
  totalCpu: number;
  totalMemory: number;
}

function normalizeContainerName(name: string): string {
  return name.replace(/^\//, "");
}

function containerMatchesId(container: DockerContainer, id?: string): boolean {
  if (!id) return false;
  const normalizedId = id.toLowerCase();
  return (
    container.id.toLowerCase() === normalizedId ||
    container.id.toLowerCase().startsWith(normalizedId) ||
    normalizedId.startsWith(container.id.toLowerCase())
  );
}

/** Fallback when compose labels are missing (v1 underscore naming). */
export function inferComposeFromName(name: string): {
  project?: string;
  service?: string;
} {
  const normalized = normalizeContainerName(name);
  const underscoreMatch = normalized.match(/^(.+?)_([^_]+)_(\d+)$/);
  if (underscoreMatch) {
    return { project: underscoreMatch[1], service: underscoreMatch[2] };
  }
  return {};
}

export function resolveComposeProject(container: DockerContainer): string | undefined {
  if (container.composeProject) return container.composeProject;
  return inferComposeFromName(container.name).project;
}

export function resolveComposeService(container: DockerContainer): string | undefined {
  if (container.composeService) return container.composeService;
  return inferComposeFromName(container.name).service;
}

export function computeGroupRollup(containers: DockerContainer[]): WorkloadGroupRollup {
  const runningContainers = containers.filter((c) =>
    isContainerRunning(c.status),
  );
  return {
    total: containers.length,
    running: runningContainers.length,
    totalCpu: runningContainers.reduce(
      (sum, c) => sum + (c.resourceStats?.cpuPercent ?? 0),
      0,
    ),
    totalMemory: runningContainers.reduce(
      (sum, c) => sum + (c.resourceStats?.memoryBytes ?? 0),
      0,
    ),
  };
}

function uniqueImages(containers: DockerContainer[]): string[] {
  return [...new Set(containers.map((c) => shortImageName(c.image)))].sort();
}

function uniqueNetworks(containers: DockerContainer[]): string[] {
  const networks = new Set<string>();
  for (const container of containers) {
    for (const network of container.networks ?? []) {
      networks.add(network);
    }
  }
  return [...networks].sort();
}

function sortContainers(containers: DockerContainer[]): DockerContainer[] {
  return [...containers].sort((a, b) => {
    const serviceA = resolveComposeService(a) ?? a.name;
    const serviceB = resolveComposeService(b) ?? b.name;
    return serviceA.localeCompare(serviceB);
  });
}

function buildGroup(
  id: string,
  name: string,
  kind: WorkloadGroupKind,
  section: WorkloadSection,
  containers: DockerContainer[],
  extras: Partial<WorkloadGroup> = {},
): WorkloadGroup {
  const sorted = sortContainers(containers);
  return {
    id,
    name,
    kind,
    section,
    containers: sorted,
    images: uniqueImages(sorted),
    networks: uniqueNetworks(sorted),
    ...extras,
  };
}

function filterByStatus(
  containers: DockerContainer[],
  statusFilter?: "running" | "stopped" | "other",
): DockerContainer[] {
  if (!statusFilter) return containers;
  return containers.filter(
    (c) => containerStatusGroup(c.status) === statusFilter,
  );
}

function filterGroupsByStatus(
  groups: WorkloadGroup[],
  statusFilter?: "running" | "stopped" | "other",
): WorkloadGroup[] {
  if (!statusFilter) return groups;

  return groups
    .map((group) => ({
      ...group,
      containers: filterByStatus(group.containers, statusFilter),
    }))
    .filter((group) => {
      if (group.containers.length > 0) return true;
      // Portal deployments without a linked container appear under Stopped
      if (group.deployment && statusFilter === "stopped") return true;
      return false;
    });
}

function filterBySearch(
  groups: WorkloadGroup[],
  query: string,
): WorkloadGroup[] {
  if (!query.trim()) return groups;
  const normalized = query.toLowerCase();

  return groups
    .map((group) => {
      const groupMatch =
        group.name.toLowerCase().includes(normalized) ||
        group.subtitle?.toLowerCase().includes(normalized) ||
        group.projectPath?.toLowerCase().includes(normalized) ||
        group.images.some((img) => img.toLowerCase().includes(normalized));

      const matchingContainers = group.containers.filter(
        (c) =>
          c.name.toLowerCase().includes(normalized) ||
          c.image.toLowerCase().includes(normalized) ||
          c.id.toLowerCase().includes(normalized) ||
          resolveComposeService(c)?.toLowerCase().includes(normalized),
      );

      if (groupMatch) return group;
      if (matchingContainers.length === 0) return null;

      return { ...group, containers: matchingContainers };
    })
    .filter((group): group is WorkloadGroup => group !== null);
}

function groupByStack(
  containers: DockerContainer[],
  deployments: Deployment[],
): WorkloadGroup[] {
  const groups: WorkloadGroup[] = [];
  const assigned = new Set<string>();

  const dockerDeployments = deployments.filter((d) => d.type === "docker");

  for (const deployment of dockerDeployments) {
    const linked = containers.filter(
      (c) =>
        containerMatchesId(c, deployment.containerId) ||
        containerMatchesId(c, deployment.container?.id),
    );
    for (const container of linked) {
      assigned.add(container.id);
    }

    groups.push(
      buildGroup(
        `portal-${deployment.id}`,
        deployment.name,
        "portal",
        "portal",
        linked,
        {
          deployment,
          subtitle: deployment.dockerImageName ?? deployment.projectType,
          projectPath: deployment.projectPath,
        },
      ),
    );
  }

  const composeMap = new Map<string, DockerContainer[]>();
  for (const container of containers) {
    if (assigned.has(container.id)) continue;

    const project = resolveComposeProject(container);
    if (!project) continue;

    const list = composeMap.get(project) ?? [];
    list.push(container);
    composeMap.set(project, list);
    assigned.add(container.id);
  }

  for (const [project, projectContainers] of composeMap) {
    groups.push(
      buildGroup(`compose-${project}`, project, "compose", "compose", projectContainers, {
        subtitle: `${projectContainers.length} services`,
      }),
    );
  }

  const standalone = containers.filter((c) => !assigned.has(c.id));
  for (const container of standalone) {
    groups.push(
      buildGroup(
        `standalone-${container.id}`,
        resolveComposeService(container) ?? normalizeContainerName(container.name),
        "standalone",
        "standalone",
        [container],
        { subtitle: shortImageName(container.image) },
      ),
    );
  }

  return groups;
}

function groupByImage(containers: DockerContainer[]): WorkloadGroup[] {
  const imageMap = new Map<string, DockerContainer[]>();

  for (const container of containers) {
    const image = shortImageName(container.image);
    const list = imageMap.get(image) ?? [];
    list.push(container);
    imageMap.set(image, list);
  }

  return [...imageMap.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([image, imageContainers]) =>
      buildGroup(`image-${image}`, image, "image", "image", imageContainers, {
        subtitle: `${imageContainers.length} container${imageContainers.length === 1 ? "" : "s"}`,
      }),
    );
}

function groupByNetwork(containers: DockerContainer[]): WorkloadGroup[] {
  const networkMap = new Map<string, DockerContainer[]>();

  for (const container of containers) {
    const networks =
      container.networks && container.networks.length > 0
        ? container.networks
        : ["(no network)"];

    for (const network of networks) {
      const list = networkMap.get(network) ?? [];
      list.push(container);
      networkMap.set(network, list);
    }
  }

  return [...networkMap.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([network, networkContainers]) =>
      buildGroup(
        `network-${network}`,
        network,
        "compose",
        "compose",
        networkContainers,
        {
          subtitle: `${networkContainers.length} attached`,
        },
      ),
    );
}

export function groupWorkloads(
  containers: DockerContainer[],
  deployments: Deployment[],
  mode: GroupByMode,
  options: {
    statusFilter?: "running" | "stopped" | "other";
    searchQuery?: string;
  } = {},
): WorkloadGroup[] {
  let groups: WorkloadGroup[];

  switch (mode) {
    case "image":
      groups = groupByImage(containers);
      break;
    case "network":
      groups = groupByNetwork(containers);
      break;
    case "flat":
      groups = containers.map((container) =>
        buildGroup(
          `flat-${container.id}`,
          normalizeContainerName(container.name),
          "standalone",
          "standalone",
          [container],
          { subtitle: shortImageName(container.image) },
        ),
      );
      break;
    case "stack":
    default:
      groups = groupByStack(containers, deployments);
      break;
  }

  groups = filterGroupsByStatus(groups, options.statusFilter);
  groups = filterBySearch(groups, options.searchQuery ?? "");

  const sectionOrder: Record<WorkloadSection, number> = {
    portal: 0,
    compose: 1,
    standalone: 2,
    image: 3,
  };

  return groups.sort((a, b) => {
    const sectionDiff = sectionOrder[a.section] - sectionOrder[b.section];
    if (sectionDiff !== 0) return sectionDiff;
    return a.name.localeCompare(b.name);
  });
}

export function sectionLabel(section: WorkloadSection): string {
  switch (section) {
    case "portal":
      return "Portal Workloads";
    case "compose":
      return "Compose Stacks";
    case "standalone":
      return "Standalone Containers";
    case "image":
      return "By Image";
  }
}

export function kindBadgeLabel(kind: WorkloadGroupKind): string {
  switch (kind) {
    case "portal":
      return "Portal";
    case "compose":
      return "Compose";
    case "image":
      return "Image";
    case "standalone":
      return "Standalone";
  }
}
