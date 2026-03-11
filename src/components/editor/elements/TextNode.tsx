import { Text } from 'react-konva';
import type { TextElement } from '../../../types/design';

interface TextNodeProps {
  element: TextElement;
}

export default function TextNode({ element }: TextNodeProps) {
  if (!element.visible) return null;

  return (
    <Text
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
      listening={false}
    />
  );
}
