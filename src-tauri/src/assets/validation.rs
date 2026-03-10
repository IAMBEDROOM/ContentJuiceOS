use super::error::AssetError;
use super::types::AssetType;

/// Maps a lowercase file extension to the corresponding asset type.
/// Returns `None` for unrecognized extensions.
pub fn detect_asset_type(extension: &str) -> Option<AssetType> {
    match extension {
        // Image
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => Some(AssetType::Image),
        // Audio
        "mp3" | "wav" | "ogg" | "aac" => Some(AssetType::Audio),
        // Video
        "mp4" | "mov" | "webm" | "mkv" => Some(AssetType::Video),
        // Font
        "ttf" | "otf" | "woff2" => Some(AssetType::Font),
        // Animation (json = Lottie, apng = animated PNG)
        "json" | "apng" => Some(AssetType::Animation),
        // Caption
        "srt" | "vtt" => Some(AssetType::Caption),
        _ => None,
    }
}

/// Validates that the given extension is in the allowlist for the asset type.
pub fn validate_format(asset_type: AssetType, extension: &str) -> Result<(), AssetError> {
    let allowed: &[&str] = match asset_type {
        AssetType::Image => &["png", "jpg", "jpeg", "gif", "webp", "svg"],
        AssetType::Audio => &["mp3", "wav", "ogg", "aac"],
        AssetType::Video => &["mp4", "mov", "webm", "mkv"],
        AssetType::Font => &["ttf", "otf", "woff2"],
        AssetType::Animation => &["json", "apng", "gif", "webm"],
        AssetType::Caption => &["srt", "vtt", "json"],
    };

    if allowed.contains(&extension) {
        Ok(())
    } else {
        Err(AssetError::FormatNotSupported(format!(
            ".{extension} is not allowed for {type_name} assets",
            type_name = asset_type.as_db_str()
        )))
    }
}

/// Returns the maximum allowed file size in bytes for the given asset type.
pub fn max_size_bytes(asset_type: AssetType) -> u64 {
    match asset_type {
        AssetType::Image => 20 * 1024 * 1024,       // 20 MB
        AssetType::Audio => 50 * 1024 * 1024,       // 50 MB
        AssetType::Video => 4 * 1024 * 1024 * 1024, // 4 GB
        AssetType::Font => 20 * 1024 * 1024,        // 20 MB
        AssetType::Animation => 50 * 1024 * 1024,   // 50 MB
        AssetType::Caption => 10 * 1024 * 1024,     // 10 MB
    }
}

/// Validates that the file size is within the allowed limit for the asset type.
pub fn validate_size(asset_type: AssetType, size_bytes: u64) -> Result<(), AssetError> {
    let limit = max_size_bytes(asset_type);
    if size_bytes > limit {
        Err(AssetError::FileTooLarge {
            limit_bytes: limit,
            actual_bytes: size_bytes,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_asset_type_images() {
        for ext in &["png", "jpg", "jpeg", "gif", "webp", "svg"] {
            assert_eq!(detect_asset_type(ext), Some(AssetType::Image), "ext: {ext}");
        }
    }

    #[test]
    fn detect_asset_type_audio() {
        for ext in &["mp3", "wav", "ogg", "aac"] {
            assert_eq!(detect_asset_type(ext), Some(AssetType::Audio), "ext: {ext}");
        }
    }

    #[test]
    fn detect_asset_type_video() {
        for ext in &["mp4", "mov", "webm", "mkv"] {
            assert_eq!(detect_asset_type(ext), Some(AssetType::Video), "ext: {ext}");
        }
    }

    #[test]
    fn detect_asset_type_font() {
        for ext in &["ttf", "otf", "woff2"] {
            assert_eq!(detect_asset_type(ext), Some(AssetType::Font), "ext: {ext}");
        }
    }

    #[test]
    fn detect_asset_type_animation() {
        for ext in &["json", "apng"] {
            assert_eq!(
                detect_asset_type(ext),
                Some(AssetType::Animation),
                "ext: {ext}"
            );
        }
    }

    #[test]
    fn detect_asset_type_caption() {
        for ext in &["srt", "vtt"] {
            assert_eq!(
                detect_asset_type(ext),
                Some(AssetType::Caption),
                "ext: {ext}"
            );
        }
    }

    #[test]
    fn detect_asset_type_unknown() {
        assert_eq!(detect_asset_type("exe"), None);
        assert_eq!(detect_asset_type("zip"), None);
        assert_eq!(detect_asset_type(""), None);
    }

    #[test]
    fn validate_format_accepts_valid() {
        assert!(validate_format(AssetType::Image, "png").is_ok());
        assert!(validate_format(AssetType::Audio, "mp3").is_ok());
        assert!(validate_format(AssetType::Video, "mp4").is_ok());
        assert!(validate_format(AssetType::Font, "ttf").is_ok());
        assert!(validate_format(AssetType::Animation, "json").is_ok());
        assert!(validate_format(AssetType::Caption, "srt").is_ok());
    }

    #[test]
    fn validate_format_rejects_invalid() {
        assert!(matches!(
            validate_format(AssetType::Image, "mp3"),
            Err(AssetError::FormatNotSupported(_))
        ));
        assert!(matches!(
            validate_format(AssetType::Audio, "png"),
            Err(AssetError::FormatNotSupported(_))
        ));
    }

    #[test]
    fn validate_format_animation_allows_gif_and_webm() {
        assert!(validate_format(AssetType::Animation, "gif").is_ok());
        assert!(validate_format(AssetType::Animation, "webm").is_ok());
    }

    #[test]
    fn validate_size_within_limit() {
        assert!(validate_size(AssetType::Image, 1024).is_ok());
        assert!(validate_size(AssetType::Image, 20 * 1024 * 1024).is_ok()); // exactly at limit
    }

    #[test]
    fn validate_size_exceeds_limit() {
        let result = validate_size(AssetType::Image, 20 * 1024 * 1024 + 1);
        assert!(matches!(
            result,
            Err(AssetError::FileTooLarge {
                limit_bytes: _,
                actual_bytes: _
            })
        ));
    }

    #[test]
    fn validate_size_video_large_limit() {
        // Video allows up to 4 GB
        assert!(validate_size(AssetType::Video, 3 * 1024 * 1024 * 1024).is_ok());
    }

    #[test]
    fn max_size_bytes_returns_correct_values() {
        assert_eq!(max_size_bytes(AssetType::Image), 20 * 1024 * 1024);
        assert_eq!(max_size_bytes(AssetType::Audio), 50 * 1024 * 1024);
        assert_eq!(max_size_bytes(AssetType::Video), 4 * 1024 * 1024 * 1024);
        assert_eq!(max_size_bytes(AssetType::Font), 20 * 1024 * 1024);
        assert_eq!(max_size_bytes(AssetType::Animation), 50 * 1024 * 1024);
        assert_eq!(max_size_bytes(AssetType::Caption), 10 * 1024 * 1024);
    }
}
