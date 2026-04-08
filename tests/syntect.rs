#![cfg(feature = "syntect")]

use std::collections::HashSet;

use palette_core::palette::Palette;
use palette_core::syntect::{scope_mapping, to_syntect_theme};
use palette_core::{load_preset, preset_ids};
use syntect::highlighting::FontStyle;

mod common;

fn resolve_preset(
    name: &str,
) -> (
    palette_core::ResolvedPalette,
    palette_core::style::ResolvedSyntaxStyles,
) {
    let manifest = common::load_preset(name);
    let palette = Palette::from_manifest(&manifest).unwrap();
    let resolved = palette.resolve();
    let styles = resolved.syntax_style;
    (resolved, styles)
}

/// Find a ThemeItem whose scope was built from the given scope selector string.
///
/// Parses the selector string the same way the implementation does, then
/// compares parsed `ScopeSelectors` for equality.
fn find_scope_item<'a>(
    scopes: &'a [syntect::highlighting::ThemeItem],
    selector: &str,
) -> Option<&'a syntect::highlighting::ThemeItem> {
    let target: syntect::highlighting::ScopeSelectors = selector.parse().unwrap();
    scopes.iter().find(|item| item.scope == target)
}

#[test]
fn theme_has_correct_foreground_and_background() {
    let (resolved, styles) = resolve_preset("tokyonight");
    let theme = to_syntect_theme(&resolved, &styles);

    let settings = &theme.settings;
    let fg = settings.foreground.unwrap();
    let bg = settings.background.unwrap();

    assert_eq!(fg.r, resolved.base.foreground.r);
    assert_eq!(fg.g, resolved.base.foreground.g);
    assert_eq!(fg.b, resolved.base.foreground.b);
    assert_eq!(fg.a, 0xFF);

    assert_eq!(bg.r, resolved.base.background.r);
    assert_eq!(bg.g, resolved.base.background.g);
    assert_eq!(bg.b, resolved.base.background.b);
    assert_eq!(bg.a, 0xFF);
}

#[test]
fn theme_has_editor_chrome_settings() {
    let (resolved, styles) = resolve_preset("golden_hour");
    let theme = to_syntect_theme(&resolved, &styles);

    let settings = &theme.settings;

    // caret ← editor.cursor
    let caret = settings.caret.unwrap();
    assert_eq!(caret.r, resolved.editor.cursor.r);
    assert_eq!(caret.g, resolved.editor.cursor.g);
    assert_eq!(caret.b, resolved.editor.cursor.b);

    // selection ← editor.selection_bg
    let sel = settings.selection.unwrap();
    assert_eq!(sel.r, resolved.editor.selection_bg.r);
    assert_eq!(sel.g, resolved.editor.selection_bg.g);
    assert_eq!(sel.b, resolved.editor.selection_bg.b);

    // find_highlight ← editor.search_bg
    let find = settings.find_highlight.unwrap();
    assert_eq!(find.r, resolved.editor.search_bg.r);
    assert_eq!(find.g, resolved.editor.search_bg.g);
    assert_eq!(find.b, resolved.editor.search_bg.b);

    // line_highlight ← base.background_highlight
    let line_hl = settings.line_highlight.unwrap();
    assert_eq!(line_hl.r, resolved.base.background_highlight.r);
    assert_eq!(line_hl.g, resolved.base.background_highlight.g);
    assert_eq!(line_hl.b, resolved.base.background_highlight.b);

    // gutter_foreground ← typography.line_number
    let gutter_fg = settings.gutter_foreground.unwrap();
    assert_eq!(gutter_fg.r, resolved.typography.line_number.r);
    assert_eq!(gutter_fg.g, resolved.typography.line_number.g);
    assert_eq!(gutter_fg.b, resolved.typography.line_number.b);
}

#[test]
fn theme_name_matches_palette_meta() {
    let (resolved, styles) = resolve_preset("tokyonight");
    let theme = to_syntect_theme(&resolved, &styles);

    let meta = resolved.meta.as_ref().unwrap();
    assert_eq!(theme.name.as_deref(), Some(meta.name.as_ref()));
}

// --- Step 2 tests ---

#[test]
fn all_syntax_fields_have_scope_mappings() {
    let (resolved, _styles) = resolve_preset("tokyonight");
    let mapped_fields: HashSet<&str> = scope_mapping().iter().map(|&(field, _)| field).collect();
    let all_fields: HashSet<&str> = resolved.syntax.all_slots().map(|(name, _)| name).collect();
    assert_eq!(
        mapped_fields,
        all_fields,
        "scope_mapping must cover every syntax field.\nmissing: {:?}\nextra: {:?}",
        all_fields.difference(&mapped_fields).collect::<Vec<_>>(),
        mapped_fields.difference(&all_fields).collect::<Vec<_>>(),
    );
}

#[test]
fn keyword_scope_has_correct_color() {
    let (resolved, styles) = resolve_preset("tokyonight");
    let theme = to_syntect_theme(&resolved, &styles);

    let keyword_item =
        find_scope_item(&theme.scopes, "keyword").expect("theme must have a keyword scope");

    let fg = keyword_item.style.foreground.unwrap();
    assert_eq!(fg.r, resolved.syntax.keywords.r);
    assert_eq!(fg.g, resolved.syntax.keywords.g);
    assert_eq!(fg.b, resolved.syntax.keywords.b);
}

#[test]
fn bold_italic_style_mapped_to_font_style() {
    let (resolved, styles) = resolve_preset("tokyonight");
    let theme = to_syntect_theme(&resolved, &styles);

    // Check that keywords with bold style map to BOLD font_style
    if styles.keywords.bold {
        let keyword_item =
            find_scope_item(&theme.scopes, "keyword").expect("keyword scope must exist");
        let fs = keyword_item.style.font_style.unwrap();
        assert!(
            fs.contains(FontStyle::BOLD),
            "keyword font_style should contain BOLD"
        );
    }

    // Check that comments with italic style map to ITALIC font_style
    if styles.comments.italic {
        let comment_item =
            find_scope_item(&theme.scopes, "comment").expect("comment scope must exist");
        let fs = comment_item.style.font_style.unwrap();
        assert!(
            fs.contains(FontStyle::ITALIC),
            "comment font_style should contain ITALIC"
        );
    }
}

#[test]
fn child_scopes_more_specific_than_parents() {
    let (resolved, styles) = resolve_preset("tokyonight");
    let theme = to_syntect_theme(&resolved, &styles);

    assert!(
        find_scope_item(&theme.scopes, "keyword").is_some(),
        "theme must have a `keyword` scope"
    );
    assert!(
        find_scope_item(&theme.scopes, "keyword.control").is_some(),
        "theme must have a `keyword.control` scope"
    );
}

#[test]
fn scopes_vec_has_no_duplicates() {
    let (resolved, styles) = resolve_preset("tokyonight");
    let theme = to_syntect_theme(&resolved, &styles);

    for (i, a) in theme.scopes.iter().enumerate() {
        for b in &theme.scopes[i + 1..] {
            assert_ne!(a.scope, b.scope, "duplicate scope selector in theme");
        }
    }
}

// --- Step 3 tests ---

#[test]
fn all_presets_produce_valid_theme() {
    for id in preset_ids() {
        let palette = load_preset(id).unwrap();
        let resolved = palette.resolve();
        let styles = resolved.syntax_style;
        let theme = to_syntect_theme(&resolved, &styles);

        assert!(
            !theme.scopes.is_empty(),
            "preset {id}: scopes must not be empty"
        );
        assert!(
            theme.settings.foreground.is_some(),
            "preset {id}: foreground must be Some"
        );
        assert!(
            theme.settings.background.is_some(),
            "preset {id}: background must be Some"
        );
    }
}

#[test]
fn light_theme_has_dark_foreground() {
    let palette = load_preset("github_light").unwrap();
    let resolved = palette.resolve();
    let styles = resolved.syntax_style;
    let theme = to_syntect_theme(&resolved, &styles);

    let fg = theme.settings.foreground.unwrap();
    let bg = theme.settings.background.unwrap();

    let fg_lum = palette_core::color::Color {
        r: fg.r,
        g: fg.g,
        b: fg.b,
    }
    .relative_luminance();
    let bg_lum = palette_core::color::Color {
        r: bg.r,
        g: bg.g,
        b: bg.b,
    }
    .relative_luminance();

    assert!(
        fg_lum < bg_lum,
        "light theme foreground (lum={fg_lum:.3}) should be darker than background (lum={bg_lum:.3})"
    );
}
