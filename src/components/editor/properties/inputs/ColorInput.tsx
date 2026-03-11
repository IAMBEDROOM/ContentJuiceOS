import { useState, useRef, useEffect } from 'react';
import { HexColorPicker } from 'react-colorful';

interface ColorInputProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
}

export default function ColorInput({ label, value, onChange }: ColorInputProps) {
  const [open, setOpen] = useState(false);
  const [text, setText] = useState(value);
  const popoverRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setText(value);
  }, [value]);

  useEffect(() => {
    if (!open) return;
    function handleClick(e: MouseEvent) {
      if (popoverRef.current && !popoverRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, [open]);

  return (
    <label className="pp-field">
      <span className="pp-field-label">{label}</span>
      <div className="pp-color-row">
        <button
          type="button"
          className="pp-color-swatch"
          style={{ backgroundColor: value }}
          onClick={() => setOpen(!open)}
        />
        <input
          type="text"
          className="pp-input pp-input-hex"
          value={text}
          onChange={(e) => setText(e.target.value)}
          onBlur={() => {
            if (/^#[0-9a-fA-F]{3,8}$/.test(text)) {
              onChange(text);
            } else {
              setText(value);
            }
          }}
          onKeyDown={(e) => {
            if (e.key === 'Enter') (e.target as HTMLInputElement).blur();
          }}
        />
        {open && (
          <div className="pp-color-popover" ref={popoverRef}>
            <HexColorPicker color={value} onChange={onChange} />
          </div>
        )}
      </div>
    </label>
  );
}
