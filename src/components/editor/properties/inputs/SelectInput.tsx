interface SelectInputProps {
  label: string;
  value: string;
  options: { value: string; label: string }[];
  onChange: (value: string) => void;
  mixed?: boolean;
}

export default function SelectInput({
  label,
  value,
  options,
  onChange,
  mixed,
}: SelectInputProps) {
  return (
    <label className="pp-field">
      <span className="pp-field-label">{label}</span>
      <select
        className="pp-input pp-input-select"
        value={mixed ? '' : value}
        onChange={(e) => onChange(e.target.value)}
      >
        {mixed && (
          <option value="" disabled>
            Mixed
          </option>
        )}
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </label>
  );
}
