import { Rect } from 'react-konva';

interface MarqueeRectProps {
  rect: { x: number; y: number; width: number; height: number };
}

export default function MarqueeRect({ rect }: MarqueeRectProps) {
  return (
    <Rect
      x={rect.x}
      y={rect.y}
      width={rect.width}
      height={rect.height}
      fill="rgba(0, 229, 255, 0.1)"
      stroke="#00E5FF"
      strokeWidth={1}
      dash={[4, 4]}
      listening={false}
    />
  );
}
