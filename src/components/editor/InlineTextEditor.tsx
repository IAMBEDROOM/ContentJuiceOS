import { useEffect, useRef, useState } from 'react';
import type { TextElement } from '../../types/design';
import './InlineTextEditor.css';

interface InlineTextEditorProps {
  element: TextElement;
  zoom: number;
  panOffset: { x: number; y: number };
  onCommit: (text: string) => void;
  onCancel: () => void;
}

export default function InlineTextEditor({ element, zoom, panOffset, onCommit, onCancel }: InlineTextEditorProps) {
  const [value, setValue] = useState(element.text);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const cancelledRef = useRef(false);

  // Position: convert from Konva center-origin to top-left screen coords
  const screenX = (element.position.x - element.size.width / 2) * zoom + panOffset.x;
  const screenY = (element.position.y - element.size.height / 2) * zoom + panOffset.y;
  const screenWidth = element.size.width * zoom;
  const screenHeight = element.size.height * zoom;

  // Auto-focus and select all on mount
  useEffect(() => {
    const ta = textareaRef.current;
    if (ta) {
      ta.focus();
      ta.select();
    }
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    e.stopPropagation();

    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      onCommit(value);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      cancelledRef.current = true;
      onCancel();
    }
  };

  const handleBlur = () => {
    if (!cancelledRef.current) {
      onCommit(value);
    }
  };

  return (
    <textarea
      ref={textareaRef}
      className="inline-text-editor"
      value={value}
      onChange={(e) => setValue(e.target.value)}
      onKeyDown={handleKeyDown}
      onBlur={handleBlur}
      style={{
        left: screenX,
        top: screenY,
        width: screenWidth,
        height: screenHeight,
        fontFamily: element.fontFamily,
        fontSize: element.fontSize * zoom,
        fontWeight: element.fontWeight,
        color: element.color,
        textAlign: element.textAlign,
        lineHeight: element.lineHeight,
        transform: `rotate(${element.rotation}deg)`,
      }}
    />
  );
}
