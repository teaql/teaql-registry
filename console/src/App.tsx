import React, { useCallback, useEffect, useState } from 'react';
import { AlertCircle, ArrowRight, Box, Database, FolderGit2, KeyRound, ScrollText, Search, ShieldCheck, Wrench } from 'lucide-react';
import { Header } from './components/Header';
import { DiagnosticsView } from './components/DiagnosticsView';
import { OfflineReadinessView } from './components/OfflineReadinessView';
import { ConnectionView } from './components/ConnectionView';
import { RepositoriesView } from './components/RepositoriesView';
import { ArtifactsView } from './components/ArtifactsView';
import { StorageOpsView } from './components/StorageOpsView';
import { ServiceLogsView } from './components/ServiceLogsView';
import { authenticate, clearCredentials, fetchRepositories, hasCredentials, setCredentials } from './api';
import { Repository } from './types';

type Page = 'diagnose' | 'offline' | 'connect' | 'repositories' | 'artifacts' | 'operations' | 'logs';
type Session = 'checking' | 'signed-out' | 'signed-in';

const navigation: { id: Page; label: string; detail: string; icon: React.ElementType }[] = [
  { id: 'diagnose', label: 'Troubleshoot', detail: 'Find a failed download', icon: Search },
  { id: 'offline', label: 'Offline readiness', detail: 'Prepare for upstream outages', icon: ShieldCheck },
  { id: 'connect', label: 'Connect a client', detail: 'URLs and access tokens', icon: KeyRound },
  { id: 'repositories', label: 'Repositories', detail: 'Configure package routes', icon: FolderGit2 },
  { id: 'artifacts', label: 'Artifacts', detail: 'Inspect published files', icon: Box },
  { id: 'operations', label: 'Storage & operations', detail: 'Cleanup and retention', icon: Database },
  { id: 'logs', label: 'Service logs', detail: 'Operational history', icon: ScrollText },
];

const LoginPage: React.FC<{ onLogin: (username: string, password: string) => Promise<void> }> = ({ onLogin }) => {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError('');
    try {
      await onLogin(username.trim(), password);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Sign in failed. Please try again.');
    } finally {
      setBusy(false);
    }
  };

  return (
    <main className="login-page">
      <form className="login-card" onSubmit={submit}>
        <div className="login-mark"><Wrench size={22} /></div>
        <h1>TeaQL Registry</h1>
        <p>Sign in to manage repositories and investigate package delivery.</p>
        {error && <div className="error-banner" role="alert"><AlertCircle size={16} />{error}</div>}
        <label className="form-label" htmlFor="login-username">Username</label>
        <input id="login-username" className="form-input" autoComplete="username" autoFocus required value={username} onChange={event => setUsername(event.target.value)} />
        <label className="form-label" htmlFor="login-password">Password</label>
        <input id="login-password" className="form-input" type="password" autoComplete="current-password" required value={password} onChange={event => setPassword(event.target.value)} />
        <button className="btn btn-primary login-submit" type="submit" disabled={busy}>{busy ? 'Signing in…' : 'Sign in'} <ArrowRight size={16} /></button>
      </form>
    </main>
  );
};

export const App: React.FC = () => {
  const [session, setSession] = useState<Session>(hasCredentials() ? 'checking' : 'signed-out');
  const [page, setPage] = useState<Page>('diagnose');
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [selectedRepo, setSelectedRepo] = useState('');
  const [selectedKeyword, setSelectedKeyword] = useState('');
  const [loadError, setLoadError] = useState('');

  const loadRepos = useCallback(async () => {
    const list = await fetchRepositories();
    setRepositories(list);
    setLoadError('');
  }, []);

  useEffect(() => {
    if (session !== 'checking') return;
    loadRepos().then(() => setSession('signed-in')).catch(() => {
      clearCredentials();
      setSession('signed-out');
    });
  }, [loadRepos, session]);

  useEffect(() => {
    const expired = () => {
      clearCredentials();
      setRepositories([]);
      setSession('signed-out');
    };
    window.addEventListener('teaql:session-expired', expired);
    return () => window.removeEventListener('teaql:session-expired', expired);
  }, []);

  const login = async (username: string, password: string) => {
    const list = await authenticate(username, password);
    setCredentials(username, password);
    setRepositories(list);
    setPage('diagnose');
    setSession('signed-in');
  };

  const signOut = () => {
    clearCredentials();
    setRepositories([]);
    setSelectedRepo('');
    setSelectedKeyword('');
    setPage('diagnose');
    setSession('signed-out');
  };

  const openArtifacts = (keyword = '', repo = '') => {
    setSelectedKeyword(keyword);
    setSelectedRepo(repo);
    setPage('artifacts');
  };

  if (session === 'checking') return <div className="session-loading">Opening TeaQL Registry…</div>;
  if (session === 'signed-out') return <LoginPage onLogin={login} />;

  return (
    <div className="app-shell">
      <Header onSignOut={signOut} />
      <div className="workspace-layout">
        <nav className="workspace-nav" aria-label="Registry navigation">
          <div className="nav-caption">WORKFLOWS</div>
          {navigation.map((item, index) => (
            <React.Fragment key={item.id}>
              {index === 5 && <div className="nav-caption nav-caption-secondary">ADMINISTRATION</div>}
              <button className={`workspace-nav-item ${page === item.id ? 'active' : ''}`} type="button" aria-current={page === item.id ? 'page' : undefined} onClick={() => setPage(item.id)}>
                <item.icon size={19} aria-hidden="true" />
                <span><strong>{item.label}</strong><small>{item.detail}</small></span>
              </button>
            </React.Fragment>
          ))}
        </nav>
        <main className="workspace-main">
          {loadError && <div className="error-banner" role="alert"><AlertCircle size={16} />{loadError}<button className="btn btn-secondary btn-sm" onClick={() => loadRepos().catch(() => setLoadError('Could not reload repositories.'))}>Retry</button></div>}
          {page === 'diagnose' && <DiagnosticsView repositories={repositories} onOpenArtifacts={openArtifacts} onOpenLogs={() => setPage('logs')} />}
          {page === 'offline' && <OfflineReadinessView repositories={repositories} onOpenArtifacts={openArtifacts} onOpenRepositories={() => setPage('repositories')} />}
          {page === 'connect' && <ConnectionView repositories={repositories} />}
          {page === 'repositories' && <RepositoriesView repositories={repositories} onRefresh={() => loadRepos().catch(() => setLoadError('Could not reload repositories.'))} onSelectRepoForSearch={repo => openArtifacts('', repo)} />}
          {page === 'artifacts' && <ArtifactsView repositories={repositories} initialRepo={selectedRepo} initialKeyword={selectedKeyword} />}
          {page === 'operations' && <StorageOpsView repositories={repositories} />}
          {page === 'logs' && <ServiceLogsView />}
        </main>
      </div>
    </div>
  );
};

export default App;
