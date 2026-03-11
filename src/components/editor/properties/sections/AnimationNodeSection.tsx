import type { AnimationElement } from '../../../../types/design';
import SelectInput from '../inputs/SelectInput';

interface AnimationNodeSectionProps {
  element: AnimationElement;
  onPropertyChange: (changes: Record<string, unknown>) => void;
}

const FIT_OPTIONS = [
  { value: 'contain', label: 'Contain' },
  { value: 'cover', label: 'Cover' },
  { value: 'fill', label: 'Fill' },
  { value: 'none', label: 'None' },
];

export default function AnimationNodeSection({
  element,
  onPropertyChange,
}: AnimationNodeSectionProps) {
  return (
    <div className="pp-section">
      <div className="pp-section-header">Animation</div>
      <SelectInput
        label="Fit Mode"
        value={element.fitMode}
        options={FIT_OPTIONS}
        onChange={(v) => onPropertyChange({ fitMode: v })}
      />
      <label className="pp-field pp-field-checkbox">
        <input
          type="checkbox"
          checked={element.playOnLoad}
          onChange={(e) => onPropertyChange({ playOnLoad: e.target.checked })}
        />
        <span>Play on Load</span>
      </label>
      <label className="pp-field pp-field-checkbox">
        <input
          type="checkbox"
          checked={element.loopAnimation}
          onChange={(e) => onPropertyChange({ loopAnimation: e.target.checked })}
        />
        <span>Loop</span>
      </label>
    </div>
  );
}
