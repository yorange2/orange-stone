//! Sparse Set — 稀疏集合组件存储。
//!
//! 提供 O(1) 插入、删除、查找，以及缓存友好的密集遍历。
//! 数据结构为 SoA（Structure of Arrays）布局，SIMD 友好。
//!
//! # 实现细节
//!
//! - `sparse`: entity.index → dense 数组中的位置（用 `u32::MAX` 表示不存在）
//! - `dense_entities` + `dense_values`: 并行数组，存储所有已插入的 (Entity, T) 对
//! - 删除使用 swap-remove，O(1) 但会改变密集遍历顺序

/// 稀疏集合 — 按 Entity 索引的泛型组件存储。
///
/// 内部使用三个数组：
/// - `sparse[i]` = 实体 i 在 dense 中的位置，或 `ABSENT`（未插入）
/// - `dense_entities[pos]` = 该位置的 Entity
/// - `dense_values[pos]` = 该位置的组件值
#[derive(Debug, Clone)]
pub struct SparseSet<T> {
    /// 映射 entity.index → dense 中的位置，u32::MAX 表示不存在
    sparse: Vec<u32>,
    /// 密集存储的 entity 数组
    dense_entities: Vec<crate::core::entity::Entity>,
    /// 密集存储的组件值数组（与 dense_entities 并行）
    dense_values: Vec<T>,
}

/// 标记"不存在"的哨兵值
const ABSENT: u32 = u32::MAX;

impl<T: Copy> SparseSet<T> {
    /// 创建一个空的稀疏集合。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense_entities: Vec::new(),
            dense_values: Vec::new(),
        }
    }

    /// 返回当前存储的组件数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.dense_values.len()
    }

    /// 如果集合为空则返回 `true`。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dense_values.is_empty()
    }

    /// 获取实体的组件值（只读引用）。
    #[must_use]
    pub fn get_ref(&self, entity: crate::core::entity::Entity) -> Option<&T> {
        let pos = self.position_of(entity)?;
        Some(&self.dense_values[pos])
    }

    /// 获取实体的组件值（Copy 类型按值返回）。
    #[must_use]
    pub fn get(&self, entity: crate::core::entity::Entity) -> Option<T> {
        self.get_ref(entity).copied()
    }

    /// 插入或更新实体的组件值。
    ///
    /// 如果该实体已有此组件，则更新值；否则新增。
    pub fn insert(&mut self, entity: crate::core::entity::Entity, value: T) {
        // 确保 sparse 数组足够大
        let idx = entity.index as usize;
        if idx >= self.sparse.len() {
            self.sparse.resize(idx + 1, ABSENT);
        }

        let pos = self.sparse[idx];
        if pos == ABSENT {
            // 新增
            let new_pos = self.dense_values.len() as u32;
            self.sparse[idx] = new_pos;
            self.dense_entities.push(entity);
            self.dense_values.push(value);
        } else {
            // 更新
            self.dense_values[pos as usize] = value;
            // generation 可能已变，更新 entity 记录
            self.dense_entities[pos as usize] = entity;
        }
    }

    /// 移除实体的组件，返回被移除的值（如果存在）。
    ///
    /// 使用 swap-remove 策略：将最后一个元素移到被删除的位置，
    /// 并更新该元素的 sparse 映射。这比保持顺序的 remove 快得多（O(1) vs O(n)）。
    pub fn remove(&mut self, entity: crate::core::entity::Entity) -> Option<T> {
        let idx = entity.index as usize;
        if idx >= self.sparse.len() {
            return None;
        }

        let pos = self.sparse[idx];
        if pos == ABSENT {
            return None;
        }

        // swap-remove: 把最后一个元素移到当前位置
        let last_idx = self.dense_values.len() - 1;
        let removed_value = self.dense_values.swap_remove(pos as usize);
        let removed_entity = self.dense_entities.swap_remove(pos as usize);
        debug_assert_eq!(removed_entity, entity, "sparse/slot mismatch");

        // 清理被删实体的 sparse 条目
        self.sparse[idx] = ABSENT;

        // 如果被移除的不是最后一个元素，需要更新被交换实体的 sparse 映射
        if (pos as usize) < last_idx {
            let swapped_entity = self.dense_entities[pos as usize];
            self.sparse[swapped_entity.index as usize] = pos;
        }

        Some(removed_value)
    }

    /// 遍历所有 (Entity, &T) 对。
    pub fn iter(&self) -> impl Iterator<Item = (crate::core::entity::Entity, &T)> {
        self.dense_entities
            .iter()
            .zip(self.dense_values.iter())
            .map(|(e, v)| (*e, v))
    }

    /// 检查实体是否存在此组件。
    #[must_use]
    pub fn contains(&self, entity: crate::core::entity::Entity) -> bool {
        self.position_of(entity).is_some()
    }

    /// 返回实体在 dense 中的位置，不存在返回 None。
    fn position_of(&self, entity: crate::core::entity::Entity) -> Option<usize> {
        let idx = entity.index as usize;
        if idx >= self.sparse.len() {
            return None;
        }
        let pos = self.sparse[idx];
        if pos == ABSENT {
            return None;
        }
        // 验证 generation 匹配
        let stored = self.dense_entities[pos as usize];
        if stored.generation != entity.generation {
            return None;
        }
        Some(pos as usize)
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
    use crate::core::entity::Entity;

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
        set.insert(e(0, 0), 2); // 更新
        assert_eq!(set.get(e(0, 0)), Some(2));
        assert_eq!(set.len(), 1); // 还是 1 个
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
        // 旧 generation 应该查不到(因为 generation 变了)
        assert_eq!(set.get(e(0, 1)), None);
        // 但原始 handle 仍然可以访问
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
        // 插入多个实体，删除中间的，验证遍历仍然正确
        let mut set = SparseSet::<i32>::new();
        let entities = [e(0, 0), e(1, 0), e(2, 0), e(3, 0)];
        for (i, &ent) in entities.iter().enumerate() {
            set.insert(ent, i as i32 * 10);
        }
        // 删除 e(1, 0) — 值 10
        set.remove(e(1, 0));

        // 剩余的应该都能正确访问
        assert_eq!(set.get(e(0, 0)), Some(0));
        assert_eq!(set.get(e(2, 0)), Some(20));
        assert_eq!(set.get(e(3, 0)), Some(30));
        assert_eq!(set.get(e(1, 0)), None);
        assert_eq!(set.len(), 3);

        // 遍历应该包含三个元素
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
        assert!(!set.contains(e(1, 1))); // 不同 generation
        assert!(!set.contains(e(2, 0))); // 不存在
    }

    #[test]
    fn many_random_removes() {
        let mut set = SparseSet::<i32>::new();
        let n = 100u32;
        for i in 0..n {
            set.insert(e(i, 0), i as i32);
        }
        // 删除偶数索引
        for i in (0..n).step_by(2) {
            set.remove(e(i, 0));
        }
        // 验证剩余的是奇数索引
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
}
