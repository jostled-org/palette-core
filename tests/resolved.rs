use palette_core::color::Color;
use palette_core::palette::{
    BaseColors, DiffColors, EditorColors, Palette, SemanticColors, SurfaceColors, SyntaxColors,
    TerminalAnsiColors, TypographyColors,
};
use palette_core::{preset, preset_ids};

mod common;

#[test]
fn complete_preset_resolves_matching_originals() {
    let palette = preset("tokyonight").unwrap();
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
        terminal_ansi: TerminalAnsiColors::default(),
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
        terminal_ansi: TerminalAnsiColors::default(),
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
        terminal_ansi: TerminalAnsiColors::default(),
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
    assert_eq!(resolved.syntax.all_slots().count(), 26);
    assert_eq!(resolved.editor.all_slots().count(), 17);
    assert_eq!(resolved.terminal_ansi.all_slots().count(), 16);
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
        26,
        "syntax incomplete"
    );
    assert_eq!(
        default.editor.populated_slots().count(),
        17,
        "editor incomplete"
    );
    assert_eq!(
        default.terminal_ansi.populated_slots().count(),
        16,
        "terminal_ansi incomplete"
    );
}

#[test]
fn all_presets_resolve_without_black_fallback() {
    let default = Palette::default();
    let black = Color::default();

    for id in preset_ids() {
        let palette = preset(id).unwrap();
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
    let palette = preset("tokyonight").unwrap();
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
