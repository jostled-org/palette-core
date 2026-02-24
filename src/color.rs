use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid hex color: {value}")]
pub struct InvalidHex {
    pub value: Arc<str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
#[cfg_attr(feature = "snapshot", serde(into = "String"))]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl Color {
    pub fn from_hex(hex: &str) -> Result<Self, InvalidHex> {
        let digits = match hex.strip_prefix('#') {
            Some(d) if d.len() == 6 && d.is_ascii() => d,
            _ => return Err(InvalidHex { value: Arc::from(hex) }),
        };

        let r = u8::from_str_radix(&digits[0..2], 16);
        let g = u8::from_str_radix(&digits[2..4], 16);
        let b = u8::from_str_radix(&digits[4..6], 16);

        match (r, g, b) {
            (Ok(r), Ok(g), Ok(b)) => Ok(Self { r, g, b }),
            _ => Err(InvalidHex { value: Arc::from(hex) }),
        }
    }

    pub fn to_hex(&self) -> String {
        self.to_string()
    }

    /// WCAG 2.1 relative luminance. Returns a value in `[0.0, 1.0]`.
    pub fn relative_luminance(&self) -> f64 {
        let linearize = |channel: u8| {
            let s = f64::from(channel) / 255.0;
            match s <= 0.04045 {
                true => s / 12.92,
                false => ((s + 0.055) / 1.055).powf(2.4),
            }
        };
        0.2126 * linearize(self.r) + 0.7152 * linearize(self.g) + 0.0722 * linearize(self.b)
    }
}

impl From<Color> for String {
    fn from(color: Color) -> Self {
        color.to_string()
    }
}
