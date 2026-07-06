// GCP Provider implementation
// Maps GCP/GKE (Kubernetes) backend to abstract cloud provider interface

import { invokeClient } from "$lib/utils/invokeClient";
import { BaseProvider } from "../base/BaseProvider";
import type { ICluster, ICloudResource, ResourceEvent } from "../../core/types";
import {
  ResourceType,
  CloudProviderType,
  ClusterStatus,
  ResourceStatus,
} from "../../core/types";
import type {
  KubernetesCluster,
  PodInfo,
  ServiceInfo,
  DeploymentInfo,
  NamespaceInfo,
} from "./GCPTypes";

export class GCPProvider extends BaseProvider {
  readonly name = "Google Cloud Platform";
  readonly type = CloudProviderType.GCP;

  async initialize(): Promise<void> {
    try {
      // #region agent log
      fetch(
        "http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "X-Debug-Session-Id": "7cbddc",
          },
          body: JSON.stringify({
            sessionId: "7cbddc",
            runId: "pre-fix",
            hypothesisId: "H2",
            location: "GCPProvider.ts:29",
            message: "GCPProvider.initialize start",
            data: {},
            timestamp: Date.now(),
          }),
        },
      ).catch(() => {});
      // #endregion agent log
      await invokeClient.post("k8s_initialize_manager");
      // #region agent log
      fetch(
        "http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "X-Debug-Session-Id": "7cbddc",
          },
          body: JSON.stringify({
            sessionId: "7cbddc",
            runId: "pre-fix",
            hypothesisId: "H2",
            location: "GCPProvider.ts:31",
            message: "GCPProvider.initialize after invoke",
            data: {},
            timestamp: Date.now(),
          }),
        },
      ).catch(() => {});
      // #endregion agent log
    } catch (error) {
      console.error("Failed to initialize GCP provider:", error);
      // #region agent log
      fetch(
        "http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "X-Debug-Session-Id": "7cbddc",
          },
          body: JSON.stringify({
            sessionId: "7cbddc",
            runId: "pre-fix",
            hypothesisId: "H2",
            location: "GCPProvider.ts:33",
            message: "GCPProvider.initialize error",
            data: { error: String(error) },
            timestamp: Date.now(),
          }),
        },
      ).catch(() => {});
      // #endregion agent log
      throw error;
    }
  }

  async connect(clusterId: string): Promise<void> {
    try {
      await invokeClient.post("k8s_connect_cluster", {
        clusterName: clusterId,
      });
      this.connected = true;

      // Update current cluster
      const clusters = await this.listClusters();
      this.currentCluster = clusters.find((c) => c.id === clusterId) || null;
    } catch (error) {
      this.connected = false;
      console.error("Failed to connect to cluster:", error);
      throw error;
    }
  }

  async disconnect(): Promise<void> {
    this.connected = false;
    this.currentCluster = null;
  }

  async listClusters(): Promise<ICluster[]> {
    try {
      // #region agent log
      fetch(
        "http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "X-Debug-Session-Id": "7cbddc",
          },
          body: JSON.stringify({
            sessionId: "7cbddc",
            runId: "pre-fix",
            hypothesisId: "H3",
            location: "GCPProvider.ts:58",
            message: "listClusters start",
            data: {},
            timestamp: Date.now(),
          }),
        },
      ).catch(() => {});
      // #endregion agent log
      const clusters =
        await invokeClient.post<KubernetesCluster[]>("k8s_load_clusters");
      // #region agent log
      fetch(
        "http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "X-Debug-Session-Id": "7cbddc",
          },
          body: JSON.stringify({
            sessionId: "7cbddc",
            runId: "pre-fix",
            hypothesisId: "H3",
            location: "GCPProvider.ts:60",
            message: "listClusters after invoke",
            data: {
              clusterCount: Array.isArray(clusters) ? clusters.length : null,
            },
            timestamp: Date.now(),
          }),
        },
      ).catch(() => {});
      // #endregion agent log
      // Handle null/undefined response (can happen with localhost strategy or errors)
      if (!clusters || !Array.isArray(clusters)) {
        // In browser/localhost mode, this might be expected
        // In Tauri mode, empty array means no kubeconfig or no clusters
        return [];
      }
      return clusters.map((c) => this.mapToCluster(c));
    } catch (error) {
      // Log the error but still return empty array to allow UI to show helpful message
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      console.error("Failed to list clusters:", error);
      // #region agent log
      fetch(
        "http://127.0.0.1:7704/ingest/4c51fb7c-6c3e-4188-9012-a753ceea53c2",
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "X-Debug-Session-Id": "7cbddc",
          },
          body: JSON.stringify({
            sessionId: "7cbddc",
            runId: "pre-fix",
            hypothesisId: "H3",
            location: "GCPProvider.ts:69",
            message: "listClusters error",
            data: { error: errorMessage },
            timestamp: Date.now(),
          }),
        },
      ).catch(() => {});
      // #endregion agent log

      // Re-throw if it's a meaningful error (not just "empty" or localhost strategy)
      // This allows the UI to show the actual error
      if (
        !errorMessage.includes("localhost") &&
        !errorMessage.includes("empty")
      ) {
        throw error;
      }

      return [];
    }
  }

  async getCluster(id: string): Promise<ICluster | null> {
    const clusters = await this.listClusters();
    return clusters.find((c) => c.id === id || c.name === id) || null;
  }

  async listResources(
    type: ResourceType,
    namespace?: string,
  ): Promise<ICloudResource[]> {
    try {
      switch (type) {
        case ResourceType.POD:
          const pods = await invokeClient.post<PodInfo[]>("k8s_list_pods", {
            namespace,
          });
          if (!pods || !Array.isArray(pods)) return [];
          return pods.map((p) => this.mapToPod(p));

        case ResourceType.SERVICE:
          const services = await invokeClient.post<ServiceInfo[]>(
            "k8s_list_services",
            { namespace },
          );
          if (!services || !Array.isArray(services)) return [];
          return services.map((s) => this.mapToService(s));

        case ResourceType.DEPLOYMENT:
          const deployments = await invokeClient.post<DeploymentInfo[]>(
            "k8s_list_deployments",
            { namespace },
          );
          if (!deployments || !Array.isArray(deployments)) return [];
          return deployments.map((d) => this.mapToDeployment(d));

        case ResourceType.STATEFULSET:
          const statefulsets = await invokeClient.post<any[]>(
            "k8s_list_statefulsets",
            { namespace },
          );
          if (!statefulsets || !Array.isArray(statefulsets)) return [];
          return statefulsets.map((ss) => this.mapToStatefulSet(ss));

        case ResourceType.DAEMONSET:
          const daemonsets = await invokeClient.post<any[]>(
            "k8s_list_daemonsets",
            { namespace },
          );
          if (!daemonsets || !Array.isArray(daemonsets)) return [];
          return daemonsets.map((ds) => this.mapToDaemonSet(ds));

        case ResourceType.JOB:
          const jobs = await invokeClient.post<any[]>("k8s_list_jobs", {
            namespace,
          });
          if (!jobs || !Array.isArray(jobs)) return [];
          return jobs.map((j) => this.mapToJob(j));

        case ResourceType.CRONJOB:
          const cronjobs = await invokeClient.post<any[]>("k8s_list_cronjobs", {
            namespace,
          });
          if (!cronjobs || !Array.isArray(cronjobs)) return [];
          return cronjobs.map((cj) => this.mapToCronJob(cj));

        case ResourceType.CONFIGMAP:
          const configmaps = await invokeClient.post<any[]>(
            "k8s_list_configmaps",
            { namespace },
          );
          if (!configmaps || !Array.isArray(configmaps)) return [];
          return configmaps.map((cm) => this.mapToConfigMap(cm));

        case ResourceType.SECRET:
          const secrets = await invokeClient.post<any[]>("k8s_list_secrets", {
            namespace,
          });
          if (!secrets || !Array.isArray(secrets)) return [];
          return secrets.map((s) => this.mapToSecret(s));

        case ResourceType.INGRESS:
          const ingresses = await invokeClient.post<any[]>(
            "k8s_list_ingresses",
            { namespace },
          );
          if (!ingresses || !Array.isArray(ingresses)) return [];
          return ingresses.map((i) => this.mapToIngress(i));

        case ResourceType.NAMESPACE:
          const namespaces = await invokeClient.post<NamespaceInfo[]>(
            "k8s_list_namespaces",
          );
          if (!namespaces || !Array.isArray(namespaces)) return [];
          return namespaces.map((n) => this.mapToNamespace(n));

        default:
          return [];
      }
    } catch (error) {
      console.error(`Failed to list ${type} resources:`, error);
      return [];
    }
  }

  async getResource(
    type: ResourceType,
    id: string,
    namespace: string,
  ): Promise<ICloudResource | null> {
    const resources = await this.listResources(type, namespace);
    return resources.find((r) => r.id === id || r.name === id) || null;
  }

  async *watchResources(
    type: ResourceType,
    namespace?: string,
  ): AsyncIterable<ResourceEvent> {
    // Note: Real-time updates are handled via Tauri events in the cloudStore
    // This async iterator pattern is kept for interface compliance but watch is managed
    // at the store level via startWatchingResources() which uses Tauri events
    // For now, return empty async iterator - actual watching happens via store events
    yield* [];
  }

  async listNamespaces(): Promise<string[]> {
    try {
      const namespaces = await invokeClient.post<NamespaceInfo[]>(
        "k8s_list_namespaces",
      );
      if (!namespaces || !Array.isArray(namespaces)) return [];
      return namespaces.map((n) => n.name);
    } catch (error) {
      console.error("Failed to list namespaces:", error);
      return [];
    }
  }

  // Mapping methods from GCP types to abstract types

  private mapToCluster(data: KubernetesCluster): ICluster {
    let status: ClusterStatus;

    if (typeof data.status === "string") {
      status = this.mapClusterStatus(data.status);
    } else if (
      data.status &&
      typeof data.status === "object" &&
      "Error" in data.status
    ) {
      status = ClusterStatus.ERROR;
    } else {
      status = ClusterStatus.DISCONNECTED;
    }

    return {
      id: data.name,
      name: data.name,
      provider: CloudProviderType.GCP,
      status,
      context: data.context,
      namespace: data.namespace,
      server: data.server,
      version: data.version,
      metadata: {
        last_connected: data.last_connected,
        ...data,
      },
    };
  }

  mapToPod(data: PodInfo): ICloudResource {
    const resource: ICloudResource = {
      id: data.name,
      name: data.name,
      namespace: data.namespace,
      type: ResourceType.POD,
      status: this.mapResourceStatus(data.status),
      provider: CloudProviderType.GCP,
      metadata: {
        ready: data.ready,
        restarts: data.restarts,
        age: data.age,
        ip: data.ip,
        node: data.node,
        containers: data.containers,
        status: data.status,
      },
    };

    // Add optional methods
    resource.getLogs = async (
      container?: string,
      tailLines?: number,
      follow?: boolean,
    ) => {
      try {
        // For non-streaming logs (follow: false), use regular post
        if (!follow) {
          return await invokeClient.post<string>("k8s_get_pod_logs", {
            namespace: data.namespace,
            podName: data.name,
            container: container || null,
            follow: false,
            tailLines: tailLines || null,
          });
        }

        // For streaming logs (follow: true), use live method
        // Note: Backend needs to emit events with pattern: k8s_get_pod_logs:stream:{streamId}
        // TODO: Implement streaming logs using invokeClient.live when backend supports it
        // For now, fall back to non-streaming
        console.warn(
          "Streaming logs (follow: true) not yet implemented, falling back to non-streaming",
        );
        return await invokeClient.post<string>("k8s_get_pod_logs", {
          namespace: data.namespace,
          podName: data.name,
          container: container || null,
          follow: false,
          tailLines: tailLines || null,
        });
      } catch (error) {
        // Don't log PodInitializing errors - they're expected and handled in the UI
        const errorMessage =
          error instanceof Error ? error.message : String(error);
        const isPodInitializing =
          errorMessage.includes("PodInitializing") ||
          errorMessage.includes("waiting to start") ||
          errorMessage.includes("is waiting to start");

        if (!isPodInitializing) {
          console.error("Failed to get pod logs:", error);
        }
        throw error;
      }
    };

    // Add method for streaming logs (using type assertion since it's not in the interface)
    // TODO: Add getLogsStream to ICloudResource interface when streaming is fully implemented
    (resource as any).getLogsStream = async (
      onData: (data: string) => void,
      onError?: (error: Error) => void,
      container?: string,
      tailLines?: number,
    ): Promise<() => void> => {
      // Use invokeClient.live for streaming logs
      // Backend should emit events: k8s_get_pod_logs:stream:{streamId}
      return await invokeClient.live<string>("k8s_get_pod_logs", {
        data: {
          namespace: data.namespace,
          podName: data.name,
          container: container || null,
          follow: true,
          tailLines: tailLines || null,
        },
        onData,
        onError,
      });
    };

    resource.exec = async (command: string[]) => {
      try {
        return await invokeClient.post<string>("k8s_exec_pod", {
          namespace: data.namespace,
          podName: data.name,
          container: null,
          command: command,
        });
      } catch (error) {
        console.error("Failed to exec into pod:", error);
        throw error;
      }
    };

    resource.delete = async () => {
      try {
        await invokeClient.post("k8s_delete_pod", {
          namespace: data.namespace,
          podName: data.name,
        });
      } catch (error) {
        console.error("Failed to delete pod:", error);
        throw error;
      }
    };

    return resource;
  }

  mapToService(data: ServiceInfo): ICloudResource {
    return {
      id: data.name,
      name: data.name,
      namespace: data.namespace,
      type: ResourceType.SERVICE,
      status: ResourceStatus.RUNNING, // Services are typically running
      provider: CloudProviderType.GCP,
      metadata: {
        cluster_ip: data.cluster_ip,
        external_ip: data.external_ip,
        ports: data.ports,
        selector: data.selector,
        age: data.age,
        name: data.name,
        namespace: data.namespace,
      },
    };
  }

  mapToDeployment(data: DeploymentInfo): ICloudResource {
    const resource: ICloudResource = {
      id: data.name,
      name: data.name,
      namespace: data.namespace,
      type: ResourceType.DEPLOYMENT,
      status:
        data.available === data.desired
          ? ResourceStatus.RUNNING
          : ResourceStatus.PENDING,
      provider: CloudProviderType.GCP,
      metadata: {
        desired: data.desired,
        current: data.current,
        up_to_date: data.up_to_date,
        available: data.available,
        age: data.age,
        labels: data.labels,
        name: data.name,
        namespace: data.namespace,
      },
    };

    // Add optional methods
    resource.scale = async (replicas: number) => {
      try {
        await invokeClient.post("k8s_scale_deployment", {
          namespace: data.namespace,
          deploymentName: data.name,
          replicas: replicas,
        });
      } catch (error) {
        console.error("Failed to scale deployment:", error);
        throw error;
      }
    };

    return resource;
  }

  mapToStatefulSet(data: any): ICloudResource {
    const resource: ICloudResource = {
      id: data.name,
      name: data.name,
      namespace: data.namespace,
      type: ResourceType.STATEFULSET,
      status:
        data.ready === data.desired
          ? ResourceStatus.RUNNING
          : ResourceStatus.PENDING,
      provider: CloudProviderType.GCP,
      metadata: {
        desired: data.desired,
        current: data.current,
        ready: data.ready,
        age: data.age,
        labels: data.labels || {},
        name: data.name,
        namespace: data.namespace,
      },
    };

    return resource;
  }

  mapToDaemonSet(data: any): ICloudResource {
    const resource: ICloudResource = {
      id: data.name,
      name: data.name,
      namespace: data.namespace,
      type: ResourceType.DAEMONSET,
      status:
        data.ready === data.desired
          ? ResourceStatus.RUNNING
          : ResourceStatus.PENDING,
      provider: CloudProviderType.GCP,
      metadata: {
        desired: data.desired,
        current: data.current,
        ready: data.ready,
        up_to_date: data.up_to_date,
        available: data.available,
        age: data.age,
        labels: data.labels || {},
        name: data.name,
        namespace: data.namespace,
      },
    };

    return resource;
  }

  mapToJob(data: any): ICloudResource {
    return {
      id: data.name,
      name: data.name,
      namespace: data.namespace,
      type: ResourceType.JOB,
      status: this.mapResourceStatus(data.status || "unknown"),
      provider: CloudProviderType.GCP,
      metadata: {
        completions: data.completions || 0,
        succeeded: data.succeeded || 0,
        failed: data.failed || 0,
        active: data.active || 0,
        parallelism: data.parallelism || 1,
        backoff_limit: data.backoff_limit || 6,
        age: data.age || "N/A",
        image: data.image,
        status: data.status,
      },
    };
  }

  mapToConfigMap(data: any): ICloudResource {
    return {
      id: data.name,
      name: data.name,
      namespace: data.namespace,
      type: ResourceType.CONFIGMAP,
      status: ResourceStatus.RUNNING, // ConfigMaps don't have a status
      provider: CloudProviderType.GCP,
      metadata: {
        data: data.data || {},
        dataKeys: Object.keys(data.data || {}),
        dataCount: Object.keys(data.data || {}).length,
        age: data.age || "N/A",
        labels: data.labels || {},
      },
    };
  }

  mapToSecret(data: any): ICloudResource {
    return {
      id: data.name,
      name: data.name,
      namespace: data.namespace,
      type: ResourceType.SECRET,
      status: ResourceStatus.RUNNING, // Secrets don't have a status
      provider: CloudProviderType.GCP,
      metadata: {
        data: data.data || {}, // Base64 encoded
        dataKeys: Object.keys(data.data || {}),
        dataCount: Object.keys(data.data || {}).length,
        age: data.age || "N/A",
        labels: data.labels || {},
        type: data.secret_type || "Opaque",
      },
    };
  }

  mapToCronJob(data: any): ICloudResource {
    return {
      id: data.name,
      name: data.name,
      namespace: data.namespace,
      type: ResourceType.CRONJOB,
      status: data.suspend ? ResourceStatus.PENDING : ResourceStatus.RUNNING,
      provider: CloudProviderType.GCP,
      metadata: {
        schedule: data.schedule || "N/A",
        suspend: data.suspend || false,
        active: data.active || 0,
        last_schedule_time: data.last_schedule_time || null,
        last_successful_time: data.last_successful_time || null,
        age: data.age || "N/A",
        image: data.image,
      },
    };
  }

  mapToIngress(data: any): ICloudResource {
    return {
      id: data.name,
      name: data.name,
      namespace: data.namespace,
      type: ResourceType.INGRESS,
      status: ResourceStatus.RUNNING, // Ingress doesn't have a status
      provider: CloudProviderType.GCP,
      metadata: {
        class: data.class || "N/A",
        addresses: data.addresses || [],
        ports: data.ports || [],
        age: data.age || "N/A",
        labels: data.labels || {},
      },
    };
  }

  private mapToNamespace(data: NamespaceInfo): ICloudResource {
    return {
      id: data.name,
      name: data.name,
      namespace: data.name,
      type: ResourceType.NAMESPACE,
      status:
        data.status === "Active"
          ? ResourceStatus.RUNNING
          : ResourceStatus.UNKNOWN,
      provider: CloudProviderType.GCP,
      metadata: {
        status: data.status,
        age: data.age,
        labels: data.labels,
        name: data.name,
      },
    };
  }
}
