use serde::{Deserialize, Serialize};

// ── Design Type ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DesignType {
    Alert,
    Overlay,
    Scene,
    Stinger,
    Panel,
}

impl DesignType {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Alert => "alert",
            Self::Overlay => "overlay",
            Self::Scene => "scene",
            Self::Stinger => "stinger",
            Self::Panel => "panel",
        }
    }
}

// ── Geometry ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

// ── Shared Visual Properties ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Shadow {
    pub color: String,
    pub offset_x: f64,
    pub offset_y: f64,
    pub blur: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Border {
    pub color: String,
    pub width: f64,
}

// ── Animation ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnimationType {
    Fade,
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    Scale,
    Bounce,
    Rotate,
    Shake,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimationProps {
    #[serde(rename = "type")]
    pub animation_type: AnimationType,
    #[serde(default = "default_duration")]
    pub duration: f64,
    #[serde(default)]
    pub delay: f64,
    #[serde(default = "default_easing")]
    pub easing: Easing,
}

fn default_duration() -> f64 {
    300.0
}

fn default_easing() -> Easing {
    Easing::EaseOut
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ElementAnimation {
    pub entrance: Option<AnimationProps>,
    pub exit: Option<AnimationProps>,
    #[serde(rename = "loop")]
    pub loop_animation: Option<AnimationProps>,
}

// ── Sound Trigger ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SoundTriggerEvent {
    OnShow,
    OnEntrance,
    Loop,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SoundTrigger {
    pub asset_id: String,
    #[serde(default = "default_volume")]
    pub volume: f64,
    #[serde(default = "default_sound_trigger_event")]
    pub event: SoundTriggerEvent,
}

fn default_volume() -> f64 {
    1.0
}

fn default_sound_trigger_event() -> SoundTriggerEvent {
    SoundTriggerEvent::OnShow
}

// ── Helper Enums ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FitMode {
    Contain,
    Cover,
    Fill,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ShapeType {
    Rectangle,
    Circle,
    Ellipse,
    RoundedRectangle,
    Line,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SoundPlayMode {
    OnShow,
    Loop,
}

// ── Element Data (variant-specific fields) ───────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextElementData {
    pub text: String,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: f64,
    #[serde(default = "default_font_weight")]
    pub font_weight: u32,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default = "default_text_align")]
    pub text_align: TextAlign,
    #[serde(default = "default_line_height")]
    pub line_height: f64,
    pub stroke: Option<Border>,
    pub shadow: Option<Shadow>,
}

fn default_font_family() -> String {
    "Inter".to_string()
}
fn default_font_size() -> f64 {
    24.0
}
fn default_font_weight() -> u32 {
    400
}
fn default_color() -> String {
    "#FFFFFF".to_string()
}
fn default_text_align() -> TextAlign {
    TextAlign::Left
}
fn default_line_height() -> f64 {
    1.4
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImageElementData {
    pub asset_id: String,
    #[serde(default = "default_fit_mode")]
    pub fit_mode: FitMode,
    #[serde(default)]
    pub border_radius: f64,
    pub border: Option<Border>,
    pub shadow: Option<Shadow>,
}

fn default_fit_mode() -> FitMode {
    FitMode::Contain
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShapeElementData {
    pub shape_type: ShapeType,
    #[serde(default = "default_color")]
    pub fill_color: String,
    pub stroke_color: Option<String>,
    #[serde(default)]
    pub stroke_width: f64,
    #[serde(default)]
    pub border_radius: f64,
    pub shadow: Option<Shadow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AnimationElementData {
    pub asset_id: String,
    #[serde(default = "default_fit_mode")]
    pub fit_mode: FitMode,
    #[serde(default = "default_true")]
    pub play_on_load: bool,
    #[serde(default = "default_true")]
    pub loop_animation: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SoundElementData {
    pub asset_id: String,
    #[serde(default = "default_volume")]
    pub volume: f64,
    #[serde(default = "default_sound_play_mode")]
    pub play_mode: SoundPlayMode,
}

fn default_sound_play_mode() -> SoundPlayMode {
    SoundPlayMode::OnShow
}

// ── Element Data Union ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "elementType", rename_all = "camelCase")]
pub enum ElementData {
    #[serde(rename = "text")]
    Text(TextElementData),
    #[serde(rename = "image")]
    Image(ImageElementData),
    #[serde(rename = "shape")]
    Shape(ShapeElementData),
    #[serde(rename = "animation")]
    Animation(AnimationElementData),
    #[serde(rename = "sound")]
    Sound(SoundElementData),
}

// ── Design Element ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DesignElement {
    pub id: String,
    pub name: String,
    pub position: Position,
    pub size: Size,
    #[serde(default)]
    pub rotation: f64,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default)]
    pub locked: bool,
    pub layer_order: u32,
    pub animation: Option<ElementAnimation>,
    pub sound: Option<SoundTrigger>,
    #[serde(flatten)]
    pub data: ElementData,
}

fn default_opacity() -> f64 {
    1.0
}

// ── Canvas & Design Tree ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanvasSize {
    #[serde(default = "default_canvas_width")]
    pub width: u32,
    #[serde(default = "default_canvas_height")]
    pub height: u32,
}

fn default_canvas_width() -> u32 {
    1920
}
fn default_canvas_height() -> u32 {
    1080
}

impl Default for CanvasSize {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DesignTree {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default = "default_canvas")]
    pub canvas: CanvasSize,
    #[serde(default = "default_background_color")]
    pub background_color: String,
    #[serde(default)]
    pub elements: Vec<DesignElement>,
}

fn default_schema_version() -> u32 {
    1
}
fn default_canvas() -> CanvasSize {
    CanvasSize::default()
}
fn default_background_color() -> String {
    "#0A0D14".to_string()
}

impl Default for DesignTree {
    fn default() -> Self {
        Self {
            schema_version: 1,
            canvas: CanvasSize::default(),
            background_color: "#0A0D14".to_string(),
            elements: vec![],
        }
    }
}

// ── Design Record ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Design {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub design_type: DesignType,
    pub config: DesignTree,
    pub thumbnail: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

// ── List Response DTO ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignListResponse {
    pub designs: Vec<Design>,
    pub total: i64,
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn base_element_json(element_type_data: Value) -> Value {
        let mut base = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Test Element",
            "position": { "x": 100.0, "y": 200.0 },
            "size": { "width": 300.0, "height": 150.0 },
            "layerOrder": 0
        });
        // Merge element-type-specific fields into the base
        if let (Some(base_map), Some(data_map)) =
            (base.as_object_mut(), element_type_data.as_object())
        {
            for (k, v) in data_map {
                base_map.insert(k.clone(), v.clone());
            }
        }
        base
    }

    #[test]
    fn test_text_element_roundtrip() {
        let json_val = base_element_json(json!({
            "elementType": "text",
            "text": "Hello World",
            "fontFamily": "Inter",
            "fontSize": 32.0,
            "fontWeight": 700,
            "color": "#00E5FF",
            "textAlign": "center",
            "lineHeight": 1.6
        }));

        let elem: DesignElement = serde_json::from_value(json_val.clone()).unwrap();
        assert_eq!(elem.name, "Test Element");
        if let ElementData::Text(ref data) = elem.data {
            assert_eq!(data.text, "Hello World");
            assert_eq!(data.font_size, 32.0);
            assert_eq!(data.font_weight, 700);
            assert_eq!(data.text_align, TextAlign::Center);
        } else {
            panic!("Expected Text variant");
        }

        let serialized = serde_json::to_value(&elem).unwrap();
        assert_eq!(serialized["elementType"], "text");
        assert_eq!(serialized["text"], "Hello World");
    }

    #[test]
    fn test_image_element_roundtrip() {
        let json_val = base_element_json(json!({
            "elementType": "image",
            "assetId": "660e8400-e29b-41d4-a716-446655440001",
            "fitMode": "cover",
            "borderRadius": 8.0,
            "shadow": {
                "color": "rgba(0,0,0,0.5)",
                "offsetX": 2.0,
                "offsetY": 4.0,
                "blur": 10.0
            }
        }));

        let elem: DesignElement = serde_json::from_value(json_val).unwrap();
        if let ElementData::Image(ref data) = elem.data {
            assert_eq!(data.fit_mode, FitMode::Cover);
            assert!(data.shadow.is_some());
            assert_eq!(data.shadow.as_ref().unwrap().blur, 10.0);
        } else {
            panic!("Expected Image variant");
        }

        let roundtrip: Value = serde_json::to_value(&elem).unwrap();
        assert_eq!(roundtrip["elementType"], "image");
        assert_eq!(roundtrip["fitMode"], "cover");
    }

    #[test]
    fn test_shape_element_roundtrip() {
        let json_val = base_element_json(json!({
            "elementType": "shape",
            "shapeType": "rounded_rectangle",
            "fillColor": "#FF007F",
            "strokeWidth": 2.0,
            "borderRadius": 12.0
        }));

        let elem: DesignElement = serde_json::from_value(json_val).unwrap();
        if let ElementData::Shape(ref data) = elem.data {
            assert_eq!(data.shape_type, ShapeType::RoundedRectangle);
            assert_eq!(data.fill_color, "#FF007F");
        } else {
            panic!("Expected Shape variant");
        }
    }

    #[test]
    fn test_animation_element_roundtrip() {
        let json_val = base_element_json(json!({
            "elementType": "animation",
            "assetId": "770e8400-e29b-41d4-a716-446655440002",
            "fitMode": "contain",
            "playOnLoad": true,
            "loopAnimation": false
        }));

        let elem: DesignElement = serde_json::from_value(json_val).unwrap();
        if let ElementData::Animation(ref data) = elem.data {
            assert!(data.play_on_load);
            assert!(!data.loop_animation);
            assert_eq!(data.fit_mode, FitMode::Contain);
        } else {
            panic!("Expected Animation variant");
        }
    }

    #[test]
    fn test_sound_element_roundtrip() {
        let json_val = base_element_json(json!({
            "elementType": "sound",
            "assetId": "880e8400-e29b-41d4-a716-446655440003",
            "volume": 0.75,
            "playMode": "loop"
        }));

        let elem: DesignElement = serde_json::from_value(json_val).unwrap();
        if let ElementData::Sound(ref data) = elem.data {
            assert_eq!(data.volume, 0.75);
            assert_eq!(data.play_mode, SoundPlayMode::Loop);
        } else {
            panic!("Expected Sound variant");
        }
    }

    #[test]
    fn test_discriminated_union_tag() {
        let json_val = base_element_json(json!({
            "elementType": "text",
            "text": "Tag test"
        }));

        let elem: DesignElement = serde_json::from_value(json_val).unwrap();
        let serialized = serde_json::to_value(&elem).unwrap();

        // The elementType tag should be at the top level of the flat JSON
        assert_eq!(serialized["elementType"], "text");
        // The text field should also be at the top level (flattened)
        assert_eq!(serialized["text"], "Tag test");
        // id should be at the top level (base fields)
        assert!(serialized["id"].is_string());
    }

    #[test]
    fn test_text_defaults_applied() {
        // Minimal text element — only required fields
        let json_val = base_element_json(json!({
            "elementType": "text",
            "text": "Defaults"
        }));

        let elem: DesignElement = serde_json::from_value(json_val).unwrap();
        // Base defaults
        assert_eq!(elem.rotation, 0.0);
        assert_eq!(elem.opacity, 1.0);
        assert!(elem.visible);
        assert!(!elem.locked);

        if let ElementData::Text(ref data) = elem.data {
            assert_eq!(data.font_family, "Inter");
            assert_eq!(data.font_size, 24.0);
            assert_eq!(data.font_weight, 400);
            assert_eq!(data.color, "#FFFFFF");
            assert_eq!(data.text_align, TextAlign::Left);
            assert_eq!(data.line_height, 1.4);
            assert!(data.stroke.is_none());
            assert!(data.shadow.is_none());
        } else {
            panic!("Expected Text variant");
        }
    }

    #[test]
    fn test_full_design_roundtrip() {
        let design_json = json!({
            "id": "aae8400-e29b-41d4-a716-446655440000",
            "name": "Test Alert Design",
            "type": "alert",
            "tags": ["alert", "custom"],
            "description": "My custom alert for new followers",
            "config": {
                "schemaVersion": 1,
                "canvas": { "width": 1920, "height": 1080 },
                "backgroundColor": "#0A0D14",
                "elements": [
                    {
                        "id": "550e8400-e29b-41d4-a716-446655440000",
                        "name": "Title",
                        "position": { "x": 0.0, "y": 0.0 },
                        "size": { "width": 500.0, "height": 100.0 },
                        "layerOrder": 0,
                        "elementType": "text",
                        "text": "New Follower!"
                    },
                    {
                        "id": "550e8400-e29b-41d4-a716-446655440001",
                        "name": "Avatar",
                        "position": { "x": 50.0, "y": 120.0 },
                        "size": { "width": 128.0, "height": 128.0 },
                        "layerOrder": 1,
                        "elementType": "image",
                        "assetId": "660e8400-e29b-41d4-a716-446655440001"
                    },
                    {
                        "id": "550e8400-e29b-41d4-a716-446655440002",
                        "name": "Background",
                        "position": { "x": 0.0, "y": 0.0 },
                        "size": { "width": 600.0, "height": 400.0 },
                        "layerOrder": 2,
                        "elementType": "shape",
                        "shapeType": "rounded_rectangle",
                        "borderRadius": 16.0
                    },
                    {
                        "id": "550e8400-e29b-41d4-a716-446655440003",
                        "name": "Sparkle",
                        "position": { "x": 200.0, "y": 50.0 },
                        "size": { "width": 64.0, "height": 64.0 },
                        "layerOrder": 3,
                        "elementType": "animation",
                        "assetId": "770e8400-e29b-41d4-a716-446655440002",
                        "fitMode": "contain"
                    },
                    {
                        "id": "550e8400-e29b-41d4-a716-446655440004",
                        "name": "Alert Sound",
                        "position": { "x": 0.0, "y": 0.0 },
                        "size": { "width": 0.0, "height": 0.0 },
                        "layerOrder": 4,
                        "elementType": "sound",
                        "assetId": "880e8400-e29b-41d4-a716-446655440003",
                        "volume": 0.8,
                        "playMode": "on_show"
                    }
                ]
            },
            "createdAt": "2026-01-15T10:30:00Z",
            "updatedAt": "2026-01-15T10:30:00Z"
        });

        let design: Design = serde_json::from_value(design_json.clone()).unwrap();
        assert_eq!(design.name, "Test Alert Design");
        assert_eq!(design.design_type, DesignType::Alert);
        assert_eq!(design.tags, vec!["alert", "custom"]);
        assert_eq!(design.description, "My custom alert for new followers");
        assert_eq!(design.config.schema_version, 1);
        assert_eq!(design.config.elements.len(), 5);

        // Roundtrip
        let serialized = serde_json::to_value(&design).unwrap();
        let deserialized: Design = serde_json::from_value(serialized).unwrap();
        assert_eq!(design, deserialized);
    }

    #[test]
    fn test_design_tree_defaults() {
        let json_val = json!({});
        let tree: DesignTree = serde_json::from_value(json_val).unwrap();
        assert_eq!(tree.schema_version, 1);
        assert_eq!(tree.canvas.width, 1920);
        assert_eq!(tree.canvas.height, 1080);
        assert_eq!(tree.background_color, "#0A0D14");
        assert!(tree.elements.is_empty());
    }

    #[test]
    fn test_element_with_animation_and_sound() {
        let json_val = base_element_json(json!({
            "elementType": "text",
            "text": "Animated Text",
            "animation": {
                "entrance": {
                    "type": "fade",
                    "duration": 500.0,
                    "delay": 100.0,
                    "easing": "ease_in_out"
                },
                "exit": {
                    "type": "slide_left",
                    "duration": 300.0,
                    "delay": 0.0,
                    "easing": "ease_out"
                },
                "loop": null
            },
            "sound": {
                "assetId": "990e8400-e29b-41d4-a716-446655440004",
                "volume": 0.5,
                "event": "on_entrance"
            }
        }));

        let elem: DesignElement = serde_json::from_value(json_val).unwrap();
        assert!(elem.animation.is_some());
        let anim = elem.animation.as_ref().unwrap();
        assert!(anim.entrance.is_some());
        assert_eq!(
            anim.entrance.as_ref().unwrap().animation_type,
            AnimationType::Fade
        );
        assert_eq!(anim.entrance.as_ref().unwrap().duration, 500.0);

        assert!(elem.sound.is_some());
        assert_eq!(elem.sound.as_ref().unwrap().volume, 0.5);
        assert_eq!(
            elem.sound.as_ref().unwrap().event,
            SoundTriggerEvent::OnEntrance
        );
    }
}
