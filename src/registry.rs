use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::error::PaletteError;
use crate::manifest::PaletteManifest;
use crate::merge::merge_manifests;
use crate::palette::Palette;

/// Display metadata for a theme, usable without parsing the full TOML.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThemeInfo {
    /// Machine identifier used for lookups.
    pub id: Arc<str>,
    /// Human-readable theme name.
    pub name: Arc<str>,
    /// Visual style tag: `"dark"`, `"light"`, etc.
    pub style: Arc<str>,
}

struct BuiltinInfo {
    id: &'static str,
    name: &'static str,
    style: &'static str,
}

macro_rules! presets {
    ($($id:literal => $file:literal, $name:literal, $style:literal),+ $(,)?) => {
        pub(crate) fn preset_toml(id: &str) -> Option<&'static str> {
            match id {
                $($id => Some(include_str!($file)),)+
                _ => None,
            }
        }

        /// All built-in preset IDs, in declaration order.
        pub fn preset_ids() -> &'static [&'static str] {
            &[$($id),+]
        }

        fn builtin_info() -> &'static [BuiltinInfo] {
            &[$(BuiltinInfo {
                id: $id,
                name: $name,
                style: $style,
            }),+]
        }
    };
}

presets! {
    "ayu_dark"              => "../presets/ayu_dark.toml",              "Ayu Dark",              "dark",
    "ayu_light"             => "../presets/ayu_light.toml",             "Ayu Light",             "light",
    "ayu_mirage"            => "../presets/ayu_mirage.toml",            "Ayu Mirage",            "mirage",
    "catppuccin"            => "../presets/catppuccin.toml",            "Catppuccin Mocha",      "mocha",
    "catppuccin_frappe"     => "../presets/catppuccin_frappe.toml",     "Catppuccin Frappe",     "frappe",
    "catppuccin_latte"      => "../presets/catppuccin_latte.toml",      "Catppuccin Latte",      "latte",
    "catppuccin_macchiato"  => "../presets/catppuccin_macchiato.toml",  "Catppuccin Macchiato",  "macchiato",
    "dracula"               => "../presets/dracula.toml",               "Dracula",               "dark",
    "everforest_dark"       => "../presets/everforest_dark.toml",       "Everforest Dark",       "dark",
    "everforest_light"      => "../presets/everforest_light.toml",      "Everforest Light",      "light",
    "github_dark"           => "../presets/github_dark.toml",           "GitHub Dark",           "dark",
    "github_light"          => "../presets/github_light.toml",          "GitHub Light",          "light",
    "golden_hour"           => "../presets/golden_hour.toml",           "Golden Hour",           "light",
    "golden_hour_dusk"      => "../presets/golden_hour_dusk.toml",      "Golden Hour (Dusk)",    "dark",
    "golden_hour_twilight"  => "../presets/golden_hour_twilight.toml",  "Golden Hour (Twilight)","dark",
    "gruvbox_dark"          => "../presets/gruvbox_dark.toml",          "Gruvbox Dark",          "dark",
    "gruvbox_light"         => "../presets/gruvbox_light.toml",         "Gruvbox Light",         "light",
    "kanagawa"              => "../presets/kanagawa.toml",              "Kanagawa",              "dark",
    "monokai"               => "../presets/monokai.toml",               "Monokai",               "dark",
    "nord"                  => "../presets/nord.toml",                  "Nord",                  "dark",
    "one_dark"              => "../presets/one_dark.toml",              "One Dark",              "dark",
    "one_light"             => "../presets/one_light.toml",             "One Light",             "light",
    "rose_pine"             => "../presets/rose_pine.toml",             "Rose Pine",             "dark",
    "rose_pine_dawn"        => "../presets/rose_pine_dawn.toml",        "Rose Pine Dawn",        "dawn",
    "rose_pine_moon"        => "../presets/rose_pine_moon.toml",        "Rose Pine Moon",        "moon",
    "solarized_dark"        => "../presets/solarized_dark.toml",        "Solarized Dark",        "dark",
    "solarized_light"       => "../presets/solarized_light.toml",       "Solarized Light",       "light",
    "tokyonight"            => "../presets/tokyonight.toml",            "TokyoNight (Night)",    "night",
    "tokyonight_day"        => "../presets/tokyonight_day.toml",        "TokyoNight Day",        "day",
    "tokyonight_moon"       => "../presets/tokyonight_moon.toml",       "TokyoNight Moon",       "moon",
    "tokyonight_storm"      => "../presets/tokyonight_storm.toml",      "TokyoNight Storm",      "storm",
}

// ---------------------------------------------------------------------------
// Shared inheritance resolution
// ---------------------------------------------------------------------------

/// Resolve a TOML theme string into a [`Palette`], applying single-level
/// inheritance if the manifest declares `inherits`.
///
/// Only one level of inheritance is supported: a variant may inherit from
/// a base, but the base itself must be self-contained.
fn resolve_with_inheritance<F>(toml_str: &str, resolve_parent: F) -> Result<Palette, PaletteError>
where
    F: FnOnce(&str) -> Result<PaletteManifest, PaletteError>,
{
    let manifest = PaletteManifest::from_toml(toml_str)?;
    resolve_manifest_impl(&manifest, resolve_parent)
}

/// Resolve a pre-parsed manifest into a [`Palette`], applying single-level
/// inheritance if the manifest declares `inherits`.
///
/// Only one level of inheritance is supported.
fn resolve_manifest_with_inheritance<F>(
    manifest: &PaletteManifest,
    resolve_parent: F,
) -> Result<Palette, PaletteError>
where
    F: FnOnce(&str) -> Result<PaletteManifest, PaletteError>,
{
    resolve_manifest_impl(manifest, resolve_parent)
}

/// Shared body: check inheritance, merge if needed, build palette.
fn resolve_manifest_impl<F>(
    manifest: &PaletteManifest,
    resolve_parent: F,
) -> Result<Palette, PaletteError>
where
    F: FnOnce(&str) -> Result<PaletteManifest, PaletteError>,
{
    let resolved = match manifest.inherits_from() {
        None => return Palette::from_manifest(manifest),
        Some(parent_id) => {
            let parent = resolve_parent(parent_id)?;
            merge_manifests(manifest, &parent)
        }
    };
    Palette::from_manifest(&resolved)
}

// ---------------------------------------------------------------------------
// Standalone preset functions (existing API)
// ---------------------------------------------------------------------------

/// Read a TOML theme file, wrapping I/O errors with the file path.
///
/// The `Arc<str>` path allocation is deferred to the error path so the
/// happy path pays nothing.
fn read_theme_file(path: &Path) -> Result<String, PaletteError> {
    std::fs::read_to_string(path).map_err(|source| PaletteError::Io {
        path: Arc::from(path.to_string_lossy().as_ref()),
        source,
    })
}

/// Load a theme from a TOML file on disk, resolving single-level inheritance
/// from sibling files or built-in presets.
///
/// Only one level of inheritance is supported: a variant may inherit from
/// a base, but the base itself must be self-contained.
pub fn load_preset_file(path: &Path) -> Result<Palette, PaletteError> {
    let toml = read_theme_file(path)?;
    resolve_with_inheritance(&toml, |parent_id| resolve_parent(path, parent_id))
}

fn resolve_parent(child_path: &Path, parent_id: &str) -> Result<PaletteManifest, PaletteError> {
    let sibling = child_path
        .parent()
        .map(|dir| dir.join(format!("{parent_id}.toml")))
        .filter(|p| p.is_file());

    match (sibling, preset_toml(parent_id)) {
        (Some(path), _) => {
            let toml = read_theme_file(&path)?;
            PaletteManifest::from_toml(&toml)
        }
        (None, Some(embedded)) => PaletteManifest::from_toml(embedded),
        (None, None) => Err(PaletteError::UnknownPreset(Arc::from(parent_id))),
    }
}

/// Load a built-in preset by ID, resolving single-level inheritance.
///
/// Returns [`PaletteError::UnknownPreset`] if the ID is not recognized.
pub fn load_preset(id: &str) -> Result<Palette, PaletteError> {
    let toml = preset_toml(id).ok_or_else(|| PaletteError::UnknownPreset(Arc::from(id)))?;
    resolve_with_inheritance(toml, |parent_id| {
        let parent_toml = preset_toml(parent_id)
            .ok_or_else(|| PaletteError::UnknownPreset(Arc::from(parent_id)))?;
        PaletteManifest::from_toml(parent_toml)
    })
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

enum Source {
    Builtin,
    Custom(Box<PaletteManifest>),
}

struct Entry {
    info: ThemeInfo,
    source: Source,
}

/// Unified theme registry combining built-in presets with custom themes.
///
/// Built-in themes carry static metadata (name, style) without parsing TOML.
/// Custom themes are added via files or directories and stored as pre-parsed
/// manifests, avoiding a second TOML parse on load.
///
/// # Thread safety
///
/// `Registry` is `Send` but not `Sync` (interior `RefCell` cache). This is
/// intentional: users can build a registry on one thread and move it to a
/// render thread. Do not replace `Arc` with `Rc` for internal keys — that
/// would make `Registry` `!Send`, pinning it to a single thread.
pub struct Registry {
    entries: Vec<Entry>,
    index: HashMap<Arc<str>, usize>,
    cache: RefCell<HashMap<Arc<str>, Palette>>,
}

impl Registry {
    /// Create a registry pre-populated with all built-in presets.
    pub fn new() -> Self {
        let entries: Vec<Entry> = builtin_info()
            .iter()
            .map(|b| Entry {
                info: ThemeInfo {
                    id: Arc::from(b.id),
                    name: Arc::from(b.name),
                    style: Arc::from(b.style),
                },
                source: Source::Builtin,
            })
            .collect();
        let index = entries
            .iter()
            .enumerate()
            .map(|(i, e)| (Arc::clone(&e.info.id), i))
            .collect();
        Self {
            entries,
            index,
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// All registered themes in insertion order (built-ins first, then custom).
    pub fn list(&self) -> impl Iterator<Item = &ThemeInfo> {
        self.entries.iter().map(|e| &e.info)
    }

    /// Load a palette by ID, resolving single-level inheritance within the
    /// registry.
    ///
    /// Only one level of inheritance is supported: a variant may inherit
    /// from a base, but the base itself must be self-contained.
    pub fn load(&self, id: &str) -> Result<Palette, PaletteError> {
        if let Some(cached) = self.cache.borrow().get(id) {
            return Ok(cached.clone());
        }
        let entry = self.find_entry(id)?;
        let palette = match &entry.source {
            Source::Builtin => {
                let toml_str =
                    preset_toml(id).ok_or_else(|| PaletteError::UnknownPreset(Arc::from(id)))?;
                resolve_with_inheritance(toml_str, |parent_id| self.resolve_manifest(parent_id))?
            }
            Source::Custom(manifest) => resolve_manifest_with_inheritance(manifest, |parent_id| {
                self.resolve_manifest(parent_id)
            })?,
        };
        self.cache
            .borrow_mut()
            .insert(Arc::from(id), palette.clone());
        Ok(palette)
    }

    /// Filter registered themes by style (e.g. "dark", "light").
    pub fn by_style(&self, style: &str) -> impl Iterator<Item = &ThemeInfo> {
        self.entries
            .iter()
            .filter(move |e| e.info.style.as_ref() == style)
            .map(|e| &e.info)
    }

    /// Register a custom theme from a TOML file on disk.
    pub fn add_file(&mut self, path: &Path) -> Result<(), PaletteError> {
        let toml = read_theme_file(path)?;
        self.add_toml(&toml)
    }

    /// Register a custom theme from a TOML string.
    ///
    /// Parses the manifest once and stores it. Subsequent [`load`](Self::load)
    /// calls use the pre-parsed manifest directly.
    pub fn add_toml(&mut self, toml: &str) -> Result<(), PaletteError> {
        let manifest = PaletteManifest::from_toml(toml)?;
        let info = theme_info_from_manifest(&manifest)?;
        self.cache.borrow_mut().remove(&info.id);
        self.upsert_entry(info, Source::Custom(Box::new(manifest)));
        Ok(())
    }

    /// Register all `.toml` files in a directory as custom themes.
    pub fn add_dir(&mut self, dir: &Path) -> Result<(), PaletteError> {
        let dir_arc: Arc<str> = Arc::from(dir.to_string_lossy().as_ref());
        let read_dir = std::fs::read_dir(dir).map_err(|source| PaletteError::Io {
            path: Arc::clone(&dir_arc),
            source,
        })?;

        for entry in read_dir {
            let entry = entry.map_err(|source| PaletteError::Io {
                path: Arc::clone(&dir_arc),
                source,
            })?;
            let path = entry.path();
            match path.extension().and_then(|e| e.to_str()) {
                Some("toml") => self.add_file(&path)?,
                _ => continue,
            }
        }

        Ok(())
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    fn find_entry(&self, id: &str) -> Result<&Entry, PaletteError> {
        self.index
            .get(id)
            .map(|&idx| &self.entries[idx])
            .ok_or_else(|| PaletteError::UnknownPreset(Arc::from(id)))
    }

    fn resolve_manifest(&self, id: &str) -> Result<PaletteManifest, PaletteError> {
        let entry = self.find_entry(id)?;
        match &entry.source {
            Source::Builtin => {
                let toml_str =
                    preset_toml(id).ok_or_else(|| PaletteError::UnknownPreset(Arc::from(id)))?;
                PaletteManifest::from_toml(toml_str)
            }
            Source::Custom(manifest) => Ok(PaletteManifest::clone(manifest)),
        }
    }

    fn upsert_entry(&mut self, info: ThemeInfo, source: Source) {
        match self.index.get(&info.id).copied() {
            Some(idx) => {
                self.entries[idx] = Entry { info, source };
            }
            None => {
                let idx = self.entries.len();
                self.index.insert(Arc::clone(&info.id), idx);
                self.entries.push(Entry { info, source });
            }
        }
    }
}

fn theme_info_from_manifest(manifest: &PaletteManifest) -> Result<ThemeInfo, PaletteError> {
    let meta = manifest.meta.as_ref().ok_or(PaletteError::MissingMeta)?;
    Ok(ThemeInfo {
        id: Arc::clone(&meta.preset_id),
        name: Arc::clone(&meta.name),
        style: Arc::clone(&meta.style),
    })
}
