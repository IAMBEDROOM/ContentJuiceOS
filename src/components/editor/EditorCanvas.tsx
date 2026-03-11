import { useRef, useState, useEffect, useCallback, useMemo } from 'react';
import { Stage, Layer, Rect } from 'react-konva';
import type Konva from 'konva';
import { useEditor } from '../../lib/editor/editorState';
import { calculateFitZoom, calculateCenterOffset, zoomAtPoint } from '../../lib/editor/viewport';
import { useElementRefs } from '../../lib/editor/useElementRefs';
import { SelectionContext } from '../../lib/editor/SelectionContext';
import EditorGrid from './EditorGrid';
import ElementRenderer from './elements/ElementRenderer';
import SelectionTransformer from './SelectionTransformer';
import MarqueeRect from './MarqueeSelection';
import { useMarquee } from '../../lib/editor/useMarquee';
import './EditorCanvas.css';

const MIN_ZOOM = 0.1;
const MAX_ZOOM = 5.0;
const ZOOM_SENSITIVITY = 0.001;

export default function EditorCanvas() {
  const { state, dispatch } = useEditor();
  const { designTree, zoom, panOffset, selectedElementIds } = state;
  const { canvas } = designTree;

  const containerRef = useRef<HTMLDivElement>(null);
  const stageRef = useRef<Konva.Stage>(null);
  const [containerSize, setContainerSize] = useState({ width: 1, height: 1 });

  // Pan state (transient, not in reducer)
  const [isPanning, setIsPanning] = useState(false);
  const [spaceHeld, setSpaceHeld] = useState(false);
  const lastPointerPos = useRef<{ x: number; y: number } | null>(null);

  // Element refs for Transformer attachment
  const { registerRef, getNodes } = useElementRefs();

  // Marquee state
  const [isMarqueeSelecting, setIsMarqueeSelecting] = useState(false);

  // ── ResizeObserver ──────────────────────────────────────────────
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

  // ── Fit-to-viewport on first load ─────────────────────────────
  const hasInitialFit = useRef(false);
  useEffect(() => {
    if (hasInitialFit.current || containerSize.width <= 1) return;
    hasInitialFit.current = true;

    const fitZoom = calculateFitZoom(canvas.width, canvas.height, containerSize.width, containerSize.height);
    const offset = calculateCenterOffset(canvas.width, canvas.height, containerSize.width, containerSize.height, fitZoom);
    dispatch({ type: 'SET_ZOOM', zoom: fitZoom });
    dispatch({ type: 'SET_PAN', offset });
  }, [containerSize, canvas.width, canvas.height, dispatch]);

  // ── Zoom (scroll wheel) ───────────────────────────────────────
  const handleWheel = useCallback(
    (e: Konva.KonvaEventObject<WheelEvent>) => {
      e.evt.preventDefault();
      const stage = stageRef.current;
      if (!stage) return;

      const pointer = stage.getPointerPosition();
      if (!pointer) return;

      const direction = e.evt.deltaY > 0 ? -1 : 1;
      const newZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoom * (1 + direction * ZOOM_SENSITIVITY * 300)));
      const newOffset = zoomAtPoint(zoom, newZoom, pointer.x, pointer.y, panOffset);

      dispatch({ type: 'SET_ZOOM', zoom: newZoom });
      dispatch({ type: 'SET_PAN', offset: newOffset });
    },
    [zoom, panOffset, dispatch],
  );

  // ── Selection handler (passed via context) ─────────────────────
  const onElementMouseDown = useCallback(
    (id: string, e: Konva.KonvaEventObject<MouseEvent>) => {
      const element = designTree.elements.find((el) => el.id === id);
      if (!element || element.locked) return;

      // Stop event from reaching stage (prevents deselect / marquee)
      e.cancelBubble = true;

      if (e.evt.shiftKey) {
        if (selectedElementIds.includes(id)) {
          dispatch({ type: 'REMOVE_FROM_SELECTION', ids: [id] });
        } else {
          dispatch({ type: 'ADD_TO_SELECTION', ids: [id] });
        }
      } else {
        // If already selected, do nothing (allow drag to start)
        if (!selectedElementIds.includes(id)) {
          dispatch({ type: 'SELECT_ELEMENTS', ids: [id] });
        }
      }
    },
    [designTree.elements, selectedElementIds, dispatch],
  );

  // ── Marquee selection ─────────────────────────────────────────
  const {
    marqueeRect,
    isMarqueeActive,
    handleStageMouseDown: marqueeMouseDown,
    handleStageMouseMove: marqueeMouseMove,
    handleStageMouseUp: marqueeMouseUp,
  } = useMarquee({
    zoom,
    panOffset,
    elements: designTree.elements,
    onSelect: (ids, additive) => {
      if (additive) {
        dispatch({ type: 'ADD_TO_SELECTION', ids });
      } else {
        dispatch({ type: 'SELECT_ELEMENTS', ids });
      }
    },
    onMarqueeActiveChange: setIsMarqueeSelecting,
  });

  // ── Pan (middle-mouse + space+drag) ───────────────────────────
  const handleMouseDown = useCallback(
    (e: Konva.KonvaEventObject<MouseEvent>) => {
      const isMiddle = e.evt.button === 1;
      const isSpaceLeft = spaceHeld && e.evt.button === 0;

      if (isMiddle || isSpaceLeft) {
        e.evt.preventDefault();
        setIsPanning(true);
        lastPointerPos.current = { x: e.evt.clientX, y: e.evt.clientY };
        return;
      }

      // Left-click on empty canvas area
      if (e.evt.button === 0 && e.target === stageRef.current) {
        // Start marquee tracking (actual marquee activates after threshold)
        marqueeMouseDown(e);
      }
    },
    [spaceHeld, marqueeMouseDown],
  );

  const handleMouseMove = useCallback(
    (e: Konva.KonvaEventObject<MouseEvent>) => {
      if (isPanning && lastPointerPos.current) {
        const dx = e.evt.clientX - lastPointerPos.current.x;
        const dy = e.evt.clientY - lastPointerPos.current.y;
        lastPointerPos.current = { x: e.evt.clientX, y: e.evt.clientY };
        dispatch({ type: 'SET_PAN', offset: { x: panOffset.x + dx, y: panOffset.y + dy } });
        return;
      }

      marqueeMouseMove(e);
    },
    [isPanning, panOffset, dispatch, marqueeMouseMove],
  );

  const handleMouseUp = useCallback(
    (e: Konva.KonvaEventObject<MouseEvent>) => {
      if (isPanning) {
        setIsPanning(false);
        lastPointerPos.current = null;
        return;
      }

      if (isMarqueeActive) {
        marqueeMouseUp(e);
        return;
      }

      // Click on empty canvas without drag → clear selection
      if (e.target === stageRef.current) {
        dispatch({ type: 'CLEAR_SELECTION' });
      }

      marqueeMouseUp(e);
    },
    [isPanning, isMarqueeActive, dispatch, marqueeMouseUp],
  );

  // ── Space key tracking + keyboard shortcuts ────────────────────
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.code === 'Space' && !e.repeat) {
        e.preventDefault();
        setSpaceHeld(true);
      }
      if (e.code === 'Escape') {
        dispatch({ type: 'CLEAR_SELECTION' });
      }
      if (e.code === 'KeyA' && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        const allIds = designTree.elements
          .filter((el) => el.visible && !el.locked && el.elementType !== 'sound')
          .map((el) => el.id);
        dispatch({ type: 'SELECT_ELEMENTS', ids: allIds });
      }
    };
    const onKeyUp = (e: KeyboardEvent) => {
      if (e.code === 'Space') {
        setSpaceHeld(false);
        setIsPanning(false);
        lastPointerPos.current = null;
      }
    };

    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('keyup', onKeyUp);
    };
  }, [designTree.elements, dispatch]);

  // ── Sorted elements ───────────────────────────────────────────
  const sortedElements = useMemo(
    () => [...designTree.elements].sort((a, b) => a.layerOrder - b.layerOrder),
    [designTree.elements],
  );

  // ── Selection context value ───────────────────────────────────
  const selectionContextValue = useMemo(
    () => ({ selectedElementIds, registerRef, onElementMouseDown }),
    [selectedElementIds, registerRef, onElementMouseDown],
  );

  // ── Cursor class ──────────────────────────────────────────────
  const cursorClass = isPanning
    ? 'panning'
    : spaceHeld
      ? 'pan-ready'
      : isMarqueeSelecting
        ? 'selecting'
        : '';

  return (
    <div ref={containerRef} className={`editor-canvas-container ${cursorClass}`}>
      <Stage
        ref={stageRef}
        width={containerSize.width}
        height={containerSize.height}
        onWheel={handleWheel}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
      >
        {/* Layer 0: Background */}
        <Layer listening={false}>
          {/* Editor background (dark, full viewport) */}
          <Rect x={0} y={0} width={containerSize.width} height={containerSize.height} fill="#08090e" />
          {/* Canvas artboard */}
          <Rect
            x={panOffset.x}
            y={panOffset.y}
            width={canvas.width * zoom}
            height={canvas.height * zoom}
            fill={designTree.backgroundColor}
            shadowColor="rgba(0,0,0,0.5)"
            shadowBlur={20}
            shadowEnabled
          />
        </Layer>

        {/* Layer 1: Grid */}
        <EditorGrid />

        {/* Layer 2: Content + Transformer + Marquee */}
        <Layer scaleX={zoom} scaleY={zoom} x={panOffset.x} y={panOffset.y}>
          <SelectionContext.Provider value={selectionContextValue}>
            {sortedElements.map((element) => (
              <ElementRenderer key={element.id} element={element} />
            ))}
            <SelectionTransformer getNodes={getNodes} />
            {marqueeRect && <MarqueeRect rect={marqueeRect} />}
          </SelectionContext.Provider>
        </Layer>
      </Stage>
    </div>
  );
}
