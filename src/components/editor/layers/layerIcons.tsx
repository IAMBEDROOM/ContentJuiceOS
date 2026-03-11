import type { DesignElement } from '../../../types/design';

interface LayerTypeIconProps {
  elementType: DesignElement['elementType'];
}

export default function LayerTypeIcon({ elementType }: LayerTypeIconProps) {
  const props = {
    width: 14,
    height: 14,
    viewBox: '0 0 14 14',
    fill: 'none',
    xmlns: 'http://www.w3.org/2000/svg',
    style: { flexShrink: 0 } as const,
  };
  const stroke = 'currentColor';

  switch (elementType) {
    case 'text':
      return (
        <svg {...props}>
          <path d="M3 3h8v2M7 3v8M5 11h4" stroke={stroke} strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      );
    case 'image':
      return (
        <svg {...props}>
          <rect x="1.5" y="2.5" width="11" height="9" rx="1" stroke={stroke} strokeWidth="1.3" />
          <circle cx="4.5" cy="5.5" r="1" fill={stroke} />
          <path d="M1.5 9.5l3-3 2 2 2.5-2.5 3.5 3.5" stroke={stroke} strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      );
    case 'shape':
      return (
        <svg {...props}>
          <rect x="2" y="2" width="10" height="10" rx="1.5" stroke={stroke} strokeWidth="1.3" />
        </svg>
      );
    case 'animation':
      return (
        <svg {...props}>
          <path d="M5 3.5v7l6-3.5-6-3.5z" stroke={stroke} strokeWidth="1.3" strokeLinejoin="round" />
        </svg>
      );
    case 'sound':
      return (
        <svg {...props}>
          <path d="M2 5.5v3h2l3 2.5V3L4 5.5H2z" stroke={stroke} strokeWidth="1.3" strokeLinejoin="round" />
          <path d="M9.5 4.5a3 3 0 010 5" stroke={stroke} strokeWidth="1.3" strokeLinecap="round" />
        </svg>
      );
    default:
      return null;
  }
}
