//! Multi-stop color gradients with perceptual interpolation.
//!
//! Gradients are defined in theme TOML files and resolved alongside the rest of
//! the palette. Stops can be hex literals or token references to palette colors.
//! Interpolation runs in OKLab (default) or OKLCH color space.
//!
//! # Defining gradients in TOML
//!
//! ```toml
//! [gradient.heat]
//! stops = ["#2563EB", "#F59E0B", "#EF4444"]
//! space = "oklch"
//!
//! [gradient.brand]
//! stops = ["base.background", "semantic.info"]
//! ```
//!
//! # Interpolating
//!
//! ```
//! use palette_core::gradient::{Gradient, GradientStop, ColorSpace};
//! use palette_core::Color;
//!
//! let stops = vec![
//!     GradientStop { color: Color::from_hex("#000000").unwrap(), position: 0.0 },
//!     GradientStop { color: Color::from_hex("#FFFFFF").unwrap(), position: 1.0 },
//! ];
//! let gradient = Gradient::new(stops, ColorSpace::OkLab).unwrap();
//!
//! let mid = gradient.at(0.5);   // perceptual midpoint
//! let css = gradient.to_css();  // linear-gradient(in oklab, #000000, #FFFFFF)
//! ```

use std::sync::Arc;

use crate::color::Color;
use crate::error::PaletteError;
use crate::manipulation::{
    lerp_oklab, lerp_oklch, oklab_to_srgb, oklch_to_oklab, srgb_to_oklab, srgb_to_oklch,
};

/// Interpolation color space for gradient stops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
pub enum ColorSpace {
    /// Perceptually uniform interpolation (default).
    #[default]
    OkLab,
    /// Polar interpolation with shortest-arc hue travel.
    OkLch,
}

/// A color reference in an unresolved gradient definition.
///
/// Produced during `Palette::from_manifest()`. Token references are validated
/// against known section/field names at parse time so that `resolve()` can
/// look them up infallibly.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
pub enum GradientColor {
    /// A concrete hex color parsed at load time.
    Literal(Color),
    /// A reference to a palette token: `"section.field"`.
    Token {
        /// Section name (e.g. `"base"`).
        section: Arc<str>,
        /// Field name within that section (e.g. `"foreground"`).
        field: Arc<str>,
    },
}

/// An unresolved gradient definition with typed stops.
///
/// Stored on [`Palette`](crate::Palette) after `from_manifest()`.
/// Each stop is a `(GradientColor, position)` pair with positions in \[0, 1\].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
pub struct GradientDef {
    stops: Box<[(GradientColor, f64)]>,
    space: ColorSpace,
}

impl GradientDef {
    /// Build a gradient definition from validated stops.
    ///
    /// Callers must ensure ≥ 2 stops with sorted positions.
    pub(crate) fn new(stops: Box<[(GradientColor, f64)]>, space: ColorSpace) -> Self {
        Self { stops, space }
    }

    /// The typed stops in this gradient definition.
    pub fn stops(&self) -> &[(GradientColor, f64)] {
        &self.stops
    }

    /// The interpolation color space.
    pub fn space(&self) -> ColorSpace {
        self.space
    }
}

/// A single stop in a gradient.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
pub struct GradientStop {
    /// The color at this stop.
    pub color: Color,
    /// Position in \[0, 1\].
    pub position: f64,
}

/// A resolved gradient with concrete color stops, ready for interpolation.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "snapshot", derive(serde::Serialize))]
pub struct Gradient {
    stops: Box<[GradientStop]>,
    space: ColorSpace,
}

impl Gradient {
    /// Construct a gradient from pre-validated stops (skips validation).
    ///
    /// Callers must guarantee ≥ 2 stops with monotonically increasing positions.
    /// Used when stops were already validated at parse time.
    pub(crate) fn new_unchecked(stops: impl Into<Box<[GradientStop]>>, space: ColorSpace) -> Self {
        Self {
            stops: stops.into(),
            space,
        }
    }

    /// Construct a gradient from concrete color stops.
    ///
    /// Returns `Err` if fewer than 2 stops or positions are not monotonically
    /// increasing.
    pub fn new(
        stops: impl Into<Box<[GradientStop]>>,
        space: ColorSpace,
    ) -> Result<Self, PaletteError> {
        let stops = stops.into();
        match stops.len() < 2 {
            true => return Err(PaletteError::InsufficientStops { count: stops.len() }),
            false => {}
        }
        for stop in stops.iter() {
            match stop.position.is_nan() || !(0.0..=1.0).contains(&stop.position) {
                true => {
                    return Err(PaletteError::InvalidGradientPosition {
                        position: stop.position,
                    });
                }
                false => {}
            }
        }
        let sorted = stops.windows(2).all(|w| w[0].position <= w[1].position);
        match sorted {
            true => Ok(Self { stops, space }),
            false => Err(PaletteError::UnsortedStops),
        }
    }

    /// Interpolate the gradient at position `t` (clamped to \[0, 1\]).
    /// NaN returns the first stop color.
    pub fn at(&self, t: f64) -> Color {
        let t = match t.is_nan() {
            true => 0.0,
            false => t.clamp(0.0, 1.0),
        };
        interpolate_at(&self.stops, self.space, t)
    }

    /// Sample `n` evenly spaced colors from the gradient.
    ///
    /// - `n == 0`: returns empty slice
    /// - `n == 1`: returns `[at(0.0)]`
    /// - `n >= 2`: endpoints guaranteed exact
    pub fn sample(&self, n: usize) -> Box<[Color]> {
        match n {
            0 => Box::new([]),
            1 => Box::new([self.at(0.0)]),
            _ => {
                let divisor = (n - 1) as f64;
                (0..n)
                    .map(|i| self.at(i as f64 / divisor))
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            }
        }
    }

    /// The color stops in this gradient.
    pub fn stops(&self) -> &[GradientStop] {
        &self.stops
    }

    /// The interpolation color space.
    pub fn space(&self) -> ColorSpace {
        self.space
    }

    /// Emit a CSS `linear-gradient()` expression.
    ///
    /// Positions are omitted when stops are evenly spaced (CSS default).
    /// No direction is included — callers prepend one if needed.
    pub fn to_css(&self) -> Box<str> {
        use std::fmt::Write;

        let space_str = match self.space {
            ColorSpace::OkLab => "oklab",
            ColorSpace::OkLch => "oklch",
        };

        let mut buf = String::with_capacity(64);
        let _ = write!(buf, "linear-gradient(in {space_str},");

        let evenly_spaced = is_evenly_spaced(&self.stops);

        for (i, stop) in self.stops.iter().enumerate() {
            let hex = stop.color.to_hex();
            match evenly_spaced {
                true => {
                    let _ = write!(buf, " {hex}");
                }
                false => {
                    let pct = stop.position * 100.0;
                    let _ = write!(buf, " {hex} {pct}%");
                }
            }
            match i < self.stops.len() - 1 {
                true => buf.push(','),
                false => {}
            }
        }

        buf.push(')');
        buf.into_boxed_str()
    }
}

/// Check whether stops are evenly spaced (equal intervals from first to last).
fn is_evenly_spaced(stops: &[GradientStop]) -> bool {
    match stops.len() {
        0..=2 => true,
        n => {
            let first = stops[0].position;
            let last = stops[n - 1].position;
            let step = (last - first) / (n - 1) as f64;
            stops
                .iter()
                .enumerate()
                .all(|(i, s)| (s.position - (first + step * i as f64)).abs() < 1e-9)
        }
    }
}

/// Find the bounding stops for `t` and interpolate.
fn interpolate_at(stops: &[GradientStop], space: ColorSpace, t: f64) -> Color {
    // Exact endpoint checks
    let first = stops[0];
    let last = stops[stops.len() - 1];
    match () {
        _ if t <= first.position => return first.color,
        _ if t >= last.position => return last.color,
        _ => {}
    }

    // Find the segment containing t via binary search on positions
    let idx = match stops[1..].iter().position(|s| s.position >= t) {
        Some(i) => i,
        None => return last.color,
    };

    let a = &stops[idx];
    let b = &stops[idx + 1];

    // Exact stop hit — no interpolation needed
    let span = b.position - a.position;
    match span <= f64::EPSILON {
        true => return a.color,
        false => {}
    }

    let local_t = (t - a.position) / span;
    interpolate_colors(a.color, b.color, space, local_t)
}

fn interpolate_colors(a: Color, b: Color, space: ColorSpace, t: f64) -> Color {
    match space {
        ColorSpace::OkLab => {
            let lab_a = srgb_to_oklab(a);
            let lab_b = srgb_to_oklab(b);
            oklab_to_srgb(lerp_oklab(lab_a, lab_b, t))
        }
        ColorSpace::OkLch => {
            let lch_a = srgb_to_oklch(a);
            let lch_b = srgb_to_oklch(b);
            oklab_to_srgb(oklch_to_oklab(lerp_oklch(lch_a, lch_b, t)))
        }
    }
}
