interface NumberInputProps {
  label: string;
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
  step?: number;
  mixed?: boolean;
}

export default function NumberInput({
  label,
  value,
  onChange,
  min,
  max,
  step = 1,
  mixed,
}: NumberInputProps) {
  return (
    <label className="pp-field">
      <span className="pp-field-label">{label}</span>
      <input
        type="number"
        className="pp-input pp-input-number"
        value={mixed ? '' : value}
        placeholder={mixed ? 'Mixed' : undefined}
        min={min}
        max={max}
        step={step}
        onChange={(e) => {
          const v = parseFloat(e.target.value);
          if (!Number.isNaN(v)) onChange(v);
        }}
      />
    </label>
  );
}
