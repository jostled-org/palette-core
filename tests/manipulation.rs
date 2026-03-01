use palette_core::color::Color;
use palette_core::manipulation::blend;

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
