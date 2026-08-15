import React from 'react';
import { Sparkles, Layers } from 'lucide-react';

export const Header: React.FC = () => {
  return (
    <header className="app-header">
      <div className="brand-container">
        <Sparkles className="brand-logo" size={22} />
        <span>TeaQL Registry</span>
        <span style={{ fontSize: '0.75rem', opacity: 0.6, fontWeight: 400, marginLeft: '0.25rem' }}>
          v0.1.0
        </span>
      </div>

      <div className="header-meta">
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.4rem', color: '#94A3B8' }}>
          <Layers size={16} />
          <span>Tenant: default</span>
        </div>
        <div className="status-badge">
          <span className="status-dot"></span>
          <span>Engine Online</span>
        </div>
      </div>
    </header>
  );
};
