import { useEffect, useState } from 'react';
import { Image } from 'react-konva';
import { convertFileSrc } from '@tauri-apps/api/core';
import type { ImageElement } from '../../../types/design';

interface ImageNodeProps {
  element: ImageElement;
}

export default function ImageNode({ element }: ImageNodeProps) {
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
      listening={false}
    />
  );
}
