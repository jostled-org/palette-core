# palette-core

TOML-defined theme system with inheritance and multi-target export.

Themes are authored as TOML presets. Variants inherit from a base and override only what changes. The crate parses, merges, and converts them into typed palettes for any rendering target.

## Usage

```rust
use palette_core::registry::load_preset;

let palette = load_preset("tokyonight_storm")?;
```

### CSS custom properties

```rust
use palette_core::css::to_css_custom_properties;
use palette_core::registry::{load_preset, preset_ids};

// Single theme
let props = to_css_custom_properties(&palette, "app");
let css = format!(":root {{\n{props}}}");

// Multiple themes for live switching
let mut css = String::new();
for id in preset_ids() {
    let palette = load_preset(id)?;
    let props = to_css_custom_properties(&palette, "app");
    css.push_str(&format!("[data-theme=\"{id}\"] {{\n{props}}}\n"));
}
```

Single-theme output:

```css
:root {
  --app-base-background: #1a1b26;
  --app-base-foreground: #c0caf5;
  --app-semantic-error: #db4b4b;
  /* ... */
}
```

Multi-theme output scopes each theme under a `data-theme` attribute. Switch themes by setting the attribute on `<html>`:

```html
<html data-theme="tokyonight">
```

```js
document.documentElement.dataset.theme = "catppuccin";
```

Then reference the variables in your components:

```css
body { background: var(--app-base-background); color: var(--app-base-foreground); }
```

The `prefix` argument namespaces every variable, so multiple palettes can coexist.

### Terminal (ratatui)

```rust
use palette_core::registry::load_preset;
use palette_core::terminal::to_terminal_theme;

let palette = load_preset("catppuccin")?;
let theme = to_terminal_theme(&palette);
// theme.base["background"], theme.semantic["error"], etc.
```

### WCAG contrast validation

```rust
use palette_core::registry::load_preset;
use palette_core::contrast::{validate_palette, ContrastLevel};

let palette = load_preset("tokyonight")?;
let violations = validate_palette(&palette, ContrastLevel::AaNormal);
for v in &violations {
    println!("{} on {}: {:.2}:1", v.foreground_label, v.background_label, v.ratio);
}
```

### Color manipulation

```rust
use palette_core::color::Color;
use palette_core::manipulation::blend;

let base = Color::from_hex("#1a1b26")?;

let hover = base.lighten(0.1);
let disabled = base.desaturate(0.3);
let accent = base.rotate_hue(180.0);

let overlay = blend(Color::from_hex("#FF0000")?, base, 0.5);
```

Available methods: `lighten`, `darken`, `saturate`, `desaturate`, `rotate_hue`. All take absolute amounts (CSS model). Non-finite inputs return the color unchanged.

### Snapshot (JSON export)

```rust
use palette_core::registry::load_preset;
use palette_core::snapshot::{to_json, to_json_value};

let palette = load_preset("nord")?;
let json = to_json(&palette)?;
let value = to_json_value(&palette)?; // serde_json::Value
```

### Platform overrides

```rust
use palette_core::registry::load_preset;
use palette_core::platform::from_sections;

let manifest = palette_core::manifest::PaletteManifest::from_toml(toml_str)?;
let overrides = from_sections(&manifest.platform)?;
// overrides["terminal"].background, overrides["web"].foreground, etc.
```

### egui

```rust
use palette_core::registry::load_preset;
use palette_core::egui::to_egui_visuals;

let palette = load_preset("github_dark")?;
ctx.set_visuals(to_egui_visuals(&palette));
```

## Features

| Feature | Dependency | What it adds |
|---------|------------|--------------|
| `terminal` | `ratatui` | `Palette` → `ratatui::style::Color` maps |
| `egui` | `egui` | `Palette` → `egui::Visuals` |
| `snapshot` | `serde_json` | JSON serialization of `Palette` |
| `platform` | — | Parse `[platform.terminal]` / `[platform.web]` overrides |
| `full` | all of the above | Everything |

Core functionality (parsing, merge, CSS export, WCAG contrast checking, color manipulation) has no optional dependencies.

## Bundled presets

| Family | Presets |
|--------|--------|
| Ayu | `ayu_dark`, `ayu_light`, `ayu_mirage` |
| Catppuccin | `catppuccin`, `catppuccin_frappe`, `catppuccin_latte`, `catppuccin_macchiato` |
| Dracula | `dracula` |
| Everforest | `everforest_dark`, `everforest_light` |
| GitHub | `github_dark`, `github_light` |
| Gruvbox | `gruvbox_dark`, `gruvbox_light` |
| Kanagawa | `kanagawa` |
| Monokai | `monokai` |
| Nord | `nord` |
| One | `one_dark`, `one_light` |
| Rosé Pine | `rose_pine`, `rose_pine_dawn`, `rose_pine_moon` |
| Solarized | `solarized_dark`, `solarized_light` |
| TokyoNight | `tokyonight`, `tokyonight_storm`, `tokyonight_day`, `tokyonight_moon` |

All presets are embedded at compile time via `include_str!`. Use `registry::preset_ids()` to list them.

## Preset format

Base presets define all sections. Variants declare `inherits` in `[meta]` and override only differing values.

```toml
[meta]
name = "My Theme Storm"
preset_id = "my_theme_storm"
schema_version = "1"
style = "storm"
kind = "preset-variant"
inherits = "my_theme"

[base]
background = "#24283b"
```

Sections: `base`, `semantic`, `diff`, `surface`, `typography`, `syntax`, `editor`, `terminal`.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
