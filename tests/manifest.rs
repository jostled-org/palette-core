use std::sync::Arc;

use palette_core::error::PaletteError;
use palette_core::manifest::PaletteManifest;

const BASE_TOML: &str = r##"
[meta]
name = "Test Theme"
preset_id = "test_theme"
schema_version = "1"
style = "dark"
kind = "preset-base"
upstream_repo = "https://github.com/example/theme"

[base]
background = "#1a1b2a"
foreground = "#c0caf5"

[semantic]
success = "#73daca"
error = "#db4b4b"

[diff]
added_bg = "#243e4a"

[surface]
menu = "#16161e"

[typography]
comment = "#565f89"

[syntax]
keywords = "#9d7cd8"

[editor]
cursor = "#c0caf5"

[terminal]
black = "#15161e"
red = "#f7768e"
"##;

const VARIANT_TOML: &str = r##"
[meta]
name = "Test Theme Storm"
preset_id = "test_theme_storm"
schema_version = "1"
style = "storm"
kind = "preset-variant"
inherits = "test_theme"

[base]
background = "#24283b"

[surface]
menu = "#1f2335"
"##;

#[test]
fn parse_full_base_preset() {
    let manifest = PaletteManifest::from_toml(BASE_TOML).unwrap();

    let meta = manifest.meta.as_ref().unwrap();
    assert_eq!(&*meta.name, "Test Theme");
    assert_eq!(&*meta.preset_id, "test_theme");
    assert_eq!(&*meta.schema_version, "1");
    assert_eq!(&*meta.style, "dark");
    assert_eq!(&*meta.kind, "preset-base");
    assert_eq!(
        meta.upstream_repo.as_deref(),
        Some("https://github.com/example/theme")
    );

    assert_eq!(manifest.base.len(), 2);
    assert_eq!(manifest.semantic.len(), 2);
    assert_eq!(manifest.diff.len(), 1);
    assert_eq!(manifest.surface.len(), 1);
    assert_eq!(manifest.typography.len(), 1);
    assert_eq!(manifest.syntax.len(), 1);
    assert_eq!(manifest.editor.len(), 1);
    assert_eq!(manifest.terminal.len(), 2);
}

#[test]
fn parse_sparse_variant() {
    let manifest = PaletteManifest::from_toml(VARIANT_TOML).unwrap();

    assert_eq!(manifest.base.len(), 1);
    assert_eq!(manifest.surface.len(), 1);
    assert!(manifest.semantic.is_empty());
    assert!(manifest.diff.is_empty());
    assert!(manifest.typography.is_empty());
    assert!(manifest.syntax.is_empty());
    assert!(manifest.editor.is_empty());
    assert!(manifest.terminal.is_empty());
}

#[test]
fn inherits_from_returns_parent_for_variant() {
    let manifest = PaletteManifest::from_toml(VARIANT_TOML).unwrap();
    assert_eq!(manifest.inherits_from(), Some("test_theme"));
}

#[test]
fn inherits_from_returns_none_for_base() {
    let manifest = PaletteManifest::from_toml(BASE_TOML).unwrap();
    assert_eq!(manifest.inherits_from(), None);
}

#[test]
fn missing_base_section_returns_error() {
    let toml = r##"
[meta]
name = "No Base"
preset_id = "no_base"
schema_version = "1"
style = "dark"
kind = "preset-base"
"##;

    let err = PaletteManifest::from_toml(toml).unwrap_err();
    assert!(matches!(err, PaletteError::MissingBase));
}

#[test]
fn empty_base_section_succeeds() {
    let toml = r##"
[meta]
name = "Empty Base"
preset_id = "empty_base"
schema_version = "1"
style = "dark"
kind = "preset-base"

[base]
"##;

    let manifest = PaletteManifest::from_toml(toml).unwrap();
    assert!(manifest.base.is_empty());
}

#[test]
fn unknown_sections_silently_ignored() {
    let toml = r##"
[meta]
name = "With Extras"
preset_id = "extras"
schema_version = "1"
style = "dark"
kind = "preset-base"

[base]
background = "#000000"

[git]
add = "#449dab"

[palette]
red = "#f7768e"
"##;

    let manifest = PaletteManifest::from_toml(toml).unwrap();
    assert_eq!(manifest.base.len(), 1);
}

#[test]
fn base_keys_accessible_via_btreemap_get() {
    let manifest = PaletteManifest::from_toml(BASE_TOML).unwrap();

    let bg: &Arc<str> = manifest.base.get("background").unwrap();
    assert_eq!(&**bg, "#1a1b2a");

    let fg: &Arc<str> = manifest.base.get("foreground").unwrap();
    assert_eq!(&**fg, "#c0caf5");
}

#[test]
fn real_preset_tokyonight_parses() {
    let toml = include_str!("../presets/tokyonight.toml");
    let manifest = PaletteManifest::from_toml(toml).unwrap();

    let meta = manifest.meta.as_ref().unwrap();
    assert_eq!(&*meta.preset_id, "tokyonight");
    assert_eq!(&*meta.kind, "preset-base");
    assert_eq!(manifest.inherits_from(), None);
    assert!(!manifest.base.is_empty());
    assert!(!manifest.terminal.is_empty());
}

#[test]
fn real_preset_tokyonight_storm_parses() {
    let toml = include_str!("../presets/tokyonight_storm.toml");
    let manifest = PaletteManifest::from_toml(toml).unwrap();

    let meta = manifest.meta.as_ref().unwrap();
    assert_eq!(&*meta.preset_id, "tokyonight_storm");
    assert_eq!(&*meta.kind, "preset-variant");
    assert_eq!(manifest.inherits_from(), Some("tokyonight"));
}

#[test]
fn gradient_section_parses_hex_stops() {
    let toml = r##"
[base]
background = "#000000"

[gradient.heat]
stops = ["#FF0000", "#00FF00", "#0000FF"]
"##;
    let manifest = PaletteManifest::from_toml(toml).unwrap();
    assert_eq!(manifest.gradient.len(), 1);
    let heat = manifest.gradient.get("heat").unwrap();
    assert_eq!(heat.stops.len(), 3);
}

#[test]
fn gradient_section_parses_explicit_positions() {
    let toml = r##"
[base]
background = "#000000"

[gradient.ramp]
stops = [
    { color = "#FF0000", at = 0.0 },
    { color = "#0000FF", at = 1.0 },
]
"##;
    let manifest = PaletteManifest::from_toml(toml).unwrap();
    let ramp = manifest.gradient.get("ramp").unwrap();
    assert_eq!(ramp.stops.len(), 2);
}

#[test]
fn gradient_section_parses_color_space() {
    let toml_oklch = r##"
[base]
background = "#000000"

[gradient.hue]
stops = ["#FF0000", "#0000FF"]
space = "oklch"
"##;
    let manifest = PaletteManifest::from_toml(toml_oklch).unwrap();
    let hue = manifest.gradient.get("hue").unwrap();
    assert_eq!(hue.space.as_deref(), Some("oklch"));

    // Default (no space key)
    let toml_default = r##"
[base]
background = "#000000"

[gradient.plain]
stops = ["#FF0000", "#0000FF"]
"##;
    let manifest = PaletteManifest::from_toml(toml_default).unwrap();
    let plain = manifest.gradient.get("plain").unwrap();
    assert!(plain.space.is_none());
}
