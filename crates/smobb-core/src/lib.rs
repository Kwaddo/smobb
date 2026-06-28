//! S.M.O.B.B. core engine: pure, non-blocking narrative state machine.
pub mod content;
pub mod decode;
pub mod engine;
pub mod recap;
pub mod save;
pub mod scene;
pub mod speaker;

#[cfg(test)]
mod smoke {
    #[test]
    fn builds() {
        assert_eq!(2 + 2, 4);
    }
}
