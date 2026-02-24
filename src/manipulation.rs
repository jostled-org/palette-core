use crate::color::Color;

struct Hsl {
    h: f64, // [0, 360)
    s: f64, // [0, 1]
    l: f64, // [0, 1]
}

fn rgb_to_hsl(color: Color) -> Hsl {
    let r = f64::from(color.r) / 255.0;
    let g = f64::from(color.g) / 255.0;
    let b = f64::from(color.b) / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let delta = max - min;

    if delta == 0.0 {
        return Hsl { h: 0.0, s: 0.0, l };
    }

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

    Hsl { h: h_raw * 60.0, s, l }
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

fn hsl_to_rgb(hsl: Hsl) -> Color {
    if hsl.s == 0.0 {
        let v = clamp_channel(hsl.l);
        return Color { r: v, g: v, b: v };
    }

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
    pub fn lighten(self, amount: f64) -> Self {
        adjust_hsl(self, amount, |hsl, a| hsl.l = (hsl.l + a).clamp(0.0, 1.0))
    }

    pub fn darken(self, amount: f64) -> Self {
        adjust_hsl(self, amount, |hsl, a| hsl.l = (hsl.l - a).clamp(0.0, 1.0))
    }

    pub fn saturate(self, amount: f64) -> Self {
        adjust_hsl(self, amount, |hsl, a| hsl.s = (hsl.s + a).clamp(0.0, 1.0))
    }

    pub fn desaturate(self, amount: f64) -> Self {
        adjust_hsl(self, amount, |hsl, a| hsl.s = (hsl.s - a).clamp(0.0, 1.0))
    }

    pub fn rotate_hue(self, degrees: f64) -> Self {
        adjust_hsl(self, degrees, |hsl, d| hsl.h = (hsl.h + d).rem_euclid(360.0))
    }
}

impl Color {
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
