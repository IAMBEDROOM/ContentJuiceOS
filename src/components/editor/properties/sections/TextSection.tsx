import type { TextElement } from '../../../../types/design';
import TextAreaInput from '../inputs/TextAreaInput';
import SelectInput from '../inputs/SelectInput';
import NumberInput from '../inputs/NumberInput';
import ColorInput from '../inputs/ColorInput';

interface TextSectionProps {
  element: TextElement;
  onPropertyChange: (changes: Record<string, unknown>) => void;
}

const FONT_OPTIONS = [
  { value: 'Inter', label: 'Inter' },
  { value: 'Arial', label: 'Arial' },
  { value: 'Helvetica', label: 'Helvetica' },
  { value: 'Georgia', label: 'Georgia' },
  { value: 'Times New Roman', label: 'Times New Roman' },
  { value: 'Courier New', label: 'Courier New' },
  { value: 'Verdana', label: 'Verdana' },
  { value: 'Trebuchet MS', label: 'Trebuchet MS' },
  { value: 'Impact', label: 'Impact' },
  { value: 'Comic Sans MS', label: 'Comic Sans MS' },
];

const WEIGHT_OPTIONS = [
  { value: '100', label: '100 - Thin' },
  { value: '200', label: '200 - Extra Light' },
  { value: '300', label: '300 - Light' },
  { value: '400', label: '400 - Regular' },
  { value: '500', label: '500 - Medium' },
  { value: '600', label: '600 - Semi Bold' },
  { value: '700', label: '700 - Bold' },
  { value: '800', label: '800 - Extra Bold' },
  { value: '900', label: '900 - Black' },
];

const ALIGN_OPTIONS: { value: string; label: string }[] = [
  { value: 'left', label: 'Left' },
  { value: 'center', label: 'Center' },
  { value: 'right', label: 'Right' },
];

export default function TextSection({ element, onPropertyChange }: TextSectionProps) {
  return (
    <div className="pp-section">
      <div className="pp-section-header">Text</div>
      <TextAreaInput
        label="Content"
        value={element.text}
        onChange={(v) => onPropertyChange({ text: v })}
      />
      <SelectInput
        label="Font"
        value={element.fontFamily}
        options={FONT_OPTIONS}
        onChange={(v) => onPropertyChange({ fontFamily: v })}
      />
      <div className="pp-field-grid">
        <NumberInput
          label="Size"
          value={element.fontSize}
          min={1}
          onChange={(v) => onPropertyChange({ fontSize: v })}
        />
        <SelectInput
          label="Weight"
          value={String(element.fontWeight)}
          options={WEIGHT_OPTIONS}
          onChange={(v) => onPropertyChange({ fontWeight: parseInt(v, 10) })}
        />
      </div>
      <ColorInput
        label="Color"
        value={element.color}
        onChange={(v) => onPropertyChange({ color: v })}
      />
      <SelectInput
        label="Align"
        value={element.textAlign}
        options={ALIGN_OPTIONS}
        onChange={(v) => onPropertyChange({ textAlign: v })}
      />
      <NumberInput
        label="Line Height"
        value={element.lineHeight}
        min={0.5}
        max={5}
        step={0.1}
        onChange={(v) => onPropertyChange({ lineHeight: v })}
      />
    </div>
  );
}
