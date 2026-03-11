import type { Design, DesignElement, DesignTree } from './design';

// ── Viewport ────────────────────────────────────────────────────────

export interface ViewportInfo {
  containerWidth: number;
  containerHeight: number;
  fitZoom: number;
}

// ── Editor State ────────────────────────────────────────────────────

export interface EditorState {
  design: Design | null;
  designTree: DesignTree;
  zoom: number;
  panOffset: { x: number; y: number };
  gridEnabled: boolean;
  gridSize: number;
  gridVisible: boolean;
  snapEnabled: boolean;
  selectedElementIds: string[];
}

// ── Editor Actions ──────────────────────────────────────────────────

export type EditorAction =
  | { type: 'SET_DESIGN'; design: Design }
  | { type: 'SET_ZOOM'; zoom: number }
  | { type: 'SET_PAN'; offset: { x: number; y: number } }
  | { type: 'TOGGLE_GRID' }
  | { type: 'SET_GRID_SIZE'; size: number }
  | { type: 'TOGGLE_SNAP' }
  | { type: 'UPDATE_DESIGN_TREE'; designTree: DesignTree }
  | { type: 'SELECT_ELEMENTS'; ids: string[] }
  | { type: 'CLEAR_SELECTION' }
  | { type: 'ADD_TO_SELECTION'; ids: string[] }
  | { type: 'REMOVE_FROM_SELECTION'; ids: string[] }
  | { type: 'UPDATE_ELEMENT'; id: string; changes: Partial<Pick<DesignElement, 'position' | 'size' | 'rotation'>> }
  | { type: 'UPDATE_ELEMENTS'; updates: Array<{ id: string; changes: Partial<Pick<DesignElement, 'position' | 'size' | 'rotation'>> }> }
  | { type: 'UPDATE_ELEMENT_PROPERTIES'; id: string; changes: Record<string, unknown> }
  | { type: 'REORDER_LAYERS'; orderedIds: string[] };
