use std::collections::BTreeMap;
use std::sync::Arc;

use palette_core::error::PaletteError;
use palette_core::manifest::PaletteManifest;
use palette_core::merge::merge_manifests;
use palette_core::palette::Palette;

mod common;

#[test]
fn full_base_resolves_all_colors() {
    let manifest = common::load_preset("tokyonight");
    let palette = Palette::from_manifest(&manifest).unwrap();

    assert!(palette.base.background.is_some());
    assert!(palette.base.background_dark.is_some());
    assert!(palette.base.background_highlight.is_some());
    assert!(palette.base.foreground.is_some());
    assert!(palette.base.foreground_dark.is_some());
    assert!(palette.base.border.is_some());
    assert!(palette.base.border_highlight.is_some());
}

#[test]
fn merged_variant_inherits() {
    let base = common::load_preset("tokyonight");
    let storm = common::load_preset("tokyonight_storm");
    let merged = merge_manifests(&storm, &base);
    let palette = Palette::from_manifest(&merged).unwrap();

    // Storm overrides background
    assert_eq!(palette.base.background.unwrap().to_hex(), "#24283B",);
    // Inherits terminal colors from base (except black)
    assert_eq!(palette.terminal_ansi.red.unwrap().to_hex(), "#F7768E",);
}

#[test]
fn sparse_section_yields_none() {
    let toml = r##"
[base]
background = "#000000"
"##;
    let manifest = PaletteManifest::from_toml(toml).unwrap();
    let palette = Palette::from_manifest(&manifest).unwrap();

    assert!(palette.base.background.is_some());
    assert!(palette.base.foreground.is_none());
    assert!(palette.syntax.keywords.is_none());
    assert!(palette.terminal_ansi.red.is_none());
}

#[test]
fn invalid_hex_returns_error() {
    let toml = r##"
[base]
background = "not-a-color"
"##;
    let manifest = PaletteManifest::from_toml(toml).unwrap();
    let err = Palette::from_manifest(&manifest).unwrap_err();

    assert!(
        matches!(
            &err,
            PaletteError::InvalidHex { section, field, value }
                if section.as_ref() == "base"
                && field.as_ref() == "background"
                && value.as_ref() == "not-a-color"
        ),
        "expected InvalidHex with context, got: {err:?}",
    );
}

#[test]
fn meta_propagates() {
    let manifest = common::load_preset("tokyonight");
    let palette = Palette::from_manifest(&manifest).unwrap();
    let meta = palette.meta.unwrap();

    assert_eq!(meta.name.as_ref(), "TokyoNight (Night)");
    assert_eq!(meta.preset_id.as_ref(), "tokyonight");
    assert_eq!(meta.style.as_ref(), "night");
}

#[test]
fn no_meta_yields_none() {
    let manifest = common::manifest_with_base(BTreeMap::from([(
        Arc::from("background"),
        Arc::from("#000000"),
    )]));
    let palette = Palette::from_manifest(&manifest).unwrap();
    assert!(palette.meta.is_none());
}

#[test]
fn default_has_legible_base_colors() {
    let palette = Palette::default();
    assert!(palette.meta.is_none());
    assert!(palette.base.background.is_some());
    assert!(palette.base.foreground.is_some());
    assert!(palette.base.border.is_some());
}

#[test]
fn default_has_semantic_colors() {
    let palette = Palette::default();
    assert!(palette.semantic.error.is_some());
    assert!(palette.semantic.warning.is_some());
    assert!(palette.semantic.success.is_some());
    assert!(palette.semantic.info.is_some());
}

#[test]
fn default_produces_valid_css() {
    let palette = Palette::default();
    let css = palette.to_css();
    assert!(css.contains("--bg:"));
    assert!(css.contains("--fg:"));
    assert!(css.contains("--error:"));
}
