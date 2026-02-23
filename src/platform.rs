use std::collections::BTreeMap;
use std::sync::Arc;

use crate::color::{Color, InvalidHex};
use crate::error::PaletteError;
use crate::manifest::PlatformSections;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
pub struct PlatformOverride {
    pub background: Option<Color>,
    pub foreground: Option<Color>,
}

pub type PlatformOverrides = BTreeMap<Arc<str>, PlatformOverride>;

fn resolve_color(hex: &str, platform: &str, field: &str) -> Result<Color, PaletteError> {
    Color::from_hex(hex).map_err(|InvalidHex { value }| PaletteError::InvalidHex {
        section: Arc::from(format!("platform.{platform}")),
        field: Arc::from(field),
        value,
    })
}

pub fn from_sections(sections: &PlatformSections) -> Result<PlatformOverrides, PaletteError> {
    sections
        .iter()
        .map(|(name, section)| {
            let background = section
                .get("background")
                .map(|hex| resolve_color(hex, name, "background"))
                .transpose()?;
            let foreground = section
                .get("foreground")
                .map(|hex| resolve_color(hex, name, "foreground"))
                .transpose()?;
            Ok((name.clone(), PlatformOverride { background, foreground }))
        })
        .collect()
}
