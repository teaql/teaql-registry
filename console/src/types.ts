export interface Repository {
  name: string;
  format: string;
  type: string;
  url: string;
  online: boolean;
}

export interface ChecksumMap {
  sha1?: string;
  sha256?: string;
  md5?: string;
  [key: string]: string | undefined;
}

export interface SearchAssetItem {
  id: number;
  path: string;
  download_url: string;
  format: string;
  repository: string;
  content_type: string;
  size: number;
  checksum: ChecksumMap;
}

export interface SearchComponentItem {
  id: number;
  name: string;
  group: string;
  version: string;
  format: string;
  repository: string;
  assets: SearchAssetItem[];
}

export interface SearchResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

export interface BlobStoreItem {
  name: string;
  type: string;
  path: string;
  bucket?: string;
  region?: string;
}

export interface PersonalAccessToken {
  id: string;
  username: string;
  token_hash: string;
  description: string;
  scopes: string[];
  created_at: string;
  expires_at?: string;
}

export interface CleanupReport {
  deleted_components_count: number;
  deleted_assets_count: number;
  freed_bytes: number;
}

export interface GcReport {
  scanned_blobs_count: number;
  orphaned_blobs_deleted: number;
  freed_bytes: number;
}

export interface ServiceLogItem {
  id: number;
  tenant_id: number;
  event_time: string;
  log_type: string;
  username: string;
  client_ip: string;
  action: string;
  repository_name: string;
  artifact_path: string;
  format_name: string;
  content_size: number;
  status: string;
  error_message: string;
}

export interface ApiError {
  message: string;
}
