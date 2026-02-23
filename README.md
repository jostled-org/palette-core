# palette-core

TOML-defined theme system with inheritance and multi-target export.

Themes are authored as TOML presets. Variants inherit from a base and override only what changes. The crate parses, merges, and converts them into typed palettes for any rendering target.

## Usage

```rust
use palette_core::registry::load_preset;
use palette_core::css::to_css_custom_properties;

let palette = load_preset("tokyonight_storm")?;
let css = to_css_custom_properties(&palette, "app");
```

### Terminal (ratatui)

```rust
use palette_core::registry::load_preset;
use palette_core::terminal::to_terminal_theme;

let palette = load_preset("catppuccin")?;
let theme = to_terminal_theme(&palette);
// theme.base["background"], theme.semantic["error"], etc.
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

Core functionality (parsing, merge, CSS export) has no optional dependencies.

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
