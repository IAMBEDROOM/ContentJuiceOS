use serde::{Deserialize, Serialize};

/// Asset type variants that mirror the DB CHECK constraint and frontend schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum AssetType {
    Image,
    Audio,
    Video,
    Font,
    Animation,
    Caption,
}

#[allow(dead_code)]
impl AssetType {
    /// Returns the subdirectory name for this asset type within the asset root.
    pub fn subdirectory(&self) -> &'static str {
        match self {
            Self::Image => "images",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Font => "fonts",
            Self::Animation => "animations",
            Self::Caption => "captions",
        }
    }

    /// Returns a slice of all asset type variants.
    pub fn all() -> &'static [AssetType] {
        &[
            Self::Image,
            Self::Audio,
            Self::Video,
            Self::Font,
            Self::Animation,
            Self::Caption,
        ]
    }

    /// Returns the lowercase DB string representation.
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Font => "font",
            Self::Animation => "animation",
            Self::Caption => "caption",
        }
    }
}

/// Result of a successful file import into the asset library.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct ImportedFile {
    /// UUID v4 identifier
    pub id: String,
    /// Path relative to asset root (e.g. "images/a1b2c3d4_photo.png")
    pub relative_path: String,
    /// Full absolute path on disk
    pub absolute_path: String,
    /// File size in bytes
    pub file_size: u64,
    /// Original filename (with extension) as provided by the user
    pub original_filename: String,
    /// Lowercase file extension without the dot
    pub format: String,
}

/// A fully-resolved asset record, matching the `assets` DB table.
/// Returned to the frontend after a successful import.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub id: String,
    pub original_filename: String,
    pub asset_type: AssetType,
    pub format: String,
    pub file_size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration: Option<f64>,
    pub tags: Vec<String>,
    pub file_path: String,
    pub created_at: String,
}

/// All subdirectories to create under the asset root.
/// Includes `voice_profiles` which has its own DB table, not part of `AssetType`.
pub const ALL_SUBDIRECTORIES: &[&str] = &[
    "images",
    "audio",
    "video",
    "fonts",
    "animations",
    "voice_profiles",
    "captions",
];
