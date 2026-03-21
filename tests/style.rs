use std::collections::HashMap;
use std::sync::Arc;

use palette_core::error::PaletteError;
use palette_core::style::{ResolvedSyntaxStyles, StyleModifiers, SyntaxStyles};
use palette_core::{load_preset, preset_ids};

fn section(pairs: &[(&str, &str)]) -> HashMap<Arc<str>, Arc<str>> {
    pairs
        .iter()
        .map(|(k, v)| (Arc::from(*k), Arc::from(*v)))
        .collect()
}

// --- StyleModifiers::parse ---

#[test]
fn parse_single_modifier_bold() {
    let m = StyleModifiers::parse("bold", "test", "f").unwrap();
    assert!(m.bold);
    assert!(!m.italic);
    assert!(!m.underline);
}

#[test]
fn parse_single_modifier_italic() {
    let m = StyleModifiers::parse("italic", "test", "f").unwrap();
    assert!(!m.bold);
    assert!(m.italic);
    assert!(!m.underline);
}

#[test]
fn parse_single_modifier_underline() {
    let m = StyleModifiers::parse("underline", "test", "f").unwrap();
    assert!(!m.bold);
    assert!(!m.italic);
    assert!(m.underline);
}

#[test]
fn parse_multiple_modifiers() {
    let m = StyleModifiers::parse("bold,italic", "test", "f").unwrap();
    assert!(m.bold);
    assert!(m.italic);
    assert!(!m.underline);
}

#[test]
fn parse_all_three_modifiers() {
    let m = StyleModifiers::parse("bold,italic,underline", "test", "f").unwrap();
    assert!(m.bold);
    assert!(m.italic);
    assert!(m.underline);
}

#[test]
fn parse_with_whitespace() {
    let m = StyleModifiers::parse(" bold , italic ", "test", "f").unwrap();
    assert!(m.bold);
    assert!(m.italic);
}

#[test]
fn parse_empty_string_returns_error() {
    let err = StyleModifiers::parse("", "sec", "field").unwrap_err();
    assert!(matches!(err, PaletteError::InvalidStyle { .. }));
}

#[test]
fn parse_whitespace_only_returns_error() {
    let err = StyleModifiers::parse("   ", "sec", "field").unwrap_err();
    assert!(matches!(err, PaletteError::InvalidStyle { .. }));
}

#[test]
fn parse_unknown_modifier_returns_error() {
    let err = StyleModifiers::parse("strikethrough", "sec", "field").unwrap_err();
    assert!(matches!(err, PaletteError::InvalidStyle { .. }));
}

#[test]
fn parse_partial_unknown_returns_error() {
    let err = StyleModifiers::parse("bold,blink", "sec", "field").unwrap_err();
    assert!(matches!(err, PaletteError::InvalidStyle { .. }));
}

// --- StyleModifiers methods ---

#[test]
fn style_modifiers_default_is_empty() {
    let d = StyleModifiers::default();
    assert!(d.is_empty());
    assert_eq!(d.to_css_value(), "normal");
}

#[test]
fn css_value_bold() {
    let m = StyleModifiers {
        bold: true,
        ..StyleModifiers::default()
    };
    assert_eq!(m.to_css_value(), "bold");
}

#[test]
fn css_value_italic() {
    let m = StyleModifiers {
        italic: true,
        ..StyleModifiers::default()
    };
    assert_eq!(m.to_css_value(), "italic");
}

#[test]
fn css_value_bold_italic() {
    let m = StyleModifiers {
        bold: true,
        italic: true,
        ..StyleModifiers::default()
    };
    assert_eq!(m.to_css_value(), "bold italic");
}

#[test]
fn is_empty_false_when_set() {
    let m = StyleModifiers {
        underline: true,
        ..StyleModifiers::default()
    };
    assert!(!m.is_empty());
}

// --- SyntaxStyles merge ---

#[test]
fn style_merge_prefers_self() {
    let italic = Some(StyleModifiers {
        italic: true,
        ..StyleModifiers::default()
    });
    let bold = Some(StyleModifiers {
        bold: true,
        ..StyleModifiers::default()
    });

    let a = SyntaxStyles {
        keywords: italic,
        ..SyntaxStyles::default()
    };
    let b = SyntaxStyles {
        keywords: bold,
        functions: bold,
        ..SyntaxStyles::default()
    };

    let merged = a.merge(&b);
    assert_eq!(merged.keywords, italic);
    assert_eq!(merged.functions, bold);
}

// --- Fallback chain ---

#[test]
fn style_fallback_resolves_from_parent() {
    let italic = Some(StyleModifiers {
        italic: true,
        ..StyleModifiers::default()
    });

    let styles = SyntaxStyles {
        keywords: italic,
        ..SyntaxStyles::default()
    };

    let resolved = ResolvedSyntaxStyles::from_group_with_fallback(&styles);
    assert_eq!(resolved.keywords_control, italic.unwrap());
    assert_eq!(resolved.keywords_import, italic.unwrap());
    assert_eq!(resolved.keywords_operator, italic.unwrap());
}

#[test]
fn style_fallback_explicit_overrides_parent() {
    let italic = Some(StyleModifiers {
        italic: true,
        ..StyleModifiers::default()
    });
    let bold = Some(StyleModifiers {
        bold: true,
        ..StyleModifiers::default()
    });

    let styles = SyntaxStyles {
        keywords: italic,
        keywords_control: bold,
        ..SyntaxStyles::default()
    };

    let resolved = ResolvedSyntaxStyles::from_group_with_fallback(&styles);
    assert_eq!(resolved.keywords_control, bold.unwrap());
    assert_eq!(resolved.keywords_import, italic.unwrap());
}

#[test]
fn style_fallback_all_sub_tokens() {
    let italic = Some(StyleModifiers {
        italic: true,
        ..StyleModifiers::default()
    });

    let styles = SyntaxStyles {
        keywords: italic,
        functions: italic,
        types: italic,
        variables: italic,
        punctuation: italic,
        comments: italic,
        constants: italic,
        attributes: italic,
        ..SyntaxStyles::default()
    };

    let resolved = ResolvedSyntaxStyles::from_group_with_fallback(&styles);

    // keywords children
    assert_eq!(resolved.keywords_control, italic.unwrap());
    assert_eq!(resolved.keywords_import, italic.unwrap());
    assert_eq!(resolved.keywords_operator, italic.unwrap());
    // functions children
    assert_eq!(resolved.functions_builtin, italic.unwrap());
    assert_eq!(resolved.functions_method, italic.unwrap());
    assert_eq!(resolved.functions_macro, italic.unwrap());
    // other fallbacks
    assert_eq!(resolved.modules, italic.unwrap());
    assert_eq!(resolved.labels, italic.unwrap());
    assert_eq!(resolved.punctuation_special, italic.unwrap());
    assert_eq!(resolved.comments_doc, italic.unwrap());
    assert_eq!(resolved.constants_char, italic.unwrap());
    assert_eq!(resolved.attributes_builtin, italic.unwrap());
}

#[test]
fn absent_style_defaults_to_no_modifiers() {
    let styles = SyntaxStyles::default();
    let resolved = ResolvedSyntaxStyles::from_group_with_fallback(&styles);
    assert!(resolved.keywords.is_empty());
    assert!(resolved.comments.is_empty());
}

// --- SyntaxStyles::from_section ---

#[test]
fn from_section_parses_styles() {
    let sec = section(&[("keywords", "italic"), ("comments_doc", "bold,italic")]);
    let styles = SyntaxStyles::from_section(&sec, "syntax_style").unwrap();

    assert_eq!(
        styles.keywords,
        Some(StyleModifiers {
            italic: true,
            ..StyleModifiers::default()
        })
    );
    assert_eq!(
        styles.comments_doc,
        Some(StyleModifiers {
            bold: true,
            italic: true,
            ..StyleModifiers::default()
        })
    );
    assert_eq!(styles.functions, None);
}

#[test]
fn from_section_bad_value_returns_error() {
    let sec = section(&[("keywords", "blink")]);
    let err = SyntaxStyles::from_section(&sec, "syntax_style").unwrap_err();
    assert!(matches!(err, PaletteError::InvalidStyle { .. }));
}

// --- Field name parity ---

#[test]
fn syntax_styles_fields_match_syntax_colors() {
    // Both default groups are all-None, so use a populated palette + resolved styles.
    let palette = palette_core::Palette::default();
    let color_names: Vec<&str> = palette.syntax.populated_slots().map(|(n, _)| n).collect();

    let resolved = ResolvedSyntaxStyles::from_group_with_fallback(&SyntaxStyles::default());
    let style_names: Vec<&str> = resolved.all_slots().map(|(n, _)| n).collect();

    assert_eq!(color_names.len(), style_names.len(), "field count mismatch");
    for (cn, sn) in color_names.iter().zip(style_names.iter()) {
        assert_eq!(cn, sn, "field name mismatch: color={cn}, style={sn}");
    }
}

// --- Preset integration ---

#[test]
fn all_presets_parse_styles_without_error() {
    for id in preset_ids() {
        let palette = load_preset(id).unwrap();
        // If it loaded, styles parsed successfully.
        // Verify resolve doesn't panic.
        let _ = palette.resolve();
    }
}

// --- CSS output ---

#[test]
fn style_css_output_includes_modifiers() {
    let mut palette = palette_core::Palette::default();
    palette.syntax_style = SyntaxStyles {
        keywords: Some(StyleModifiers {
            italic: true,
            ..StyleModifiers::default()
        }),
        comments_doc: Some(StyleModifiers {
            bold: true,
            italic: true,
            ..StyleModifiers::default()
        }),
        ..SyntaxStyles::default()
    };

    let css = palette.to_css();
    assert!(
        css.contains("--syn-keyword-style: italic;"),
        "missing keyword style in CSS"
    );
    assert!(
        css.contains("--syn-comment-doc-style: bold italic;"),
        "missing comment-doc style in CSS"
    );
}

#[test]
fn style_css_omits_empty_modifiers() {
    let palette = palette_core::Palette::default();
    let css = palette.to_css();
    // Default palette has no styles set, so no style vars should appear.
    assert!(
        !css.contains("-style:"),
        "empty styles should not emit CSS vars"
    );
}
