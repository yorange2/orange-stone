# 🟠 Orange Stone

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Phase](https://img.shields.io/badge/phase-3-blue)]()

*English version: [README.md](README.md)*

用 Rust 编写的高性能炉石传说（Hearthstone）模拟器，专为强化学习训练而设计。

## 为什么选择 Orange Stone？

现有的炉石模拟器（[RosettaStone](https://github.com/utilForever/RosettaStone)、[SabberStone](https://github.com/HearthSim/SabberStone)）主要面向对局重现和 AI 对战。Orange Stone 的目标不同——它是为**大规模并行模拟**而生的，核心场景是：

> **保真度是硬性要求**：卡牌效果、结算/触发顺序、目标规则必须与真实炉石语义一致，以 RosettaStone/SabberStone 为正确性基准（见路线图里程碑 F）。下面的工程特性是同一保真引擎的属性——而非简化游戏规则的许可。

- 在 RL 训练中，每步需要模拟数万个对局来收集经验
- 需要确定性的状态回放，便于经验重放和调试
- 需要与 Python ML 生态（PyTorch、JAX、RLlib）无缝对接
- 批量将游戏状态编码为张量，供神经网络消费

Rust 的所有权系统天然适合这一场景：零成本抽象的批量模拟、无 GC 停顿的确定性延迟、编译期保证的并发安全。

## 当前进度

- ✅ **Phase 1**：核心框架 — ECS、GameState (CoW)、基础 Action/Event 循环
- ✅ **Phase 2**：基础规则 — 法力水晶、回合流程、战吼、亡语、嘲讽、圣盾、冲锋、风怒
- 🔄 **Phase 3**：完整规则 — 光环、奥秘、武器、英雄技能、抉择/连击、274+ 张经典卡牌
- ⬜ **Phase 4**：RL 接口 — Gym API、批量模拟、张量化观察、奖励函数
- ⬜ **Phase 5**：性能优化 — SIMD、零拷贝反序列化、GPU 批量推理

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

### 事件驱动规则引擎

```
[Action] → [Event] → [Trigger 匹配] → [新 Event] → ...
```

Action（出牌、攻击、结束回合）产生 Event，Trigger（战吼、亡语、光环、奥秘）响应 Event 产生新事件，在优先队列中按序解析直到队列为空。

### 确定性 + 高性能

| 指标 | 当前 | 目标 |
|------|------|------|
| 单核模拟速度 | ~7,000 matches/s | > 50,000 steps/s |
| 卡牌覆盖 | 316 张经典卡牌 | 全套标准卡牌 |
| 内存占用 | < 1 MB / instance | < 1 MB / game instance |
| 集成测试 | 131 个 | 持续增加 |

## 快速开始

### 前置要求

- Rust 1.85+

### 构建

```bash
git clone https://github.com/yorange2/orange-stone.git
cd orange-stone
cargo build --release
```

### 运行测试

```bash
# 全部测试
cargo test

# 1000轮 Bot 对战（大规模卡牌覆盖测试）
cargo test run_1000 --release -- --nocapture

# 可视化 Bot 对战过程
cargo test bot_game -- --nocapture
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
    actions = agent.compute_actions(states)
    next_states, rewards, terminals, infos = env.step(actions)
    ...
```

## 架构

```
src/
├── core/           # ECS 核心：Entity、Component、World、GameState(CoW)
├── engine/         # 规则引擎：验证、事件入队、效果解析、光环、奥秘
├── cards/          # 卡牌定义：CardDef、vanilla! 宏、274+ 张经典卡牌
├── sim/            # 模拟层：GameBuilder、GreedyBot、SmartBot、RNG
└── lib.rs
tests/
├── gameplay.rs     # 36 个规则引擎集成测试
├── bot_game.rs     # 2 个可视化 Bot 对战测试
└── battle_1000.rs  # 大规模卡牌覆盖测试（1000轮）
```

## 贡献

欢迎贡献！请先阅读 [CLAUDE.md](CLAUDE.md) 了解项目架构和编码规范。

- commit message 使用中文
- 所有 pub API 需有文档注释
- 核心逻辑需有单元测试
- 新增卡牌效果建议添加属性测试

## 致谢

- [RosettaStone](https://github.com/utilForever/RosettaStone) — C++ 炉石模拟器，规则引擎的重要参考
- [SabberStone](https://github.com/HearthSim/SabberStone) — C# 炉石模拟器，卡牌实现的重要参考
- [HearthSim](https://hearthsim.info/) — 炉石模拟社区

## 许可

MIT License
