import type { Border } from '../../../../types/design';
import ColorInput from '../inputs/ColorInput';
import NumberInput from '../inputs/NumberInput';

interface BorderSectionProps {
  /** Uses `border` for image elements, `stroke` for text elements */
  border: Border | undefined;
  onChange: (border: Border | undefined) => void;
}

export default function BorderSection({ border, onChange }: BorderSectionProps) {
  if (!border) {
    return (
      <div className="pp-section">
        <button
          type="button"
          className="pp-add-btn"
          onClick={() => onChange({ color: '#FFFFFF', width: 2 })}
        >
          + Add Border
        </button>
      </div>
    );
  }

  return (
    <div className="pp-section">
      <div className="pp-section-header">Border</div>
      <ColorInput
        label="Color"
        value={border.color}
        onChange={(v) => onChange({ ...border, color: v })}
      />
      <NumberInput
        label="Width"
        value={border.width}
        min={0}
        onChange={(v) => onChange({ ...border, width: v })}
      />
      <button type="button" className="pp-remove-btn" onClick={() => onChange(undefined)}>
        Remove Border
      </button>
    </div>
  );
}
