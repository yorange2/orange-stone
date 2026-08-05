# 架构路径图 — 发现与优先级

> 最后更新：2026-08-05
> 记录 2026-08-05 架构评审（Orange Stone 对比 RosettaStone (C++) 与 SabberStone (C#)）的发现，以及由此得出的按优先级排列的工作项。
> 关联文档：[architecture-roadmap.md](architecture-roadmap.md)

## 摘要

架构形态正确，符合其目标（RL 训练）：类型化 ECS + 代际索引、CoW 游戏状态、确定性事件循环、零卡牌代码的数据驱动效果。与 RosettaStone/SabberStone 相比，刻意做出的取舍都有依据。真正的缺口有两处：**P0 热路径数据结构**（O(n) 事件队列、光环全量扫描）与 **P1 设计债**（整份状态 CoW 拷贝、奥秘通过改写队列中待处理事件实现而非统一伤害管线）。

| 优先级 | 事项 | 位置 | 工作量 |
|--------|------|------|--------|
| P0 | `EventQueue::pop_front` 是 `Vec::remove(0)` — 每次出队 O(n) | `src/core/event.rs:161` | 小 |
| P0 | 光环解析每次查询全量扫描 — O(实体数 × 光环数) | `src/core/world.rs:609` | 中 |
| P1 | CoW 是整份 `Inner` 深拷贝，并非结构共享（CLAUDE.md 声称与现实不符） | `src/core/state.rs:190` | 中 |
| P1 | 每个事件处理器先 collect `Vec` 绕借用检查 — 分配抖动 | `src/engine/rules.rs`（多处） | 中 |
| P1 | 伤害结算在 `enqueue` 中预排；奥秘改写待处理事件 — 新增重定向类卡牌需逐个特判 | `src/engine/rules.rs:341`、`src/core/event.rs:192` | 大 |
| P2 | `Action` 无目标选择 — 法术/战吼目标由引擎随机决定 | `src/core/action.rs:13` | 大 |
| P2 | `py_bind/` 与 `rl/` 尚不存在；CLAUDE.md 模块图落后于代码 | — | 大 |
| P2 | 卡牌数据库为约 400 个手写 const；考虑用 `build.rs` 从官方 JSON 生成 | `src/cards/` | 中 |
| P2 | 代码注释为中文，违反 CLAUDE.md 自身的"注释用英文"规范 | 全库 | 小 |

---

## 1. 定位：为什么本架构与 RosettaStone / SabberStone 不同

一句话对比：RosettaStone 与 SabberStone 是**以保真为先的模拟器**（完整目标选择、完整触发时序、官方 JSON 卡牌数据 + 每卡 Power 脚本）；Orange Stone 是**以 RL 训练为先的环境**（确定性回放、无玩家选择、纯数据卡牌、状态克隆廉价）。

| 维度 | Orange Stone | RosettaStone | SabberStone |
|------|--------------|--------------|-------------|
| 实体模型 | 类型化 ECS 组件（SparseSet、Copy）+ 代际索引 | 扁平 `GameTag → int` Tag 字典 | 类继承树 + Tag 字典 |
| 悬垂引用 | 结构上不可能（`Entity{index, generation}`） | 可能，手工管理 | 可能，手工管理 |
| 状态模型 | `Arc` Copy-on-Write，clone O(1) | 每局单可变状态 | 单可变状态 + GC |
| 卡牌实现 | **纯数据**：`CardDef` const + 封闭 `CardEffect` 枚举，零卡牌代码 | JSON + C++ Power 类 | JSON + C# Power 类 |
| 触发系统 | `apply_event` 内按组件特判 | 触发器注册表 + 事件链 | TriggerManager + TaskQueue + GameStep 状态机 |
| 光环 | 查询时计算（天然一致） | Aura 管理 + Enchantment | AuraManager + EnchantmentManager |
| 目标选择 | 无 — 引擎随机选目标 | 完整玩家选择 | 完整玩家选择 |
| 随机性 | `GameRng` 内嵌于 `GameState`，可复现 | 每局 RNG，非回放设计 | 带 seed 的 `System.Random` |
| RL 工具链 | 自对弈运行器 + 机器人 + 卡牌覆盖追踪 | — | — |

### 应保留的优势

- **安全模型**：代际索引 + 类型化组件 + CoW，把 RS/SB 的经典 bug 类型（变形/回手/亡语期间的悬垂实体引用）变成编译期错误。
- **确定性优先**：同优先级稳定 FIFO 事件队列（`event.rs` 刻意弃用 `BinaryHeap`）、内嵌 RNG、完整事件日志作为回放数据 — RS 与 SB 均未做到。
- **零卡牌代码**：卡牌是静态数据，可静态校验（ID 唯一性、`pool.rs` 卡池封闭性）、覆盖追踪、测试。
- **性能底子**：无 GC、无虚调用、Copy 组件、SoA 稀疏集 — 理论上限远高于 SabberStone，高于 RosettaStone 的 Tag 查表。
- **测试基建**：`GameBuilder`（绕过规则直接构造状态）、`sim/battle.rs`（带 `CardTracker` 的自动化自对弈）— 三项目中独有。

---

## 2. 已知问题

### P0 — 热路径数据结构

- **事件队列每次操作 O(n)** — `src/core/event.rs:161`（`pop_front` 用 `Vec::remove(0)`）与 `:149`（`push_with_priority` 扫描后插入）。事件循环是引擎的绝对热路径；应改为 `VecDeque` 或带队头索引的实现（保持同优先级内稳定 FIFO 语义）。
- **光环解析每次查询全量扫描** — `effective_attack` / `effective_health` / `effective_cost`（`src/core/world.rs:609, 640, 672`）各自遍历所有存活光环组件。热路径（攻击验证、伤害结算、出牌验证）上 O(实体数 × 光环数)。结构上永远正确，但自对弈量上来后应做索引（例如按所属玩家 + 目标类型分组）。

### P1 — 设计债

- **CoW 是整份状态深拷贝** — 共享 clone 后首次修改，`Arc::make_mut` 克隆整个 `Inner`（全部稀疏集 + 区域表）（`src/core/state.rs:190`）。CLAUDE.md 声称"共享不变部分"（持久化数据结构），目前并不成立。在当前的实体数量下没问题；要做 MCTS 式分支时这是第一个要升级的点。
- **事件处理器中的分配抖动** — 几乎每个 `apply_event` 分支都先 collect `Vec<Entity>` 以满足借用检查再修改（`src/engine/rules.rs` 通篇）。确定且正确，但每个事件都分配。
- **奥秘改写待处理事件队列** — 攻击结算（攻方伤害 + 反击）在 `enqueue` 中预排（`src/engine/rules.rs:341-369`），重定向类奥秘（误导、崇高牺牲、法术扭曲者等）通过 `EventQueue::redirect_damage` / `replace_damage` / `redirect_damages`（`src/core/event.rs:192-253`）改写已入队的事件。这是引擎中最不优雅的角落：每张新的重定向类卡牌都需要在 `src/engine/secret.rs` 中特判，而不是接入统一的伤害结算管线。

### P2 — 范围/保真度缺口（多为刻意取舍）

- **无玩家目标选择** — `Action::PlayCard { card }` 没有目标参数（`src/core/action.rs:13`）；所有 `EffectTarget` 由引擎随机解析。对随机自对弈足够，但 agent 永远学不会选择"打脸还是解场"。在 RL 环境保真之前需要补齐。
- **抉择自动随机、连击自动判定** — `src/engine/rules.rs:570-586`（抉择走 RNG）与 `:536`（连击由 `cards_played_this_turn` 判定）。无玩家决策面。
- **已记录的简化** — 过载只是触发标记而非法力锁定（`src/core/component.rs:380`）；潜行为永久潜行（`src/core/component.rs:365`）。必须持续记录，以免与真实炉石做对局验证时踩坑。
- **`py_bind/` 与 `rl/` 不存在** — CLAUDE.md 模块图（Phase 4 目标）领先于代码。当前 `sim/battle.rs` + `sim/bot.rs` 是 RL 测试的前置基建。
- **卡牌数据库为手写** — `src/cards/` 下约 400 个 `const CardDef`。类型检查完备、速度飞快，但每个新系列都是手工劳动，且与官方卡牌数据没有机制性关联。
- **注释语言漂移** — 代码注释为中文，而 CLAUDE.md 要求英文注释。

---

## 3. 路径图

### 里程碑 A — 热路径（小改动，先做）

- [x] **A1** — `EventQueue`：用 O(1) 出队（队头索引或 `VecDeque`）替换 `Vec::remove(0)` / insert，保持同优先级内稳定 FIFO。*(PR #35：按优先级分桶的 `VecDeque`)*
- [x] **A2** — 光环索引：按（所属玩家、目标类型）分组活跃光环，使 `effective_attack/health/cost` 不再重扫全集。*(PR #36：`AuraIndex` 按（拥有者、效果种类）分桶，增量维护)*
- [x] **A3** — 为事件循环与效果解析添加 criterion 基准测试，使 A1/A2 可量化。*(PR #37：`benches/event_queue.rs` + `benches/effect_resolution.rs`)*

### 里程碑 B — 伤害管线统一（设计工作）

- [ ] **B1** — 将伤害结算收敛为单一管线：免疫 → 圣盾 → 护甲 → 生命值 → 死亡检查（`rules.rs` 中的 `DamageDealt` 处理器 + 反击已部分就位）。
- [ ] **B2** — 将奥秘/反应性效果挂在管线节点上，而非改写已入队事件；退役 `redirect_damage` / `replace_damage` / `redirect_damages` 特判（`src/core/event.rs:192-253`）。
- [ ] **B3** — 用现有自对弈覆盖工具重验全部奥秘卡牌（误导、崇高牺牲、爆炸陷阱、法术扭曲者、蒸发等）在统一管线下的行为。

### 里程碑 C — RL 环境准备（CLAUDE.md Phase 4）

- [ ] **C1** — 为 `Action::PlayCard` 增加可选目标（缺省时引擎随机回退），解锁保真的决策空间。
- [ ] **C2** — PyO3 绑定（`py_bind/`）+ Gym-like 环境（`rl/env.rs`）。
- [ ] **C3** — 观察空间张量化（`rl/obs.rs`）。
- [ ] **C4** — 奖励函数配置（`rl/reward.rs`）。
- [ ] **C5** — 批量模拟 API：多个 `GameState` 并行（rayon）。

### 里程碑 D — 规模

- [ ] **D1** — 将 CoW 从整份 `Inner` 克隆升级为结构共享（分段 arena / 持久化向量），待实体数量证明其必要性。
- [ ] **D2** — 用 `build.rs` 从官方 CardDefs JSON 生成 `CardDef` const（保留"卡牌即数据"红利，消灭手工劳动）。
- [ ] **D3** — `GameState` 的 bincode/rkyv 序列化，用于分布式训练。
- [ ] **D4** — 恢复注释语言纪律（英文），与 CLAUDE.md 一致。

---

## 4. 参考点

- RosettaStone：<https://github.com/utilForever/RosettaStone>（C++）— 以代码实现 Power 的时序保真参考。
- SabberStone：<https://github.com/HearthSim/SabberStone>（C#）— HearthSim 参考实现；我们的卡牌效果语义以它为参照。
- 对比仅在此路径图中维护；CLAUDE.md 仍是设计意图文档。
