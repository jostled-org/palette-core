# palette-core

TOML-defined theme system with inheritance and multi-target export.

Themes are authored as TOML presets. Variants inherit from a base and override only what changes. The crate parses, merges, and converts them into typed palettes for any rendering target.

## Quick start

### 1. Ship a single preset

Pick a built-in theme and render it for your target.

**CSS**

```rust
use palette_core::load_preset;

let palette = load_preset("tokyonight")?;
let css = format!(":root {{\n{}}}", palette.to_css(None));
```

```css
:root {
  --bg: #1A1B26;
  --fg: #C0CAF5;
  --error: #DB4B4B;
  --syn-keyword: #BB9AF7;
  --ed-cursor: #C0CAF5;
  /* ... 95 variables total */
}
```

Prefix is optional — pass `Some("app")` to namespace variables as `--app-bg`, `--app-fg`, etc. See [CSS variables reference](docs/css-variables.md) for the full list.

**Terminal (ratatui)**

```rust
use palette_core::load_preset;
use palette_core::terminal::to_terminal_theme;

let palette = load_preset("catppuccin")?;
let theme = to_terminal_theme(&palette);
// theme.base.background, theme.syntax.keywords, etc.
```

**egui**

```rust
use palette_core::load_preset;
use palette_core::egui::to_egui_visuals;

let palette = load_preset("github_dark")?;
ctx.set_visuals(to_egui_visuals(&palette));
```

**JSON**

```rust
use palette_core::load_preset;

let palette = load_preset("nord")?;
let json = palette.to_json()?;
```

**WASM**

```js
import { loadPreset, loadPresetCss } from "palette-core";

const palette = loadPreset("tokyonight");
console.log(palette.name());       // "TokyoNight (Night)"
console.log(palette.toCss());      // CSS custom properties (no prefix)

const css = loadPresetCss("dracula");           // no prefix
const prefixed = loadPresetCss("dracula", "app"); // --app-bg, --app-fg, etc.
```

### Beyond single presets

The [guide](docs/guide.md) covers:

- **Theme switching** — `Registry` for listing presets, generating all themes as CSS `[data-theme]` selectors, swapping at runtime
- **Developer-defined presets** — custom base themes and variants that inherit from built-ins
- **End-user presets** — loading user-supplied TOML files at runtime, with full inheritance support

## Utilities

### WCAG contrast validation

```rust
use palette_core::{load_preset, ContrastLevel};
use palette_core::contrast::validate_palette;

let palette = load_preset("tokyonight")?;
let violations = validate_palette(&palette, ContrastLevel::AaNormal);
for v in &violations {
    println!("{} on {}: {:.2}:1", v.foreground_label, v.background_label, v.ratio);
}
```

### Color manipulation

```rust
use palette_core::Color;

let base = Color::from_hex("#1a1b26")?;

let hover = base.lighten(0.1);
let disabled = base.desaturate(0.3);
let accent = base.rotate_hue(180.0);
let overlay = base.blend(Color::from_hex("#FF0000")?, 0.5);
let ratio = base.contrast_ratio(&Color::from_hex("#FFFFFF")?);
```

Methods: `lighten`, `darken`, `saturate`, `desaturate`, `rotate_hue`, `blend`, `contrast_ratio`, `meets_level`. Manipulation methods take absolute amounts (CSS model). Non-finite inputs return the color unchanged.

### Platform overrides

```rust
let manifest = palette_core::manifest::PaletteManifest::from_toml(toml_str)?;
let overrides = palette_core::platform::from_sections(&manifest.platform)?;
// overrides["terminal"].background, overrides["web"].foreground, etc.
```

## Features

| Feature | Dependency | What it adds |
|---------|------------|--------------|
| `terminal` | `ratatui` | `Palette` → `ratatui::style::Color` maps |
| `egui` | `egui` | `Palette` → `egui::Visuals` |
| `snapshot` | `serde_json` | JSON serialization of `Palette` |
| `platform` | — | Parse `[platform.terminal]` / `[platform.web]` overrides |
| `wasm` | `wasm-bindgen`, `js-sys` | JavaScript bindings via `wasm-bindgen` (includes `snapshot`) |
| `full` | all except `wasm` | `terminal` + `egui` + `snapshot` + `platform` |

Core functionality (parsing, merge, CSS export, WCAG contrast, color manipulation) requires no optional dependencies.

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

All presets are embedded at compile time via `include_str!`. Use `preset_ids()` to list them.

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
