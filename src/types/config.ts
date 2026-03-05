import { z } from "zod";
import { PlatformSchema } from "./platform";
import { AlertTypeSchema } from "./events";

// ── General Settings ─────────────────────────────────────────────────

export const GeneralSettingsSchema = z.object({
  language: z.string().default("en"),
  launchOnStartup: z.boolean().default(false),
  minimizeToTray: z.boolean().default(true),
  checkForUpdates: z.boolean().default(true),
  mediaDirectory: z.string().default(""),
  backupIntervalHours: z.number().int().positive().default(24),
  maxBackups: z.number().int().positive().default(7),
});
export type GeneralSettings = z.infer<typeof GeneralSettingsSchema>;

// ── Appearance Settings ──────────────────────────────────────────────

export const AppearanceSettingsSchema = z.object({
  uiScale: z.number().min(0.5).max(2).default(1),
  showPlatformIcons: z.boolean().default(true),
});
export type AppearanceSettings = z.infer<typeof AppearanceSettingsSchema>;

// ── Server Settings ──────────────────────────────────────────────────

export const ServerSettingsSchema = z.object({
  httpPort: z.number().int().min(1024).max(65535).default(4848),
  socketIoPort: z.number().int().min(1024).max(65535).default(4849),
});
export type ServerSettings = z.infer<typeof ServerSettingsSchema>;

// ── OBS Settings ─────────────────────────────────────────────────────

export const ObsSettingsSchema = z.object({
  host: z.string().default("localhost"),
  port: z.number().int().min(1).max(65535).default(4455),
  password: z.string().default(""),
  autoConnect: z.boolean().default(false),
});
export type ObsSettings = z.infer<typeof ObsSettingsSchema>;

// ── Alert Configuration ──────────────────────────────────────────────

export const AlertConfigSchema = z.object({
  alertType: AlertTypeSchema,
  enabled: z.boolean().default(true),
  enabledPlatforms: z.array(PlatformSchema).default(["twitch", "youtube", "kick"]),
  minAmount: z.number().nonnegative().default(0),
  designId: z.string().uuid().optional(),
  duration: z.number().int().positive().default(5000),
  cooldown: z.number().int().nonnegative().default(0),
  ttsEnabled: z.boolean().default(false),
  variations: z.array(z.string().uuid()).default([]),
});
export type AlertConfig = z.infer<typeof AlertConfigSchema>;

// ── Alert Queue Settings ─────────────────────────────────────────────

export const AlertQueueModeSchema = z.enum(["sequential", "priority"]);
export type AlertQueueMode = z.infer<typeof AlertQueueModeSchema>;

export const AlertQueueSettingsSchema = z.object({
  mode: AlertQueueModeSchema.default("sequential"),
  delayBetween: z.number().int().nonnegative().default(1000),
  maxQueueLength: z.number().int().positive().default(50),
  staleThreshold: z.number().int().positive().default(300000),
});
export type AlertQueueSettings = z.infer<typeof AlertQueueSettingsSchema>;

// ── Bot Command ──────────────────────────────────────────────────────

export const PermissionLevelSchema = z.enum([
  "everyone",
  "subscriber",
  "vip",
  "moderator",
  "broadcaster",
]);
export type PermissionLevel = z.infer<typeof PermissionLevelSchema>;

export const BotCommandSchema = z.object({
  trigger: z.string(),
  response: z.string(),
  cooldown: z.number().int().nonnegative().default(10),
  permissionLevel: PermissionLevelSchema.default("everyone"),
  platforms: z.array(PlatformSchema).default(["twitch", "youtube", "kick"]),
});
export type BotCommand = z.infer<typeof BotCommandSchema>;

// ── Timed Message ────────────────────────────────────────────────────

export const TimedMessageSchema = z.object({
  message: z.string(),
  intervalMinutes: z.number().int().positive().default(15),
  minChatMessages: z.number().int().nonnegative().default(5),
});
export type TimedMessage = z.infer<typeof TimedMessageSchema>;

// ── Auto-Mod Settings ────────────────────────────────────────────────

export const AutoModSettingsSchema = z.object({
  linkFiltering: z.boolean().default(false),
  capsFilter: z.boolean().default(false),
  spamFilter: z.boolean().default(false),
  bannedWords: z.array(z.string()).default([]),
  enabledPlatforms: z.array(PlatformSchema).default(["twitch", "youtube", "kick"]),
});
export type AutoModSettings = z.infer<typeof AutoModSettingsSchema>;

// ── Cache TTL Settings ───────────────────────────────────────────────

export const CacheTtlSettingsSchema = z.object({
  channelInfo: z.number().int().positive().default(300),
  emotes: z.number().int().positive().default(3600),
  badges: z.number().int().positive().default(3600),
  categories: z.number().int().positive().default(600),
});
export type CacheTtlSettings = z.infer<typeof CacheTtlSettingsSchema>;

// ── App Config (Root) ────────────────────────────────────────────────

export const AppConfigSchema = z.object({
  general: GeneralSettingsSchema.default({
    language: "en",
    launchOnStartup: false,
    minimizeToTray: true,
    checkForUpdates: true,
    mediaDirectory: "",
    backupIntervalHours: 24,
    maxBackups: 7,
  }),
  appearance: AppearanceSettingsSchema.default({
    uiScale: 1,
    showPlatformIcons: true,
  }),
  server: ServerSettingsSchema.default({
    httpPort: 4848,
    socketIoPort: 4849,
  }),
  obs: ObsSettingsSchema.default({
    host: "localhost",
    port: 4455,
    password: "",
    autoConnect: false,
  }),
  alertQueue: AlertQueueSettingsSchema.default({
    mode: "sequential",
    delayBetween: 1000,
    maxQueueLength: 50,
    staleThreshold: 300000,
  }),
  cacheTtl: CacheTtlSettingsSchema.default({
    channelInfo: 300,
    emotes: 3600,
    badges: 3600,
    categories: 600,
  }),
});
export type AppConfig = z.infer<typeof AppConfigSchema>;
