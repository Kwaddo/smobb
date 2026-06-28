#[test]
fn hasan_cutoff_filled_and_no_pkg_leak() {
    let raw = include_str!("../../../content/hasan.ron");
    assert!(raw.contains("I'm sorry that it had to come to this."));
    assert!(!raw.contains("com.company"));
    let _ = smobb_core::content::load();
}
