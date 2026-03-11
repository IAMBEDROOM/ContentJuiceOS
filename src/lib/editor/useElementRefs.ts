import { useRef, useCallback } from 'react';
import type Konva from 'konva';

export interface ElementRefRegistry {
  refMap: React.MutableRefObject<Map<string, Konva.Node>>;
  registerRef: (id: string, node: Konva.Node | null) => void;
  getNodes: (ids: string[]) => Konva.Node[];
}

export function useElementRefs(): ElementRefRegistry {
  const refMap = useRef<Map<string, Konva.Node>>(new Map());

  const registerRef = useCallback((id: string, node: Konva.Node | null) => {
    if (node) {
      refMap.current.set(id, node);
    } else {
      refMap.current.delete(id);
    }
  }, []);

  const getNodes = useCallback((ids: string[]): Konva.Node[] => {
    const nodes: Konva.Node[] = [];
    for (const id of ids) {
      const node = refMap.current.get(id);
      if (node) nodes.push(node);
    }
    return nodes;
  }, []);

  return { refMap, registerRef, getNodes };
}
