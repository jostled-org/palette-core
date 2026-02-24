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
let css = format!(":root {{\n{}}}", palette.to_css("app"));
```

```css
:root {
  --app-base-background: #1A1B26;
  --app-base-foreground: #C0CAF5;
  --app-semantic-error: #DB4B4B;
  /* ... */
}
```

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
console.log(palette.name());         // "TokyoNight (Night)"
console.log(palette.toCss("app"));   // CSS custom properties

const css = loadPresetCss("dracula", "app");
```

### 2. Default preset with end-user theme switching

Use a `Registry` to expose all built-in presets. Load a default at startup. Let users pick from the list.

```rust
use palette_core::Registry;

let reg = Registry::new();

// Populate a settings menu
for info in reg.list() {
    println!("{} ({})", info.name, info.style);
}

// Load the user's choice (or fall back to a default)
let user_choice = "catppuccin";
let palette = reg.load(user_choice)?;
```

**CSS — generate all themes for live switching**

```rust
use palette_core::Registry;

let reg = Registry::new();
let mut css = String::new();
for info in reg.list() {
    let palette = reg.load(&info.id)?;
    css.push_str(&format!(
        "[data-theme=\"{}\"] {{\n{}}}\n",
        info.id,
        palette.to_css("app"),
    ));
}
```

Switch themes in the browser by setting a data attribute:

```js
document.documentElement.dataset.theme = "catppuccin";
```

Reference the variables in CSS:

```css
body {
    background: var(--app-base-background);
    color: var(--app-base-foreground);
}
```

**Terminal — swap themes at runtime**

```rust
use palette_core::Registry;
use palette_core::terminal::to_terminal_theme;

let reg = Registry::new();
let theme = to_terminal_theme(&reg.load("tokyonight_storm")?);
```

**WASM**

```js
import { JsRegistry } from "palette-core";

const reg = new JsRegistry();
const themes = reg.list(); // [{id, name, style}, ...]

const palette = reg.load("dracula");
```

### 3. Developer-defined custom presets

Add your own presets — either full themes or variants that inherit from a built-in.

**Variant that inherits from a built-in:**

```toml
# corporate_dark.toml
[meta]
name = "Corporate Dark"
preset_id = "corporate_dark"
schema_version = "1"
style = "dark"
kind = "preset-variant"
inherits = "tokyonight"

[semantic]
error = "#FF3333"
info = "#0099FF"
```

This theme gets all of tokyonight's colors, overriding only the semantic values.

**Full custom preset:**

```toml
# brand.toml
[meta]
name = "Brand Theme"
preset_id = "brand"
schema_version = "1"
style = "light"
kind = "preset-base"

[base]
background = "#FFFFFF"
foreground = "#1A1A1A"

[semantic]
error = "#CC0000"
success = "#008800"
```

**Register and use:**

```rust
use std::path::Path;
use palette_core::Registry;

let mut reg = Registry::new();

// Add a single file
reg.add_file(Path::new("themes/corporate_dark.toml"))?;

// Or add an entire directory of .toml files
reg.add_dir(Path::new("themes/"))?;

// Custom themes appear alongside built-ins
for info in reg.list() {
    println!("{}: {} ({})", info.id, info.name, info.style);
}

// Load like any other theme — inheritance resolves automatically
let palette = reg.load("corporate_dark")?;
```

Custom variants can inherit from built-ins or from other custom presets already in the registry.

**WASM**

```js
import { JsRegistry } from "palette-core";

const reg = new JsRegistry();
reg.addToml(corporateDarkToml);

const palette = reg.load("corporate_dark");
```

### 4. End-user-defined presets

Let your users load their own theme files at runtime. The same registry handles built-in, developer, and user themes.

```rust
use std::path::Path;
use palette_core::Registry;

let mut reg = Registry::new();

// Developer themes ship with the app
reg.add_dir(Path::new("themes/"))?;

// End-user themes loaded from a config directory
let user_themes_dir = dirs::config_dir()
    .map(|d| d.join("myapp/themes"));

if let Some(dir) = user_themes_dir.as_deref() {
    if dir.is_dir() {
        reg.add_dir(dir)?;
    }
}

// All themes — built-in, developer, and user — are in one list
for info in reg.list() {
    println!("{}: {} ({})", info.id, info.name, info.style);
}

let palette = reg.load(&user_selected_theme_id)?;
```

A user preset with the same `preset_id` as an existing theme replaces it, so users can override built-ins or developer themes.

User presets support inheritance too — a user can write a variant that inherits from any theme already in the registry:

```toml
# ~/.config/myapp/themes/my_nord.toml
[meta]
name = "My Nord"
preset_id = "my_nord"
schema_version = "1"
style = "dark"
kind = "preset-variant"
inherits = "nord"

[base]
background = "#1a1a2e"
```

**WASM — user-supplied TOML string**

```js
const reg = new JsRegistry();
reg.addToml(userTomlString);
const palette = reg.load("my_nord");
```

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
