use smobb_core::content;
#[test]
fn ali_loads_with_canon() {
    let c = content::load();
    assert!(c.scenes.contains_key("ali_master_control"));
    let ali = c.locations.get("ali").unwrap();
    assert!(ali
        .commands
        .iter()
        .any(|b| b.keys.iter().any(|k| k == "quint9440")));
}
