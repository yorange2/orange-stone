//! SmallList — a stack-buffer list that avoids heap allocation for small collections.
//!
//! Event handling frequently needs a short-lived snapshot of entities or
//! `(entity, effect)` pairs: scan the `World` under an immutable borrow, release
//! the borrow, then mutate the `GameState`. Collecting such snapshots into a
//! `Vec` allocates on the heap for every event; `SmallList` keeps the first
//! `INLINE` elements in a stack array and only spills to the heap when the
//! collection grows beyond that.
//!
//! In practice the snapshot lists are bounded by the board size (7 minions,
//! plus the hero and weapon), so `SmallList` almost never allocates on the
//! hot path. `INLINE` defaults to 16 (two full boards); trigger lists
//! explicitly use a smaller inline capacity.

//! The unsafe code here is confined to this module: moving `T` values in and
//! out of the `MaybeUninit` inline slots under the `len` invariant. All safety
//! conditions are documented at each site.
#![allow(unsafe_code)]

use std::mem::MaybeUninit;

/// A list with inline storage — no heap allocation until the length exceeds `INLINE`.
#[derive(Debug)]
pub struct SmallList<T, const INLINE: usize = 16> {
    /// The first `min(len, INLINE)` elements live here.
    inline: [MaybeUninit<T>; INLINE],
    /// Elements beyond `INLINE` spill here (empty in the common case).
    spilled: Vec<T>,
    /// Total number of elements.
    len: usize,
}

impl<T, const INLINE: usize> SmallList<T, INLINE> {
    /// Create an empty list.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inline: [const { MaybeUninit::uninit() }; INLINE],
            spilled: Vec::new(),
            len: 0,
        }
    }

    /// Number of stored elements.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the list is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Whether the list contains a value equal to `value`.
    #[must_use]
    pub fn contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.iter().any(|item| item == value)
    }

    /// Append an element.
    pub fn push(&mut self, value: T) {
        if self.len < INLINE {
            // Safety: `len < INLINE`, so the slot is uninitialized and writable.
            self.inline[self.len].write(value);
        } else {
            self.spilled.push(value);
        }
        self.len += 1;
    }

    /// Clear the list, dropping all elements.
    pub fn clear(&mut self) {
        for slot in &mut self.inline[..self.len.min(INLINE)] {
            // Safety: slots up to `len` hold initialized values.
            unsafe { slot.assume_init_drop() };
        }
        self.spilled.clear();
        self.len = 0;
    }

    /// Read the element at `index`.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        if index < INLINE {
            // Safety: `index < min(len, INLINE)`, so the slot is initialized.
            Some(unsafe { self.inline[index].assume_init_ref() })
        } else {
            self.spilled.get(index - INLINE)
        }
    }

    /// Iterate over the elements.
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.into_iter()
    }

    /// Remove and return the element at `index`, shifting the tail left.
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "SmallList::remove: index out of bounds");
        if index < INLINE {
            // Safety: the slot holds an initialized value; read it out.
            let value = unsafe { self.inline[index].as_ptr().read() };
            let inline_end = self.len.min(INLINE);
            for i in index..inline_end - 1 {
                // Safety: slots `i + 1` and `i` are initialized; move the value left.
                unsafe {
                    self.inline[i] = MaybeUninit::new(self.inline[i + 1].as_ptr().read());
                }
            }
            if self.len > INLINE {
                // Pull the first spilled element into the now-free last inline slot.
                self.inline[INLINE - 1] = MaybeUninit::new(self.spilled.remove(0));
            }
            self.len -= 1;
            value
        } else {
            let value = self.spilled.remove(index - INLINE);
            self.len -= 1;
            value
        }
    }

    /// Extend from an iterator.
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }

    /// Swap the elements at `a` and `b` (both must be in bounds).
    fn swap_elems(&mut self, a: usize, b: usize) {
        debug_assert!(a < self.len && b < self.len);
        if a < INLINE && b < INLINE {
            self.inline.swap(a, b);
        } else if a < INLINE {
            // a inline, b spilled
            let temp = unsafe { self.inline[a].as_ptr().read() };
            self.inline[a] =
                MaybeUninit::new(std::mem::replace(&mut self.spilled[b - INLINE], temp));
        } else if b < INLINE {
            let temp = unsafe { self.inline[b].as_ptr().read() };
            self.inline[b] =
                MaybeUninit::new(std::mem::replace(&mut self.spilled[a - INLINE], temp));
        } else {
            self.spilled.swap(a - INLINE, b - INLINE);
        }
    }

    /// Sort by the key function (stable, insertion sort — lists are small).
    pub fn sort_by_key<K: Ord>(&mut self, mut f: impl FnMut(&T) -> K) {
        for i in 1..self.len {
            let mut j = i;
            while j > 0 {
                let key_j = f(self.get(j).expect("in bounds"));
                let key_prev = f(self.get(j - 1).expect("in bounds"));
                if key_j < key_prev {
                    self.swap_elems(j, j - 1);
                    j -= 1;
                } else {
                    break;
                }
            }
        }
    }
}

impl<T, const INLINE: usize> Default for SmallList<T, INLINE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const INLINE: usize> FromIterator<T> for SmallList<T, INLINE> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

/// Indexing — `list[i]` returns a reference to the `i`-th element.
impl<T, const INLINE: usize> std::ops::Index<usize> for SmallList<T, INLINE> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.get(index).expect("SmallList: index out of bounds")
    }
}

impl<T, const INLINE: usize> IntoIterator for SmallList<T, INLINE> {
    type Item = T;
    type IntoIter = SmallListIntoIter<T, INLINE>;

    fn into_iter(self) -> Self::IntoIter {
        SmallListIntoIter {
            list: self,
            next: 0,
        }
    }
}

/// Shared iteration (`for x in &list`) — mirrors `Vec`'s API.
impl<'a, T, const INLINE: usize> IntoIterator for &'a SmallList<T, INLINE> {
    type Item = &'a T;
    type IntoIter = std::iter::Chain<std::slice::Iter<'a, T>, std::slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        let inline_len = self.len.min(INLINE);
        // Safety: slots `0..inline_len` are initialized and contiguous.
        let inline = unsafe { std::slice::from_raw_parts(self.inline.as_ptr().cast(), inline_len) };
        inline.iter().chain(self.spilled.iter())
    }
}

/// Consuming iterator for `SmallList`.
pub struct SmallListIntoIter<T, const INLINE: usize> {
    list: SmallList<T, INLINE>,
    next: usize,
}

impl<T, const INLINE: usize> Iterator for SmallListIntoIter<T, INLINE> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.next >= self.list.len {
            return None;
        }
        let value = self.list.remove(self.next);
        // `remove` shifts the tail left, so the next element is at the same index.
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.list.len - self.next;
        (remaining, Some(remaining))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_iterate_inline() {
        let mut list: SmallList<u32, 4> = SmallList::new();
        for i in 0..4 {
            list.push(i);
        }
        assert_eq!(list.len(), 4);
        assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![0, 1, 2, 3]);
        assert_eq!(list[2], 2);
    }

    #[test]
    fn spills_to_heap_when_exceeding_inline() {
        let mut list: SmallList<u32, 4> = SmallList::new();
        for i in 0..10 {
            list.push(i);
        }
        assert_eq!(list.len(), 10);
        assert_eq!(
            list.iter().copied().collect::<Vec<_>>(),
            (0..10).collect::<Vec<_>>()
        );
        assert_eq!(list[9], 9);
    }

    #[test]
    fn remove_shifts_inline() {
        let mut list: SmallList<u32, 4> = SmallList::new();
        for i in 0..4 {
            list.push(i);
        }
        assert_eq!(list.remove(1), 1);
        assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![0, 2, 3]);
    }

    #[test]
    fn remove_from_spill_pulls_inline() {
        let mut list: SmallList<u32, 2> = SmallList::new();
        for i in 0..5 {
            list.push(i);
        }
        // Remove from the inline region while spilled elements exist
        assert_eq!(list.remove(0), 0);
        assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![1, 2, 3, 4]);
        // Remove from the spilled region
        assert_eq!(list.remove(3), 4);
        assert_eq!(list.iter().copied().collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn consuming_iteration() {
        let mut list: SmallList<u32, 4> = SmallList::new();
        for i in 0..3 {
            list.push(i);
        }
        let mut total = 0;
        for v in list {
            total += v;
        }
        assert_eq!(total, 3);
    }

    #[test]
    fn from_iterator() {
        let list: SmallList<u32, 4> = (0..6).collect();
        assert_eq!(
            list.iter().copied().collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 5]
        );
    }

    #[test]
    fn sort_by_key() {
        let mut list: SmallList<u32, 4> = SmallList::new();
        for i in [5, 3, 9, 1, 7, 2] {
            list.push(i);
        }
        list.sort_by_key(|&v| v);
        assert_eq!(
            list.iter().copied().collect::<Vec<_>>(),
            vec![1, 2, 3, 5, 7, 9]
        );
    }

    #[test]
    fn clear_drops_elements() {
        let mut list: SmallList<String, 2> = SmallList::new();
        list.push(String::from("a"));
        list.push(String::from("b"));
        list.push(String::from("c")); // spills
        list.clear();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        list.push(String::from("d"));
        assert_eq!(
            list.iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["d"]
        );
    }

    #[test]
    fn empty_list() {
        let list: SmallList<u32, 4> = SmallList::new();
        assert!(list.is_empty());
        assert_eq!(list.get(0), None);
        assert_eq!(list.iter().count(), 0);
    }
}
