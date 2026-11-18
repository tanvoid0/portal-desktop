import Settings from '~icons/fa6-solid/gear';
import Database from '~icons/fa6-solid/database';
import Code from '~icons/fa6-solid/code';
import Globe from '~icons/fa6-solid/globe';
import Container from '~icons/fa6-solid/cube';
import type { ProcessedSDKConfig } from '../services/sdkConfigService';
import type { SDKItem } from '../components/sidebar/sdkSidebarTypes';
import { getSDKIcon } from './sdkIcons';

export interface SDKManager {
  id: string;
  name: string;
  display_name: string;
  installed: boolean;
  version: string | null;
  supports_installation: boolean;
  supports_version_switching: boolean;
  install_command: string | null;
  website: string | null;
}

export type SDKCategory = {
  key: string;
  title: string;
  icon: typeof Settings;
  items: SDKItem[];
};

/** Group SDK configs and managers into the sidebar's category sections. */
export function buildSDKCategories(
  sdkConfigs: ProcessedSDKConfig[],
  sdkManagers: SDKManager[]
): SDKCategory[] {
  /**
   * Convert SDK configs to SDKItem format. Languages are the one category that
   * can report a version through an installed SDK manager.
   */
  function toSDKItems(category: string, viaManagers = false): SDKItem[] {
    return sdkConfigs
      .filter((config) => config.category === category)
      .map((config) => {
        const rawVersion =
          config.sdk_version ||
          (viaManagers ? config.sdk_managers.find((m) => m.installed)?.version : null);

        return {
          id: config.id,
          name: config.name,
          displayName: config.display_name,
          icon: config.icon,
          category,
          installed:
            config.sdk_installed ||
            (viaManagers && config.sdk_managers.some((m) => m.installed)) ||
            false,
          enabled: true,
          version: (rawVersion ?? '').trim().replace(/^v/, '') || undefined,
          description: config.description,
          hasToggle: true,
          hasService: config.service_config != null,
          serviceRunning: config.service_running ?? null,
          port: config.service_port ?? null,
        } satisfies SDKItem;
      });
  }

  return [
    {
      key: 'manager',
      title: 'SDK Managers',
      icon: Settings,
      items: sdkManagers.map(
        (m) =>
          ({
            id: m.id,
            name: m.name ?? m.id,
            displayName: m.display_name ?? m.name ?? m.id,
            icon: getSDKIcon(m.id),
            category: 'manager',
            installed: m.installed,
            enabled: true,
            version: (m.version ?? '').trim().replace(/^v/, '') || undefined,
            description: undefined,
            hasToggle: false,
            hasService: false,
            serviceRunning: null,
            port: null,
          }) satisfies SDKItem
      ),
    },
    {
      key: 'language',
      title: 'Language & Runtime',
      icon: Code,
      items: toSDKItems('language', true),
    },
    { key: 'database', title: 'Database Server', icon: Database, items: toSDKItems('database') },
    { key: 'web', title: 'Web Server', icon: Globe, items: toSDKItems('server') },
    {
      key: 'container',
      title: 'Container Platform',
      icon: Container,
      items: toSDKItems('container'),
    },
    { key: 'ai', title: 'AI SDKs', icon: Code, items: toSDKItems('ai') },
  ];
}
