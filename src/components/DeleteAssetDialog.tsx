import { useEffect, useCallback } from 'react';
import type { Asset, AssetReference } from '../types/platform';
import './DeleteAssetDialog.css';

interface DeleteAssetDialogProps {
  assets: Asset[];
  references: Map<string, AssetReference[]>;
  onConfirm: (force: boolean) => void;
  onCancel: () => void;
}

export default function DeleteAssetDialog({
  assets,
  references,
  onConfirm,
  onCancel,
}: DeleteAssetDialogProps) {
  // Categorize references
  const allRefs: AssetReference[] = [];
  const blockedAssets: { asset: Asset; reason: string }[] = [];
  const softRefAssets: { asset: Asset; refs: AssetReference[] }[] = [];

  for (const asset of assets) {
    const refs = references.get(asset.id) ?? [];
    const hasSourceVideoRef = refs.some((r) => r.refType === 'project');

    // We need to distinguish project FK refs from config refs.
    // For simplicity, if it has a project ref, we treat it as potentially blocked.
    // The backend will give the definitive answer — here we show warnings.
    if (hasSourceVideoRef) {
      blockedAssets.push({
        asset,
        reason: 'Used as a project source video',
      });
    }

    const softRefs = refs.filter((r) => r.refType !== 'project');
    if (softRefs.length > 0) {
      softRefAssets.push({ asset, refs: softRefs });
      allRefs.push(...softRefs);
    }
  }

  const hasRefs = allRefs.length > 0 || blockedAssets.length > 0;
  const hasOnlySoftRefs = allRefs.length > 0 && blockedAssets.length === 0;
  const deletableCount = assets.length - blockedAssets.length;

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === 'Escape') onCancel();
    },
    [onCancel],
  );

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) onCancel();
  };

  return (
    <div className="delete-dialog-backdrop" onClick={handleBackdropClick}>
      <div className="delete-dialog">
        <h3>Delete {assets.length === 1 ? 'Asset' : `${assets.length} Assets`}?</h3>

        {!hasRefs && (
          <p>
            {assets.length === 1
              ? `"${assets[0].originalFilename}" will be permanently removed from your library and disk.`
              : `${assets.length} assets will be permanently removed from your library and disk.`}
          </p>
        )}

        {blockedAssets.length > 0 && (
          <div className="delete-dialog-refs">
            <h4>Cannot be deleted (project source video)</h4>
            <ul className="delete-blocked-list">
              {blockedAssets.map(({ asset, reason }) => (
                <li key={asset.id} className="delete-blocked-item">
                  {asset.originalFilename} &mdash; {reason}. Delete the project first.
                </li>
              ))}
            </ul>
          </div>
        )}

        {softRefAssets.length > 0 && (
          <div className="delete-dialog-refs">
            <h4>Referenced by other items</h4>
            <ul className="delete-ref-list">
              {softRefAssets.flatMap(({ refs }) =>
                refs.map((ref) => (
                  <li key={`${ref.refId}-${ref.refType}`} className="delete-ref-item">
                    <span className="delete-ref-badge">{ref.refType.replace('_', ' ')}</span>
                    <span className="delete-ref-name">{ref.refName}</span>
                  </li>
                )),
              )}
            </ul>
            <p style={{ marginTop: 8 }}>Deleting will leave broken references in these items.</p>
          </div>
        )}

        <div className="delete-dialog-actions">
          <button className="btn-cancel" onClick={onCancel}>
            Cancel
          </button>
          {deletableCount > 0 && (
            <button className="btn-delete" onClick={() => onConfirm(hasOnlySoftRefs)}>
              {hasOnlySoftRefs ? 'Delete Anyway' : 'Delete'}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
