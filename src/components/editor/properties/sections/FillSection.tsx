import type { ShapeElement } from '../../../../types/design';
import ColorInput from '../inputs/ColorInput';
import NumberInput from '../inputs/NumberInput';

interface FillSectionProps {
  element: ShapeElement;
  onPropertyChange: (changes: Record<string, unknown>) => void;
}

export default function FillSection({ element, onPropertyChange }: FillSectionProps) {
  return (
    <div className="pp-section">
      <div className="pp-section-header">Fill & Stroke</div>
      <ColorInput
        label="Fill"
        value={element.fillColor}
        onChange={(v) => onPropertyChange({ fillColor: v })}
      />
      <ColorInput
        label="Stroke"
        value={element.strokeColor ?? '#000000'}
        onChange={(v) => onPropertyChange({ strokeColor: v })}
      />
      <div className="pp-field-grid">
        <NumberInput
          label="Stroke W"
          value={element.strokeWidth}
          min={0}
          onChange={(v) => onPropertyChange({ strokeWidth: v })}
        />
        <NumberInput
          label="Radius"
          value={element.borderRadius}
          min={0}
          onChange={(v) => onPropertyChange({ borderRadius: v })}
        />
      </div>
    </div>
  );
}
