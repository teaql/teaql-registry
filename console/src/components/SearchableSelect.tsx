import React, { useEffect, useId, useMemo, useRef, useState } from 'react';
import { Check, ChevronDown, Search } from 'lucide-react';

export interface SelectOption {
  value: string;
  label: string;
  searchText?: string;
}

interface Props {
  id?: string;
  ariaLabel?: string;
  value: string;
  options: SelectOption[];
  onChange: (value: string) => void;
  className?: string;
  disabled?: boolean;
}

export const SearchableSelect: React.FC<Props> = ({ id, ariaLabel, value, options, onChange, className = '', disabled = false }) => {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const [highlight, setHighlight] = useState(0);
  const rootRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const listId = useId();
  const searchable = options.length >= 8;
  const selected = options.find(option => option.value === value);
  const filtered = useMemo(() => {
    const needle = query.trim().toLocaleLowerCase();
    if (!needle) return options;
    return options.filter(option => `${option.label} ${option.value} ${option.searchText || ''}`.toLocaleLowerCase().includes(needle));
  }, [options, query]);

  useEffect(() => {
    if (!open) return;
    if (searchable) searchRef.current?.focus();
    const closeOutside = (event: Event) => {
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener('pointerdown', closeOutside);
    document.addEventListener('focusin', closeOutside);
    return () => {
      document.removeEventListener('pointerdown', closeOutside);
      document.removeEventListener('focusin', closeOutside);
    };
  }, [open, searchable]);

  const show = () => {
    if (disabled) return;
    setQuery('');
    setHighlight(Math.max(0, options.findIndex(option => option.value === value)));
    setOpen(true);
  };

  const choose = (option: SelectOption) => {
    onChange(option.value);
    setOpen(false);
    setQuery('');
    triggerRef.current?.focus();
  };

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Escape' && open) {
      event.preventDefault();
      setOpen(false);
      triggerRef.current?.focus();
      return;
    }
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      if (!open) { show(); return; }
      setHighlight(index => Math.max(0, Math.min(filtered.length - 1, index + (event.key === 'ArrowDown' ? 1 : -1))));
      return;
    }
    if (open && event.key === 'Enter') {
      event.preventDefault();
      if (filtered[highlight]) choose(filtered[highlight]);
    }
  };

  return (
    <div ref={rootRef} className={`searchable-select ${className}`}>
      <button
        ref={triggerRef}
        id={id}
        type="button"
        className="select-trigger"
        aria-label={ariaLabel}
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-controls={listId}
        disabled={disabled}
        onClick={() => open ? setOpen(false) : show()}
        onKeyDown={handleKeyDown}
      >
        <span className="select-value">{selected?.label || 'Select an option'}</span>
        <ChevronDown size={17} className={`select-chevron ${open ? 'open' : ''}`} aria-hidden="true" />
      </button>
      {open && (
        <div className="select-popover">
          {searchable && (
            <div className="select-search">
              <Search size={16} aria-hidden="true" />
              <input
                ref={searchRef}
                type="search"
                role="combobox"
                aria-label={`Search ${ariaLabel || 'options'}`}
                aria-controls={listId}
                aria-expanded={true}
                aria-autocomplete="list"
                aria-activedescendant={filtered[highlight] ? `${listId}-option-${highlight}` : undefined}
                placeholder="Search to filter…"
                value={query}
                onChange={event => { setQuery(event.target.value); setHighlight(0); }}
                onKeyDown={handleKeyDown}
              />
            </div>
          )}
          <div id={listId} className="select-options" role="listbox" aria-label={ariaLabel || 'Options'}>
            {filtered.length === 0 ? <div className="select-empty">No matching options</div> : filtered.map((option, index) => (
              <button
                key={option.value}
                id={`${listId}-option-${index}`}
                type="button"
                role="option"
                aria-selected={option.value === value}
                className={`select-option ${index === highlight ? 'highlighted' : ''}`}
                onMouseEnter={() => setHighlight(index)}
                onClick={() => choose(option)}
              >
                <span>{option.label}</span>
                {option.value === value && <Check size={15} aria-hidden="true" />}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
