#[test]
fn guide_no_swing_and_redacted_fixed() {
    let raw = include_str!("../../../content/guide.ron");
    assert!(!raw.contains(".png"));
    assert!(!raw.contains("REDATCTED"));
    assert!(raw.contains("HAMEED"));
    assert!(smobb_core::content::load().locations.contains_key("guide"));
}
