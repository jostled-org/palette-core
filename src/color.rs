use std::fmt;
use std::sync::Arc;

use crate::error::PaletteError;

/// Returned when a hex string cannot be parsed as an RGB color.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid hex color: {value}")]
pub struct InvalidHex {
    /// The original string that failed to parse.
    pub value: Arc<str>,
}

impl InvalidHex {
    /// Convert into a [`PaletteError::InvalidHex`] with section and field context.
    pub(crate) fn into_palette_error(self, section: Arc<str>, field: Arc<str>) -> PaletteError {
        PaletteError::InvalidHex {
            section,
            field,
            value: self.value,
        }
    }
}

/// 8-bit RGB color.
///
/// Constructed from a `#RRGGBB` hex string via [`Color::from_hex`] or directly
/// from field values. Displays as uppercase hex (`#1A1A2E`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
#[cfg_attr(feature = "snapshot", serde(into = "String"))]
pub struct Color {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl Color {
    /// Parse a `#RRGGBB` hex string into a [`Color`].
    pub fn from_hex(hex: &str) -> Result<Self, InvalidHex> {
        let digits = match hex.strip_prefix('#') {
            Some(d) if d.len() == 6 && d.is_ascii() => d,
            _ => {
                return Err(InvalidHex {
                    value: Arc::from(hex),
                });
            }
        };

        let r = u8::from_str_radix(&digits[0..2], 16);
        let g = u8::from_str_radix(&digits[2..4], 16);
        let b = u8::from_str_radix(&digits[4..6], 16);

        match (r, g, b) {
            (Ok(r), Ok(g), Ok(b)) => Ok(Self { r, g, b }),
            _ => Err(InvalidHex {
                value: Arc::from(hex),
            }),
        }
    }

    /// Format as a `#RRGGBB` hex string.
    pub fn to_hex(&self) -> Box<str> {
        let mut buf = String::with_capacity(7);
        use std::fmt::Write;
        let _ = write!(buf, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b);
        buf.into_boxed_str()
    }

    /// WCAG 2.1 relative luminance midpoint threshold.
    ///
    /// Colors with `relative_luminance() > LUMINANCE_MIDPOINT` are perceptually
    /// light. Derived from the geometric mean of black (0.0) and white (1.0)
    /// luminance under the WCAG contrast formula: `(1.05 / 0.05) = 21:1` yields
    /// a midpoint of approximately 0.179.
    pub const LUMINANCE_MIDPOINT: f64 = 0.179;

    /// Returns `true` if this color is perceptually light.
    ///
    /// Uses [`Self::LUMINANCE_MIDPOINT`] (WCAG relative luminance midpoint).
    pub fn is_light(&self) -> bool {
        self.relative_luminance() > Self::LUMINANCE_MIDPOINT
    }

    /// WCAG 2.1 relative luminance. Returns a value in `[0.0, 1.0]`.
    pub fn relative_luminance(&self) -> f64 {
        let lin = crate::manipulation::srgb_to_linear;
        0.2126 * lin(self.r) + 0.7152 * lin(self.g) + 0.0722 * lin(self.b)
    }
}

impl From<Color> for String {
    fn from(color: Color) -> Self {
        String::from(color.to_hex())
    }
}
