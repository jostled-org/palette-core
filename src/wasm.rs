use wasm_bindgen::prelude::*;

use crate::color::Color;
use crate::contrast::ContrastLevel;
use crate::palette::Palette;
use crate::registry::{Registry, ThemeInfo};

fn to_js_error(err: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&err.to_string())
}

pub fn parse_contrast_level(s: &str) -> Result<ContrastLevel, JsValue> {
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
        self.inner.to_hex()
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

#[wasm_bindgen]
impl JsPalette {
    pub fn name(&self) -> Option<String> {
        self.inner.meta.as_ref().map(|m| m.name.to_string())
    }

    #[wasm_bindgen(js_name = "presetId")]
    pub fn preset_id(&self) -> Option<String> {
        self.inner.meta.as_ref().map(|m| m.preset_id.to_string())
    }

    pub fn style(&self) -> Option<String> {
        self.inner.meta.as_ref().map(|m| m.style.to_string())
    }

    #[wasm_bindgen(js_name = "toCss")]
    pub fn to_css(&self, prefix: Option<String>) -> String {
        crate::css::to_css_custom_properties(&self.inner, prefix.as_deref())
    }

    #[wasm_bindgen(js_name = "toJson")]
    pub fn to_json(&self) -> Result<String, JsValue> {
        crate::snapshot::to_json(&self.inner).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "baseSlots")]
    pub fn base_slots(&self) -> js_sys::Map {
        slots_to_js_map(self.inner.base.populated_slots())
    }

    #[wasm_bindgen(js_name = "semanticSlots")]
    pub fn semantic_slots(&self) -> js_sys::Map {
        slots_to_js_map(self.inner.semantic.populated_slots())
    }

    #[wasm_bindgen(js_name = "diffSlots")]
    pub fn diff_slots(&self) -> js_sys::Map {
        slots_to_js_map(self.inner.diff.populated_slots())
    }

    #[wasm_bindgen(js_name = "surfaceSlots")]
    pub fn surface_slots(&self) -> js_sys::Map {
        slots_to_js_map(self.inner.surface.populated_slots())
    }

    #[wasm_bindgen(js_name = "typographySlots")]
    pub fn typography_slots(&self) -> js_sys::Map {
        slots_to_js_map(self.inner.typography.populated_slots())
    }

    #[wasm_bindgen(js_name = "syntaxSlots")]
    pub fn syntax_slots(&self) -> js_sys::Map {
        slots_to_js_map(self.inner.syntax.populated_slots())
    }

    #[wasm_bindgen(js_name = "editorSlots")]
    pub fn editor_slots(&self) -> js_sys::Map {
        slots_to_js_map(self.inner.editor.populated_slots())
    }

    #[wasm_bindgen(js_name = "terminalAnsiSlots")]
    pub fn terminal_ansi_slots(&self) -> js_sys::Map {
        slots_to_js_map(self.inner.terminal_ansi.populated_slots())
    }
}

#[wasm_bindgen(js_name = "loadPreset")]
pub fn load_preset(id: &str) -> Result<JsPalette, JsValue> {
    crate::registry::load_preset(id)
        .map(|p| JsPalette { inner: p })
        .map_err(to_js_error)
}

#[wasm_bindgen(js_name = "loadPresetCss")]
pub fn load_preset_css(id: &str, prefix: Option<String>) -> Result<String, JsValue> {
    let palette = crate::registry::load_preset(id).map_err(to_js_error)?;
    Ok(crate::css::to_css_custom_properties(&palette, prefix.as_deref()))
}

#[wasm_bindgen(js_name = "loadPresetJson")]
pub fn load_preset_json(id: &str) -> Result<String, JsValue> {
    let palette = crate::registry::load_preset(id).map_err(to_js_error)?;
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
    id: Box<str>,
    name: Box<str>,
    style: Box<str>,
}

impl JsThemeInfo {
    fn from_theme_info(info: &ThemeInfo) -> Self {
        Self {
            id: Box::from(info.id.as_ref()),
            name: Box::from(info.name.as_ref()),
            style: Box::from(info.style.as_ref()),
        }
    }
}

#[wasm_bindgen]
impl JsThemeInfo {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn style(&self) -> String {
        self.style.to_string()
    }
}

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
        Self { inner: Registry::new() }
    }

    pub fn list(&self) -> Vec<JsThemeInfo> {
        self.inner.list().map(JsThemeInfo::from_theme_info).collect()
    }

    pub fn load(&self, id: &str) -> Result<JsPalette, JsValue> {
        self.inner
            .load(id)
            .map(|p| JsPalette { inner: p })
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "addToml")]
    pub fn add_toml(&mut self, toml: &str) -> Result<(), JsValue> {
        self.inner
            .add_toml(toml.to_owned())
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "byStyle")]
    pub fn by_style(&self, style: &str) -> Vec<JsThemeInfo> {
        self.inner.by_style(style).map(JsThemeInfo::from_theme_info).collect()
    }
}
