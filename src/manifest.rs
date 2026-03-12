use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use serde::Deserialize;

use crate::error::PaletteError;

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
    pub meta: Option<ManifestMeta>,
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
                meta: raw.meta,
                base,
                semantic: raw.semantic,
                diff: raw.diff,
                surface: raw.surface,
                typography: raw.typography,
                syntax: raw.syntax,
                editor: raw.editor,
                terminal: raw.terminal,
                syntax_style: raw.syntax_style,
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

/// Check every section key against the known field set.
///
/// This is opt-in validation for theme lint tooling — not called during
/// normal [`PaletteManifest::from_manifest`](crate::Palette::from_manifest).
pub fn validate_fields(manifest: &PaletteManifest) -> Vec<UnknownField> {
    macro_rules! known_fields {
        ($(#[$_meta:meta])* $name:ident { $($field:ident),+ $(,)? }) => {
            &[$(stringify!($field)),+]
        };
    }

    let checks: &[(&str, &ManifestSection, &[&str])] = &[
        (
            "base",
            &manifest.base,
            known_fields!(BaseColors {
                background,
                background_dark,
                background_highlight,
                foreground,
                foreground_dark,
                border,
                border_highlight,
            }),
        ),
        (
            "semantic",
            &manifest.semantic,
            known_fields!(SemanticColors {
                success,
                warning,
                error,
                info,
                hint,
            }),
        ),
        (
            "diff",
            &manifest.diff,
            known_fields!(DiffColors {
                added,
                added_bg,
                added_fg,
                modified,
                modified_bg,
                modified_fg,
                removed,
                removed_bg,
                removed_fg,
                text_bg,
                ignored,
            }),
        ),
        (
            "surface",
            &manifest.surface,
            known_fields!(SurfaceColors {
                menu,
                sidebar,
                statusline,
                float,
                popup,
                overlay,
                highlight,
                selection,
                focus,
                search,
            }),
        ),
        (
            "typography",
            &manifest.typography,
            known_fields!(TypographyColors {
                comment,
                gutter,
                line_number,
                selection_text,
                link,
                title,
            }),
        ),
        (
            "editor",
            &manifest.editor,
            known_fields!(EditorColors {
                cursor,
                cursor_text,
                match_paren,
                selection_bg,
                selection_fg,
                inlay_hint_bg,
                inlay_hint_fg,
                search_bg,
                search_fg,
                diagnostic_error,
                diagnostic_warn,
                diagnostic_info,
                diagnostic_hint,
                diagnostic_underline_error,
                diagnostic_underline_warn,
                diagnostic_underline_info,
                diagnostic_underline_hint,
            }),
        ),
        (
            "terminal",
            &manifest.terminal,
            known_fields!(TerminalAnsiColors {
                black,
                red,
                green,
                yellow,
                blue,
                magenta,
                cyan,
                white,
                bright_black,
                bright_red,
                bright_green,
                bright_yellow,
                bright_blue,
                bright_magenta,
                bright_cyan,
                bright_white,
            }),
        ),
    ];

    // Syntax fields via syntax_fields! macro
    macro_rules! syntax_known {
        ($(#[$_meta:meta])* $_name:ident { $($field:ident),+ $(,)? }) => {
            const SYNTAX_FIELDS: &[&str] = &[$(stringify!($field)),+];
        };
    }
    crate::palette::syntax_fields!(syntax_known);

    let mut unknowns = Vec::new();

    for (section_name, section, known) in checks {
        for key in section.keys() {
            if !known.contains(&key.as_ref()) {
                unknowns.push(UnknownField {
                    section: Box::from(*section_name),
                    field: Box::from(key.as_ref()),
                });
            }
        }
    }

    // Check syntax section
    for key in manifest.syntax.keys() {
        if !SYNTAX_FIELDS.contains(&key.as_ref()) {
            unknowns.push(UnknownField {
                section: Box::from("syntax"),
                field: Box::from(key.as_ref()),
            });
        }
    }

    // Check syntax_style section (same field names as syntax)
    for key in manifest.syntax_style.keys() {
        if !SYNTAX_FIELDS.contains(&key.as_ref()) {
            unknowns.push(UnknownField {
                section: Box::from("syntax_style"),
                field: Box::from(key.as_ref()),
            });
        }
    }

    unknowns
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
    #[cfg(feature = "platform")]
    #[serde(default)]
    platform: PlatformSections,
}
