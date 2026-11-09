// One descriptor per cloud list route. /cloud/[resource] renders whatever is in here,
// so adding a resource type means adding a row below — not a new route file.

import { ResourceStatus, ResourceType, type ICloudResource } from '$lib/domains/cloud/core/types';
import type { PageStat } from '$lib/components/shell';

export interface ResourceColumn {
  key: string;
  label: string;
  width?: string;
}

export interface ResourceFilter {
  placeholder: string;
  options: (resources: ICloudResource[]) => { value: string; label: string }[];
  match: (resource: ICloudResource, value: string) => boolean;
}

export interface ResourceDescriptor {
  /** URL slug — `/cloud/<slug>`. */
  slug: string;
  type: ResourceType;
  /** Plural display name, used for the header, search box and empty states. */
  label: string;
  description: string;
  /** Sidebar / command palette icon. */
  icon: string;
  /** Sidebar group heading. */
  group: 'Workloads' | 'Configuration' | 'Networking';
  /** Extra palette search terms beyond the slug and label. */
  keywords?: string[];
  columns: ResourceColumn[];
  /** Stats shown next to the total. */
  stats?: (resources: ICloudResource[]) => PageStat[];
  filter?: ResourceFilter;
  /**
   * Detail route prefix — detail pages were not moved, so workloads sit under
   * /cloud/workloads/<plural> while configmaps, secrets and ingress sit under /cloud/<plural>.
   */
  detailBase: string;
  /** `/cloud/<slug>/new` exists. */
  canCreate?: boolean;
  /** Resource has logs — adds the logs shortcut and the row logs button. */
  hasLogs?: boolean;
  /** Resource can be scaled from the list. */
  canScale?: boolean;
  /** Resource supports a rollout restart from the list (the `r` shortcut). */
  canRestart?: boolean;
}

const count = (resources: ICloudResource[], predicate: (r: ICloudResource) => boolean) =>
  resources.filter(predicate).length;

/** Status filter whose options come from the statuses actually present. */
const statusFilter: ResourceFilter = {
  placeholder: 'All Statuses',
  options: (resources) =>
    Array.from(new Set(resources.map((r) => r.status)))
      .sort()
      .map((status) => ({ value: status, label: status })),
  match: (resource, value) => resource.status === value,
};

const lifecycleStats = (resources: ICloudResource[]): PageStat[] => [
  { label: 'Running', value: count(resources, (r) => r.status === ResourceStatus.RUNNING) },
  { label: 'Pending', value: count(resources, (r) => r.status === ResourceStatus.PENDING) },
  { label: 'Failed', value: count(resources, (r) => r.status === ResourceStatus.FAILED) },
];

export const RESOURCE_DESCRIPTORS: ResourceDescriptor[] = [
  {
    slug: 'pods',
    type: ResourceType.POD,
    label: 'Pods',
    description: 'Container instances',
    icon: '📦',
    group: 'Workloads',
    keywords: ['pod'],
    columns: [
      { key: 'name', label: 'Name', width: 'w-1/4' },
      { key: 'status', label: 'Status', width: 'w-1/8' },
      { key: 'ready', label: 'Ready', width: 'w-1/8' },
      { key: 'restarts', label: 'Restarts', width: 'w-1/8' },
      { key: 'age', label: 'Age', width: 'w-1/8' },
      { key: 'namespace', label: 'Namespace', width: 'w-1/6' },
      { key: 'actions', label: 'Actions', width: 'w-1/6' },
    ],
    stats: (resources) => [
      ...lifecycleStats(resources),
      { label: 'Succeeded', value: count(resources, (r) => r.status === ResourceStatus.SUCCEEDED) },
    ],
    filter: statusFilter,
    detailBase: '/cloud/workloads/pods',
    hasLogs: true,
  },
  {
    slug: 'services',
    type: ResourceType.SERVICE,
    label: 'Services',
    description: 'Network services',
    icon: '🔗',
    group: 'Workloads',
    keywords: ['service', 'svc'],
    columns: [
      { key: 'name', label: 'Name', width: 'w-1/4' },
      { key: 'type', label: 'Type', width: 'w-1/8' },
      { key: 'clusterIP', label: 'Cluster IP', width: 'w-1/8' },
      { key: 'ports', label: 'Ports', width: 'w-1/4' },
      { key: 'age', label: 'Age', width: 'w-1/8' },
      { key: 'namespace', label: 'Namespace', width: 'w-1/6' },
    ],
    stats: (resources) => [
      { label: 'ClusterIP', value: count(resources, (r) => r.metadata?.type === 'ClusterIP') },
      {
        label: 'LoadBalancer',
        value: count(resources, (r) => r.metadata?.type === 'LoadBalancer'),
      },
      { label: 'NodePort', value: count(resources, (r) => r.metadata?.type === 'NodePort') },
    ],
    detailBase: '/cloud/workloads/services',
  },
  {
    slug: 'deployments',
    type: ResourceType.DEPLOYMENT,
    label: 'Deployments',
    description: 'Stateless workloads',
    icon: '🚀',
    group: 'Workloads',
    keywords: ['deployment', 'deploy'],
    columns: [
      { key: 'name', label: 'Name', width: 'w-1/4' },
      { key: 'status', label: 'Status', width: 'w-1/8' },
      { key: 'replicas', label: 'Replicas', width: 'w-1/8' },
      { key: 'ready', label: 'Ready', width: 'w-1/8' },
      { key: 'age', label: 'Age', width: 'w-1/8' },
      { key: 'namespace', label: 'Namespace', width: 'w-1/6' },
    ],
    stats: lifecycleStats,
    filter: statusFilter,
    detailBase: '/cloud/workloads/deployments',
    canScale: true,
    canRestart: true,
  },
  {
    slug: 'statefulsets',
    type: ResourceType.STATEFULSET,
    label: 'StatefulSets',
    description: 'Stateful workloads with stable network identities',
    icon: '🗄️',
    group: 'Workloads',
    keywords: ['statefulset', 'sts'],
    columns: [
      { key: 'name', label: 'Name', width: 'w-1/4' },
      { key: 'status', label: 'Status', width: 'w-1/8' },
      { key: 'replicas', label: 'Replicas', width: 'w-1/8' },
      { key: 'ready', label: 'Ready', width: 'w-1/8' },
      { key: 'age', label: 'Age', width: 'w-1/8' },
      { key: 'namespace', label: 'Namespace', width: 'w-1/6' },
    ],
    stats: lifecycleStats,
    filter: statusFilter,
    detailBase: '/cloud/workloads/statefulsets',
  },
  {
    slug: 'daemonsets',
    type: ResourceType.DAEMONSET,
    label: 'DaemonSets',
    description: 'Daemon workloads that run on every node',
    icon: '👹',
    group: 'Workloads',
    keywords: ['daemonset', 'ds'],
    columns: [
      { key: 'name', label: 'Name', width: 'w-1/4' },
      { key: 'status', label: 'Status', width: 'w-1/8' },
      { key: 'desired', label: 'Desired', width: 'w-1/8' },
      { key: 'current', label: 'Current', width: 'w-1/8' },
      { key: 'ready', label: 'Ready', width: 'w-1/8' },
      { key: 'age', label: 'Age', width: 'w-1/8' },
      { key: 'namespace', label: 'Namespace', width: 'w-1/6' },
    ],
    stats: lifecycleStats,
    filter: statusFilter,
    detailBase: '/cloud/workloads/daemonsets',
  },
  {
    slug: 'jobs',
    type: ResourceType.JOB,
    label: 'Jobs',
    description: 'One-time tasks',
    icon: '⚙️',
    group: 'Workloads',
    keywords: ['job'],
    columns: [
      { key: 'name', label: 'Name', width: 'w-1/4' },
      { key: 'status', label: 'Status', width: 'w-1/8' },
      { key: 'completions', label: 'Completions', width: 'w-1/8' },
      { key: 'succeeded', label: 'Succeeded', width: 'w-1/8' },
      { key: 'failed', label: 'Failed', width: 'w-1/8' },
      { key: 'age', label: 'Age', width: 'w-1/8' },
      { key: 'namespace', label: 'Namespace', width: 'w-1/6' },
    ],
    stats: (resources) => [
      {
        label: 'Completed',
        value: count(
          resources,
          (r) => r.status === ResourceStatus.SUCCEEDED || (r.status as string) === 'completed'
        ),
      },
      { label: 'Running', value: count(resources, (r) => r.status === ResourceStatus.RUNNING) },
      { label: 'Failed', value: count(resources, (r) => r.status === ResourceStatus.FAILED) },
    ],
    filter: statusFilter,
    detailBase: '/cloud/workloads/jobs',
  },
  {
    slug: 'cronjobs',
    type: ResourceType.CRONJOB,
    label: 'CronJobs',
    description: 'Scheduled tasks',
    icon: '⏰',
    group: 'Workloads',
    keywords: ['cronjob', 'cron'],
    columns: [
      { key: 'name', label: 'Name', width: 'w-1/4' },
      { key: 'schedule', label: 'Schedule', width: 'w-1/6' },
      { key: 'suspend', label: 'Suspend', width: 'w-1/8' },
      { key: 'active', label: 'Active', width: 'w-1/8' },
      { key: 'lastSchedule', label: 'Last Schedule', width: 'w-1/6' },
      { key: 'age', label: 'Age', width: 'w-1/8' },
      { key: 'namespace', label: 'Namespace', width: 'w-1/6' },
    ],
    stats: (resources) => [
      { label: 'Active', value: count(resources, (r) => !r.metadata?.suspend) },
      { label: 'Suspended', value: count(resources, (r) => Boolean(r.metadata?.suspend)) },
      { label: 'With active jobs', value: count(resources, (r) => (r.metadata?.active || 0) > 0) },
    ],
    filter: {
      placeholder: 'All CronJobs',
      options: () => [
        { value: 'active', label: 'Active' },
        { value: 'suspended', label: 'Suspended' },
      ],
      match: (resource, value) =>
        value === 'suspended' ? Boolean(resource.metadata?.suspend) : !resource.metadata?.suspend,
    },
    detailBase: '/cloud/workloads/cronjobs',
  },
  {
    slug: 'configmaps',
    type: ResourceType.CONFIGMAP,
    label: 'ConfigMaps',
    description: 'Configuration data',
    icon: '⚙️',
    group: 'Configuration',
    keywords: ['configmap', 'cm'],
    columns: [
      { key: 'name', label: 'Name', width: 'w-1/4' },
      { key: 'dataCount', label: 'Data Keys', width: 'w-1/8' },
      { key: 'age', label: 'Age', width: 'w-1/8' },
      { key: 'namespace', label: 'Namespace', width: 'w-1/6' },
    ],
    stats: (resources) => [
      { label: 'With data', value: count(resources, (r) => (r.metadata?.dataCount || 0) > 0) },
    ],
    detailBase: '/cloud/configmaps',
    canCreate: true,
  },
  {
    slug: 'secrets',
    type: ResourceType.SECRET,
    label: 'Secrets',
    description: 'Sensitive data',
    icon: '🔐',
    group: 'Configuration',
    keywords: ['secret'],
    columns: [
      { key: 'name', label: 'Name', width: 'w-1/4' },
      { key: 'type', label: 'Type', width: 'w-1/8' },
      { key: 'dataCount', label: 'Data Keys', width: 'w-1/8' },
      { key: 'age', label: 'Age', width: 'w-1/8' },
      { key: 'namespace', label: 'Namespace', width: 'w-1/6' },
    ],
    stats: (resources) => [
      { label: 'With data', value: count(resources, (r) => (r.metadata?.dataCount || 0) > 0) },
    ],
    detailBase: '/cloud/secrets',
    canCreate: true,
  },
  {
    slug: 'ingress',
    type: ResourceType.INGRESS,
    label: 'Ingress',
    description: 'External access rules',
    icon: '🌐',
    group: 'Networking',
    keywords: ['ingress'],
    columns: [
      { key: 'name', label: 'Name', width: 'w-1/4' },
      { key: 'class', label: 'Class', width: 'w-1/6' },
      { key: 'addresses', label: 'Addresses', width: 'w-1/4' },
      { key: 'ports', label: 'Hosts', width: 'w-1/4' },
      { key: 'age', label: 'Age', width: 'w-1/8' },
      { key: 'namespace', label: 'Namespace', width: 'w-1/6' },
    ],
    stats: (resources) => [
      {
        label: 'With addresses',
        value: count(resources, (r) => (r.metadata?.addresses || []).length > 0),
      },
    ],
    detailBase: '/cloud/ingress',
  },
];

export function getResourceDescriptor(slug: string | undefined): ResourceDescriptor | undefined {
  return RESOURCE_DESCRIPTORS.find((descriptor) => descriptor.slug === slug);
}

export function resourceDetailPath(
  descriptor: ResourceDescriptor,
  name: string,
  namespace: string
): string {
  return `${descriptor.detailBase}/${name}?namespace=${namespace}`;
}
