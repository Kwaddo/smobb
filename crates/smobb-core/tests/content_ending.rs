#[test]
fn ending_uses_hameed() {
    let raw = include_str!("../../../content/ending.ron");
    assert!(raw.contains("QASSIM HAMEED"));
    assert!(!raw.contains("QASSIM HUJAIR"));
    assert!(smobb_core::content::load().scenes.contains_key("ending"));
}
