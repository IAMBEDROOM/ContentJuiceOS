import { z } from 'zod';

// ── Platform & Connection ────────────────────────────────────────────

export const PlatformSchema = z.enum(['twitch', 'youtube', 'kick']);
export type Platform = z.infer<typeof PlatformSchema>;

export const ConnectionStatusSchema = z.enum(['connected', 'disconnected', 'expired', 'revoked']);
export type ConnectionStatus = z.infer<typeof ConnectionStatusSchema>;

export const PlatformConnectionSchema = z.object({
  id: z.string().uuid(),
  platform: PlatformSchema,
  platformUserId: z.string(),
  platformUsername: z.string(),
  displayName: z.string(),
  avatarUrl: z.string().nullable(),
  scopes: z.string(),
  status: ConnectionStatusSchema,
  connectedAt: z.string().nullable(),
  lastRefreshedAt: z.string().nullable(),
  createdAt: z.string(),
  updatedAt: z.string(),
});
export type PlatformConnection = z.infer<typeof PlatformConnectionSchema>;

// ── Asset Types & Formats ────────────────────────────────────────────

export const AssetTypeSchema = z.enum(['image', 'audio', 'video', 'font', 'animation', 'caption']);
export type AssetType = z.infer<typeof AssetTypeSchema>;

export const ImageFormatSchema = z.enum(['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg']);
export type ImageFormat = z.infer<typeof ImageFormatSchema>;

export const AudioFormatSchema = z.enum(['mp3', 'wav', 'ogg', 'flac', 'aac']);
export type AudioFormat = z.infer<typeof AudioFormatSchema>;

export const VideoFormatSchema = z.enum(['mp4', 'webm', 'mov', 'mkv']);
export type VideoFormat = z.infer<typeof VideoFormatSchema>;

export const FontFormatSchema = z.enum(['ttf', 'otf', 'woff', 'woff2']);
export type FontFormat = z.infer<typeof FontFormatSchema>;

export const AnimationFormatSchema = z.enum(['json', 'gif', 'webp', 'webm']);
export type AnimationFormat = z.infer<typeof AnimationFormatSchema>;

export const CaptionFormatSchema = z.enum(['srt', 'vtt', 'ass', 'json']);
export type CaptionFormat = z.infer<typeof CaptionFormatSchema>;

export const AssetSchema = z.object({
  id: z.string().uuid(),
  originalFilename: z.string(),
  type: AssetTypeSchema,
  format: z.string(),
  fileSize: z.number().int().nonnegative(),
  dimensions: z
    .object({ width: z.number().int().positive(), height: z.number().int().positive() })
    .optional(),
  duration: z.number().nonnegative().optional(),
  tags: z.array(z.string()).default([]),
  filePath: z.string(),
  createdAt: z.string().datetime(),
});
export type Asset = z.infer<typeof AssetSchema>;

// ── Timestamps Mixin ─────────────────────────────────────────────────

export const TimestampsSchema = z.object({
  createdAt: z.string().datetime(),
  updatedAt: z.string().datetime(),
});
export type Timestamps = z.infer<typeof TimestampsSchema>;
