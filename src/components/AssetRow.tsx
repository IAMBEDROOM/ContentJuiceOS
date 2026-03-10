import { useState, useRef } from 'react';
import type { Asset } from '../types/platform';
import { assetFileUrl } from '../lib/assets';
import './AssetRow.css';

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m}:${s.toString().padStart(2, '0')}`;
}

function formatDate(dateStr: string): string {
  // "YYYY-MM-DD HH:MM:SS" → "Mar 10, 2026"
  const d = new Date(dateStr.replace(' ', 'T'));
  if (isNaN(d.getTime())) return dateStr;
  return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
}

interface AssetRowProps {
  asset: Asset;
  assetRoot: string;
}

export default function AssetRow({ asset, assetRoot }: AssetRowProps) {
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

  return (
    <div className="asset-row" onClick={asset.assetType === 'audio' ? toggleAudio : undefined}>
      <div className="asset-row-thumb">
        {asset.assetType === 'image' ? (
          <img src={fileUrl} alt="" className="asset-row-img" loading="lazy" />
        ) : (
          <span className="asset-row-type-icon">{asset.assetType.charAt(0).toUpperCase()}</span>
        )}
      </div>
      <span className="asset-row-filename" title={asset.originalFilename}>
        {asset.originalFilename}
      </span>
      <span className="asset-row-badge">{asset.assetType}</span>
      <span className="asset-row-size">{formatSize(asset.fileSize)}</span>
      <span className="asset-row-duration">
        {asset.duration != null ? formatDuration(asset.duration) : '--'}
      </span>
      <span className="asset-row-date">{formatDate(asset.createdAt)}</span>
      {asset.assetType === 'audio' && (
        <audio ref={audioRef} src={fileUrl} onEnded={() => setPlaying(false)} />
      )}
    </div>
  );
}
