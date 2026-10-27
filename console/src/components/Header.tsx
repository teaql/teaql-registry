import React from 'react';
import { Sparkles, LogOut } from 'lucide-react';

export const Header: React.FC<{ onSignOut: () => void }> = ({ onSignOut }) => {
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
        <span className="header-context">Registry console</span>
        <button type="button" className="header-signout" onClick={onSignOut}><LogOut size={16} /> Sign out</button>
      </div>
    </header>
  );
};
