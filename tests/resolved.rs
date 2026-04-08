use std::sync::Arc;

use palette_core::color::Color;
use palette_core::manifest::PaletteManifest;
use palette_core::palette::{
    AnsiColors, BaseColors, DiffColors, EditorColors, Palette, SemanticColors, SurfaceColors,
    SyntaxColors, TypographyColors,
};
use palette_core::style::SyntaxStyles;
use palette_core::{load_preset, preset_ids};

mod common;

#[test]
fn complete_preset_resolves_matching_originals() {
    let palette = load_preset("tokyonight").unwrap();
    let resolved = palette.resolve();

    // Every populated slot in the original should match the resolved value.
    for (name, color) in palette.base.populated_slots() {
        let resolved_color = resolved
            .base
            .all_slots()
            .find(|(n, _)| *n == name)
            .unwrap()
            .1;
        assert_eq!(color, resolved_color, "base.{name} mismatch");
    }
    for (name, color) in palette.syntax.populated_slots() {
        let resolved_color = resolved
            .syntax
            .all_slots()
            .find(|(n, _)| *n == name)
            .unwrap()
            .1;
        assert_eq!(color, resolved_color, "syntax.{name} mismatch");
    }
}

#[test]
fn sparse_palette_fills_gaps_from_default() {
    let sparse = Palette {
        base: BaseColors {
            background: Some(Color {
                r: 0xFF,
                g: 0,
                b: 0,
            }),
            ..BaseColors::default()
        },
        ..Palette::default()
    };

    // Override a single field, make everything else None.
    let very_sparse = Palette {
        meta: None,
        base: BaseColors {
            background: Some(Color {
                r: 0xFF,
                g: 0,
                b: 0,
            }),
            ..BaseColors::default()
        },
        semantic: SemanticColors::default(),
        diff: DiffColors::default(),
        surface: SurfaceColors::default(),
        typography: TypographyColors::default(),
        syntax: SyntaxColors::default(),
        editor: EditorColors::default(),
        terminal: AnsiColors::default(),
        syntax_style: SyntaxStyles::default(),
        gradients: Arc::from([]),
        #[cfg(feature = "platform")]
        platform: Default::default(),
    };

    let resolved = very_sparse.resolve();

    // The overridden field keeps its value.
    assert_eq!(
        resolved.base.background,
        Color {
            r: 0xFF,
            g: 0,
            b: 0
        }
    );

    // Gaps filled from Palette::default().
    let default_palette = Palette::default();
    assert_eq!(
        resolved.base.foreground,
        default_palette.base.foreground.unwrap()
    );
    assert_eq!(
        resolved.semantic.error,
        default_palette.semantic.error.unwrap()
    );

    // resolve() and resolve_with(default) produce the same result.
    assert_eq!(resolved, sparse.resolve_with(&Palette::default()));
}

#[test]
fn resolve_with_custom_fallback_precedence() {
    let red = Color {
        r: 0xFF,
        g: 0,
        b: 0,
    };
    let green = Color {
        r: 0,
        g: 0xFF,
        b: 0,
    };
    let blue = Color {
        r: 0,
        g: 0,
        b: 0xFF,
    };

    let primary = Palette {
        meta: None,
        base: BaseColors {
            background: Some(red),
            ..BaseColors::default()
        },
        semantic: SemanticColors::default(),
        diff: DiffColors::default(),
        surface: SurfaceColors::default(),
        typography: TypographyColors::default(),
        syntax: SyntaxColors::default(),
        editor: EditorColors::default(),
        terminal: AnsiColors::default(),
        syntax_style: SyntaxStyles::default(),
        gradients: Arc::from([]),
        #[cfg(feature = "platform")]
        platform: Default::default(),
    };

    let fallback = Palette {
        meta: None,
        base: BaseColors {
            background: Some(blue),
            foreground: Some(green),
            ..BaseColors::default()
        },
        semantic: SemanticColors::default(),
        diff: DiffColors::default(),
        surface: SurfaceColors::default(),
        typography: TypographyColors::default(),
        syntax: SyntaxColors::default(),
        editor: EditorColors::default(),
        terminal: AnsiColors::default(),
        syntax_style: SyntaxStyles::default(),
        gradients: Arc::from([]),
        #[cfg(feature = "platform")]
        platform: Default::default(),
    };

    let resolved = primary.resolve_with(&fallback);

    // primary wins when populated.
    assert_eq!(resolved.base.background, red);
    // fallback fills gaps.
    assert_eq!(resolved.base.foreground, green);
    // neither has it → Color::default() (black).
    assert_eq!(resolved.base.border, Color::default());
}

#[test]
fn all_slots_count_matches_expected_per_group() {
    let palette = Palette::default();
    let resolved = palette.resolve();

    assert_eq!(resolved.base.all_slots().count(), 7);
    assert_eq!(resolved.semantic.all_slots().count(), 5);
    assert_eq!(resolved.diff.all_slots().count(), 11);
    assert_eq!(resolved.surface.all_slots().count(), 10);
    assert_eq!(resolved.typography.all_slots().count(), 6);
    assert_eq!(resolved.syntax.all_slots().count(), 38);
    assert_eq!(resolved.editor.all_slots().count(), 17);
    assert_eq!(resolved.terminal.all_slots().count(), 16);
}

#[test]
fn default_palette_completeness() {
    let default = Palette::default();

    // Every group should have all slots populated in the default palette.
    assert_eq!(default.base.populated_slots().count(), 7, "base incomplete");
    assert_eq!(
        default.semantic.populated_slots().count(),
        5,
        "semantic incomplete"
    );
    assert_eq!(
        default.diff.populated_slots().count(),
        11,
        "diff incomplete"
    );
    assert_eq!(
        default.surface.populated_slots().count(),
        10,
        "surface incomplete"
    );
    assert_eq!(
        default.typography.populated_slots().count(),
        6,
        "typography incomplete"
    );
    assert_eq!(
        default.syntax.populated_slots().count(),
        38,
        "syntax incomplete"
    );
    assert_eq!(
        default.editor.populated_slots().count(),
        17,
        "editor incomplete"
    );
    assert_eq!(
        default.terminal.populated_slots().count(),
        16,
        "terminal incomplete"
    );
}

#[test]
fn all_presets_resolve_without_black_fallback() {
    let default = Palette::default();
    let black = Color::default();

    for id in preset_ids() {
        let palette = load_preset(id).unwrap();
        let resolved = palette.resolve();

        // Base background should never fall back to black (every preset defines it).
        assert_ne!(
            resolved.base.background, black,
            "{id}: base.background fell through to black"
        );

        // Verify foreground is populated (from preset or default).
        let expected_fg = palette
            .base
            .foreground
            .unwrap_or_else(|| default.base.foreground.unwrap());
        assert_eq!(
            resolved.base.foreground, expected_fg,
            "{id}: base.foreground mismatch"
        );
    }
}

#[test]
fn meta_preserved_through_resolution() {
    let palette = load_preset("tokyonight").unwrap();
    let resolved = palette.resolve();

    assert!(resolved.meta.is_some());
    assert_eq!(
        resolved.meta.as_ref().unwrap().preset_id.as_ref(),
        "tokyonight"
    );
}

#[test]
fn merge_prefers_self_over_fallback() {
    let a = BaseColors {
        background: Some(Color { r: 1, g: 2, b: 3 }),
        ..BaseColors::default()
    };
    let b = BaseColors {
        background: Some(Color {
            r: 10,
            g: 20,
            b: 30,
        }),
        foreground: Some(Color {
            r: 40,
            g: 50,
            b: 60,
        }),
        ..BaseColors::default()
    };

    let merged = a.merge(&b);
    assert_eq!(merged.background, Some(Color { r: 1, g: 2, b: 3 }));
    assert_eq!(
        merged.foreground,
        Some(Color {
            r: 40,
            g: 50,
            b: 60
        })
    );
}

#[test]
fn syntax_fallback_resolves_from_parent() {
    let red = Color {
        r: 0xFF,
        g: 0,
        b: 0,
    };
    let palette = Palette {
        syntax: SyntaxColors {
            keywords: Some(red),
            ..SyntaxColors::default()
        },
        ..Palette::default()
    };
    // Use an empty fallback so the default palette doesn't fill keywords_control.
    let empty = Palette {
        syntax: SyntaxColors::default(),
        ..Palette::default()
    };

    let resolved = palette.resolve_with(&empty);
    assert_eq!(resolved.syntax.keywords_control, red);
}

#[test]
fn syntax_fallback_explicit_overrides_parent() {
    let red = Color {
        r: 0xFF,
        g: 0,
        b: 0,
    };
    let blue = Color {
        r: 0,
        g: 0,
        b: 0xFF,
    };
    let palette = Palette {
        syntax: SyntaxColors {
            keywords: Some(red),
            keywords_control: Some(blue),
            ..SyntaxColors::default()
        },
        ..Palette::default()
    };
    let empty = Palette {
        syntax: SyntaxColors::default(),
        ..Palette::default()
    };

    let resolved = palette.resolve_with(&empty);
    assert_eq!(resolved.syntax.keywords_control, blue);
}

#[test]
fn syntax_fallback_all_sub_tokens_resolve_from_parent() {
    let green = Color {
        r: 0,
        g: 0xFF,
        b: 0,
    };

    let palette = Palette {
        syntax: SyntaxColors {
            keywords: Some(green),
            functions: Some(green),
            types: Some(green),
            variables: Some(green),
            punctuation: Some(green),
            comments: Some(green),
            constants: Some(green),
            attributes: Some(green),
            ..SyntaxColors::default()
        },
        ..Palette::default()
    };
    // Empty syntax fallback so sub-tokens exercise the intra-group chain.
    let empty = Palette {
        syntax: SyntaxColors::default(),
        ..Palette::default()
    };

    let resolved = palette.resolve_with(&empty);

    assert_eq!(resolved.syntax.keywords_control, green);
    assert_eq!(resolved.syntax.keywords_import, green);
    assert_eq!(resolved.syntax.keywords_operator, green);
    assert_eq!(resolved.syntax.functions_builtin, green);
    assert_eq!(resolved.syntax.functions_method, green);
    assert_eq!(resolved.syntax.functions_macro, green);
    assert_eq!(resolved.syntax.modules, green);
    assert_eq!(resolved.syntax.labels, green);
    assert_eq!(resolved.syntax.punctuation_special, green);
    assert_eq!(resolved.syntax.comments_doc, green);
    assert_eq!(resolved.syntax.constants_char, green);
    assert_eq!(resolved.syntax.attributes_builtin, green);
}

#[test]
fn resolved_tokyonight_is_dark() {
    let palette = load_preset("tokyonight").unwrap();
    let resolved = palette.resolve();
    assert!(!resolved.is_light());
}

#[test]
fn resolved_github_light_is_light() {
    let palette = load_preset("github_light").unwrap();
    let resolved = palette.resolve();
    assert!(resolved.is_light());
}

#[test]
fn resolved_is_light_threshold_boundary() {
    // Luminance of 0.179 is the WCAG perceptual midpoint.
    // A color just above the threshold should be light.
    // RGB (124, 124, 124) has luminance ~0.195 (above 0.179).
    let above = Color {
        r: 124,
        g: 124,
        b: 124,
    };
    assert!(
        above.relative_luminance() > 0.179,
        "expected luminance above threshold, got {}",
        above.relative_luminance()
    );

    let palette_above = Palette {
        base: BaseColors {
            background: Some(above),
            ..BaseColors::default()
        },
        ..Palette::default()
    };
    assert!(palette_above.resolve().is_light());

    // RGB (115, 115, 115) has luminance ~0.162 (below 0.179).
    let below = Color {
        r: 115,
        g: 115,
        b: 115,
    };
    assert!(
        below.relative_luminance() <= 0.179,
        "expected luminance at or below threshold, got {}",
        below.relative_luminance()
    );

    let palette_below = Palette {
        base: BaseColors {
            background: Some(below),
            ..BaseColors::default()
        },
        ..Palette::default()
    };
    assert!(!palette_below.resolve().is_light());
}

// 4.T1: gradient_with_token_references_resolves
#[test]
fn gradient_with_token_references_resolves() {
    let toml = r##"
[base]
background = "#000000"
foreground = "#FFFFFF"

[gradient.brand]
stops = ["base.background", "base.foreground"]
"##;
    let manifest = PaletteManifest::from_toml(toml).unwrap();
    let palette = Palette::from_manifest(&manifest).unwrap();
    let resolved = palette.resolve();
    let gradient = resolved
        .gradient("brand")
        .expect("gradient 'brand' should exist");
    let stops = gradient.stops();
    assert_eq!(stops[0].color, Color { r: 0, g: 0, b: 0 });
    assert_eq!(
        stops[stops.len() - 1].color,
        Color {
            r: 255,
            g: 255,
            b: 255,
        }
    );
}

// 4.T2: resolved_palette_gradients_iterator
#[test]
fn resolved_palette_gradients_iterator() {
    let toml = r##"
[base]
background = "#000000"
foreground = "#FFFFFF"

[gradient.brand]
stops = ["#FF0000", "#0000FF"]

[gradient.heat]
stops = ["#FF0000", "#FFFF00", "#00FF00"]
"##;
    let manifest = PaletteManifest::from_toml(toml).unwrap();
    let palette = Palette::from_manifest(&manifest).unwrap();
    let resolved = palette.resolve();
    let names: Vec<&str> = resolved.gradients().map(|(name, _)| name).collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"brand"));
    assert!(names.contains(&"heat"));
}

// 4.T3: gradient_missing_returns_none
#[test]
fn gradient_missing_returns_none() {
    let toml = r##"
[base]
background = "#000000"
foreground = "#FFFFFF"
"##;
    let manifest = PaletteManifest::from_toml(toml).unwrap();
    let palette = Palette::from_manifest(&manifest).unwrap();
    let resolved = palette.resolve();
    assert!(resolved.gradient("anything").is_none());
}
