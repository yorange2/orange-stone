# 开放池卡路线图 —— 实现 4 张被跳过的复制卡（并打标记）

> 状态：**活跃**（2026-08-07 创建）。英文对照：`pool-open-cards-roadmap.md`。
> 范围：`docs/classic-cards-zh.md` 标 ⏸️ 的 4 张卡 —— 心灵视界、思维窃取、
> 心灵游戏、游学者周卓。它们当初按[池封闭](finished/classic-cards-roadmap.md#pool-closure)
> 规则被移出经典卡路线图。本路线图忠实实现这 4 张卡，**并引入一个永久标记**，
> 让池封闭这条契约在将来支持其他系列时仍然可审计。

## 当初为什么跳过，为什么现在不再是阻塞

经典卡路线图要求卡池**封闭**：任何已实现的卡都不能产出经典系列之外的牌。引擎里
所有随机产牌都天然满足这一点 —— `cards/pool.rs` 只从 `ALL_CARDS` 的过滤子集和
固定衍生物池里抽。

这 4 张卡**在构造上**无法满足，因为它们根本不抽池，而是读**对手真实的牌**：

| 卡牌 | 职业 | 费用 | 效果 | 读取来源 |
| --- | --- | --- | --- | --- |
| 心灵视界 (Mind Vision) | 牧师 | 1 | 复制对手手牌中随机一张进自己手牌。 | 敌方手牌 |
| 思维窃取 (Thoughtsteal) | 牧师 | 3 | 复制对手牌库中 2 张牌进自己手牌。 | 敌方牌库 |
| 心灵游戏 (Mindgames) | 牧师 | 4 | 把对手牌库里随机一个随从的复制召唤到战场。 | 敌方牌库 |
| 游学者周卓 (Lorewalker Cho) | 中立传说 | 2 | 任一玩家施法后，将其复制放入另一名玩家手牌。 | 被施放的法术 |

**今天不存在泄漏**：`ALL_CARDS` 全是经典卡，对手手牌/牌库里的东西本来就在池内。
风险在**将来**：一旦支持第二个系列，这 4 张就是唯一能把牌跨池搬运的卡。标记正是
为此而设 —— 把封闭性从一条没写下来的假设，变成一条带已知例外清单的可检查性质。

**非目标**：支持多系列。本路线图只把例外变得显式、可被机器读取。

---

## 标记方案

### 契约

一张卡是**开放池卡**（pool-open），当且仅当它的结算会把某张牌搬进某个区域，而这张
牌并非从 `cards/pool.rs` 控制的池里抽出来的。

三个部分，一个事实来源：

1. **注册表（权威、机器可读）** —— `sets::POOL_OPEN_CARDS: &[&str]`，与
   `ALL_CARDS` 并列的卡 ID 列表。
2. **代码注释（给人看）** —— 每张卡的文档块写 `(pool-open: reads the opponent's
   deck)` 之类。**刻意**与保真账本的 `(simplified: …)` 用不同关键词：这些卡**不是**
   欠债，它们是忠实实现；Python 的欠债提取器认的是 "simplified"，不能把它们捞进去。
3. **文档** —— 新建常设注册表 `docs/pool-openness.md`（+ `-zh`）；
   `classic-cards.md` / `-zh.md` 图例新增 🔓，这 4 行的 ⏸️ 改为 ✅ 🔓。

### 强制不变量（M0 测试）

M1 新增的 4 个「读区域」效果变体**只允许**出现在 `POOL_OPEN_CARDS` 里的卡上；
注册表里每个 ID 都必须能被 `card_by_id` 解析。这样将来任何读对手区域的新卡，要么
登记进注册表，要么 `cargo test` 挂掉。

### 被否决的方案

给 `CardDef` 加 `pool_open: bool` 字段。`CardDef` 有约 415 处穷举式结构体字面量且
没有 `Default`，加一个字段就要改每一处字面量，而表达力并不比 ID 注册表多。

### 决策点 D1（已定）—— RL 训练卡池保留这 4 张

`hearthstone_os/decks.py::full_pool()` 现在过滤掉硬币、衍生物（`id` 以 `t` 结尾）
和保真债 ID，得到 392 张。

**已定：保留（392 → 396），同时把标记暴露出去，让改主意只差一个开关。** 今天池是
封闭的，收进来完全站得住；这几张是忠实实现，排除只会白白损失训练多样性。M0 交付
`GameEnv.pool_open_card_ids()` 和 `full_pool(include_pool_open: bool = True)`，
将来真上了第二个系列，改默认值是一行的事。

这个开关不是摆设 —— 它是让 D1 可逆的逃生口。谁要加第二个系列，就在同一次改动里把
默认值改成 `False`，池立刻回到封闭的 392，引擎一行都不用动。

---

## 里程碑

一个里程碑一个 PR。每个 PR 都要保持 `cargo test` 通过、`cargo fmt` / `cargo clippy`
干净、`cargo bench` 在噪声水平。F5 差分场景放 `tests/differential.rs`，前缀 `po_*`
（沿用 `w0_*` 的惯例）。

### M0 —— 标记机制（先不动卡牌行为）

- [x] `sets::POOL_OPEN_CARDS` 注册表（初始为空，由 M2/M3 填充）。
- [x] `docs/pool-openness.md` + `-zh.md`：上述契约、卡表，以及对照
      `fidelity-debt.md` 的维护约定（新增开放池卡必须在同一次改动里同时补注册表行、
      代码注释、文档行）。
- [x] 在 `fidelity-debt.md` / `-zh.md` 的维护章节加交叉引用：pool-open ≠ simplified，
      这些卡的注释里不许出现 "simplified" 字样。
- [x] `py_bind`：新增静态方法 `GameEnv.pool_open_card_ids()`（对照 `all_card_ids()`）。
- [x] Rust 测试 `pool_open_registry_is_well_formed`：ID 都能解析、都不是衍生物、无重复。
- [x] RL 侧（orange-reinforcement 单独 PR）：`full_pool(include_pool_open=True)`，
      并加一个测试断言开关正好让池大小变化 `len(registry)`。

### M1 —— 引擎原语

- [x] **效果变体**（`core/effect.rs`）—— 注意每个都要在 `CardEffect` **和** serde 镜像
      `CardEffectDe` 里各加一个分支，外加 `From` 实现：
      - `CopyRandomEnemyHandCard { count }` —— 心灵视界。
      - `CopyRandomEnemyDeckCards { count }` —— 思维窃取；对牌库实体**不放回**抽样
        （同名卡的两张是两个实体，可以都被抽中；同一个实体不能被抽两次）。
      - `SummonRandomEnemyDeckMinion { fallback_card_id }` —— 心灵游戏。
      - `CopyCastSpellToOtherPlayerHand` —— 游学者周卓。
- [x] **复制辅助函数**（`engine/trigger.rs`）：`copy_card_to_hand(state, src_entity,
      to_player)` —— 用 `card_by_id` 解析源实体的 `card_id`，复用 `add_card_to_hand`。
      复制的是**基础卡定义**，不带区域内的附魔（作为已知细节写进文档；符合经典时期
      行为，也让复制品与新产出的牌不可区分）。
- [x] **全局法术触发**：新增 `TriggerEvent::AnySpellCast`，在
      `rules.rs::trigger_applies` 的全局分支里与 `SecretPlayed` / `MinionDied` 并列；
      `Event::SpellCast` 处理要把法术实体作为 `subject` 传下去 ——
      `fire_triggers(..., player, Some(spell), None)`。现有 `FriendlySpellCast` 触发都
      没有 `race` / `max_attack` 条件，所以由 `None` 改成传 subject 在行为上是中性的；
      用一个回归场景钉住这一点。
- [x] **注册钩子**：周卓的触发按卡 ID 在 `cards::apply_card_keywords` 里注册（现成的
      「按 ID 挂特殊关键词」钩子 —— 理由同 M0 否决的方案：不加 `CardDef` 字段）。
- [x] 确定性：所有随机挑选都走 `state.rng_mut()`；补一个回放测试（同 seed + 同动作
      序列 → 复制结果逐位一致）。

### M2 —— 三张牧师法术

- [x] `PRIEST_024` **心灵视界**（1 费）。敌方手牌为空 → 不产牌，但法术照常消耗。
- [x] `PRIEST_025` **思维窃取**（3 费）。牌库只剩 1 张 → 只复制 1 张；牌库空 → 0 张；
      不触发疲劳（没有「抽牌」发生）。
- [x] `PRIEST_026` **心灵游戏**（4 费）+ `PRIEST_026t` **虚无的暗影**（0/1 衍生物，
      `t` 后缀让它天然不进 RL 池）。真实炉石：对手牌库**不被修改**；牌库里没有随从
      则召唤虚无的暗影；战场已满（7）则什么都不召唤 —— `resolve_summon` 现有的战场
      上限检查已覆盖。
- [x] 注册表 + `(pool-open: …)` 注释 + `docs/pool-openness.md` 行。
- [x] F5 场景 `po_mind_vision_*`、`po_thoughtsteal_*`、`po_mindgames_*`（上述每个边界
      各一个）。

### M3 —— 游学者周卓

- [x] `LEGENDARY_024` **游学者周卓**（2 费 0/4 中立传说）。
- [x] 方向规则：复制品给**施法者的对手**，不是给周卓的主人。结算时读 subject 法术的
      归属（`world.player(subject)`）再取 `.opponent()` —— 所以周卓主人自己施法时，
      是在给敌方送牌。
- [x] 场景：(a) 周卓主人施法 → 敌方拿到复制；(b) 敌方施法 → 周卓主人拿到；
      (c) 双方各一个周卓 → 各触发一次，且复制不连锁（进手牌不算施法）；
      (d) 法术打死周卓时，只有触发点上周卓还活着才复制 —— 以 SabberStone 为准钉住
      结论并写进文档；(e) 周卓被沉默 → 不再复制。
- [x] 注册表 + 注释 + 文档行。

### M4 —— 卡池、文档与 RL 同步

- [ ] `classic-cards.md` / `-zh.md`：4 行 ⏸️ → ✅ 🔓，图例加 🔓，分职业表与总计更新
      （牧师已实现 24 → 27、中立传说 23 → 24、总计已实现 371 → 375、跳过 4 → 0）。
- [ ] `ALL_CARDS` 415 → 420 个 ID（4 张可收藏 + 1 张衍生物）；`battle_1000.rs` 打印
      新的数量。
- [ ] 用 venv 解释器重编 wheel
      （`maturin build --release --interpreter …/.venv/bin/python`）并重装，重新实测
      `full_pool()` → 预期 **396**。
- [ ] 跑 `tools/orange_stone_m5_smoke.py` 的卡池压力测试（每张卡都能正常打出）。
- [ ] 吞吐检查：`cargo bench` + ~970 局/s 基线。周卓的产牌会拉长对局，要报出差值，
      不要默认「没影响」。
- [ ] 在工作区 `CLAUDE.md` 登记本路线图；完成后把两份文件移入 `docs/finished/`。
- [ ] **不需要重训**。392 → 396 是 1% 的卡池变化，且现有卡的引擎语义没变，与 PR #108
      的情况不同。如果仍要重训，必须沿用同一口径（`--pool full` 30k × 3 seed），
      否则数字不可比。

### M5 —— 手牌上限与爆牌（D2：已定，本路线图内做）

引擎**没有 10 张手牌上限、也没有爆牌**（`add_card_to_hand` 和
`draw_card_no_queue` 都是无条件追加）。这是疲劳路线图明确推给账本的既有缺口，而且
目前哪儿都没登记。

周卓和思维窃取会让「手牌溢出」从罕见变成常态，而 `rl/obs.rs` 在
`MAX_HAND = 10` 处截断手牌 —— 第 10 张之后的牌对 agent 不可见却仍可打出，是一处
静默的观测/动作错配。

- [ ] 在 `fidelity-debt.md` 的机制清单里登记为 F-A11（这是真欠债，和 pool-open 标记
      性质不同）。
- [ ] 实现：手牌上限 10；**抽到**的第 11 张被销毁（爆牌，进墓地，但仍算「已抽」，
      牌库照常消耗）；**产出**的第 11 张根本不创建。
- [ ] 场景：满手抽牌爆牌；满手心灵视界无事发生；9 张手牌时思维窃取恰好复制 1 张；
      心灵游戏不受影响（它召唤的是战场，不占手牌）；周卓任一方向满手都无事发生；
      疲劳交互不变。
- [ ] F-A11 按欠债离开账本的同一套流程销掉：实现 + F5 场景 + 移除账本行（这里不涉及
      `(simplified: …)` 注释 —— F-A11 是机制清单缺口，不是逐卡标记）。
- [ ] `rl/obs.rs` 不需要改动：上限落地后 `MAX_HAND = 10` 从「截断」变成「精确边界」。
      在 obs 测试里加一条断言钉住 —— 手牌永远不会超过 `MAX_HAND`。

**D2（已定）：在本路线图内做，即 M5。** 周卓和思维窃取正是让这个缺口咬人的卡，
修复要跟着暴露它的卡一起落地，而不是先留下观测/动作错配再开后续。M5 是关闭本路线图
的前置条件，不是可选的尾巴。

---

## 风险

| 风险 | 对策 |
| --- | --- |
| 周卓每次施法都给双方产牌 → 对局变长、吞吐下降 | M4 实测；若 局/s 明显回退，合并前先报出来 |
| 复制品绕过构筑规则（同名超过 2 张、手里出现别的职业卡） | 这符合真实炉石 —— 用场景钉住，免得以后被当 bug「修掉」 |
| `Event::SpellCast` 开始传 subject，对现有法术触发是静默行为变更 | 对现有 `FriendlySpellCast` 卡加回归场景 |
| 心灵游戏遇到满场 / 牌库无随从 | 显式场景；复用 `resolve_summon` 的上限检查 |
| `(pool-open: …)` 注释被 Python 提取器误当成保真债 | 提取器认的是 "simplified"；M0 补交叉引用说明和测试 |

## 完成标准

`POOL_OPEN_CARDS` 恰好 4 个 ID；封闭性测试禁止未注册的读区域效果；
`classic-cards.md` 的 ⏸️ 归零；RL 池实测 396；`docs/pool-openness.md`（+ zh）成为常设
注册表；两份路线图文件归档到 `docs/finished/`。
