import type { DesignElement } from '../../../../types/design';
import NumberInput from '../inputs/NumberInput';

interface TransformSectionProps {
  elements: DesignElement[];
  onPropertyChange: (changes: Record<string, unknown>) => void;
}

function allSame<T>(values: T[]): boolean {
  return values.every((v) => v === values[0]);
}

export default function TransformSection({ elements, onPropertyChange }: TransformSectionProps) {
  const xs = elements.map((e) => e.position.x);
  const ys = elements.map((e) => e.position.y);
  const ws = elements.map((e) => e.size.width);
  const hs = elements.map((e) => e.size.height);
  const rs = elements.map((e) => e.rotation);
  const os = elements.map((e) => e.opacity);

  const multi = elements.length > 1;

  return (
    <div className="pp-section">
      <div className="pp-section-header">Transform</div>
      <div className="pp-field-grid">
        <NumberInput
          label="X"
          value={xs[0]}
          mixed={multi && !allSame(xs)}
          onChange={(v) => onPropertyChange({ position: { ...elements[0].position, x: v } })}
        />
        <NumberInput
          label="Y"
          value={ys[0]}
          mixed={multi && !allSame(ys)}
          onChange={(v) => onPropertyChange({ position: { ...elements[0].position, y: v } })}
        />
        <NumberInput
          label="W"
          value={ws[0]}
          min={1}
          mixed={multi && !allSame(ws)}
          onChange={(v) => onPropertyChange({ size: { ...elements[0].size, width: v } })}
        />
        <NumberInput
          label="H"
          value={hs[0]}
          min={1}
          mixed={multi && !allSame(hs)}
          onChange={(v) => onPropertyChange({ size: { ...elements[0].size, height: v } })}
        />
        <NumberInput
          label="Rotation"
          value={rs[0]}
          step={1}
          mixed={multi && !allSame(rs)}
          onChange={(v) => onPropertyChange({ rotation: v })}
        />
        <NumberInput
          label="Opacity"
          value={Math.round(os[0] * 100)}
          min={0}
          max={100}
          step={1}
          mixed={multi && !allSame(os)}
          onChange={(v) => onPropertyChange({ opacity: v / 100 })}
        />
      </div>
    </div>
  );
}
