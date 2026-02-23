#![cfg(feature = "platform")]

use palette_core::palette::Palette;
use palette_core::registry::load_preset;

mod common;

#[test]
fn base_preset_has_terminal_platform() {
    let palette = load_preset("tokyonight").unwrap();

    let terminal = palette.platform.get("terminal").unwrap();
    assert!(terminal.background.is_some());
    assert!(terminal.foreground.is_some());
}

#[test]
fn base_preset_has_web_platform() {
    let palette = load_preset("tokyonight").unwrap();

    let web = palette.platform.get("web").unwrap();
    assert!(web.background.is_some());
    assert!(web.foreground.is_some());
}

#[test]
fn terminal_and_web_backgrounds_differ() {
    let palette = load_preset("tokyonight").unwrap();

    let terminal_bg = palette.platform.get("terminal").unwrap().background.unwrap();
    let web_bg = palette.platform.get("web").unwrap().background.unwrap();
    assert_ne!(terminal_bg, web_bg);
}

#[test]
fn empty_manifest_has_no_platforms() {
    let manifest = common::manifest_with_base(
        [("background".into(), "#112233".into())].into_iter().collect(),
    );
    let palette = Palette::from_manifest(&manifest).unwrap();

    assert!(palette.platform.is_empty());
}

#[test]
fn variant_overrides_platform_background() {
    let storm = load_preset("tokyonight_storm").unwrap();
    let base = load_preset("tokyonight").unwrap();

    let storm_bg = storm.platform.get("terminal").unwrap().background.unwrap();
    let base_bg = base.platform.get("terminal").unwrap().background.unwrap();
    assert_ne!(storm_bg, base_bg);
}

#[test]
fn variant_inherits_platform_foreground() {
    let storm = load_preset("tokyonight_storm").unwrap();
    let base = load_preset("tokyonight").unwrap();

    let storm_fg = storm.platform.get("terminal").unwrap().foreground;
    let base_fg = base.platform.get("terminal").unwrap().foreground;
    assert_eq!(storm_fg, base_fg);
}

#[test]
fn platform_override_resolves_hex_values() {
    let palette = load_preset("tokyonight").unwrap();

    let terminal = palette.platform.get("terminal").unwrap();
    let bg = terminal.background.unwrap();
    assert_eq!(bg.to_hex(), "#16161E");
}
