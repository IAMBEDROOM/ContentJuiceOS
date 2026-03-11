import { useEffect, useState } from 'react';
import { Image } from 'react-konva';
import { convertFileSrc } from '@tauri-apps/api/core';
import type Konva from 'konva';
import type { ImageElement } from '../../../types/design';
import { useEditor } from '../../../lib/editor/editorState';
import { computeDragEndValues, computeDragMoveSnap, computeTransformEndValues } from '../../../lib/editor/transformHandlers';

interface ImageNodeProps {
  element: ImageElement;
  isSelected: boolean;
  registerRef: (id: string, node: Konva.Node | null) => void;
  onSelect: (id: string, e: Konva.KonvaEventObject<MouseEvent>) => void;
}

export default function ImageNode({ element, isSelected, registerRef, onSelect }: ImageNodeProps) {
  const { state, dispatch } = useEditor();
  const [image, setImage] = useState<HTMLImageElement | null>(null);

  useEffect(() => {
    const img = new window.Image();
    img.crossOrigin = 'anonymous';
    img.onload = () => setImage(img);
    img.onerror = () => setImage(null);
    img.src = convertFileSrc(element.assetId);
  }, [element.assetId]);

  if (!element.visible || !image) return null;

  return (
    <Image
      ref={(node) => registerRef(element.id, node)}
      image={image}
      x={element.position.x}
      y={element.position.y}
      width={element.size.width}
      height={element.size.height}
      rotation={element.rotation}
      offsetX={element.size.width / 2}
      offsetY={element.size.height / 2}
      opacity={element.opacity}
      cornerRadius={element.borderRadius}
      stroke={element.border?.color}
      strokeWidth={element.border?.width}
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
