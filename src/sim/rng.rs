//! 可复现的随机数生成器 — xorshift64 实现。
//!
//! 用于保证游戏的确定性：给定相同的 seed 和相同的操作序列，
//! 所有随机调用产生完全一致的结果。
//! 记录每次随机调用的类型和结果，用于回放验证。
//!
//! # 示例
//!
//! ```rust
//! use orange_stone::sim::rng::GameRng;
//!
//! // 相同 seed 产生相同的随机序列
//! let mut rng1 = GameRng::new(42);
//! let mut rng2 = GameRng::new(42);
//! let seq1: Vec<u32> = (0..5).map(|_| rng1.next_u32()).collect();
//! let seq2: Vec<u32> = (0..5).map(|_| rng2.next_u32()).collect();
//! assert_eq!(seq1, seq2);
//! ```

/// 随机调用记录 — 用于回放验证。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RngCall {
    /// next_u32 调用
    NextU32 {
        /// 调用时 RNG 的内部状态
        state_before: u64,
        /// 返回的结果
        result: u32,
    },
    /// next_usize 调用
    NextUsize {
        /// 调用时 RNG 的内部状态
        state_before: u64,
        /// 传入的 max 参数
        max: usize,
        /// 返回的结果
        result: usize,
    },
}

/// 可复现的随机数生成器。
///
/// 基于 xorshift64 算法，状态为一个 `u64`。
/// 记录每次随机调用，用于回放验证。
#[derive(Debug, Clone)]
pub struct GameRng {
    /// 当前内部状态
    state: u64,
    /// 初始 seed
    seed: u64,
    /// 所有随机调用的记录（用于回放）
    pub calls: Vec<RngCall>,
}

impl GameRng {
    /// 使用给定的 seed 创建新的随机数生成器。
    ///
    /// seed 不能为 0（xorshift 要求非零状态）。
    #[must_use]
    pub fn new(seed: u64) -> Self {
        // xorshift 要求非零状态
        let state = if seed == 0 {
            0xDEAD_BEEF_CAFE_BABE
        } else {
            seed
        };
        Self {
            state,
            seed,
            calls: Vec::new(),
        }
    }

    /// 返回初始化时的 seed。
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// 生成下一个 `u32` 随机数。
    ///
    /// 使用 xorshift64 算法。
    pub fn next_u32(&mut self) -> u32 {
        let state_before = self.state;
        // xorshift64
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        let result = (x >> 32) as u32 ^ x as u32;
        self.calls.push(RngCall::NextU32 {
            state_before,
            result,
        });
        result
    }

    /// 生成范围 `[0, max)` 内的随机 `usize`。
    ///
    /// # Panics
    ///
    /// 如果 `max == 0`。
    pub fn next_usize(&mut self, max: usize) -> usize {
        assert!(max > 0, "max must be > 0, got {max}");
        let state_before = self.state;
        // 使用 rejection sampling 避免 bias
        let mask = u32::MAX;
        let limit = mask - (mask % max as u32);
        loop {
            let val = self.next_u32();
            if val < limit {
                let result = (val as usize) % max;
                // 覆盖 next_u32 的自动记录（我们想要更精确的 RngCall）
                self.calls.pop();
                self.calls.push(RngCall::NextUsize {
                    state_before,
                    max,
                    result,
                });
                return result;
            }
        }
    }

    /// 重置 RNG 到初始状态（用于回放）。
    pub fn reset(&mut self) {
        self.state = if self.seed == 0 {
            0xDEAD_BEEF_CAFE_BABE
        } else {
            self.seed
        };
        self.calls.clear();
    }

    /// 返回本次 RNG 会话的调用次数。
    #[must_use]
    pub fn call_count(&self) -> usize {
        self.calls.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_same_sequence() {
        let mut rng1 = GameRng::new(42);
        let mut rng2 = GameRng::new(42);

        let seq1: Vec<u32> = (0..100).map(|_| rng1.next_u32()).collect();
        let seq2: Vec<u32> = (0..100).map(|_| rng2.next_u32()).collect();

        assert_eq!(seq1, seq2);
    }

    #[test]
    fn different_seeds_produce_different_sequences() {
        let mut rng1 = GameRng::new(42);
        let mut rng2 = GameRng::new(99);

        let seq1: Vec<u32> = (0..10).map(|_| rng1.next_u32()).collect();
        let seq2: Vec<u32> = (0..10).map(|_| rng2.next_u32()).collect();

        assert_ne!(seq1, seq2);
    }

    #[test]
    fn zero_seed_is_handled() {
        let rng = GameRng::new(0);
        assert_eq!(rng.seed(), 0);
        // 不应该 panic
    }

    #[test]
    fn calls_are_recorded() {
        let mut rng = GameRng::new(1);
        rng.next_u32();
        rng.next_usize(10);

        assert_eq!(rng.call_count(), 2);
        assert!(matches!(rng.calls[0], RngCall::NextU32 { .. }));
        assert!(matches!(rng.calls[1], RngCall::NextUsize { max: 10, .. }));
    }

    #[test]
    fn reset_restores_state() {
        let mut rng = GameRng::new(77);
        let first = rng.next_u32();
        rng.next_u32();
        rng.next_u32();
        rng.reset();
        assert_eq!(rng.next_u32(), first);
        assert_eq!(rng.call_count(), 1);
    }

    #[test]
    fn next_usize_in_range() {
        let mut rng = GameRng::new(123);
        for _ in 0..1000 {
            let val = rng.next_usize(5);
            assert!(val < 5, "value {val} should be < 5");
        }
    }

    #[test]
    #[should_panic(expected = "max must be > 0")]
    fn next_usize_zero_panics() {
        let mut rng = GameRng::new(1);
        rng.next_usize(0);
    }
}
