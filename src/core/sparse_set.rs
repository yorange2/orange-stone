//! Sparse Set — sparse component storage (segmented arena + copy-on-write).
//!
//! Provides O(1) insert, remove, and lookup, plus cache-friendly dense iteration.
//! Data uses a SoA (Structure of Arrays) layout.
//!
//! # Structural sharing (D1)
//!
//! Dense data is stored in fixed-size **pages**; both the page table and the pages are
//! shared via `Arc`:
//! - `Clone` is an O(1) reference-count bump — cloning `World`/`GameState` no longer
//!   deep-copies entity data (CoW upgraded from "deep-copy the whole Inner" to structural sharing)
//! - The first write to a page copies that page via `Arc::make_mut` (copy-on-write)
//!
//! Iteration order = dense order (insertion order, with swap-remove on deletion), unchanged,
//! so all determinism semantics (aura index rebuild, event order) are preserved.

use crate::core::entity::Entity;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Page capacity — a power of two so slots can be encoded/decoded cheaply.
const PAGE_SHIFT: usize = 6;
/// Page capacity
const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
/// Mask for the in-page offset
const OFFSET_MASK: usize = PAGE_SIZE - 1;

/// Sparse set — a generic component store indexed by Entity.
///
/// Internal layout:
/// - `sparse[i]` = the dense slot of entity i (`page * PAGE_SIZE + offset`), or `ABSENT`
/// - `pages` / `entities`: parallel fixed-size page arrays (value pages + entity pages), shared via `Arc`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseSet<T> {
    /// Maps entity.index → dense slot; `u32::MAX` means absent
    sparse: Vec<u32>,
    /// Dense value storage — fixed-size pages, copy-on-write
    pages: Arc<Vec<Arc<Vec<T>>>>,
    /// Dense entity storage (parallel to pages)
    entities: Arc<Vec<Arc<Vec<Entity>>>>,
    /// Total number of elements
    len: usize,
}

/// Sentinel value marking "absent"
const ABSENT: u32 = u32::MAX;

impl<T: Copy> SparseSet<T> {
    /// Create an empty sparse set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            pages: Arc::new(Vec::new()),
            entities: Arc::new(Vec::new()),
            len: 0,
        }
    }

    /// Returns the number of stored components.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Dense slot → (page, offset within page).
    fn slot_parts(slot: usize) -> (usize, usize) {
        (slot >> PAGE_SHIFT, slot & OFFSET_MASK)
    }

    /// Read the value at a dense slot (the caller must guarantee the slot is valid).
    fn slot_value(&self, slot: usize) -> T {
        let (page, offset) = Self::slot_parts(slot);
        self.pages[page][offset]
    }

    /// Read the entity at a dense slot.
    fn slot_entity(&self, slot: usize) -> Entity {
        let (page, offset) = Self::slot_parts(slot);
        self.entities[page][offset]
    }

    /// Get mutable access to a page (copy-on-write: copies the page first when shared).
    fn page_mut<R>(&mut self, page: usize, f: impl FnOnce(&mut Vec<T>) -> R) -> R {
        let mut page_arc = self.pages[page].clone();
        let result = f(Arc::make_mut(&mut page_arc));
        Arc::make_mut(&mut self.pages)[page] = page_arc;
        result
    }

    /// Get mutable access to an entity page (parallel to `page_mut`).
    fn entity_page_mut<R>(&mut self, page: usize, f: impl FnOnce(&mut Vec<Entity>) -> R) -> R {
        let mut page_arc = self.entities[page].clone();
        let result = f(Arc::make_mut(&mut page_arc));
        Arc::make_mut(&mut self.entities)[page] = page_arc;
        result
    }

    /// Write to a dense slot (copy-on-write the target page).
    fn slot_write(&mut self, slot: usize, entity: Entity, value: T) {
        let (page, offset) = Self::slot_parts(slot);
        self.page_mut(page, |values| values[offset] = value);
        self.entity_page_mut(page, |ents| ents[offset] = entity);
    }

    /// Get an entity's component value (read-only reference).
    #[must_use]
    pub fn get_ref(&self, entity: Entity) -> Option<&T> {
        let slot = self.position_of(entity)?;
        let (page, offset) = Self::slot_parts(slot);
        Some(&self.pages[page][offset])
    }

    /// Get an entity's component value (Copy types are returned by value).
    #[must_use]
    pub fn get(&self, entity: Entity) -> Option<T> {
        self.get_ref(entity).copied()
    }

    /// Insert or update an entity's component value.
    ///
    /// If the entity already has this component, its value is updated; otherwise it is added.
    /// Updating an existing slot only copies its page (O(PAGE_SIZE));
    /// appending copies the page table (a shallow copy, O(number of pages)).
    pub fn insert(&mut self, entity: Entity, value: T) {
        // Make sure the sparse array is large enough
        let idx = entity.index as usize;
        if idx >= self.sparse.len() {
            self.sparse.resize(idx + 1, ABSENT);
        }

        let slot = self.sparse[idx];
        if slot != ABSENT {
            // Update the existing slot
            self.slot_write(slot as usize, entity, value);
            return;
        }

        // Append to the last page (create a new one if it is full)
        let slot = self.len;
        let (page, _) = Self::slot_parts(slot);
        let pages = Arc::make_mut(&mut self.pages);
        let entities = Arc::make_mut(&mut self.entities);
        if page == pages.len() {
            pages.push(Arc::new(Vec::with_capacity(PAGE_SIZE)));
            entities.push(Arc::new(Vec::with_capacity(PAGE_SIZE)));
        }
        {
            let mut page_arc = pages[page].clone();
            Arc::make_mut(&mut page_arc).push(value);
            pages[page] = page_arc;
            let mut ent_arc = entities[page].clone();
            Arc::make_mut(&mut ent_arc).push(entity);
            entities[page] = ent_arc;
        }
        self.sparse[idx] = slot as u32;
        self.len += 1;
    }

    /// Remove an entity's component, returning the removed value (if any).
    ///
    /// Uses the swap-remove strategy: the last element is moved into the deleted position
    /// and its sparse mapping is updated. This is much faster than an order-preserving remove (O(1) vs O(n)).
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let idx = entity.index as usize;
        if idx >= self.sparse.len() {
            return None;
        }

        let slot = self.sparse[idx];
        if slot == ABSENT {
            return None;
        }
        let slot = slot as usize;

        let removed_value = self.slot_value(slot);
        let last_slot = self.len - 1;
        if slot == last_slot {
            // Remove the last element: pop it straight from the final page
            let (page, _) = Self::slot_parts(slot);
            let pages = Arc::make_mut(&mut self.pages);
            let entities = Arc::make_mut(&mut self.entities);
            {
                let mut page_arc = pages[page].clone();
                Arc::make_mut(&mut page_arc).pop();
                pages[page] = page_arc;
                let mut ent_arc = entities[page].clone();
                Arc::make_mut(&mut ent_arc).pop();
                entities[page] = ent_arc;
            }
            // Drop the final page if it becomes empty
            if pages[page].is_empty() {
                pages.pop();
                entities.pop();
            }
        } else {
            // swap-remove: move the last element into the deleted slot
            let moved_entity = self.slot_entity(last_slot);
            let moved_value = self.slot_value(last_slot);
            self.slot_write(slot, moved_entity, moved_value);
            // Pop the moved element from the final page
            let (last_page, _) = Self::slot_parts(last_slot);
            let pages = Arc::make_mut(&mut self.pages);
            let entities = Arc::make_mut(&mut self.entities);
            {
                let mut page_arc = pages[last_page].clone();
                Arc::make_mut(&mut page_arc).pop();
                pages[last_page] = page_arc;
                let mut ent_arc = entities[last_page].clone();
                Arc::make_mut(&mut ent_arc).pop();
                entities[last_page] = ent_arc;
            }
            if pages[last_page].is_empty() {
                pages.pop();
                entities.pop();
            }
            // Update the moved entity's sparse mapping
            self.sparse[moved_entity.index as usize] = slot as u32;
        }

        // Clear the removed entity's sparse entry
        self.sparse[idx] = ABSENT;
        self.len -= 1;

        Some(removed_value)
    }

    /// Iterate over all (Entity, &T) pairs (in dense order).
    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.pages
            .iter()
            .zip(self.entities.iter())
            .flat_map(|(values, ents)| values.iter().zip(ents.iter()).map(|(v, e)| (*e, v)))
    }

    /// Check whether the entity has this component.
    #[must_use]
    pub fn contains(&self, entity: Entity) -> bool {
        self.position_of(entity).is_some()
    }

    /// Returns the entity's slot in dense storage, or None if absent.
    fn position_of(&self, entity: Entity) -> Option<usize> {
        let idx = entity.index as usize;
        if idx >= self.sparse.len() {
            return None;
        }
        let slot = self.sparse[idx];
        if slot == ABSENT {
            return None;
        }
        // Verify the generation matches
        let stored = self.slot_entity(slot as usize);
        if stored.generation != entity.generation {
            return None;
        }
        Some(slot as usize)
    }
}

impl<T: Copy> Default for SparseSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(idx: u32, generation: u32) -> Entity {
        Entity::new(idx, generation)
    }

    #[test]
    fn insert_and_get() {
        let mut set = SparseSet::<i32>::new();
        set.insert(e(0, 0), 42);
        assert_eq!(set.get(e(0, 0)), Some(42));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn insert_update() {
        let mut set = SparseSet::<i32>::new();
        set.insert(e(0, 0), 1);
        set.insert(e(0, 0), 2); // update
        assert_eq!(set.get(e(0, 0)), Some(2));
        assert_eq!(set.len(), 1); // still 1
    }

    #[test]
    fn get_missing() {
        let set = SparseSet::<i32>::new();
        assert_eq!(set.get(e(0, 0)), None);
    }

    #[test]
    fn generation_mismatch_is_missing() {
        let mut set = SparseSet::<i32>::new();
        set.insert(e(0, 0), 42);
        // An old generation must not be found (the generation changed)
        assert_eq!(set.get(e(0, 1)), None);
        // But the original handle still works
        assert_eq!(set.get(e(0, 0)), Some(42));
    }

    #[test]
    fn remove_existing() {
        let mut set = SparseSet::<i32>::new();
        set.insert(e(0, 0), 10);
        let removed = set.remove(e(0, 0));
        assert_eq!(removed, Some(10));
        assert_eq!(set.get(e(0, 0)), None);
        assert!(set.is_empty());
    }

    #[test]
    fn remove_missing() {
        let mut set = SparseSet::<i32>::new();
        assert_eq!(set.remove(e(0, 0)), None);
    }

    #[test]
    fn swap_remove_correctness() {
        // Insert several entities, remove one in the middle, and verify iteration is still correct
        let mut set = SparseSet::<i32>::new();
        let entities = [e(0, 0), e(1, 0), e(2, 0), e(3, 0)];
        for (i, &ent) in entities.iter().enumerate() {
            set.insert(ent, i as i32 * 10);
        }
        // Remove e(1, 0) — value 10
        set.remove(e(1, 0));

        // The remaining ones should all be accessible
        assert_eq!(set.get(e(0, 0)), Some(0));
        assert_eq!(set.get(e(2, 0)), Some(20));
        assert_eq!(set.get(e(3, 0)), Some(30));
        assert_eq!(set.get(e(1, 0)), None);
        assert_eq!(set.len(), 3);

        // Iteration should contain three elements
        let mut found: Vec<(u32, i32)> = set.iter().map(|(e, &v)| (e.index, v)).collect();
        found.sort_by_key(|(idx, _)| *idx);
        assert_eq!(found, vec![(0, 0), (2, 20), (3, 30)]);
    }

    #[test]
    fn iter_visits_all() {
        let mut set = SparseSet::<i32>::new();
        for i in 0..5u32 {
            set.insert(e(i, 0), (i * 5) as i32);
        }
        let mut values: Vec<i32> = set.iter().map(|(_, &v)| v).collect();
        values.sort();
        assert_eq!(values, vec![0, 5, 10, 15, 20]);
    }

    #[test]
    fn contains_check() {
        let mut set = SparseSet::<i32>::new();
        set.insert(e(1, 0), 100);
        assert!(set.contains(e(1, 0)));
        assert!(!set.contains(e(1, 1))); // different generation
        assert!(!set.contains(e(2, 0))); // absent
    }

    #[test]
    fn many_random_removes() {
        let mut set = SparseSet::<i32>::new();
        let n = 100u32;
        for i in 0..n {
            set.insert(e(i, 0), i as i32);
        }
        // Remove even indices
        for i in (0..n).step_by(2) {
            set.remove(e(i, 0));
        }
        // Verify only odd indices remain
        for i in 0..n {
            if i % 2 == 0 {
                assert!(!set.contains(e(i, 0)), "even index {i} should be removed");
            } else {
                assert!(set.contains(e(i, 0)), "odd index {i} should still exist");
                assert_eq!(set.get(e(i, 0)), Some(i as i32));
            }
        }
        assert_eq!(set.len(), (n / 2) as usize);
    }

    #[test]
    fn clone_is_structurally_shared() {
        // Writes after cloning do not affect each other (copy-on-write), and cloning itself copies no data
        let mut set = SparseSet::<i32>::new();
        for i in 0..(PAGE_SIZE * 3) as u32 {
            set.insert(e(i, 0), i as i32);
        }
        let mut clone = set.clone();
        // Shared page table (Arc reference count)
        assert_eq!(Arc::strong_count(&set.pages), 2, "pages shared after clone");

        // Modify the clone: update an existing slot + append
        clone.insert(e(0, 0), -1);
        clone.insert(e(999, 0), 999);
        assert_eq!(set.get(e(0, 0)), Some(0), "original unaffected");
        assert_eq!(set.get(e(999, 0)), None, "original unaffected");
        assert_eq!(clone.get(e(0, 0)), Some(-1));
        assert_eq!(clone.get(e(999, 0)), Some(999));

        // The original set remains writable
        set.insert(e(2000, 0), 2000);
        assert_eq!(clone.get(e(2000, 0)), None);
        assert_eq!(set.get(e(2000, 0)), Some(2000));
    }

    #[test]
    fn clone_then_remove_independent() {
        let mut set = SparseSet::<i32>::new();
        for i in 0..(PAGE_SIZE + 10) as u32 {
            set.insert(e(i, 0), i as i32);
        }
        let mut clone = set.clone();
        clone.remove(e(0, 0));
        assert_eq!(set.get(e(0, 0)), Some(0), "original keeps removed element");
        assert_eq!(clone.get(e(0, 0)), None);
        // The original set's iteration is unaffected
        let count = set.iter().count();
        assert_eq!(count, PAGE_SIZE + 10);
    }

    #[test]
    fn cross_page_swap_remove() {
        // Remove a slot that crosses a page boundary and verify swap-remove updates the sparse mapping correctly
        let mut set = SparseSet::<i32>::new();
        for i in 0..(PAGE_SIZE + 5) as u32 {
            set.insert(e(i, 0), i as i32);
        }
        // Remove a slot in the middle of the first page
        set.remove(e(3, 0));
        assert_eq!(set.len(), PAGE_SIZE + 4);
        // The moved last element (originally on the final page) must be accessible
        let moved = e(PAGE_SIZE as u32 + 4, 0);
        assert!(set.contains(moved), "moved element must keep its identity");
        assert_eq!(set.get(moved), Some((PAGE_SIZE + 4) as i32));
        // The removed slot must be inaccessible
        assert!(!set.contains(e(3, 0)));
        // Iteration count is correct
        assert_eq!(set.iter().count(), PAGE_SIZE + 4);
    }
}
