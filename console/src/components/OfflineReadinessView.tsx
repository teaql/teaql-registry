import React, { useState } from 'react';
import { AlertCircle, ArrowRight, CheckCircle2, ClipboardCheck, Search, ShieldCheck } from 'lucide-react';
import { searchComponents } from '../api';
import { Repository, SearchComponentItem } from '../types';
import { SearchableSelect } from './SearchableSelect';

interface Props {
  repositories: Repository[];
  onOpenArtifacts: (keyword?: string, repository?: string) => void;
  onOpenRepositories: () => void;
}

type PackageFormat = 'maven2' | 'npm' | 'cargo';
type CheckResult = { coordinate: string; found: boolean; repository?: string };

function parseCoordinate(line: string, format: PackageFormat) {
  if (format === 'maven2') {
    const parts = line.split(':').map(part => part.trim());
    if (parts.length !== 3 || parts.some(part => !part)) return null;
    return { group: parts[0], name: parts[1], version: parts[2] };
  }
  const separator = line.lastIndexOf('@');
  if (separator <= 0 || separator === line.length - 1) return null;
  return { name: line.slice(0, separator).trim(), version: line.slice(separator + 1).trim() };
}

export const OfflineReadinessView: React.FC<Props> = ({ repositories, onOpenArtifacts, onOpenRepositories }) => {
  const [format, setFormat] = useState<PackageFormat>('maven2');
  const [repository, setRepository] = useState('');
  const [input, setInput] = useState('');
  const [results, setResults] = useState<CheckResult[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const matchingRepos = repositories.filter(repo => repo.format === format && repo.type.toLowerCase() !== 'group');

  const check = async (event: React.FormEvent) => {
    event.preventDefault();
    const lines = [...new Set(input.split(/\r?\n/).map(line => line.trim()).filter(Boolean))];
    if (lines.length === 0 || lines.length > 20) { setError('Enter 1–20 coordinates, one per line.'); return; }
    if (lines.some(line => !parseCoordinate(line, format))) { setError(format === 'maven2' ? 'Use group:artifact:version for every line.' : 'Use package@version for every line.'); return; }
    setBusy(true);
    setError('');
    setResults(null);
    try {
      const checked: CheckResult[] = [];
      for (const coordinate of lines) {
        const parsed = parseCoordinate(coordinate, format)!;
        let page = 1;
        let match: SearchComponentItem | undefined;
        do {
          const response = await searchComponents({ ...parsed, format, repository: repository || undefined, page });
          match = response.items.find(item => item.name === parsed.name && item.version === parsed.version && (!('group' in parsed) || item.group === parsed.group));
          if (match || page * response.page_size >= response.total || response.items.length === 0) break;
          page += 1;
        } while (true);
        checked.push({ coordinate, found: Boolean(match), repository: match?.repository });
      }
      setResults(checked);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Inventory check failed.');
    } finally {
      setBusy(false);
    }
  };

  return (
    <div>
      <div className="page-intro"><div><div className="eyebrow">02 · BEFORE DISCONNECTING UPSTREAM</div><h1 className="page-title">Offline readiness</h1><p className="page-desc">Compare exact package versions with the Registry inventory, then verify with a native client.</p></div></div>
      <div className="notice-panel"><ShieldCheck size={20} /><div><strong>What this check means</strong><p>“Recorded” means a component exists in the catalog. It does not prove all metadata, transitive dependencies, or blobs can be served while upstream is down. A build from a clean client cache during a controlled upstream outage is the final check.</p></div></div>
      <div className="workflow-grid">
        <section className="workflow-panel">
          <div className="panel-heading"><ClipboardCheck size={19} /><h2>Check required versions</h2></div>
          <form onSubmit={check}>
            <div className="form-group"><label className="form-label" htmlFor="offline-format">Package format</label><select id="offline-format" className="form-select" value={format} onChange={event => { setFormat(event.target.value as PackageFormat); setRepository(''); setResults(null); }}><option value="maven2">Maven</option><option value="npm">npm</option><option value="cargo">Cargo</option></select></div>
            <div className="form-group"><label className="form-label" htmlFor="offline-repo">Repository</label><SearchableSelect id="offline-repo" value={repository} onChange={setRepository} options={[{ value: '', label: `All ${format} repositories` }, ...matchingRepos.map(repo => ({ value: repo.name, label: `${repo.name} (${repo.type})` }))]} /></div>
            <div className="form-group"><label className="form-label" htmlFor="offline-input">Exact versions · one per line · up to 20</label><textarea id="offline-input" className="form-input coordinate-input" value={input} onChange={event => setInput(event.target.value)} placeholder={format === 'maven2' ? 'org.example:library:1.2.3' : format === 'npm' ? '@scope/package@1.2.3' : 'serde@1.0.0'} /></div>
            {error && <div className="error-banner" role="alert"><AlertCircle size={16} />{error}</div>}
            <button className="btn btn-primary" type="submit" disabled={busy}>{busy ? 'Checking inventory…' : 'Check inventory'} <Search size={16} /></button>
          </form>
        </section>
        <section className="workflow-panel">
          <div className="panel-heading"><CheckCircle2 size={19} /><h2>Result and next step</h2></div>
          {results === null ? <p className="muted">Enter the versions from your dependency manifest or lockfile. Include transitive dependencies when available.</p> : <>
            <div className="result-summary">{results.filter(item => item.found).length} of {results.length} recorded in the catalog</div>
            <ul className="compact-list">{results.map(item => <li key={item.coordinate}><div><strong>{item.coordinate}</strong><small>{item.found ? `Recorded in ${item.repository}` : 'No matching component found'}</small></div><span className={`readiness-status ${item.found ? 'found' : 'missing'}`}>{item.found ? 'Recorded' : 'Missing'}</span></li>)}</ul>
          </>}
          <div className="readiness-next"><strong>Finish with a native client</strong><ol><li>Start from a clean client package cache.</li><li>Make the upstream unavailable in a controlled test.</li><li>Run the project build through this Registry and record missing requests.</li></ol></div>
          <button className="text-action" type="button" onClick={() => onOpenArtifacts('', repository)}>Inspect artifacts <ArrowRight size={15} /></button>
          <button className="text-action" type="button" onClick={onOpenRepositories}>Review proxy repositories <ArrowRight size={15} /></button>
        </section>
      </div>
    </div>
  );
};
