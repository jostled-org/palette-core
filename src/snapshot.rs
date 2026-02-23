use crate::palette::Palette;

pub fn to_json(palette: &Palette) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(palette)
}

pub fn to_json_value(palette: &Palette) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(palette)
}
