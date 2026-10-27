import React, { useState } from 'react';
import { Check, Copy, Link2, Terminal } from 'lucide-react';
import { Repository } from '../types';
import { AccessTokensView } from './AccessTokensView';
import { SearchableSelect } from './SearchableSelect';

export const ConnectionView: React.FC<{ repositories: Repository[] }> = ({ repositories }) => {
  const [selected, setSelected] = useState('');
  const [copied, setCopied] = useState(false);
  const repository = repositories.find(item => item.name === selected) || repositories[0];
  const endpoint = repository ? new URL(repository.url, window.location.origin).href : '';
  const format = repository?.format.toLowerCase();
  let snippet = endpoint;
  if (format === 'npm') snippet = `npm config set registry ${endpoint.replace(/\/$/, '')}/npm/`;
  if (format === 'pypi') snippet = `pip install --index-url ${endpoint.replace(/\/$/, '')}/simple/ PACKAGE`;
  if (format === 'maven2') snippet = `mvn dependency:get -DremoteRepositories=teaql::default::${endpoint} -Dartifact=GROUP:ARTIFACT:VERSION`;
  if (format === 'cargo') snippet = `[registries.teaql]\nindex = "sparse+${endpoint.replace(/\/$/, '')}/cargo/index/"`;
  if (format === 'docker') snippet = `docker login ${window.location.host}\ndocker pull ${window.location.host}/${repository.name}/IMAGE:TAG`;

  const copy = async () => {
    await navigator.clipboard.writeText(snippet);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div>
      <div className="page-intro"><div><div className="eyebrow">03 · FIRST CLIENT SETUP</div><h1 className="page-title">Connect a native client</h1><p className="page-desc">Choose a repository, copy its endpoint, and issue credentials for your build.</p></div></div>
      <section className="workflow-panel connection-panel">
        <div className="panel-heading"><Link2 size={19} /><h2>Repository endpoint</h2></div>
        {repositories.length === 0 ? <p className="muted">No repositories have been configured yet.</p> : <>
          <div className="form-group"><label className="form-label" htmlFor="connection-repo">Repository</label><SearchableSelect id="connection-repo" value={repository?.name || ''} onChange={setSelected} options={repositories.map(repo => ({ value: repo.name, label: `${repo.name} · ${repo.format} · ${repo.type}` }))} /></div>
          <div className="endpoint-box"><strong>Endpoint</strong><code>{endpoint}</code></div>
          <div className="panel-heading snippet-heading"><Terminal size={19} /><h2>Client example</h2></div>
          <div className="code-snippet-box"><pre>{snippet}</pre><button type="button" className="copy-btn" aria-label="Copy client example" onClick={copy}>{copied ? <Check size={16} /> : <Copy size={16} />}</button></div>
          <p className="muted snippet-note">Use a repository URL reachable by the client. Configure authentication with a personal access token below when publishing or reading private packages.</p>
        </>}
      </section>
      <AccessTokensView />
    </div>
  );
};
