#![cfg(feature = "terminal")]

use std::collections::BTreeMap;
use std::sync::Arc;

use ratatui::style::Color as RatatuiColor;

use palette_core::color::Color;
use palette_core::palette::Palette;
use palette_core::terminal::{to_ratatui_color, to_terminal_theme};

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
    let manifest = common::manifest_with_base(BTreeMap::from([(
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
    assert!(theme.terminal_ansi.black.is_none());
}

#[test]
fn terminal_ansi_maps_all_16_colors() {
    let manifest = common::load_preset("tokyonight");
    let palette = Palette::from_manifest(&manifest).unwrap();
    let theme = to_terminal_theme(&palette);

    assert_eq!(
        theme.terminal_ansi.black,
        Some(RatatuiColor::Rgb(21, 22, 30)),
    );
    // Verify all 16 ANSI slots are populated
    assert!(theme.terminal_ansi.red.is_some());
    assert!(theme.terminal_ansi.green.is_some());
    assert!(theme.terminal_ansi.yellow.is_some());
    assert!(theme.terminal_ansi.blue.is_some());
    assert!(theme.terminal_ansi.magenta.is_some());
    assert!(theme.terminal_ansi.cyan.is_some());
    assert!(theme.terminal_ansi.white.is_some());
    assert!(theme.terminal_ansi.bright_black.is_some());
    assert!(theme.terminal_ansi.bright_red.is_some());
    assert!(theme.terminal_ansi.bright_green.is_some());
    assert!(theme.terminal_ansi.bright_yellow.is_some());
    assert!(theme.terminal_ansi.bright_blue.is_some());
    assert!(theme.terminal_ansi.bright_magenta.is_some());
    assert!(theme.terminal_ansi.bright_cyan.is_some());
    assert!(theme.terminal_ansi.bright_white.is_some());
}
