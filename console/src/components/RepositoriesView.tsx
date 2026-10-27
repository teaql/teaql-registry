import React, { useState } from 'react';
import { Repository } from '../types';
import { createRepository } from '../api';
import { Plus, Copy, Check, ExternalLink, Package, Server } from 'lucide-react';

interface Props {
  repositories: Repository[];
  onRefresh: () => void;
  onSelectRepoForSearch: (repoName: string) => void;
}

export const RepositoriesView: React.FC<Props> = ({ repositories, onRefresh, onSelectRepoForSearch }) => {
  const [activeFilter, setActiveFilter] = useState<string>('ALL');
  const [copiedUrl, setCopiedUrl] = useState<string | null>(null);
  const [showCreateForm, setShowCreateForm] = useState<boolean>(false);
  const [createError, setCreateError] = useState('');

  // New Repo Form State
  const [name, setName] = useState('');
  const [format, setFormat] = useState('maven2');
  const [type, setType] = useState('hosted');
  const [blobStoreName, setBlobStoreName] = useState('default');
  const [remoteUrl, setRemoteUrl] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const formats = ['ALL', 'maven2', 'docker', 'npm', 'pypi', 'gomod', 'cargo', 'nuget', 'swift', 'dart', 'rubygems', 'composer', 'conan', 'hex', 'raw'];

  const filteredRepos = repositories.filter((repo) => {
    if (activeFilter === 'ALL') return true;
    return repo.format.toLowerCase() === activeFilter.toLowerCase();
  });

  const handleCopy = (urlPath: string) => {
    const fullUrl = `${window.location.origin}${urlPath}`;
    navigator.clipboard.writeText(fullUrl);
    setCopiedUrl(urlPath);
    setTimeout(() => setCopiedUrl(null), 2000);
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    setIsSubmitting(true);
    setCreateError('');
    try {
      await createRepository({
        name: name.trim(), format, type, blobStoreName,
        remoteUrl: type === 'proxy' ? remoteUrl.trim() : undefined,
      });
      setShowCreateForm(false);
      setName('');
      setRemoteUrl('');
      onRefresh();
    } catch (cause) {
      setCreateError(cause instanceof Error ? cause.message : 'Failed to create repository');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div>
      <div className="page-intro">
        <div>
          <h1 className="page-title">Repositories</h1>
          <p className="page-desc">Manage package repositories across 14 ecosystems with multi-tier storage.</p>
        </div>
        <button className="btn btn-primary" onClick={() => setShowCreateForm(!showCreateForm)}>
          <Plus size={16} /> {showCreateForm ? 'Close form' : 'Create Repository'}
        </button>
      </div>

      {showCreateForm && (
        <section className="repo-card create-repository-panel" aria-label="Create repository">
          <h2 style={{ fontSize: '1.25rem', fontWeight: 600, marginBottom: '1rem' }}>Create Repository</h2>
          {createError && <div className="error-banner" role="alert">{createError}</div>}
          <form onSubmit={handleCreate} className="create-repository-grid">
            <div className="form-group"><label className="form-label" htmlFor="repo-name">Repository Name</label><input id="repo-name" className="form-input" placeholder="e.g. my-releases" value={name} onChange={e => setName(e.target.value)} required /></div>
            <div className="form-group"><label className="form-label" htmlFor="repo-format">Format</label><select id="repo-format" className="form-select" value={format} onChange={e => setFormat(e.target.value)}>{formats.filter(item => item !== 'ALL').map(item => <option key={item} value={item}>{item.toUpperCase()}</option>)}</select></div>
            <div className="form-group"><label className="form-label" htmlFor="repo-type">Type</label><select id="repo-type" className="form-select" value={type} onChange={e => setType(e.target.value)}><option value="hosted">Hosted</option><option value="proxy">Proxy</option><option value="group">Group</option></select></div>
            <div className="form-group"><label className="form-label" htmlFor="repo-blobstore">BlobStore</label><input id="repo-blobstore" className="form-input" value={blobStoreName} onChange={e => setBlobStoreName(e.target.value)} required /></div>
            {type === 'proxy' && <div className="form-group create-repository-wide"><label className="form-label" htmlFor="repo-upstream">Remote Upstream URL</label><input id="repo-upstream" className="form-input" type="url" placeholder="https://repo.maven.apache.org/maven2" value={remoteUrl} onChange={e => setRemoteUrl(e.target.value)} required /></div>}
            <div className="create-repository-actions"><button type="button" className="btn btn-secondary" onClick={() => setShowCreateForm(false)}>Cancel</button><button type="submit" className="btn btn-primary" disabled={isSubmitting}>{isSubmitting ? 'Creating…' : 'Create repository'}</button></div>
          </form>
        </section>
      )}

      {/* Format Filter Pills */}
      <div className="filter-pills">
        {formats.map((fmt) => (
          <button
            key={fmt}
            className={`pill-btn ${activeFilter === fmt ? 'active' : ''}`}
            onClick={() => setActiveFilter(fmt)}
          >
            {fmt.toUpperCase()}
          </button>
        ))}
      </div>

      {/* Card Grid */}
      <div className="card-grid">
        {filteredRepos.map((repo) => (
          <div key={repo.name} className="repo-card">
            <div className="card-header">
              <div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                  <Package size={18} color="#29B5E8" />
                  <span className="repo-name">{repo.name}</span>
                </div>
                <div style={{ fontSize: '0.8rem', color: '#64748B', marginTop: '0.2rem' }}>
                  Format: <strong>{repo.format}</strong>
                </div>
              </div>
              <span className={`badge badge-${repo.type.toLowerCase()}`}>{repo.type}</span>
            </div>

            <div className="repo-url-box">
              <span>{repo.url}</span>
              <button
                className="copy-btn"
                title="Copy full repository URL"
                onClick={() => handleCopy(repo.url)}
              >
                {copiedUrl === repo.url ? <Check size={14} color="#10B981" /> : <Copy size={14} />}
              </button>
            </div>

            <div style={{ marginTop: '1.25rem', display: 'flex', justifyContent: 'flex-end', gap: '0.5rem' }}>
              <button
                className="btn btn-secondary btn-sm"
                onClick={() => onSelectRepoForSearch(repo.name)}
              >
                <ExternalLink size={13} /> Browse Artifacts
              </button>
            </div>
          </div>
        ))}
      </div>

    </div>
  );
};
