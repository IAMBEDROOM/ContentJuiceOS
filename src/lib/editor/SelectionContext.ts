import { createContext, useContext } from 'react';
import type Konva from 'konva';

export interface SelectionContextValue {
  selectedElementIds: string[];
  registerRef: (id: string, node: Konva.Node | null) => void;
  onElementMouseDown: (id: string, e: Konva.KonvaEventObject<MouseEvent>) => void;
}

export const SelectionContext = createContext<SelectionContextValue | null>(null);

export function useSelection(): SelectionContextValue {
  const ctx = useContext(SelectionContext);
  if (!ctx) {
    throw new Error('useSelection must be used within a SelectionContext.Provider');
  }
  return ctx;
}
