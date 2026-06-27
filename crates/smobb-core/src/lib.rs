//! S.M.O.B.B. core engine: pure, non-blocking narrative state machine.
pub mod speaker;
pub mod scene;
pub mod content;
pub mod recap;
pub mod engine;

#[cfg(test)]
mod smoke {
    #[test]
    fn builds() {
        assert_eq!(2 + 2, 4);
    }
}
