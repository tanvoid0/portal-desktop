export interface ConfigFile {
  name: string;
  path: string;
  content: string;
  format: 'json' | 'yaml' | 'toml' | 'ini' | 'xml' | 'text';
  lastModified: string;
}

export interface EnvironmentVariable {
  name: string;
  value: string;
  scope: 'global' | 'session' | 'project';
}
