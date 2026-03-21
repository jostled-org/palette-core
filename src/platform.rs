//! Platform-specific color overrides (e.g. macOS, Windows, Linux).

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::color::Color;
use crate::error::PaletteError;
use crate::manifest::PlatformSections;

/// Background/foreground overrides for a single platform target.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
pub struct PlatformOverride {
    /// Background color override for this platform.
    pub background: Option<Color>,
    /// Foreground color override for this platform.
    pub foreground: Option<Color>,
}

/// Map of platform name to its color overrides.
pub type PlatformOverrides = BTreeMap<Arc<str>, PlatformOverride>;

fn resolve_color(hex: &str, platform: &str, field: &str) -> Result<Color, PaletteError> {
    Color::from_hex(hex).map_err(|e| {
        e.into_palette_error(Arc::from(format!("platform.{platform}")), Arc::from(field))
    })
}

const VALID_FIELDS: &[&str] = &["background", "foreground"];

/// Parse `[platform.*]` TOML sections into typed overrides.
///
/// Returns an error for any unrecognized key in a platform section.
pub fn from_sections(sections: &PlatformSections) -> Result<PlatformOverrides, PaletteError> {
    sections
        .iter()
        .map(|(name, section)| {
            validate_platform_keys(name, section)?;
            let background = section
                .get("background")
                .map(|hex| resolve_color(hex, name, "background"))
                .transpose()?;
            let foreground = section
                .get("foreground")
                .map(|hex| resolve_color(hex, name, "foreground"))
                .transpose()?;
            Ok((
                name.clone(),
                PlatformOverride {
                    background,
                    foreground,
                },
            ))
        })
        .collect()
}

fn validate_platform_keys(
    platform: &str,
    section: &crate::manifest::ManifestSection,
) -> Result<(), PaletteError> {
    for key in section.keys() {
        match VALID_FIELDS.contains(&key.as_ref()) {
            true => {}
            false => {
                return Err(PaletteError::UnknownField {
                    section: Arc::from(format!("platform.{platform}")),
                    field: Arc::clone(key),
                });
            }
        }
    }
    Ok(())
}
