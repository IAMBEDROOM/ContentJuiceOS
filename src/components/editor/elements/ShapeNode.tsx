import { Rect, Circle, Ellipse, Line } from 'react-konva';
import type { ShapeElement } from '../../../types/design';

interface ShapeNodeProps {
  element: ShapeElement;
}

export default function ShapeNode({ element }: ShapeNodeProps) {
  if (!element.visible) return null;

  const common = {
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
    listening: false,
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
