import React, { useState, useEffect } from 'react';
import { Header } from './components/Header';
import { RepositoriesView } from './components/RepositoriesView';
import { ArtifactsView } from './components/ArtifactsView';
import { StorageOpsView } from './components/StorageOpsView';
import { AccessTokensView } from './components/AccessTokensView';
import { fetchRepositories } from './api';
import { Repository } from './types';
import { FolderGit2, Search, Database, Key } from 'lucide-react';

export const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'repositories' | 'artifacts' | 'storage' | 'tokens'>('repositories');
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [selectedRepoForSearch, setSelectedRepoForSearch] = useState<string>('');

  const loadRepos = async () => {
    const list = await fetchRepositories();
    setRepositories(list);
  };

  useEffect(() => {
    loadRepos();
  }, []);

  const handleSelectRepoForSearch = (repoName: string) => {
    setSelectedRepoForSearch(repoName);
    setActiveTab('artifacts');
  };

  return (
    <div>
      <Header />

      {/* Low-Density Nav Bar */}
      <div className="nav-bar-container">
        <div className="nav-tabs">
          <button
            className={`nav-tab ${activeTab === 'repositories' ? 'active' : ''}`}
            onClick={() => setActiveTab('repositories')}
          >
            <FolderGit2 size={18} />
            <span>Repositories ({repositories.length})</span>
          </button>

          <button
            className={`nav-tab ${activeTab === 'artifacts' ? 'active' : ''}`}
            onClick={() => setActiveTab('artifacts')}
          >
            <Search size={18} />
            <span>Artifacts & Search</span>
          </button>

          <button
            className={`nav-tab ${activeTab === 'storage' ? 'active' : ''}`}
            onClick={() => setActiveTab('storage')}
          >
            <Database size={18} />
            <span>Storage & Ops</span>
          </button>

          <button
            className={`nav-tab ${activeTab === 'tokens' ? 'active' : ''}`}
            onClick={() => setActiveTab('tokens')}
          >
            <Key size={18} />
            <span>Access Tokens</span>
          </button>
        </div>
      </div>

      {/* Main Content Area */}
      <main className="main-content">
        {activeTab === 'repositories' && (
          <RepositoriesView
            repositories={repositories}
            onRefresh={loadRepos}
            onSelectRepoForSearch={handleSelectRepoForSearch}
          />
        )}

        {activeTab === 'artifacts' && (
          <ArtifactsView
            repositories={repositories}
            initialRepo={selectedRepoForSearch}
          />
        )}

        {activeTab === 'storage' && (
          <StorageOpsView repositories={repositories} />
        )}

        {activeTab === 'tokens' && (
          <AccessTokensView />
        )}
      </main>
    </div>
  );
};
export default App;
