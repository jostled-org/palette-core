use std::sync::Arc;

use crate::color::Color;
use crate::error::PaletteError;
use crate::manifest::{ManifestSection, PaletteManifest};

fn resolve_color(
    section: &ManifestSection,
    section_name: &str,
    field: &str,
) -> Result<Option<Color>, PaletteError> {
    match section.get(field) {
        None => Ok(None),
        Some(hex) => Color::from_hex(hex)
            .map(Some)
            .map_err(|e| e.into_palette_error(Arc::from(section_name), Arc::from(field))),
    }
}

macro_rules! color_group {
    ($(#[$meta:meta])* $name:ident { $($field:ident),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        #[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
        pub struct $name {
            $(
                #[doc = concat!("`", stringify!($field), "` slot.")]
                pub $field: Option<Color>,
            )+
        }

        impl $name {
            fn from_section(
                section: &ManifestSection,
                section_name: &str,
            ) -> Result<Self, PaletteError> {
                Ok(Self {
                    $($field: resolve_color(section, section_name, stringify!($field))?,)+
                })
            }

            /// Merge two groups, preferring `self` values over `fallback`.
            pub fn merge(&self, fallback: &Self) -> Self {
                Self {
                    $($field: self.$field.or(fallback.$field),)+
                }
            }

            /// Iterate over slots that have a color assigned.
            pub fn populated_slots(&self) -> impl Iterator<Item = (&'static str, &Color)> {
                [$(
                    (stringify!($field), self.$field.as_ref()),
                )+]
                .into_iter()
                .filter_map(|(name, color)| color.map(|c| (name, c)))
            }
        }
    };
}

/// Single source of truth for color group field lists.
///
/// Invokes `$macro_name!` once per group, passing the struct name and its
/// fields. Both `color_group!` and `terminal_group!` consume this so
/// additions stay in sync at compile time.
macro_rules! color_fields {
    ($macro_name:ident) => {
        $macro_name!(
            /// Core background, foreground, and border colors.
            BaseColors {
            background,
            background_dark,
            background_highlight,
            foreground,
            foreground_dark,
            border,
            border_highlight,
        });

        $macro_name!(
            /// Status colors: success, warning, error, info, hint.
            SemanticColors {
            success,
            warning,
            error,
            info,
            hint,
        });

        $macro_name!(
            /// Version-control diff highlighting: added, modified, removed.
            DiffColors {
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
        });

        $macro_name!(
            /// UI surface colors: menus, sidebars, popups, overlays.
            SurfaceColors {
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
        });

        $macro_name!(
            /// Text chrome: comments, gutter, line numbers, links.
            TypographyColors {
            comment,
            gutter,
            line_number,
            selection_text,
            link,
            title,
        });

        $macro_name!(
            /// Syntax-highlighting token colors.
            SyntaxColors {
            keywords,
            keywords_fn,
            functions,
            variables,
            variables_builtin,
            parameters,
            properties,
            types,
            types_builtin,
            constants,
            numbers,
            booleans,
            strings,
            strings_doc,
            strings_escape,
            strings_regex,
            operators,
            punctuation,
            punctuation_bracket,
            annotations,
            attributes,
            constructor,
            tag,
            tag_delimiter,
            tag_attribute,
            comments,
        });

        $macro_name!(
            /// Editor chrome: cursor, selections, diagnostics, inlay hints.
            EditorColors {
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
        });

        $macro_name!(
            /// Standard 16-color ANSI terminal palette.
            TerminalAnsiColors {
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
        });
    };
}

color_fields!(color_group);
pub(crate) use color_fields;

/// Theme identity: name, preset ID, and style tag (e.g. "dark", "light").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
pub struct PaletteMeta {
    /// Human-readable theme name.
    pub name: Arc<str>,
    /// Machine identifier used for lookups.
    pub preset_id: Arc<str>,
    /// Visual style tag: `"dark"`, `"light"`, etc.
    pub style: Arc<str>,
}

/// Resolved color palette ready for rendering.
///
/// Built from a [`PaletteManifest`] (parsed TOML) via [`Palette::from_manifest`],
/// or obtained directly from [`preset`](crate::preset), [`load_preset`](crate::load_preset),
/// or [`Registry::load`](crate::Registry::load). Each field is a color group
/// whose slots are `Option<Color>` — absent slots mean the theme defers to
/// the renderer's default.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
pub struct Palette {
    /// Theme identity, if parsed from a manifest with `[meta]`.
    pub meta: Option<PaletteMeta>,
    /// Core background and foreground colors.
    pub base: BaseColors,
    /// Status colors (success, warning, error, info, hint).
    pub semantic: SemanticColors,
    /// Version-control diff highlighting.
    pub diff: DiffColors,
    /// UI surface colors (menus, sidebars, overlays).
    pub surface: SurfaceColors,
    /// Text chrome (comments, line numbers, links).
    pub typography: TypographyColors,
    /// Syntax-highlighting token colors.
    pub syntax: SyntaxColors,
    /// Editor chrome (cursor, selections, diagnostics).
    pub editor: EditorColors,
    /// Standard 16-color ANSI terminal palette.
    pub terminal_ansi: TerminalAnsiColors,
    /// Per-platform color overrides.
    #[cfg(feature = "platform")]
    pub platform: crate::platform::PlatformOverrides,
}

const fn c(r: u8, g: u8, b: u8) -> Option<Color> {
    Some(Color { r, g, b })
}

impl Default for Palette {
    /// Neutral dark palette with enough colors for legible rendering.
    ///
    /// Covers base, semantic, and surface slots. Syntax, editor, terminal,
    /// and diff slots are `None` — downstream renderers should apply their
    /// own defaults for those.
    fn default() -> Self {
        Self {
            meta: None,
            base: BaseColors {
                background: c(0x1a, 0x1a, 0x2e),
                background_dark: c(0x13, 0x13, 0x22),
                background_highlight: c(0x24, 0x24, 0x3e),
                foreground: c(0xd0, 0xd0, 0xd0),
                foreground_dark: c(0x80, 0x80, 0x90),
                border: c(0x3a, 0x3a, 0x4e),
                border_highlight: c(0x50, 0x50, 0x68),
            },
            semantic: SemanticColors {
                success: c(0x50, 0xc8, 0x78),
                warning: c(0xe0, 0xb0, 0x50),
                error: c(0xe0, 0x50, 0x50),
                info: c(0x50, 0x90, 0xe0),
                hint: c(0x70, 0x70, 0x88),
            },
            surface: SurfaceColors {
                menu: c(0x1e, 0x1e, 0x32),
                sidebar: c(0x1e, 0x1e, 0x32),
                statusline: c(0x1e, 0x1e, 0x32),
                float: c(0x1e, 0x1e, 0x32),
                popup: c(0x1e, 0x1e, 0x32),
                overlay: c(0x1e, 0x1e, 0x32),
                highlight: c(0x2a, 0x2a, 0x44),
                selection: c(0x30, 0x30, 0x50),
                focus: c(0x50, 0x90, 0xe0),
                search: c(0xe0, 0xb0, 0x50),
            },
            diff: DiffColors {
                added: c(0x50, 0xc8, 0x78),
                added_bg: c(0x1e, 0x33, 0x2a),
                added_fg: c(0x50, 0xc8, 0x78),
                modified: c(0xe0, 0xb0, 0x50),
                modified_bg: c(0x33, 0x2e, 0x1e),
                modified_fg: c(0xe0, 0xb0, 0x50),
                removed: c(0xe0, 0x50, 0x50),
                removed_bg: c(0x33, 0x1e, 0x1e),
                removed_fg: c(0xe0, 0x50, 0x50),
                text_bg: c(0x30, 0x30, 0x50),
                ignored: c(0x70, 0x70, 0x88),
            },
            typography: TypographyColors {
                comment: c(0x70, 0x70, 0x88),
                gutter: c(0x40, 0x40, 0x58),
                line_number: c(0x50, 0x50, 0x68),
                selection_text: c(0xd0, 0xd0, 0xd0),
                link: c(0x50, 0x90, 0xe0),
                title: c(0xd0, 0xd0, 0xd0),
            },
            syntax: SyntaxColors {
                keywords: c(0xc0, 0x80, 0xe0),
                keywords_fn: c(0xc0, 0x80, 0xe0),
                functions: c(0x60, 0xa0, 0xe0),
                variables: c(0xd0, 0xd0, 0xd0),
                variables_builtin: c(0xe0, 0x80, 0x80),
                parameters: c(0xd0, 0xb0, 0x80),
                properties: c(0x80, 0xc0, 0xe0),
                types: c(0x60, 0xd0, 0xb0),
                types_builtin: c(0x60, 0xd0, 0xb0),
                constants: c(0xe0, 0xb0, 0x50),
                numbers: c(0xe0, 0xb0, 0x50),
                booleans: c(0xe0, 0xb0, 0x50),
                strings: c(0x80, 0xc8, 0x80),
                strings_doc: c(0x70, 0x70, 0x88),
                strings_escape: c(0xe0, 0x80, 0x80),
                strings_regex: c(0xe0, 0x80, 0x80),
                operators: c(0xd0, 0xd0, 0xd0),
                punctuation: c(0xa0, 0xa0, 0xb0),
                punctuation_bracket: c(0xa0, 0xa0, 0xb0),
                annotations: c(0xe0, 0xb0, 0x50),
                attributes: c(0xe0, 0xb0, 0x50),
                constructor: c(0x60, 0xd0, 0xb0),
                tag: c(0xe0, 0x80, 0x80),
                tag_delimiter: c(0xa0, 0xa0, 0xb0),
                tag_attribute: c(0xd0, 0xb0, 0x80),
                comments: c(0x70, 0x70, 0x88),
            },
            editor: EditorColors {
                cursor: c(0xd0, 0xd0, 0xd0),
                cursor_text: c(0x1a, 0x1a, 0x2e),
                match_paren: c(0xe0, 0xb0, 0x50),
                selection_bg: c(0x30, 0x30, 0x50),
                selection_fg: c(0xd0, 0xd0, 0xd0),
                inlay_hint_bg: c(0x24, 0x24, 0x3e),
                inlay_hint_fg: c(0x70, 0x70, 0x88),
                search_bg: c(0x50, 0x40, 0x20),
                search_fg: c(0xe0, 0xb0, 0x50),
                diagnostic_error: c(0xe0, 0x50, 0x50),
                diagnostic_warn: c(0xe0, 0xb0, 0x50),
                diagnostic_info: c(0x50, 0x90, 0xe0),
                diagnostic_hint: c(0x70, 0x70, 0x88),
                diagnostic_underline_error: c(0xe0, 0x50, 0x50),
                diagnostic_underline_warn: c(0xe0, 0xb0, 0x50),
                diagnostic_underline_info: c(0x50, 0x90, 0xe0),
                diagnostic_underline_hint: c(0x70, 0x70, 0x88),
            },
            terminal_ansi: TerminalAnsiColors {
                black: c(0x1a, 0x1a, 0x2e),
                red: c(0xe0, 0x50, 0x50),
                green: c(0x50, 0xc8, 0x78),
                yellow: c(0xe0, 0xb0, 0x50),
                blue: c(0x50, 0x90, 0xe0),
                magenta: c(0xc0, 0x80, 0xe0),
                cyan: c(0x60, 0xd0, 0xb0),
                white: c(0xd0, 0xd0, 0xd0),
                bright_black: c(0x50, 0x50, 0x68),
                bright_red: c(0xf0, 0x70, 0x70),
                bright_green: c(0x70, 0xe0, 0x98),
                bright_yellow: c(0xf0, 0xc8, 0x70),
                bright_blue: c(0x70, 0xb0, 0xf0),
                bright_magenta: c(0xd8, 0xa0, 0xf0),
                bright_cyan: c(0x80, 0xe8, 0xd0),
                bright_white: c(0xf0, 0xf0, 0xf0),
            },
            #[cfg(feature = "platform")]
            platform: crate::platform::PlatformOverrides::default(),
        }
    }
}

impl Palette {
    /// Build a palette from a parsed manifest, resolving hex strings to [`Color`] values.
    pub fn from_manifest(manifest: &PaletteManifest) -> Result<Self, PaletteError> {
        let meta = manifest.meta.as_ref().map(|m| PaletteMeta {
            name: Arc::clone(&m.name),
            preset_id: Arc::clone(&m.preset_id),
            style: Arc::clone(&m.style),
        });

        Ok(Self {
            meta,
            base: BaseColors::from_section(&manifest.base, "base")?,
            semantic: SemanticColors::from_section(&manifest.semantic, "semantic")?,
            diff: DiffColors::from_section(&manifest.diff, "diff")?,
            surface: SurfaceColors::from_section(&manifest.surface, "surface")?,
            typography: TypographyColors::from_section(&manifest.typography, "typography")?,
            syntax: SyntaxColors::from_section(&manifest.syntax, "syntax")?,
            editor: EditorColors::from_section(&manifest.editor, "editor")?,
            terminal_ansi: TerminalAnsiColors::from_section(&manifest.terminal, "terminal")?,
            #[cfg(feature = "platform")]
            platform: crate::platform::from_sections(&manifest.platform)?,
        })
    }
}
