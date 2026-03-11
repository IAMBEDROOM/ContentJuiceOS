import { useEffect, useState } from 'react';
import { Group, Image, Rect, Text } from 'react-konva';
import { convertFileSrc } from '@tauri-apps/api/core';
import type Konva from 'konva';
import type { AnimationElement } from '../../../types/design';
import { useEditor } from '../../../lib/editor/editorState';
import { computeDragEndValues, computeDragMoveSnap, computeTransformEndValues } from '../../../lib/editor/transformHandlers';

interface AnimationNodeProps {
  element: AnimationElement;
  isSelected: boolean;
  registerRef: (id: string, node: Konva.Node | null) => void;
  onSelect: (id: string, e: Konva.KonvaEventObject<MouseEvent>) => void;
}

export default function AnimationNode({ element, isSelected, registerRef, onSelect }: AnimationNodeProps) {
  const { state, dispatch } = useEditor();
  const [image, setImage] = useState<HTMLImageElement | null>(null);

  useEffect(() => {
    const img = new window.Image();
    img.crossOrigin = 'anonymous';
    img.onload = () => setImage(img);
    img.onerror = () => setImage(null);
    img.src = convertFileSrc(element.assetId);
  }, [element.assetId]);

  if (!element.visible) return null;

  return (
    <Group
      ref={(node) => registerRef(element.id, node)}
      x={element.position.x}
      y={element.position.y}
      width={element.size.width}
      height={element.size.height}
      rotation={element.rotation}
      offsetX={element.size.width / 2}
      offsetY={element.size.height / 2}
      opacity={element.opacity}
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
    >
      {image ? (
        <Image
          image={image}
          width={element.size.width}
          height={element.size.height}
        />
      ) : (
        <Rect
          width={element.size.width}
          height={element.size.height}
          fill="rgba(255,255,255,0.05)"
        />
      )}
      {/* Dashed border to indicate animation element */}
      <Rect
        width={element.size.width}
        height={element.size.height}
        stroke="#00E5FF"
        strokeWidth={1}
        dash={[6, 4]}
        fill="transparent"
      />
      <Text
        text="ANI"
        x={4}
        y={4}
        fontSize={10}
        fill="#00E5FF"
        opacity={0.7}
      />
    </Group>
  );
}
