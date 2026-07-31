#![allow(dead_code)]

#[cfg(feature = "serde")]
pub fn test_serde_roundtrip<
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + core::fmt::Debug + PartialEq + Eq,
>(
    value: &T,
) {
    test_serde_roundtrip_and_maybe_expect_content(value, None);
}

#[cfg(feature = "serde")]
pub fn test_serde_roundtrip_and_expect_content<
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + core::fmt::Debug + PartialEq + Eq,
>(
    value: &T,
    expected_content: &str,
) {
    test_serde_roundtrip_and_maybe_expect_content(value, Some(expected_content));
}

#[cfg(feature = "serde")]
fn test_serde_roundtrip_and_maybe_expect_content<
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + core::fmt::Debug + PartialEq + Eq,
>(
    value: &T,
    expected_content: Option<&str>,
) {
    let json_str = serde_json::to_string(value).unwrap();
    if let Some(expected_content) = expected_content {
        assert_eq!(json_str, expected_content);
    }
    let deserialized: T = serde_json::from_str(&json_str).unwrap();
    assert_eq!(value, &deserialized)
}
