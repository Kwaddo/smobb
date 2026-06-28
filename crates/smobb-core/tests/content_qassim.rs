#[test]
fn qassim_loads() {
    let c = smobb_core::content::load();
    let q = c.locations.get("qassim").unwrap();
    assert!(q
        .commands
        .iter()
        .any(|b| b.keys.iter().any(|k| k == "7549009")));
}
