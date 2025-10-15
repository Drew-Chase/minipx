import { Server, Certificate, ResourceMetric, SystemStats, Runtime } from '../types';

const API_BASE = '/api';

async function fetchAPI<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE}${endpoint}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: response.statusText }));
    throw new Error(error.error || 'API request failed');
  }

  return response.json();
}

// Server API
export const serverAPI = {
  list: () => fetchAPI<Server[]>('/servers'),
  get: (id: string) => fetchAPI<Server>(`/servers/${id}`),
  create: (data: Partial<Server>) => fetchAPI<Server>('/servers', {
    method: 'POST',
    body: JSON.stringify(data),
  }),
  update: (id: string, data: Partial<Server>) => fetchAPI<Server>(`/servers/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  }),
  delete: (id: string) => fetchAPI<void>(`/servers/${id}`, {
    method: 'DELETE',
  }),
  start: (id: string) => fetchAPI<{ message: string }>(`/servers/${id}/start`, {
    method: 'POST',
  }),
  stop: (id: string) => fetchAPI<{ message: string }>(`/servers/${id}/stop`, {
    method: 'POST',
  }),
  restart: (id: string) => fetchAPI<{ message: string }>(`/servers/${id}/restart`, {
    method: 'POST',
  }),
  uploadBinary: async (serverId: string, file: File) => {
    const formData = new FormData();
    formData.append('serverId', serverId);
    formData.append('file', file);

    const response = await fetch(`${API_BASE}/servers/upload`, {
      method: 'POST',
      body: formData,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || 'Upload failed');
    }

    return response.json();
  },
};

// Certificate API
export const certificateAPI = {
  list: () => fetchAPI<Certificate[]>('/certificates'),
  get: (id: string) => fetchAPI<Certificate>(`/certificates/${id}`),
  create: (data: Partial<Certificate>) => fetchAPI<Certificate>('/certificates', {
    method: 'POST',
    body: JSON.stringify(data),
  }),
  delete: (id: string) => fetchAPI<void>(`/certificates/${id}`, {
    method: 'DELETE',
  }),
  uploadCertificate: async (certificateId: string, certFile: File, keyFile?: File) => {
    const formData = new FormData();
    formData.append('certificateId', certificateId);
    formData.append('cert', certFile);
    if (keyFile) {
      formData.append('key', keyFile);
    }

    const response = await fetch(`${API_BASE}/certificates/upload`, {
      method: 'POST',
      body: formData,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || 'Certificate upload failed');
    }

    return response.json();
  },
};

// Metrics API
export const metricsAPI = {
  getSystemStats: () => fetchAPI<SystemStats>('/metrics/system'),
  getServerMetrics: (serverId: string) => fetchAPI<ResourceMetric>(`/metrics/server/${serverId}`),
  getServerMetricsHistory: (serverId: string) => fetchAPI<ResourceMetric[]>(`/metrics/server/${serverId}/history`),
};

// Runtime API
export const runtimeAPI = {
  list: () => fetchAPI<Runtime[]>('/runtimes'),
  detect: () => fetchAPI<Runtime[]>('/runtimes/detect', {
    method: 'POST',
  }),
  scanArchive: (files: string[]) => fetchAPI<{ executables: string[] }>('/runtimes/scan-archive', {
    method: 'POST',
    body: JSON.stringify({ files }),
  }),
};
