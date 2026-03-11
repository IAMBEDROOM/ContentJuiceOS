import { useCallback } from 'react';
import { useEditor } from '../../../lib/editor/editorState';
import type {
  DesignElement,
  TextElement,
  ImageElement,
  ShapeElement,
  AnimationElement,
  Border,
  Shadow,
} from '../../../types/design';
import TransformSection from './sections/TransformSection';
import TextSection from './sections/TextSection';
import FillSection from './sections/FillSection';
import ImageSection from './sections/ImageSection';
import AnimationNodeSection from './sections/AnimationNodeSection';
import BorderSection from './sections/BorderSection';
import ShadowSection from './sections/ShadowSection';
import './PropertiesPanel.css';

export default function PropertiesPanel() {
  const { state, dispatch } = useEditor();
  const { selectedElementIds, designTree } = state;

  const selectedElements = selectedElementIds
    .map((id) => designTree.elements.find((el) => el.id === id))
    .filter((el): el is DesignElement => el !== undefined);

  const updateProperty = useCallback(
    (id: string, changes: Record<string, unknown>) => {
      dispatch({ type: 'UPDATE_ELEMENT_PROPERTIES', id, changes });
    },
    [dispatch],
  );

  const handlePropertyChange = useCallback(
    (changes: Record<string, unknown>) => {
      for (const el of selectedElements) {
        updateProperty(el.id, changes);
      }
    },
    [selectedElements, updateProperty],
  );

  // No selection
  if (selectedElements.length === 0) {
    return (
      <div className="properties-panel">
        <div className="pp-empty">No selection</div>
      </div>
    );
  }

  const first = selectedElements[0];
  const multi = selectedElements.length > 1;
  const allSameType = selectedElements.every((el) => el.elementType === first.elementType);

  // Helper to make border change handler for text (uses `stroke`) or image (uses `border`)
  function makeBorderHandler(el: DesignElement) {
    return (border: Border | undefined) => {
      const key = el.elementType === 'text' ? 'stroke' : 'border';
      updateProperty(el.id, { [key]: border });
    };
  }

  function getBorder(el: DesignElement): Border | undefined {
    if (el.elementType === 'text') return (el as TextElement).stroke;
    if (el.elementType === 'image') return (el as ImageElement).border;
    return undefined;
  }

  function getShadow(el: DesignElement): Shadow | undefined {
    if ('shadow' in el) return el.shadow as Shadow | undefined;
    return undefined;
  }

  return (
    <div className="properties-panel">
      {/* Header */}
      <div className="pp-header">
        {multi ? (
          <span className="pp-header-title">{selectedElements.length} elements selected</span>
        ) : (
          <input
            className="pp-name-input"
            value={first.name}
            onChange={(e) => updateProperty(first.id, { name: e.target.value })}
          />
        )}
      </div>

      {/* Transform — always shown */}
      <TransformSection elements={selectedElements} onPropertyChange={handlePropertyChange} />

      {/* Type-specific sections — only if single selection or all same type */}
      {(!multi || allSameType) && first.elementType === 'text' && (
        <TextSection
          element={first as TextElement}
          onPropertyChange={(changes) => updateProperty(first.id, changes)}
        />
      )}

      {(!multi || allSameType) && first.elementType === 'image' && (
        <ImageSection
          element={first as ImageElement}
          onPropertyChange={(changes) => updateProperty(first.id, changes)}
        />
      )}

      {(!multi || allSameType) && first.elementType === 'shape' && (
        <FillSection
          element={first as ShapeElement}
          onPropertyChange={(changes) => updateProperty(first.id, changes)}
        />
      )}

      {(!multi || allSameType) && first.elementType === 'animation' && (
        <AnimationNodeSection
          element={first as AnimationElement}
          onPropertyChange={(changes) => updateProperty(first.id, changes)}
        />
      )}

      {/* Border — text and image */}
      {!multi &&
        (first.elementType === 'text' || first.elementType === 'image') && (
          <BorderSection border={getBorder(first)} onChange={makeBorderHandler(first)} />
        )}

      {/* Shadow — text, image, shape */}
      {!multi &&
        (first.elementType === 'text' ||
          first.elementType === 'image' ||
          first.elementType === 'shape') && (
          <ShadowSection
            shadow={getShadow(first)}
            onChange={(shadow) => updateProperty(first.id, { shadow })}
          />
        )}
    </div>
  );
}
