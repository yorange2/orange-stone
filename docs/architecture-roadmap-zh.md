# 架构路径图 — 发现与优先级

> 最后更新：2026-08-05
> 记录架构评审（Orange Stone 对比 RosettaStone (C++) 与 SabberStone (C#)）的发现，以及由此得出的按优先级排列的工作项。
> 评审 I（2026-08-05）：热路径与范围发现（里程碑 A–F）。评审 II（2026-08-05）：保真架构 — 对照 RS/SB 的结算顺序与数值修饰模型（里程碑 G）。
> 关联文档：[architecture-roadmap.md](architecture-roadmap.md)

## 摘要

架构形态正确，符合其目标：类型化 ECS + 代际索引、CoW 游戏状态、确定性事件循环、零卡牌代码的数据驱动效果。

**保真政策（2026-08-05）：绝对保真于炉石传说是硬性要求。** 每张卡牌的效果、结算/触发顺序、目标规则都必须与真实炉石语义一致；RL 训练的工程特性（确定性、回放、廉价克隆、张量化 I/O）是同一引擎的工程属性，而非简化游戏规则的许可。此前"刻意保留"的简化（过载法力锁定、永久潜行、抉择自动随机、单体消灭命中全部匹配随从等）现已成为保真欠债，纳入里程碑 F 跟踪。

状态：里程碑 A–D（热路径、伤害管线、RL 准备、规模）已完成（PR #35–#46）。里程碑 E 已完成（PR #51：E1 分配抖动 — E2/E3 并入里程碑 G；E4 仍为数据入库任务）。剩余工作：里程碑 G（保真引擎架构）与里程碑 F（卡牌级保真 — 依赖 G）。

**评审 II 结论（2026-08-05）：** 扁平优先级事件队列 + 即席组件扫描 + 直接写基础数值，无法表达炉石传说的结算语义。RS/SB 依靠**游戏步骤状态机**（回合开始触发 → 法力 → 抽牌 → 行动 → 回合结束触发 → 死亡阶段 → 收尾）、**注册式触发器**（按上场顺序、先手玩家优先）、**死亡分批模型**与**附魔层**（数值修饰）达到保真。里程碑 G 补齐这些原语；里程碑 F 再逐卡对其审计。

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
| 结算模型 | 扁平优先级事件队列 + 逐事件处理器 | GameStep 状态机（Main/Death/Final 步骤） | GameStep 状态机（BeginStep … FinalWrapUp） |
| 光环 | 查询时计算（天然一致） | Aura 管理 + Enchantment | AuraManager + EnchantmentManager |
| 数值修饰 | 直接写基础组件（无附魔层） | GameTag 值上的 Enchantment 对象 | EnchantmentInfo + EnchantmentManager |
| 目标选择 | 显式目标优先 `select_target` + 随机回退（PR #40） | 完整玩家选择 | 完整玩家选择 |
| 开局与抉择 | 无换牌/硬币；抽牌为随机下标 | Choice 系统（Mulligan/General/HeroPower/TaskList） | Choice + ChoiceManager（MULLIGAN/GENERAL/HERO_POWER/TASK_LIST） |
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
- ✅ **已解决（E1，PR #51）** — 事件处理器中的分配抖动：几乎每个 `apply_event` 分支都先 collect `Vec<Entity>` 以满足借用检查再修改（`src/engine/rules.rs`、`src/engine/trigger.rs` 通篇）。现热路径零分配：drain 类 collect 改用 `std::mem::take`（复用既有缓冲），扫描后修改的瞬态列表改用 `SmallList`（`src/core/small_list.rs`）— 栈数组 + 堆溢出，覆盖全部事件处理器、触发、奥秘与光环的目标/触发列表（实际受场上规模约束）。`select_target` 改为接收列表视图；回合开始攻击重置限定 `Zone::Play`（仅场上角色持有攻击状态）。
- ✅ **已解决（B2，PR #39）** — 奥秘原会改写待处理事件队列：攻击伤害 + 反击在 `enqueue` 预排，重定向类奥秘通过 `redirect_damage` / `replace_damage` / `redirect_damages`（`src/core/event.rs:192-253`）改写已入队事件。现攻击经统一 `ResolveAttack` 管线事件结算（反击按当前状态计算）；`redirect_damage` / `replace_damage` 已删除，误导/崇高牺牲使用统一 `redirect_attack` 原语。⚠️ 一个遗留：`redirect_damages` 仍为法术扭曲者保留（法术来源重定向）— 见 E2。

### P2 — 范围/保真度缺口（多为刻意取舍）

- ✅ **已解决（C1，PR #40）** — `Action::PlayCard { card }` 原无目标参数（`src/core/action.rs:13`），所有 `EffectTarget` 由引擎随机解析。现为 `PlayCard { card, target }` + `select_target`（显式目标优先、随机回退），接入 20+ 单目标解析器 — agent 可以学"打脸还是解场"。
- ❌ **未解决（E3）** — 抉择自动随机、连击自动判定 — `src/engine/rules.rs`（抉择走 RNG；连击由 `cards_played_this_turn` 判定）。无玩家决策面。
- ❌ **未解决（F1、F2）** — 已记录的简化，现为保真欠债：过载只是触发标记而非法力锁定（`src/core/component.rs`）；潜行为永久潜行，且其"不能被单目标效果指定"的声称在目标解析中并未实现（`src/core/component.rs`、`src/engine/rules.rs:229`）。按保真政策，这些必须实现，而非仅仅记录。
- ✅ **已解决（C2/C3/C4，PR #42）** — `py_bind/` 与 `rl/` 原不存在，CLAUDE.md 模块图（Phase 4 目标）领先于代码。现 `rl/`（环境、观察、奖励）+ PyO3 `GameEnv`（`py` feature，abi3）已就位且 wheel 实测通过；`sim/battle.rs` + `sim/bot.rs` 仍为自对弈前置基建。
- ⚠️ **部分解决（D2，PR #45）** — 卡牌数据库原为约 400 个手写 `const CardDef`，与官方数据无机制性关联。`build.rs` 生成管线已就绪（官方格式 JSON → 静态卡牌常量，与手写库逐字段验证），但仓库只携带 4 张样例卡 — 完整官方数据库尚未入库（见 E4）。
- ✅ **已解决（D4，PR #46）** — 代码注释原为中文，而 CLAUDE.md 要求英文。全仓库英文化完成（约 2100 行注释）；以注释剥离 diff 验证代码逐字节未动。

### 保真架构 — 结算顺序与数值修饰模型（评审 II，2026-08-05）

评审 II 对比的是引擎的*结算机制*与 RS/SB 的差距。扁平事件队列 + 即席组件扫描 + 直接写组件，无法表达炉石传说的结算顺序；各项均在里程碑 G 中跟踪。

- ✅ **已解决（G1，PR #52）** — 无游戏步骤状态机。现 `Step` 状态机（RS/SB GameStep 模拟，`src/core/state.rs`）：回合开始序列 StartTriggers → ManaRefill → DrawStep → Main；回合结束序列 EndTriggers → WrapUp → 对手 TurnStarted；死亡步骤在 Main 中出现待处理死亡时进入（G3 将改为标记待处理死亡）。`TurnStarted` 不再内联回满法力并抽牌 — 法力回填移入 ManaRefill 步骤、抽牌移入 DrawStep 步骤，回合开始类奥秘/触发（`OnFriendlyTurnStart`、`CardDef::start_turn_effect`）在抽牌*之前*触发；先手玩家第 1 回合免抽（`rules.rs` DrawStep 守卫 + 初始状态不经 DrawStep）；`TurnEnded` 只标记 EndTriggers — 回合结束效果先于收尾清理触发（临时增益在效果结算后才过期）；开局法力修正为先手 1 水晶。
- ✅ **已解决（G2，PR #53）** — 触发检测原为即席组件扫描（稀疏集下标序）。现统一 `Trigger` 组件（`TriggerEvent` + `TriggerTiming`，RS `ITrigger` / SB `TriggerManager` 对应）：按上场顺序（区域表序）触发、当前玩家优先于对手、显式"每当"与"之后"时序、按区域校验有效性；`start_turn_effect` 已接入。新增受伤触发类：苦痛侍僧改为 `ThisMinionDamaged` 抽牌（移除亡语误建模），暴乱狂战士/铸甲师改为 `FriendlyMinionDamaged`（+1 攻击 / +1 护甲，补上 `GainArmor::FriendlyHero` 解析）。旧逐类触发组件（`EndTurnEffect`/`StartTurnEffect`/`SpellTrigger`/`DeathTrigger`/`SummonTrigger`/`OverloadTrigger`）退役，沉默/清除效果组件统一走 `remove_trigger`。
- ✅ **已解决（G3，PR #54）** — 无死亡分批。现待处理死亡标记（`Inner::pending_deaths`，生命 ≤ 0 即"死亡"但仍在场上），死亡步骤在任意步骤边界优先进入（HS 死亡阶段语义），按上场顺序 + 当前玩家优先分批处理，处理完返回被中断的步骤（`return_step`）；`MinionDied` 处理时重新检查生命值（死亡前被治疗到 0 以上 → 存活）；先移入坟场再结算亡语与死亡触发（死亡触发看到死者已移除）。
- ❌ **未解决（G4）** — 增益直接写入基础 Attack/Health（`trigger.rs:660`），费用修饰直接写入基础 Cost（`trigger.rs:414`）。无附魔层 → 沉默（只剥增益、保留基础值与伤害）、变形、复制（无面操纵者）、"直到回合结束"过期、跨区域增益保留均无法表达。
- ✅ **已解决（G5，PR #56）** — 费用原为基础组件 + 即席光环减免（`effective_play_cost`，`rules.rs:121`，校验与扣费两处重复）。现单一费用组成 `engine::cost::play_cost`（校验、扣费、bot 共用）；`World::effective_cost` 为修饰栈：基础 + 附魔增量 → 设置类修饰（`CostModifier`：Set 设值 / Min 下限）→ 手牌光环减免 → 下限 0。`ManaRefill` 步骤已就位 F1 过载锁定的生效点。
- ✅ **已解决（G6，PR #57）** — 决策面原仅有 `PlayCard { card, target }`，抉择走引擎随机。现抉择系统：`GameEngine::apply_choices` 返回 `Resolution::{Done, NeedsChoice}`，暂停时把 `PendingChoice`（`ChoiceKind`：ChooseOne / Discover / Mulligan，含选项标签与 Discover 卡池）交给玩家，`Action::Choose { choice_id, option }` 继续结算；`GameEngine::apply` 为默认策略（随机 — RNG 确定性保持，bot/自对弈不变）。抉择（法术与随从，如塞纳留斯/丛林守护者）、发现（`AddRandomCardToHand` 卡池即选项）、召唤位置（`PlayCard.position`，0=最左）、英雄技能目标（`HeroPower.target`）全部解锁；`Event::ChoiceResolved` 承载选择结算。
- ✅ **已解决（G7，PR #58）** — 原无开局流程：`GameState::new` 空牌库开局、抽牌取随机下标。现开局流程：`GameBuilder::build` 洗牌（Fisher–Yates，种子确定）；抽牌改为有序牌库**牌顶抽牌**（退役随机下标）；`GameState::begin_game` 发起手（先手 3 张、后手 4 张 + 硬币），并以 G6 协议挂出换牌抉择（`ChoiceKind::Mulligan` — 换回牌库、重洗、再抽牌顶；硬币不可换），先手玩家第 1 回合免抽（G1 已就位）；新增硬币卡 `THE_COIN`（`GainManaThisTurn` — 本回合 +1 法力，不增加永久水晶）。
- ✅ **已解决（G8，PR #59）** — 奥秘原事后匹配事件（`secret.rs:26`）：`WhenEnemySpellCast` 在法术效果结算*之后*才触发，反制类奥秘无法抢先。现出牌边界拦截（`secret::intercept_counter_secrets`）：法术反制在效果结算前否定法术（顺带修复法术反制从未注册 `Secret` 组件、完全不触发的 bug — `Secret::effect` 改为可选）；法术扭曲者先召唤 1/3 标记并把法术单体效果重定向到它（AOE 不受影响，符合炉石）；退役 `redirect_damages` 队列改写（E2）。
- ❌ **未解决（G9）** — 出牌目标仅在校验时做候选集成员检查，非法时随机回退（`trigger.rs:34`）；炉石精确的重新校验/空发规则缺失；潜行的"不能成为单目标"条款未落实（`rules.rs:229`）。

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

- [x] **E1** — 消除逐事件分配抖动（P1）：用借用拆分、小型 Vec 复用或竞技场分配替代 `rules.rs`/`trigger.rs` 中为满足借用检查的 `Vec` collect。*(PR #51：drain 处 `mem::take` + `SmallList` 栈缓冲列表)*
- [ ] **E2** — 退役最后一个队列改写原语 `redirect_damages`（B2 遗留）：让法术扭曲者接入伤害管线（例如在 `DamageDealt` 处做伤害来源拦截），而非改写待结算的法术伤害事件。*(⏩ 并入 G8：步骤边界拦截取代队列改写)*
- [ ] **E3** — 抉择/连击决策面（P2）：在 `Action` 中暴露抉择，让 agent 决策而非引擎随机 / 自动判定连击。*(⏩ 由 G6 取代：抉择系统覆盖)*
- [ ] **E4** — 完成 D2：将完整官方 CardDefs JSON 入库并重新生成静态卡牌常量（手写效果字段叠加在生成的静态属性之上）。

### 里程碑 G — 保真引擎架构（F 的前置条件）

评审 II 结论：以下原语正是 RS/SB 达到保真的手段 — GameStep 状态机、注册式触发器、死亡分批与附魔层。F 的卡牌级修复只有在它们之上才有意义；先做 G，再做 F4/F5。

- [x] **G1** — 游戏步骤状态机：仿照 RS/SB 的 `GameStep` 状态机建模炉石的结算步骤（回合开始触发 → 法力 → 抽牌 → 行动 → 回合结束触发 → 死亡阶段 → 收尾）；优先级事件队列退化为步骤*内部*的事件流，步骤由状态进入（有待处理死亡 → 进入死亡步骤）。修复回合开始/结束顺序、临时效果过期、首回合抽牌。*(PR #52：`Step` 状态机 + `advance_step`；死亡步骤事件级分批，G3 升级为标记式)*
- [x] **G2** — 注册式触发器：以逐实体触发器注册（RS `ITrigger` / SB `TriggerManager`）取代即席 `iter_*_trigger` 扫描：按上场顺序触发、先手玩家优先、显式"每当"与"之后"时序、按区域校验有效性。接入 `start_turn_effect`（已声明但从未触发）；补充缺失的触发类（受伤触发 — 解锁苦痛侍僧、暴乱狂战士、铸甲师）。*(PR #53：统一 `Trigger` 组件 + `fire_triggers`)*
- [x] **G3** — 死亡分批：待处理死亡标记（生命 ≤ 0 即"死亡"但仍在场上）、死亡按上场顺序在同一个步骤内处理、`MinionDied` 在处理时重新检查生命值（在自身死亡被处理前被治疗/加血到 0 以上 → 存活）、死亡触发看到随从已被移除。*(PR #54：`pending_deaths` + 死亡步骤优先进入/返回中断步骤)*
- [x] **G4** — 附魔层：数值变为基础值 + 附魔增量（+ 伤害）而非直接写组件；增益/减益/费用效果挂附魔，光环保持查询时计算。使沉默（只剥附魔）、变形、复制（无面操纵者）、"直到回合结束"过期正确。*(PR #55：`Enchantment` + `Damage` 组件；设置类修饰遗留至 G5)*
- [x] **G5** — 费用管理器：费用 = 基础值 + 修饰栈，遵循炉石规则（下限 0、"不能低于 X"、设为固定值、冻结费用）；退役即席 `effective_play_cost` 组合。过载（F1）在法力回满步骤锁定法力。*(PR #56：`engine::cost::play_cost` + `CostModifier` 栈；冻结费用类与 F1 锁定由 F1 落地)*
- [x] **G6** — 抉择系统：在 `Action` 中暴露 `Choice` 对象（SB ChoiceType：Mulligan / General / HeroPower / TaskList），覆盖抉择、发现、对手抉择（生而平等式）、召唤位置、英雄技能目标。*(吸收 E3 与 F3。)* *(PR #57：`apply_choices` 暂停协议 + `PendingChoice`/`Action::Choose`；Mulligan 类由 G7 使用)*
- [x] **G7** — 开局流程：对局开始洗牌、起手（3 / 4 张 + 硬币）、换牌、有序牌库 + 牌顶抽牌（退役随机下标抽牌）、先手玩家首回合免抽。*(PR #58：build 洗牌 + 牌顶抽牌 + `begin_game` 起手/硬币/换牌抉择)*
- [x] **G8** — 奥秘拦截：奥秘在步骤边界触发 — 反制类（法术反制、法术扭曲者）在效果结算*之前*，后置类保持现有触发点。退役 `redirect_damages`（E2）。*(PR #59：出牌边界拦截 + `Secret::effect` 可选化)*
- [ ] **G9** — 结算时的目标合法性：按炉石规则在结算时重新校验出牌目标（新获潜行、目标被移除、"不能被指定"）；精确的重新选择/空发语义；落实潜行的单目标排除（F2 的目标侧）。

### 里程碑 F — 绝对保真（硬性要求）

卡牌级修复；其依赖的引擎原语在里程碑 G 落地。

- [ ] **F1** — 过载法力锁定：按真实炉石语义实现（每张牌各自的过载数值在下一回合锁定拥有者的法力），取代仅作触发标记的现状。*(由 G1/G5 解锁：锁定在法力回满步骤生效)*
- [ ] **F2** — 潜行完整保真：角色攻击后移除潜行；单目标效果不能指定潜行角色（目前只阻止了攻击）。*(目标侧随 G9 落地)*
- [ ] **F3** — 抉择的玩家选择：在 `Action` 中暴露抉择（引擎随机选择不保真）。*(⏩ 由 G6 取代)*
- [ ] **F4** — 保真审计：逐 `CardEffect` 对照真实炉石语义排查；修复已知偏差（例如单体消灭目前会消灭*全部*匹配随从；消灭受伤随从同理；核验目标集合、伤害时序、光环叠加规则）。*(被 G2–G5 阻塞：先有结算/附魔原语，逐效果修复才有意义)*
- [ ] **F5** — 差分验证：以 SabberStone（及/或 RosettaStone）为参考实现，对局结果 / 事件序列的符合性测试，使保真回归可被机械捕获。*(依赖 G1–G9：差分对比的是步骤级结算)*

---

## 4. 参考点

- RosettaStone：<https://github.com/utilForever/RosettaStone>（C++）— 以代码实现 Power 的时序保真参考。
- SabberStone：<https://github.com/HearthSim/SabberStone>（C#）— HearthSim 参考实现；我们的卡牌效果语义以它为参照。
- 对比仅在此路径图中维护；CLAUDE.md 仍是设计意图文档。
