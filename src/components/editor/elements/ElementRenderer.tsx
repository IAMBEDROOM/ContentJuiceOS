import type { DesignElement } from '../../../types/design';
import { useSelection } from '../../../lib/editor/SelectionContext';
import TextNode from './TextNode';
import ImageNode from './ImageNode';
import ShapeNode from './ShapeNode';
import AnimationNode from './AnimationNode';
import SoundNode from './SoundNode';

interface ElementRendererProps {
  element: DesignElement;
}

export default function ElementRenderer({ element }: ElementRendererProps) {
  const { selectedElementIds, registerRef, onElementMouseDown, onElementDblClick, editingTextId } = useSelection();
  const isSelected = selectedElementIds.includes(element.id);

  // Sound elements stay non-interactive on canvas
  if (element.elementType === 'sound') {
    return <SoundNode element={element} />;
  }

  const interactionProps = {
    isSelected,
    registerRef,
    onSelect: onElementMouseDown,
  };

  switch (element.elementType) {
    case 'text':
      return <TextNode element={element} {...interactionProps} onDblClick={onElementDblClick} editingTextId={editingTextId} />;
    case 'image':
      return <ImageNode element={element} {...interactionProps} />;
    case 'shape':
      return <ShapeNode element={element} {...interactionProps} />;
    case 'animation':
      return <AnimationNode element={element} {...interactionProps} />;
    default:
      return null;
  }
}
