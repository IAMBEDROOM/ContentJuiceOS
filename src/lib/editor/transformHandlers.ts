import type Konva from 'konva';
import { snapToGrid } from './grid';

/**
 * After a Transformer resize/rotate, read the node's final transform values.
 * Konva applies scaleX/scaleY during resize — we bake those into width/height
 * and reset scale to 1 so the element's stored size stays authoritative.
 */
export function computeTransformEndValues(node: Konva.Node): {
  position: { x: number; y: number };
  size: { width: number; height: number };
  rotation: number;
} {
  const scaleX = node.scaleX();
  const scaleY = node.scaleY();
  const width = Math.max(5, node.width() * scaleX);
  const height = Math.max(5, node.height() * scaleY);

  // Reset scale so visual matches stored size
  node.scaleX(1);
  node.scaleY(1);
  node.width(width);
  node.height(height);

  // Re-center the origin
  node.offsetX(width / 2);
  node.offsetY(height / 2);

  return {
    position: { x: node.x(), y: node.y() },
    size: { width, height },
    rotation: node.rotation(),
  };
}

/** Read the node's position after a drag, optionally snapping to grid. */
export function computeDragEndValues(
  node: Konva.Node,
  snapEnabled: boolean,
  gridSize: number,
): { x: number; y: number } {
  let x = node.x();
  let y = node.y();

  if (snapEnabled) {
    x = snapToGrid(x, gridSize);
    y = snapToGrid(y, gridSize);
    node.x(x);
    node.y(y);
  }

  return { x, y };
}

/** Live snap during drag for visual feedback. */
export function computeDragMoveSnap(
  node: Konva.Node,
  snapEnabled: boolean,
  gridSize: number,
): void {
  if (!snapEnabled) return;

  node.x(snapToGrid(node.x(), gridSize));
  node.y(snapToGrid(node.y(), gridSize));
}
