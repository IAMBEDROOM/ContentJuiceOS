import { useRef, useEffect } from 'react';
import type { AssetType } from '../types/platform';
import './AssetToolbar.css';

const TYPE_CHIPS: { label: string; value: AssetType | null }[] = [
  { label: 'All', value: null },
  { label: 'Image', value: 'image' },
  { label: 'Audio', value: 'audio' },
  { label: 'Video', value: 'video' },
  { label: 'Font', value: 'font' },
  { label: 'Animation', value: 'animation' },
  { label: 'Caption', value: 'caption' },
];

interface AssetToolbarProps {
  search: string;
  onSearchChange: (value: string) => void;
  typeFilter: AssetType | null;
  onTypeFilterChange: (value: AssetType | null) => void;
  viewMode: 'grid' | 'list';
  onViewModeChange: (mode: 'grid' | 'list') => void;
  onImport: () => void;
  importing: boolean;
  selectedCount?: number;
  onDeleteSelected?: () => void;
  onSelectAll?: () => void;
  onClearSelection?: () => void;
}

export default function AssetToolbar({
  search,
  onSearchChange,
  typeFilter,
  onTypeFilterChange,
  viewMode,
  onViewModeChange,
  onImport,
  importing,
  selectedCount = 0,
  onDeleteSelected,
  onSelectAll,
  onClearSelection,
}: AssetToolbarProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout>>();

  // Debounced search input
  const handleInput = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => onSearchChange(val), 300);
  };

  useEffect(() => {
    return () => clearTimeout(timerRef.current);
  }, []);

  return (
    <div className="asset-toolbar">
      <div className="asset-toolbar-left">
        <input
          ref={inputRef}
          className="asset-search"
          type="text"
          placeholder="Search assets..."
          defaultValue={search}
          onChange={handleInput}
        />
        <div className="asset-type-chips">
          {TYPE_CHIPS.map((chip) => (
            <button
              key={chip.label}
              className={`asset-chip ${typeFilter === chip.value ? 'active' : ''}`}
              onClick={() => onTypeFilterChange(chip.value)}
            >
              {chip.label}
            </button>
          ))}
        </div>
      </div>

      <div className="asset-toolbar-right">
        {selectedCount > 0 && (
          <div className="asset-selection-actions">
            <button className="btn btn-delete-selected" onClick={onDeleteSelected}>
              Delete {selectedCount} selected
            </button>
            <button className="btn btn-clear-selection" onClick={onClearSelection}>
              Clear
            </button>
          </div>
        )}
        {selectedCount === 0 && onSelectAll && (
          <button className="btn btn-select-all" onClick={onSelectAll}>
            Select
          </button>
        )}
        <div className="asset-view-toggle">
          <button
            className={`view-btn ${viewMode === 'grid' ? 'active' : ''}`}
            onClick={() => onViewModeChange('grid')}
            title="Grid view"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <rect x="1" y="1" width="6" height="6" rx="1" />
              <rect x="9" y="1" width="6" height="6" rx="1" />
              <rect x="1" y="9" width="6" height="6" rx="1" />
              <rect x="9" y="9" width="6" height="6" rx="1" />
            </svg>
          </button>
          <button
            className={`view-btn ${viewMode === 'list' ? 'active' : ''}`}
            onClick={() => onViewModeChange('list')}
            title="List view"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <rect x="1" y="2" width="14" height="2.5" rx="1" />
              <rect x="1" y="6.75" width="14" height="2.5" rx="1" />
              <rect x="1" y="11.5" width="14" height="2.5" rx="1" />
            </svg>
          </button>
        </div>

        <button className="btn btn-import" onClick={onImport} disabled={importing}>
          {importing ? 'Importing...' : 'Import'}
        </button>
      </div>
    </div>
  );
}
