import React, { useState, useEffect } from 'react';
import { SearchComponentItem, Repository } from '../types';
import { searchComponents } from '../api';
import { Search, Download, Copy, Check, Terminal, FileCode, Box, ShieldCheck } from 'lucide-react';

interface Props {
  repositories: Repository[];
  initialRepo?: string;
}

export const ArtifactsView: React.FC<Props> = ({ repositories, initialRepo }) => {
  const [keyword, setKeyword] = useState('');
  const [selectedRepo, setSelectedRepo] = useState(initialRepo || '');
  const [components, setComponents] = useState<SearchComponentItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [copiedSnippet, setCopiedSnippet] = useState<string | null>(null);

  const performSearch = async () => {
    setLoading(true);
    const res = await searchComponents({
      keyword: keyword.trim() || undefined,
      repository: selectedRepo || undefined,
    });
    setComponents(res.items);
    setLoading(false);
  };

  useEffect(() => {
    performSearch();
  }, [selectedRepo]);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    performSearch();
  };

  const handleCopy = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedSnippet(id);
    setTimeout(() => setCopiedSnippet(null), 2000);
  };

  const renderInstallSnippet = (comp: SearchComponentItem) => {
    const fmt = comp.format.toLowerCase();
    let snippet = '';

    if (fmt === 'npm') {
      snippet = `npm install ${comp.name}@${comp.version} --registry=${window.location.origin}/repository/${comp.repository}/npm/`;
    } else if (fmt === 'pypi') {
      snippet = `pip install ${comp.name}==${comp.version} --index-url ${window.location.origin}/repository/${comp.repository}/simple/`;
    } else if (fmt === 'cargo') {
      snippet = `cargo add ${comp.name} --registry teaql`;
    } else if (fmt === 'docker') {
      snippet = `docker pull ${window.location.host}/${comp.repository}/${comp.name}:${comp.version}`;
    } else if (fmt === 'maven2') {
      snippet = `<dependency>\n  <groupId>${comp.group}</groupId>\n  <artifactId>${comp.name}</artifactId>\n  <version>${comp.version}</version>\n</dependency>`;
    } else if (fmt === 'gomod') {
      snippet = `go get ${comp.name}@v${comp.version}`;
    } else if (fmt === 'nuget') {
      snippet = `dotnet add package ${comp.name} --version ${comp.version} --source ${window.location.origin}/repository/${comp.repository}/v3/index.json`;
    } else {
      snippet = `curl -O ${window.location.origin}/repository/${comp.repository}/${comp.name}`;
    }

    const snippetKey = `snippet-${comp.id}`;

    return (
      <div className="code-snippet-box">
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', overflowX: 'auto' }}>
          <Terminal size={14} color="#29B5E8" />
          <pre style={{ margin: 0, fontFamily: 'inherit' }}>{snippet}</pre>
        </div>
        <button
          className="copy-btn"
          title="Copy command"
          onClick={() => handleCopy(snippet, snippetKey)}
        >
          {copiedSnippet === snippetKey ? <Check size={14} color="#10B981" /> : <Copy size={14} />}
        </button>
      </div>
    );
  };

  return (
    <div>
      <div className="page-intro">
        <div>
          <h1 className="page-title">Artifacts & Packages</h1>
          <p className="page-desc">Search, inspect and install artifacts with auto-generated CLI snippets.</p>
        </div>
      </div>

      {/* Wide Search Bar */}
      <form onSubmit={handleSearchSubmit} className="search-container">
        <Search size={20} color="#94A3B8" />
        <input
          className="search-input"
          placeholder="Search components by package name, group, version, keyword..."
          value={keyword}
          onChange={(e) => setKeyword(e.target.value)}
        />
        <select
          className="form-select"
          style={{ width: '220px', border: 'none', background: '#F1F5F9', fontWeight: 500 }}
          value={selectedRepo}
          onChange={(e) => setSelectedRepo(e.target.value)}
        >
          <option value="">All Repositories</option>
          {repositories.map((r) => (
            <option key={r.name} value={r.name}>
              {r.name} ({r.format})
            </option>
          ))}
        </select>
        <button type="submit" className="btn btn-primary btn-sm" style={{ padding: '0.5rem 1rem' }}>
          Search
        </button>
      </form>

      {/* Results List */}
      {loading ? (
        <div style={{ textAlign: 'center', padding: '3rem', color: '#64748B' }}>Searching artifacts...</div>
      ) : components.length === 0 ? (
        <div className="repo-card" style={{ textAlign: 'center', padding: '3.5rem 1rem' }}>
          <Box size={40} color="#94A3B8" style={{ margin: '0 auto 1rem auto' }} />
          <h3 style={{ fontSize: '1.15rem', fontWeight: 600 }}>No artifacts found</h3>
          <p style={{ color: '#64748B', fontSize: '0.9rem', maxWidth: '420px', margin: '0.5rem auto 0 auto' }}>
            Try publishing artifacts using your package manager (mvn deploy, npm publish, docker push) or check your search keywords.
          </p>
        </div>
      ) : (
        components.map((comp) => (
          <div key={comp.id} className="artifact-card">
            <div className="artifact-title-row">
              <div>
                <span className="artifact-name">{comp.name}</span>
                <span style={{ marginLeft: '0.75rem', fontWeight: 600, color: '#0369A1' }}>
                  v{comp.version}
                </span>
                {comp.group && (
                  <span style={{ marginLeft: '0.75rem', fontSize: '0.85rem', color: '#64748B' }}>
                    Group: {comp.group}
                  </span>
                )}
              </div>
              <span className="badge badge-hosted">{comp.repository}</span>
            </div>

            {/* Quick Install Snippet */}
            {renderInstallSnippet(comp)}

            {/* Assets List inside this Component */}
            {comp.assets && comp.assets.length > 0 && (
              <div style={{ marginTop: '1rem', borderTop: '1px solid #F1F5F9', paddingTop: '0.75rem' }}>
                <div style={{ fontSize: '0.8rem', fontWeight: 600, color: '#64748B', marginBottom: '0.5rem' }}>
                  FILES & CHECKSUMS ({comp.assets.length})
                </div>
                {comp.assets.map((asset) => (
                  <div
                    key={asset.id}
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                      padding: '0.4rem 0',
                      fontSize: '0.85rem',
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                      <FileCode size={14} color="#64748B" />
                      <span style={{ fontFamily: 'var(--font-mono)' }}>{asset.path}</span>
                      <span style={{ color: '#94A3B8', fontSize: '0.8rem' }}>
                        ({(asset.size / 1024).toFixed(1)} KB)
                      </span>
                    </div>

                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                      {asset.checksum?.sha256 && (
                        <span
                          style={{
                            fontSize: '0.75rem',
                            color: '#059669',
                            display: 'flex',
                            alignItems: 'center',
                            gap: '0.2rem',
                          }}
                          title={`SHA-256: ${asset.checksum.sha256}`}
                        >
                          <ShieldCheck size={13} /> SHA-256
                        </span>
                      )}
                      <a
                        href={asset.download_url}
                        target="_blank"
                        rel="noreferrer"
                        className="btn btn-secondary btn-sm"
                        style={{ padding: '0.2rem 0.5rem' }}
                      >
                        <Download size={12} /> Download
                      </a>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        ))
      )}
    </div>
  );
};
