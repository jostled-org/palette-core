use std::sync::Arc;

/// Errors produced when loading or parsing theme manifests.
#[derive(Debug, thiserror::Error)]
pub enum PaletteError {
    #[error("failed to parse manifest: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("failed to read {path}: {source}")]
    Io {
        path: Arc<str>,
        source: std::io::Error,
    },

    #[error("manifest missing required [base] section")]
    MissingBase,

    #[error("manifest missing required [meta] section")]
    MissingMeta,

    #[error("invalid hex `{value}` in [{section}].{field}")]
    InvalidHex {
        section: Arc<str>,
        field: Arc<str>,
        value: Arc<str>,
    },

    #[error("unknown preset: {0}")]
    UnknownPreset(Arc<str>),
}
