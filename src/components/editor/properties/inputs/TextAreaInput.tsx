import { useState, useEffect, useRef } from 'react';

interface TextAreaInputProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
}

export default function TextAreaInput({ label, value, onChange }: TextAreaInputProps) {
  const [local, setLocal] = useState(value);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    setLocal(value);
  }, [value]);

  function handleChange(newVal: string) {
    setLocal(newVal);
    if (timerRef.current !== null) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => onChange(newVal), 250);
  }

  return (
    <label className="pp-field">
      <span className="pp-field-label">{label}</span>
      <textarea
        className="pp-input pp-input-textarea"
        value={local}
        rows={3}
        onChange={(e) => handleChange(e.target.value)}
        onBlur={() => {
          if (timerRef.current !== null) clearTimeout(timerRef.current);
          if (local !== value) onChange(local);
        }}
      />
    </label>
  );
}
