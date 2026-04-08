use palette_core::color::Color;
use palette_core::manipulation::{
    blend, lerp_oklab, lerp_oklch, oklab_to_srgb, oklch_to_oklab, srgb_to_oklab, srgb_to_oklch,
};

fn color(hex: &str) -> Color {
    Color::from_hex(hex).unwrap()
}

fn assert_channel_eq(actual: Color, expected: Color, tolerance: u8, label: &str) {
    assert!(
        actual.r.abs_diff(expected.r) <= tolerance
            && actual.g.abs_diff(expected.g) <= tolerance
            && actual.b.abs_diff(expected.b) <= tolerance,
        "{label}: expected {expected:?}, got {actual:?}",
    );
}

// --- HSL roundtrip (via lighten(0.0) as identity) ---

#[test]
fn roundtrip_black() {
    let c = color("#000000");
    assert_channel_eq(c.lighten(0.0), c, 1, "black roundtrip");
}

#[test]
fn roundtrip_white() {
    let c = color("#FFFFFF");
    assert_channel_eq(c.lighten(0.0), c, 1, "white roundtrip");
}

#[test]
fn roundtrip_midgray() {
    let c = color("#808080");
    assert_channel_eq(c.lighten(0.0), c, 1, "midgray roundtrip");
}

#[test]
fn roundtrip_orange_red() {
    let c = color("#FF4500");
    assert_channel_eq(c.lighten(0.0), c, 1, "orange-red roundtrip");
}

// --- lighten ---

#[test]
fn lighten_black_to_midgray() {
    let result = color("#000000").lighten(0.5);
    assert_channel_eq(result, color("#808080"), 1, "black lighten 0.5");
}

#[test]
fn lighten_white_clamps() {
    let result = color("#FFFFFF").lighten(0.5);
    assert_channel_eq(result, color("#FFFFFF"), 0, "white lighten clamps");
}

#[test]
fn lighten_midgray_gets_lighter() {
    let original = color("#808080");
    let lighter = original.lighten(0.1);
    assert!(
        lighter.relative_luminance() > original.relative_luminance(),
        "lighten should increase luminance"
    );
}

// --- darken ---

#[test]
fn darken_white_to_midgray() {
    let result = color("#FFFFFF").darken(0.5);
    assert_channel_eq(result, color("#808080"), 1, "white darken 0.5");
}

#[test]
fn darken_black_clamps() {
    let result = color("#000000").darken(0.5);
    assert_channel_eq(result, color("#000000"), 0, "black darken clamps");
}

#[test]
fn darken_midgray_gets_darker() {
    let original = color("#808080");
    let darker = original.darken(0.1);
    assert!(
        darker.relative_luminance() < original.relative_luminance(),
        "darken should decrease luminance"
    );
}

// --- saturate / desaturate ---

#[test]
fn saturate_gray_uses_hue_zero() {
    // Achromatic gray has hue=0 in HSL. Saturating it produces a reddish tint
    // because the arbitrary hue takes effect once saturation > 0.
    let gray = color("#808080");
    let result = gray.saturate(0.5);
    assert!(
        result.r > result.g,
        "saturated gray should lean red (hue=0): {result:?}"
    );
}

#[test]
fn saturate_increases_saturation() {
    // A muted color should become more vivid
    let muted = color("#996666");
    let vivid = muted.saturate(0.3);
    // The red channel should increase or the others decrease
    assert!(
        vivid.r > muted.r || vivid.g < muted.g || vivid.b < muted.b,
        "saturate should increase vividness: {muted:?} -> {vivid:?}"
    );
}

#[test]
fn desaturate_full_produces_gray() {
    let c = color("#FF0000");
    let result = c.desaturate(1.0);
    // Full desaturation: all channels should be equal (gray)
    assert!(
        result.r.abs_diff(result.g) <= 1 && result.g.abs_diff(result.b) <= 1,
        "desaturate(1.0) should produce gray, got {result:?}"
    );
}

// --- rotate_hue ---

#[test]
fn rotate_zero_identity() {
    let c = color("#FF4500");
    assert_channel_eq(c.rotate_hue(0.0), c, 1, "rotate 0");
}

#[test]
fn rotate_360_identity() {
    let c = color("#FF4500");
    assert_channel_eq(c.rotate_hue(360.0), c, 1, "rotate 360");
}

#[test]
fn rotate_red_180_to_cyan() {
    let result = color("#FF0000").rotate_hue(180.0);
    assert_channel_eq(result, color("#00FFFF"), 1, "red 180 -> cyan");
}

#[test]
fn rotate_negative_equals_positive() {
    let c = color("#FF0000");
    let neg90 = c.rotate_hue(-90.0);
    let pos270 = c.rotate_hue(270.0);
    assert_channel_eq(neg90, pos270, 1, "-90 == 270");
}

// --- blend ---

#[test]
fn blend_alpha_zero_returns_bg() {
    let fg = color("#FF0000");
    let bg = color("#0000FF");
    assert_channel_eq(blend(fg, bg, 0.0), bg, 0, "blend alpha=0");
}

#[test]
fn blend_alpha_one_returns_fg() {
    let fg = color("#FF0000");
    let bg = color("#0000FF");
    assert_channel_eq(blend(fg, bg, 1.0), fg, 0, "blend alpha=1");
}

#[test]
fn blend_half_averages() {
    let fg = color("#FF0000");
    let bg = color("#0000FF");
    let result = blend(fg, bg, 0.5);
    let expected = Color {
        r: 128,
        g: 0,
        b: 128,
    };
    assert_channel_eq(result, expected, 1, "blend alpha=0.5");
}

#[test]
fn blend_clamps_alpha_above_one() {
    let fg = color("#FF0000");
    let bg = color("#0000FF");
    assert_channel_eq(blend(fg, bg, 1.5), fg, 0, "blend alpha>1 clamps");
}

#[test]
fn blend_clamps_alpha_below_zero() {
    let fg = color("#FF0000");
    let bg = color("#0000FF");
    assert_channel_eq(blend(fg, bg, -0.5), bg, 0, "blend alpha<0 clamps");
}

// --- NaN guards ---

#[test]
fn lighten_nan_returns_unchanged() {
    let c = color("#FF4500");
    assert_eq!(c.lighten(f64::NAN), c);
}

#[test]
fn darken_nan_returns_unchanged() {
    let c = color("#FF4500");
    assert_eq!(c.darken(f64::NAN), c);
}

#[test]
fn saturate_nan_returns_unchanged() {
    let c = color("#FF4500");
    assert_eq!(c.saturate(f64::NAN), c);
}

#[test]
fn desaturate_nan_returns_unchanged() {
    let c = color("#FF4500");
    assert_eq!(c.desaturate(f64::NAN), c);
}

#[test]
fn rotate_hue_nan_returns_unchanged() {
    let c = color("#FF4500");
    assert_eq!(c.rotate_hue(f64::NAN), c);
}

#[test]
fn blend_nan_alpha_returns_bg() {
    let fg = color("#FF0000");
    let bg = color("#0000FF");
    assert_eq!(blend(fg, bg, f64::NAN), bg);
}

#[test]
fn lighten_infinity_returns_unchanged() {
    let c = color("#FF4500");
    assert_eq!(c.lighten(f64::INFINITY), c);
}

#[test]
fn blend_infinity_returns_bg() {
    let fg = color("#FF0000");
    let bg = color("#0000FF");
    assert_eq!(blend(fg, bg, f64::INFINITY), bg);
}

// --- OkLab round-trip ---

#[test]
fn oklab_round_trip_black() {
    let c = Color { r: 0, g: 0, b: 0 };
    let lab = srgb_to_oklab(c);
    let back = oklab_to_srgb(lab);
    assert_eq!(back, c);
}

#[test]
fn oklab_round_trip_white() {
    let c = Color {
        r: 255,
        g: 255,
        b: 255,
    };
    let lab = srgb_to_oklab(c);
    let back = oklab_to_srgb(lab);
    assert_eq!(back, c);
}

#[test]
fn oklab_round_trip_primary_colors() {
    for (label, c) in [
        ("red", Color { r: 255, g: 0, b: 0 }),
        ("green", Color { r: 0, g: 255, b: 0 }),
        ("blue", Color { r: 0, g: 0, b: 255 }),
    ] {
        let back = oklab_to_srgb(srgb_to_oklab(c));
        assert_channel_eq(back, c, 1, &format!("{label} oklab round-trip"));
    }
}

#[test]
fn oklab_known_reference_values() {
    // Red (#FF0000)
    let red_lab = srgb_to_oklab(color("#FF0000"));
    assert!((red_lab.l - 0.6279).abs() < 0.002, "red L: {}", red_lab.l);
    assert!((red_lab.a - 0.2249).abs() < 0.002, "red a: {}", red_lab.a);
    assert!((red_lab.b - 0.1264).abs() < 0.002, "red b: {}", red_lab.b);

    // Blue (#0000FF)
    let blue_lab = srgb_to_oklab(color("#0000FF"));
    assert!(
        (blue_lab.l - 0.4520).abs() < 0.002,
        "blue L: {}",
        blue_lab.l
    );
    assert!(
        (blue_lab.a - (-0.0324)).abs() < 0.002,
        "blue a: {}",
        blue_lab.a
    );
    assert!(
        (blue_lab.b - (-0.3115)).abs() < 0.002,
        "blue b: {}",
        blue_lab.b
    );

    // 50% gray (#808080) — L≈0.5999 (plan had incorrect 0.5340)
    let gray_lab = srgb_to_oklab(color("#808080"));
    assert!(
        (gray_lab.l - 0.5999).abs() < 0.002,
        "gray L: {}",
        gray_lab.l
    );
    assert!(gray_lab.a.abs() < 0.002, "gray a: {}", gray_lab.a);
    assert!(gray_lab.b.abs() < 0.002, "gray b: {}", gray_lab.b);
}

#[test]
fn oklab_midpoint_not_muddy() {
    let blue = srgb_to_oklab(color("#0000FF"));
    let yellow = srgb_to_oklab(color("#FFFF00"));
    let mid = lerp_oklab(blue, yellow, 0.5);
    let mid_color = oklab_to_srgb(mid);

    // sRGB linear blend of blue+yellow produces gray-ish
    let srgb_mid = blend(color("#0000FF"), color("#FFFF00"), 0.5);

    // OkLab midpoint should have higher saturation (not muddy gray)
    let mid_sat = channel_spread(mid_color);
    let srgb_sat = channel_spread(srgb_mid);
    assert!(
        mid_sat > srgb_sat,
        "OkLab midpoint should be more saturated than sRGB blend: oklab={mid_sat}, srgb={srgb_sat}"
    );
}

/// Rough measure of how "colorful" a color is (max channel - min channel).
fn channel_spread(c: Color) -> u8 {
    let max = c.r.max(c.g).max(c.b);
    let min = c.r.min(c.g).min(c.b);
    max - min
}

// --- OKLCH ---

#[test]
fn oklch_round_trip_preserves_hue() {
    let red = color("#FF0000");
    let lch = srgb_to_oklch(red);
    // Known OKLab red hue is approximately 29°
    assert!(
        (lch.h - 29.0).abs() < 2.0,
        "red hue should be ~29°, got {}",
        lch.h
    );
    let back = oklab_to_srgb(oklch_to_oklab(lch));
    assert_channel_eq(back, red, 1, "oklch red round-trip");
}

#[test]
fn oklch_shortest_arc_interpolation() {
    // Two colors ~30° apart in hue
    let a = srgb_to_oklch(color("#FF0000")); // hue ~29°
    let b = srgb_to_oklch(color("#FF8800")); // hue ~55° (orange)
    let mid = lerp_oklch(a, b, 0.5);
    let mid_hue = mid.h;

    // Midpoint hue should be between the two input hues
    let (lo, hi) = match a.h < b.h {
        true => (a.h, b.h),
        false => (b.h, a.h),
    };
    assert!(
        mid_hue >= lo && mid_hue <= hi,
        "interpolated hue {mid_hue} should be between {lo} and {hi}"
    );
}

#[test]
fn oklch_hue_wrap_around_zero() {
    // Construct OKLCH values with hues crossing 0°
    let a = srgb_to_oklch(color("#FF0000")); // hue ~29°
    // We need hue ~350°. Manually construct to ensure exact hue values.
    let mut lch_a = a;
    lch_a.h = 350.0;
    let mut lch_b = a;
    lch_b.h = 10.0;

    let mid = lerp_oklch(lch_a, lch_b, 0.5);
    // Should go the short way: 350 -> 0 -> 10, midpoint ~0°
    assert!(
        mid.h > 355.0 || mid.h < 5.0,
        "hue wrap midpoint should be ~0°, got {}",
        mid.h
    );
}

#[test]
fn oklch_achromatic_hue_handled() {
    let black = color("#000000");
    let lch = srgb_to_oklch(black);
    // Achromatic: chroma ~0, hue should be 0 (not NaN)
    assert!(lch.h.is_finite(), "achromatic hue should be finite");
    assert!(
        lch.c.abs() < 0.001,
        "black chroma should be ~0, got {}",
        lch.c
    );

    // Interpolate between achromatic and chromatic
    let red_lch = srgb_to_oklch(color("#FF0000"));
    let mid = lerp_oklch(lch, red_lch, 0.5);
    let mid_color = oklab_to_srgb(oklch_to_oklab(mid));
    // Result should be a valid color (not NaN-corrupted)
    assert!(
        mid_color.r > 0 || mid_color.g > 0 || mid_color.b > 0,
        "interpolation with achromatic should produce valid color"
    );
}
