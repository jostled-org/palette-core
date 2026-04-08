#![allow(dead_code)]

use palette_core::manifest::{ManifestSection, PaletteManifest};
use std::collections::HashMap;

pub fn load_preset(name: &str) -> PaletteManifest {
    let path = format!("presets/{name}.toml");
    let content = std::fs::read_to_string(&path).unwrap();
    PaletteManifest::from_toml(&content).unwrap()
}

pub fn manifest_with_base(base: ManifestSection) -> PaletteManifest {
    PaletteManifest {
        meta: None,
        base,
        semantic: HashMap::new(),
        diff: HashMap::new(),
        surface: HashMap::new(),
        typography: HashMap::new(),
        syntax: HashMap::new(),
        editor: HashMap::new(),
        terminal: HashMap::new(),
        syntax_style: HashMap::new(),
        gradient: HashMap::new(),
        #[cfg(feature = "platform")]
        platform: Default::default(),
    }
}
