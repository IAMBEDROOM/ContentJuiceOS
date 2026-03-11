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
  selectedElementIds: [],
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
    case 'ADD_ELEMENT':
      return {
        ...state,
        designTree: {
          ...state.designTree,
          elements: [...state.designTree.elements, action.element],
        },
        selectedElementIds: [action.element.id],
      };
    case 'SELECT_ELEMENTS': {
      const selectable = action.ids.filter((id) => {
        const el = state.designTree.elements.find((e) => e.id === id);
        return el && !el.locked && el.elementType !== 'sound';
      });
      return { ...state, selectedElementIds: selectable };
    }
    case 'CLEAR_SELECTION':
      return { ...state, selectedElementIds: [] };
    case 'ADD_TO_SELECTION': {
      const newIds = action.ids.filter((id) => {
        const el = state.designTree.elements.find((e) => e.id === id);
        return el && !el.locked && el.elementType !== 'sound' && !state.selectedElementIds.includes(id);
      });
      return { ...state, selectedElementIds: [...state.selectedElementIds, ...newIds] };
    }
    case 'REMOVE_FROM_SELECTION':
      return {
        ...state,
        selectedElementIds: state.selectedElementIds.filter((id) => !action.ids.includes(id)),
      };
    case 'UPDATE_ELEMENT':
      return {
        ...state,
        designTree: {
          ...state.designTree,
          elements: state.designTree.elements.map((el) =>
            el.id === action.id ? { ...el, ...action.changes } : el,
          ),
        },
      };
    case 'UPDATE_ELEMENT_PROPERTIES':
      return {
        ...state,
        designTree: {
          ...state.designTree,
          elements: state.designTree.elements.map((el) =>
            el.id === action.id ? { ...el, ...action.changes } : el,
          ) as typeof state.designTree.elements,
        },
      };
    case 'REORDER_LAYERS': {
      const count = action.orderedIds.length;
      const orderMap = new Map(action.orderedIds.map((id, i) => [id, count - 1 - i]));
      return {
        ...state,
        designTree: {
          ...state.designTree,
          elements: state.designTree.elements.map((el) => {
            const newOrder = orderMap.get(el.id);
            return newOrder !== undefined ? { ...el, layerOrder: newOrder } : el;
          }) as typeof state.designTree.elements,
        },
      };
    }
    case 'UPDATE_ELEMENTS': {
      const updateMap = new Map(action.updates.map((u) => [u.id, u.changes]));
      return {
        ...state,
        designTree: {
          ...state.designTree,
          elements: state.designTree.elements.map((el) => {
            const changes = updateMap.get(el.id);
            return changes ? { ...el, ...changes } : el;
          }),
        },
      };
    }
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
