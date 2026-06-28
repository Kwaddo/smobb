use smobb_core::content;
#[test]
fn bader_loads() {
    let c = content::load();
    let b = c.locations.get("bader").unwrap();
    assert!(b
        .commands
        .iter()
        .any(|x| x.keys.iter().any(|k| k == "tranavaloop")));
}
