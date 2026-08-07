# 战吼目标债路线图 — 清偿账本 §12（20 张卡）

> 活跃路线图（尚未归档）。引擎机制路线图 M1 审计（PR #102）逐一核查了所有
> 带目标字段的随从战吼，发现 20 张卡的 `EffectTarget` 与真实炉石不符，已登记到
> F4/F5 账本 `docs/fidelity-debt.md` §12（每张带 `(simplified: …)` 注释），
> RL 池 391 → 371。本路线图逐波清偿。英文镜像：
> `battlecry-target-debt-roadmap.md`。

## 为什么

引擎机制 M1（PR #102）让**机制**生效——`PlayCard { target }` 现在贯穿随从战吼、
G9 哑火被遵守——但这 20 张卡的目标仍然不对。审计分为三组：

- **目标范围错误（11 张）**——引擎能打真实炉石不能打的，或打错一侧：残酷的
  监工、铁炉堡火枪手、暴风城突击队员、精灵弓箭手、火元素、SI:7 特工、
  阿莱克丝塔萨、铁喙猫头鹰、破法者、王牌猎人、暗鳞治愈者。
- **指向性战吼被建模成 `Self_`（7 张）**——选中的目标被丢弃，效果永远打自己：
  圣殿执行者、叫嚣的中士、黑铁矮人、年青/年迈的酒仙、大地之环先知、巫医。
- **效果形状债（2 张）**：疯狂投弹者（`DealDamage AllCharacters` 在
  `resolve_deal_damage` 里是空操作）、霜狼督军（固定 +1/+1 自 buff 而非每个
  其他友方随从 +1/+1）。

## 原则（与 W0–W12 路线图同一契约）

1. **机制先行**：wave 只有在所需机制存在并被 F5 场景钉住后才发（M1/G9 机制
   已就位——这些 wave 大多是改目标或新增 `EffectTarget` 变体）。
2. **每张卡一个差分场景**（`tests/differential.rs`）：目标集合（炉石允许的
   双侧）、G9 哑火、死亡阶段交互；可镜像处做 SabberStone 对照
   （`docs/differential_sabberstone.md`）。
3. **卡池走账本**（fidelity-debt.md 维护约定）：卡修好后删除其 §12 行和
   `(simplified: …)` 注释，然后失效 `~/.cache/orange_stone_debt_ids.txt`——
   RL 池自动回升。§12 清空时池必须回到 391。
4. 每 wave 一个 PR；每 wave 保持 `cargo test` 全绿、fmt/clippy 干净、
   bench 无回退（噪声级）。

## 机制盘点（2026-08-07）

**已存在**（代码核实——M1 贯穿、`select_target` G9 哑火、带 explicit 的解析器）：

| 机制 | 位置 | 使用方 |
| --- | --- | --- |
| `PlayCard { target }` → 战吼（M1） | `Event::MinionSummoned` | 所有 wave |
| G9 哑火（显式目标离开合法集合 → 不回退随机） | `trigger.rs::select_target` | 所有 wave |
| `EffectTarget::FriendlyMinion`（指向） | `env.rs::candidates_for_target`、解析器 | 圣殿执行者、叫嚣的中士、黑铁矮人、酒仙 |
| `EffectTarget::AnyMinion`（指向，双侧） | M1 | 铁喙猫头鹰、破法者 |
| `EffectTarget::EnemyMinionAttackLE`（敌方限定） | `resolve_destroy_minion` | 王牌猎人（改目标） |
| `GainStatsThisTurn`（本回合临时强化） | `trigger.rs::resolve_gain_stats_this_turn` | 叫嚣的中士、黑铁矮人 |
| `GainStatsPerHandCard`（计数模式） | `trigger.rs` | 霜狼督军（每友方随从变体） |
| `ReturnToHand` 指向解析器 | `trigger.rs::resolve_return_to_hand` | 酒仙（改目标为 `FriendlyMinion`） |
| `GameRng` 每局确定性 | `sim::rng` | 疯狂投弹者随机弹 |

**缺失 / 待补**（各组前置）：

- `EffectTarget::AnyCharacter`——任一侧任意角色（英雄+随从）单点指向：
  暴风城突击队员、精灵弓箭手、火元素、SI:7 特工、大地之环先知、巫医。
- `EffectTarget::AnyHero`——任一侧英雄（阿莱克丝塔萨）。
- 暗鳞治愈者的友方角色范围（含英雄）。
- `resolve_restore_health` 目前不收 `explicit_target`——必须收，供大地之环
  先知 / 巫医用。
- 疯狂投弹者战吼：3 次随机 1 伤（新效果变体或伤害解析器的卡 ID 特例）。
- `GainStatsPerFriendlyMinion`（霜狼督军）。
- `validate_play_card` / `legal_actions`：预期无需改动——目标在解析时重新
  校验（G9）；legal 侧只需新增 `candidates_for_target` 分支。

## Wave

### W13 — AnyCharacter / AnyHero 目标机制 + 首批改目标 ✅ 完成（PR #104）

- 新增 `EffectTarget::AnyCharacter` 和 `EffectTarget::AnyHero`；接
  `candidates_for_target`（legal 侧）与各解析器（解析侧，含 G9）；
  `play_targets` 自动覆盖。
- AnyEnemy → AnyCharacter 改目标：**暴风城突击队员、精灵弓箭手、火元素、
  SI:7 特工**（含 combo 路径）。
- 每卡 F5 场景（`w13_*`）：友方目标合法、目标离开集合 G9 哑火。
- **验收**：`cargo test` 全绿（含 `w13_*`）；clippy 干净；账本减 4 行；池 375。

### W14 — 敌方范围修正 ✅ 完成（PR #105）

- **铁炉堡火枪手**：`AnyEnemy` → `AnyEnemyMinion`。
- **铁喙猫头鹰 / 破法者**：`AnyEnemyMinion` → `AnyMinion`（沉默友方随从必须合法）。
- **王牌猎人**：`AnyMinionAttackGE` → 敌方限定（照 `EnemyMinionAttackLE` 模式，
  保留 ≥7）。
- **阿莱克丝塔萨**：`AnyEnemy` → `AnyHero`（只对英雄 SetAttack）。
- **暗鳞治愈者**：友方随从 → 友方角色含英雄（治疗所有友方角色）。
- **残酷的监工**：`AnyEnemyMinion` → `FriendlyMinion`（打伤一个友方随从并
  给它 +2 攻击）。
- 每卡 F5 场景（`w14_*`）。
- **验收**：`cargo test` 全绿；账本减 7 行；池 368。

### W15 — Self_ 指向性战吼 ✅ 完成（PR #106）

- **圣殿执行者**：`Self_` → `FriendlyMinion` +3/+3。
- **叫嚣的中士 / 黑铁矮人**：`Self_` → `FriendlyMinion` `GainStatsThisTurn`
  +2 攻击。
- **年青 / 年迈的酒仙**：`Self_` → `FriendlyMinion`（把友方随从弹回手牌——
  源随从不再弹自己）。
- **大地之环先知 / 巫医**：`Self_` → `AnyCharacter` 治疗——需要
  `resolve_restore_health` 收 `explicit_target`（复用 W13 的 `AnyCharacter`）。
- 每卡 F5 场景（`w15_*`）：选中的友方目标被强化/治疗/弹回；G9 哑火。
- **验收**：`cargo test` 全绿；账本减 7 行；池 361。

### W16 — 效果形状债 + 收尾（一个 PR）

- **疯狂投弹者**：把空操作的 `DealDamage AllCharacters` 战吼换成对其他角色
  的 3 次随机 1 伤（每局 RNG；排除源随从）。
- **霜狼督军**：`GainStats Self_` → 每个其他友方随从 +1/+1
  （`GainStatsPerFriendlyMinion`）。
- 每卡 F5 场景（`w16_*`）。
- 收尾：更新 `orange-reinforcement/hearthstone_os/decks.py` 里过期的池公式
  注释（跨仓库 docs 提交——现在还写着 W8 之前的"24 债 / 368"；本 wave 后应为
  0 / 391）；失效缓存后验证 `_load_debt_ids()` 返回空集；把本路线图归档到
  `docs/finished/`（en + zh）并更新工作区根 CLAUDE.md 指针。
- **验收**：`cargo test` 全绿；§12 清空；RL 池 391；提取器缓存重跑干净。

## Wave 核算

| Wave | 范围 | 卡数 | PR |
| --- | --- | --- | --- |
| W13 AnyCharacter/AnyHero 机制 | 引擎 + F5 | 4 | PR #104 ✅ |
| W14 敌方范围修正 | 引擎 + F5 | 7 | PR #105 ✅ |
| W15 Self_ 指向性战吼 | 引擎 + F5 | 7 | PR #106 ✅ |
| W16 效果形状 + 收尾 | 引擎 + F5 + docs | 2 | 一个 PR（+ 跨仓库 docs） |

## 超出范围

- §12 清单之外的任何内容（过程中发现的新债按账本维护约定登记）。
- RL 侧策略/训练改动——池通过提取器自动回升。
- 168 维观测张量改动。
