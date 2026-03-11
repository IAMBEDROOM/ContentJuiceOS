import { Group, Rect, Text } from 'react-konva';
import type { SoundElement } from '../../../types/design';

interface SoundNodeProps {
  element: SoundElement;
}

export default function SoundNode({ element }: SoundNodeProps) {
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
      {/* Translucent background */}
      <Rect
        width={element.size.width}
        height={element.size.height}
        fill="rgba(255,0,127,0.08)"
        stroke="#FF007F"
        strokeWidth={1}
        dash={[6, 4]}
      />
      {/* Speaker icon placeholder */}
      <Text
        text={'\u{1F50A}'}
        x={0}
        y={0}
        width={element.size.width}
        height={element.size.height}
        align="center"
        verticalAlign="middle"
        fontSize={Math.min(element.size.width, element.size.height) * 0.4}
        opacity={0.5}
      />
    </Group>
  );
}
