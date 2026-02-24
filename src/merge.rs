use crate::manifest::{ManifestSection, PaletteManifest};

fn merge_sections(primary: &ManifestSection, fallback: &ManifestSection) -> ManifestSection {
    let mut merged = fallback.clone();
    for (key, value) in primary {
        merged.insert(key.clone(), value.clone());
    }
    merged
}

#[cfg(feature = "platform")]
fn merge_platform_sections(
    primary: &crate::manifest::PlatformSections,
    fallback: &crate::manifest::PlatformSections,
) -> crate::manifest::PlatformSections {
    let mut merged = primary.clone();
    for (platform, section) in fallback {
        let existing = merged.entry(platform.clone()).or_default();
        *existing = merge_sections(existing, section);
    }
    merged
}

pub fn merge_manifests(variant: &PaletteManifest, base: &PaletteManifest) -> PaletteManifest {
    PaletteManifest {
        meta: variant.meta.clone(),
        base: merge_sections(&variant.base, &base.base),
        semantic: merge_sections(&variant.semantic, &base.semantic),
        diff: merge_sections(&variant.diff, &base.diff),
        surface: merge_sections(&variant.surface, &base.surface),
        typography: merge_sections(&variant.typography, &base.typography),
        syntax: merge_sections(&variant.syntax, &base.syntax),
        editor: merge_sections(&variant.editor, &base.editor),
        terminal: merge_sections(&variant.terminal, &base.terminal),
        #[cfg(feature = "platform")]
        platform: merge_platform_sections(&variant.platform, &base.platform),
    }
}
