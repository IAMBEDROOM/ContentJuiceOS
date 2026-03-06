import { z } from 'zod';
import { PlatformSchema } from './platform';

// ── Alert Types ──────────────────────────────────────────────────────

export const AlertTypeSchema = z.enum([
  'follow',
  'subscription',
  'gift_sub',
  'raid',
  'bits',
  'super_chat',
  'super_sticker',
  'donation',
]);
export type AlertType = z.infer<typeof AlertTypeSchema>;

export const SubTierSchema = z.enum(['prime', 'tier1', 'tier2', 'tier3']);
export type SubTier = z.infer<typeof SubTierSchema>;

export const AlertEventSchema = z.object({
  id: z.string().uuid(),
  type: AlertTypeSchema,
  platform: PlatformSchema,
  username: z.string(),
  displayName: z.string(),
  message: z.string().optional(),
  amount: z.number().nonnegative().optional(),
  currency: z.string().optional(),
  subTier: SubTierSchema.optional(),
  giftCount: z.number().int().nonnegative().optional(),
  raidViewerCount: z.number().int().nonnegative().optional(),
  isAnonymous: z.boolean().default(false),
  timestamp: z.string().datetime(),
});
export type AlertEvent = z.infer<typeof AlertEventSchema>;

export const AlertVariableSchema = z.enum([
  'username',
  'displayName',
  'amount',
  'currency',
  'message',
  'subTier',
  'giftCount',
  'raidViewerCount',
  'platform',
]);
export type AlertVariable = z.infer<typeof AlertVariableSchema>;

// ── Chat ─────────────────────────────────────────────────────────────

export const ChatBadgeSchema = z.object({
  id: z.string(),
  label: z.string(),
  imageUrl: z.string().url(),
});
export type ChatBadge = z.infer<typeof ChatBadgeSchema>;

export const ChatEmoteSchema = z.object({
  id: z.string(),
  code: z.string(),
  imageUrl: z.string().url(),
  startIndex: z.number().int().nonnegative(),
  endIndex: z.number().int().nonnegative(),
});
export type ChatEmote = z.infer<typeof ChatEmoteSchema>;

export const ChatMessageSchema = z.object({
  id: z.string(),
  platform: PlatformSchema,
  username: z.string(),
  displayName: z.string(),
  messageText: z.string(),
  badges: z.array(ChatBadgeSchema).default([]),
  emotes: z.array(ChatEmoteSchema).default([]),
  isMod: z.boolean().default(false),
  isSub: z.boolean().default(false),
  isVip: z.boolean().default(false),
  timestamp: z.string().datetime(),
  colour: z.string().optional(),
  replyTo: z.string().optional(),
});
export type ChatMessage = z.infer<typeof ChatMessageSchema>;

// ── Moderation ───────────────────────────────────────────────────────

export const ModerationActionTypeSchema = z.enum([
  'timeout',
  'ban',
  'unban',
  'delete_message',
  'slow_mode',
  'followers_only',
  'subscribers_only',
  'emote_only',
  'clear_chat',
]);
export type ModerationActionType = z.infer<typeof ModerationActionTypeSchema>;

export const ModerationActionSchema = z.object({
  id: z.string().uuid(),
  type: ModerationActionTypeSchema,
  platform: PlatformSchema,
  moderatorUsername: z.string(),
  targetUsername: z.string().optional(),
  reason: z.string().optional(),
  duration: z.number().int().nonnegative().optional(),
  timestamp: z.string().datetime(),
});
export type ModerationAction = z.infer<typeof ModerationActionSchema>;

// ── OBS & Socket.IO ──────────────────────────────────────────────────

export const ObsConnectionStatusSchema = z.enum([
  'disconnected',
  'connecting',
  'connected',
  'error',
]);
export type ObsConnectionStatus = z.infer<typeof ObsConnectionStatusSchema>;

export const StreamStatusSchema = z.enum(['offline', 'live', 'starting', 'ending']);
export type StreamStatus = z.infer<typeof StreamStatusSchema>;

export const SocketNamespaceSchema = z.enum(['/overlays', '/control']);
export type SocketNamespace = z.infer<typeof SocketNamespaceSchema>;
