export type SDKSidebarVariant = "collapsed" | "expanded";

export interface SDKItem {
  id: string;
  name: string;
  displayName: string;
  icon: string;
  category: string;
  installed: boolean;
  enabled: boolean;
  version?: string;
  description?: string;
  hasToggle?: boolean;
  hasService: boolean;
  serviceRunning?: boolean | null;
  port?: number | null;
}

