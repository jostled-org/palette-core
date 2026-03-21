use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::color::Color;
use crate::contrast::ContrastLevel;
use crate::palette::Palette;
use crate::registry::{Registry, ThemeInfo};

fn to_js_error(err: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&err.to_string())
}

/// Parse a WCAG contrast level string into a [`ContrastLevel`] variant.
///
/// | Input string  | Variant                       |
/// |---------------|-------------------------------|
/// | `"aa"`        | [`ContrastLevel::AaNormal`]   |
/// | `"aa-large"`  | [`ContrastLevel::AaLarge`]    |
/// | `"aaa"`       | [`ContrastLevel::AaaNormal`]  |
/// | `"aaa-large"` | [`ContrastLevel::AaaLarge`]   |
pub(crate) fn parse_contrast_level(s: &str) -> Result<ContrastLevel, JsValue> {
    match s {
        "aa" => Ok(ContrastLevel::AaNormal),
        "aa-large" => Ok(ContrastLevel::AaLarge),
        "aaa" => Ok(ContrastLevel::AaaNormal),
        "aaa-large" => Ok(ContrastLevel::AaaLarge),
        _ => Err(JsValue::from_str(&format!("unknown contrast level: {s}"))),
    }
}

fn slots_to_js_map<'a>(slots: impl Iterator<Item = (&'static str, &'a Color)>) -> js_sys::Map {
    let map = js_sys::Map::new();
    for (name, color) in slots {
        let js_color = JsColor::from_color(*color);
        map.set(&JsValue::from_str(name), &js_color.into());
    }
    map
}

#[wasm_bindgen]
pub struct JsColor {
    inner: Color,
}

impl JsColor {
    pub fn from_color(color: Color) -> Self {
        Self { inner: color }
    }

    pub fn as_color(&self) -> &Color {
        &self.inner
    }
}

#[wasm_bindgen]
impl JsColor {
    #[wasm_bindgen(js_name = "fromHex")]
    pub fn from_hex(hex: &str) -> Result<JsColor, JsValue> {
        Color::from_hex(hex)
            .map(|c| Self { inner: c })
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "toHex")]
    pub fn to_hex(&self) -> String {
        String::from(self.inner.to_hex())
    }

    #[wasm_bindgen(getter)]
    pub fn r(&self) -> u8 {
        self.inner.r
    }

    #[wasm_bindgen(getter)]
    pub fn g(&self) -> u8 {
        self.inner.g
    }

    #[wasm_bindgen(getter)]
    pub fn b(&self) -> u8 {
        self.inner.b
    }

    pub fn lighten(&self, amount: f64) -> JsColor {
        Self {
            inner: self.inner.lighten(amount),
        }
    }

    pub fn darken(&self, amount: f64) -> JsColor {
        Self {
            inner: self.inner.darken(amount),
        }
    }

    pub fn saturate(&self, amount: f64) -> JsColor {
        Self {
            inner: self.inner.saturate(amount),
        }
    }

    pub fn desaturate(&self, amount: f64) -> JsColor {
        Self {
            inner: self.inner.desaturate(amount),
        }
    }

    #[wasm_bindgen(js_name = "rotateHue")]
    pub fn rotate_hue(&self, degrees: f64) -> JsColor {
        Self {
            inner: self.inner.rotate_hue(degrees),
        }
    }

    #[wasm_bindgen(js_name = "relativeLuminance")]
    pub fn relative_luminance(&self) -> f64 {
        self.inner.relative_luminance()
    }
}

#[wasm_bindgen]
pub struct JsPalette {
    inner: Palette,
}

/// Generate a `#[wasm_bindgen] impl` block with methods returning `Option<String>`
/// from palette meta fields.
macro_rules! palette_meta_getters {
    ($ty:ident, $($(#[$attr:meta])* $fn_name:ident => $field:ident),+ $(,)?) => {
        #[wasm_bindgen]
        impl $ty {
            $(
                $(#[$attr])*
                pub fn $fn_name(&self) -> Option<String> {
                    self.inner.meta.as_ref().map(|m| m.$field.to_string())
                }
            )+
        }
    };
}

/// Generate a `#[wasm_bindgen] impl` block with methods returning `js_sys::Map`
/// from palette slot group fields.
macro_rules! palette_slot_getters {
    ($ty:ident, $($(#[$attr:meta])* $fn_name:ident => $field:ident),+ $(,)?) => {
        #[wasm_bindgen]
        impl $ty {
            $(
                $(#[$attr])*
                pub fn $fn_name(&self) -> js_sys::Map {
                    slots_to_js_map(self.inner.$field.populated_slots())
                }
            )+
        }
    };
}

palette_meta_getters!(JsPalette,
    name => name,
    #[wasm_bindgen(js_name = "presetId")]
    preset_id => preset_id,
    style => style,
);

palette_slot_getters!(JsPalette,
    #[wasm_bindgen(js_name = "baseSlots")]
    base_slots => base,
    #[wasm_bindgen(js_name = "semanticSlots")]
    semantic_slots => semantic,
    #[wasm_bindgen(js_name = "diffSlots")]
    diff_slots => diff,
    #[wasm_bindgen(js_name = "surfaceSlots")]
    surface_slots => surface,
    #[wasm_bindgen(js_name = "typographySlots")]
    typography_slots => typography,
    #[wasm_bindgen(js_name = "syntaxSlots")]
    syntax_slots => syntax,
    #[wasm_bindgen(js_name = "editorSlots")]
    editor_slots => editor,
    #[wasm_bindgen(js_name = "terminalAnsiSlots")]
    terminal_slots => terminal,
);

#[wasm_bindgen]
impl JsPalette {
    /// CSS block with `:root` selector, no prefix.
    #[wasm_bindgen(js_name = "toCss")]
    pub fn to_css(&self) -> String {
        self.inner.to_css()
    }

    /// CSS block with a custom selector and optional prefix.
    #[wasm_bindgen(js_name = "toCssScoped")]
    pub fn to_css_scoped(&self, selector: &str, prefix: Option<String>) -> String {
        self.inner.to_css_scoped(selector, prefix.as_deref())
    }

    #[wasm_bindgen(js_name = "toJson")]
    pub fn to_json(&self) -> Result<String, JsValue> {
        crate::snapshot::to_json(&self.inner).map_err(to_js_error)
    }

    /// Style modifier slots as a `Map<string, string>` (e.g. `"bold,italic"`).
    #[wasm_bindgen(js_name = "syntaxStyleSlots")]
    pub fn syntax_style_slots(&self) -> js_sys::Map {
        let map = js_sys::Map::new();
        for (name, style) in self.inner.syntax_style.populated_slots() {
            if !style.is_empty() {
                map.set(
                    &JsValue::from_str(name),
                    &JsValue::from_str(style.to_css_value()),
                );
            }
        }
        map
    }
}

fn load_preset_palette(id: &str) -> Result<Palette, JsValue> {
    crate::registry::load_preset(id).map_err(to_js_error)
}

#[wasm_bindgen(js_name = "loadPreset")]
pub fn load_preset(id: &str) -> Result<JsPalette, JsValue> {
    load_preset_palette(id).map(|p| JsPalette { inner: p })
}

/// Load a built-in preset by ID, returning `undefined` if not found.
#[wasm_bindgen(js_name = "preset")]
pub fn preset_js(id: &str) -> Option<JsPalette> {
    crate::registry::load_preset(id)
        .ok()
        .map(|p| JsPalette { inner: p })
}

#[wasm_bindgen(js_name = "loadPresetCss")]
pub fn load_preset_css(id: &str) -> Result<String, JsValue> {
    load_preset_palette(id).map(|p| p.to_css())
}

#[wasm_bindgen(js_name = "loadPresetJson")]
pub fn load_preset_json(id: &str) -> Result<String, JsValue> {
    let palette = load_preset_palette(id)?;
    crate::snapshot::to_json(&palette).map_err(to_js_error)
}

#[wasm_bindgen(js_name = "presetIds")]
pub fn preset_ids_js() -> Vec<String> {
    crate::registry::preset_ids()
        .iter()
        .map(|s| (*s).to_owned())
        .collect()
}

#[wasm_bindgen(js_name = "contrastRatio")]
pub fn contrast_ratio_js(a: &JsColor, b: &JsColor) -> f64 {
    crate::contrast::contrast_ratio(&a.inner, &b.inner)
}

#[wasm_bindgen(js_name = "meetsContrastLevel")]
pub fn meets_contrast_level_js(fg: &JsColor, bg: &JsColor, level: &str) -> Result<bool, JsValue> {
    let parsed = parse_contrast_level(level)?;
    Ok(crate::contrast::meets_level(&fg.inner, &bg.inner, parsed))
}

#[wasm_bindgen(js_name = "blend")]
pub fn blend_js(fg: &JsColor, bg: &JsColor, alpha: f64) -> JsColor {
    JsColor {
        inner: crate::manipulation::blend(fg.inner, bg.inner, alpha),
    }
}

// ---------------------------------------------------------------------------
// Registry wrappers
// ---------------------------------------------------------------------------

#[wasm_bindgen]
pub struct JsThemeInfo {
    id: Arc<str>,
    name: Arc<str>,
    style: Arc<str>,
}

impl JsThemeInfo {
    fn from_theme_info(info: &ThemeInfo) -> Self {
        Self {
            id: Arc::clone(&info.id),
            name: Arc::clone(&info.name),
            style: Arc::clone(&info.style),
        }
    }
}

/// Generate a `#[wasm_bindgen] impl` block with getter methods returning `String`
/// from `Arc<str>` fields.
macro_rules! arc_str_getters {
    ($ty:ident, $($(#[$attr:meta])* $fn_name:ident => $field:ident),+ $(,)?) => {
        #[wasm_bindgen]
        impl $ty {
            $(
                $(#[$attr])*
                pub fn $fn_name(&self) -> String {
                    self.$field.to_string()
                }
            )+
        }
    };
}

arc_str_getters!(JsThemeInfo,
    #[wasm_bindgen(getter)]
    id => id,
    #[wasm_bindgen(getter)]
    name => name,
    #[wasm_bindgen(getter)]
    style => style,
);

#[wasm_bindgen]
pub struct JsRegistry {
    inner: Registry,
}

impl Default for JsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl JsRegistry {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Registry::new(),
        }
    }

    pub fn list(&self) -> Vec<JsThemeInfo> {
        self.inner
            .list()
            .map(JsThemeInfo::from_theme_info)
            .collect()
    }

    pub fn load(&self, id: &str) -> Result<JsPalette, JsValue> {
        self.inner
            .load(id)
            .map(|p| JsPalette { inner: p })
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "addToml")]
    pub fn add_toml(&mut self, toml: &str) -> Result<(), JsValue> {
        self.inner.add_toml(toml).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "byStyle")]
    pub fn by_style(&self, style: &str) -> Vec<JsThemeInfo> {
        self.inner
            .by_style(style)
            .map(JsThemeInfo::from_theme_info)
            .collect()
    }
}
