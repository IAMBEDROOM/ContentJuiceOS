import type { TextElement } from '../../types/design';

export function createTextElement(
  canvasWidth: number,
  canvasHeight: number,
  nextLayerOrder: number,
): TextElement {
  return {
    id: crypto.randomUUID(),
    elementType: 'text',
    name: 'Text',
    position: { x: canvasWidth / 2, y: canvasHeight / 2 },
    size: { width: 300, height: 60 },
    rotation: 0,
    opacity: 1,
    visible: true,
    locked: false,
    layerOrder: nextLayerOrder,
    text: 'New Text',
    fontFamily: 'Inter',
    fontSize: 24,
    fontWeight: 400,
    color: '#FFFFFF',
    textAlign: 'left' as const,
    lineHeight: 1.4,
  };
}
