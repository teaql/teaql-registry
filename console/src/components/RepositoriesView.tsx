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
  const [showModal, setShowModal] = useState<boolean>(false);

  // New Repo Form State
  const [name, setName] = useState('');
  const [format, setFormat] = useState('maven2');
  const [type, setType] = useState('hosted');
  const [blobStoreName, setBlobStoreName] = useState('default');
  const [remoteUrl, setRemoteUrl] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const formats = ['ALL', 'maven2', 'docker', 'npm', 'pypi', 'gomod', 'cargo', 'nuget', 'raw'];

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
    const ok = await createRepository({
      name: name.trim(),
      format,
      type,
      blobStoreName,
      remoteUrl: type === 'proxy' ? remoteUrl.trim() : undefined,
    });
    setIsSubmitting(false);
    if (ok) {
      setShowModal(false);
      setName('');
      setRemoteUrl('');
      onRefresh();
    } else {
      alert('Failed to create repository');
    }
  };

  return (
    <div>
      <div className="page-intro">
        <div>
          <h1 className="page-title">Repositories</h1>
          <p className="page-desc">Manage package repositories across 8 ecosystems with multi-tier storage.</p>
        </div>
        <button className="btn btn-primary" onClick={() => setShowModal(true)}>
          <Plus size={16} /> Create Repository
        </button>
      </div>

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

      {/* Create Modal */}
      {showModal && (
        <div className="modal-backdrop" onClick={() => setShowModal(false)}>
          <div className="modal-card" onClick={(e) => e.stopPropagation()}>
            <h2 style={{ fontSize: '1.25rem', fontWeight: 600, marginBottom: '1rem' }}>Create Repository</h2>
            <form onSubmit={handleCreate}>
              <div className="form-group">
                <label className="form-label">Repository Name</label>
                <input
                  className="form-input"
                  placeholder="e.g. my-releases"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  required
                />
              </div>

              <div className="form-group">
                <label className="form-label">Format</label>
                <select className="form-select" value={format} onChange={(e) => setFormat(e.target.value)}>
                  <option value="maven2">Maven2</option>
                  <option value="docker">Docker</option>
                  <option value="npm">NPM</option>
                  <option value="pypi">PyPI</option>
                  <option value="cargo">Cargo (Rust)</option>
                  <option value="gomod">Go Modules</option>
                  <option value="nuget">NuGet (.NET)</option>
                  <option value="raw">Raw (Generic)</option>
                </select>
              </div>

              <div className="form-group">
                <label className="form-label">Type</label>
                <select className="form-select" value={type} onChange={(e) => setType(e.target.value)}>
                  <option value="hosted">Hosted (Private Packages)</option>
                  <option value="proxy">Proxy (Remote Caching)</option>
                  <option value="group">Group (Virtual Aggregation)</option>
                </select>
              </div>

              {type === 'proxy' && (
                <div className="form-group">
                  <label className="form-label">Remote Upstream URL</label>
                  <input
                    className="form-input"
                    placeholder="https://repo.maven.apache.org/maven2"
                    value={remoteUrl}
                    onChange={(e) => setRemoteUrl(e.target.value)}
                    required
                  />
                </div>
              )}

              <div className="form-group">
                <label className="form-label">BlobStore</label>
                <input
                  className="form-input"
                  value={blobStoreName}
                  onChange={(e) => setBlobStoreName(e.target.value)}
                  placeholder="default"
                />
              </div>

              <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '0.75rem', marginTop: '1.5rem' }}>
                <button type="button" className="btn btn-secondary" onClick={() => setShowModal(false)}>
                  Cancel
                </button>
                <button type="submit" className="btn btn-primary" disabled={isSubmitting}>
                  {isSubmitting ? 'Creating...' : 'Create'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
