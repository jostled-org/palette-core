//! Resolved palette types with concrete [`Color`] fields.
//!
//! A [`ResolvedPalette`] mirrors [`Palette`](crate::Palette) but every color
//! slot is a bare [`Color`] instead of `Option<Color>`. Obtain one by calling
//! [`Palette::resolve`] (fills gaps from [`Palette::default`]) or
//! [`Palette::resolve_with`] (fills gaps from a custom fallback).

use std::sync::{Arc, LazyLock};

use crate::color::Color;
use crate::contrast::{ContrastLevel, adjust_contrast};
use crate::gradient::{Gradient, GradientColor, GradientDef, GradientStop};
use crate::palette::Palette;
use crate::style::ResolvedSyntaxStyles;

static DEFAULT_PALETTE: LazyLock<Palette> = LazyLock::new(Palette::default);

macro_rules! resolved_group {
    ($(#[$_meta:meta])* $color_type:ident { $($field:ident),+ $(,)? }) => {
        pastey::paste! {
            #[doc = concat!("Resolved version of [`", stringify!($color_type), "`](crate::palette::", stringify!($color_type), ") with concrete [`Color`] fields.")]
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            #[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
            pub struct [<Resolved $color_type>] {
                $(
                    #[doc = concat!("`", stringify!($field), "` slot.")]
                    pub $field: Color,
                )+
            }

            impl [<Resolved $color_type>] {
                /// Build from an optional-field group, filling gaps with [`Color::default`] (black).
                ///
                /// Use [`Palette::resolve`] with the complete default palette to
                /// avoid black fallbacks. Black only appears when using
                /// [`Palette::resolve_with`] against an incomplete custom fallback.
                pub fn from_group(group: &crate::palette::$color_type) -> Self {
                    Self {
                        $($field: group.$field.unwrap_or_default(),)+
                    }
                }

                /// Iterate over all slots as `(name, &Color)` pairs.
                pub fn all_slots(&self) -> impl Iterator<Item = (&'static str, &Color)> {
                    [$(
                        (stringify!($field), &self.$field),
                    )+]
                    .into_iter()
                }

                /// Iterate over all slots as `(name, &mut Color)` pairs.
                pub fn all_slots_mut(&mut self) -> impl Iterator<Item = (&'static str, &mut Color)> {
                    [$(
                        (stringify!($field), &mut self.$field),
                    )+]
                    .into_iter()
                }
            }
        }
    };
}

crate::palette::color_fields!(resolved_group);

impl ResolvedSyntaxColors {
    /// Build from an optional-field group, resolving sub-token slots through
    /// their fallback parent before falling back to [`Color::default`].
    ///
    /// For the 12 sub-token slots (e.g. `keywords_control`), resolution order
    /// is: `self.slot → self.parent → Color::default()`.
    pub fn from_group_with_fallback(group: &crate::palette::SyntaxColors) -> Self {
        let mut resolved = Self::from_group(group);
        crate::palette::resolve_syntax_fallback!(resolved, group);
        resolved
    }
}

/// Fully resolved palette where every color slot is a concrete [`Color`].
///
/// Built via [`Palette::resolve`] or [`Palette::resolve_with`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
pub struct ResolvedPalette {
    /// Theme identity, if the source palette had metadata.
    pub meta: Option<std::sync::Arc<crate::palette::PaletteMeta>>,
    /// Core background and foreground colors.
    pub base: ResolvedBaseColors,
    /// Status colors (success, warning, error, info, hint).
    pub semantic: ResolvedSemanticColors,
    /// Version-control diff highlighting.
    pub diff: ResolvedDiffColors,
    /// UI surface colors (menus, sidebars, overlays).
    pub surface: ResolvedSurfaceColors,
    /// Text chrome (comments, line numbers, links).
    pub typography: ResolvedTypographyColors,
    /// Syntax-highlighting token colors.
    pub syntax: ResolvedSyntaxColors,
    /// Editor chrome (cursor, selections, diagnostics).
    pub editor: ResolvedEditorColors,
    /// Standard 16-color ANSI terminal palette.
    pub terminal: ResolvedAnsiColors,
    /// Syntax token style modifiers.
    pub syntax_style: ResolvedSyntaxStyles,
    /// Named gradients, sorted by name for binary search lookup.
    gradients: Box<[(Arc<str>, Gradient)]>,
}

impl ResolvedPalette {
    /// Returns `true` if the background color is perceptually light.
    ///
    /// Uses the WCAG relative luminance midpoint (0.179) as the threshold.
    pub fn is_light(&self) -> bool {
        self.base.background.is_light()
    }

    /// Look up a named gradient. Returns `None` if no gradient with that name exists.
    pub fn gradient(&self, name: &str) -> Option<&Gradient> {
        self.gradients
            .binary_search_by_key(&name, |(k, _)| k.as_ref())
            .ok()
            .map(|i| &self.gradients[i].1)
    }

    /// Iterate over all named gradients as `(name, gradient)` pairs.
    pub fn gradients(&self) -> impl Iterator<Item = (&str, &Gradient)> {
        self.gradients.iter().map(|(k, v)| (k.as_ref(), v))
    }
}

impl Palette {
    /// Resolve all `Option<Color>` slots using [`Palette::default`] as fallback.
    pub fn resolve(&self) -> ResolvedPalette {
        self.resolve_with(&DEFAULT_PALETTE)
    }

    /// Resolve all slots and nudge foreground colors to meet the given
    /// [`ContrastLevel`]. The TOML-defined colors remain authoritative;
    /// only failing foreground slots are lightened or darkened.
    pub fn resolve_with_contrast(&self, level: ContrastLevel) -> ResolvedPalette {
        let mut resolved = self.resolve();
        adjust_contrast(&mut resolved, level);
        resolved
    }

    /// Resolve all `Option<Color>` slots using a custom fallback palette.
    ///
    /// Slots absent in both `self` and `fallback` resolve to
    /// [`Color::default`] (black). Use [`resolve`](Self::resolve) with the
    /// complete default palette to avoid this.
    ///
    /// Each `.merge()` produces a stack-allocated group of `Option<Color>`
    /// (Copy types) consumed immediately by `from_group`. No heap allocation.
    pub fn resolve_with(&self, fallback: &Palette) -> ResolvedPalette {
        let base = ResolvedBaseColors::from_group(&self.base.merge(&fallback.base));
        let semantic = ResolvedSemanticColors::from_group(&self.semantic.merge(&fallback.semantic));
        let diff = ResolvedDiffColors::from_group(&self.diff.merge(&fallback.diff));
        let surface = ResolvedSurfaceColors::from_group(&self.surface.merge(&fallback.surface));
        let typography =
            ResolvedTypographyColors::from_group(&self.typography.merge(&fallback.typography));
        let syntax =
            ResolvedSyntaxColors::from_group_with_fallback(&self.syntax.merge(&fallback.syntax));
        let editor = ResolvedEditorColors::from_group(&self.editor.merge(&fallback.editor));
        let terminal = ResolvedAnsiColors::from_group(&self.terminal.merge(&fallback.terminal));
        let syntax_style = ResolvedSyntaxStyles::from_group_with_fallback(
            &self.syntax_style.merge(&fallback.syntax_style),
        );

        let mut resolved = ResolvedPalette {
            meta: self.meta.clone(),
            base,
            semantic,
            diff,
            surface,
            typography,
            syntax,
            editor,
            terminal,
            syntax_style,
            gradients: Box::new([]),
        };
        resolved.gradients = resolve_gradients(&self.gradients, &resolved);
        resolved
    }
}

/// Find a named slot within an `all_slots()` iterator.
fn find_slot<'a>(mut slots: impl Iterator<Item = (&'static str, &'a Color)>, field: &str) -> Color {
    slots
        .find(|(name, _)| *name == field)
        .map(|(_, color)| *color)
        .unwrap_or_default()
}

impl ResolvedPalette {
    /// Look up a resolved color by section and field name.
    ///
    /// Token references were validated at parse time, so an unrecognized
    /// section/field falls back to `Color::default()`.
    fn lookup_token(&self, section: &str, field: &str) -> Color {
        match section {
            "base" => find_slot(self.base.all_slots(), field),
            "semantic" => find_slot(self.semantic.all_slots(), field),
            "diff" => find_slot(self.diff.all_slots(), field),
            "surface" => find_slot(self.surface.all_slots(), field),
            "typography" => find_slot(self.typography.all_slots(), field),
            "syntax" => find_slot(self.syntax.all_slots(), field),
            "editor" => find_slot(self.editor.all_slots(), field),
            "terminal" => find_slot(self.terminal.all_slots(), field),
            _ => Color::default(),
        }
    }
}

/// Resolve all gradient definitions into concrete gradients.
///
/// Input is already sorted by name (from `parse_gradients`), so the output
/// preserves that order for binary-search lookup.
fn resolve_gradients(
    defs: &[(Arc<str>, GradientDef)],
    resolved: &ResolvedPalette,
) -> Box<[(Arc<str>, Gradient)]> {
    defs.iter()
        .map(|(name, def)| {
            let stops: Vec<GradientStop> = def
                .stops()
                .iter()
                .map(|(gc, position)| {
                    let color = match gc {
                        GradientColor::Literal(c) => *c,
                        GradientColor::Token { section, field } => {
                            resolved.lookup_token(section, field)
                        }
                    };
                    GradientStop {
                        color,
                        position: *position,
                    }
                })
                .collect();
            // Validated at parse time to have ≥ 2 sorted stops.
            (
                Arc::clone(name),
                Gradient::new_unchecked(stops, def.space()),
            )
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}
