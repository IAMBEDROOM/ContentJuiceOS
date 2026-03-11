import { Rect, Circle, Ellipse, Line } from 'react-konva';
import type Konva from 'konva';
import type { ShapeElement } from '../../../types/design';
import { useEditor } from '../../../lib/editor/editorState';
import { computeDragEndValues, computeDragMoveSnap, computeTransformEndValues } from '../../../lib/editor/transformHandlers';

interface ShapeNodeProps {
  element: ShapeElement;
  isSelected: boolean;
  registerRef: (id: string, node: Konva.Node | null) => void;
  onSelect: (id: string, e: Konva.KonvaEventObject<MouseEvent>) => void;
}

export default function ShapeNode({ element, isSelected, registerRef, onSelect }: ShapeNodeProps) {
  const { state, dispatch } = useEditor();

  if (!element.visible) return null;

  const common = {
    ref: ((node: Konva.Node | null) => registerRef(element.id, node)) as React.LegacyRef<never>,
    x: element.position.x,
    y: element.position.y,
    rotation: element.rotation,
    opacity: element.opacity,
    fill: element.fillColor,
    stroke: element.strokeColor,
    strokeWidth: element.strokeWidth,
    shadowColor: element.shadow?.color,
    shadowOffsetX: element.shadow?.offsetX,
    shadowOffsetY: element.shadow?.offsetY,
    shadowBlur: element.shadow?.blur,
    shadowEnabled: !!element.shadow,
    listening: !element.locked,
    draggable: isSelected && !element.locked,
    onMouseDown: (e: Konva.KonvaEventObject<MouseEvent>) => onSelect(element.id, e),
    onDragMove: (e: Konva.KonvaEventObject<MouseEvent>) =>
      computeDragMoveSnap(e.target, state.snapEnabled, state.gridSize),
    onDragEnd: (e: Konva.KonvaEventObject<MouseEvent>) => {
      const pos = computeDragEndValues(e.target, state.snapEnabled, state.gridSize);
      dispatch({ type: 'UPDATE_ELEMENT', id: element.id, changes: { position: pos } });
    },
    onTransformEnd: (e: Konva.KonvaEventObject<Event>) => {
      const values = computeTransformEndValues(e.target);
      dispatch({ type: 'UPDATE_ELEMENT', id: element.id, changes: values });
    },
  };

  switch (element.shapeType) {
    case 'rectangle':
      return (
        <Rect
          {...common}
          width={element.size.width}
          height={element.size.height}
          offsetX={element.size.width / 2}
          offsetY={element.size.height / 2}
        />
      );
    case 'rounded_rectangle':
      return (
        <Rect
          {...common}
          width={element.size.width}
          height={element.size.height}
          offsetX={element.size.width / 2}
          offsetY={element.size.height / 2}
          cornerRadius={element.borderRadius}
        />
      );
    case 'circle':
      return (
        <Circle
          {...common}
          radius={Math.min(element.size.width, element.size.height) / 2}
        />
      );
    case 'ellipse':
      return (
        <Ellipse
          {...common}
          radiusX={element.size.width / 2}
          radiusY={element.size.height / 2}
        />
      );
    case 'line':
      return (
        <Line
          {...common}
          points={[0, 0, element.size.width, element.size.height]}
          offsetX={element.size.width / 2}
          offsetY={element.size.height / 2}
        />
      );
    default:
      return null;
  }
}
