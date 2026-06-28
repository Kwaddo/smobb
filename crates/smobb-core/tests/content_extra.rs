#[test]
fn extra_has_twelve_and_map() {
    let raw = include_str!("../../../content/extra.ron");
    let c = smobb_core::content::load();
    assert!(c.scenes.contains_key("parable_map"));
    for n in 1..=12 {
        assert!(raw.contains(&format!("\"{n}\"")), "missing story key {n}");
    }
}
