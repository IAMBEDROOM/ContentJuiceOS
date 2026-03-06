import { invoke } from '@tauri-apps/api/core';
import {
  GeneralSettingsSchema,
  AppearanceSettingsSchema,
  ServerSettingsSchema,
  ObsSettingsSchema,
  AlertQueueSettingsSchema,
  CacheTtlSettingsSchema,
  AppConfigSchema,
  type GeneralSettings,
  type AppearanceSettings,
  type ServerSettings,
  type ObsSettings,
  type AlertQueueSettings,
  type CacheTtlSettings,
  type AppConfig,
} from '../types/config';

type SectionName = 'general' | 'appearance' | 'server' | 'obs' | 'alertQueue' | 'cacheTtl';

type SectionTypeMap = {
  general: GeneralSettings;
  appearance: AppearanceSettings;
  server: ServerSettings;
  obs: ObsSettings;
  alertQueue: AlertQueueSettings;
  cacheTtl: CacheTtlSettings;
};

const sectionSchemas = {
  general: GeneralSettingsSchema,
  appearance: AppearanceSettingsSchema,
  server: ServerSettingsSchema,
  obs: ObsSettingsSchema,
  alertQueue: AlertQueueSettingsSchema,
  cacheTtl: CacheTtlSettingsSchema,
} as const;

export async function getConfigSection<S extends SectionName>(
  section: S,
): Promise<SectionTypeMap[S]> {
  const raw = await invoke('get_config_section', { section });
  return sectionSchemas[section].parse(raw) as SectionTypeMap[S];
}

export async function setConfigSection<S extends SectionName>(
  section: S,
  data: SectionTypeMap[S],
): Promise<void> {
  await invoke('set_config_section', { section, data });
}

export async function getFullConfig(): Promise<AppConfig> {
  const raw = await invoke('get_full_config');
  return AppConfigSchema.parse(raw);
}
