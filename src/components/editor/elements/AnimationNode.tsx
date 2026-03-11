import { useEffect, useState } from 'react';
import { Group, Image, Rect, Text } from 'react-konva';
import { convertFileSrc } from '@tauri-apps/api/core';
import type { AnimationElement } from '../../../types/design';

interface AnimationNodeProps {
  element: AnimationElement;
}

export default function AnimationNode({ element }: AnimationNodeProps) {
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
      x={element.position.x}
      y={element.position.y}
      rotation={element.rotation}
      offsetX={element.size.width / 2}
      offsetY={element.size.height / 2}
      opacity={element.opacity}
      listening={false}
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
