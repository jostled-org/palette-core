# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-22

Initial release.

### Added

- TOML-defined palette manifest with 8 sections (base, semantic, diff, surface, typography, syntax, editor, terminal)
- Palette inheritance and manifest merging
- 28 built-in presets (Catppuccin, Tokyo Night, Dracula, Nord, Gruvbox, etc.)
- CSS custom property export
- egui `Visuals` mapping (feature: `egui`)
- ratatui terminal theme mapping (feature: `terminal`)
- Hex color parsing with `InvalidHex` error type
