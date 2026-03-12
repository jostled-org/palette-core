//! Text style modifiers for syntax tokens.
//!
//! [`SyntaxStyles`](crate::style::SyntaxStyles) mirrors the field names of
//! [`SyntaxColors`](crate::palette::SyntaxColors), each slot an
//! `Option<StyleModifiers>`. Resolution through
//! [`ResolvedSyntaxStyles`](crate::style::ResolvedSyntaxStyles) applies the
//! same parent→child fallback chain as colors, defaulting to no modifiers.

use std::sync::Arc;

use crate::error::PaletteError;
use crate::manifest::ManifestSection;

/// Text style modifiers for a single syntax token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
pub struct StyleModifiers {
    /// Render the token in bold weight.
    pub bold: bool,
    /// Render the token in italic style.
    pub italic: bool,
    /// Render the token with an underline.
    pub underline: bool,
}

impl StyleModifiers {
    /// Parse a comma-separated modifier string (e.g. `"bold,italic"`).
    ///
    /// Whitespace around each modifier name is trimmed. An empty string
    /// or unknown modifier name returns [`PaletteError::InvalidStyle`].
    pub fn parse(s: &str, section: &str, field: &str) -> Result<Self, PaletteError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(PaletteError::InvalidStyle {
                section: Arc::from(section),
                field: Arc::from(field),
                value: Arc::from(s),
            });
        }

        let mut result = Self::default();
        for part in trimmed.split(',') {
            match part.trim() {
                "bold" => result.bold = true,
                "italic" => result.italic = true,
                "underline" => result.underline = true,
                _ => {
                    return Err(PaletteError::InvalidStyle {
                        section: Arc::from(section),
                        field: Arc::from(field),
                        value: Arc::from(s),
                    });
                }
            }
        }
        Ok(result)
    }

    /// True when no modifiers are set.
    pub fn is_empty(self) -> bool {
        !self.bold && !self.italic && !self.underline
    }

    /// CSS value string for the combined modifiers.
    ///
    /// Returns `"normal"` when empty, otherwise space-separated names
    /// like `"bold"`, `"italic"`, or `"bold italic"`.
    pub fn to_css_value(self) -> &'static str {
        match (self.bold, self.italic, self.underline) {
            (false, false, false) => "normal",
            (true, false, false) => "bold",
            (false, true, false) => "italic",
            (false, false, true) => "underline",
            (true, true, false) => "bold italic",
            (true, false, true) => "bold underline",
            (false, true, true) => "italic underline",
            (true, true, true) => "bold italic underline",
        }
    }
}

impl std::fmt::Display for StyleModifiers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_css_value())
    }
}

fn resolve_style(
    section: &ManifestSection,
    section_name: &str,
    field: &str,
) -> Result<Option<StyleModifiers>, PaletteError> {
    match section.get(field) {
        None => Ok(None),
        Some(val) => StyleModifiers::parse(val, section_name, field).map(Some),
    }
}

macro_rules! style_group {
    ($(#[$_meta:meta])* $_color_type:ident { $($field:ident),+ $(,)? }) => {
        /// Unresolved syntax style modifiers — each slot is `Option<StyleModifiers>`.
        ///
        /// Field names match [`SyntaxColors`](crate::palette::SyntaxColors) exactly.
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        #[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
        pub struct SyntaxStyles {
            $(
                #[doc = concat!("`", stringify!($field), "` slot.")]
                pub $field: Option<StyleModifiers>,
            )+
        }

        impl SyntaxStyles {
            /// Parse a `[syntax_style]` manifest section into style modifiers.
            pub fn from_section(
                section: &ManifestSection,
                section_name: &str,
            ) -> Result<Self, PaletteError> {
                Ok(Self {
                    $($field: resolve_style(section, section_name, stringify!($field))?,)+
                })
            }

            /// Merge two style groups, preferring `self` values over `fallback`.
            pub fn merge(&self, fallback: &Self) -> Self {
                Self {
                    $($field: self.$field.or(fallback.$field),)+
                }
            }

            /// Iterate over slots that have a style assigned.
            pub fn populated_slots(&self) -> impl Iterator<Item = (&'static str, &StyleModifiers)> {
                [$(
                    (stringify!($field), self.$field.as_ref()),
                )+]
                .into_iter()
                .filter_map(|(name, style)| style.map(|s| (name, s)))
            }
        }
    };
}

crate::palette::syntax_fields!(style_group);

macro_rules! resolved_style_group {
    ($(#[$_meta:meta])* $_color_type:ident { $($field:ident),+ $(,)? }) => {
        /// Resolved syntax styles where every slot is a concrete [`StyleModifiers`].
        ///
        /// Built via [`ResolvedSyntaxStyles::from_group_with_fallback`], which applies
        /// the same parent→child fallback chain as
        /// [`ResolvedSyntaxColors`](crate::resolved::ResolvedSyntaxColors).
        /// Absent slots default to [`StyleModifiers::default`] (no modifiers).
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
        pub struct ResolvedSyntaxStyles {
            $(
                #[doc = concat!("`", stringify!($field), "` slot.")]
                pub $field: StyleModifiers,
            )+
        }

        impl ResolvedSyntaxStyles {
            fn from_group(group: &SyntaxStyles) -> Self {
                Self {
                    $($field: group.$field.unwrap_or_default(),)+
                }
            }

            /// Iterate over all slots as `(name, &StyleModifiers)` pairs.
            pub fn all_slots(&self) -> impl Iterator<Item = (&'static str, &StyleModifiers)> {
                [$(
                    (stringify!($field), &self.$field),
                )+]
                .into_iter()
            }
        }
    };
}

crate::palette::syntax_fields!(resolved_style_group);

impl ResolvedSyntaxStyles {
    /// Resolve styles with parent→child fallback, mirroring
    /// [`ResolvedSyntaxColors::from_group_with_fallback`](crate::resolved::ResolvedSyntaxColors::from_group_with_fallback).
    pub fn from_group_with_fallback(group: &SyntaxStyles) -> Self {
        let mut resolved = Self::from_group(group);
        macro_rules! apply_fallback {
            ($($child:ident => $parent:ident),+ $(,)?) => {
                $(resolved.$child = group.$child.or(group.$parent).unwrap_or_default();)+
            };
        }
        crate::palette::syntax_fallback!(apply_fallback);
        resolved
    }
}
