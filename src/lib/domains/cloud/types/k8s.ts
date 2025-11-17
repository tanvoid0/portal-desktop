// Kubernetes log types for cloud domain
export interface K8sLog {
  timestamp: string;
  level: string;
  message: string;
  pod: string;
  container: string;
}

