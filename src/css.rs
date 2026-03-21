use std::fmt::Write;

use crate::color::Color;
use crate::palette::Palette;

/// Map a section/field pair to its short CSS custom property name.
///
/// Returns `None` if no explicit name is registered. The test
/// `all_fields_have_explicit_css_names` guarantees every field has a mapping.
pub fn css_name(section: &str, field: &str) -> Option<&'static str> {
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
        ("syntax", "keywords_control") => Some("syn-keyword-ctrl"),
        ("syntax", "keywords_import") => Some("syn-keyword-import"),
        ("syntax", "keywords_operator") => Some("syn-keyword-op"),
        ("syntax", "functions_builtin") => Some("syn-fn-builtin"),
        ("syntax", "functions_method") => Some("syn-fn-method"),
        ("syntax", "functions_macro") => Some("syn-fn-macro"),
        ("syntax", "modules") => Some("syn-module"),
        ("syntax", "labels") => Some("syn-label"),
        ("syntax", "punctuation_special") => Some("syn-punct-special"),
        ("syntax", "comments_doc") => Some("syn-comment-doc"),
        ("syntax", "constants_char") => Some("syn-const-char"),
        ("syntax", "attributes_builtin") => Some("syn-attr-builtin"),

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

/// Write a single CSS custom property declaration.
fn write_property(
    out: &mut String,
    prefix: Option<&str>,
    slot: &str,
    value: &dyn std::fmt::Display,
) {
    // String::write_fmt is infallible
    let _ = match prefix {
        Some(p) => writeln!(out, "  --{p}-{slot}: {value};"),
        None => writeln!(out, "  --{slot}: {value};"),
    };
}

fn write_section<'a>(
    out: &mut String,
    prefix: Option<&str>,
    section: &str,
    slots: impl Iterator<Item = (&'static str, &'a Color)>,
) {
    for (field, color) in slots {
        let slot = match css_name(section, field) {
            Some(name) => name,
            None => field,
        };
        write_property(out, prefix, slot, color);
    }
}

impl Palette {
    /// Complete CSS block with `:root` selector and no prefix.
    ///
    /// For custom selectors or prefixed variables, use [`to_css_scoped`](Self::to_css_scoped).
    /// For bare declarations without a selector, use [`to_css_custom_properties`].
    pub fn to_css(&self) -> String {
        self.to_css_scoped(":root", None)
    }

    /// Complete CSS block with a custom selector and optional prefix.
    pub fn to_css_scoped(&self, selector: &str, prefix: Option<&str>) -> String {
        let mut out = String::with_capacity(1024);
        let _ = writeln!(out, "{selector} {{");
        write_declarations(&mut out, self, prefix);
        let _ = writeln!(out, "}}");
        out
    }
}

/// Bare CSS custom-property declarations without a selector block.
pub fn to_css_custom_properties(palette: &Palette, prefix: Option<&str>) -> String {
    let mut out = String::with_capacity(1024);
    write_declarations(&mut out, palette, prefix);
    out
}

/// Write all palette declarations into an existing buffer.
fn write_declarations(out: &mut String, palette: &Palette, prefix: Option<&str>) {
    write_section(out, prefix, "base", palette.base.populated_slots());
    write_section(out, prefix, "semantic", palette.semantic.populated_slots());
    write_section(out, prefix, "diff", palette.diff.populated_slots());
    write_section(out, prefix, "surface", palette.surface.populated_slots());
    write_section(
        out,
        prefix,
        "typography",
        palette.typography.populated_slots(),
    );
    write_section(out, prefix, "syntax", palette.syntax.populated_slots());
    write_section(out, prefix, "editor", palette.editor.populated_slots());
    write_section(out, prefix, "terminal", palette.terminal.populated_slots());
    write_style_section(out, prefix, &palette.syntax_style);
}

fn write_style_section(
    out: &mut String,
    prefix: Option<&str>,
    styles: &crate::style::SyntaxStyles,
) {
    for (field, style) in styles.populated_slots() {
        if style.is_empty() {
            continue;
        }
        let slot = match css_name("syntax", field) {
            Some(name) => name,
            None => field,
        };
        let suffix_slot = format!("{slot}-style");
        write_property(out, prefix, &suffix_slot, &style.to_css_value());
    }
}
