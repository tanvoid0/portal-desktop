import { invokeClient } from "$lib/utils/invokeClient";

export const podApi = {
  getLogs(params: {
    namespace: string;
    podName: string;
    container?: string | null;
    follow?: boolean;
    tailLines?: number | null;
  }) {
    return invokeClient.post<string>("k8s_get_pod_logs", {
      namespace: params.namespace,
      podName: params.podName,
      container: params.container ?? null,
      follow: params.follow ?? false,
      tailLines: params.tailLines ?? null,
    });
  },

  getYaml(namespace: string, podName: string) {
    return invokeClient.post<string>("k8s_get_pod_yaml", { namespace, podName });
  },

  getMetrics(namespace: string, podName: string) {
    return invokeClient.post<unknown>("k8s_get_pod_metrics", { namespace, podName });
  },

  deletePod(namespace: string, podName: string) {
    return invokeClient.post("k8s_delete_pod", { namespace, podName });
  },

  startPortForward(params: {
    namespace: string;
    podName: string;
    localPort: number;
    remotePort: number;
  }) {
    return invokeClient.post("k8s_start_port_forward", params);
  },

  stopPortForward(id: string) {
    return invokeClient.post("k8s_stop_port_forward", { id });
  },

  listPortForwards() {
    return invokeClient.post<unknown[]>("k8s_list_port_forwards");
  },
};
