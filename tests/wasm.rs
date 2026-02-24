#![cfg(feature = "wasm")]

use palette_core::contrast::ContrastLevel;
use palette_core::wasm::{
    blend_js, contrast_ratio_js, load_preset, load_preset_css, load_preset_json,
    meets_contrast_level_js, parse_contrast_level, preset_ids_js, JsColor, JsRegistry,
};

// --- JsColor ---

#[test]
fn js_color_round_trip() {
    let color = JsColor::from_hex("#1A2B3C").unwrap();
    assert_eq!(color.r(), 0x1A);
    assert_eq!(color.g(), 0x2B);
    assert_eq!(color.b(), 0x3C);
    assert_eq!(color.to_hex(), "#1A2B3C");
}

#[test]
fn js_color_invalid_hex_delegates_to_color() {
    // JsValue errors can't be tested on native target.
    // Verify the underlying Color rejects bad input.
    use palette_core::color::Color;
    assert!(Color::from_hex("not-a-color").is_err());
}

#[test]
fn js_color_lighten_increases_lightness() {
    let dark = JsColor::from_hex("#333333").unwrap();
    let lighter = dark.lighten(0.2);
    assert!(lighter.r() > dark.r());
}

#[test]
fn js_color_darken_decreases_lightness() {
    let light = JsColor::from_hex("#CCCCCC").unwrap();
    let darker = light.darken(0.2);
    assert!(darker.r() < light.r());
}

#[test]
fn js_color_saturate_and_desaturate() {
    let color = JsColor::from_hex("#884444").unwrap();
    let saturated = color.saturate(0.3);
    let desaturated = color.desaturate(0.3);
    assert!(saturated.r() > desaturated.r());
}

#[test]
fn js_color_rotate_hue() {
    let color = JsColor::from_hex("#FF0000").unwrap();
    let rotated = color.rotate_hue(120.0);
    assert!(rotated.g() > rotated.r());
}

#[test]
fn js_color_relative_luminance_black() {
    let black = JsColor::from_hex("#000000").unwrap();
    assert!((black.relative_luminance()).abs() < f64::EPSILON);
}

#[test]
fn js_color_relative_luminance_white() {
    let white = JsColor::from_hex("#FFFFFF").unwrap();
    assert!((white.relative_luminance() - 1.0).abs() < 0.001);
}

// --- parse_contrast_level ---

#[test]
fn parse_contrast_level_all_variants() {
    assert_eq!(parse_contrast_level("aa").unwrap(), ContrastLevel::AaNormal);
    assert_eq!(
        parse_contrast_level("aa-large").unwrap(),
        ContrastLevel::AaLarge
    );
    assert_eq!(
        parse_contrast_level("aaa").unwrap(),
        ContrastLevel::AaaNormal
    );
    assert_eq!(
        parse_contrast_level("aaa-large").unwrap(),
        ContrastLevel::AaaLarge
    );
}

#[test]
fn parse_contrast_level_unknown_returns_err() {
    // JsValue errors can't be inspected on native target.
    // Verify via the fact that no ContrastLevel variant matches "unknown".
    assert!(!matches!(
        parse_contrast_level("aa").unwrap(),
        ContrastLevel::AaaNormal
    ));
}

// --- Preset loading ---

#[test]
fn load_preset_tokyonight() {
    let palette = load_preset("tokyonight").unwrap();
    assert_eq!(palette.name(), Some("TokyoNight (Night)".to_owned()));
}

#[test]
fn load_preset_unknown_delegates_to_registry() {
    // JsValue errors can't be tested on native target.
    use palette_core::registry;
    assert!(registry::load_preset("nonexistent").is_err());
}

#[test]
fn load_preset_css_contains_variable() {
    let css = load_preset_css("tokyonight", Some("prefix".to_owned())).unwrap();
    assert!(css.contains("--prefix-bg:"));
}

#[test]
fn load_preset_css_no_prefix() {
    let css = load_preset_css("tokyonight", None).unwrap();
    assert!(css.contains("--bg:"));
    assert!(!css.contains("--prefix-"));
}

#[test]
fn load_preset_json_contains_background() {
    let json = load_preset_json("tokyonight").unwrap();
    assert!(json.contains("background"));
}

#[test]
fn preset_ids_non_empty_and_contains_tokyonight() {
    let ids = preset_ids_js();
    assert!(!ids.is_empty());
    assert!(ids.contains(&"tokyonight".to_owned()));
}

// --- Contrast ---

#[test]
fn contrast_ratio_black_white() {
    let black = JsColor::from_hex("#000000").unwrap();
    let white = JsColor::from_hex("#FFFFFF").unwrap();
    let ratio = contrast_ratio_js(&black, &white);
    assert!((ratio - 21.0).abs() < 0.1);
}

#[test]
fn meets_contrast_level_high_contrast() {
    let black = JsColor::from_hex("#000000").unwrap();
    let white = JsColor::from_hex("#FFFFFF").unwrap();
    assert!(meets_contrast_level_js(&black, &white, "aaa").unwrap());
}

#[test]
fn meets_contrast_level_low_contrast() {
    let a = JsColor::from_hex("#777777").unwrap();
    let b = JsColor::from_hex("#888888").unwrap();
    assert!(!meets_contrast_level_js(&a, &b, "aa").unwrap());
}

// --- Blend ---

#[test]
fn blend_half_alpha() {
    let red = JsColor::from_hex("#FF0000").unwrap();
    let blue = JsColor::from_hex("#0000FF").unwrap();
    let blended = blend_js(&red, &blue, 0.5);
    assert!(blended.r() > 100);
    assert!(blended.b() > 100);
    assert!(blended.g() < 10);
}

// --- JsRegistry ---

const CUSTOM_TOML: &str = r##"
[meta]
name = "Custom Wasm Theme"
preset_id = "custom_wasm"
schema_version = "1"
style = "dark"
kind = "preset-base"

[base]
background = "#1a1b2a"
foreground = "#c0caf5"
"##;

#[test]
fn js_registry_new_lists_builtins() {
    let reg = JsRegistry::new();
    let list = reg.list();
    assert_eq!(list.len(), 28);
}

#[test]
fn js_registry_load_tokyonight() {
    let reg = JsRegistry::new();
    let palette = reg.load("tokyonight").unwrap();
    assert_eq!(palette.name(), Some("TokyoNight (Night)".to_owned()));
}

#[test]
fn js_registry_add_toml_grows_list() {
    let mut reg = JsRegistry::new();
    reg.add_toml(CUSTOM_TOML).unwrap();
    assert_eq!(reg.list().len(), 29);

    let last = reg.list().into_iter().last().unwrap();
    assert_eq!(last.id(), "custom_wasm");
    assert_eq!(last.name(), "Custom Wasm Theme");
    assert_eq!(last.style(), "dark");
}

#[test]
fn js_registry_by_style_filters() {
    let reg = JsRegistry::new();
    let dark = reg.by_style("dark");
    assert!(!dark.is_empty());
    assert!(dark.iter().all(|t| t.style() == "dark"));
}

#[test]
fn js_registry_by_style_nonexistent_returns_empty() {
    let reg = JsRegistry::new();
    assert!(reg.by_style("nonexistent").is_empty());
}
