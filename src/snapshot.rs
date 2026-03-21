//! JSON serialization for palettes.

use crate::palette::Palette;

impl Palette {
    /// Serialize to a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serialize to a [`serde_json::Value`] for programmatic inspection.
    pub fn to_json_value(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}
