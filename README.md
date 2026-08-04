# 🟠 Orange Stone

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

用 Rust 编写的高性能炉石传说（Hearthstone）模拟器，专为强化学习训练而设计。

## 为什么选择 Orange Stone？

现有的炉石模拟器（[RosettaStone](https://github.com/utilForever/RosettaStone)、[SabberStone](https://github.com/HearthSim/SabberStone)）主要面向对局重现和 AI 对战。Orange Stone 的目标不同——它是为**大规模并行模拟**而生的，核心场景是：

- 在 RL 训练中，每步需要模拟数万个对局来收集经验
- 要求确定性的状态回放，便于经验重放和调试
- 需要与 Python ML 生态（PyTorch、JAX、RLlib）无缝对接
- 批量将游戏状态编码为张量，供神经网络消费

Rust 的所有权系统天然适合这一场景：零成本抽象的批量模拟、无 GC 停顿的确定性延迟、编译期保证的并发安全。

## 设计亮点

### ECS 架构代替继承树

传统模拟器使用 `Minion → Weapon → Spell` 的继承树来表示卡牌。Orange Stone 使用 Entity Component System，将卡牌效果拆解为可组合的 Component：

```rust
// 一张"战吼：抽一张牌"的随从
entity
    .with(Health(5))
    .with(Attack(3))
    .with(Cost(4))
    .with(Battlecry(DrawCard(1)))
    .with(Zone(ZoneType::Hand));
```

这让卡牌效果可以灵活组合，数据布局对缓存友好，适合批量操作。

### 不可变状态 + Copy-on-Write

每次状态变更产生新的 `GameState`，通过 CoW 共享不变的部分：

```
state_0 ──→ state_1 ──→ state_2
  │                       │
  └───→ branch_1 ──→ branch_2   (MCTS 分支，几乎零额外开销)
```

这对 RL 至关重要：MCTS 搜索、经验回放、分布式训练都依赖高效的状态复制和回退。

### 确定性 + 高性能

| 指标 | 目标 |
|------|------|
| 单核模拟速度 | > 50,000 steps/s |
| 多核并行 | 线性扩展至物理核心数 |
| 内存占用 | < 1 MB / game instance |
| Python 绑定开销 | < 1 μs / call |

## 快速开始

> 项目处于早期开发阶段，以下内容会持续更新。

### 前置要求

- Rust 1.80+
- Python 3.10+（用于 RL 接口）

### 构建

```bash
git clone https://github.com/yourname/orange-stone.git
cd orange-stone
cargo build --release
```

### 运行测试

```bash
cargo test
```

### Python 绑定（计划中）

```python
import orange_stone as os

# 创建批量环境
env = os.BatchedGameEnv(num_envs=1024)

# 重置所有环境
states = env.reset()

# RL 训练循环
for step in range(max_steps):
    actions = agent.compute_actions(states)  # 你的 RL agent
    next_states, rewards, terminals, infos = env.step(actions)
    ...
```

## 开发阶段

- [ ] **Phase 1**：核心框架 — ECS、GameState、基础 Action/Event、白板随从对战
- [ ] **Phase 2**：基础规则 — 法力水晶、回合流程、战吼、亡语、嘲讽
- [ ] **Phase 3**：完整规则 — 光环、奥秘、武器、复杂交互时序
- [ ] **Phase 4**：RL 接口 — Gym API、批量模拟、张量化观察、奖励函数
- [ ] **Phase 5**：性能优化 — SIMD、零拷贝反序列化、GPU 批量推理

## 贡献

欢迎贡献！请先阅读 [CLAUDE.md](CLAUDE.md) 了解项目架构和编码规范。

## 致谢

- [RosettaStone](https://github.com/utilForever/RosettaStone) — C++ 炉石模拟器，规则引擎的重要参考
- [SabberStone](https://github.com/HearthSim/SabberStone) — C# 炉石模拟器，卡牌实现的重要参考
- [HearthSim](https://hearthsim.info/) — 炉石模拟社区

## 许可

MIT License
