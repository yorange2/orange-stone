# 保真债 — 简化卡清单（F4/F5 持续审计账本）

> **现状：账本已清空——`src/cards/` 里没有任何 `(simplified: …)` 标记，RL 卡池
> 为满池 392 张（415 个卡 ID − 22 个衍生物 − 硬币）。**
>
> 历程：67 张保真债在 W0~W7 全部清偿（W0 接线 PR #79 清 13 张、W1 种族 PR #80 清 11 张、
> W2 触发 PR #81 清 8 张、W3 谓词 PR #82 清 9 张、W4 费用/武器 PR #83 清 8 张、
> W5 目标结构 PR #84 清 7 张、W6 特殊机制 PR #85 清 8 张、W7 收尾 PR #86 清最后 3 张）。
> 2026-08-06 补登记轮又发现 27 张已知简化卡没有标记、一直静默混在 RL 卡池里，登记为
> §11（F-A8 过载修复后降为 24 张）；§11 已由 W8~W12（PR #97~#101）全部清偿。
> 引擎机制路线图 M1 的战吼目标审计登记了 20 张进 §12，已由 W13~W16（PR #104~#107）
> 全部清偿。2026-08-07 的 classic-cards 状态复审又查出 10 张仍有偏差的卡，当日全部
> 修复并记录为 §13（这批从未进过债务集合，卡池始终是 392）。
> 本账本是 F4 逐效果保真审计的**权威记录**。一张卡**离开账本**的唯一条件：
> 真实炉石效果已实现**且**通过 F5 差分测试验证。不要静默重写卡牌——改动必须
> 同时更新本账本、代码注释和下游简化债提取器（见[维护约定](#维护约定)）。
>
> **2026-08-06 修复轮（PR #77）**：下面的结构性发现（F-A1…F-A7）已全部解决——
> 4 处过期注释清理、Worgen Infiltrator 修复、3 处卡 ID 冲突修复、10 张卡补入
> `ALL_CARDS`（7 个重复条目去重）、Python 提取器重写（PR #31）。剩下的是
> 下面各组的逐机制实现工作与 F5 验证协议。
>
> **2026-08-06 W0 接线轮（PR #79）**：13 张接线卡全部落地（机制已存在，只差
> 接线）——见路线图 W0。顺带补齐四个小原语：`EffectTarget::EventSubject`
> （触发事件主体作目标，正义之剑）、`EffectTarget::OtherFriendlyMinion`
> （"另一个"友方随从，女祭司/铁匠）、武器实体注册触发 + 摧毁后离场（正义之剑
> 断剑后不再触发）、法术结算后先处理死亡再触发施法触发（野炎术师被自己的法术
> 杀死后不触发）。对应 16 个差分场景（`tests/differential.rs` 的 `w0_*`）。
>
> **2026-08-06 W1 种族轮（PR #80）**：11 张种族卡全部落地——`CardDef.race`
> 字段（Beast/Murloc/Demon，召唤时生效，`EntityView`/Python 绑定暴露 race）、
> 种族条件目标（`FriendlyRace` / `AllOtherFriendlyRace` / `AnyRace`）、种族条件
> 光环（`FriendlyRace` / `OtherFriendlyRace` 目标 + `GrantCharge` 冲锋光环，
> 苔原犀牛）、种族条件触发（`Trigger.race` 字段——鱼人招潮者 / 食腐土狼 /
> 饥饿的秃鹫）、按种族过滤牌库抽牌（感知恶魔）、硬编码 `BEAST_POOL` /
> `DEMON_POOL` 换成字段驱动池（含逐位一致测试，`w1_race_pools_are_field_driven`）。
> 对应 12 个差分场景（`tests/differential.rs` 的 `w1_*`）。
>
> **2026-08-06 W2 触发轮（PR #81）**：8 张触发/奥秘卡全部落地——5 个新触发类：
> `CharacterHealed`（治疗，任何角色被治疗都触发）、`Attacked`（实体攻击——
> 智慧祝福把"本随从攻击时抽牌"挂在目标随从身上）、`CardPlayed`（打出卡牌，
> 友方作用域）、`SecretPlayed`（奥秘打出，双方都触发）、`MinionDied`（任意
> 随从死亡，双方都触发）；以及 3 个摧毁奥秘效果（SI:7 随机一个、吞秘巨蟒 /
> 照明弹全部 + 组合增益/抽牌）。治疗触发只在"真的治疗到"时触发（满血角色
> 不是治疗事件）。对应 8 个差分场景（`tests/differential.rs` 的 `w2_*`）。
>
> **2026-08-06 W3 谓词轮（PR #82）**：9 张条件谓词卡全部落地——攻击区间目标
> （`EnemyMinionAttackLE` 科多兽、`AnyMinionAttackGE` 猎潮驯兽师）、手牌数计数
> （`GainStatsPerHandCard` 暮光幼龙）、英雄血量阈值（`MortalStrike`）、受伤
> 友方/任意目标（`DamagedFriendlyMinion` / `DamagedMinion` 狂暴）、受伤计数
> （`DrawPerDamagedFriendlyCharacter` 战斗怒火）、控制奥秘（
> `GainStatsIfOwnSecret` 以太奥秘学者）、"每回合首个随从"状态（
> `AuraEffect::FirstMinionDiscount` + 每玩家 `minions_played_this_turn`
> 计数器，微型召唤师）、圣盾吸收（`AbsorbDivineShields` 血骑士，每盾 +3/+3，
> 双方圣盾都吸收）。对应 9 个差分场景（`tests/differential.rs` 的 `w3_*`）。
>
> **2026-08-06 W4 费用/武器轮（PR #83）**：8 张费用与武器互动卡全部落地——
> 手牌区费用光环（`IncreaseMinionCost` 法力怨魂——双方都加费；
> `IncreaseMinionCostFriendly` 风险投资公司雇佣兵——只加自己的，都叠在 G5
> 修正栈上）、按武器攻击减费（恐怖海盗，`play_cost` 中减武器攻击力）、武器
> 耐久削减（血帆海盗，耐久归零即摧毁）、武器装备谓词（南海船工——
> `ChargeWithWeapon` 光环，`effective_charge` 检查武器在场）、敌方法术 0 费
> （米米尔隆——`spells_cost_zero` 玩家标志，回合结束清除）、给对手水晶
> （奥术傀儡——空水晶）。对应 9 个差分场景（`tests/differential.rs` 的
> `w4_*`）。
>
> **2026-08-06 W5 目标结构轮（PR #84）**：7 张目标结构与效果组合卡全部落地——
> `SetPlayedMinionHealth`（忏悔——奥秘把打出的随从生命设为 1，带事件上下文）、
> `SilenceAllEnemyMinionsAndDraw`（群体驱散）、`SwapAttackAndHealth`（疯狂炼金师，
> 用附魔差量表达交换）、`FreezeAdjacent`（冰锥术——冻结随机敌方随从及其邻位）、
> `GrantAdjacentTaunt`（日怒保卫者）与 `GrantAdjacentSpellDamage`（远古法师）、
> `FullHealAndTaunt`（先祖治疗——满血 + 嘲讽组合）。对应 7 个差分场景
> （`tests/differential.rs` 的 `w5_*`）。
>
> **2026-08-06 W6 特殊机制轮（PR #85）**：8 张特殊机制卡全部落地——概率效果
> （`ChanceDraw` 纳特·帕格 50% 抽牌）、本回合临时增益（`GainStatsThisTurn`
> 法力沸腾者，回合结束自动清除）、群体圣盾（`GrantDivineShieldAllFriendly`
> 正义）、排除自身的全场伤害（`YseraAwakens`——伊瑟拉之醒放过伊瑟拉）、
> 抽牌-按费用伤害（`DrawAndDamageByCost` 神圣愤怒）、受伤友方回合开始治疗
> （`RestoreDamagedFriendly` 光明之泉——从回合结束改到回合开始）、群体增益+嘲讽
> （`GainStatsAndTauntAllFriendly` 自然之力）。顺手牵羊核实时其实已忠实——
> `OtherClass` 池用职业组表过滤非潜行者卡，只清了过期注释。对应 8 个差分场景
> （`tests/differential.rs` 的 `w6_*`）。
>
> **2026-08-06 W7 收尾轮（PR #86）**：最后 3 张全部落地——手牌区交换
> （`SwapWithHandMinion` 闹钟机器人）、伤害反射奥秘（`ReflectDamage` 以眼还眼——
> 新 `SecretTrigger::WhenFriendlyHeroDamaged`）、1 生命复活奥秘
> （`ResurrectDiedMinion` 救赎）。对应 3 个差分场景（`tests/differential.rs`
> 的 `w7_*`）。**账本至此清空；RL 卡池 391 张 = 全经典构筑池满规模。**
>
> **执行计划**：[docs/fidelity-debt-roadmap-zh.md](../finished/fidelity-debt-roadmap-zh.md)
> （英文版 `finished/fidelity-debt-roadmap.md`）——按依赖排序的 8 个 wave（W0 接线 …
> W7 收尾）覆盖全部 67 张卡；一张卡完成 = 账本行、代码注释、差分场景三者
> 同时落地。

**权威来源**：`src/cards/classic_*.rs` 中卡牌常量上的 `(simplified: ...)` 文档注释。
Python 侧（`orange-reinforcement/hearthstone_os/decks.py::_load_debt_ids`）靠解析
这些注释提取清单，并把它们排除出 RL 训练卡池——所以注释措辞的改动会波及训练
卡池（任何修改后记得使 `~/.cache/orange_stone_debt_ids.txt` 缓存失效）。

67 处标记对应 67 个唯一卡 ID——修前的 3 处 ID 冲突（Sword of Justice /
Repentance / Lightwell）已改成官方 ID（EX1_365 / EX1_349 / EX1_341），
Mass Dispel 现在可取了。67 张全部在 `ALL_CARDS` 里（补入 10 张、去重 7 个
重复条目后共 413 个唯一条目，PR #77）。其中 4 处是过期注释（卡已忠实，已
清理，见 §10）；真实债务是 67 张——**67 张全部清偿（W0 13 + W1 11 +
W2 8 + W3 9 + W4 8 + W5 7 + W6 8 + W7 3），账本在 2026-08-06 W7 收尾轮
PR #86 后一度清空，同日补登记 27 张既有简化债（§11）重新开账**。

---

## 账本（按缺失机制分组）

### 1. Enrage — 受伤条件增益（4 张）✅ 已解决（W0，PR #79）

4 张全部接线完成：接上已有的 `ThisMinionDamaged` 触发槽（`apply_card_keywords`
按卡 ID 注册，与 Acolyte of Pain 同模式），增益为永久附魔。差分场景：
`w0_gurubashi_berserker_enrage_permanent`、`w0_tauren_warrior_enrage_with_taunt`、
`w0_angry_chicken_enrage_fires_before_death`、`w0_spiteful_smith_buffs_weapon_on_damage`。

### 2. 种族 — 野兽 / 鱼人 / 恶魔（9 张）✅ 已解决（W1，PR #80）

全部落地：`CardDef.race` 字段 + 种族条件目标/光环/触发 + 牌库过滤抽牌 +
字段驱动池（差分场景 `w1_*`，池一致性见 `w1_race_pools_are_field_driven`）。
F-A6 补注的 Starving Buzzard（HUNTER_013）与 Scavenging Hyena（HUNTER_014）
也随本轮离开账本。

### 3. 事件触发 — 召唤 / 治疗 / 死亡 / 奥秘 / 攻击 / 打出（2 张）✅ 已解决（W7，PR #86）

Alarm-o-Bot（手牌区交换）随 W7 落地；Ethereal Arcanist（控制奥秘谓词）随
W3 落地——本节清空（`w7_alarm_o_bot_swaps_with_hand_minion`）。

### 4. 条件目标与状态（9 张）✅ 已解决（W3，PR #82）

全部落地：攻击区间（≤2 / ≥7）、手牌数计数、英雄血量阈值、受伤友方/任意目标、
受伤计数、控制奥秘、"每回合首个随从"状态、圣盾吸收（差分场景 `w3_*`）。

### 5. 邻位 / 多目标（4 张）✅ 已解决（W5，PR #84）

全部落地：邻位冻结（冰锥术）、邻位增益（日怒保卫者嘲讽 / 远古法师法强）、
交换攻击/生命（疯狂炼金师）（差分场景 `w5_*`）。

### 6. 费用与武器条件光环（8 张）✅ 已解决（W4，PR #83）

全部落地：手牌区费用光环（全局/己方）、按武器攻击减费、武器耐久削减、
武器装备谓词（条件冲锋）、敌方法术 0 费、给对手水晶（差分场景 `w4_*`）。

### 7. 本回合临时增益（1 张）✅ 已解决（W6，PR #85）

`GainStatsThisTurn`——附魔层 `UntilEndOfTurn` 到期（差分场景 `w6_mana_addict_buff_expires_at_turn_end`）。

### 8. 概率效果（1 张）✅ 已解决（W6，PR #85）

`ChanceDraw`——回合结束时按概率抽牌（差分场景 `w6_nat_pagle_chance_draw`）。

### 9. 复合与其他（8 张）✅ 已解决（W5 PR #84 + W6 PR #85）

W5 清偿：神圣愤怒（抽牌-按费用伤害）、先祖治疗（满血 + 嘲讽）。
W6 清偿：正义（群体圣盾）、伊瑟拉之醒（排除自身的全场伤害）、光明之泉（受伤友方
回合开始治疗）、自然之力（群体增益 + 嘲讽）、顺手牵羊（职业过滤抽卡——W6 当时判定
已忠实；2026-08-06 的修正把池子从「任意非潜行者卡」收窄为其余八个职业的职业卡）。
差分场景 `w5_*` / `w6_*`。

### 10. 已解决 — 标记简化但实际已忠实（4 张，PR #77 清理）

注释早于落地 PR（前三张是 #72 潜行；Multi-Shot 的 `DealDamageToTwo` 随双随机目标
效果落地）。def 里本来就是真实效果，过期的 "simplified" 措辞已删除——这也会把
它们从 Python 简化债集合里移除（4 张卡回到 RL 卡池）。

| ID | 卡名 | 修了什么 |
| --- | --- | --- |
| NEUTRAL_C10 | Jungle Panther | 注释清理——def 里 `stealth: true` |
| NEUTRAL_T14 | Stranglethorn Tiger | 注释清理——def 里 `stealth: true` |
| NEUTRAL_T15 | Ravenholdt Assassin | 注释清理——def 里 `stealth: true` |
| HUNTER_012 | Multi-Shot | 注释清理——`DealDamageToTwo` 就是真实效果 |

---


### 11. 2026-08-06 补登记 — 既有简化债（27 → 24 → 19 → 15 → 10 → 5 → 0 张）✅ 已清空

2026-08-06 状态审计（`docs/classic-cards-zh.md` 对照代码）发现 27 张已知简化卡
没有 `(simplified: …)` 标记——一直静默混在 RL 卡池里，当时全部补注释并登记于此
（F-A8 过载修复后出账 3 张，剩 24 张）。清偿过程：
W8（路线图 PR #97）清 5 张：阿曼尼狂战士 / 暴怒的狼人 / 格罗玛什 / 战歌指挥官 /
北郡牧师；W9（PR #98）清 4 张：真银圣剑 / 血吼 / 鹰角弓 / 狂野怒火；
W10（PR #99）清 5 张：愤怒 / 利爪德鲁伊 / 知识古树 / 战争古树 / 追踪术；
W11（PR #100）清 5 张：奥妮克希亚 / 死亡之翼 / 海巨人 / 伺机待发 / 冷血；
W12（PR #101）清最后 5 张：水元素 / 秘教暗影祭司 / 先知维伦 / 毒刃 / 阿古斯保护者。
**本节清空，RL 卡池回到全经典构筑池 391 张。**

### 12. 战吼目标集合 — M1 审计（20 → 16 → 9 → 2 → 0 张）✅ 已清空

引擎机制路线图 M1 的审计（随从战吼显式目标）逐张核对了每个带目标效果的随从战吼，
把引擎的 `EffectTarget` 与真实炉石对照，20 张不匹配的卡按维护约定登记于此
（每张都带 `(simplified: …)` 注释，RL 卡池一度降至 371）。
错误范围的行由 W13（PR #104）与 W14（PR #105）清偿，`Self_` 建模的行由
W15（PR #106）清偿，效果形状的行由 W16（PR #107）清偿。
**本节清空——RL 卡池回到满池 392（415 个卡 ID − 22 个衍生物 − 硬币；路线图里写的
391 是在过期的 414 卡 wheel 上量的，见 PR #104）。**

*（20 行于 2026-08-07 由引擎机制 M1 审计登记，同日由战吼目标债路线图 W13~W16 全部清偿）*

### 13. classic-cards 状态复审 — 2026-08-07（10 张）✅ 已清空

对照代码复审 `docs/classic-cards.md` 时，发现 10 张卡的实现仍与真实炉石有偏差。
它们都**没有** `(simplified: …)` 标记、也从未登记进本账本，因此从来没有被排除出
RL 卡池——整个过程卡池一直是 392。这 10 张当日全部修复，而不是登记为债务。

| ID | 卡名 | 原先 | 现在 |
| --- | --- | --- | --- |
| CLASSIC_018 | 阿曼尼狂战士 | 永久可叠加 +3 | 条件激怒 |
| NEUTRAL_008 | 暴怒的狼人 | 永久可叠加 +1 与风怒 | 条件激怒 |
| NEUTRAL_C11 | 牛头人战士 | 永久可叠加 +3 | 条件激怒 |
| NEUTRAL_C15 | 恶毒铁匠 | 永久武器 +2 | 条件激怒 |
| NEUTRAL_R02 | 愤怒的小鸡 | 永久可叠加 +5 | 条件激怒 |
| WARRIOR_010 | 格罗玛什 | 永久可叠加 +6 | 条件激怒 |
| WARRIOR_008 | 战歌指挥官 | 冲锋光环覆盖所有其他友方 | 召唤触发，攻击力 ≤3 |
| PRIEST_004 | 北郡牧师 | 友方*角色*被治疗 | 任意随从、双方均可 |
| LEGENDARY_010 | 奥妮克希亚 | 固定 5 只雏龙 | 填满战场（空场 6 只） |
| PRIEST_012 | 先知维伦 | +1 法术伤害（等于没做） | 法术/英雄技能的伤害与治疗翻倍 |

配套的引擎改动：

- **`Enrage` 组件**（`core/component.rs`）：攻击加成 + 可选风怒 + 可选武器攻击加成。
  它在 `World::effective_attack` 与 `World::max_attacks` 里**读取时求值**，而不是写进
  附魔——这正是它不叠加、治疗回满立刻失效、并且能被沉默剥离的原因。古拉巴什狂暴者
  保留原来的 `ThisMinionDamaged` 触发，因为它的真实文本确实是永久可叠加增益。
  `compute_attacker_damage` 改用 `effective_attack` 读武器，恶毒铁匠的加成才能进到
  英雄的攻击里。
- **`Trigger.max_attack`**：触发事件主体的攻击力上限条件，与已有的 `race` 条件并列。
  战歌指挥官用它；冲锋是一次性授予那个随从的，因此在指挥官离场后依然保留。
- **`TriggerEvent::MinionHealed`**：取代 `FriendlyCharacterHealed`（只有北郡牧师在用）。
  全局作用域，只认随从。
- **法术伤害管线**。`World::total_spell_damage` 早就存在，但**零调用者**：法术伤害
  只是存在场上，从未施加到任何伤害上——全套法术伤害卡（狗头人地卜师、达拉然法师、
  食人魔法师、大法师、血法师萨尔诺斯、青玉龙、玛里苟斯、远古法师）实际都等同白板，
  维伦的「+1 法术伤害」也毫无作用。现在 `trigger::apply_spell_power` 会对来源是法术
  或英雄技能的效果改写伤害与治疗数值——先加法术伤害，再由维伦翻倍，与炉石一致
  （心灵之火 5 → 有地卜师 6 → 有维伦 10 → 两者都有 12）。攻击、战吼、亡语不受影响，
  这同时收窄了维伦的治疗翻倍：此前它会把该玩家产生的**所有**治疗翻倍，连巫医的战吼
  也算在内。

F5 覆盖：`w8_amani_berserker_enrage`、`enrage_does_not_stack_and_ends_at_full_health`、
`enrage_is_removed_by_silence`、`w8_raging_worgen_enrage_and_windfury`、
`w0_spiteful_smith_buffs_weapon_while_damaged`、`w8_grommash_hellscream_enrage`、
`w8_warsong_commander_charges_only_small_summons`、
`warsong_charge_outlives_the_commander`、
`w8_northshire_cleric_draws_on_any_minion_heal`、
`northshire_cleric_ignores_a_heal_that_restores_nothing`、
`w11_onyxia_fills_the_board_with_whelps`、`spell_damage_boosts_spell_damage`、
`spell_damage_does_not_boost_battlecry_damage`、
`prophet_velen_doubles_spell_damage_after_spell_damage`、
`prophet_velen_leaves_minion_effects_alone`。全量 `cargo test` 通过（488 项）；
`cargo clippy --all-targets` 无警告；`cargo bench` 在受影响路径上无显著变化
（`effective_stats/aura_board_14_minions`、`effect_resolution/*`，p 均 > 0.05）。

## 2026-08-06 审计发现（已在 PR #77 / PR #31 全部解决）

这些是整理账本时挖出来的 F4 工作项。所有结构性发现都已解决；剩下的是上面
各组的逐机制实现工作与 F5 验证协议。

- ✅ **F-A1 — 4 处过期注释**（见 §10）。已解决：注释里的 "(simplified…)" 已删；
  Python 提取器不再排除这些卡（缓存已失效）；RL 卡池 +4（3 张潜行随从 +
  Multi-Shot）。
- ✅ **F-A2 — Worgen Infiltrator（NEUTRAL_C08）**：潜行机制已有时仍是白板。
  已解决：手写 def，`stealth: true`。
- ✅ **F-A3 — 卡 ID 冲突（3 处）**：
  - `SWORD_OF_JUSTICE` 原来用 `"PALADIN_017"`（= Holy Wrath）→ 现为 EX1_365
  - `REPENTANCE` 原来用 `"PALADIN_018"`（= Righteousness）→ 现为 EX1_349
  - `LIGHTWELL` 原来用 `"PRIEST_018"`（= Mass Dispel）→ 现为 EX1_341；
    **Mass Dispel 现在可取了**（之前 `card_by_id("PRIEST_018")` 返回 Lightwell）。
- ✅ **F-A4 — 10 张不在 `ALL_CARDS`**（Houndmaster、Tundra Rhino、Flare、Sword
  of Justice、Repentance、Blessing of Wisdom、Scavenging Hyena、Starving
  Buzzard、Eye for an Eye、Redemption）。按"漏加"处理：10 张全部补入
  `ALL_CARDS`（现 413 个唯一条目）。RL 卡池不受影响——6 张简化卡和 4 张新标注
  的卡（F-A6）都在简化债集合里被排除。
- ✅ **F-A5 — `ALL_CARDS` 原有 7 个重复常量条目**（410 项 / 403 个唯一）。
  已解决：去重后为 413 个唯一条目（403 + 补入 10）。
- ✅ **F-A6 — 未标注的简化**：Starving Buzzard（HUNTER_013）和 Scavenging Hyena
  （HUNTER_014）是种族简化（"召唤一个随从"/"一个友方随从死亡" vs 真实的
  "**野兽**"）但没写标记；Eye for an Eye 和 Redemption（PALADIN_020/021）是只有
  触发没有效果的奥秘。已解决：四张都补了显式 `(simplified: …)` 注释，成为已
  记录债务，且不会进 RL 卡池。
- ✅ **F-A7 — Python 简化债提取器错位**（PR #31 已修）：
  `hearthstone_os/decks.py::_load_debt_ids` 按 `pub const` 切块，把每处
  `simplified` 注释记到了**上一张**卡的 ID 上。导致 RL 训练卡池混进约 12 张真
  简化卡（Nat Pagle、Multi-Shot、Secretkeeper、Mass Dispel、Rampage、Cone of
  Cold、Ancestral Healing 等）、漏掉约 15 张干净卡——321 卡池的规模纯属巧合。
  已改成"注释解析正下方的常量"。`~/.cache/orange_stone_debt_ids.txt` 缓存已
  失效；M5 的训练数字是修前卡池产出的，重训后会漂移。


## F-A8 — 8 处卡 ID 冲突 + 过载接线错位 ✅ 已解决（PR #88，2026-08-06）

F-A3 修过 3 处 ID 冲突；本轮又发现 8 处。`card_by_id` 取首个匹配，每对里有一张
卡永远无法按 ID 引用，且过载接线（`cards/mod.rs::apply_card_keywords` 按裸 ID
匹配）随之错位：

| ID | 卡 A | 卡 B |
| --- | --- | --- |
| MAGE_016 | 冰锥术 | 法力浮龙 |
| PALADIN_016 | 阿古斯保护者 | 受祝福的勇士 |
| PRIEST_016 | 圣光闪耀 | 神圣之灵 |
| PRIEST_017 | 库尔提拉斯牧师 | 心灵之火 |
| SHAMAN_016 | 叉状闪电 | 风怒 |
| SHAMAN_017 | 熔岩爆裂 | 风语者 |
| SHAMAN_018 | 雷铸战斧 | 先祖治疗 |
| SHAMAN_019 | 土元素 | 先祖之魂 |

过载现状：闪电箭 / 闪电风暴 / 野性狼魂 / 沙尘暴 / 熔岩爆裂 / 土元素*碰巧*
拿到正确过载量（ID 恰好对上匹配表），叉状闪电拿到 1 而非 2（已登记 §11），
风怒 / 风语者 / 先祖之魂被错误地加上过载，雷铸战斧 / 毁灭之锤 拿不到过载
（已登记 §11）。匹配表的注释也已过期（写的是 Feral Spirit / Forked Lightning /
Lightning Storm / Totem Golem——最后一张根本不在卡池里）。

修复已落地（PR #88）：每对第二张卡重排为真实炉石 ID（法力浮龙 CS2_027、
受祝福的勇士 CS2_089、神圣之灵 CS2_235、心灵之火 CS1_129、风怒 CS2_039、
风语者 CS2_041、先祖治疗 CS2_003、先祖之魂 CS2_289——重复 ID 归零）；
过载匹配表按实际 ID 重接并修正（叉状闪电 2、补雷铸战斧 1 与毁灭之锤 2，
风怒 / 风语者 / 先祖之魂随重排自动脱离）；F5 差分场景 `f8_*` 钉死新行为；
§11 移除 3 张已修复的过载卡（剩 24 张）。RL 侧不引用所动 ID——无需改 RL
卡组配置。全量 `cargo test` 通过（402 个）。

## F-A9 — 先手玩家缺少回合 1 抽牌 ✅ 已解决（官方规则，2026-08-06）

炉石官方规则（暴雪《开局：换牌》）：先手玩家的第 4 张牌在其回合 1 开始时抽入；
后手玩家在自己的第一个回合也正常抽牌。`Step::DrawStep` 曾带回合 1 守卫跳过先手
抽牌（`src/engine/rules.rs`），单元测试 `first_player_does_not_draw_on_turn_one`
钉住的是错误行为；正式开局（`sim::battle::build_game_state`）只给先手发
`hand_size` 张，P1 以 3 张开局而非官方 4 张——放大了后手优势（对照：P1 胜率
约 23%，而简化引擎约 34%——后者本来就抽了回合 1 的牌）。

修复已落地：`build_game_state` 给先手发 `hand_size + 1` 张（开局 + 回合 1 抽牌）；
移除 DrawStep 的回合 1 守卫（回合 1 进入 DrawStep 的状态正常抽牌）；`begin_game`/
换牌流程在开局完成（双方换牌都已解决）后抽第 4 张。测试重钉：
`first_player_draws_on_turn_one`、`second_player_draws_on_first_turn_first_player_from_turn_two`、
换牌完成时的手牌数、env 开局形态测试（默认 4/3，`hand_size 4` → 5/4，带硬币 →
4/5）。对照重测（48 seed）：P1 胜率 23% → 46%（简版 33%），同 seed 一致率 58%。
全量 `cargo test` 通过（405 项）。

## F-A10 — env 限步平局对纯 EndTurn 僵局从不生效 ✅ 已解决（2026-08-06）

引擎当时还没有疲劳（`trigger.rs` — "fatigue in Phase 3+"），抽干双方牌库的对局
永远僵住；env 用 `max_steps`（默认 5000，以平局结束回合）兜底。但限步检查挂在
EndTurn 分支**之后**的 `else if` 链里——僵局状态下每个动作都是 EndTurn，检查
永远不可达，回合无限跑下去（`test_batched_matches_single_per_seed`，seed 3）。
第二个缺陷：结构化观测的 `done` 标志由 `state.step() == GameOver` 推导，而限步
平局把状态机留在 Main——即使 env 正确标记了也不会把 `done` 暴露给 Python。

修复已落地：限步检查移到 EndTurn 分支之前（`rl/env.rs::step`）；
`GameEnv::is_done()` 返回 env 的回合标志而非重新推导；`rl::views::observation`
把 done 标志作为参数（状态本身无法表达限步平局）。由
`step_limit_ends_end_turn_stall_in_draw` 钉住；`GameEnv` 与 `BatchEnv` 的结构化
观测现在都把限步平局报为 done。

**疲劳闭环（2026-08-06，路线图 PR #94）**：底层缺口已闭合——引擎实现官方疲劳
（空牌库抽牌尝试对抽牌英雄造成 1、2、3、……递增伤害；
`docs/finished/fatigue-roadmap.md`）。抽干牌库的对局现在以真实胜者结束；
`max_steps` / `max_turns` 降级为兜底（只有护甲/治疗循环理论上能熬过疲劳）。

## 机制盘点（引擎有 vs 缺）

**已有**（对应卡基本是接线活）：
`summon_trigger` / `spell_trigger` / `death_trigger` / `start|end_turn_effect` /
`aura`（`AuraTarget` 含 OtherFriendlyMinions）/ 费用修正栈（G5）/ `SilenceMinion`+
`AllEnemyMinions` / `FreezeCharacter` / `FullHeal` / `DestroyMinion` 目标（含
`DamagedEnemyMinion`）/ `DealDamageToTwo` / `DestroyAdjacent` / `GrantCharge` /
潜行 / 扰咒 / 奥秘 / `ThisMinionDamaged`（激怒，W0 接线 4 张）/ 剧毒 Poison
组件（W0 接入 Emperor Cobra）/ `EffectTarget::EventSubject` 与
`OtherFriendlyMinion`（W0）/ 武器实体触发注册 + 摧毁后离场（W0）/ 法术结算后
先处理死亡再触发施法触发（W0）/ `CardDef.race` 字段 + 种族条件目标
（`FriendlyRace`/`AllOtherFriendlyRace`/`AnyRace`）+ 种族条件光环
（`GrantCharge` 冲锋光环）+ `Trigger.race` 种族条件触发 + 字段驱动种族池（W1）/
触发类补全：`CharacterHealed`、`Attacked`（实体攻击）、`CardPlayed`、
`SecretPlayed`、`MinionDied`（任意死亡）+ 摧毁奥秘效果（W2）/
手牌区费用光环（`IncreaseMinionCost` / `IncreaseMinionCostFriendly`）+
按武器攻击减费 + 武器耐久削减 + `ChargeWithWeapon` 条件冲锋 + 敌方法术 0 费 +
给对手水晶（W4）/ 生命设为 1（忏悔）、交换攻击/生命（疯狂炼金师）、邻位
增益/冻结（日怒保卫者 / 远古法师 / 冰锥术）、双效果组合（群体驱散、先祖治疗，
W5）。

**缺**（§11 登记的 24 张债按机制分组；全部已由 W8~W12 清偿）：
1. ~~激怒接线（阿曼尼狂战士、暴怒的狼人、格罗玛什——`ThisMinionDamaged` 已存在，
   纯接线活，W0 先例）~~ ✅ W8
2. ~~冲锋光环（战歌指挥官）~~ ✅ W8
3. ~~治疗抽牌触发（北郡牧师）~~ ✅ W8
4. ~~武器攻击特效（真银圣剑回血、血吼减攻、鹰角弓耐久）~~ ✅ W9
5. ~~发现（追踪术）；抉择第二分支（愤怒、利爪德鲁伊、知识古树、战争古树）~~ ✅ W10
6. ~~战吼（奥妮克希亚、阿古斯保护者、死亡之翼弃牌）；受伤冻结（水元素）；
   控制（秘教暗影祭司）；治疗翻倍（先知维伦）；毒刃的 1 伤+抽 1~~
   ✅ W11/W12（W11 顺带修了阿古斯保护者的邻位增益——一处未登记的静默错实现）
7. ~~减费（海巨人、伺机待发）；连击基础分支（冷血）~~ ✅ W11

**2026-08-07 §13 新增机制**：条件激怒（`Enrage` 组件，读取时求值）、
触发的攻击力上限条件（`Trigger.max_attack`，战歌指挥官）、
`TriggerEvent::MinionHealed`（任意随从被治疗，北郡牧师）、
法术伤害管线（`trigger::apply_spell_power`——法术伤害加成 + 先知维伦翻倍，
只作用于法术与英雄技能）。

## 每张卡的 F5 验收

离开账本的每张卡必须带着：
1. `tests/differential.rs` 里的一个场景，钉死真实炉石的结算顺序（目标集合、
   触发时机、死亡阶段交互），以及
2. 语义可镜像的部分，按 `docs/differential_sabberstone.md` 与 SabberStone
   做牌组级一致性对照。

## 维护约定

- **新增卡**：`src/cards/` 里出现新的 `(simplified …)` 注释，就必须在本账本
  加一行（反之亦然）。RL 训练卡池排除本集合（`hearthstone_os/decks.py`），
  所以卡池成员的变动都要走账本。
- **修卡**：实现 → F5 差分测试 → 删账本行 → 删代码注释 → 使
  `~/.cache/orange_stone_debt_ids.txt` 失效（Python 提取器缓存了解析结果）。
- **注释措辞**：Python 提取器按卡牌文档块里是否含 "simplified" 字样识别。
  只有真实债务才写 "simplified"；已忠实就写 "(verified)" 或干脆不写。
- **开放池 ≠ 欠债**：4 张开放池卡（心灵视界、思维窃取、心灵游戏、游学者周卓）的
  注释写 `(pool-open: …)`，**不写** `(simplified: …)` —— 它们是读对手牌的忠实实现，
  不是简化。提取器认的是 "simplified"，所以它们留在 RL 卡池里；注册表在
  `docs/pool-openness-zh.md`。
