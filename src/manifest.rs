use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use serde::Deserialize;

use crate::error::PaletteError;

/// A single gradient stop in TOML: either a bare string or `{ color, at }`.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RawGradientStop {
    /// Shorthand: `"#FF0000"` or `"base.foreground"` — position auto-assigned.
    Shorthand(String),
    /// Explicit: `{ color = "...", at = 0.3 }`.
    Explicit {
        /// Hex color or token reference.
        color: String,
        /// Position in \[0, 1\].
        at: f64,
    },
}

/// Raw gradient definition as deserialized from TOML.
#[derive(Debug, Clone, Deserialize)]
pub struct RawGradientDef {
    /// Ordered color stops.
    pub stops: Vec<RawGradientStop>,
    /// Interpolation color space name (e.g. `"oklch"`). `None` means default (OkLab).
    #[serde(default)]
    pub space: Option<String>,
}

/// Named gradient definitions: `[gradient.name]` sections in TOML.
pub type GradientSections = HashMap<Arc<str>, RawGradientDef>;

/// A single TOML section mapping slot names to hex color strings.
pub type ManifestSection = HashMap<Arc<str>, Arc<str>>;

/// Platform-keyed overrides, e.g. `[platform.macos]`.
pub type PlatformSections = BTreeMap<Arc<str>, ManifestSection>;

/// The `[meta]` section of a theme TOML file.
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestMeta {
    /// Human-readable theme name.
    pub name: Arc<str>,
    /// Machine identifier used for lookups.
    pub preset_id: Arc<str>,
    /// Schema version string (e.g. `"1"`).
    pub schema_version: Arc<str>,
    /// Visual style tag: `"dark"`, `"light"`, etc.
    pub style: Arc<str>,
    /// Theme kind (e.g. `"base"`, `"variant"`).
    pub kind: Arc<str>,
    /// Parent preset ID for inheritance.
    #[serde(default)]
    pub inherits: Option<Arc<str>>,
    /// Upstream repository URL, if ported from another project.
    #[serde(default)]
    pub upstream_repo: Option<Arc<str>>,
}

/// Parsed but unresolved theme manifest.
///
/// Holds raw hex strings grouped by section. Convert to a [`Palette`](crate::Palette)
/// via [`Palette::from_manifest`](crate::Palette::from_manifest) after resolving
/// inheritance with [`merge_manifests`](crate::merge::merge_manifests).
#[derive(Debug, Clone)]
pub struct PaletteManifest {
    /// Theme identity and inheritance metadata.
    pub meta: Option<Arc<ManifestMeta>>,
    /// Core background/foreground hex values.
    pub base: ManifestSection,
    /// Status color hex values (success, error, etc.).
    pub semantic: ManifestSection,
    /// Diff highlighting hex values.
    pub diff: ManifestSection,
    /// UI surface hex values (menus, sidebars, etc.).
    pub surface: ManifestSection,
    /// Text chrome hex values (comments, line numbers, etc.).
    pub typography: ManifestSection,
    /// Syntax token hex values.
    pub syntax: ManifestSection,
    /// Editor chrome hex values (cursor, selections, etc.).
    pub editor: ManifestSection,
    /// ANSI terminal color hex values.
    pub terminal: ManifestSection,
    /// Syntax token style modifiers (bold, italic, underline).
    pub syntax_style: ManifestSection,
    /// Named gradient definitions parsed from `[gradient.*]` sections.
    pub gradient: GradientSections,
    /// Per-platform color overrides.
    #[cfg(feature = "platform")]
    pub platform: PlatformSections,
}

impl PaletteManifest {
    /// Parse a TOML string into a manifest. Requires a `[base]` section.
    pub fn from_toml(s: &str) -> Result<Self, PaletteError> {
        let raw: RawManifest = toml::from_str(s)?;

        match raw.base {
            None => Err(PaletteError::MissingBase),
            Some(base) => Ok(Self {
                meta: raw.meta.map(Arc::new),
                base,
                semantic: raw.semantic,
                diff: raw.diff,
                surface: raw.surface,
                typography: raw.typography,
                syntax: raw.syntax,
                editor: raw.editor,
                terminal: raw.terminal,
                syntax_style: raw.syntax_style,
                gradient: raw.gradient,
                #[cfg(feature = "platform")]
                platform: raw.platform,
            }),
        }
    }

    /// The parent preset ID if this manifest uses inheritance.
    pub fn inherits_from(&self) -> Option<&str> {
        self.meta.as_ref().and_then(|m| m.inherits.as_deref())
    }
}

/// A field key present in a manifest section that is not recognized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownField {
    /// The TOML section name (e.g. `"syntax"`).
    pub section: Box<str>,
    /// The unrecognized field key.
    pub field: Box<str>,
}

impl std::fmt::Display for UnknownField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}].{}", self.section, self.field)
    }
}

/// Field-name arrays from [`color_fields!`](crate::palette::color_fields) --
/// single source of truth for validation. Unsorted (semantic order);
/// [`validate_fields`] sorts once per call for `binary_search`.
pub(crate) mod known_fields {
    macro_rules! emit {
        ($(#[$_meta:meta])* BaseColors { $($field:ident),+ $(,)? }) => {
            pub(crate) const BASE: &[&str] = &[$(stringify!($field)),+];
        };
        ($(#[$_meta:meta])* SemanticColors { $($field:ident),+ $(,)? }) => {
            pub(crate) const SEMANTIC: &[&str] = &[$(stringify!($field)),+];
        };
        ($(#[$_meta:meta])* DiffColors { $($field:ident),+ $(,)? }) => {
            pub(crate) const DIFF: &[&str] = &[$(stringify!($field)),+];
        };
        ($(#[$_meta:meta])* SurfaceColors { $($field:ident),+ $(,)? }) => {
            pub(crate) const SURFACE: &[&str] = &[$(stringify!($field)),+];
        };
        ($(#[$_meta:meta])* TypographyColors { $($field:ident),+ $(,)? }) => {
            pub(crate) const TYPOGRAPHY: &[&str] = &[$(stringify!($field)),+];
        };
        ($(#[$_meta:meta])* SyntaxColors { $($field:ident),+ $(,)? }) => {
            pub(crate) const SYNTAX: &[&str] = &[$(stringify!($field)),+];
        };
        ($(#[$_meta:meta])* EditorColors { $($field:ident),+ $(,)? }) => {
            pub(crate) const EDITOR: &[&str] = &[$(stringify!($field)),+];
        };
        ($(#[$_meta:meta])* AnsiColors { $($field:ident),+ $(,)? }) => {
            pub(crate) const TERMINAL: &[&str] = &[$(stringify!($field)),+];
        };
    }
    crate::palette::color_fields!(emit);

    /// Map a section name to its known field list. Returns `None` for
    /// unrecognized section names.
    pub(crate) fn fields_for_section(section: &str) -> Option<&'static [&'static str]> {
        match section {
            "base" => Some(BASE),
            "semantic" => Some(SEMANTIC),
            "diff" => Some(DIFF),
            "surface" => Some(SURFACE),
            "typography" => Some(TYPOGRAPHY),
            "syntax" => Some(SYNTAX),
            "editor" => Some(EDITOR),
            "terminal" => Some(TERMINAL),
            _ => None,
        }
    }
}

struct SortedFields {
    base: Box<[&'static str]>,
    semantic: Box<[&'static str]>,
    diff: Box<[&'static str]>,
    surface: Box<[&'static str]>,
    typography: Box<[&'static str]>,
    syntax: Box<[&'static str]>,
    editor: Box<[&'static str]>,
    terminal: Box<[&'static str]>,
}

fn sort_fields(fields: &[&'static str]) -> Box<[&'static str]> {
    let mut sorted = fields.to_vec();
    sorted.sort_unstable();
    sorted.into_boxed_slice()
}

/// Check every section key against the known field set.
///
/// This is opt-in validation for theme lint tooling -- not called during
/// normal [`PaletteManifest::from_manifest`](crate::Palette::from_manifest).
pub fn validate_fields(manifest: &PaletteManifest) -> Box<[UnknownField]> {
    use std::sync::LazyLock;

    fn check_section(
        unknowns: &mut Vec<UnknownField>,
        section_name: &str,
        section: &ManifestSection,
        sorted: &[&str],
    ) {
        for key in section.keys() {
            match sorted.binary_search(&key.as_ref()) {
                Ok(_) => {}
                Err(_) => unknowns.push(UnknownField {
                    section: Box::from(section_name),
                    field: Box::from(key.as_ref()),
                }),
            }
        }
    }

    // Sort each known-field slice once per process.
    static SORTED: LazyLock<SortedFields> = LazyLock::new(|| SortedFields {
        base: sort_fields(known_fields::BASE),
        semantic: sort_fields(known_fields::SEMANTIC),
        diff: sort_fields(known_fields::DIFF),
        surface: sort_fields(known_fields::SURFACE),
        typography: sort_fields(known_fields::TYPOGRAPHY),
        syntax: sort_fields(known_fields::SYNTAX),
        editor: sort_fields(known_fields::EDITOR),
        terminal: sort_fields(known_fields::TERMINAL),
    });

    let s = &*SORTED;
    let mut unknowns = Vec::new();

    // Section-to-manifest mapping is manual; field lists come from
    // color_fields! via the known_fields module (single source of truth).
    check_section(&mut unknowns, "base", &manifest.base, &s.base);
    check_section(&mut unknowns, "semantic", &manifest.semantic, &s.semantic);
    check_section(&mut unknowns, "diff", &manifest.diff, &s.diff);
    check_section(&mut unknowns, "surface", &manifest.surface, &s.surface);
    check_section(
        &mut unknowns,
        "typography",
        &manifest.typography,
        &s.typography,
    );
    check_section(&mut unknowns, "editor", &manifest.editor, &s.editor);
    check_section(&mut unknowns, "terminal", &manifest.terminal, &s.terminal);

    // Syntax and syntax_style sections share the same valid field names.
    check_section(&mut unknowns, "syntax", &manifest.syntax, &s.syntax);
    check_section(
        &mut unknowns,
        "syntax_style",
        &manifest.syntax_style,
        &s.syntax,
    );

    unknowns.into_boxed_slice()
}

#[derive(Deserialize)]
struct RawManifest {
    #[serde(default)]
    meta: Option<ManifestMeta>,
    #[serde(default)]
    base: Option<ManifestSection>,
    #[serde(default)]
    semantic: ManifestSection,
    #[serde(default)]
    diff: ManifestSection,
    #[serde(default)]
    surface: ManifestSection,
    #[serde(default)]
    typography: ManifestSection,
    #[serde(default)]
    syntax: ManifestSection,
    #[serde(default)]
    editor: ManifestSection,
    #[serde(default)]
    terminal: ManifestSection,
    #[serde(default)]
    syntax_style: ManifestSection,
    #[serde(default)]
    gradient: GradientSections,
    #[cfg(feature = "platform")]
    #[serde(default)]
    platform: PlatformSections,
}
