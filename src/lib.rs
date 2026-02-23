//! TOML-defined theme system with inheritance and multi-target export.

pub mod color;
pub mod error;
pub mod manifest;
pub mod merge;
pub mod palette;
pub mod registry;

pub mod contrast;
pub mod css;

#[cfg(feature = "terminal")]
pub mod terminal;

#[cfg(feature = "platform")]
pub mod platform;

#[cfg(feature = "snapshot")]
pub mod snapshot;

#[cfg(feature = "egui")]
pub mod egui;
