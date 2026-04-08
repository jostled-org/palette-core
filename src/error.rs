use std::sync::Arc;

/// Errors produced when loading or parsing theme manifests.
#[derive(Debug, thiserror::Error)]
pub enum PaletteError {
    /// TOML deserialization failed.
    #[error("failed to parse manifest: {0}")]
    Parse(#[from] toml::de::Error),

    /// File read failed.
    #[error("failed to read {path}: {source}")]
    Io {
        /// Path that could not be read.
        path: Arc<str>,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// Manifest has no `[base]` section.
    #[error("manifest missing required [base] section")]
    MissingBase,

    /// Manifest has no `[meta]` section (required for registry).
    #[error("manifest missing required [meta] section")]
    MissingMeta,

    /// A hex color string in the manifest is malformed.
    #[error("invalid hex `{value}` in [{section}].{field}")]
    InvalidHex {
        /// TOML section containing the bad value.
        section: Arc<str>,
        /// Field name within the section.
        field: Arc<str>,
        /// The malformed hex string.
        value: Arc<str>,
    },

    /// A style modifier string in the manifest is malformed.
    #[error("invalid style `{value}` in [{section}].{field}")]
    InvalidStyle {
        /// TOML section containing the bad value.
        section: Arc<str>,
        /// Field name within the section.
        field: Arc<str>,
        /// The malformed style string.
        value: Arc<str>,
    },

    /// A field key in the manifest is not recognized for its section.
    #[error("unknown field `{field}` in [{section}]")]
    UnknownField {
        /// TOML section containing the unrecognized key.
        section: Arc<str>,
        /// The unrecognized field name.
        field: Arc<str>,
    },

    /// No built-in or registered preset matches the given ID.
    #[error("unknown preset: {0}")]
    UnknownPreset(Arc<str>),

    /// A gradient has fewer than 2 color stops.
    #[error("gradient requires at least 2 stops, got {count}")]
    InsufficientStops {
        /// Number of stops provided.
        count: usize,
    },

    /// Gradient stop positions are not monotonically increasing.
    #[error("gradient stop positions are not sorted")]
    UnsortedStops,

    /// A gradient stop position is outside the normalized `[0, 1]` range.
    #[error("gradient stop position must be in [0, 1], got {position}")]
    InvalidGradientPosition {
        /// The out-of-range position value.
        position: f64,
    },

    /// A gradient mixes shorthand and explicit stop syntaxes.
    #[error("gradient [{gradient}] mixes shorthand and explicit stop syntax")]
    MixedGradientStopKinds {
        /// Name of the gradient definition.
        gradient: Arc<str>,
    },

    /// A gradient stop references a token path that does not exist.
    #[error(
        "invalid gradient reference in [{gradient}] stop {stop_index}: \"{reference}\" is not a known color field"
    )]
    InvalidGradientRef {
        /// Name of the gradient definition.
        gradient: Arc<str>,
        /// Zero-based index of the offending stop.
        stop_index: usize,
        /// The raw `"section.field"` string that failed validation.
        reference: Arc<str>,
    },

    /// A gradient specifies an unrecognized interpolation color space.
    #[error("invalid color space \"{value}\" in [gradient.{gradient}]")]
    InvalidColorSpace {
        /// Name of the gradient definition.
        gradient: Arc<str>,
        /// The unrecognized color space string.
        value: Arc<str>,
    },
}
