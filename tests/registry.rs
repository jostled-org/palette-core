use palette_core::color::Color;
use palette_core::error::PaletteError;
use palette_core::registry::{load_preset, preset_ids};

#[test]
fn all_presets_load_with_background() {
    for id in preset_ids() {
        let palette = load_preset(id).unwrap_or_else(|e| panic!("preset {id} failed: {e}"));
        assert!(
            palette.base.background.is_some(),
            "preset {id} missing base.background"
        );
    }
}

#[test]
fn variant_inherits_parent_colors() {
    let storm = load_preset("tokyonight_storm").unwrap();

    assert_eq!(
        storm.base.background,
        Some(Color::from_hex("#24283b").unwrap()),
        "storm should use its own background"
    );
    assert_eq!(
        storm.semantic.success,
        Some(Color::from_hex("#73daca").unwrap()),
        "storm should inherit success from tokyonight base"
    );
}

#[test]
fn unknown_preset_returns_error() {
    let result = load_preset("nonexistent");
    assert!(matches!(result, Err(PaletteError::UnknownPreset(_))));
}

#[test]
fn preset_ids_list_is_complete() {
    assert_eq!(preset_ids().len(), 28);
}
