#![cfg(feature = "terminal")]

use std::collections::HashMap;
use std::sync::Arc;

use ratatui::style::Color as RatatuiColor;

use palette_core::color::Color;
use palette_core::palette::Palette;
use palette_core::terminal::{
    style, to_ratatui_color, to_resolved_terminal_theme, to_terminal_theme,
};

mod common;

#[test]
fn single_color_converts_rgb() {
    let color = Color {
        r: 26,
        g: 27,
        b: 42,
    };
    assert_eq!(to_ratatui_color(&color), RatatuiColor::Rgb(26, 27, 42));
}

#[test]
fn base_background_matches_source() {
    let manifest = common::load_preset("tokyonight");
    let palette = Palette::from_manifest(&manifest).unwrap();
    let theme = to_terminal_theme(&palette);

    assert_eq!(theme.base.background, Some(RatatuiColor::Rgb(26, 27, 42)),);
}

#[test]
fn empty_sections_produce_none_fields() {
    let manifest = common::manifest_with_base(HashMap::from([(
        Arc::from("background"),
        Arc::from("#000000"),
    )]));
    let palette = Palette::from_manifest(&manifest).unwrap();
    let theme = to_terminal_theme(&palette);

    assert_eq!(theme.base.background, Some(RatatuiColor::Rgb(0, 0, 0)));
    assert!(theme.base.foreground.is_none());
    assert!(theme.semantic.success.is_none());
    assert!(theme.diff.added.is_none());
    assert!(theme.surface.menu.is_none());
    assert!(theme.typography.comment.is_none());
    assert!(theme.syntax.keywords.is_none());
    assert!(theme.editor.cursor.is_none());
    assert!(theme.terminal.black.is_none());
}

#[test]
fn terminal_maps_all_16_colors() {
    let manifest = common::load_preset("tokyonight");
    let palette = Palette::from_manifest(&manifest).unwrap();
    let theme = to_terminal_theme(&palette);

    assert_eq!(theme.terminal.black, Some(RatatuiColor::Rgb(21, 22, 30)),);
    // Verify all 16 ANSI slots are populated
    assert!(theme.terminal.red.is_some());
    assert!(theme.terminal.green.is_some());
    assert!(theme.terminal.yellow.is_some());
    assert!(theme.terminal.blue.is_some());
    assert!(theme.terminal.magenta.is_some());
    assert!(theme.terminal.cyan.is_some());
    assert!(theme.terminal.white.is_some());
    assert!(theme.terminal.bright_black.is_some());
    assert!(theme.terminal.bright_red.is_some());
    assert!(theme.terminal.bright_green.is_some());
    assert!(theme.terminal.bright_yellow.is_some());
    assert!(theme.terminal.bright_blue.is_some());
    assert!(theme.terminal.bright_magenta.is_some());
    assert!(theme.terminal.bright_cyan.is_some());
    assert!(theme.terminal.bright_white.is_some());
}

#[test]
fn resolved_terminal_theme_has_no_options() {
    let palette = Palette::from_manifest(&common::load_preset("tokyonight")).unwrap();
    let resolved = palette.resolve();
    let theme = to_resolved_terminal_theme(&resolved);

    // Every field is a bare RatatuiColor, no Option wrapper.
    assert_eq!(
        theme.base.background,
        RatatuiColor::Rgb(
            resolved.base.background.r,
            resolved.base.background.g,
            resolved.base.background.b
        )
    );
    assert_eq!(
        theme.terminal.black,
        RatatuiColor::Rgb(
            resolved.terminal.black.r,
            resolved.terminal.black.g,
            resolved.terminal.black.b
        )
    );
}

#[test]
fn resolved_terminal_matches_original_when_populated() {
    let palette = Palette::from_manifest(&common::load_preset("tokyonight")).unwrap();
    let original = to_terminal_theme(&palette);
    let resolved = to_resolved_terminal_theme(&palette.resolve());

    // For populated slots, resolved and original should match.
    assert_eq!(original.base.background.unwrap(), resolved.base.background);
    assert_eq!(original.base.foreground.unwrap(), resolved.base.foreground);
    assert_eq!(original.terminal.red.unwrap(), resolved.terminal.red);
}

#[test]
fn chromatic_returns_12_non_grayscale_colors() {
    let palette = Palette::from_manifest(&common::load_preset("tokyonight")).unwrap();
    let theme = to_resolved_terminal_theme(&palette.resolve());
    let colors = theme.terminal.chromatic();

    assert_eq!(colors.len(), 12);
    assert_eq!(colors[0], theme.terminal.red);
    assert_eq!(colors[11], theme.terminal.bright_cyan);
    assert!(!colors.contains(&theme.terminal.black));
    assert!(!colors.contains(&theme.terminal.white));
    assert!(!colors.contains(&theme.terminal.bright_black));
    assert!(!colors.contains(&theme.terminal.bright_white));
}

#[test]
fn style_builder_sets_fg_and_bg() {
    let fg = RatatuiColor::Rgb(0xC0, 0xCA, 0xF5);
    let bg = RatatuiColor::Rgb(0x1A, 0x1B, 0x2A);
    let s = style(fg, bg);

    assert_eq!(s.fg, Some(fg));
    assert_eq!(s.bg, Some(bg));
}
