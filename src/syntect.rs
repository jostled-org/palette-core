//! Convert a resolved palette into a syntect `Theme`.
//!
//! This module is a leaf export node — same position as `terminal`, `egui`,
//! and `css`. It consumes [`ResolvedPalette`] and [`crate::style::ResolvedSyntaxStyles`]
//! by reference and returns an owned `Theme`.
//!
//! # Usage
//!
//! ```
//! use palette_core::load_preset;
//! use palette_core::syntect::to_syntect_theme;
//!
//! let palette = load_preset("tokyonight").unwrap();
//! let resolved = palette.resolve();
//! let theme = to_syntect_theme(&resolved, &resolved.syntax_style);
//!
//! // Pass the theme to syntect's highlighter:
//! // let mut h = HighlightLines::new(syntax, &theme);
//! ```

use std::collections::HashMap;

use syntect::highlighting::{
    Color as SyntectColor, FontStyle, ScopeSelectors, StyleModifier, Theme, ThemeItem,
    ThemeSettings,
};

use crate::color::Color;
use crate::resolved::ResolvedPalette;
use crate::style::{ResolvedSyntaxStyles, StyleModifiers};

/// Static mapping from palette-core syntax field names to TextMate scope selectors.
///
/// Each entry is `(field_name, &[scope_selector])`. One `ThemeItem` is emitted per
/// scope selector string, all sharing the color and style of the named field.
const SCOPE_MAP: &[(&str, &[&str])] = &[
    ("keywords", &["keyword"]),
    (
        "keywords_fn",
        &["keyword.declaration.function", "storage.type.function"],
    ),
    ("keywords_control", &["keyword.control"]),
    ("keywords_import", &["keyword.control.import"]),
    ("keywords_operator", &["keyword.operator"]),
    ("functions", &["entity.name.function"]),
    ("functions_builtin", &["support.function"]),
    (
        "functions_method",
        &["entity.name.function.method", "meta.function-call.method"],
    ),
    (
        "functions_macro",
        &["entity.name.function.macro", "support.function.macro"],
    ),
    ("variables", &["variable"]),
    ("variables_builtin", &["variable.language"]),
    ("parameters", &["variable.parameter"]),
    (
        "properties",
        &["variable.other.property", "entity.name.tag.yaml"],
    ),
    ("types", &["entity.name.type", "support.type"]),
    ("types_builtin", &["storage.type", "support.type.builtin"]),
    ("constants", &["constant", "constant.other"]),
    ("constants_char", &["constant.character"]),
    ("numbers", &["constant.numeric"]),
    (
        "booleans",
        &["constant.language.boolean", "constant.language"],
    ),
    ("strings", &["string"]),
    (
        "strings_doc",
        &[
            "string.quoted.docstring",
            "comment.block.documentation string",
        ],
    ),
    ("strings_escape", &["constant.character.escape"]),
    ("strings_regex", &["string.regexp"]),
    (
        "operators",
        &["keyword.operator.assignment", "keyword.operator.arithmetic"],
    ),
    ("punctuation", &["punctuation"]),
    (
        "punctuation_bracket",
        &["punctuation.section", "punctuation.definition.group"],
    ),
    ("punctuation_special", &["punctuation.special"]),
    (
        "annotations",
        &["meta.annotation", "storage.type.annotation"],
    ),
    ("attributes", &["entity.other.attribute-name"]),
    ("attributes_builtin", &["support.other.attribute"]),
    (
        "constructor",
        &[
            "entity.name.function.constructor",
            "meta.function-call.constructor",
        ],
    ),
    ("modules", &["entity.name.namespace", "entity.name.module"]),
    ("labels", &["entity.name.label"]),
    ("tag", &["entity.name.tag"]),
    ("tag_delimiter", &["punctuation.definition.tag"]),
    (
        "tag_attribute",
        &[
            "entity.other.attribute-name.html",
            "entity.other.attribute-name.jsx",
        ],
    ),
    ("comments", &["comment"]),
    ("comments_doc", &["comment.block.documentation"]),
];

/// Return the static scope mapping table.
///
/// Each entry pairs a palette-core syntax field name with the TextMate scope
/// selectors it maps to. Used by tests to verify complete coverage.
pub fn scope_mapping() -> &'static [(&'static str, &'static [&'static str])] {
    SCOPE_MAP
}

/// Convert a palette-core [`Color`] to a syntect [`SyntectColor`] with full opacity.
fn to_syntect_color(color: &Color) -> SyntectColor {
    SyntectColor {
        r: color.r,
        g: color.g,
        b: color.b,
        a: 0xFF,
    }
}

/// Convert palette-core [`StyleModifiers`] to a syntect [`FontStyle`] bitflag.
fn to_font_style(mods: &StyleModifiers) -> FontStyle {
    let mut fs = FontStyle::empty();
    if mods.bold {
        fs |= FontStyle::BOLD;
    }
    if mods.italic {
        fs |= FontStyle::ITALIC;
    }
    if mods.underline {
        fs |= FontStyle::UNDERLINE;
    }
    fs
}

/// Build a syntect [`Theme`] from a resolved palette and its syntax styles.
///
/// The returned theme can be passed directly to `syntect::easy::HighlightLines`
/// or serialized via syntect's `.tmTheme` writer.
///
/// Global `ThemeSettings` are populated from base and editor colors.
/// `Theme.scopes` contains one `ThemeItem` per TextMate scope selector,
/// covering all 38 syntax fields with their colors and font styles.
pub fn to_syntect_theme(palette: &ResolvedPalette, styles: &ResolvedSyntaxStyles) -> Theme {
    let name = palette.meta.as_ref().map(|m| String::from(m.name.as_ref()));

    let settings = ThemeSettings {
        foreground: Some(to_syntect_color(&palette.base.foreground)),
        background: Some(to_syntect_color(&palette.base.background)),
        caret: Some(to_syntect_color(&palette.editor.cursor)),
        line_highlight: Some(to_syntect_color(&palette.base.background_highlight)),
        selection: Some(to_syntect_color(&palette.editor.selection_bg)),
        selection_foreground: Some(to_syntect_color(&palette.editor.selection_fg)),
        find_highlight: Some(to_syntect_color(&palette.editor.search_bg)),
        find_highlight_foreground: Some(to_syntect_color(&palette.editor.search_fg)),
        gutter: Some(to_syntect_color(&palette.typography.gutter)),
        gutter_foreground: Some(to_syntect_color(&palette.typography.line_number)),
        misspelling: Some(to_syntect_color(&palette.semantic.error)),
        brackets_foreground: Some(to_syntect_color(&palette.editor.match_paren)),
        ..ThemeSettings::default()
    };

    let scopes = build_scope_items(palette, styles);

    Theme {
        name,
        author: None,
        settings,
        scopes,
    }
}

/// Parse a scope selector string. Returns `None` only if syntect's parser
/// rejects the string — which cannot happen for our static table entries.
fn parse_scope(s: &str) -> Option<ScopeSelectors> {
    s.parse().ok()
}

/// Build the `ThemeItem` vec from the scope mapping table.
///
/// Collects color and style slots into hash maps for O(1) field lookup,
/// avoiding repeated linear scans of every slot per SCOPE_MAP entry.
fn build_scope_items(palette: &ResolvedPalette, styles: &ResolvedSyntaxStyles) -> Vec<ThemeItem> {
    let colors: HashMap<&str, Color> = palette
        .syntax
        .all_slots()
        .map(|(name, c)| (name, *c))
        .collect();
    let font_styles: HashMap<&str, StyleModifiers> =
        styles.all_slots().map(|(name, s)| (name, *s)).collect();

    let capacity: usize = SCOPE_MAP.iter().map(|(_, scopes)| scopes.len()).sum();
    let mut items = Vec::with_capacity(capacity);

    for &(field, scope_strs) in SCOPE_MAP {
        let color = colors
            .get(field)
            .map(to_syntect_color)
            .unwrap_or(SyntectColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0xFF,
            });
        let font_style = font_styles
            .get(field)
            .map(to_font_style)
            .unwrap_or_else(|| to_font_style(&StyleModifiers::default()));

        let style = StyleModifier {
            foreground: Some(color),
            background: None,
            font_style: Some(font_style),
        };

        for scope_str in scope_strs {
            if let Some(scope) = parse_scope(scope_str) {
                items.push(ThemeItem { scope, style });
            }
        }
    }

    items
}
