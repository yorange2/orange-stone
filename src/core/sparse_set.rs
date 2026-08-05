//! Sparse Set — 稀疏集合组件存储（分段竞技场 + 写时复制）。
//!
//! 提供 O(1) 插入、删除、查找，以及缓存友好的密集遍历。
//! 数据为 SoA（Structure of Arrays）布局。
//!
//! # 结构共享（D1）
//!
//! 稠密数据按固定大小的**页**存储，页表与页都通过 `Arc` 共享：
//! - `Clone` 是 O(1) 的引用计数递增 — `World`/`GameState` 克隆不再
//!   深拷贝实体数据（CoW 从"整个 Inner 深拷贝"升级为结构性共享）
//! - 首次写入某页时通过 `Arc::make_mut` 复制该页（写时复制）
//!
//! 遍历顺序 = 稠密顺序（插入顺序，删除为 swap-remove），与之前一致，
//! 因此所有确定性语义（光环索引重建、事件顺序）保持不变。

use crate::core::entity::Entity;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 每页容量 — 2 的幂便于槽位编解码。
const PAGE_SHIFT: usize = 6;
/// 每页容量
const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
/// 页内偏移掩码
const OFFSET_MASK: usize = PAGE_SIZE - 1;

/// 稀疏集合 — 按 Entity 索引的泛型组件存储。
///
/// 内部结构：
/// - `sparse[i]` = 实体 i 的稠密槽位（`page * PAGE_SIZE + offset`），或 `ABSENT`
/// - `pages` / `entities`：并行的定长页数组（值页 + 实体页），`Arc` 共享
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseSet<T> {
    /// 映射 entity.index → 稠密槽位，`u32::MAX` 表示不存在
    sparse: Vec<u32>,
    /// 稠密值存储 — 定长页，写时复制
    pages: Arc<Vec<Arc<Vec<T>>>>,
    /// 稠密实体存储（与 pages 并行）
    entities: Arc<Vec<Arc<Vec<Entity>>>>,
    /// 元素总数
    len: usize,
}

/// 标记"不存在"的哨兵值
const ABSENT: u32 = u32::MAX;

impl<T: Copy> SparseSet<T> {
    /// 创建一个空的稀疏集合。
    #[must_use]
    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            pages: Arc::new(Vec::new()),
            entities: Arc::new(Vec::new()),
            len: 0,
        }
    }

    /// 返回当前存储的组件数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// 如果集合为空则返回 `true`。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 稠密槽位 → (页, 页内偏移)。
    fn slot_parts(slot: usize) -> (usize, usize) {
        (slot >> PAGE_SHIFT, slot & OFFSET_MASK)
    }

    /// 读取稠密槽位的值（调用方需保证槽位有效）。
    fn slot_value(&self, slot: usize) -> T {
        let (page, offset) = Self::slot_parts(slot);
        self.pages[page][offset]
    }

    /// 读取稠密槽位的实体。
    fn slot_entity(&self, slot: usize) -> Entity {
        let (page, offset) = Self::slot_parts(slot);
        self.entities[page][offset]
    }

    /// 获取某个页的可变访问（写时复制：页共享时先复制该页）。
    fn page_mut<R>(&mut self, page: usize, f: impl FnOnce(&mut Vec<T>) -> R) -> R {
        let mut page_arc = self.pages[page].clone();
        let result = f(Arc::make_mut(&mut page_arc));
        Arc::make_mut(&mut self.pages)[page] = page_arc;
        result
    }

    /// 获取某个实体页的可变访问（与 `page_mut` 并行）。
    fn entity_page_mut<R>(&mut self, page: usize, f: impl FnOnce(&mut Vec<Entity>) -> R) -> R {
        let mut page_arc = self.entities[page].clone();
        let result = f(Arc::make_mut(&mut page_arc));
        Arc::make_mut(&mut self.entities)[page] = page_arc;
        result
    }

    /// 写入稠密槽位（写时复制目标页）。
    fn slot_write(&mut self, slot: usize, entity: Entity, value: T) {
        let (page, offset) = Self::slot_parts(slot);
        self.page_mut(page, |values| values[offset] = value);
        self.entity_page_mut(page, |ents| ents[offset] = entity);
    }

    /// 获取实体的组件值（只读引用）。
    #[must_use]
    pub fn get_ref(&self, entity: Entity) -> Option<&T> {
        let slot = self.position_of(entity)?;
        let (page, offset) = Self::slot_parts(slot);
        Some(&self.pages[page][offset])
    }

    /// 获取实体的组件值（Copy 类型按值返回）。
    #[must_use]
    pub fn get(&self, entity: Entity) -> Option<T> {
        self.get_ref(entity).copied()
    }

    /// 插入或更新实体的组件值。
    ///
    /// 如果该实体已有此组件，则更新值；否则新增。
    /// 更新已有槽位时只复制所在页（O(PAGE_SIZE)）；
    /// 追加时复制页表（浅拷贝，O(页数)）。
    pub fn insert(&mut self, entity: Entity, value: T) {
        // 确保 sparse 数组足够大
        let idx = entity.index as usize;
        if idx >= self.sparse.len() {
            self.sparse.resize(idx + 1, ABSENT);
        }

        let slot = self.sparse[idx];
        if slot != ABSENT {
            // 更新已有槽位
            self.slot_write(slot as usize, entity, value);
            return;
        }

        // 追加到最后一个页（若已满则新建一页）
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

    /// 移除实体的组件，返回被移除的值（如果存在）。
    ///
    /// 使用 swap-remove 策略：将最后一个元素移到被删除的位置，
    /// 并更新该元素的 sparse 映射。这比保持顺序的 remove 快得多（O(1) vs O(n)）。
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
            // 移除最后一个元素：直接从末页弹出
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
            // 末页变空则丢弃
            if pages[page].is_empty() {
                pages.pop();
                entities.pop();
            }
        } else {
            // swap-remove：末元素移到被删槽位
            let moved_entity = self.slot_entity(last_slot);
            let moved_value = self.slot_value(last_slot);
            self.slot_write(slot, moved_entity, moved_value);
            // 从末页弹出被移走的元素
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
            // 更新被移动实体的 sparse 映射
            self.sparse[moved_entity.index as usize] = slot as u32;
        }

        // 清理被删实体的 sparse 条目
        self.sparse[idx] = ABSENT;
        self.len -= 1;

        Some(removed_value)
    }

    /// 遍历所有 (Entity, &T) 对（稠密顺序）。
    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.pages
            .iter()
            .zip(self.entities.iter())
            .flat_map(|(values, ents)| values.iter().zip(ents.iter()).map(|(v, e)| (*e, v)))
    }

    /// 检查实体是否存在此组件。
    #[must_use]
    pub fn contains(&self, entity: Entity) -> bool {
        self.position_of(entity).is_some()
    }

    /// 返回实体在稠密存储中的槽位，不存在返回 None。
    fn position_of(&self, entity: Entity) -> Option<usize> {
        let idx = entity.index as usize;
        if idx >= self.sparse.len() {
            return None;
        }
        let slot = self.sparse[idx];
        if slot == ABSENT {
            return None;
        }
        // 验证 generation 匹配
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

    #[test]
    fn clone_is_structurally_shared() {
        // 克隆后写入互不影响（写时复制），且克隆本身不复制数据
        let mut set = SparseSet::<i32>::new();
        for i in 0..(PAGE_SIZE * 3) as u32 {
            set.insert(e(i, 0), i as i32);
        }
        let mut clone = set.clone();
        // 共享页表（Arc 引用计数）
        assert_eq!(Arc::strong_count(&set.pages), 2, "pages shared after clone");

        // 修改克隆：更新已有槽位 + 追加
        clone.insert(e(0, 0), -1);
        clone.insert(e(999, 0), 999);
        assert_eq!(set.get(e(0, 0)), Some(0), "original unaffected");
        assert_eq!(set.get(e(999, 0)), None, "original unaffected");
        assert_eq!(clone.get(e(0, 0)), Some(-1));
        assert_eq!(clone.get(e(999, 0)), Some(999));

        // 原集合继续可写
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
        // 原集合遍历不受影响
        let count = set.iter().count();
        assert_eq!(count, PAGE_SIZE + 10);
    }

    #[test]
    fn cross_page_swap_remove() {
        // 删除跨越页边界的槽位，验证 swap-remove 正确更新 sparse 映射
        let mut set = SparseSet::<i32>::new();
        for i in 0..(PAGE_SIZE + 5) as u32 {
            set.insert(e(i, 0), i as i32);
        }
        // 删除第一页中间的槽位
        set.remove(e(3, 0));
        assert_eq!(set.len(), PAGE_SIZE + 4);
        // 被移动的末元素（原来在最后一页）应可访问
        let moved = e(PAGE_SIZE as u32 + 4, 0);
        assert!(set.contains(moved), "moved element must keep its identity");
        assert_eq!(set.get(moved), Some((PAGE_SIZE + 4) as i32));
        // 删除的槽位不可访问
        assert!(!set.contains(e(3, 0)));
        // 遍历数量正确
        assert_eq!(set.iter().count(), PAGE_SIZE + 4);
    }
}
