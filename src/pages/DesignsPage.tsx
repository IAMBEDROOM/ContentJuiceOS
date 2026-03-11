import { useEffect, useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import type { Design, DesignType } from '../types/design';
import { listDesigns, createDesign } from '../lib/designs';
import './DesignsPage.css';

const DESIGN_TYPES: DesignType[] = ['alert', 'overlay', 'scene', 'stinger', 'panel'];

export default function DesignsPage() {
  const navigate = useNavigate();
  const [designs, setDesigns] = useState<Design[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [typeFilter, setTypeFilter] = useState<DesignType | null>(null);

  const fetchDesigns = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await listDesigns({
        typeFilter: typeFilter ?? undefined,
      });
      setDesigns(result.designs);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load designs');
    } finally {
      setLoading(false);
    }
  }, [typeFilter]);

  useEffect(() => {
    fetchDesigns();
  }, [fetchDesigns]);

  const handleCreate = async () => {
    try {
      const design = await createDesign({
        name: 'Untitled Design',
        designType: 'overlay',
      });
      navigate(`/designs/${design.id}/edit`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create design');
    }
  };

  const formatDate = (dateStr: string) => {
    try {
      return new Date(dateStr).toLocaleDateString(undefined, {
        month: 'short',
        day: 'numeric',
        year: 'numeric',
      });
    } catch {
      return dateStr;
    }
  };

  return (
    <div className="designs-page">
      <div className="designs-page-header">
        <h1>Designs</h1>
        <button className="create-btn" onClick={handleCreate}>
          + Create Design
        </button>
      </div>

      <div className="designs-filter-bar">
        <button
          className={typeFilter === null ? 'active' : ''}
          onClick={() => setTypeFilter(null)}
        >
          All
        </button>
        {DESIGN_TYPES.map((t) => (
          <button
            key={t}
            className={typeFilter === t ? 'active' : ''}
            onClick={() => setTypeFilter(t)}
          >
            {t}
          </button>
        ))}
      </div>

      {loading ? (
        <div className="designs-loading">Loading designs...</div>
      ) : error ? (
        <div className="designs-error">{error}</div>
      ) : designs.length === 0 ? (
        <div className="designs-empty">
          <p>No designs yet</p>
          <button className="create-btn" onClick={handleCreate}>
            Create your first design
          </button>
        </div>
      ) : (
        <div className="designs-grid">
          {designs.map((design) => (
            <div key={design.id} className="design-card">
              <div className="design-card-preview">
                {design.config.canvas.width} x {design.config.canvas.height}
              </div>
              <div className="design-card-body">
                <div className="design-card-name">{design.name}</div>
                <div className="design-card-meta">
                  <span className="design-card-type">{design.type}</span>
                  <span>{formatDate(design.updatedAt)}</span>
                </div>
                <div className="design-card-actions">
                  <button
                    className="edit-btn"
                    onClick={() => navigate(`/designs/${design.id}/edit`)}
                  >
                    Edit
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
