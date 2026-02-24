use std::collections::BTreeMap;
use std::sync::Arc;

use palette_core::css::to_css_custom_properties;
use palette_core::palette::Palette;

mod common;

#[test]
fn no_prefix_produces_bare_variables() {
    let manifest = common::load_preset("tokyonight");
    let palette = Palette::from_manifest(&manifest).unwrap();
    let css = to_css_custom_properties(&palette, None);

    assert!(
        css.contains("--bg: #"),
        "expected bare --bg variable, got:\n{css}",
    );
}

#[test]
fn prefix_prepended_to_all_variables() {
    let manifest = common::load_preset("tokyonight");
    let palette = Palette::from_manifest(&manifest).unwrap();
    let css = to_css_custom_properties(&palette, Some("mx"));

    assert!(
        css.contains("--mx-bg: #"),
        "expected prefixed --mx-bg variable, got:\n{css}",
    );

    let var_lines: Vec<&str> = css.lines().filter(|l| l.trim_start().starts_with("--")).collect();
    assert!(
        var_lines.iter().all(|l| l.trim_start().starts_with("--mx-")),
        "all variables should have --mx- prefix, got:\n{}",
        var_lines.join("\n"),
    );
}

#[test]
fn all_populated_slots_present() {
    let manifest = common::load_preset("tokyonight");
    let palette = Palette::from_manifest(&manifest).unwrap();
    let css = to_css_custom_properties(&palette, None);

    let populated_count = palette.base.populated_slots().count()
        + palette.semantic.populated_slots().count()
        + palette.diff.populated_slots().count()
        + palette.surface.populated_slots().count()
        + palette.typography.populated_slots().count()
        + palette.syntax.populated_slots().count()
        + palette.editor.populated_slots().count()
        + palette.terminal_ansi.populated_slots().count();

    let css_line_count = css.lines().filter(|l| l.contains("--")).count();
    assert_eq!(css_line_count, populated_count);
}

#[test]
fn none_slots_absent() {
    let manifest = common::manifest_with_base(
        BTreeMap::from([(Arc::from("background"), Arc::from("#000000"))]),
    );
    let palette = Palette::from_manifest(&manifest).unwrap();
    let css = to_css_custom_properties(&palette, None);

    assert!(css.contains("--bg:"));
    assert!(!css.contains("--fg"));
    assert!(!css.contains("--success"));
    assert!(!css.contains("--ansi-"));
}

#[test]
fn field_names_map_to_short_css_names() {
    let manifest = common::manifest_with_base(
        BTreeMap::from([(Arc::from("background_dark"), Arc::from("#111111"))]),
    );
    let palette = Palette::from_manifest(&manifest).unwrap();
    let css = to_css_custom_properties(&palette, None);

    assert!(
        css.contains("--bg-dark:"),
        "background_dark should map to --bg-dark, got:\n{css}",
    );
    assert!(!css.contains("background_dark"), "raw field names should not appear in CSS output");
}

#[test]
fn all_css_names_match_design_spec() {
    let manifest = common::load_preset("tokyonight");
    let palette = Palette::from_manifest(&manifest).unwrap();
    let css = to_css_custom_properties(&palette, None);

    // Spot-check one variable from each section
    assert!(css.contains("--bg:"), "base: --bg");
    assert!(css.contains("--error:"), "semantic: --error");
    assert!(css.contains("--syn-keyword:"), "syntax: --syn-keyword");
    assert!(css.contains("--ed-cursor:"), "editor: --ed-cursor");
    assert!(css.contains("--diff-added:"), "diff: --diff-added");
    assert!(css.contains("--ansi-red:"), "terminal: --ansi-red");
    assert!(css.contains("--ui-menu:"), "surface: --ui-menu");
    assert!(css.contains("--text-comment:"), "typography: --text-comment");
}
