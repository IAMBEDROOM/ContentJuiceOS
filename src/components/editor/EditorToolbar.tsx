import { useCallback, useState } from 'react';
import { useEditor } from '../../lib/editor/editorState';
import { calculateFitZoom, calculateCenterOffset } from '../../lib/editor/viewport';
import { updateDesign } from '../../lib/designs';
import { createTextElement } from '../../lib/editor/elementFactories';
import './EditorToolbar.css';

interface EditorToolbarProps {
  containerWidth: number;
  containerHeight: number;
}

export default function EditorToolbar({ containerWidth, containerHeight }: EditorToolbarProps) {
  const { state, dispatch } = useEditor();
  const { design, designTree, zoom, gridVisible, gridSize, snapEnabled } = state;
  const [saving, setSaving] = useState(false);

  const zoomPercent = Math.round(zoom * 100);

  const handleZoomIn = useCallback(() => {
    dispatch({ type: 'SET_ZOOM', zoom: Math.min(5, zoom * 1.25) });
  }, [zoom, dispatch]);

  const handleZoomOut = useCallback(() => {
    dispatch({ type: 'SET_ZOOM', zoom: Math.max(0.1, zoom / 1.25) });
  }, [zoom, dispatch]);

  const handleFitToViewport = useCallback(() => {
    const { width: cw, height: ch } = designTree.canvas;
    const fitZoom = calculateFitZoom(cw, ch, containerWidth, containerHeight);
    const offset = calculateCenterOffset(cw, ch, containerWidth, containerHeight, fitZoom);
    dispatch({ type: 'SET_ZOOM', zoom: fitZoom });
    dispatch({ type: 'SET_PAN', offset });
  }, [designTree.canvas, containerWidth, containerHeight, dispatch]);

  const handleZoom100 = useCallback(() => {
    const { width: cw, height: ch } = designTree.canvas;
    const offset = calculateCenterOffset(cw, ch, containerWidth, containerHeight, 1);
    dispatch({ type: 'SET_ZOOM', zoom: 1 });
    dispatch({ type: 'SET_PAN', offset });
  }, [designTree.canvas, containerWidth, containerHeight, dispatch]);

  const handleSave = useCallback(async () => {
    if (!design) return;
    setSaving(true);
    try {
      await updateDesign(design.id, { config: designTree });
    } finally {
      setSaving(false);
    }
  }, [design, designTree]);

  const handleAddText = useCallback(() => {
    const nextLayerOrder = Math.max(0, ...designTree.elements.map((e) => e.layerOrder)) + 1;
    const element = createTextElement(designTree.canvas.width, designTree.canvas.height, nextLayerOrder);
    dispatch({ type: 'ADD_ELEMENT', element });
  }, [designTree, dispatch]);

  const handleGridSizeChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const val = parseInt(e.target.value, 10);
      if (!isNaN(val)) dispatch({ type: 'SET_GRID_SIZE', size: val });
    },
    [dispatch],
  );

  return (
    <div className="editor-toolbar">
      <span className="design-name">{design?.name ?? 'Untitled'}</span>

      {/* Element creation */}
      <div className="toolbar-group">
        <button onClick={handleAddText} title="Add Text Element">T</button>
      </div>

      <div className="toolbar-divider" />

      {/* Zoom controls */}
      <div className="toolbar-group">
        <button onClick={handleZoomOut} title="Zoom Out">-</button>
        <span className="zoom-display">{zoomPercent}%</span>
        <button onClick={handleZoomIn} title="Zoom In">+</button>
        <button onClick={handleFitToViewport} title="Fit to Viewport">Fit</button>
        <button onClick={handleZoom100} title="100%">100%</button>
      </div>

      <div className="toolbar-divider" />

      {/* Grid controls */}
      <div className="toolbar-group">
        <span className="toolbar-label">Grid</span>
        <button
          onClick={() => dispatch({ type: 'TOGGLE_GRID' })}
          className={gridVisible ? 'active' : ''}
          title="Toggle Grid"
        >
          {gridVisible ? 'ON' : 'OFF'}
        </button>
        <input
          type="number"
          value={gridSize}
          onChange={handleGridSizeChange}
          min={5}
          max={200}
          title="Grid Size"
        />
      </div>

      <div className="toolbar-divider" />

      {/* Snap control */}
      <div className="toolbar-group">
        <span className="toolbar-label">Snap</span>
        <button
          onClick={() => dispatch({ type: 'TOGGLE_SNAP' })}
          className={snapEnabled ? 'active' : ''}
          title="Toggle Snap"
        >
          {snapEnabled ? 'ON' : 'OFF'}
        </button>
      </div>

      <div className="toolbar-divider" />

      <button className="save-btn" onClick={handleSave} disabled={saving}>
        {saving ? 'Saving...' : 'Save'}
      </button>
    </div>
  );
}
