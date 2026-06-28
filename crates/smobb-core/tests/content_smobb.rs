#[test]
fn smobb_loads() {
    let c = smobb_core::content::load();
    let s = c.locations.get("smobb").unwrap();
    assert!(s
        .commands
        .iter()
        .any(|b| b.keys.iter().any(|k| k == "mavis")));
}
