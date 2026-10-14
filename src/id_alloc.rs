use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU64, Ordering};

/// Simple process-local id allocator for emergent entities (Tables, Delegates, etc.).
/// Starts at 1 by default but can be seeded in tests.
pub static ID_COUNTER: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(1));

/// Return the next unique id (wraps on overflow).
pub fn next_id() -> u64 {
    ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Set the internal counter (for tests or deterministic runs).
#[allow(dead_code)]
pub fn set_counter(v: u64) {
    ID_COUNTER.store(v, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_increment_and_are_unique() {
        // seed deterministically
        set_counter(100);
        let a = next_id();
        let b = next_id();
        assert!(a != b);
        assert_eq!(a + 1, b);
    }
}
