import { useEffect, useState, useReducer, useRef, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import type { Design } from '../types/design';
import { getDesign } from '../lib/designs';
import {
  EditorContext,
  editorReducer,
  initialEditorState,
} from '../lib/editor/editorState';
import EditorToolbar from '../components/editor/EditorToolbar';
import EditorCanvas from '../components/editor/EditorCanvas';
import './DesignEditorPage.css';

export default function DesignEditorPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [state, dispatch] = useReducer(editorReducer, initialEditorState);
  const [containerSize, setContainerSize] = useState({ width: 1, height: 1 });
  const containerRef = useRef<HTMLDivElement>(null);

  // Track container size for toolbar's fit-to-viewport
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) return;
      const { width, height } = entry.contentRect;
      setContainerSize({ width, height });
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const loadDesign = useCallback(async () => {
    if (!id) {
      setError('No design ID provided');
      setLoading(false);
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const design: Design = await getDesign(id);
      dispatch({ type: 'SET_DESIGN', design });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load design');
    } finally {
      setLoading(false);
    }
  }, [id]);

  useEffect(() => {
    loadDesign();
  }, [loadDesign]);

  if (loading) {
    return (
      <div className="design-editor-page">
        <div className="design-editor-loading">Loading editor...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="design-editor-page">
        <button className="design-editor-back" onClick={() => navigate('/designs')}>
          &larr; Back to Designs
        </button>
        <div className="design-editor-error">{error}</div>
      </div>
    );
  }

  return (
    <EditorContext.Provider value={{ state, dispatch }}>
      <div className="design-editor-page">
        <div className="design-editor-header">
          <button className="design-editor-back" onClick={() => navigate('/designs')}>
            &larr; Back
          </button>
          <EditorToolbar
            containerWidth={containerSize.width}
            containerHeight={containerSize.height}
          />
        </div>
        <div ref={containerRef} className="design-editor-canvas-area">
          <EditorCanvas />
        </div>
      </div>
    </EditorContext.Provider>
  );
}
