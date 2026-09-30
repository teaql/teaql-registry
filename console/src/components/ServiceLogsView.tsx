import React, { useState, useEffect } from 'react';
import { ServiceLogItem } from '../types';
import { fetchServiceLogs } from '../api';
import { ScrollText, RefreshCw, AlertCircle } from 'lucide-react';

export const ServiceLogsView: React.FC = () => {
  const [logs, setLogs] = useState<ServiceLogItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filterLogType, setFilterLogType] = useState('');
  const [filterAction, setFilterAction] = useState('');
  const [filterUsername, setFilterUsername] = useState('');

  const loadLogs = async () => {
    setLoading(true);
    setError(null);
    try {
      const items = await fetchServiceLogs({
        log_type: filterLogType || undefined,
        action: filterAction || undefined,
        username: filterUsername || undefined,
        size: 100,
      });
      setLogs(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load logs');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadLogs();
  }, [filterLogType, filterAction]);

  const formatTime = (iso: string) => {
    try {
      return new Date(iso).toLocaleString();
    } catch {
      return iso;
    }
  };

  const formatSize = (bytes: number) => {
    if (bytes === 0) return '—';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1048576).toFixed(1)} MB`;
  };

  return (
    <div>
      <div className="page-intro">
        <div>
          <h1 className="page-title">Service Logs</h1>
          <p className="page-desc">
            Best-effort per-tenant operational history for artifact upload and download operations.
          </p>
        </div>
        <button className="btn btn-secondary" onClick={loadLogs} disabled={loading}>
          <RefreshCw size={16} className={loading ? 'spin' : ''} /> Refresh
        </button>
      </div>

      {error && (
        <div className="error-banner">
          <AlertCircle size={16} /> {error}
        </div>
      )}

      {/* Filters */}
      <div className="logs-filter-bar">
        <select className="form-select" value={filterLogType} onChange={(e) => setFilterLogType(e.target.value)}>
          <option value="">All Types</option>
          <option value="service">Service</option>
          <option value="system">System</option>
        </select>
        <select className="form-select" value={filterAction} onChange={(e) => setFilterAction(e.target.value)}>
          <option value="">All Actions</option>
          <option value="upload">Upload</option>
          <option value="download">Download</option>
        </select>
        <input
          className="form-input"
          placeholder="Filter by username..."
          value={filterUsername}
          onChange={(e) => setFilterUsername(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && loadLogs()}
          style={{ minWidth: '180px' }}
        />
        <button className="btn btn-primary btn-sm" onClick={loadLogs}>Apply</button>
      </div>

      {/* Table */}
      {loading ? (
        <div style={{ textAlign: 'center', padding: '3rem', color: '#64748B' }}>Loading logs...</div>
      ) : logs.length === 0 ? (
        <div className="repo-card empty-state">
          <ScrollText size={40} color="#94A3B8" className="empty-state-icon" />
          <h3 style={{ fontSize: '1.15rem', fontWeight: 600 }}>No log entries found</h3>
          <p style={{ color: '#64748B', fontSize: '0.9rem', marginTop: '0.5rem' }}>
            Logs appear when artifacts are uploaded or downloaded.
          </p>
        </div>
      ) : (
        <div className="logs-table-container">
          <table className="logs-table">
            <thead>
              <tr>
                <th>Time</th>
                <th>Type</th>
                <th>User</th>
                <th>Action</th>
                <th>Repository</th>
                <th>Path</th>
                <th>Format</th>
                <th>Size</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody>
              {logs.map((log) => (
                <tr key={log.id}>
                  <td style={{ whiteSpace: 'nowrap' }}>{formatTime(log.event_time)}</td>
                  <td>
                    <span className={`log-badge log-badge-${log.log_type}`}>
                      {log.log_type}
                    </span>
                  </td>
                  <td>{log.username}</td>
                  <td>{log.action}</td>
                  <td>{log.repository_name || '—'}</td>
                  <td style={{ maxWidth: '200px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}
                    title={log.artifact_path}>
                    {log.artifact_path || '—'}
                  </td>
                  <td>{log.format_name || '—'}</td>
                  <td>{formatSize(log.content_size)}</td>
                  <td>
                    <span className={`log-badge log-badge-${log.status}`}>
                      {log.status}
                    </span>
                    {log.error_message && (
                      <span title={log.error_message} style={{ marginLeft: '0.3rem', cursor: 'help' }}>
                        <AlertCircle size={13} color="#DC2626" />
                      </span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};
