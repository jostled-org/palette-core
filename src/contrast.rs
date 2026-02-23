use std::sync::Arc;

use crate::color::Color;
use crate::palette::Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContrastLevel {
    AaNormal,
    AaLarge,
    AaaNormal,
    AaaLarge,
}

impl ContrastLevel {
    pub fn threshold(self) -> f64 {
        match self {
            Self::AaNormal | Self::AaaLarge => 4.5,
            Self::AaLarge => 3.0,
            Self::AaaNormal => 7.0,
        }
    }

    pub fn passes(self, ratio: f64) -> bool {
        ratio >= self.threshold()
    }
}

#[derive(Debug, Clone)]
pub struct ContrastViolation {
    pub foreground_label: Arc<str>,
    pub background_label: Arc<str>,
    pub foreground: Color,
    pub background: Color,
    pub ratio: f64,
    pub level: ContrastLevel,
}

/// WCAG 2.1 contrast ratio between two colors. Returns `[1.0, 21.0]`.
pub fn contrast_ratio(a: &Color, b: &Color) -> f64 {
    let la = a.relative_luminance();
    let lb = b.relative_luminance();
    let (lighter, darker) = match la >= lb {
        true => (la, lb),
        false => (lb, la),
    };
    (lighter + 0.05) / (darker + 0.05)
}

pub fn meets_level(fg: &Color, bg: &Color, level: ContrastLevel) -> bool {
    level.passes(contrast_ratio(fg, bg))
}

fn check_pair(
    fg_label: &str,
    bg_label: &str,
    fg: Option<&Color>,
    bg: Option<&Color>,
    level: ContrastLevel,
) -> Option<ContrastViolation> {
    let (fg_color, bg_color) = match (fg, bg) {
        (Some(f), Some(b)) => (*f, *b),
        _ => return None,
    };
    let ratio = contrast_ratio(&fg_color, &bg_color);
    match level.passes(ratio) {
        true => None,
        false => Some(ContrastViolation {
            foreground_label: Arc::from(fg_label),
            background_label: Arc::from(bg_label),
            foreground: fg_color,
            background: bg_color,
            ratio,
            level,
        }),
    }
}

pub fn validate_palette(palette: &Palette, level: ContrastLevel) -> Vec<ContrastViolation> {
    let mut violations = Vec::new();
    let mut push = |v: Option<ContrastViolation>| {
        if let Some(v) = v {
            violations.push(v);
        }
    };

    // Core readability
    push(check_pair("base.foreground", "base.background", palette.base.foreground.as_ref(), palette.base.background.as_ref(), level));
    push(check_pair("base.foreground_dark", "base.background", palette.base.foreground_dark.as_ref(), palette.base.background.as_ref(), level));
    push(check_pair("base.foreground", "base.background_dark", palette.base.foreground.as_ref(), palette.base.background_dark.as_ref(), level));
    push(check_pair("base.foreground", "base.background_highlight", palette.base.foreground.as_ref(), palette.base.background_highlight.as_ref(), level));

    // Semantic over background
    for (name, color) in palette.semantic.populated_slots() {
        let label = format!("semantic.{name}");
        push(check_pair(&label, "base.background", Some(color), palette.base.background.as_ref(), level));
    }

    // Editor pairs
    push(check_pair("editor.selection_fg", "editor.selection_bg", palette.editor.selection_fg.as_ref(), palette.editor.selection_bg.as_ref(), level));
    push(check_pair("editor.inlay_hint_fg", "editor.inlay_hint_bg", palette.editor.inlay_hint_fg.as_ref(), palette.editor.inlay_hint_bg.as_ref(), level));
    push(check_pair("editor.search_fg", "editor.search_bg", palette.editor.search_fg.as_ref(), palette.editor.search_bg.as_ref(), level));
    push(check_pair("editor.cursor_text", "editor.cursor", palette.editor.cursor_text.as_ref(), palette.editor.cursor.as_ref(), level));

    // Diff pairs
    push(check_pair("diff.added_fg", "diff.added_bg", palette.diff.added_fg.as_ref(), palette.diff.added_bg.as_ref(), level));
    push(check_pair("diff.modified_fg", "diff.modified_bg", palette.diff.modified_fg.as_ref(), palette.diff.modified_bg.as_ref(), level));
    push(check_pair("diff.removed_fg", "diff.removed_bg", palette.diff.removed_fg.as_ref(), palette.diff.removed_bg.as_ref(), level));

    // Typography over background
    push(check_pair("typography.comment", "base.background", palette.typography.comment.as_ref(), palette.base.background.as_ref(), level));
    push(check_pair("typography.line_number", "base.background", palette.typography.line_number.as_ref(), palette.base.background.as_ref(), level));

    // Syntax over background
    for (name, color) in palette.syntax.populated_slots() {
        let label = format!("syntax.{name}");
        push(check_pair(&label, "base.background", Some(color), palette.base.background.as_ref(), level));
    }

    violations
}
