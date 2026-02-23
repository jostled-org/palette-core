#![allow(dead_code)]

use std::collections::BTreeMap;
use std::sync::Arc;

use palette_core::manifest::PaletteManifest;

pub fn load_preset(name: &str) -> PaletteManifest {
    let path = format!("presets/{name}.toml");
    let content = std::fs::read_to_string(&path).unwrap();
    PaletteManifest::from_toml(&content).unwrap()
}

pub fn manifest_with_base(base: BTreeMap<Arc<str>, Arc<str>>) -> PaletteManifest {
    PaletteManifest {
        meta: None,
        base,
        semantic: BTreeMap::new(),
        diff: BTreeMap::new(),
        surface: BTreeMap::new(),
        typography: BTreeMap::new(),
        syntax: BTreeMap::new(),
        editor: BTreeMap::new(),
        terminal: BTreeMap::new(),
        #[cfg(feature = "platform")]
        platform: BTreeMap::new(),
    }
}
