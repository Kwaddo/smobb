use smobb_core::content;
#[test]
fn bevil_loads() {
    let c = content::load();
    let b = c.locations.get("bevil").unwrap();
    assert!(b
        .commands
        .iter()
        .any(|x| x.keys.iter().any(|k| k == "7789980")));
}
