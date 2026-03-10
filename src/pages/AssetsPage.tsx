import { useEffect, useState, useCallback, useRef } from 'react';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import type { Asset, AssetType, AssetReference } from '../types/platform';
import {
  listAssets,
  importAsset,
  getAssetRoot,
  openImportDialog,
  checkAssetReferences,
  deleteAsset,
  deleteAssetsBatch,
} from '../lib/assets';
import AssetToolbar from '../components/AssetToolbar';
import AssetCard from '../components/AssetCard';
import AssetRow from '../components/AssetRow';
import DeleteAssetDialog from '../components/DeleteAssetDialog';
import './AssetsPage.css';

const PAGE_SIZE = 30;

export default function AssetsPage() {
  const [assets, setAssets] = useState<Asset[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [typeFilter, setTypeFilter] = useState<AssetType | null>(null);
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [assetRoot, setAssetRoot] = useState('');
  const [importing, setImporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const [deleteTargets, setDeleteTargets] = useState<Asset[]>([]);
  const [deleteRefs, setDeleteRefs] = useState<Map<string, AssetReference[]>>(new Map());
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    return () => { mountedRef.current = false; };
  }, []);

  // Resolve asset root on mount
  useEffect(() => {
    getAssetRoot().then(setAssetRoot).catch(() => {});
  }, []);

  // Fetch assets when filters change
  const fetchAssets = useCallback(async (reset = true) => {
    setLoading(true);
    setError(null);
    try {
      const offset = reset ? 0 : assets.length;
      const result = await listAssets({
        typeFilter: typeFilter ?? undefined,
        search: search || undefined,
        limit: PAGE_SIZE,
        offset,
      });
      if (!mountedRef.current) return;
      setAssets(reset ? result.assets : [...assets, ...result.assets]);
      setTotal(result.total);
    } catch (e) {
      if (mountedRef.current) setError(String(e));
    } finally {
      if (mountedRef.current) setLoading(false);
    }
  }, [search, typeFilter, assets]);

  // Re-fetch when search or type filter changes
  useEffect(() => {
    fetchAssets(true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [search, typeFilter]);

  // Import handler
  const handleImport = useCallback(async () => {
    setImporting(true);
    setError(null);
    try {
      const paths = await openImportDialog();
      if (!paths || paths.length === 0) {
        setImporting(false);
        return;
      }
      for (const path of paths) {
        await importAsset(path);
      }
      if (mountedRef.current) await fetchAssets(true);
    } catch (e) {
      if (mountedRef.current) setError(String(e));
    } finally {
      if (mountedRef.current) setImporting(false);
    }
  }, [fetchAssets]);

  // Import from file paths (drag-and-drop)
  const importPaths = useCallback(async (paths: string[]) => {
    setImporting(true);
    setError(null);
    try {
      for (const path of paths) {
        await importAsset(path);
      }
      if (mountedRef.current) await fetchAssets(true);
    } catch (e) {
      if (mountedRef.current) setError(String(e));
    } finally {
      if (mountedRef.current) setImporting(false);
    }
  }, [fetchAssets]);

  // Selection handlers
  const handleSelect = useCallback((id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  const handleSelectAll = useCallback(() => {
    setSelectedIds(new Set(assets.map((a) => a.id)));
  }, [assets]);

  const handleClearSelection = useCallback(() => {
    setSelectedIds(new Set());
  }, []);

  // Deletion handlers
  const handleDeleteRequest = useCallback(async (ids: string[]) => {
    setError(null);
    try {
      const targets = assets.filter((a) => ids.includes(a.id));
      const refsMap = new Map<string, AssetReference[]>();
      for (const id of ids) {
        const refs = await checkAssetReferences(id);
        if (refs.length > 0) refsMap.set(id, refs);
      }
      setDeleteTargets(targets);
      setDeleteRefs(refsMap);
      setShowDeleteDialog(true);
    } catch (e) {
      setError(String(e));
    }
  }, [assets]);

  const handleDeleteConfirm = useCallback(async (force: boolean) => {
    setShowDeleteDialog(false);
    setError(null);
    try {
      if (deleteTargets.length === 1) {
        await deleteAsset(deleteTargets[0].id, force);
      } else {
        await deleteAssetsBatch(deleteTargets.map((a) => a.id), force);
      }
      setSelectedIds(new Set());
      if (mountedRef.current) await fetchAssets(true);
    } catch (e) {
      if (mountedRef.current) setError(String(e));
    }
  }, [deleteTargets, fetchAssets]);

  const handleDeleteCancel = useCallback(() => {
    setShowDeleteDialog(false);
    setDeleteTargets([]);
    setDeleteRefs(new Map());
  }, []);

  const selectionMode = selectedIds.size > 0;

  // Drag-and-drop via Tauri webview event
  useEffect(() => {
    const webview = getCurrentWebview();
    const unlisten = webview.onDragDropEvent((event) => {
      if (event.payload.type === 'over') {
        setDragOver(true);
      } else if (event.payload.type === 'drop') {
        setDragOver(false);
        importPaths(event.payload.paths);
      } else {
        setDragOver(false);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [importPaths]);

  const hasMore = assets.length < total;

  return (
    <div className="assets-page">
      <h2 className="assets-title">Asset Library</h2>
      <p className="assets-subtitle">
        Import and manage images, audio, video, fonts, and more.
      </p>

      <AssetToolbar
        search={search}
        onSearchChange={setSearch}
        typeFilter={typeFilter}
        onTypeFilterChange={setTypeFilter}
        viewMode={viewMode}
        onViewModeChange={setViewMode}
        onImport={handleImport}
        importing={importing}
        selectedCount={selectedIds.size}
        onDeleteSelected={() => handleDeleteRequest([...selectedIds])}
        onSelectAll={handleSelectAll}
        onClearSelection={handleClearSelection}
      />

      {error && <div className="assets-error">{error}</div>}

      {dragOver && (
        <div className="assets-drop-overlay">
          <span>Drop files to import</span>
        </div>
      )}

      {!loading && assets.length === 0 && (
        <div className="assets-empty">
          <p>No assets yet</p>
          <p className="assets-empty-hint">
            Click Import or drag files here to get started.
          </p>
        </div>
      )}

      {viewMode === 'grid' ? (
        <div className="assets-grid">
          {assets.map((asset) => (
            <AssetCard
              key={asset.id}
              asset={asset}
              assetRoot={assetRoot}
              onDelete={(id) => handleDeleteRequest([id])}
              selected={selectedIds.has(asset.id)}
              onSelect={handleSelect}
              selectionMode={selectionMode}
            />
          ))}
        </div>
      ) : (
        <div className="assets-list">
          {assets.length > 0 && (
            <div className="assets-list-header">
              <span />
              <span>Filename</span>
              <span>Type</span>
              <span>Size</span>
              <span>Duration</span>
              <span>Date</span>
            </div>
          )}
          {assets.map((asset) => (
            <AssetRow
              key={asset.id}
              asset={asset}
              assetRoot={assetRoot}
              onDelete={(id) => handleDeleteRequest([id])}
              selected={selectedIds.has(asset.id)}
              onSelect={handleSelect}
              selectionMode={selectionMode}
            />
          ))}
        </div>
      )}

      {hasMore && (
        <button
          className="btn btn-load-more"
          onClick={() => fetchAssets(false)}
          disabled={loading}
        >
          {loading ? 'Loading...' : 'Load More'}
        </button>
      )}

      {showDeleteDialog && (
        <DeleteAssetDialog
          assets={deleteTargets}
          references={deleteRefs}
          onConfirm={handleDeleteConfirm}
          onCancel={handleDeleteCancel}
        />
      )}
    </div>
  );
}
