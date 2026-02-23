use palette_core::color::{Color, InvalidHex};

#[test]
fn from_hex_lowercase() {
    let color = Color::from_hex("#1a1b2a").unwrap();
    assert_eq!(color, Color { r: 26, g: 27, b: 42 });
}

#[test]
fn from_hex_uppercase() {
    let color = Color::from_hex("#AABBCC").unwrap();
    assert_eq!(color, Color { r: 170, g: 187, b: 204 });
}

#[test]
fn from_hex_mixed_case() {
    let color = Color::from_hex("#aAbBcC").unwrap();
    assert_eq!(color, Color { r: 170, g: 187, b: 204 });
}

#[test]
fn from_hex_missing_hash() {
    let err = Color::from_hex("1a1b2a").unwrap_err();
    assert!(
        matches!(&err, InvalidHex { value } if value.as_ref() == "1a1b2a"),
        "expected InvalidHex, got: {err:?}",
    );
}

#[test]
fn from_hex_invalid_digits() {
    let err = Color::from_hex("#gggggg").unwrap_err();
    assert!(
        matches!(&err, InvalidHex { value } if value.as_ref() == "#gggggg"),
        "expected InvalidHex, got: {err:?}",
    );
}

#[test]
fn from_hex_wrong_length() {
    let err = Color::from_hex("#abc").unwrap_err();
    assert!(
        matches!(&err, InvalidHex { value } if value.as_ref() == "#abc"),
        "expected InvalidHex, got: {err:?}",
    );
}

#[test]
fn from_hex_empty() {
    let err = Color::from_hex("").unwrap_err();
    assert!(
        matches!(&err, InvalidHex { value } if value.as_ref() == ""),
        "expected InvalidHex, got: {err:?}",
    );
}

#[test]
fn to_hex_uppercase_format() {
    let color = Color { r: 26, g: 27, b: 42 };
    assert_eq!(color.to_hex(), "#1A1B2A");
}

#[test]
fn roundtrip() {
    let original = Color { r: 0, g: 128, b: 255 };
    let hex = original.to_hex();
    let parsed = Color::from_hex(&hex).unwrap();
    assert_eq!(parsed, original);
}

#[test]
fn from_hex_non_ascii_returns_error() {
    assert!(Color::from_hex("#café00").is_err());
}
