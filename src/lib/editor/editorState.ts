import { createContext, useContext } from 'react';
import type { DesignTree } from '../../types/design';
import type { EditorState, EditorAction } from '../../types/editor';

// ── Default Design Tree ─────────────────────────────────────────────

const defaultDesignTree: DesignTree = {
  schemaVersion: 1,
  canvas: { width: 1920, height: 1080 },
  backgroundColor: '#0A0D14',
  elements: [],
};

// ── Initial State ───────────────────────────────────────────────────

export const initialEditorState: EditorState = {
  design: null,
  designTree: defaultDesignTree,
  zoom: 1,
  panOffset: { x: 0, y: 0 },
  gridEnabled: true,
  gridSize: 20,
  gridVisible: true,
  snapEnabled: true,
};

// ── Reducer ─────────────────────────────────────────────────────────

export function editorReducer(state: EditorState, action: EditorAction): EditorState {
  switch (action.type) {
    case 'SET_DESIGN':
      return {
        ...state,
        design: action.design,
        designTree: action.design.config,
      };
    case 'SET_ZOOM':
      return { ...state, zoom: action.zoom };
    case 'SET_PAN':
      return { ...state, panOffset: action.offset };
    case 'TOGGLE_GRID':
      return { ...state, gridVisible: !state.gridVisible };
    case 'SET_GRID_SIZE':
      return { ...state, gridSize: Math.max(5, action.size) };
    case 'TOGGLE_SNAP':
      return { ...state, snapEnabled: !state.snapEnabled };
    case 'UPDATE_DESIGN_TREE':
      return { ...state, designTree: action.designTree };
    default:
      return state;
  }
}

// ── Context ─────────────────────────────────────────────────────────

export interface EditorContextValue {
  state: EditorState;
  dispatch: React.Dispatch<EditorAction>;
}

export const EditorContext = createContext<EditorContextValue | null>(null);

export function useEditor(): EditorContextValue {
  const ctx = useContext(EditorContext);
  if (!ctx) {
    throw new Error('useEditor must be used within an EditorProvider');
  }
  return ctx;
}
