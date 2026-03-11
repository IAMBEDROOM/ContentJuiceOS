import type { Design, DesignTree } from './design';

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
}

// ── Editor Actions ──────────────────────────────────────────────────

export type EditorAction =
  | { type: 'SET_DESIGN'; design: Design }
  | { type: 'SET_ZOOM'; zoom: number }
  | { type: 'SET_PAN'; offset: { x: number; y: number } }
  | { type: 'TOGGLE_GRID' }
  | { type: 'SET_GRID_SIZE'; size: number }
  | { type: 'TOGGLE_SNAP' }
  | { type: 'UPDATE_DESIGN_TREE'; designTree: DesignTree };
