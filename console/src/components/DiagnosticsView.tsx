import React, { useEffect, useState } from 'react';
import { AlertCircle, ArrowRight, CheckCircle2, Search, ScrollText } from 'lucide-react';
import { fetchServiceLogs, searchComponents } from '../api';
import { Repository, SearchComponentItem, ServiceLogItem } from '../types';

interface Props {
  repositories: Repository[];
  onOpenArtifacts: (keyword?: string, repository?: string) => void;
  onOpenLogs: () => void;
}

export const DiagnosticsView: React.FC<Props> = ({ repositories, onOpenArtifacts, onOpenLogs }) => {
  const [query, setQuery] = useState('');
  const [repository, setRepository] = useState('');
  const [searched, setSearched] = useState(false);
  const [results, setResults] = useState<SearchComponentItem[]>([]);
  const [logs, setLogs] = useState<ServiceLogItem[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');

  useEffect(() => {
    fetchServiceLogs({ size: 100 }).then(setLogs).catch(() => setLogs([]));
  }, []);

  const search = async (event: React.FormEvent) => {
    event.preventDefault();
    if (!query.trim()) return;
    setBusy(true);
    setError('');
    try {
      const matches = await searchComponents({ keyword: query.trim(), repository: repository || undefined });
      setResults(matches.items);
      fetchServiceLogs({ size: 100 }).then(setLogs).catch(() => setLogs([]));
      setSearched(true);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Unable to search package records.');
    } finally {
      setBusy(false);
    }
  };

  const failures = logs.filter(log => {
    const failed = log.status?.toLowerCase() !== 'success' && log.status?.toLowerCase() !== 'ok';
    const matchesQuery = !query.trim() || `${log.artifact_path} ${log.repository_name} ${log.error_message}`.toLowerCase().includes(query.trim().toLowerCase());
    return failed && matchesQuery && (!repository || log.repository_name === repository);
  }).slice(0, 8);

  return (
    <div>
      <div className="page-intro">
        <div><div className="eyebrow">01 · FIRST RESPONSE</div><h1 className="page-title">Troubleshoot a package download</h1><p className="page-desc">Find the package record and recent request errors before changing client settings.</p></div>
      </div>
      <form className="diagnostic-search" onSubmit={search}>
        <div className="diagnostic-search-field"><Search size={20} /><input className="search-input" aria-label="Package name, group or version" placeholder="Package name, group or version" value={query} onChange={event => setQuery(event.target.value)} required /></div>
        <select className="form-select" aria-label="Repository" value={repository} onChange={event => setRepository(event.target.value)}><option value="">All repositories</option>{repositories.map(repo => <option key={repo.name} value={repo.name}>{repo.name}</option>)}</select>
        <button className="btn btn-primary" type="submit" disabled={busy}>{busy ? 'Checking…' : 'Investigate'} <ArrowRight size={16} /></button>
      </form>
      {error && <div className="error-banner" role="alert"><AlertCircle size={16} />{error}</div>}
      <div className="workflow-grid">
        <section className="workflow-panel">
          <div className="panel-heading"><Search size={19} /><h2>Package inventory</h2></div>
          {!searched ? <p className="muted">Search by package name or version to see what the Registry has recorded.</p> : results.length === 0 ? <div className="empty-result"><AlertCircle size={21} /><strong>No matching component found</strong><p>Check the repository and version. A proxy may need to fetch the package from upstream; this inventory alone does not identify the cause of a client failure.</p></div> : <>
            <div className="result-summary"><CheckCircle2 size={18} /> {results.length} matching component{results.length === 1 ? '' : 's'} shown</div>
            <ul className="compact-list">{results.slice(0, 8).map(item => <li key={item.id}><div><strong>{item.group ? `${item.group}:` : ''}{item.name}</strong><small>{item.version} · {item.repository} · {item.assets?.length || 0} file(s)</small></div></li>)}</ul>
            <button className="text-action" type="button" onClick={() => onOpenArtifacts(query.trim(), repository)}>Inspect files and checksums <ArrowRight size={15} /></button>
          </>}
        </section>
        <section className="workflow-panel">
          <div className="panel-heading"><ScrollText size={19} /><h2>Recent request failures</h2></div>
          <p className="muted">Best-effort service history. Search matches the most recent 100 entries.</p>
          {failures.length === 0 ? <div className="empty-result"><CheckCircle2 size={21} /><strong>No matching failure in recent logs</strong><p>Try the native client again, then refresh the service logs for the exact response.</p></div> : <ul className="compact-list">{failures.map(log => <li key={log.id}><div><strong>{log.status || 'Failed'} · {log.repository_name || 'Registry'}</strong><small>{log.artifact_path || log.action} · {new Date(log.event_time).toLocaleString()}</small>{log.error_message && <span className="log-error-detail">{log.error_message}</span>}</div></li>)}</ul>}
          <button className="text-action" type="button" onClick={onOpenLogs}>Open all service logs <ArrowRight size={15} /></button>
        </section>
      </div>
    </div>
  );
};
