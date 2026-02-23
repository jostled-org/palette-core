#![cfg(feature = "snapshot")]

use palette_core::palette::Palette;
use palette_core::snapshot::{to_json, to_json_value};

mod common;

#[test]
fn palette_serializes_to_json() {
    let manifest = common::load_preset("tokyonight");
    let palette = Palette::from_manifest(&manifest).unwrap();
    let json = to_json(&palette).unwrap();

    assert!(json.contains("\"background\""));
    assert!(json.contains("#1A1B2A"));
}

#[test]
fn palette_serializes_to_json_value() {
    let manifest = common::load_preset("tokyonight");
    let palette = Palette::from_manifest(&manifest).unwrap();
    let value = to_json_value(&palette).unwrap();

    let base = value.get("base").unwrap();
    let bg = base.get("background").unwrap().as_str().unwrap();
    assert_eq!(bg, "#1A1B2A");
}

#[test]
fn snapshot_includes_meta() {
    let manifest = common::load_preset("tokyonight");
    let palette = Palette::from_manifest(&manifest).unwrap();
    let json = to_json(&palette).unwrap();

    assert!(json.contains("\"preset_id\""));
    assert!(json.contains("tokyonight"));
}

#[test]
fn snapshot_omits_none_colors() {
    let manifest = common::manifest_with_base(
        [("background".into(), "#112233".into())].into_iter().collect(),
    );
    let palette = Palette::from_manifest(&manifest).unwrap();
    let value = to_json_value(&palette).unwrap();

    let base = value.get("base").unwrap();
    assert!(base.get("background").unwrap().is_string());
    assert!(base.get("foreground").unwrap().is_null());
}
