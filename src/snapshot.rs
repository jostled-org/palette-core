//! JSON serialization for palettes.

use crate::palette::Palette;

impl Palette {
    /// Serialize to a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        to_json(self)
    }

    /// Serialize to a [`serde_json::Value`] for programmatic inspection.
    pub fn to_json_value(&self) -> Result<serde_json::Value, serde_json::Error> {
        to_json_value(self)
    }
}

/// Serialize a palette to a pretty-printed JSON string.
pub fn to_json(palette: &Palette) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(palette)
}

/// Serialize a palette to a [`serde_json::Value`].
pub fn to_json_value(palette: &Palette) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(palette)
}
