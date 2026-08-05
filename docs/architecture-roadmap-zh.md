# 架构路径图 — 发现与优先级

> 最后更新：2026-08-05
> 记录 2026-08-05 架构评审（Orange Stone 对比 RosettaStone (C++) 与 SabberStone (C#)）的发现，以及由此得出的按优先级排列的工作项。
> 关联文档：[architecture-roadmap.md](architecture-roadmap.md)

## 摘要

架构形态正确，符合其目标：类型化 ECS + 代际索引、CoW 游戏状态、确定性事件循环、零卡牌代码的数据驱动效果。

**保真政策（2026-08-05）：绝对保真于炉石传说是硬性要求。** 每张卡牌的效果、结算/触发顺序、目标规则都必须与真实炉石语义一致；RL 训练的工程特性（确定性、回放、廉价克隆、张量化 I/O）是同一引擎的工程属性，而非简化游戏规则的许可。此前"刻意保留"的简化（过载法力锁定、永久潜行、抉择自动随机、单体消灭命中全部匹配随从等）现已成为保真欠债，纳入里程碑 F 跟踪。

状态：里程碑 A–D（热路径、伤害管线、RL 准备、规模）已完成（PR #35–#46）。剩余工作：里程碑 E（工程欠债）与里程碑 F（保真）。

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

> 上表各项的状态见 §2 标注（已解决项对应 PR #35–#46；其余在里程碑 E 与 F 中跟踪）。
| P2 | 代码注释为中文，违反 CLAUDE.md 自身的"注释用英文"规范 | 全库 | 小 |

---

## 1. 定位：为什么本架构与 RosettaStone / SabberStone 不同

一句话对比：RosettaStone 与 SabberStone 是**以保真为先的模拟器**（完整目标选择、完整触发时序、官方 JSON 卡牌数据 + 每卡 Power 脚本）；Orange Stone 是**以 RL 训练为先的环境**（确定性回放、无玩家选择、纯数据卡牌、状态克隆廉价）。

### 保真政策 — 绝对保真是硬性要求

**Orange Stone 必须绝对保真于炉石传说。** 卡牌效果、触发/结算顺序、目标规则与资源机制都必须与真实游戏一致——RosettaStone/SabberStone 的保真优先姿态就是正确性基准；RL 工程特性（确定性、回放、廉价克隆、张量化 I/O）是同一引擎的工程属性，而非简化规则的许可。因此每一条已记录的简化都是**欠债**而非取舍：在里程碑 F 中跟踪并必须消除。执行机制是与 SabberStone/RosettaStone 的差分对局验证（F5）加逐效果保真审计（F4）。

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

- ✅ **已解决（A1，PR #35）** — 事件队列原为每次操作 O(n)（`src/core/event.rs:161` 的 `pop_front` 用 `Vec::remove(0)`；`:149` 的 `push_with_priority` 扫描后插入）。现为三个按优先级分桶的 `VecDeque`：O(1) 入队/出队，同优先级内稳定 FIFO 保持不变。
- ✅ **已解决（A2，PR #36）** — `effective_attack` / `effective_health` / `effective_cost`（`src/core/world.rs`）原各自遍历所有存活光环组件 — 每次查询 O(实体数 × 光环数)。现为 `AuraIndex`：按（拥有者、效果种类）分桶活跃光环源，由光环/区域/玩家相关变异方法增量维护；查询只扫相关分桶，无锁，`World` 保持 `Send + Sync`。

### P1 — 设计债

- ✅ **已解决（D1，PR #44）** — CoW 原为整份 `Inner` 深拷贝（`src/core/state.rs:190`）：`Arc::make_mut` 克隆全部稀疏集 + 区域表。现 `SparseSet` 为分段竞技场（定长页，页表与页均 `Arc` 共享）：克隆 O(1)，首次写入只复制所触页 — 结构共享已兑现 CLAUDE.md 的声明。
- ❌ **未解决（E1）** — 事件处理器中的分配抖动：几乎每个 `apply_event` 分支都先 collect `Vec<Entity>` 以满足借用检查再修改（`src/engine/rules.rs`、`src/engine/trigger.rs` 通篇）。确定且正确，但每个事件都分配。
- ✅ **已解决（B2，PR #39）** — 奥秘原会改写待处理事件队列：攻击伤害 + 反击在 `enqueue` 预排，重定向类奥秘通过 `redirect_damage` / `replace_damage` / `redirect_damages`（`src/core/event.rs:192-253`）改写已入队事件。现攻击经统一 `ResolveAttack` 管线事件结算（反击按当前状态计算）；`redirect_damage` / `replace_damage` 已删除，误导/崇高牺牲使用统一 `redirect_attack` 原语。⚠️ 一个遗留：`redirect_damages` 仍为法术扭曲者保留（法术来源重定向）— 见 E2。

### P2 — 范围/保真度缺口（多为刻意取舍）

- ✅ **已解决（C1，PR #40）** — `Action::PlayCard { card }` 原无目标参数（`src/core/action.rs:13`），所有 `EffectTarget` 由引擎随机解析。现为 `PlayCard { card, target }` + `select_target`（显式目标优先、随机回退），接入 20+ 单目标解析器 — agent 可以学"打脸还是解场"。
- ❌ **未解决（E3）** — 抉择自动随机、连击自动判定 — `src/engine/rules.rs`（抉择走 RNG；连击由 `cards_played_this_turn` 判定）。无玩家决策面。
- ❌ **未解决（F1、F2）** — 已记录的简化，现为保真欠债：过载只是触发标记而非法力锁定（`src/core/component.rs`）；潜行为永久潜行，且其"不能被单目标效果指定"的声称在目标解析中并未实现（`src/core/component.rs`、`src/engine/rules.rs:229`）。按保真政策，这些必须实现，而非仅仅记录。
- ✅ **已解决（C2/C3/C4，PR #42）** — `py_bind/` 与 `rl/` 原不存在，CLAUDE.md 模块图（Phase 4 目标）领先于代码。现 `rl/`（环境、观察、奖励）+ PyO3 `GameEnv`（`py` feature，abi3）已就位且 wheel 实测通过；`sim/battle.rs` + `sim/bot.rs` 仍为自对弈前置基建。
- ⚠️ **部分解决（D2，PR #45）** — 卡牌数据库原为约 400 个手写 `const CardDef`，与官方数据无机制性关联。`build.rs` 生成管线已就绪（官方格式 JSON → 静态卡牌常量，与手写库逐字段验证），但仓库只携带 4 张样例卡 — 完整官方数据库尚未入库（见 E4）。
- ✅ **已解决（D4，PR #46）** — 代码注释原为中文，而 CLAUDE.md 要求英文。全仓库英文化完成（约 2100 行注释）；以注释剥离 diff 验证代码逐字节未动。

---

## 3. 路径图

### 里程碑 A — 热路径（小改动，先做）

- [x] **A1** — `EventQueue`：用 O(1) 出队（队头索引或 `VecDeque`）替换 `Vec::remove(0)` / insert，保持同优先级内稳定 FIFO。*(PR #35：按优先级分桶的 `VecDeque`)*
- [x] **A2** — 光环索引：按（所属玩家、目标类型）分组活跃光环，使 `effective_attack/health/cost` 不再重扫全集。*(PR #36：`AuraIndex` 按（拥有者、效果种类）分桶，增量维护)*
- [x] **A3** — 为事件循环与效果解析添加 criterion 基准测试，使 A1/A2 可量化。*(PR #37：`benches/event_queue.rs` + `benches/effect_resolution.rs`)*

### 里程碑 B — 伤害管线统一（设计工作）

- [x] **B1** — 将伤害结算收敛为单一管线：免疫 → 圣盾 → 护甲 → 生命值 → 死亡检查（`rules.rs` 中的 `DamageDealt` 处理器 + 反击已部分就位）。*(PR #39：提取 `queue_death_events`，攻击经 `ResolveAttack` 统一入管线)*
- [x] **B2** — 将奥秘/反应性效果挂在管线节点上，而非改写已入队事件；退役 `redirect_damage` / `replace_damage` / `redirect_damages` 特判（`src/core/event.rs:192-253`）。*(PR #39：`redirect_attack` 统一原语；`redirect_damage`/`replace_damage` 删除；`redirect_damages` 保留为法术来源重定向原语)*
- [x] **B3** — 用现有自对弈覆盖工具重验全部奥秘卡牌（误导、崇高牺牲、爆炸陷阱、法术扭曲者、蒸发等）在统一管线下的行为。*(PR #39：全部奥秘测试通过 + 3 个新管线测试)*

### 里程碑 C — RL 环境准备（CLAUDE.md Phase 4）

- [x] **C1** — 为 `Action::PlayCard` 增加可选目标（缺省时引擎随机回退），解锁保真的决策空间。*(PR #40：`select_target` 显式目标优先，20+ 单目标解析器接入)*
- [x] **C2** — PyO3 绑定（`py_bind/`）+ Gym-like 环境（`rl/env.rs`）。*(PR #42：`GameEnv` reset/step/legal_actions；`py` feature 门控的 `GameEnv` 类，maturin wheel 实测通过)*
- [x] **C3** — 观察空间张量化（`rl/obs.rs`）。*(PR #42：168 维固定长度观察)*
- [x] **C4** — 奖励函数配置（`rl/reward.rs`）。*(PR #42：稀疏胜/负 + 密集整形分量)*
- [x] **C5** — 批量模拟 API：多个 `GameState` 并行（rayon）。*(PR #41：`BatchSimulator`)*

### 里程碑 D — 规模

- [x] **D1** — 将 CoW 从整份 `Inner` 克隆升级为结构共享（分段 arena / 持久化向量），待实体数量证明其必要性。*(PR #44：`SparseSet` 分段页 + `Arc` 写时复制，克隆 O(1))*
- [x] **D2** — 用 `build.rs` 从官方 CardDefs JSON 生成 `CardDef` const（保留"卡牌即数据"红利，消灭手工劳动）。*(PR #45：官方格式 JSON → 生成的静态卡牌常量，与手写库逐字段验证)*
- [x] **D3** — `GameState` 的 bincode/rkyv 序列化，用于分布式训练。*(PR #43：serde + bincode `to_bytes`/`from_bytes`，`&'static str` 卡牌 ID 经静态库回注)*
- [x] **D4** — 恢复注释语言纪律（英文），与 CLAUDE.md 一致。*(PR #46：全仓库注释英文化)*

### 里程碑 E — 剩余欠债（后续工作）

- [ ] **E1** — 消除逐事件分配抖动（P1）：用借用拆分、小型 Vec 复用或竞技场分配替代 `rules.rs`/`trigger.rs` 中为满足借用检查的 `Vec` collect。
- [ ] **E2** — 退役最后一个队列改写原语 `redirect_damages`（B2 遗留）：让法术扭曲者接入伤害管线（例如在 `DamageDealt` 处做伤害来源拦截），而非改写待结算的法术伤害事件。
- [ ] **E3** — 抉择/连击决策面（P2）：在 `Action` 中暴露抉择，让 agent 决策而非引擎随机 / 自动判定连击。
- [ ] **E4** — 完成 D2：将完整官方 CardDefs JSON 入库并重新生成静态卡牌常量（手写效果字段叠加在生成的静态属性之上）。

### 里程碑 F — 绝对保真（硬性要求）

- [ ] **F1** — 过载法力锁定：按真实炉石语义实现（每张牌各自的过载数值在下一回合锁定拥有者的法力），取代仅作触发标记的现状。
- [ ] **F2** — 潜行完整保真：角色攻击后移除潜行；单目标效果不能指定潜行角色（目前只阻止了攻击）。
- [ ] **F3** — 抉择的玩家选择：在 `Action` 中暴露抉择（引擎随机选择不保真）。
- [ ] **F4** — 保真审计：逐 `CardEffect` 对照真实炉石语义排查；修复已知偏差（例如单体消灭目前会消灭*全部*匹配随从；消灭受伤随从同理；核验目标集合、伤害时序、光环叠加规则）。
- [ ] **F5** — 差分验证：以 SabberStone（及/或 RosettaStone）为参考实现，对局结果 / 事件序列的符合性测试，使保真回归可被机械捕获。

---

## 4. 参考点

- RosettaStone：<https://github.com/utilForever/RosettaStone>（C++）— 以代码实现 Power 的时序保真参考。
- SabberStone：<https://github.com/HearthSim/SabberStone>（C#）— HearthSim 参考实现；我们的卡牌效果语义以它为参照。
- 对比仅在此路径图中维护；CLAUDE.md 仍是设计意图文档。
