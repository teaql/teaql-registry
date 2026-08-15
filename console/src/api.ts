import { Repository, SearchComponentItem, SearchResponse, BlobStoreItem, PersonalAccessToken, CleanupReport, GcReport } from './types';

const API_BASE = '/service/rest/v1';

export async function fetchRepositories(): Promise<Repository[]> {
  try {
    const res = await fetch(`${API_BASE}/repositories`);
    if (!res.ok) throw new Error('Failed to fetch repositories');
    return await res.json();
  } catch (err) {
    console.error(err);
    return [];
  }
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
    headers: { 'Content-Type': 'application/json' },
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
  return res.ok;
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

  try {
    const res = await fetch(`${API_BASE}/search?${query.toString()}`);
    if (!res.ok) throw new Error('Search failed');
    return await res.json();
  } catch (err) {
    console.error(err);
    return { items: [], total: 0, page: 1, page_size: 20 };
  }
}

export async function fetchBlobStores(): Promise<BlobStoreItem[]> {
  try {
    const res = await fetch(`${API_BASE}/blobstores`);
    if (!res.ok) throw new Error('Failed to fetch blobstores');
    return await res.json();
  } catch (err) {
    console.error(err);
    return [];
  }
}

export async function runGarbageCollection(): Promise<GcReport | null> {
  try {
    const res = await fetch(`${API_BASE}/gc/run`, { method: 'POST' });
    if (!res.ok) throw new Error('GC failed');
    return await res.json();
  } catch (err) {
    console.error(err);
    return null;
  }
}

export async function runRetentionCleanup(repoName: string, maxVersions: number): Promise<CleanupReport | null> {
  try {
    const res = await fetch(`${API_BASE}/cleanup/run`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        repository: repoName,
        max_versions_per_component: maxVersions,
        snapshot_only: false,
      }),
    });
    if (!res.ok) throw new Error('Cleanup failed');
    return await res.json();
  } catch (err) {
    console.error(err);
    return null;
  }
}

export async function fetchTokens(): Promise<PersonalAccessToken[]> {
  try {
    const res = await fetch(`${API_BASE}/tokens`);
    if (!res.ok) throw new Error('Failed to fetch tokens');
    return await res.json();
  } catch (err) {
    console.error(err);
    return [];
  }
}

export async function createToken(description: string, scopes: string[], days: number): Promise<{ token: string; pat: PersonalAccessToken } | null> {
  try {
    const res = await fetch(`${API_BASE}/tokens`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        username: 'admin',
        description,
        scopes,
        expires_in_days: days > 0 ? days : null,
      }),
    });
    if (!res.ok) throw new Error('Create token failed');
    return await res.json();
  } catch (err) {
    console.error(err);
    return null;
  }
}

export async function revokeToken(tokenId: string): Promise<boolean> {
  try {
    const res = await fetch(`${API_BASE}/tokens/${tokenId}`, { method: 'DELETE' });
    return res.ok;
  } catch (err) {
    console.error(err);
    return false;
  }
}
