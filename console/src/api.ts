import { Repository, SearchComponentItem, SearchResponse, BlobStoreItem, PersonalAccessToken, CleanupReport, GcReport, ServiceLogItem } from './types';

const API_BASE = '/service/rest/v1';

function getAuthHeaders(): HeadersInit {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  const creds = sessionStorage.getItem('teaql-auth');
  if (creds) {
    headers['Authorization'] = `Basic ${creds}`;
  }
  return headers;
}

export function setCredentials(username: string, password: string) {
  const encoded = btoa(`${username}:${password}`);
  sessionStorage.setItem('teaql-auth', encoded);
}

export function clearCredentials() {
  sessionStorage.removeItem('teaql-auth');
}

export function hasCredentials(): boolean {
  return sessionStorage.getItem('teaql-auth') !== null;
}

export async function fetchRepositories(): Promise<Repository[]> {
  const res = await fetch(`${API_BASE}/repositories`, { headers: getAuthHeaders() });
  if (!res.ok) throw new Error('Failed to fetch repositories');
  return await res.json();
}

export async function createRepository(payload: {
  name: string;
  format: string;
  type: string;
  blobStoreName: string;
  writePolicy?: string;
  remoteUrl?: string;
}): Promise<boolean> {
  const url = `${API_BASE}/repositories/${payload.format}/${payload.type}`;
  const res = await fetch(url, {
    method: 'POST',
    headers: getAuthHeaders(),
    body: JSON.stringify({
      name: payload.name,
      online: true,
      storage: {
        blobStoreName: payload.blobStoreName,
        strictContentTypeValidation: false,
        writePolicy: payload.writePolicy || 'ALLOW_WRITE',
      },
      proxy: payload.type === 'proxy' ? { remoteUrl: payload.remoteUrl || '' } : undefined,
    }),
  });
  if (!res.ok) throw new Error('Failed to create repository');
  return true;
}

export async function searchComponents(params: {
  keyword?: string;
  name?: string;
  repository?: string;
  format?: string;
  page?: number;
}): Promise<SearchResponse<SearchComponentItem>> {
  const query = new URLSearchParams();
  if (params.keyword) query.set('keyword', params.keyword);
  if (params.name) query.set('name', params.name);
  if (params.repository) query.set('repository', params.repository);
  if (params.format) query.set('format', params.format);
  if (params.page) query.set('page', params.page.toString());
  query.set('page_size', '20');

  const res = await fetch(`${API_BASE}/search?${query.toString()}`, { headers: getAuthHeaders() });
  if (!res.ok) throw new Error('Search failed');
  return await res.json();
}

export async function fetchBlobStores(): Promise<BlobStoreItem[]> {
  const res = await fetch(`${API_BASE}/blobstores`, { headers: getAuthHeaders() });
  if (!res.ok) throw new Error('Failed to fetch blobstores');
  return await res.json();
}

export async function runGarbageCollection(): Promise<GcReport> {
  const res = await fetch(`${API_BASE}/gc/run`, {
    method: 'POST',
    headers: getAuthHeaders(),
  });
  if (!res.ok) throw new Error('GC failed');
  return await res.json();
}

export async function runRetentionCleanup(repoName: string, maxVersions: number): Promise<CleanupReport> {
  const res = await fetch(`${API_BASE}/cleanup/run`, {
    method: 'POST',
    headers: getAuthHeaders(),
    body: JSON.stringify({
      repository: repoName,
      max_versions_per_component: maxVersions,
      snapshot_only: false,
    }),
  });
  if (!res.ok) throw new Error('Cleanup failed');
  return await res.json();
}

export async function fetchTokens(): Promise<PersonalAccessToken[]> {
  const res = await fetch(`${API_BASE}/tokens`, { headers: getAuthHeaders() });
  if (!res.ok) throw new Error('Failed to fetch tokens');
  return await res.json();
}

export async function createToken(description: string, scopes: string[], days: number): Promise<{ token: string; pat: PersonalAccessToken }> {
  const res = await fetch(`${API_BASE}/tokens`, {
    method: 'POST',
    headers: getAuthHeaders(),
    body: JSON.stringify({
      description,
      scopes,
      expires_in_days: days > 0 ? days : null,
    }),
  });
  if (!res.ok) throw new Error('Create token failed');
  return await res.json();
}

export async function revokeToken(tokenId: string): Promise<boolean> {
  const res = await fetch(`${API_BASE}/tokens/${tokenId}`, {
    method: 'DELETE',
    headers: getAuthHeaders(),
  });
  if (!res.ok) throw new Error('Failed to revoke token');
  return true;
}

export async function fetchServiceLogs(params?: {
  tenant_id?: number;
  log_type?: string;
  username?: string;
  action?: string;
  size?: number;
}): Promise<ServiceLogItem[]> {
  const query = new URLSearchParams();
  if (params?.tenant_id) query.set('tenant_id', params.tenant_id.toString());
  if (params?.log_type) query.set('log_type', params.log_type);
  if (params?.username) query.set('username', params.username);
  if (params?.action) query.set('action', params.action);
  if (params?.size) query.set('size', params.size.toString());

  const res = await fetch(`${API_BASE}/service-logs?${query.toString()}`, { headers: getAuthHeaders() });
  if (!res.ok) throw new Error('Failed to fetch service logs');
  return await res.json();
}
