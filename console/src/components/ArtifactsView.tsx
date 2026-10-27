import React, { useState, useEffect } from 'react';
import { SearchComponentItem, Repository, RepositoryFacet } from '../types';
import { fetchArtifactFacets, searchComponents } from '../api';
import { Search, Download, Copy, Check, FileCode, Box, ShieldCheck, SlidersHorizontal, X } from 'lucide-react';

interface Props {
  repositories: Repository[];
  initialRepo?: string;
  initialKeyword?: string;
}

export const ArtifactsView: React.FC<Props> = ({ repositories, initialRepo, initialKeyword }) => {
  const [keyword, setKeyword] = useState(initialKeyword || '');
  const [groupInput, setGroupInput] = useState('');
  const [appliedName, setAppliedName] = useState(initialKeyword || '');
  const [appliedGroup, setAppliedGroup] = useState('');
  const [selectedRepo, setSelectedRepo] = useState(initialRepo || '');
  const [selectedFormat, setSelectedFormat] = useState('');
  const [repositoryFilter, setRepositoryFilter] = useState('');
  const [facetRepositories, setFacetRepositories] = useState<RepositoryFacet[]>([]);
  const [facetLoading, setFacetLoading] = useState(false);
  const [facetError, setFacetError] = useState('');
  const [components, setComponents] = useState<SearchComponentItem[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [loading, setLoading] = useState(false);
  const [copiedSnippet, setCopiedSnippet] = useState<string | null>(null);
  const [error, setError] = useState('');

  useEffect(() => {
    setKeyword(initialKeyword || '');
    setAppliedName(initialKeyword || '');
    setGroupInput('');
    setAppliedGroup('');
    setSelectedRepo(initialRepo || '');
    setSelectedFormat('');
    setPage(1);
  }, [initialKeyword, initialRepo]);

  useEffect(() => {
    let active = true;
    setFacetLoading(true);
    setFacetError('');
    fetchArtifactFacets({ name: appliedName || undefined, group: appliedGroup || undefined })
      .then(response => { if (active) setFacetRepositories(response.repositories); })
      .catch(cause => { if (active) setFacetError(cause instanceof Error ? cause.message : 'Could not load search facets'); })
      .finally(() => { if (active) setFacetLoading(false); });
    return () => { active = false; };
  }, [appliedName, appliedGroup]);

  useEffect(() => {
    let active = true;
    setLoading(true);
    setError('');
    searchComponents({
      name: appliedName || undefined,
      group: appliedGroup || undefined,
      repository: selectedRepo || undefined,
      format: selectedFormat || undefined,
      page,
      faceted: true,
    })
      .then(response => {
        if (!active) return;
        setComponents(response.items);
        setTotal(response.total);
      })
      .catch(cause => { if (active) setError(cause instanceof Error ? cause.message : 'Search failed'); })
      .finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, [appliedName, appliedGroup, selectedRepo, selectedFormat, page]);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setAppliedName(keyword.trim());
    setAppliedGroup(groupInput.trim());
    setPage(1);
  };

  const formats = Array.from(new Set(facetRepositories.map(repo => repo.format))).sort((left, right) => left.localeCompare(right));
  const visibleRepositories = facetRepositories.filter(repo =>
    (!selectedFormat || repo.format === selectedFormat)
    && repo.name.toLowerCase().includes(repositoryFilter.trim().toLowerCase()));
  const pageCount = Math.max(1, Math.ceil(total / 20));
  const hasFilters = Boolean(appliedName || appliedGroup || selectedFormat || selectedRepo);

  const clearFilters = () => {
    setKeyword('');
    setGroupInput('');
    setAppliedName('');
    setAppliedGroup('');
    setSelectedFormat('');
    setSelectedRepo('');
    setRepositoryFilter('');
    setPage(1);
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
    } else if (fmt === 'swift') {
      snippet = `.package(id: "${comp.group}.${comp.name}", exact: "${comp.version}")`;
    } else if (fmt === 'dart') {
      snippet = `dart pub add ${comp.name} --hosted ${window.location.origin}/repository/${comp.repository}/dart`;
    } else if (fmt === 'rubygems') {
      snippet = `gem install ${comp.name} -v ${comp.version} --source ${window.location.origin}/repository/${comp.repository}/rubygems`;
    } else if (fmt === 'composer') {
      const packageName = comp.group ? `${comp.group}/${comp.name}` : comp.name;
      snippet = `composer require ${packageName}:${comp.version}`;
    } else if (fmt === 'conan') {
      snippet = `conan install --requires=${comp.name}/${comp.version.split('#')[0]} -r=${comp.repository} --build=missing`;
    } else if (fmt === 'hex') {
      snippet = `mix hex.package fetch ${comp.name} ${comp.version} --repo=${comp.repository}`;
    } else {
      snippet = `curl -O ${window.location.origin}/repository/${comp.repository}/${comp.name}`;
    }

    const snippetKey = `snippet-${comp.id}`;

    return (
      <div className="code-snippet-box">
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', overflowX: 'auto' }}>
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
          <div className="eyebrow">05 · VERIFY A PUBLICATION</div>
          <h1 className="page-title">Artifacts & Packages</h1>
          <p className="page-desc">Find a package, narrow by its repository and format, then inspect the exact files.</p>
        </div>
      </div>

      <form onSubmit={handleSearchSubmit} className="search-container">
        <div className="artifact-query-fields">
          <label className="artifact-query-field">
            <span>Package name</span>
            <span className="artifact-query-input"><Search size={18} aria-hidden="true" /><input className="search-input" placeholder="Search packages…" value={keyword} onChange={event => setKeyword(event.target.value)} /></span>
          </label>
          <label className="artifact-query-field">
            <span>Group / namespace</span>
            <input className="search-input" placeholder="Any group" value={groupInput} onChange={event => setGroupInput(event.target.value)} />
          </label>
        </div>
        <button type="submit" className="btn btn-primary">
          Search
        </button>
      </form>

      <div className="artifact-facet-layout">
        <aside className="artifact-facet-panel" aria-label="Artifact search facets">
          <div className="artifact-facet-title"><SlidersHorizontal size={18} aria-hidden="true" /><strong>Refine results</strong>{hasFilters && <button type="button" onClick={clearFilters}>Clear all</button>}</div>
          {facetLoading && <p className="muted">Finding facets across {repositories.length} repositories…</p>}
          {facetError && <p className="facet-error" role="alert">{facetError}</p>}

          <div className="artifact-facet-group">
            <div className="artifact-facet-heading"><h2>Format</h2><span>{formats.length}</span></div>
            <div className="artifact-facet-options">
              <button type="button" className={`artifact-facet-option ${!selectedFormat ? 'active' : ''}`} aria-pressed={!selectedFormat} onClick={() => { setSelectedFormat(''); setPage(1); }}>All formats</button>
              {formats.map(format => (
                <button key={format} type="button" className={`artifact-facet-option ${selectedFormat === format ? 'active' : ''}`} aria-pressed={selectedFormat === format} onClick={() => {
                  setSelectedFormat(format);
                  if (selectedRepo && facetRepositories.find(repo => repo.name === selectedRepo)?.format !== format) setSelectedRepo('');
                  setPage(1);
                }}>{format.toUpperCase()}</button>
              ))}
            </div>
          </div>

          <div className="artifact-facet-group">
            <div className="artifact-facet-heading"><h2>Repository</h2><span>{visibleRepositories.length}</span></div>
            {facetRepositories.length >= 8 && <div className="artifact-facet-search"><Search size={15} aria-hidden="true" /><input type="search" aria-label="Filter repositories" placeholder="Find a repository…" value={repositoryFilter} onChange={event => setRepositoryFilter(event.target.value)} /></div>}
            <div className="artifact-facet-options artifact-repository-options">
              <button type="button" className={`artifact-facet-option ${!selectedRepo ? 'active' : ''}`} aria-pressed={!selectedRepo} onClick={() => { setSelectedRepo(''); setPage(1); }}>All repositories</button>
              {visibleRepositories.map(repo => (
                <button key={repo.name} type="button" className={`artifact-facet-option ${selectedRepo === repo.name ? 'active' : ''}`} aria-pressed={selectedRepo === repo.name} onClick={() => { setSelectedRepo(repo.name); setPage(1); }}>
                  <span>{repo.name}</span><small>{repo.format.toUpperCase()}</small>
                </button>
              ))}
              {!facetLoading && visibleRepositories.length === 0 && <p className="artifact-facet-empty">No matching repositories</p>}
            </div>
          </div>
        </aside>

        <section className="artifact-results" aria-label="Artifact search results">
          <div className="artifact-results-toolbar">
            <div><strong>{loading ? 'Searching…' : `${total.toLocaleString()} artifact records`}</strong><span>Newest version first within each package</span></div>
            {total > 0 && <span>Page {page} of {pageCount}</span>}
          </div>
          {hasFilters && <div className="artifact-active-filters" aria-label="Active search filters">
            {appliedName && <button type="button" onClick={() => { setKeyword(''); setAppliedName(''); setPage(1); }}>Name: {appliedName} <X size={13} /></button>}
            {appliedGroup && <button type="button" onClick={() => { setGroupInput(''); setAppliedGroup(''); setPage(1); }}>Group: {appliedGroup} <X size={13} /></button>}
            {selectedFormat && <button type="button" onClick={() => { setSelectedFormat(''); setPage(1); }}>Format: {selectedFormat} <X size={13} /></button>}
            {selectedRepo && <button type="button" onClick={() => { setSelectedRepo(''); setPage(1); }}>Repository: {selectedRepo} <X size={13} /></button>}
          </div>}
          {error && <div className="error-banner" role="alert">{error}</div>}

      {loading ? (
        <div style={{ textAlign: 'center', padding: '3rem', color: '#64748B' }}>Searching artifacts...</div>
      ) : components.length === 0 ? (
        <div className="repo-card" style={{ textAlign: 'center', padding: '3.5rem 1rem' }}>
          <Box size={40} color="#94A3B8" style={{ margin: '0 auto 1rem auto' }} />
          <h3 style={{ fontSize: '1.15rem', fontWeight: 600 }}>No artifacts found</h3>
          <p style={{ color: '#64748B', fontSize: '0.9rem', maxWidth: '420px', margin: '0.5rem auto 0 auto' }}>
            Try another package name, group, format, or repository.
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
          {total > 20 && <div className="artifact-pagination">
            <button type="button" className="btn btn-secondary btn-sm" disabled={page <= 1 || loading} onClick={() => setPage(page - 1)}>Previous</button>
            <span>Page {page} of {pageCount}</span>
            <button type="button" className="btn btn-secondary btn-sm" disabled={page >= pageCount || loading} onClick={() => setPage(page + 1)}>Next</button>
          </div>}
        </section>
      </div>
    </div>
  );
};
