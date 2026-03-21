use crate::color::Color;
use crate::palette::Palette;
use crate::resolved::ResolvedPalette;

/// WCAG 2.1 conformance level for contrast checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContrastLevel {
    /// AA for normal text (≥ 4.5:1).
    AaNormal,
    /// AA for large text (≥ 3.0:1).
    AaLarge,
    /// AAA for normal text (≥ 7.0:1).
    AaaNormal,
    /// AAA for large text (≥ 4.5:1).
    AaaLarge,
}

impl ContrastLevel {
    /// Minimum contrast ratio required for this level.
    pub fn threshold(self) -> f64 {
        match self {
            Self::AaNormal | Self::AaaLarge => 4.5,
            Self::AaLarge => 3.0,
            Self::AaaNormal => 7.0,
        }
    }

    /// Whether the given ratio meets this conformance level.
    pub fn passes(self, ratio: f64) -> bool {
        ratio >= self.threshold()
    }
}

/// A foreground/background pair that fails a contrast check.
#[derive(Debug, Clone, PartialEq)]
pub struct ContrastViolation {
    /// Dot-path label of the foreground slot (e.g. `"base.foreground"`).
    pub foreground_label: Box<str>,
    /// Dot-path label of the background slot (e.g. `"base.background"`).
    pub background_label: Box<str>,
    /// The foreground color that was tested.
    pub foreground: Color,
    /// The background color that was tested.
    pub background: Color,
    /// Measured contrast ratio.
    pub ratio: f64,
    /// The conformance level that was not met.
    pub level: ContrastLevel,
}

/// WCAG 2.1 contrast ratio between two colors. Returns `[1.0, 21.0]`.
pub fn contrast_ratio(fg: &Color, bg: &Color) -> f64 {
    contrast_ratio_with_lum(fg.relative_luminance(), bg.relative_luminance())
}

/// Contrast ratio from pre-computed relative luminance values.
fn contrast_ratio_with_lum(l_fg: f64, l_bg: f64) -> f64 {
    let (lighter, darker) = match l_fg >= l_bg {
        true => (l_fg, l_bg),
        false => (l_bg, l_fg),
    };
    (lighter + 0.05) / (darker + 0.05)
}

/// Whether `fg` over `bg` meets the given [`ContrastLevel`].
pub fn meets_level(fg: &Color, bg: &Color, level: ContrastLevel) -> bool {
    level.passes(contrast_ratio(fg, bg))
}

impl Color {
    /// WCAG 2.1 contrast ratio against another color.
    pub fn contrast_ratio(&self, other: &Color) -> f64 {
        contrast_ratio(self, other)
    }

    /// Whether contrast against `other` meets the given [`ContrastLevel`].
    pub fn meets_level(&self, other: &Color, level: ContrastLevel) -> bool {
        meets_level(self, other, level)
    }
}

fn check_pair(
    fg_prefix: &str,
    fg_name: &str,
    bg_prefix: &str,
    bg_name: &str,
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
            foreground_label: format!("{fg_prefix}.{fg_name}").into_boxed_str(),
            background_label: format!("{bg_prefix}.{bg_name}").into_boxed_str(),
            foreground: fg_color,
            background: bg_color,
            ratio,
            level,
        }),
    }
}

/// Single source of truth for static foreground/background contrast pairs.
///
/// Semantic and syntax slots use dynamic iteration (`populated_slots` /
/// `all_slots_mut`) and are handled separately in each consumer.
macro_rules! for_each_static_pair {
    ($callback:ident ! ($($ctx:tt)*)) => {
        // Core readability
        $callback!($($ctx)*, base.foreground, base.background);
        $callback!($($ctx)*, base.foreground_dark, base.background);
        $callback!($($ctx)*, base.foreground, base.background_dark);
        $callback!($($ctx)*, base.foreground, base.background_highlight);
        // Focus surface
        $callback!($($ctx)*, base.foreground, surface.focus);
        // Editor pairs
        $callback!($($ctx)*, editor.selection_fg, editor.selection_bg);
        $callback!($($ctx)*, editor.inlay_hint_fg, editor.inlay_hint_bg);
        $callback!($($ctx)*, editor.search_fg, editor.search_bg);
        $callback!($($ctx)*, editor.cursor_text, editor.cursor);
        // Diff pairs
        $callback!($($ctx)*, diff.added_fg, diff.added_bg);
        $callback!($($ctx)*, diff.modified_fg, diff.modified_bg);
        $callback!($($ctx)*, diff.removed_fg, diff.removed_bg);
        // Typography over background
        $callback!($($ctx)*, typography.comment, base.background);
        $callback!($($ctx)*, typography.line_number, base.background);
    };
}

/// Check all semantically paired slots in a palette for contrast violations.
///
/// Returns an empty slice when every tested pair meets the given level.
pub fn validate_palette(palette: &Palette, level: ContrastLevel) -> Box<[ContrastViolation]> {
    let mut violations = Vec::new();
    let mut push = |v: Option<ContrastViolation>| {
        if let Some(v) = v {
            violations.push(v);
        }
    };

    macro_rules! validate_static_pair {
        ($palette:ident, $level:ident, $fg_section:ident . $fg_field:ident, $bg_section:ident . $bg_field:ident) => {
            push(check_pair(
                stringify!($fg_section),
                stringify!($fg_field),
                stringify!($bg_section),
                stringify!($bg_field),
                $palette.$fg_section.$fg_field.as_ref(),
                $palette.$bg_section.$bg_field.as_ref(),
                $level,
            ));
        };
    }

    for_each_static_pair!(validate_static_pair!(palette, level));

    // Semantic over background (dynamic iteration)
    for (name, color) in palette.semantic.populated_slots() {
        push(check_pair(
            "semantic",
            name,
            "base",
            "background",
            Some(color),
            palette.base.background.as_ref(),
            level,
        ));
    }

    // Syntax over background (dynamic iteration)
    for (name, color) in palette.syntax.populated_slots() {
        push(check_pair(
            "syntax",
            name,
            "base",
            "background",
            Some(color),
            palette.base.background.as_ref(),
            level,
        ));
    }

    violations.into_boxed_slice()
}

/// Nudge a foreground color's lightness until it meets the given contrast level
/// against `bg`. Returns `fg` unchanged if the pair already passes or if no
/// lightness adjustment can reach the target.
///
/// Only HSL lightness is modified; hue and saturation are preserved.
pub fn nudge_foreground(fg: Color, bg: Color, level: ContrastLevel) -> Color {
    let threshold = level.threshold();
    if contrast_ratio(&fg, &bg) >= threshold {
        return fg;
    }

    let fg_lum = fg.relative_luminance();
    let bg_lum = bg.relative_luminance();

    // Try the natural direction first: lighter fg if fg is lighter, darker otherwise.
    let primary_lighten = fg_lum >= bg_lum;
    match nudge_direction(fg, bg_lum, threshold, primary_lighten) {
        Some(result) => result,
        None => nudge_direction(fg, bg_lum, threshold, !primary_lighten).unwrap_or(fg),
    }
}

fn nudge_direction(fg: Color, bg_lum: f64, threshold: f64, lighten: bool) -> Option<Color> {
    // Check if the extreme can reach the target at all.
    let extreme = match lighten {
        true => fg.lighten(1.0),
        false => fg.darken(1.0),
    };
    match contrast_ratio_with_lum(extreme.relative_luminance(), bg_lum) >= threshold {
        true => {}
        false => return None,
    }

    let mut lo: f64 = 0.0;
    let mut hi: f64 = 1.0;
    let mut best = extreme;

    // Binary search for the minimal lightness shift that meets the threshold.
    // bg luminance is cached — it never changes across iterations.
    for _ in 0..20 {
        let mid = (lo + hi) / 2.0;
        let candidate = match lighten {
            true => fg.lighten(mid),
            false => fg.darken(mid),
        };
        match contrast_ratio_with_lum(candidate.relative_luminance(), bg_lum) >= threshold {
            true => {
                best = candidate;
                hi = mid;
            }
            false => lo = mid,
        }
    }
    Some(best)
}

/// Adjust all semantically paired foreground slots on a resolved palette so
/// they meet the given contrast level. Mirrors the pairs checked by
/// [`validate_palette`].
pub fn adjust_contrast(resolved: &mut ResolvedPalette, level: ContrastLevel) {
    macro_rules! adjust_static_pair {
        ($resolved:ident, $level:ident, $fg_section:ident . $fg_field:ident, $bg_section:ident . $bg_field:ident) => {
            $resolved.$fg_section.$fg_field = nudge_foreground(
                $resolved.$fg_section.$fg_field,
                $resolved.$bg_section.$bg_field,
                $level,
            );
        };
    }

    for_each_static_pair!(adjust_static_pair!(resolved, level));

    // Semantic over background (dynamic — all resolved slots)
    resolved.semantic.success =
        nudge_foreground(resolved.semantic.success, resolved.base.background, level);
    resolved.semantic.warning =
        nudge_foreground(resolved.semantic.warning, resolved.base.background, level);
    resolved.semantic.error =
        nudge_foreground(resolved.semantic.error, resolved.base.background, level);
    resolved.semantic.info =
        nudge_foreground(resolved.semantic.info, resolved.base.background, level);
    resolved.semantic.hint =
        nudge_foreground(resolved.semantic.hint, resolved.base.background, level);

    // Syntax over background (dynamic — cached bg luminance via nudge_foreground)
    let bg = resolved.base.background;
    for (_, slot) in resolved.syntax.all_slots_mut() {
        *slot = nudge_foreground(*slot, bg, level);
    }
}
