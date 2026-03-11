import { useState, useRef, useEffect } from 'react';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import type { DesignElement } from '../../../types/design';
import LayerTypeIcon from './layerIcons';

interface LayerRowProps {
  element: DesignElement;
  isSelected: boolean;
  onSelect: (id: string, addToSelection: boolean) => void;
  onRename: (id: string, name: string) => void;
  onToggleVisible: (id: string) => void;
  onToggleLock: (id: string) => void;
}

export default function LayerRow({
  element,
  isSelected,
  onSelect,
  onRename,
  onToggleVisible,
  onToggleLock,
}: LayerRowProps) {
  const [editing, setEditing] = useState(false);
  const [editValue, setEditValue] = useState(element.name);
  const inputRef = useRef<HTMLInputElement>(null);

  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: element.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.3 : undefined,
  };

  useEffect(() => {
    if (editing) {
      inputRef.current?.focus();
      inputRef.current?.select();
    }
  }, [editing]);

  const commitRename = () => {
    const trimmed = editValue.trim();
    if (trimmed && trimmed !== element.name) {
      onRename(element.id, trimmed);
    } else {
      setEditValue(element.name);
    }
    setEditing(false);
  };

  const cancelRename = () => {
    setEditValue(element.name);
    setEditing(false);
  };

  const rowClasses = [
    'lp-row',
    isSelected && 'lp-row--selected',
    !element.visible && 'lp-row--hidden',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={rowClasses}
      onClick={(e) => onSelect(element.id, e.shiftKey)}
    >
      {/* Drag handle */}
      <button
        className="lp-btn lp-drag-handle"
        {...attributes}
        {...listeners}
        tabIndex={-1}
        aria-label="Drag to reorder"
      >
        <svg width="10" height="14" viewBox="0 0 10 14" fill="currentColor">
          <circle cx="3" cy="3" r="1.2" />
          <circle cx="7" cy="3" r="1.2" />
          <circle cx="3" cy="7" r="1.2" />
          <circle cx="7" cy="7" r="1.2" />
          <circle cx="3" cy="11" r="1.2" />
          <circle cx="7" cy="11" r="1.2" />
        </svg>
      </button>

      {/* Type icon */}
      <LayerTypeIcon elementType={element.elementType} />

      {/* Name / inline edit */}
      {editing ? (
        <input
          ref={inputRef}
          className="lp-name-input"
          value={editValue}
          onChange={(e) => setEditValue(e.target.value)}
          onBlur={commitRename}
          onKeyDown={(e) => {
            if (e.key === 'Enter') commitRename();
            if (e.key === 'Escape') cancelRename();
          }}
          onClick={(e) => e.stopPropagation()}
        />
      ) : (
        <span
          className="lp-name"
          onDoubleClick={(e) => {
            e.stopPropagation();
            setEditValue(element.name);
            setEditing(true);
          }}
        >
          {element.name}
        </span>
      )}

      {/* Visibility toggle */}
      <button
        className="lp-btn"
        title={element.visible ? 'Hide' : 'Show'}
        onClick={(e) => {
          e.stopPropagation();
          onToggleVisible(element.id);
        }}
      >
        {element.visible ? (
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M1 7s2.5-4 6-4 6 4 6 4-2.5 4-6 4-6-4-6-4z" stroke="currentColor" strokeWidth="1.3" />
            <circle cx="7" cy="7" r="2" stroke="currentColor" strokeWidth="1.3" />
          </svg>
        ) : (
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M1 7s2.5-4 6-4 6 4 6 4-2.5 4-6 4-6-4-6-4z" stroke="currentColor" strokeWidth="1.3" />
            <line x1="2" y1="2" x2="12" y2="12" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          </svg>
        )}
      </button>

      {/* Lock toggle */}
      <button
        className="lp-btn"
        title={element.locked ? 'Unlock' : 'Lock'}
        onClick={(e) => {
          e.stopPropagation();
          onToggleLock(element.id);
        }}
      >
        {element.locked ? (
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="3" y="6" width="8" height="6" rx="1" stroke="currentColor" strokeWidth="1.3" fill="currentColor" fillOpacity="0.3" />
            <path d="M5 6V4.5a2 2 0 014 0V6" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          </svg>
        ) : (
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="3" y="6" width="8" height="6" rx="1" stroke="currentColor" strokeWidth="1.3" />
            <path d="M5 6V4.5a2 2 0 014 0V6" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          </svg>
        )}
      </button>
    </div>
  );
}
