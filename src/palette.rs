use std::sync::Arc;

use crate::color::{Color, InvalidHex};
use crate::error::PaletteError;
use crate::manifest::{ManifestSection, PaletteManifest};

fn resolve_color(
    section: &ManifestSection,
    section_name: &str,
    field: &str,
) -> Result<Option<Color>, PaletteError> {
    match section.get(field) {
        None => Ok(None),
        Some(hex) => Color::from_hex(hex).map(Some).map_err(|InvalidHex { value }| {
            PaletteError::InvalidHex {
                section: Arc::from(section_name),
                field: Arc::from(field),
                value,
            }
        }),
    }
}

macro_rules! color_group {
    ($(#[$meta:meta])* $name:ident { $($field:ident),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
        pub struct $name {
            $(pub $field: Option<Color>,)+
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

color_group!(BaseColors {
    background,
    background_dark,
    background_highlight,
    foreground,
    foreground_dark,
    border,
    border_highlight,
});

color_group!(SemanticColors {
    success,
    warning,
    error,
    info,
    hint,
});

color_group!(DiffColors {
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

color_group!(SurfaceColors {
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

color_group!(TypographyColors {
    comment,
    gutter,
    line_number,
    selection_text,
    link,
    title,
});

color_group!(SyntaxColors {
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

color_group!(EditorColors {
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

color_group!(TerminalAnsiColors {
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
pub struct PaletteMeta {
    pub name: Arc<str>,
    pub preset_id: Arc<str>,
    pub style: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
pub struct Palette {
    pub meta: Option<PaletteMeta>,
    pub base: BaseColors,
    pub semantic: SemanticColors,
    pub diff: DiffColors,
    pub surface: SurfaceColors,
    pub typography: TypographyColors,
    pub syntax: SyntaxColors,
    pub editor: EditorColors,
    pub terminal_ansi: TerminalAnsiColors,
    #[cfg(feature = "platform")]
    pub platform: crate::platform::PlatformOverrides,
}

impl Palette {
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
