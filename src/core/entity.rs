//! Generational Index — safe entity references.
//!
//! `Entity` consists of `index` (slot address) and `generation` (generation version number).
//! When an entity is destroyed, its slot's generation increments, so any code holding an old
//! handle is rejected on access, preventing dangling references and ABA problems.
use serde::{Deserialize, Serialize};

/// Entity handle — a safe reference to a slot in the World.
///
/// # Lifecycle
///
/// ```text
/// spawn    → Entity { index: 0, generation: 0 }
/// despawn  → the slot's generation becomes 1, old Entity handles are invalidated
/// spawn    → Entity { index: 0, generation: 1 } (slot reused)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Entity {
    /// Slot index in the entity array
    pub index: u32,
    /// Generation version of the current slot, used to detect stale references
    pub generation: u32,
}

impl Entity {
    /// Create a new Entity handle.
    #[must_use]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_equality() {
        let a = Entity::new(0, 0);
        let b = Entity::new(0, 0);
        let c = Entity::new(0, 1);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn entity_copy_is_cheap() {
        let a = Entity::new(5, 3);
        let b = a; // Copy
        assert_eq!(a, b);
    }
}
