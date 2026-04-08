use crate::color::Color;

/// OKLab perceptual color space (Björn Ottosson).
///
/// L is lightness [0, 1], a and b are chromatic channels (unbounded but
/// typically small for sRGB gamut colors).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OkLab {
    /// Lightness \[0, 1\].
    pub l: f64,
    /// Green–red chromatic channel.
    pub a: f64,
    /// Blue–yellow chromatic channel.
    pub b: f64,
}

/// OKLCH polar form of OKLab.
///
/// L is lightness [0, 1], C is chroma (≥0), h is hue in degrees [0, 360).
/// Achromatic colors (C ≈ 0) have h = 0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OkLch {
    /// Lightness \[0, 1\].
    pub l: f64,
    /// Chroma (≥0).
    pub c: f64,
    /// Hue in degrees \[0, 360).
    pub h: f64,
}

// --- sRGB ↔ linear RGB ---

pub(crate) fn srgb_to_linear(channel: u8) -> f64 {
    let s = f64::from(channel) / 255.0;
    match s <= 0.04045 {
        true => s / 12.92,
        false => ((s + 0.055) / 1.055).powf(2.4),
    }
}

fn linear_to_srgb(c: f64) -> u8 {
    let s = match c <= 0.0031308 {
        true => 12.92 * c,
        false => 1.055 * c.powf(1.0 / 2.4) - 0.055,
    };
    (s * 255.0).round().clamp(0.0, 255.0) as u8
}

// --- sRGB ↔ OKLab (Björn Ottosson matrices) ---

/// Convert an sRGB [`Color`] to [`OkLab`].
pub fn srgb_to_oklab(color: Color) -> OkLab {
    let r = srgb_to_linear(color.r);
    let g = srgb_to_linear(color.g);
    let b = srgb_to_linear(color.b);

    // Linear RGB → LMS (using Ottosson's M1 matrix)
    let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
    let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
    let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

    // Cube root (LMS → LMS')
    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    // LMS' → OKLab (using Ottosson's M2 matrix)
    OkLab {
        l: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        a: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    }
}

/// Convert [`OkLab`] back to an sRGB [`Color`].
pub fn oklab_to_srgb(lab: OkLab) -> Color {
    // OKLab → LMS' (inverse of M2)
    let l_ = lab.l + 0.3963377774 * lab.a + 0.2158037573 * lab.b;
    let m_ = lab.l - 0.1055613458 * lab.a - 0.0638541728 * lab.b;
    let s_ = lab.l - 0.0894841775 * lab.a - 1.2914855480 * lab.b;

    // Cube (LMS' → LMS)
    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    // LMS → linear RGB (inverse of M1)
    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

    Color {
        r: linear_to_srgb(r),
        g: linear_to_srgb(g),
        b: linear_to_srgb(b),
    }
}

// --- OKLab ↔ OKLCH ---

/// Convert [`OkLab`] to [`OkLch`] (polar form).
pub fn oklab_to_oklch(lab: OkLab) -> OkLch {
    let c = (lab.a * lab.a + lab.b * lab.b).sqrt();
    let h = match c < 1e-10 {
        true => 0.0,
        false => lab.b.atan2(lab.a).to_degrees().rem_euclid(360.0),
    };
    OkLch { l: lab.l, c, h }
}

/// Convert [`OkLch`] back to [`OkLab`].
pub fn oklch_to_oklab(lch: OkLch) -> OkLab {
    let h_rad = lch.h.to_radians();
    OkLab {
        l: lch.l,
        a: lch.c * h_rad.cos(),
        b: lch.c * h_rad.sin(),
    }
}

/// Convert an sRGB [`Color`] to [`OkLch`].
pub fn srgb_to_oklch(color: Color) -> OkLch {
    oklab_to_oklch(srgb_to_oklab(color))
}

// --- Interpolation helpers ---

/// Linearly interpolate between two [`OkLab`] values.
pub fn lerp_oklab(a: OkLab, b: OkLab, t: f64) -> OkLab {
    OkLab {
        l: a.l + (b.l - a.l) * t,
        a: a.a + (b.a - a.a) * t,
        b: a.b + (b.b - a.b) * t,
    }
}

/// Interpolate between two [`OkLch`] values with shortest-arc hue.
pub fn lerp_oklch(a: OkLch, b: OkLch, t: f64) -> OkLch {
    let l = a.l + (b.l - a.l) * t;
    let c = a.c + (b.c - a.c) * t;
    let h = shortest_arc_lerp(a.h, b.h, t);
    OkLch { l, c, h }
}

fn shortest_arc_lerp(h0: f64, h1: f64, t: f64) -> f64 {
    let mut diff = h1 - h0;
    match () {
        _ if diff > 180.0 => diff -= 360.0,
        _ if diff < -180.0 => diff += 360.0,
        _ => {}
    }
    (h0 + diff * t).rem_euclid(360.0)
}

pub(crate) struct Hsl {
    pub(crate) h: f64, // [0, 360)
    pub(crate) s: f64, // [0, 1]
    pub(crate) l: f64, // [0, 1]
}

pub(crate) fn rgb_to_hsl(color: Color) -> Hsl {
    let r = f64::from(color.r) / 255.0;
    let g = f64::from(color.g) / 255.0;
    let b = f64::from(color.b) / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let delta = max - min;

    match delta == 0.0 {
        true => Hsl { h: 0.0, s: 0.0, l },
        false => hsl_from_delta(r, g, b, max, l, delta),
    }
}

fn hsl_from_delta(r: f64, g: f64, b: f64, max: f64, l: f64, delta: f64) -> Hsl {
    let s = match l > 0.5 {
        true => delta / (2.0 - 2.0 * l),
        false => delta / (2.0 * l),
    };

    let h_raw = match (max == r, max == g) {
        (true, _) if g >= b => (g - b) / delta,
        (true, _) => (g - b) / delta + 6.0,
        (_, true) => (b - r) / delta + 2.0,
        _ => (r - g) / delta + 4.0,
    };

    Hsl {
        h: h_raw * 60.0,
        s,
        l,
    }
}

fn hue_to_channel(p: f64, q: f64, t: f64) -> f64 {
    let t = t.rem_euclid(1.0);
    match () {
        _ if t < 1.0 / 6.0 => p + (q - p) * 6.0 * t,
        _ if t < 1.0 / 2.0 => q,
        _ if t < 2.0 / 3.0 => p + (q - p) * (2.0 / 3.0 - t) * 6.0,
        _ => p,
    }
}

fn clamp_channel(v: f64) -> u8 {
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}

pub(crate) fn hsl_to_rgb(hsl: Hsl) -> Color {
    match hsl.s == 0.0 {
        true => {
            let v = clamp_channel(hsl.l);
            Color { r: v, g: v, b: v }
        }
        false => hsl_chromatic_to_rgb(hsl),
    }
}

fn hsl_chromatic_to_rgb(hsl: Hsl) -> Color {
    let q = match hsl.l < 0.5 {
        true => hsl.l * (1.0 + hsl.s),
        false => hsl.l + hsl.s - hsl.l * hsl.s,
    };
    let p = 2.0 * hsl.l - q;
    let h = hsl.h / 360.0;
    Color {
        r: clamp_channel(hue_to_channel(p, q, h + 1.0 / 3.0)),
        g: clamp_channel(hue_to_channel(p, q, h)),
        b: clamp_channel(hue_to_channel(p, q, h - 1.0 / 3.0)),
    }
}

fn adjust_hsl(color: Color, amount: f64, adjust: fn(&mut Hsl, f64)) -> Color {
    match amount.is_finite() {
        true => {
            let mut hsl = rgb_to_hsl(color);
            adjust(&mut hsl, amount);
            hsl_to_rgb(hsl)
        }
        false => color,
    }
}

impl Color {
    /// Increase lightness by `amount` (0.0–1.0) in HSL space.
    pub fn lighten(self, amount: f64) -> Self {
        adjust_hsl(self, amount, |hsl, a| hsl.l = (hsl.l + a).clamp(0.0, 1.0))
    }

    /// Decrease lightness by `amount` (0.0–1.0) in HSL space.
    pub fn darken(self, amount: f64) -> Self {
        adjust_hsl(self, amount, |hsl, a| hsl.l = (hsl.l - a).clamp(0.0, 1.0))
    }

    /// Increase saturation by `amount` (0.0–1.0) in HSL space.
    pub fn saturate(self, amount: f64) -> Self {
        adjust_hsl(self, amount, |hsl, a| hsl.s = (hsl.s + a).clamp(0.0, 1.0))
    }

    /// Decrease saturation by `amount` (0.0–1.0) in HSL space.
    pub fn desaturate(self, amount: f64) -> Self {
        adjust_hsl(self, amount, |hsl, a| hsl.s = (hsl.s - a).clamp(0.0, 1.0))
    }

    /// Rotate hue by `degrees` on the HSL color wheel.
    pub fn rotate_hue(self, degrees: f64) -> Self {
        adjust_hsl(self, degrees, |hsl, d| {
            hsl.h = (hsl.h + d).rem_euclid(360.0)
        })
    }
}

impl Color {
    /// Alpha-composite `self` over `bg`. See [`blend`].
    pub fn blend(self, bg: Color, alpha: f64) -> Color {
        blend(self, bg, alpha)
    }
}

/// Alpha-composite `fg` over `bg` in RGB space.
///
/// `alpha` is clamped to `[0, 1]`. Non-finite alpha returns `bg`.
pub fn blend(fg: Color, bg: Color, alpha: f64) -> Color {
    match alpha.is_finite() {
        true => {
            let a = alpha.clamp(0.0, 1.0);
            let mix = |f: u8, b: u8| -> u8 {
                let v = f64::from(b) * (1.0 - a) + f64::from(f) * a;
                v.round().clamp(0.0, 255.0) as u8
            };
            Color {
                r: mix(fg.r, bg.r),
                g: mix(fg.g, bg.g),
                b: mix(fg.b, bg.b),
            }
        }
        false => bg,
    }
}
