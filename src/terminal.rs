//! Ratatui integration: convert a [`Palette`] into terminal-native colors.

use ratatui::style::Color as RatatuiColor;
use ratatui::style::Modifier;

use crate::color::Color;
use crate::palette::Palette;
use crate::style::{ResolvedSyntaxStyles, StyleModifiers, SyntaxStyles};

/// Convert a [`Color`] to a ratatui RGB color.
pub fn to_ratatui_color(color: &Color) -> RatatuiColor {
    RatatuiColor::Rgb(color.r, color.g, color.b)
}

macro_rules! terminal_group {
    ($(#[$_meta:meta])* $color_type:ident { $($field:ident),+ $(,)? }) => {
        pastey::paste! {
            #[doc = concat!("Ratatui-native version of [`", stringify!($color_type), "`](crate::palette::", stringify!($color_type), ").")]
            #[derive(Debug, Clone)]
            pub struct [<Terminal $color_type>] {
                $(
                    #[doc = concat!("`", stringify!($field), "` slot.")]
                    pub $field: Option<RatatuiColor>,
                )+
            }

            impl [<Terminal $color_type>] {
                fn from_palette(group: &crate::palette::$color_type) -> Self {
                    Self {
                        $($field: group.$field.map(|c| to_ratatui_color(&c)),)+
                    }
                }
            }
        }
    };
}

macro_rules! resolved_terminal_group {
    ($(#[$_meta:meta])* $color_type:ident { $($field:ident),+ $(,)? }) => {
        pastey::paste! {
            #[doc = concat!("Resolved ratatui-native version of [`", stringify!($color_type), "`](crate::palette::", stringify!($color_type), ").")]
            #[derive(Debug, Clone)]
            pub struct [<ResolvedTerminal $color_type>] {
                $(
                    #[doc = concat!("`", stringify!($field), "` slot.")]
                    pub $field: RatatuiColor,
                )+
            }

            impl [<ResolvedTerminal $color_type>] {
                fn from_resolved(group: &crate::resolved::[<Resolved $color_type>]) -> Self {
                    Self {
                        $($field: to_ratatui_color(&group.$field),)+
                    }
                }
            }
        }
    };
}

crate::palette::color_fields!(terminal_group);
crate::palette::color_fields!(resolved_terminal_group);

/// Convert a [`StyleModifiers`] to a ratatui [`Modifier`] bitset.
pub fn to_ratatui_modifier(style: &StyleModifiers) -> Modifier {
    let mut m = Modifier::empty();
    if style.bold {
        m |= Modifier::BOLD;
    }
    if style.italic {
        m |= Modifier::ITALIC;
    }
    if style.underline {
        m |= Modifier::UNDERLINED;
    }
    m
}

/// Ratatui-native syntax style modifiers, keyed by syntax field name.
#[derive(Debug, Clone)]
pub struct TerminalSyntaxStyles {
    /// Populated style modifier slots.
    slots: Box<[(&'static str, Modifier)]>,
}

impl TerminalSyntaxStyles {
    fn from_palette(styles: &SyntaxStyles) -> Self {
        let slots: Vec<_> = styles
            .populated_slots()
            .map(|(name, s)| (name, to_ratatui_modifier(s)))
            .collect();
        Self {
            slots: slots.into_boxed_slice(),
        }
    }

    /// Iterate over populated style slots.
    pub fn populated_slots(&self) -> impl Iterator<Item = (&'static str, Modifier)> + '_ {
        self.slots.iter().copied()
    }
}

/// Resolved ratatui-native syntax style modifiers (every slot a concrete [`Modifier`]).
#[derive(Debug, Clone)]
pub struct ResolvedTerminalSyntaxStyles {
    /// All 38 resolved style modifier slots.
    slots: Box<[(&'static str, Modifier)]>,
}

impl ResolvedTerminalSyntaxStyles {
    fn from_resolved(styles: &ResolvedSyntaxStyles) -> Self {
        let slots: Vec<_> = styles
            .all_slots()
            .map(|(name, s)| (name, to_ratatui_modifier(s)))
            .collect();
        Self {
            slots: slots.into_boxed_slice(),
        }
    }

    /// Iterate over all style slots.
    pub fn all_slots(&self) -> impl Iterator<Item = (&'static str, Modifier)> + '_ {
        self.slots.iter().copied()
    }
}

/// Complete ratatui-native theme mirroring every [`Palette`] color group.
#[derive(Debug, Clone)]
pub struct TerminalTheme {
    /// Core background and foreground colors.
    pub base: TerminalBaseColors,
    /// Status colors (success, warning, error, info, hint).
    pub semantic: TerminalSemanticColors,
    /// Version-control diff highlighting.
    pub diff: TerminalDiffColors,
    /// UI surface colors (menus, sidebars, overlays).
    pub surface: TerminalSurfaceColors,
    /// Text chrome (comments, line numbers, links).
    pub typography: TerminalTypographyColors,
    /// Syntax-highlighting token colors.
    pub syntax: TerminalSyntaxColors,
    /// Editor chrome (cursor, selections, diagnostics).
    pub editor: TerminalEditorColors,
    /// Standard 16-color ANSI terminal palette.
    pub terminal: TerminalAnsiColors,
    /// Syntax token style modifiers.
    pub syntax_style: TerminalSyntaxStyles,
}

/// Convert an entire [`Palette`] into a [`TerminalTheme`].
pub fn to_terminal_theme(palette: &Palette) -> TerminalTheme {
    TerminalTheme {
        base: TerminalBaseColors::from_palette(&palette.base),
        semantic: TerminalSemanticColors::from_palette(&palette.semantic),
        diff: TerminalDiffColors::from_palette(&palette.diff),
        surface: TerminalSurfaceColors::from_palette(&palette.surface),
        typography: TerminalTypographyColors::from_palette(&palette.typography),
        syntax: TerminalSyntaxColors::from_palette(&palette.syntax),
        editor: TerminalEditorColors::from_palette(&palette.editor),
        terminal: TerminalAnsiColors::from_palette(&palette.terminal),
        syntax_style: TerminalSyntaxStyles::from_palette(&palette.syntax_style),
    }
}

/// Resolved ratatui-native theme where every slot is a concrete [`RatatuiColor`].
#[derive(Debug, Clone)]
pub struct ResolvedTerminalTheme {
    /// Core background and foreground colors.
    pub base: ResolvedTerminalBaseColors,
    /// Status colors (success, warning, error, info, hint).
    pub semantic: ResolvedTerminalSemanticColors,
    /// Version-control diff highlighting.
    pub diff: ResolvedTerminalDiffColors,
    /// UI surface colors (menus, sidebars, overlays).
    pub surface: ResolvedTerminalSurfaceColors,
    /// Text chrome (comments, line numbers, links).
    pub typography: ResolvedTerminalTypographyColors,
    /// Syntax-highlighting token colors.
    pub syntax: ResolvedTerminalSyntaxColors,
    /// Editor chrome (cursor, selections, diagnostics).
    pub editor: ResolvedTerminalEditorColors,
    /// Standard 16-color ANSI terminal palette.
    pub terminal: ResolvedTerminalAnsiColors,
    /// Syntax token style modifiers.
    pub syntax_style: ResolvedTerminalSyntaxStyles,
}

impl ResolvedTerminalAnsiColors {
    /// The 12 chromatic ANSI colors, skipping black/white/bright_black/bright_white.
    ///
    /// Useful for cycling distinct accent colors across panels, tabs, or chart series.
    pub fn chromatic(&self) -> [RatatuiColor; 12] {
        [
            self.red,
            self.green,
            self.yellow,
            self.blue,
            self.magenta,
            self.cyan,
            self.bright_red,
            self.bright_green,
            self.bright_yellow,
            self.bright_blue,
            self.bright_magenta,
            self.bright_cyan,
        ]
    }
}

/// Convert a [`ResolvedPalette`](crate::resolved::ResolvedPalette) into a [`ResolvedTerminalTheme`].
pub fn to_resolved_terminal_theme(
    resolved: &crate::resolved::ResolvedPalette,
) -> ResolvedTerminalTheme {
    ResolvedTerminalTheme {
        base: ResolvedTerminalBaseColors::from_resolved(&resolved.base),
        semantic: ResolvedTerminalSemanticColors::from_resolved(&resolved.semantic),
        diff: ResolvedTerminalDiffColors::from_resolved(&resolved.diff),
        surface: ResolvedTerminalSurfaceColors::from_resolved(&resolved.surface),
        typography: ResolvedTerminalTypographyColors::from_resolved(&resolved.typography),
        syntax: ResolvedTerminalSyntaxColors::from_resolved(&resolved.syntax),
        editor: ResolvedTerminalEditorColors::from_resolved(&resolved.editor),
        terminal: ResolvedTerminalAnsiColors::from_resolved(&resolved.terminal),
        syntax_style: ResolvedTerminalSyntaxStyles::from_resolved(&resolved.syntax_style),
    }
}
