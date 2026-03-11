import { useMemo } from 'react';
import { Layer, Line } from 'react-konva';
import { generateGridLines } from '../../lib/editor/grid';
import { useEditor } from '../../lib/editor/editorState';

export default function EditorGrid() {
  const { state } = useEditor();
  const { designTree, gridSize, gridVisible, zoom, panOffset } = state;

  const lines = useMemo(
    () => generateGridLines(designTree.canvas.width, designTree.canvas.height, gridSize),
    [designTree.canvas.width, designTree.canvas.height, gridSize],
  );

  if (!gridVisible) return null;

  return (
    <Layer listening={false} scaleX={zoom} scaleY={zoom} x={panOffset.x} y={panOffset.y}>
      {lines.map((line) => (
        <Line
          key={line.key}
          points={line.points}
          stroke="rgba(255,255,255,0.06)"
          strokeWidth={1 / zoom}
        />
      ))}
    </Layer>
  );
}
