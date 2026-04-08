use std::collections::HashMap;
use std::sync::Arc;

use crate::manifest::PaletteManifest;

fn merge_map<V: Clone>(
    primary: &HashMap<Arc<str>, V>,
    fallback: &HashMap<Arc<str>, V>,
) -> HashMap<Arc<str>, V> {
    let mut merged = HashMap::with_capacity(primary.len() + fallback.len());
    for (key, value) in primary {
        merged.insert(key.clone(), value.clone());
    }
    for (key, value) in fallback {
        merged.entry(key.clone()).or_insert_with(|| value.clone());
    }
    merged
}

#[cfg(feature = "platform")]
fn merge_platform_sections(
    primary: &crate::manifest::PlatformSections,
    fallback: &crate::manifest::PlatformSections,
) -> crate::manifest::PlatformSections {
    let mut merged = crate::manifest::PlatformSections::new();
    for (platform, section) in primary {
        match fallback.get(platform) {
            Some(fb) => {
                merged.insert(platform.clone(), merge_map(section, fb));
            }
            None => {
                merged.insert(platform.clone(), section.clone());
            }
        }
    }
    for (platform, section) in fallback {
        merged
            .entry(platform.clone())
            .or_insert_with(|| section.clone());
    }
    merged
}

/// Overlay `variant` onto `base`, filling missing slots from the parent.
pub fn merge_manifests(variant: &PaletteManifest, base: &PaletteManifest) -> PaletteManifest {
    PaletteManifest {
        meta: variant.meta.clone(),
        base: merge_map(&variant.base, &base.base),
        semantic: merge_map(&variant.semantic, &base.semantic),
        diff: merge_map(&variant.diff, &base.diff),
        surface: merge_map(&variant.surface, &base.surface),
        typography: merge_map(&variant.typography, &base.typography),
        syntax: merge_map(&variant.syntax, &base.syntax),
        editor: merge_map(&variant.editor, &base.editor),
        terminal: merge_map(&variant.terminal, &base.terminal),
        syntax_style: merge_map(&variant.syntax_style, &base.syntax_style),
        gradient: merge_map(&variant.gradient, &base.gradient),
        #[cfg(feature = "platform")]
        platform: merge_platform_sections(&variant.platform, &base.platform),
    }
}
