import type { ImageElement } from '../../../../types/design';
import SelectInput from '../inputs/SelectInput';
import NumberInput from '../inputs/NumberInput';

interface ImageSectionProps {
  element: ImageElement;
  onPropertyChange: (changes: Record<string, unknown>) => void;
}

const FIT_OPTIONS = [
  { value: 'contain', label: 'Contain' },
  { value: 'cover', label: 'Cover' },
  { value: 'fill', label: 'Fill' },
  { value: 'none', label: 'None' },
];

export default function ImageSection({ element, onPropertyChange }: ImageSectionProps) {
  return (
    <div className="pp-section">
      <div className="pp-section-header">Image</div>
      <SelectInput
        label="Fit Mode"
        value={element.fitMode}
        options={FIT_OPTIONS}
        onChange={(v) => onPropertyChange({ fitMode: v })}
      />
      <NumberInput
        label="Border Radius"
        value={element.borderRadius}
        min={0}
        onChange={(v) => onPropertyChange({ borderRadius: v })}
      />
    </div>
  );
}
