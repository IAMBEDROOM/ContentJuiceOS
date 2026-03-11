import { z } from 'zod';

// ── Design Type ──────────────────────────────────────────────────────

export const DesignTypeSchema = z.enum(['alert', 'overlay', 'scene', 'stinger', 'panel']);
export type DesignType = z.infer<typeof DesignTypeSchema>;

// ── Geometry ─────────────────────────────────────────────────────────

export const PositionSchema = z.object({
  x: z.number(),
  y: z.number(),
});
export type Position = z.infer<typeof PositionSchema>;

export const SizeSchema = z.object({
  width: z.number().nonnegative(),
  height: z.number().nonnegative(),
});
export type Size = z.infer<typeof SizeSchema>;

// ── Animation ────────────────────────────────────────────────────────

export const AnimationTypeSchema = z.enum([
  'fade',
  'slide_left',
  'slide_right',
  'slide_up',
  'slide_down',
  'scale',
  'bounce',
  'rotate',
  'shake',
  'none',
]);
export type AnimationType = z.infer<typeof AnimationTypeSchema>;

export const EasingSchema = z.enum([
  'linear',
  'ease_in',
  'ease_out',
  'ease_in_out',
  'bounce',
  'elastic',
]);
export type Easing = z.infer<typeof EasingSchema>;

export const AnimationPropsSchema = z.object({
  type: AnimationTypeSchema,
  duration: z.number().nonnegative().default(300),
  delay: z.number().nonnegative().default(0),
  easing: EasingSchema.default('ease_out'),
});
export type AnimationProps = z.infer<typeof AnimationPropsSchema>;

export const ElementAnimationSchema = z.object({
  entrance: AnimationPropsSchema.optional(),
  exit: AnimationPropsSchema.optional(),
  loop: AnimationPropsSchema.optional(),
});
export type ElementAnimation = z.infer<typeof ElementAnimationSchema>;

// ── Sound Trigger ────────────────────────────────────────────────────

export const SoundTriggerEventSchema = z.enum(['on_show', 'on_entrance', 'loop']);
export type SoundTriggerEvent = z.infer<typeof SoundTriggerEventSchema>;

export const SoundTriggerSchema = z.object({
  assetId: z.string().uuid(),
  volume: z.number().min(0).max(1).default(1),
  event: SoundTriggerEventSchema.default('on_show'),
});
export type SoundTrigger = z.infer<typeof SoundTriggerSchema>;

// ── Base Element (internal) ──────────────────────────────────────────

const BaseElementSchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  position: PositionSchema,
  size: SizeSchema,
  rotation: z.number().default(0),
  opacity: z.number().min(0).max(1).default(1),
  visible: z.boolean().default(true),
  locked: z.boolean().default(false),
  layerOrder: z.number().int().nonnegative(),
  animation: ElementAnimationSchema.optional(),
  sound: SoundTriggerSchema.optional(),
});

// ── Shared Visual Properties ─────────────────────────────────────────

export const ShadowSchema = z.object({
  color: z.string(),
  offsetX: z.number(),
  offsetY: z.number(),
  blur: z.number().nonnegative(),
});
export type Shadow = z.infer<typeof ShadowSchema>;

export const BorderSchema = z.object({
  color: z.string(),
  width: z.number().nonnegative(),
});
export type Border = z.infer<typeof BorderSchema>;

// ── Concrete Element Types ───────────────────────────────────────────

export const TextElementSchema = BaseElementSchema.extend({
  elementType: z.literal('text'),
  text: z.string(),
  fontFamily: z.string().default('Inter'),
  fontSize: z.number().positive().default(24),
  fontWeight: z.number().int().min(100).max(900).default(400),
  color: z.string().default('#FFFFFF'),
  textAlign: z.enum(['left', 'center', 'right']).default('left'),
  lineHeight: z.number().positive().default(1.4),
  stroke: BorderSchema.optional(),
  shadow: ShadowSchema.optional(),
});
export type TextElement = z.infer<typeof TextElementSchema>;

export const ImageElementSchema = BaseElementSchema.extend({
  elementType: z.literal('image'),
  assetId: z.string().uuid(),
  fitMode: z.enum(['contain', 'cover', 'fill', 'none']).default('contain'),
  borderRadius: z.number().nonnegative().default(0),
  border: BorderSchema.optional(),
  shadow: ShadowSchema.optional(),
});
export type ImageElement = z.infer<typeof ImageElementSchema>;

export const ShapeTypeSchema = z.enum([
  'rectangle',
  'circle',
  'ellipse',
  'rounded_rectangle',
  'line',
]);
export type ShapeType = z.infer<typeof ShapeTypeSchema>;

export const ShapeElementSchema = BaseElementSchema.extend({
  elementType: z.literal('shape'),
  shapeType: ShapeTypeSchema,
  fillColor: z.string().default('#FFFFFF'),
  strokeColor: z.string().optional(),
  strokeWidth: z.number().nonnegative().default(0),
  borderRadius: z.number().nonnegative().default(0),
  shadow: ShadowSchema.optional(),
});
export type ShapeElement = z.infer<typeof ShapeElementSchema>;

export const AnimationElementSchema = BaseElementSchema.extend({
  elementType: z.literal('animation'),
  assetId: z.string().uuid(),
  fitMode: z.enum(['contain', 'cover', 'fill', 'none']).default('contain'),
  playOnLoad: z.boolean().default(true),
  loopAnimation: z.boolean().default(true),
});
export type AnimationElement = z.infer<typeof AnimationElementSchema>;

export const SoundElementSchema = BaseElementSchema.extend({
  elementType: z.literal('sound'),
  assetId: z.string().uuid(),
  volume: z.number().min(0).max(1).default(1),
  playMode: z.enum(['on_show', 'loop']).default('on_show'),
});
export type SoundElement = z.infer<typeof SoundElementSchema>;

// ── Design Element Union ─────────────────────────────────────────────

export const DesignElementSchema = z.discriminatedUnion('elementType', [
  TextElementSchema,
  ImageElementSchema,
  ShapeElementSchema,
  AnimationElementSchema,
  SoundElementSchema,
]);
export type DesignElement = z.infer<typeof DesignElementSchema>;

// ── Canvas & Design Tree ─────────────────────────────────────────────

export const CanvasSizeSchema = z.object({
  width: z.number().int().positive().default(1920),
  height: z.number().int().positive().default(1080),
});
export type CanvasSize = z.infer<typeof CanvasSizeSchema>;

export const DesignTreeSchema = z.object({
  schemaVersion: z.number().int().positive().default(1),
  canvas: CanvasSizeSchema.default({ width: 1920, height: 1080 }),
  backgroundColor: z.string().default('#0A0D14'),
  elements: z.array(DesignElementSchema).default([]),
});
export type DesignTree = z.infer<typeof DesignTreeSchema>;

// ── Design Record ────────────────────────────────────────────────────

export const DesignSchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  type: DesignTypeSchema,
  config: DesignTreeSchema,
  thumbnail: z.string().optional(),
  tags: z.array(z.string()).default([]),
  description: z.string().default(''),
  createdAt: z.string().datetime(),
  updatedAt: z.string().datetime(),
});
export type Design = z.infer<typeof DesignSchema>;
