# CLAUDE.md

## 项目概述

Orange Stone 是一个用 Rust 编写的炉石传说（Hearthstone）模拟器，主要面向强化学习（Reinforcement Learning）训练场景。

## 设计目标

### 保真要求（硬性约束）

**绝对保真于炉石传说是硬性要求**：卡牌效果、结算/触发顺序、目标规则与资源机制必须与真实炉石语义一致，以 RosettaStone/SabberStone 为正确性基准（执行机制见 Phase 3 的对局结果对比验证与 `docs/finished/architecture-roadmap.md` 里程碑 F）。以下 RL 诉求是同一引擎的工程属性，而非简化游戏规则的许可；任何已记录的简化（过载法力锁定、永久潜行、抉择自动随机等）都是待消除的欠债，不得新增。

### 核心诉求：强化学习友好

与 RosettaStone (C++) 和 SabberStone (C#) 等现有模拟器相比，Orange Stone 的首要设计目标是服务于 RL 训练：

1. **批量推理 (Batched Inference)**：支持同时运行大量对局实例，充分利用 GPU 进行并行推理。
2. **高吞吐量**：每秒可模拟数十万甚至百万步，使 RL agent 能在合理时间内完成足够多的探索。
3. **确定性回放 (Deterministic Replay)**：给定相同的初始状态和动作序列，能够完全重现对局，便于经验回放和调试。
4. **Python 绑定**：通过 PyO3 提供 Python 接口，与现有的 RL 框架（如 PyTorch、TensorFlow、RLlib）无缝集成。
5. **Game State 序列化**：支持高效的游戏状态序列化/反序列化，便于分布式训练中的状态传输。

### 相比现有方案的优势

| 问题 | RosettaStone (C++) | SabberStone (C#) | Orange Stone (Rust) |
|------|-------------------|-------------------|---------------------|
| 内存安全 | 手动管理，易出 Bug | GC 导致不可预测的延迟 | 所有权系统，零成本抽象 |
| 并发安全 | 需要小心加锁 | GC 停顿影响吞吐 | 编译期保证无数据竞争 |
| 批量模拟 | 需自行实现 | 受 GC 限制 | 天然适合并行，无 GC 停顿 |
| Python 集成 | 通过 pybind11 | pythonnet（较重） | PyO3，性能接近原生 |
| 性能 | 最高 | 受 GC 影响 | 接近 C++，更安全 |

## 技术栈

- **语言**：Rust (stable, edition 2024)
- **Python 绑定**：PyO3 + maturin
- **序列化**：serde + bincode / rkyv（零拷贝反序列化）
- **并行**：rayon + tokio（异步 I/O）
- **SIMD**：可能使用 portable-simd 加速批量运算
- **测试**：cargo test + proptest（属性测试）
- **基准测试**：criterion

## 架构设计原则

### 1. ECS (Entity Component System) 架构

游戏实体（随从、英雄、武器、法术等）使用 ECS 架构管理，而非传统的继承树：

```
Entity (卡片实例)
  ├── Component: Health, Attack, Cost, ...
  ├── Component: Aura, Deathrattle, Battlecry, ...
  └── Component: Zone (DECK, HAND, PLAY, GRAVEYARD, ...)
```

**优势**：
- 缓存友好的数据布局（SoA），SIMD 友好
- 灵活的卡牌效果组合，无需复杂的多继承
- 批量处理同类 Component，提升模拟吞吐
- 热插拔效果，方便实现复杂的卡牌交互

### 2. 不可变游戏状态 + Copy-on-Write

```
GameState (不可变)
  ├── 对局元数据 (turn, phase, active_player, ...)
  ├── Entity 数组
  └── Zone 状态 (每个区域的实体列表)
```

每次状态变更产生新的 GameState，共享不变的部分（类似 Git 的对象模型或 Clojure 的持久化数据结构）。这带来：
- 无锁并发：多个 worker 可以同时读取同一个状态
- 回滚简单：保留状态指针即可回退
- MCTS / 树搜索友好：分支时不复制全部数据

### 3. 事件驱动规则引擎

```
[Action] → [Event] → [Trigger 匹配] → [新 Action] → ...
```

- Action：玩家主动操作（出牌、攻击、英雄技能等）
- Event：游戏规则产生的事件（随从死亡、回合开始、抽牌等）
- Trigger：卡牌效果对事件的响应（亡语、光环、奥秘等）
- 队列按优先级和顺序解析，直到队列为空

### 4. 模块划分

```
src/
├── core/           # 核心抽象
│   ├── entity.rs   # Entity + Component 定义
│   ├── state.rs    # GameState + CoW 实现
│   ├── zone.rs     # Zone 枚举 (Deck, Hand, Play, ...)
│   ├── action.rs   # Action 枚举
│   └── event.rs    # Event 枚举 + EventQueue
├── engine/         # 规则引擎
│   ├── rule.rs     # 规则处理器
│   ├── trigger.rs  # Trigger 匹配与调度
│   └── aura.rs     # 光环系统
├── cards/          # 卡牌定义（数据驱动）
│   ├── loader.rs   # 卡牌数据加载器
│   └── db/         # 卡牌数据库 (JSON/YAML)
├── sim/            # 模拟器
│   ├── game.rs     # 对局管理
│   ├── rng.rs      # 可复现的随机数生成器
│   └── replay.rs   # 对局回放
├── rl/             # 强化学习接口
│   ├── env.rs      # Gym-like 环境接口
│   ├── obs.rs      # 观察空间定义
│   └── reward.rs   # 奖励函数
├── py_bind/        # Python 绑定 (PyO3)
│   └── mod.rs
└── lib.rs
```

### 5. 批量模拟架构

```
┌─────────────────────────────────────────┐
│              Python RL Agent             │
│         (PyTorch / JAX / RLlib)          │
└──────────┬──────────────────┬────────────┘
           │ actions          │ states, rewards
           ▼                  ▲
┌──────────────────────────────────────────┐
│           Orange Stone (Rust)            │
│  ┌────────┐ ┌────────┐ ┌────────┐       │
│  │ Game 1 │ │ Game 2 │ │ Game N │  ...  │
│  └────────┘ └────────┘ └────────┘       │
│       ▲           ▲           ▲          │
│       └───────────┴───────────┘          │
│          Batch State → Tensor            │
└──────────────────────────────────────────┘
```

## 开发阶段

### Phase 1: 核心框架
- ECS 框架、GameState、Zone、基础 Action/Event
- 简单的随从交换对局（无效果的白板随从）

### Phase 2: 基础规则
- 法力水晶、抽牌、回合流程
- 战吼、亡语、嘲讽等基础关键词
- 随机数生成（可复现）

### Phase 3: 完整规则
- 光环、奥秘、武器、英雄技能
- 复杂的卡牌交互时序
- 与 SabberStone/RosettaStone 的对局结果对比验证

### Phase 4: RL 接口
- Gym-like Python 环境
- 批量模拟 API
- 观察空间张量化
- 奖励函数配置

### Phase 5: 性能优化
- SIMD 加速
- 零拷贝反序列化
- GPU 批量推理集成

## 随机性管理

对于 RL 训练至关重要：

```rust
/// 可复现的随机数生成器
/// 每个对局有自己的 RNG 状态，确保对局结果可复现
pub struct GameRng {
    rng: SmallRng,  // 或 Xoshiro256StarStar
    seed: u64,
    calls: Vec<RngCall>,  // 记录所有随机调用用于回放
}
```

- 所有随机事件（抽牌、伤害随机范围、发现等）通过统一的 RNG 接口
- 支持设定 seed 复现对局
- 支持记录和重放随机调用序列

## 参考项目

- [RosettaStone](https://github.com/utilForever/RosettaStone) (C++) — 规则引擎设计参考
- [SabberStone](https://github.com/HearthSim/SabberStone) (C#) — 卡牌实现参考
- [HearthSim](https://hearthsim.info/) — 炉石模拟社区

## 编码规范

- `cargo fmt` 格式化
- `cargo clippy` 无警告
- 所有 pub API 有文档注释
- 核心逻辑有单元测试
- 新增卡牌效果需要属性测试验证正确性
- 性能敏感路径有 benchmark
- 所有代码注释应当使用英文
- 所有 commit message 和 PR message 应当使用英文
- **文档同步**：`README.md`（英文）和 `README_zh.md`（中文）必须始终保持内容同步。任何对其中一份文档的更新，必须同时以对应语言更新另一份文档，两份文档的结构、信息量和时效性应当完全一致。
