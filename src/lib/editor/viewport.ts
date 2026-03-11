const PADDING_FACTOR = 0.9;

/** Calculate zoom level to fit canvas within container with padding. */
export function calculateFitZoom(
  canvasW: number,
  canvasH: number,
  containerW: number,
  containerH: number,
): number {
  if (canvasW <= 0 || canvasH <= 0 || containerW <= 0 || containerH <= 0) return 1;
  const scaleX = containerW / canvasW;
  const scaleY = containerH / canvasH;
  return Math.min(scaleX, scaleY) * PADDING_FACTOR;
}

/** Calculate offset to center canvas within container at given zoom. */
export function calculateCenterOffset(
  canvasW: number,
  canvasH: number,
  containerW: number,
  containerH: number,
  zoom: number,
): { x: number; y: number } {
  return {
    x: (containerW - canvasW * zoom) / 2,
    y: (containerH - canvasH * zoom) / 2,
  };
}

/** Adjust panOffset so the point under the cursor stays fixed during zoom. */
export function zoomAtPoint(
  oldZoom: number,
  newZoom: number,
  pointerX: number,
  pointerY: number,
  panOffset: { x: number; y: number },
): { x: number; y: number } {
  // The canvas point under the cursor: (pointerX - panX) / oldZoom
  // After zoom, it should still be at pointerX: newPanX = pointerX - canvasPoint * newZoom
  const canvasX = (pointerX - panOffset.x) / oldZoom;
  const canvasY = (pointerY - panOffset.y) / oldZoom;
  return {
    x: pointerX - canvasX * newZoom,
    y: pointerY - canvasY * newZoom,
  };
}

/** Convert screen coordinates to canvas coordinates. */
export function screenToCanvas(
  screenX: number,
  screenY: number,
  zoom: number,
  panOffset: { x: number; y: number },
): { x: number; y: number } {
  return {
    x: (screenX - panOffset.x) / zoom,
    y: (screenY - panOffset.y) / zoom,
  };
}

/** Convert canvas coordinates to screen coordinates. */
export function canvasToScreen(
  canvasX: number,
  canvasY: number,
  zoom: number,
  panOffset: { x: number; y: number },
): { x: number; y: number } {
  return {
    x: canvasX * zoom + panOffset.x,
    y: canvasY * zoom + panOffset.y,
  };
}
