import React, { useState, useEffect } from 'react';
import { BlobStoreItem, Repository, GcReport, CleanupReport } from '../types';
import { fetchBlobStores, runGarbageCollection, runRetentionCleanup } from '../api';
import { Database, Trash2, RefreshCw, HardDrive, CheckCircle2, ShieldAlert, AlertCircle } from 'lucide-react';

interface Props {
  repositories: Repository[];
}

export const StorageOpsView: React.FC<Props> = ({ repositories }) => {
  const [blobStores, setBlobStores] = useState<BlobStoreItem[]>([]);
  const [gcRunning, setGcRunning] = useState(false);
  const [gcReport, setGcReport] = useState<GcReport | null>(null);
  const [error, setError] = useState<string | null>(null);

  const [selectedRepo, setSelectedRepo] = useState(repositories[0]?.name || '');
  const [maxVersions, setMaxVersions] = useState(5);
  const [cleanupRunning, setCleanupRunning] = useState(false);
  const [cleanupReport, setCleanupReport] = useState<CleanupReport | null>(null);

  useEffect(() => {
    fetchBlobStores()
      .then(setBlobStores)
      .catch(e => setError(e instanceof Error ? e.message : 'Failed to fetch blobstores'));

    if (!selectedRepo && repositories.length > 0) {
      setSelectedRepo(repositories[0].name);
    }
  }, [repositories, selectedRepo]);

  const handleGc = async () => {
    setGcRunning(true);
    setGcReport(null);
    setError(null);
    try {
      const rep = await runGarbageCollection();
      setGcReport(rep);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to run garbage collection');
    } finally {
      setGcRunning(false);
    }
  };

  const handleCleanup = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedRepo) return;
    setCleanupRunning(true);
    setCleanupReport(null);
    setError(null);
    try {
      const rep = await runRetentionCleanup(selectedRepo, maxVersions);
      setCleanupReport(rep);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to run retention cleanup');
    } finally {
      setCleanupRunning(false);
    }
  };

  return (
    <div>
      <div className="page-intro">
        <div>
          <h1 className="page-title">Storage & Operations</h1>
          <p className="page-desc">Monitor object storage pools, run garbage collection, and enforce retention policies.</p>
        </div>
      </div>

      {error && (
        <div className="error-banner">
          <AlertCircle size={16} /> {error}
        </div>
      )}

      {/* BlobStore Cards Grid */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))', gap: '1.25rem', marginBottom: '2rem' }}>
        {blobStores.length === 0 ? (
          <div style={{ color: '#64748B', padding: '2rem', textAlign: 'center', background: '#F8FAFC', borderRadius: 'var(--radius-lg)' }}>
            Loading storage information...
          </div>
        ) : (
          blobStores.map((store) => (
            <div className="repo-card" key={store.name}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginBottom: '1rem' }}>
                {store.type === 's3' ? <Database size={22} color="#29B5E8" /> : <HardDrive size={22} color="#10B981" />}
                <div>
                  <div style={{ fontWeight: 600, fontSize: '1.1rem' }}>{store.name}</div>
                  <div style={{ fontSize: '0.8rem', color: '#64748B' }}>
                    {store.type === 's3' ? 'S3-Compatible Object Storage' : 'Local File System Storage'}
                  </div>
                </div>
              </div>
              <div style={{ fontSize: '0.875rem', color: '#475569', lineHeight: '1.7' }}>
                <div>• Storage Type: <strong>{store.type.toUpperCase()}</strong></div>
                {store.type === 's3' && store.bucket && (
                  <div>• Target Bucket: <strong>{store.bucket}</strong></div>
                )}
                {store.type === 'file' && store.path && (
                  <div>• Path: <strong>{store.path}</strong></div>
                )}
                <div>• Status: <strong>Online</strong></div>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Operations 2-Column Grid */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(400px, 1fr))', gap: '1.5rem' }}>
        {/* Garbage Collection Card */}
        <div className="repo-card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
            <Trash2 size={20} color="#0369A1" />
            <h3 style={{ fontSize: '1.15rem', fontWeight: 600 }}>BlobStore Garbage Collection</h3>
          </div>
          <p style={{ fontSize: '0.875rem', color: '#64748B', marginBottom: '1.25rem' }}>
            Scans for unreferenced binary blobs in object storage and safely reclaims disk space.
          </p>

          <button
            className="btn btn-primary"
            onClick={handleGc}
            disabled={gcRunning}
            style={{ width: '100%', justifyContent: 'center' }}
          >
            {gcRunning ? <RefreshCw size={16} className="spin" /> : <Trash2 size={16} />}
            {gcRunning ? 'Running Garbage Collection...' : 'Run Garbage Collection Now'}
          </button>

          {gcReport && (
            <div style={{ marginTop: '1.25rem', padding: '1rem', background: '#F0FDF4', borderRadius: 'var(--radius-md)', border: '1px solid #DCFCE7' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.4rem', color: '#166534', fontWeight: 600, fontSize: '0.9rem', marginBottom: '0.4rem' }}>
                <CheckCircle2 size={16} /> Garbage Collection Completed
              </div>
              <div style={{ fontSize: '0.85rem', color: '#15803D' }}>
                • Scanned Blobs: {gcReport.scanned_blobs_count}<br />
                • Orphaned Blobs Purged: {gcReport.orphaned_blobs_deleted}<br />
                • Freed Storage Space: {(gcReport.freed_bytes / 1024).toFixed(2)} KB
              </div>
            </div>
          )}
        </div>

        {/* Retention Policy Card */}
        <div className="repo-card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
            <ShieldAlert size={20} color="#B45309" />
            <h3 style={{ fontSize: '1.15rem', fontWeight: 600 }}>Retention & Snapshot Cleanup</h3>
          </div>
          <p style={{ fontSize: '0.875rem', color: '#64748B', marginBottom: '1.25rem' }}>
            Enforce retention rules to prune outdated versions and temporary test builds.
          </p>

          <form onSubmit={handleCleanup}>
            <div className="form-group">
              <label className="form-label">Target Repository</label>
              <select
                className="form-select"
                value={selectedRepo}
                onChange={(e) => setSelectedRepo(e.target.value)}
              >
                {repositories.map((r) => (
                  <option key={r.name} value={r.name}>
                    {r.name} ({r.format})
                  </option>
                ))}
              </select>
            </div>

            <div className="form-group">
              <label className="form-label">Keep Max Versions per Component</label>
              <input
                type="number"
                min={1}
                max={50}
                className="form-input"
                value={maxVersions}
                onChange={(e) => setMaxVersions(parseInt(e.target.value) || 1)}
              />
            </div>

            <button
              type="submit"
              className="btn btn-secondary"
              disabled={cleanupRunning}
              style={{ width: '100%', justifyContent: 'center' }}
            >
              {cleanupRunning ? 'Cleaning up...' : 'Execute Retention Policy'}
            </button>
          </form>

          {cleanupReport && (
            <div style={{ marginTop: '1.25rem', padding: '1rem', background: '#F0FDF4', borderRadius: 'var(--radius-md)', border: '1px solid #DCFCE7' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.4rem', color: '#166534', fontWeight: 600, fontSize: '0.9rem', marginBottom: '0.4rem' }}>
                <CheckCircle2 size={16} /> Cleanup Completed
              </div>
              <div style={{ fontSize: '0.85rem', color: '#15803D' }}>
                • Deleted Old Versions: {cleanupReport.deleted_components_count}<br />
                • Deleted Asset Files: {cleanupReport.deleted_assets_count}<br />
                • Freed Storage Space: {(cleanupReport.freed_bytes / 1024).toFixed(2)} KB
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
