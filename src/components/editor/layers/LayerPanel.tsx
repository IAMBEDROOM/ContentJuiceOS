import { useState, useMemo, useCallback } from 'react';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import type { DragEndEvent } from '@dnd-kit/core';
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { useEditor } from '../../../lib/editor/editorState';
import LayerRow from './LayerRow';
import './LayerPanel.css';

export default function LayerPanel() {
  const { state, dispatch } = useEditor();
  const [collapsed, setCollapsed] = useState(false);

  // Sort elements by layerOrder descending (front = top of list)
  const sortedElements = useMemo(() => {
    return [...state.designTree.elements].sort((a, b) => b.layerOrder - a.layerOrder);
  }, [state.designTree.elements]);

  const sortedIds = useMemo(() => sortedElements.map((el) => el.id), [sortedElements]);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      const { active, over } = event;
      if (!over || active.id === over.id) return;

      const oldIndex = sortedIds.indexOf(active.id as string);
      const newIndex = sortedIds.indexOf(over.id as string);
      if (oldIndex === -1 || newIndex === -1) return;

      const newOrder = [...sortedIds];
      newOrder.splice(oldIndex, 1);
      newOrder.splice(newIndex, 0, active.id as string);

      dispatch({ type: 'REORDER_LAYERS', orderedIds: newOrder });
    },
    [sortedIds, dispatch],
  );

  const handleSelect = useCallback(
    (id: string, addToSelection: boolean) => {
      if (addToSelection) {
        if (state.selectedElementIds.includes(id)) {
          dispatch({ type: 'REMOVE_FROM_SELECTION', ids: [id] });
        } else {
          dispatch({ type: 'ADD_TO_SELECTION', ids: [id] });
        }
      } else {
        dispatch({ type: 'SELECT_ELEMENTS', ids: [id] });
      }
    },
    [state.selectedElementIds, dispatch],
  );

  const handleRename = useCallback(
    (id: string, name: string) => {
      dispatch({ type: 'UPDATE_ELEMENT_PROPERTIES', id, changes: { name } });
    },
    [dispatch],
  );

  const handleToggleVisible = useCallback(
    (id: string) => {
      const el = state.designTree.elements.find((e) => e.id === id);
      if (!el) return;
      dispatch({
        type: 'UPDATE_ELEMENT_PROPERTIES',
        id,
        changes: { visible: !el.visible },
      });
    },
    [state.designTree.elements, dispatch],
  );

  const handleToggleLock = useCallback(
    (id: string) => {
      const el = state.designTree.elements.find((e) => e.id === id);
      if (!el) return;
      const willLock = !el.locked;
      dispatch({
        type: 'UPDATE_ELEMENT_PROPERTIES',
        id,
        changes: { locked: willLock },
      });
      // Deselect if locking an already-selected element
      if (willLock && state.selectedElementIds.includes(id)) {
        dispatch({ type: 'REMOVE_FROM_SELECTION', ids: [id] });
      }
    },
    [state.designTree.elements, state.selectedElementIds, dispatch],
  );

  return (
    <div className={`layer-panel${collapsed ? ' layer-panel--collapsed' : ''}`}>
      <div className="lp-header">
        {!collapsed && <span className="lp-header-title">Layers</span>}
        <button
          className="lp-btn lp-collapse-btn"
          title={collapsed ? 'Expand layers' : 'Collapse layers'}
          onClick={() => setCollapsed((c) => !c)}
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            {collapsed ? (
              <path d="M5 3l4 4-4 4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
            ) : (
              <path d="M9 3L5 7l4 4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
            )}
          </svg>
        </button>
      </div>
      {!collapsed && (
        sortedElements.length === 0 ? (
          <div className="lp-empty">No layers</div>
        ) : (
          <div className="lp-list">
            <DndContext
              sensors={sensors}
              collisionDetection={closestCenter}
              onDragEnd={handleDragEnd}
            >
              <SortableContext items={sortedIds} strategy={verticalListSortingStrategy}>
                {sortedElements.map((el) => (
                  <LayerRow
                    key={el.id}
                    element={el}
                    isSelected={state.selectedElementIds.includes(el.id)}
                    onSelect={handleSelect}
                    onRename={handleRename}
                    onToggleVisible={handleToggleVisible}
                    onToggleLock={handleToggleLock}
                  />
                ))}
              </SortableContext>
            </DndContext>
          </div>
        )
      )}
    </div>
  );
}
