import type { Shadow } from '../../../../types/design';
import ColorInput from '../inputs/ColorInput';
import NumberInput from '../inputs/NumberInput';

interface ShadowSectionProps {
  shadow: Shadow | undefined;
  onChange: (shadow: Shadow | undefined) => void;
}

export default function ShadowSection({ shadow, onChange }: ShadowSectionProps) {
  if (!shadow) {
    return (
      <div className="pp-section">
        <button
          type="button"
          className="pp-add-btn"
          onClick={() => onChange({ color: 'rgba(0,0,0,0.5)', offsetX: 2, offsetY: 2, blur: 4 })}
        >
          + Add Shadow
        </button>
      </div>
    );
  }

  return (
    <div className="pp-section">
      <div className="pp-section-header">Shadow</div>
      <ColorInput
        label="Color"
        value={shadow.color}
        onChange={(v) => onChange({ ...shadow, color: v })}
      />
      <div className="pp-field-grid">
        <NumberInput
          label="Offset X"
          value={shadow.offsetX}
          onChange={(v) => onChange({ ...shadow, offsetX: v })}
        />
        <NumberInput
          label="Offset Y"
          value={shadow.offsetY}
          onChange={(v) => onChange({ ...shadow, offsetY: v })}
        />
        <NumberInput
          label="Blur"
          value={shadow.blur}
          min={0}
          onChange={(v) => onChange({ ...shadow, blur: v })}
        />
      </div>
      <button type="button" className="pp-remove-btn" onClick={() => onChange(undefined)}>
        Remove Shadow
      </button>
    </div>
  );
}
