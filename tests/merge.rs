use std::collections::HashMap;
use std::sync::Arc;

use palette_core::manifest::{ManifestSection, PaletteManifest};
use palette_core::merge::merge_manifests;

fn section(pairs: &[(&str, &str)]) -> ManifestSection {
    pairs
        .iter()
        .map(|(k, v)| (Arc::from(*k), Arc::from(*v)))
        .collect()
}

fn empty() -> ManifestSection {
    HashMap::new()
}

fn make_manifest(
    name: &str,
    preset_id: &str,
    base: ManifestSection,
    terminal: ManifestSection,
) -> PaletteManifest {
    let toml = format!(
        r##"
[meta]
name = "{name}"
preset_id = "{preset_id}"
schema_version = "1"
style = "dark"
kind = "preset-base"

[base]
placeholder = "#000000"
"##
    );
    let mut m = PaletteManifest::from_toml(&toml).unwrap();
    m.base = base;
    m.terminal = terminal;
    m
}

#[test]
fn variant_key_overrides_base_key() {
    let variant = make_manifest("V", "v", section(&[("bg", "#111")]), empty());
    let base = make_manifest("B", "b", section(&[("bg", "#222")]), empty());

    let merged = merge_manifests(&variant, &base);

    assert_eq!(&**merged.base.get("bg").unwrap(), "#111");
}

#[test]
fn base_key_fills_gap_when_variant_section_empty() {
    let variant = make_manifest("V", "v", empty(), empty());
    let base = make_manifest("B", "b", section(&[("bg", "#222")]), empty());

    let merged = merge_manifests(&variant, &base);

    assert_eq!(&**merged.base.get("bg").unwrap(), "#222");
}

#[test]
fn base_key_fills_gap_when_variant_section_lacks_key() {
    let variant = make_manifest("V", "v", section(&[("fg", "#aaa")]), empty());
    let base = make_manifest("B", "b", section(&[("bg", "#222")]), empty());

    let merged = merge_manifests(&variant, &base);

    assert_eq!(&**merged.base.get("fg").unwrap(), "#aaa");
    assert_eq!(&**merged.base.get("bg").unwrap(), "#222");
}

#[test]
fn meta_taken_from_variant() {
    let variant = make_manifest("Variant", "variant_id", empty(), empty());
    let base = make_manifest("Base", "base_id", empty(), empty());

    let merged = merge_manifests(&variant, &base);

    let meta = merged.meta.as_ref().unwrap();
    assert_eq!(&*meta.name, "Variant");
    assert_eq!(&*meta.preset_id, "variant_id");
}

#[test]
fn full_merge_produces_union_of_all_keys() {
    let variant = make_manifest(
        "V",
        "v",
        section(&[("bg", "#111"), ("fg", "#aaa")]),
        section(&[("black", "#000")]),
    );
    let base = make_manifest(
        "B",
        "b",
        section(&[("bg", "#222"), ("border", "#333")]),
        section(&[("black", "#010"), ("red", "#f00")]),
    );

    let merged = merge_manifests(&variant, &base);

    assert_eq!(merged.base.len(), 3);
    assert_eq!(&**merged.base.get("bg").unwrap(), "#111");
    assert_eq!(&**merged.base.get("fg").unwrap(), "#aaa");
    assert_eq!(&**merged.base.get("border").unwrap(), "#333");

    assert_eq!(merged.terminal.len(), 2);
    assert_eq!(&**merged.terminal.get("black").unwrap(), "#000");
    assert_eq!(&**merged.terminal.get("red").unwrap(), "#f00");
}

// 5.T1: variant_inherits_base_gradient
#[test]
fn variant_inherits_base_gradient() {
    let base_toml = r##"
[meta]
name = "Base"
preset_id = "base"
schema_version = "1"
style = "dark"
kind = "preset-base"

[base]
background = "#000000"
foreground = "#FFFFFF"

[gradient.brand]
stops = ["#FF0000", "#0000FF"]
"##;
    let variant_toml = r##"
[meta]
name = "Variant"
preset_id = "variant"
schema_version = "1"
style = "dark"
kind = "preset-variant"

[base]
background = "#111111"
"##;
    let base = PaletteManifest::from_toml(base_toml).unwrap();
    let variant = PaletteManifest::from_toml(variant_toml).unwrap();
    let merged = merge_manifests(&variant, &base);
    let palette = palette_core::Palette::from_manifest(&merged).unwrap();
    let resolved = palette.resolve();
    let gradient = resolved
        .gradient("brand")
        .expect("variant should inherit base gradient");
    assert_eq!(gradient.stops().len(), 2);
}

// 5.T2: variant_overrides_base_gradient
#[test]
fn variant_overrides_base_gradient() {
    let base_toml = r##"
[meta]
name = "Base"
preset_id = "base"
schema_version = "1"
style = "dark"
kind = "preset-base"

[base]
background = "#000000"
foreground = "#FFFFFF"

[gradient.brand]
stops = ["#000000", "#FFFFFF"]
"##;
    let variant_toml = r##"
[meta]
name = "Variant"
preset_id = "variant"
schema_version = "1"
style = "dark"
kind = "preset-variant"

[base]
background = "#111111"

[gradient.brand]
stops = ["#FF0000", "#0000FF"]
"##;
    let base = PaletteManifest::from_toml(base_toml).unwrap();
    let variant = PaletteManifest::from_toml(variant_toml).unwrap();
    let merged = merge_manifests(&variant, &base);
    let palette = palette_core::Palette::from_manifest(&merged).unwrap();
    let resolved = palette.resolve();
    let gradient = resolved.gradient("brand").unwrap();
    let stops = gradient.stops();
    // Variant's red and blue, not base's black and white
    assert_eq!(
        stops[0].color,
        palette_core::color::Color { r: 255, g: 0, b: 0 }
    );
    assert_eq!(
        stops[1].color,
        palette_core::color::Color { r: 0, g: 0, b: 255 }
    );
}

// 5.T3: variant_adds_new_gradient
#[test]
fn variant_adds_new_gradient() {
    let base_toml = r##"
[meta]
name = "Base"
preset_id = "base"
schema_version = "1"
style = "dark"
kind = "preset-base"

[base]
background = "#000000"
foreground = "#FFFFFF"

[gradient.brand]
stops = ["#FF0000", "#0000FF"]
"##;
    let variant_toml = r##"
[meta]
name = "Variant"
preset_id = "variant"
schema_version = "1"
style = "dark"
kind = "preset-variant"

[base]
background = "#111111"

[gradient.heat]
stops = ["#FF0000", "#FFFF00", "#00FF00"]
"##;
    let base = PaletteManifest::from_toml(base_toml).unwrap();
    let variant = PaletteManifest::from_toml(variant_toml).unwrap();
    let merged = merge_manifests(&variant, &base);
    let palette = palette_core::Palette::from_manifest(&merged).unwrap();
    let resolved = palette.resolve();
    assert!(
        resolved.gradient("brand").is_some(),
        "base gradient should be inherited"
    );
    assert!(
        resolved.gradient("heat").is_some(),
        "variant gradient should be present"
    );
}

#[test]
fn real_preset_tokyonight_storm_merge() {
    let base_toml = include_str!("../presets/tokyonight.toml");
    let storm_toml = include_str!("../presets/tokyonight_storm.toml");

    let base = PaletteManifest::from_toml(base_toml).unwrap();
    let storm = PaletteManifest::from_toml(storm_toml).unwrap();

    let merged = merge_manifests(&storm, &base);

    // Storm overrides base.background
    assert_eq!(&**merged.base.get("background").unwrap(), "#24283b");

    // Storm doesn't override terminal.red — falls back to base
    assert_eq!(&**merged.terminal.get("red").unwrap(), "#f7768e");

    // Meta is the variant's
    let meta = merged.meta.as_ref().unwrap();
    assert_eq!(&*meta.preset_id, "tokyonight_storm");
}
