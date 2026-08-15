import React, { useState, useEffect } from 'react';
import { PersonalAccessToken } from '../types';
import { fetchTokens, createToken, revokeToken } from '../api';
import { Key, Plus, Trash2, Copy, Check, Terminal, Shield } from 'lucide-react';

export const AccessTokensView: React.FC = () => {
  const [tokens, setTokens] = useState<PersonalAccessToken[]>([]);
  const [description, setDescription] = useState('');
  const [days, setDays] = useState(30);
  const [createdSecret, setCreatedSecret] = useState<string | null>(null);
  const [copiedSecret, setCopiedSecret] = useState(false);

  const loadTokens = async () => {
    const list = await fetchTokens();
    setTokens(list);
  };

  useEffect(() => {
    loadTokens();
  }, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!description.trim()) return;
    const res = await createToken(description.trim(), ['read', 'write'], days);
    if (res) {
      setCreatedSecret(res.token);
      setDescription('');
      loadTokens();
    }
  };

  const handleRevoke = async (id: string) => {
    if (confirm('Are you sure you want to revoke this token?')) {
      await revokeToken(id);
      loadTokens();
    }
  };

  const handleCopySecret = () => {
    if (createdSecret) {
      navigator.clipboard.writeText(createdSecret);
      setCopiedSecret(true);
      setTimeout(() => setCopiedSecret(false), 2000);
    }
  };

  return (
    <div>
      <div className="page-intro">
        <div>
          <h1 className="page-title">Access Tokens (CLI & CI/CD)</h1>
          <p className="page-desc">Generate Personal Access Tokens to authenticate Maven, npm, Docker, and Cargo CLI tools.</p>
        </div>
      </div>

      {/* 2-Column Grid */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(360px, 1fr))', gap: '1.5rem', marginBottom: '2rem' }}>
        {/* Token Creation Card */}
        <div className="repo-card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
            <Key size={20} color="#29B5E8" />
            <h3 style={{ fontSize: '1.15rem', fontWeight: 600 }}>Generate Personal Access Token</h3>
          </div>
          <p style={{ fontSize: '0.875rem', color: '#64748B', marginBottom: '1.25rem' }}>
            Tokens are scoped credentials for automated CI/CD builds without exposing master passwords.
          </p>

          <form onSubmit={handleCreate}>
            <div className="form-group">
              <label className="form-label">Token Description / Purpose</label>
              <input
                className="form-input"
                placeholder="e.g. GitHub Actions CI Deploy"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                required
              />
            </div>

            <div className="form-group">
              <label className="form-label">Expiration</label>
              <select className="form-select" value={days} onChange={(e) => setDays(parseInt(e.target.value))}>
                <option value={30}>30 Days</option>
                <option value={90}>90 Days</option>
                <option value={365}>1 Year</option>
                <option value={0}>Never Expire</option>
              </select>
            </div>

            <button type="submit" className="btn btn-primary" style={{ width: '100%', justifyContent: 'center' }}>
              <Plus size={16} /> Generate Token
            </button>
          </form>

          {createdSecret && (
            <div style={{ marginTop: '1.25rem', padding: '1rem', background: '#FEF3C7', borderRadius: 'var(--radius-md)', border: '1px solid #FDE68A' }}>
              <div style={{ fontWeight: 600, color: '#92400E', fontSize: '0.875rem', marginBottom: '0.25rem' }}>
                Save your token now (will not be shown again):
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginTop: '0.5rem' }}>
                <input
                  readOnly
                  value={createdSecret}
                  className="form-input"
                  style={{ fontFamily: 'var(--font-mono)', fontSize: '0.85rem', background: '#FFFFFF' }}
                />
                <button className="btn btn-primary btn-sm" onClick={handleCopySecret}>
                  {copiedSecret ? <Check size={14} /> : <Copy size={14} />}
                </button>
              </div>
            </div>
          )}
        </div>

        {/* CLI Configuration Cheat Sheet */}
        <div className="repo-card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
            <Terminal size={20} color="#64748B" />
            <h3 style={{ fontSize: '1.15rem', fontWeight: 600 }}>CLI Integration Guide</h3>
          </div>
          <p style={{ fontSize: '0.875rem', color: '#64748B', marginBottom: '1rem' }}>
            Use your token with standard CLI tools:
          </p>

          <div style={{ fontSize: '0.85rem', color: '#334155' }}>
            <div style={{ fontWeight: 600, marginBottom: '0.2rem' }}>1. NPM (~/.npmrc):</div>
            <div className="code-snippet-box" style={{ margin: '0 0 0.75rem 0', padding: '0.5rem 0.75rem' }}>
              <code>//localhost:8081/repository/npm-hosted/:_authToken=YOUR_TOKEN</code>
            </div>

            <div style={{ fontWeight: 600, marginBottom: '0.2rem' }}>2. Docker CLI:</div>
            <div className="code-snippet-box" style={{ margin: '0 0 0.75rem 0', padding: '0.5rem 0.75rem' }}>
              <code>docker login localhost:8081 -u admin -p YOUR_TOKEN</code>
            </div>

            <div style={{ fontWeight: 600, marginBottom: '0.2rem' }}>3. Cargo (~/.cargo/credentials.toml):</div>
            <div className="code-snippet-box" style={{ margin: 0, padding: '0.5rem 0.75rem' }}>
              <code>[registries.teaql]{'\n'}token = "YOUR_TOKEN"</code>
            </div>
          </div>
        </div>
      </div>

      {/* Active Tokens List */}
      <h2 style={{ fontSize: '1.25rem', fontWeight: 600, marginBottom: '1rem' }}>Active Tokens</h2>
      {tokens.length === 0 ? (
        <div className="repo-card" style={{ color: '#64748B', textAlign: 'center', padding: '2rem' }}>
          No active personal access tokens. Generate one above to use CLI authentication.
        </div>
      ) : (
        <div className="card-grid">
          {tokens.map((token) => (
            <div key={token.id} className="repo-card">
              <div className="card-header">
                <div>
                  <div style={{ fontWeight: 600, fontSize: '1rem' }}>{token.description}</div>
                  <div style={{ fontSize: '0.8rem', color: '#64748B', marginTop: '0.2rem' }}>
                    Created: {new Date(token.created_at).toLocaleDateString()}
                  </div>
                </div>
                <button
                  className="btn btn-secondary btn-sm"
                  style={{ color: '#DC2626' }}
                  onClick={() => handleRevoke(token.id)}
                >
                  <Trash2 size={13} /> Revoke
                </button>
              </div>

              <div style={{ display: 'flex', gap: '0.4rem', marginTop: '0.75rem' }}>
                {token.scopes.map((s) => (
                  <span key={s} className="badge badge-hosted" style={{ fontSize: '0.7rem' }}>
                    {s}
                  </span>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
