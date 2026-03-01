#![cfg(feature = "egui")]

use egui::Color32;

use palette_core::color::Color;
use palette_core::egui::{to_color32, to_egui_visuals};
use palette_core::palette::Palette;

mod common;

fn tokyonight_visuals() -> ::egui::Visuals {
    let manifest = common::load_preset("tokyonight");
    let palette = Palette::from_manifest(&manifest).unwrap();
    to_egui_visuals(&palette)
}

#[test]
fn single_color_converts_to_color32() {
    let color = Color {
        r: 26,
        g: 27,
        b: 42,
    };
    assert_eq!(to_color32(&color), Color32::from_rgb(26, 27, 42));
}

#[test]
fn panel_fill_matches_background() {
    let v = tokyonight_visuals();
    assert_eq!(v.panel_fill, Color32::from_rgb(26, 27, 42));
}

#[test]
fn window_fill_matches_background() {
    let v = tokyonight_visuals();
    assert_eq!(v.window_fill, v.panel_fill);
}

#[test]
fn error_fg_maps_semantic_error() {
    let v = tokyonight_visuals();
    assert_eq!(v.error_fg_color, Color32::from_rgb(219, 75, 75));
}

#[test]
fn selection_bg_maps_surface_selection() {
    let v = tokyonight_visuals();
    assert_eq!(v.selection.bg_fill, Color32::from_rgb(40, 52, 87));
}

#[test]
fn window_stroke_maps_border() {
    let v = tokyonight_visuals();
    // border = "#15161e" => (21, 22, 30)
    assert_eq!(v.window_stroke.color, Color32::from_rgb(21, 22, 30));
}

#[test]
fn text_cursor_maps_editor_cursor() {
    let v = tokyonight_visuals();
    // cursor = "#c0caf5" => (192, 202, 245)
    assert_eq!(v.text_cursor.stroke.color, Color32::from_rgb(192, 202, 245));
}

#[test]
fn code_bg_maps_background_dark() {
    let v = tokyonight_visuals();
    // background_dark = "#16161e" => (22, 22, 30)
    assert_eq!(v.code_bg_color, Color32::from_rgb(22, 22, 30));
}

#[test]
fn hovered_stroke_maps_border_highlight() {
    let v = tokyonight_visuals();
    // border_highlight = "#27a1b9" => (39, 161, 185)
    assert_eq!(
        v.widgets.hovered.bg_stroke.color,
        Color32::from_rgb(39, 161, 185)
    );
}

#[test]
fn weak_text_maps_foreground_dark() {
    let v = tokyonight_visuals();
    // foreground_dark = "#a9b1d6" => (169, 177, 214)
    assert_eq!(v.weak_text_color, Some(Color32::from_rgb(169, 177, 214)));
}
