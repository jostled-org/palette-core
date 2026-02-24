use crate::color::Color;
use crate::palette::Palette;

pub fn to_color32(color: &Color) -> ::egui::Color32 {
    ::egui::Color32::from_rgb(color.r, color.g, color.b)
}

macro_rules! apply_color {
    ($field:expr => $($target:expr),+) => {
        match $field {
            Some(c) => {
                let color = to_color32(c);
                $($target = color;)+
            }
            None => {}
        }
    };
    ($field:expr => Some $($target:expr),+) => {
        match $field {
            Some(c) => {
                let color = to_color32(c);
                $($target = Some(color);)+
            }
            None => {}
        }
    };
}

macro_rules! apply_stroke {
    ($field:expr, $width:expr => $($target:expr),+) => {
        match $field {
            Some(c) => {
                let stroke = ::egui::Stroke::new($width, to_color32(c));
                $($target = stroke;)+
            }
            None => {}
        }
    };
}

pub fn to_egui_visuals(palette: &Palette) -> ::egui::Visuals {
    let mut v = ::egui::Visuals::dark();

    // Background fills
    apply_color!(&palette.base.background =>
        v.panel_fill, v.window_fill, v.faint_bg_color, v.extreme_bg_color);
    apply_color!(&palette.base.background_dark =>
        v.extreme_bg_color, v.code_bg_color);

    // Text colors
    apply_color!(&palette.base.foreground => Some v.override_text_color);
    apply_color!(&palette.base.foreground_dark => Some v.weak_text_color);

    // Semantic colors
    apply_color!(&palette.semantic.error => v.error_fg_color);
    apply_color!(&palette.semantic.warning => v.warn_fg_color);
    apply_color!(&palette.semantic.info => v.hyperlink_color);

    // Window stroke
    apply_stroke!(&palette.base.border, 1.0 =>
        v.window_stroke,
        v.widgets.noninteractive.bg_stroke);
    apply_stroke!(&palette.base.border_highlight, 1.0 =>
        v.widgets.hovered.bg_stroke,
        v.widgets.active.bg_stroke);

    // Widget backgrounds
    apply_color!(&palette.surface.highlight =>
        v.widgets.hovered.bg_fill, v.widgets.active.bg_fill);
    apply_color!(&palette.surface.overlay =>
        v.widgets.noninteractive.bg_fill);
    apply_color!(&palette.surface.menu =>
        v.widgets.open.bg_fill);

    // Widget foreground strokes
    apply_stroke!(&palette.base.foreground, 1.0 =>
        v.widgets.noninteractive.fg_stroke);
    apply_stroke!(&palette.base.foreground_dark, 1.0 =>
        v.widgets.inactive.fg_stroke);

    // Selection
    apply_color!(&palette.surface.selection => v.selection.bg_fill);
    apply_stroke!(&palette.editor.selection_fg, 1.0 => v.selection.stroke);

    // Text cursor
    apply_stroke!(&palette.editor.cursor, 2.0 => v.text_cursor.stroke);

    // Typography
    apply_color!(&palette.typography.link => v.hyperlink_color);

    v
}
