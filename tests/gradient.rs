use palette_core::color::Color;
use palette_core::error::PaletteError;
use palette_core::gradient::{ColorSpace, Gradient, GradientStop};
use palette_core::manifest::PaletteManifest;
use palette_core::palette::Palette;

fn color(hex: &str) -> Color {
    Color::from_hex(hex).unwrap()
}

fn stop(hex: &str, position: f64) -> GradientStop {
    GradientStop {
        color: color(hex),
        position,
    }
}

// 2.T1: two_stop_gradient_endpoints
#[test]
fn two_stop_gradient_endpoints() {
    let g = Gradient::new(
        vec![stop("#000000", 0.0), stop("#FFFFFF", 1.0)],
        ColorSpace::OkLab,
    )
    .unwrap();
    assert_eq!(g.at(0.0), color("#000000"));
    assert_eq!(g.at(1.0), color("#FFFFFF"));
}

// 2.T2: two_stop_gradient_midpoint
#[test]
fn two_stop_gradient_midpoint() {
    let g = Gradient::new(
        vec![stop("#000000", 0.0), stop("#FFFFFF", 1.0)],
        ColorSpace::OkLab,
    )
    .unwrap();
    let mid = g.at(0.5);
    // Midpoint should be approximately middle gray but NOT the sRGB midpoint
    let srgb_mid = Color {
        r: 128,
        g: 128,
        b: 128,
    };
    // OkLab perceptual midpoint differs from sRGB linear midpoint
    assert_ne!(
        mid, srgb_mid,
        "OkLab midpoint should differ from sRGB midpoint"
    );
    // Should still be gray-ish (all channels roughly equal)
    assert!(
        mid.r.abs_diff(mid.g) <= 2 && mid.g.abs_diff(mid.b) <= 2,
        "midpoint should be gray-ish: {mid:?}"
    );
}

// 2.T3: three_stop_gradient_segments
#[test]
fn three_stop_gradient_segments() {
    let g = Gradient::new(
        vec![
            stop("#FF0000", 0.0),
            stop("#00FF00", 0.5),
            stop("#0000FF", 1.0),
        ],
        ColorSpace::OkLab,
    )
    .unwrap();

    // At 0.25: between red and green
    let q1 = g.at(0.25);
    assert!(
        q1.r > 0 && q1.g > 0,
        "at 0.25 should blend red and green: {q1:?}"
    );

    // At 0.75: between green and blue
    let q3 = g.at(0.75);
    assert!(
        q3.g > 0 || q3.b > 0,
        "at 0.75 should blend green and blue: {q3:?}"
    );

    // At 0.5: exactly on the green stop
    let mid = g.at(0.5);
    assert_eq!(mid, color("#00FF00"), "at 0.5 should be exactly green");
}

// 2.T4: gradient_clamps_out_of_range
#[test]
fn gradient_clamps_out_of_range() {
    let g = Gradient::new(
        vec![stop("#FF0000", 0.0), stop("#0000FF", 1.0)],
        ColorSpace::OkLab,
    )
    .unwrap();
    assert_eq!(g.at(-0.5), color("#FF0000"));
    assert_eq!(g.at(1.5), color("#0000FF"));
    assert_eq!(g.at(f64::NAN), color("#FF0000"));
}

// 2.T5: sample_returns_evenly_spaced_colors
#[test]
fn sample_returns_evenly_spaced_colors() {
    let g = Gradient::new(
        vec![stop("#000000", 0.0), stop("#FFFFFF", 1.0)],
        ColorSpace::OkLab,
    )
    .unwrap();
    let samples = g.sample(5);
    assert_eq!(samples.len(), 5);
    assert_eq!(samples[0], color("#000000"));
    assert_eq!(samples[4], color("#FFFFFF"));
    // Monotonically increasing luminance
    for i in 1..samples.len() {
        assert!(
            samples[i].relative_luminance() >= samples[i - 1].relative_luminance(),
            "luminance should increase: {:?} vs {:?}",
            samples[i - 1],
            samples[i]
        );
    }
}

// 2.T6: sample_two_returns_exact_endpoints
#[test]
fn sample_two_returns_exact_endpoints() {
    let g = Gradient::new(
        vec![stop("#FF0000", 0.0), stop("#0000FF", 1.0)],
        ColorSpace::OkLab,
    )
    .unwrap();
    let samples = g.sample(2);
    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0], color("#FF0000"));
    assert_eq!(samples[1], color("#0000FF"));
}

// 2.T7: sample_one_returns_first_stop
#[test]
fn sample_one_returns_first_stop() {
    let g = Gradient::new(
        vec![stop("#FF0000", 0.0), stop("#0000FF", 1.0)],
        ColorSpace::OkLab,
    )
    .unwrap();
    let samples = g.sample(1);
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0], g.at(0.0));
}

// 2.T8: sample_zero_returns_empty
#[test]
fn sample_zero_returns_empty() {
    let g = Gradient::new(
        vec![stop("#FF0000", 0.0), stop("#0000FF", 1.0)],
        ColorSpace::OkLab,
    )
    .unwrap();
    let samples = g.sample(0);
    assert!(samples.is_empty());
}

// 2.T9: gradient_requires_at_least_two_stops
#[test]
fn gradient_requires_at_least_two_stops() {
    let empty: Vec<GradientStop> = vec![];
    assert!(matches!(
        Gradient::new(empty, ColorSpace::OkLab),
        Err(PaletteError::InsufficientStops { count: 0 })
    ));

    assert!(matches!(
        Gradient::new(vec![stop("#FF0000", 0.0)], ColorSpace::OkLab),
        Err(PaletteError::InsufficientStops { count: 1 })
    ));

    assert!(
        Gradient::new(
            vec![stop("#FF0000", 0.0), stop("#0000FF", 1.0)],
            ColorSpace::OkLab
        )
        .is_ok()
    );
}

#[test]
fn gradient_requires_positions_within_unit_interval() {
    assert!(matches!(
        Gradient::new(
            vec![stop("#FF0000", -0.1), stop("#0000FF", 1.0)],
            ColorSpace::OkLab
        ),
        Err(PaletteError::InvalidGradientPosition { position }) if position == -0.1
    ));

    assert!(matches!(
        Gradient::new(
            vec![stop("#FF0000", 0.0), stop("#0000FF", 1.1)],
            ColorSpace::OkLab
        ),
        Err(PaletteError::InvalidGradientPosition { position }) if position == 1.1
    ));
}

// 2.T10: stops_accessor_returns_original_stops
#[test]
fn stops_accessor_returns_original_stops() {
    let stops = vec![
        stop("#FF0000", 0.0),
        stop("#00FF00", 0.5),
        stop("#0000FF", 1.0),
    ];
    let g = Gradient::new(stops.clone(), ColorSpace::OkLab).unwrap();
    let returned = g.stops();
    assert_eq!(returned.len(), 3);
    for (got, expected) in returned.iter().zip(&stops) {
        assert_eq!(got.color, expected.color);
        assert!((got.position - expected.position).abs() < f64::EPSILON);
    }
    assert_eq!(g.space(), ColorSpace::OkLab);
}

// 6.T1: oklch_gradient_preserves_chroma
#[test]
fn oklch_gradient_preserves_chroma() {
    let red = stop("#FF0000", 0.0);
    let blue = stop("#0000FF", 1.0);

    let oklab_gradient = Gradient::new(vec![red, blue], ColorSpace::OkLab).unwrap();
    let oklch_gradient = Gradient::new(vec![red, blue], ColorSpace::OkLch).unwrap();

    let oklab_mid = oklab_gradient.at(0.5);
    let oklch_mid = oklch_gradient.at(0.5);

    // OKLCH preserves chroma through hue rotation; OkLab can reduce it.
    // Compare chroma (saturation) of midpoints using OKLCH representation.
    let oklab_mid_lch = palette_core::manipulation::srgb_to_oklch(oklab_mid);
    let oklch_mid_lch = palette_core::manipulation::srgb_to_oklch(oklch_mid);

    assert!(
        oklch_mid_lch.c > oklab_mid_lch.c,
        "OKLCH midpoint chroma ({}) should exceed OkLab midpoint chroma ({})",
        oklch_mid_lch.c,
        oklab_mid_lch.c,
    );
}

// 6.T2: oklch_gradient_from_toml
#[test]
fn oklch_gradient_from_toml() {
    let toml = r##"
[base]
background = "#FF0000"
foreground = "#0000FF"

[gradient.vibrant]
stops = ["base.background", "base.foreground"]
space = "oklch"

[gradient.smooth]
stops = ["base.background", "base.foreground"]
"##;
    let manifest = PaletteManifest::from_toml(toml).unwrap();
    let palette = Palette::from_manifest(&manifest).unwrap();
    let resolved = palette.resolve();

    let vibrant = resolved
        .gradient("vibrant")
        .expect("gradient 'vibrant' should exist");
    let smooth = resolved
        .gradient("smooth")
        .expect("gradient 'smooth' should exist");

    assert_eq!(vibrant.space(), ColorSpace::OkLch);
    assert_eq!(smooth.space(), ColorSpace::OkLab);

    // The midpoints must differ because the two spaces interpolate differently
    let vibrant_mid = vibrant.at(0.5);
    let smooth_mid = smooth.at(0.5);
    assert_ne!(
        vibrant_mid, smooth_mid,
        "OKLCH and OkLab midpoints should differ for red→blue"
    );
}

// 4.T4: end_to_end_toml_to_interpolation
#[test]
fn end_to_end_toml_to_interpolation() {
    let toml = r##"
[base]
background = "#000000"
foreground = "#FFFFFF"

[gradient.fade]
stops = ["base.background", "base.foreground"]
"##;
    let manifest = PaletteManifest::from_toml(toml).unwrap();
    let palette = Palette::from_manifest(&manifest).unwrap();
    let resolved = palette.resolve();
    let gradient = resolved
        .gradient("fade")
        .expect("gradient 'fade' should exist");
    let mid = gradient.at(0.5);
    // Midpoint should differ from both endpoints
    assert_ne!(mid, Color { r: 0, g: 0, b: 0 });
    assert_ne!(
        mid,
        Color {
            r: 255,
            g: 255,
            b: 255,
        }
    );
    // Should be a valid non-black color (some luminance)
    assert!(mid.r > 0 || mid.g > 0 || mid.b > 0);
}

// 7.T1: gradient_to_css_two_stops
#[test]
fn gradient_to_css_two_stops() {
    let g = Gradient::new(
        vec![stop("#000000", 0.0), stop("#FFFFFF", 1.0)],
        ColorSpace::OkLab,
    )
    .unwrap();
    let css = g.to_css();
    assert!(
        css.starts_with("linear-gradient(in oklab,"),
        "should start with 'linear-gradient(in oklab,': {css}"
    );
    assert!(css.contains("#000000"), "should contain #000000: {css}");
    assert!(css.contains("#FFFFFF"), "should contain #FFFFFF: {css}");
    assert!(css.ends_with(')'), "should end with ')': {css}");
}

// 7.T2: gradient_to_css_explicit_positions
#[test]
fn gradient_to_css_explicit_positions() {
    let g = Gradient::new(
        vec![
            stop("#FF0000", 0.0),
            stop("#00FF00", 0.3),
            stop("#0000FF", 1.0),
        ],
        ColorSpace::OkLab,
    )
    .unwrap();
    let css = g.to_css();
    assert!(css.contains("0%"), "should contain 0%: {css}");
    assert!(css.contains("30%"), "should contain 30%: {css}");
    assert!(css.contains("100%"), "should contain 100%: {css}");
}

// 7.T3: gradient_to_css_oklch_space
#[test]
fn gradient_to_css_oklch_space() {
    let g = Gradient::new(
        vec![stop("#FF0000", 0.0), stop("#0000FF", 1.0)],
        ColorSpace::OkLch,
    )
    .unwrap();
    let css = g.to_css();
    assert!(
        css.starts_with("linear-gradient(in oklch,"),
        "should start with 'linear-gradient(in oklch,': {css}"
    );
}

// 7.T4: gradient_to_css_evenly_spaced_omits_positions
#[test]
fn gradient_to_css_evenly_spaced_omits_positions() {
    let g = Gradient::new(
        vec![stop("#FF0000", 0.0), stop("#0000FF", 1.0)],
        ColorSpace::OkLab,
    )
    .unwrap();
    let css = g.to_css();
    // Evenly spaced stops should not contain percentage positions
    assert!(
        !css.contains('%'),
        "evenly spaced should omit positions: {css}"
    );
}
