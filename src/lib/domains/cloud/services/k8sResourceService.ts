/**
 * Kubernetes resource service.
 *
 * Single home for the k8s resource commands that the cloud routes used to call
 * via raw `invoke()`. Routes/components import these typed methods instead of
 * `@tauri-apps/api/core`, matching the backend signatures in
 * `src-tauri/src/domains/kubernetes/commands.rs`.
 */

import { invokeClient } from "$lib/utils/invokeClient";

export type PodMetrics = Record<string, unknown>;

export const k8sResourceService = {
  /** Get a resource's manifest as YAML. */
  getResourceYaml(
    kind: string,
    namespace: string,
    name: string,
  ): Promise<string> {
    return invokeClient.request<string>("k8s_get_resource_yaml", {
      data: { kind, namespace, name },
    });
  },

  /** Apply (create/update) a resource from YAML; returns a status message. */
  applyResourceYaml(namespace: string, yamlContent: string): Promise<string> {
    return invokeClient.request<string>("k8s_apply_resource_yaml", {
      data: { namespace, yamlContent },
    });
  },

  deleteConfigmap(namespace: string, name: string): Promise<void> {
    return invokeClient.request("k8s_delete_configmap", {
      data: { namespace, name },
    });
  },

  deleteSecret(namespace: string, name: string): Promise<void> {
    return invokeClient.request("k8s_delete_secret", {
      data: { namespace, name },
    });
  },

  scaleDeployment(
    namespace: string,
    deploymentName: string,
    replicas: number,
  ): Promise<void> {
    return invokeClient.request("k8s_scale_deployment", {
      data: { namespace, deploymentName, replicas },
    });
  },

  /** Restart/rollback a deployment; returns a status message. */
  rollbackDeployment(namespace: string, name: string): Promise<string> {
    return invokeClient.request<string>("k8s_rollback_deployment", {
      data: { namespace, name },
    });
  },

  /** Metrics for all pods (optionally scoped to a namespace). */
  getAllPodsMetrics(namespace?: string | null): Promise<PodMetrics> {
    return invokeClient.request<PodMetrics>("k8s_get_all_pods_metrics", {
      data: { namespace: namespace ?? null },
    });
  },
};
