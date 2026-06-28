#[test]
fn josh_fate_exists() {
    let c = smobb_core::content::load();
    assert!(c.scenes.contains_key("josh_fate"));
    assert!(c.scenes.contains_key("bader_josh"));
}
