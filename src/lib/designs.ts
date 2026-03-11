import { invoke } from '@tauri-apps/api/core';
import type { Design, DesignTree, DesignType } from '../types/design';

export interface DesignListResponse {
  designs: Design[];
  total: number;
}

/** Creates a new design with a generated UUID and default config. */
export async function createDesign(params: {
  name: string;
  designType: DesignType;
  config?: DesignTree;
  description?: string;
  tags?: string[];
}): Promise<Design> {
  return invoke<Design>('create_design', {
    name: params.name,
    designType: params.designType,
    config: params.config ?? null,
    description: params.description ?? null,
    tags: params.tags ?? null,
  });
}

/** Retrieves a single design by ID. */
export async function getDesign(id: string): Promise<Design> {
  return invoke<Design>('get_design', { id });
}

/** Lists designs with optional filtering, search, and pagination. */
export async function listDesigns(params?: {
  typeFilter?: DesignType;
  search?: string;
  limit?: number;
  offset?: number;
}): Promise<DesignListResponse> {
  return invoke<DesignListResponse>('list_designs', {
    typeFilter: params?.typeFilter ?? null,
    search: params?.search ?? null,
    limit: params?.limit ?? null,
    offset: params?.offset ?? null,
  });
}

/** Updates specific fields of a design. Pass null for thumbnail to clear it. */
export async function updateDesign(
  id: string,
  updates: {
    name?: string;
    config?: DesignTree;
    thumbnail?: string | null;
    tags?: string[];
    description?: string;
  },
): Promise<Design> {
  return invoke<Design>('update_design', {
    id,
    name: updates.name ?? null,
    config: updates.config ?? null,
    // null → clear thumbnail (sends ""), undefined → don't change (sends null)
    thumbnail: updates.thumbnail === null ? '' : (updates.thumbnail ?? null),
    tags: updates.tags ?? null,
    description: updates.description ?? null,
  });
}

/** Deletes a design by ID. */
export async function deleteDesign(id: string): Promise<void> {
  return invoke<void>('delete_design', { id });
}

/** Duplicates a design with a new ID and "(Copy)" name suffix. */
export async function duplicateDesign(id: string): Promise<Design> {
  return invoke<Design>('duplicate_design', { id });
}
