import type { DesignElement } from '../../../types/design';
import TextNode from './TextNode';
import ImageNode from './ImageNode';
import ShapeNode from './ShapeNode';
import AnimationNode from './AnimationNode';
import SoundNode from './SoundNode';

interface ElementRendererProps {
  element: DesignElement;
}

export default function ElementRenderer({ element }: ElementRendererProps) {
  switch (element.elementType) {
    case 'text':
      return <TextNode element={element} />;
    case 'image':
      return <ImageNode element={element} />;
    case 'shape':
      return <ShapeNode element={element} />;
    case 'animation':
      return <AnimationNode element={element} />;
    case 'sound':
      return <SoundNode element={element} />;
    default:
      return null;
  }
}
