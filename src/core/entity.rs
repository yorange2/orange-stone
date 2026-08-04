//! 代际索引（Generational Index）— 安全的实体引用。
//!
//! `Entity` 由 `index`（槽位地址）和 `generation`（代际版本号）组成。
//! 当实体被销毁时，其槽位的 generation 递增，使得任何持有旧 handle 的代码
//! 在访问时被拒绝，避免悬垂引用和 ABA 问题。

/// 实体句柄 — 指向 World 中某个实体槽位的安全引用。
///
/// # 生命周期
///
/// ```text
/// spawn    → Entity { index: 0, generation: 0 }
/// despawn  → 槽位 generation 变为 1，旧 Entity 失效
/// spawn    → Entity { index: 0, generation: 1 }（槽位复用）
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    /// 在实体数组中的槽位索引
    pub index: u32,
    /// 当前槽位的代际版本号，用于检测过期引用
    pub generation: u32,
}

impl Entity {
    /// 创建一个新的 Entity handle。
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
