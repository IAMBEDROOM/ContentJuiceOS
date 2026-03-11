import { Text } from 'react-konva';
import type Konva from 'konva';
import type { TextElement } from '../../../types/design';
import { useEditor } from '../../../lib/editor/editorState';
import { computeDragEndValues, computeDragMoveSnap, computeTransformEndValues } from '../../../lib/editor/transformHandlers';

interface TextNodeProps {
  element: TextElement;
  isSelected: boolean;
  registerRef: (id: string, node: Konva.Node | null) => void;
  onSelect: (id: string, e: Konva.KonvaEventObject<MouseEvent>) => void;
}

export default function TextNode({ element, isSelected, registerRef, onSelect }: TextNodeProps) {
  const { state, dispatch } = useEditor();

  if (!element.visible) return null;

  return (
    <Text
      ref={(node) => registerRef(element.id, node)}
      x={element.position.x}
      y={element.position.y}
      width={element.size.width}
      height={element.size.height}
      rotation={element.rotation}
      offsetX={element.size.width / 2}
      offsetY={element.size.height / 2}
      opacity={element.opacity}
      text={element.text}
      fontFamily={element.fontFamily}
      fontSize={element.fontSize}
      fontStyle={element.fontWeight >= 700 ? 'bold' : 'normal'}
      fill={element.color}
      align={element.textAlign}
      lineHeight={element.lineHeight}
      stroke={element.stroke?.color}
      strokeWidth={element.stroke?.width}
      shadowColor={element.shadow?.color}
      shadowOffsetX={element.shadow?.offsetX}
      shadowOffsetY={element.shadow?.offsetY}
      shadowBlur={element.shadow?.blur}
      shadowEnabled={!!element.shadow}
      listening={!element.locked}
      draggable={isSelected && !element.locked}
      onMouseDown={(e) => onSelect(element.id, e)}
      onDragMove={(e) => computeDragMoveSnap(e.target, state.snapEnabled, state.gridSize)}
      onDragEnd={(e) => {
        const pos = computeDragEndValues(e.target, state.snapEnabled, state.gridSize);
        dispatch({ type: 'UPDATE_ELEMENT', id: element.id, changes: { position: pos } });
      }}
      onTransformEnd={(e) => {
        const values = computeTransformEndValues(e.target);
        dispatch({ type: 'UPDATE_ELEMENT', id: element.id, changes: values });
      }}
    />
  );
}
