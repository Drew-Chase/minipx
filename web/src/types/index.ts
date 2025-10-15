export interface Server {
  id: string;
  name: string;
  domain: string;
  host: string;
  port: number;
  path: string;
  ssl_enabled: boolean;
  redirect_to_https: boolean;
  listen_port: number | null;
  status: 'running' | 'stopped' | 'error' | 'restarting';
  binary_path: string;
  startup_command: string | null;
  runtime_id: string | null;
  main_executable: string | null;
  created_at: string;
  updated_at: string;
}

export interface Certificate {
  id: string;
  name: string;
  domain: string;
  cert_path: string;
  key_path: string | null;
  is_letsencrypt: boolean;
  expiry_date: string | null;
  created_at: string;
  updated_at: string;
}

export interface ResourceMetric {
  id: string;
  server_id: string;
  cpu_usage: number;
  memory_usage: number;
  disk_usage: number;
  network_in: number;
  network_out: number;
  timestamp: string;
}

export interface SystemStats {
  cpu_usage: number;
  memory_usage: number;
  memory_total: number;
  memory_used: number;
  disk_usage: number;
  disk_total: number;
  disk_used: number;
  network_in: number;
  network_out: number;
}

export type ThemeMode = 'light' | 'dark';
export type ColorblindMode = 'none' | 'protanopia' | 'deuteranopia' | 'tritanopia';

export interface ThemeConfig {
  mode: ThemeMode;
  colorblindMode: ColorblindMode;
}

export interface Runtime {
  id: string;
  name: string;
  display_name: string;
  version: string;
  executable_path: string;
  runtime_type: 'java' | 'dotnet' | 'nodejs' | 'python' | 'go' | 'binary';
  detected_at: string;
  is_available: boolean;
}

export interface ArchiveFile {
  name: string;
  path: string;
  size: number;
  isExecutable: boolean;
}

export interface StartupTemplate {
  name: string;
  runtime_type: string;
  template: string;
  variables: string[];
}
