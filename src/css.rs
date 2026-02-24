use std::borrow::Cow;
use std::fmt::Write;

use crate::color::Color;
use crate::palette::Palette;

fn css_name(section: &str, field: &str) -> Option<&'static str> {
    match (section, field) {
        // Core — base (no section prefix)
        ("base", "background") => Some("bg"),
        ("base", "background_dark") => Some("bg-dark"),
        ("base", "background_highlight") => Some("bg-hi"),
        ("base", "foreground") => Some("fg"),
        ("base", "foreground_dark") => Some("fg-dark"),
        ("base", "border") => Some("border"),
        ("base", "border_highlight") => Some("border-hi"),

        // Core — semantic (no section prefix)
        ("semantic", "success") => Some("success"),
        ("semantic", "warning") => Some("warning"),
        ("semantic", "error") => Some("error"),
        ("semantic", "info") => Some("info"),
        ("semantic", "hint") => Some("hint"),

        // Surfaces — ui-
        ("surface", "menu") => Some("ui-menu"),
        ("surface", "sidebar") => Some("ui-sidebar"),
        ("surface", "statusline") => Some("ui-statusline"),
        ("surface", "float") => Some("ui-float"),
        ("surface", "popup") => Some("ui-popup"),
        ("surface", "overlay") => Some("ui-overlay"),
        ("surface", "highlight") => Some("ui-hi"),
        ("surface", "selection") => Some("ui-sel"),
        ("surface", "focus") => Some("ui-focus"),
        ("surface", "search") => Some("ui-search"),

        // Text — text-
        ("typography", "comment") => Some("text-comment"),
        ("typography", "gutter") => Some("text-gutter"),
        ("typography", "line_number") => Some("text-line-num"),
        ("typography", "selection_text") => Some("text-sel"),
        ("typography", "link") => Some("text-link"),
        ("typography", "title") => Some("text-title"),

        // Syntax — syn-
        ("syntax", "keywords") => Some("syn-keyword"),
        ("syntax", "keywords_fn") => Some("syn-keyword-fn"),
        ("syntax", "functions") => Some("syn-fn"),
        ("syntax", "variables") => Some("syn-var"),
        ("syntax", "variables_builtin") => Some("syn-var-builtin"),
        ("syntax", "parameters") => Some("syn-param"),
        ("syntax", "properties") => Some("syn-prop"),
        ("syntax", "types") => Some("syn-type"),
        ("syntax", "types_builtin") => Some("syn-type-builtin"),
        ("syntax", "constants") => Some("syn-const"),
        ("syntax", "numbers") => Some("syn-number"),
        ("syntax", "booleans") => Some("syn-bool"),
        ("syntax", "strings") => Some("syn-string"),
        ("syntax", "strings_doc") => Some("syn-string-doc"),
        ("syntax", "strings_escape") => Some("syn-string-esc"),
        ("syntax", "strings_regex") => Some("syn-string-re"),
        ("syntax", "operators") => Some("syn-op"),
        ("syntax", "punctuation") => Some("syn-punct"),
        ("syntax", "punctuation_bracket") => Some("syn-punct-bracket"),
        ("syntax", "annotations") => Some("syn-annotation"),
        ("syntax", "attributes") => Some("syn-attr"),
        ("syntax", "constructor") => Some("syn-ctor"),
        ("syntax", "tag") => Some("syn-tag"),
        ("syntax", "tag_delimiter") => Some("syn-tag-delim"),
        ("syntax", "tag_attribute") => Some("syn-tag-attr"),
        ("syntax", "comments") => Some("syn-comment"),

        // Editor — ed-
        ("editor", "cursor") => Some("ed-cursor"),
        ("editor", "cursor_text") => Some("ed-cursor-text"),
        ("editor", "match_paren") => Some("ed-match-paren"),
        ("editor", "selection_bg") => Some("ed-sel-bg"),
        ("editor", "selection_fg") => Some("ed-sel-fg"),
        ("editor", "inlay_hint_bg") => Some("ed-hint-bg"),
        ("editor", "inlay_hint_fg") => Some("ed-hint-fg"),
        ("editor", "search_bg") => Some("ed-search-bg"),
        ("editor", "search_fg") => Some("ed-search-fg"),
        ("editor", "diagnostic_error") => Some("ed-diag-error"),
        ("editor", "diagnostic_warn") => Some("ed-diag-warn"),
        ("editor", "diagnostic_info") => Some("ed-diag-info"),
        ("editor", "diagnostic_hint") => Some("ed-diag-hint"),
        ("editor", "diagnostic_underline_error") => Some("ed-diag-ul-error"),
        ("editor", "diagnostic_underline_warn") => Some("ed-diag-ul-warn"),
        ("editor", "diagnostic_underline_info") => Some("ed-diag-ul-info"),
        ("editor", "diagnostic_underline_hint") => Some("ed-diag-ul-hint"),

        // Diff — diff-
        ("diff", "added") => Some("diff-added"),
        ("diff", "added_bg") => Some("diff-added-bg"),
        ("diff", "added_fg") => Some("diff-added-fg"),
        ("diff", "modified") => Some("diff-modified"),
        ("diff", "modified_bg") => Some("diff-modified-bg"),
        ("diff", "modified_fg") => Some("diff-modified-fg"),
        ("diff", "removed") => Some("diff-removed"),
        ("diff", "removed_bg") => Some("diff-removed-bg"),
        ("diff", "removed_fg") => Some("diff-removed-fg"),
        ("diff", "text_bg") => Some("diff-text-bg"),
        ("diff", "ignored") => Some("diff-ignored"),

        // ANSI — ansi-
        ("terminal", "black") => Some("ansi-black"),
        ("terminal", "red") => Some("ansi-red"),
        ("terminal", "green") => Some("ansi-green"),
        ("terminal", "yellow") => Some("ansi-yellow"),
        ("terminal", "blue") => Some("ansi-blue"),
        ("terminal", "magenta") => Some("ansi-magenta"),
        ("terminal", "cyan") => Some("ansi-cyan"),
        ("terminal", "white") => Some("ansi-white"),
        ("terminal", "bright_black") => Some("ansi-bright-black"),
        ("terminal", "bright_red") => Some("ansi-bright-red"),
        ("terminal", "bright_green") => Some("ansi-bright-green"),
        ("terminal", "bright_yellow") => Some("ansi-bright-yellow"),
        ("terminal", "bright_blue") => Some("ansi-bright-blue"),
        ("terminal", "bright_magenta") => Some("ansi-bright-magenta"),
        ("terminal", "bright_cyan") => Some("ansi-bright-cyan"),
        ("terminal", "bright_white") => Some("ansi-bright-white"),

        _ => None,
    }
}

fn fallback_slot(section: &str, field: &str) -> String {
    format!("{section}-{}", field.replace('_', "-"))
}

fn write_section<'a>(
    out: &mut String,
    prefix: Option<&str>,
    section: &str,
    slots: impl Iterator<Item = (&'static str, &'a Color)>,
) {
    for (field, color) in slots {
        let slot: Cow<'static, str> = match css_name(section, field) {
            Some(name) => Cow::Borrowed(name),
            None => Cow::Owned(fallback_slot(section, field)),
        };
        // String::write_fmt is infallible
        let _ = match prefix {
            Some(p) => writeln!(out, "  --{p}-{slot}: {color};"),
            None => writeln!(out, "  --{slot}: {color};"),
        };
    }
}

impl Palette {
    pub fn to_css(&self, prefix: Option<&str>) -> String {
        to_css_custom_properties(self, prefix)
    }
}

pub fn to_css_custom_properties(palette: &Palette, prefix: Option<&str>) -> String {
    let mut out = String::with_capacity(3072);
    write_section(&mut out, prefix, "base", palette.base.populated_slots());
    write_section(&mut out, prefix, "semantic", palette.semantic.populated_slots());
    write_section(&mut out, prefix, "diff", palette.diff.populated_slots());
    write_section(&mut out, prefix, "surface", palette.surface.populated_slots());
    write_section(&mut out, prefix, "typography", palette.typography.populated_slots());
    write_section(&mut out, prefix, "syntax", palette.syntax.populated_slots());
    write_section(&mut out, prefix, "editor", palette.editor.populated_slots());
    write_section(&mut out, prefix, "terminal", palette.terminal_ansi.populated_slots());
    out
}
