use smobb_core::content;
#[test]
fn hub_loads() {
    let c = content::load();
    let hub = c.locations.get("hub").expect("hub location");
    assert!(hub
        .commands
        .iter()
        .any(|b| b.keys.iter().any(|k| k == "mavis")));
    assert!(hub
        .commands
        .iter()
        .any(|b| b.keys.iter().any(|k| k == "end")));
}
