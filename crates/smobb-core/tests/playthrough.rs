use smobb_core::content;
use smobb_core::engine::Engine;

#[test]
fn real_content_password_chain_runs() {
    let mut e = Engine::new(content::load());
    let _ = e.start();
    // hub -> hasan -> ali (learn 7549009) -> back
    let _ = e.advance("hasan");
    let _ = e.advance("ali");
    let _ = e.advance("end");
    // hub -> qassim -> 7549009 (learn 9285005) -> back
    let _ = e.advance("qassim");
    let _ = e.advance("7549009");
    let _ = e.advance("end");
    // hub -> josh -> 9285005 (learn K1LL) -> k1ll (learn end7312) -> back
    let _ = e.advance("josh");
    let _ = e.advance("9285005");
    let _ = e.advance("k1ll");
    let _ = e.advance("end");
    // hub -> end7312 (the ending) -> choose save
    let _ = e.advance("end7312");
    let _ = e.advance("save");
    assert!(
        e.recap_entries().iter().any(|r| r.contains("!SAVE")),
        "recap after ending should contain the !SAVE unlock; got: {:?}",
        e.recap_entries()
    );
}
