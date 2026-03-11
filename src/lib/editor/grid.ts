export interface GridLine {
  points: number[];
  key: string;
}

/** Snap a value to the nearest grid line. */
export function snapToGrid(value: number, gridSize: number): number {
  return Math.round(value / gridSize) * gridSize;
}

/** Generate horizontal and vertical grid lines for the canvas. */
export function generateGridLines(canvasW: number, canvasH: number, gridSize: number): GridLine[] {
  const lines: GridLine[] = [];

  // Vertical lines
  for (let x = 0; x <= canvasW; x += gridSize) {
    lines.push({ points: [x, 0, x, canvasH], key: `v-${x}` });
  }

  // Horizontal lines
  for (let y = 0; y <= canvasH; y += gridSize) {
    lines.push({ points: [0, y, canvasW, y], key: `h-${y}` });
  }

  return lines;
}
