import { useState, useCallback } from 'react';
import type Konva from 'konva';
import type { DesignElement } from '../../types/design';
import { screenToCanvas } from './viewport';

const DRAG_THRESHOLD = 3;

interface MarqueeState {
  active: boolean;
  start: { x: number; y: number };
  current: { x: number; y: number };
}

export interface UseMarqueeProps {
  zoom: number;
  panOffset: { x: number; y: number };
  elements: DesignElement[];
  onSelect: (ids: string[], additive: boolean) => void;
  onMarqueeActiveChange: (active: boolean) => void;
}

export function useMarquee({ zoom, panOffset, elements, onSelect, onMarqueeActiveChange }: UseMarqueeProps) {
  const [marquee, setMarquee] = useState<MarqueeState | null>(null);

  const handleStageMouseDown = useCallback(
    (e: Konva.KonvaEventObject<MouseEvent>) => {
      if (e.evt.button !== 0) return;

      const pos = { x: e.evt.clientX, y: e.evt.clientY };
      setMarquee({ active: false, start: pos, current: pos });
    },
    [],
  );

  const handleStageMouseMove = useCallback(
    (e: Konva.KonvaEventObject<MouseEvent>) => {
      if (!marquee) return;

      const current = { x: e.evt.clientX, y: e.evt.clientY };
      const dx = current.x - marquee.start.x;
      const dy = current.y - marquee.start.y;

      if (!marquee.active && Math.sqrt(dx * dx + dy * dy) > DRAG_THRESHOLD) {
        setMarquee({ ...marquee, current, active: true });
        onMarqueeActiveChange(true);
      } else if (marquee.active) {
        setMarquee({ ...marquee, current });
      }
    },
    [marquee, onMarqueeActiveChange],
  );

  const handleStageMouseUp = useCallback(
    (e: Konva.KonvaEventObject<MouseEvent>) => {
      if (!marquee) return;

      if (marquee.active) {
        const p1 = screenToCanvas(marquee.start.x, marquee.start.y, zoom, panOffset);
        const p2 = screenToCanvas(marquee.current.x, marquee.current.y, zoom, panOffset);

        const minX = Math.min(p1.x, p2.x);
        const minY = Math.min(p1.y, p2.y);
        const maxX = Math.max(p1.x, p2.x);
        const maxY = Math.max(p1.y, p2.y);

        const hitIds = elements
          .filter((el) => {
            if (el.locked || !el.visible || el.elementType === 'sound') return false;
            const halfW = el.size.width / 2;
            const halfH = el.size.height / 2;
            const elMinX = el.position.x - halfW;
            const elMinY = el.position.y - halfH;
            const elMaxX = el.position.x + halfW;
            const elMaxY = el.position.y + halfH;
            return elMinX < maxX && elMaxX > minX && elMinY < maxY && elMaxY > minY;
          })
          .map((el) => el.id);

        onSelect(hitIds, e.evt.shiftKey);
        onMarqueeActiveChange(false);
      }

      setMarquee(null);
    },
    [marquee, zoom, panOffset, elements, onSelect, onMarqueeActiveChange],
  );

  let marqueeRect: { x: number; y: number; width: number; height: number } | null = null;
  if (marquee?.active) {
    const p1 = screenToCanvas(marquee.start.x, marquee.start.y, zoom, panOffset);
    const p2 = screenToCanvas(marquee.current.x, marquee.current.y, zoom, panOffset);
    marqueeRect = {
      x: Math.min(p1.x, p2.x),
      y: Math.min(p1.y, p2.y),
      width: Math.abs(p2.x - p1.x),
      height: Math.abs(p2.y - p1.y),
    };
  }

  return {
    marqueeRect,
    isMarqueeActive: marquee?.active ?? false,
    handleStageMouseDown,
    handleStageMouseMove,
    handleStageMouseUp,
  };
}
