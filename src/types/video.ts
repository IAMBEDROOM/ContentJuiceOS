import { z } from 'zod';

// ── Export Settings ──────────────────────────────────────────────────

export const AspectRatioSchema = z.enum(['16:9', '9:16', '1:1', '4:5']);
export type AspectRatio = z.infer<typeof AspectRatioSchema>;

export const VideoCodecSchema = z.enum(['h264', 'h265']);
export type VideoCodec = z.infer<typeof VideoCodecSchema>;

export const ExportResolutionSchema = z.object({
  width: z.number().int().positive(),
  height: z.number().int().positive(),
});
export type ExportResolution = z.infer<typeof ExportResolutionSchema>;

export const ExportSettingsSchema = z.object({
  resolution: ExportResolutionSchema,
  codec: VideoCodecSchema.default('h264'),
  bitrate: z.number().int().positive().default(8_000_000),
  fps: z.number().int().positive().default(30),
  format: z.enum(['mp4', 'webm', 'mov']).default('mp4'),
});
export type ExportSettings = z.infer<typeof ExportSettingsSchema>;

export const ExportPresetSchema = z.object({
  id: z.string().uuid(),
  label: z.string(),
  aspectRatio: AspectRatioSchema,
  settings: ExportSettingsSchema,
});
export type ExportPreset = z.infer<typeof ExportPresetSchema>;

// ── Timeline ─────────────────────────────────────────────────────────

export const TrimPointSchema = z.object({
  startTime: z.number().nonnegative(),
  endTime: z.number().nonnegative(),
});
export type TrimPoint = z.infer<typeof TrimPointSchema>;

export const AudioTrackSchema = z.object({
  id: z.string().uuid(),
  assetId: z.string().uuid(),
  isOriginalAudio: z.boolean().default(false),
  volume: z.number().min(0).max(1).default(1),
  fadeIn: z.number().nonnegative().default(0),
  fadeOut: z.number().nonnegative().default(0),
  muted: z.boolean().default(false),
});
export type AudioTrack = z.infer<typeof AudioTrackSchema>;

export const CropRegionSchema = z.object({
  x: z.number().nonnegative(),
  y: z.number().nonnegative(),
  width: z.number().positive(),
  height: z.number().positive(),
});
export type CropRegion = z.infer<typeof CropRegionSchema>;

// ── Captions ─────────────────────────────────────────────────────────

export const CaptionWordSchema = z.object({
  word: z.string(),
  startTime: z.number().nonnegative(),
  endTime: z.number().nonnegative(),
  confidence: z.number().min(0).max(1).optional(),
});
export type CaptionWord = z.infer<typeof CaptionWordSchema>;

export const CaptionSegmentSchema = z.object({
  id: z.string(),
  startTime: z.number().nonnegative(),
  endTime: z.number().nonnegative(),
  text: z.string(),
  words: z.array(CaptionWordSchema).default([]),
});
export type CaptionSegment = z.infer<typeof CaptionSegmentSchema>;

export const CaptionStyleSchema = z.object({
  fontFamily: z.string().default('Inter'),
  fontSize: z.number().positive().default(48),
  color: z.string().default('#FFFFFF'),
  outline: z.object({ color: z.string(), width: z.number().nonnegative() }).optional(),
  shadow: z
    .object({
      color: z.string(),
      offsetX: z.number(),
      offsetY: z.number(),
      blur: z.number().nonnegative(),
    })
    .optional(),
  background: z.object({ color: z.string(), padding: z.number().nonnegative() }).optional(),
  position: z.enum(['top', 'center', 'bottom']).default('bottom'),
  alignment: z.enum(['left', 'center', 'right']).default('center'),
});
export type CaptionStyle = z.infer<typeof CaptionStyleSchema>;

export const CaptionTrackSchema = z.object({
  segments: z.array(CaptionSegmentSchema).default([]),
  style: CaptionStyleSchema.default({
    fontFamily: 'Inter',
    fontSize: 48,
    color: '#FFFFFF',
    position: 'bottom',
    alignment: 'center',
  }),
});
export type CaptionTrack = z.infer<typeof CaptionTrackSchema>;

// ── Video Project ────────────────────────────────────────────────────

export const ProjectStatusSchema = z.enum(['draft', 'exported']);
export type ProjectStatus = z.infer<typeof ProjectStatusSchema>;

export const VideoProjectConfigSchema = z.object({
  sourceVideoAssetId: z.string().uuid(),
  aspectRatio: AspectRatioSchema.default('16:9'),
  cropRegion: CropRegionSchema.optional(),
  trimPoints: z.array(TrimPointSchema).default([]),
  audioTracks: z.array(AudioTrackSchema).default([]),
  captionTrack: CaptionTrackSchema.optional(),
  exportSettings: ExportSettingsSchema.optional(),
});
export type VideoProjectConfig = z.infer<typeof VideoProjectConfigSchema>;

export const VideoProjectSchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  sourceVideoAssetId: z.string().uuid(),
  config: VideoProjectConfigSchema,
  thumbnail: z.string().optional(),
  status: ProjectStatusSchema.default('draft'),
  createdAt: z.string().datetime(),
  updatedAt: z.string().datetime(),
});
export type VideoProject = z.infer<typeof VideoProjectSchema>;
