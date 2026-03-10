import { useState, useRef } from 'react';
import type { Asset } from '../types/platform';
import { assetFileUrl } from '../lib/assets';
import './AssetCard.css';

/** Format byte sizes for display. */
function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

interface AssetCardProps {
  asset: Asset;
  assetRoot: string;
  onDelete?: (id: string) => void;
  selected?: boolean;
  onSelect?: (id: string) => void;
  selectionMode?: boolean;
}

export default function AssetCard({ asset, assetRoot, onDelete, selected, onSelect, selectionMode }: AssetCardProps) {
  const [playing, setPlaying] = useState(false);
  const audioRef = useRef<HTMLAudioElement>(null);
  const absolutePath = assetRoot + '/' + asset.filePath;
  const fileUrl = assetFileUrl(absolutePath);

  const toggleAudio = () => {
    const el = audioRef.current;
    if (!el) return;
    if (playing) {
      el.pause();
      el.currentTime = 0;
    } else {
      el.play();
    }
    setPlaying(!playing);
  };

  const handleAudioEnded = () => setPlaying(false);

  return (
    <div className={`asset-card${selected ? ' selected' : ''}`}>
      {selectionMode && (
        <label className="asset-card-checkbox" onClick={(e) => e.stopPropagation()}>
          <input
            type="checkbox"
            checked={selected ?? false}
            onChange={() => onSelect?.(asset.id)}
          />
        </label>
      )}
      {onDelete && (
        <button
          className="asset-card-delete-btn"
          onClick={(e) => { e.stopPropagation(); onDelete(asset.id); }}
          title="Delete asset"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
          </svg>
        </button>
      )}
      <div className="asset-card-preview">
        {asset.assetType === 'image' && (
          <img src={fileUrl} alt={asset.originalFilename} className="asset-card-img" loading="lazy" />
        )}
        {asset.assetType === 'audio' && (
          <button className="asset-card-icon-btn" onClick={toggleAudio} title={playing ? 'Stop' : 'Play'}>
            <svg width="32" height="32" viewBox="0 0 24 24" fill="currentColor">
              {playing ? (
                <>
                  <rect x="6" y="5" width="4" height="14" rx="1" />
                  <rect x="14" y="5" width="4" height="14" rx="1" />
                </>
              ) : (
                <path d="M8 5v14l11-7z" />
              )}
            </svg>
            <audio ref={audioRef} src={fileUrl} onEnded={handleAudioEnded} />
          </button>
        )}
        {asset.assetType === 'video' && (
          <div className="asset-card-icon" title="Video">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="currentColor">
              <path d="M4 4h16a2 2 0 012 2v12a2 2 0 01-2 2H4a2 2 0 01-2-2V6a2 2 0 012-2zm6 4v8l6-4-6-4z" />
            </svg>
          </div>
        )}
        {asset.assetType === 'font' && (
          <div className="asset-card-icon" title="Font">
            <span className="asset-card-icon-letter">Aa</span>
          </div>
        )}
        {asset.assetType === 'animation' && (
          <div className="asset-card-icon" title="Animation">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="currentColor">
              <circle cx="12" cy="12" r="3" />
              <path d="M12 2a10 10 0 100 20 10 10 0 000-20zm0 18a8 8 0 110-16 8 8 0 010 16z" opacity="0.3" />
              <path d="M12 6a6 6 0 100 12 6 6 0 000-12zm0 10a4 4 0 110-8 4 4 0 010 8z" opacity="0.5" />
            </svg>
          </div>
        )}
        {asset.assetType === 'caption' && (
          <div className="asset-card-icon" title="Caption">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="currentColor">
              <rect x="2" y="4" width="20" height="16" rx="2" fill="none" stroke="currentColor" strokeWidth="2" />
              <rect x="5" y="14" width="14" height="3" rx="1" opacity="0.6" />
            </svg>
          </div>
        )}
      </div>

      <div className="asset-card-info">
        <span className="asset-card-filename" title={asset.originalFilename}>
          {asset.originalFilename}
        </span>
        <div className="asset-card-meta">
          <span className="asset-card-badge">{asset.assetType}</span>
          <span className="asset-card-size">{formatSize(asset.fileSize)}</span>
        </div>
      </div>
    </div>
  );
}
