import { useEffect, useRef } from 'react';
import { Transformer } from 'react-konva';
import type Konva from 'konva';
import { useSelection } from '../../lib/editor/SelectionContext';
import type { ElementRefRegistry } from '../../lib/editor/useElementRefs';

const ROTATION_SNAPS = [0, 45, 90, 135, 180, 225, 270, 315];
const ROTATION_SNAP_TOLERANCE = 5;
const MIN_SIZE = 5;

interface SelectionTransformerProps {
  getNodes: ElementRefRegistry['getNodes'];
}

export default function SelectionTransformer({ getNodes }: SelectionTransformerProps) {
  const { selectedElementIds } = useSelection();
  const transformerRef = useRef<Konva.Transformer>(null);

  useEffect(() => {
    const tr = transformerRef.current;
    if (!tr) return;

    const nodes = getNodes(selectedElementIds);
    tr.nodes(nodes);
    tr.getLayer()?.batchDraw();
  }, [selectedElementIds, getNodes]);

  if (selectedElementIds.length === 0) return null;

  return (
    <Transformer
      ref={transformerRef}
      borderStroke="#00E5FF"
      borderStrokeWidth={1.5}
      anchorStroke="#00E5FF"
      anchorFill="#151A26"
      anchorSize={8}
      anchorCornerRadius={2}
      rotateAnchorOffset={25}
      rotationSnaps={ROTATION_SNAPS}
      rotationSnapTolerance={ROTATION_SNAP_TOLERANCE}
      keepRatio={false}
      enabledAnchors={[
        'top-left',
        'top-center',
        'top-right',
        'middle-left',
        'middle-right',
        'bottom-left',
        'bottom-center',
        'bottom-right',
      ]}
      boundBoxFunc={(_oldBox, newBox) => {
        // Enforce minimum size
        if (Math.abs(newBox.width) < MIN_SIZE || Math.abs(newBox.height) < MIN_SIZE) {
          return _oldBox;
        }
        return newBox;
      }}
      onTransform={(e) => {
        // Lock aspect ratio when Shift is held
        const tr = transformerRef.current;
        if (tr) {
          tr.keepRatio((e.evt as MouseEvent).shiftKey);
        }
      }}
    />
  );
}
