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

## F-A11 — 没有手牌上限、没有爆牌 ✅ 已解决（2026-08-07，开放池路线图 M5）

引擎没有 10 张手牌上限：`add_card_to_hand` 和 `draw_card_no_queue` 都是无条件
追加（疲劳路线图明确把该缺口推给账本——`docs/finished/fatigue-roadmap.md`
「手牌上限爆牌……独立的既有缺口」）。开放池卡（游学者周卓、思维窃取、心灵视界）
让手牌溢出从罕见变成常态，而 `rl/obs.rs` 在 `MAX_HAND = 10` 处截断手牌——
第 10 张之后的牌对 agent 不可见却仍可打出，是一处静默的观测/动作错配。

修复：手牌上限 10。**抽到**的第 11 张被销毁（爆牌——进墓地，但仍算「已抽」，
牌库照常消耗）；**产出**的第 11 张根本不创建（`add_card_to_hand` 拒绝）。所有
产牌路径（偷窃、安东尼达斯、周卓、思维窃取、心灵视界、随机池）都走同一对函数，
上限是集中式的。由 `tests/differential.rs` 的 `po_*` 手牌上限场景钉住。

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
  `pool-openness-zh.md`。

### 15. 2025–2026 扩展 M2-W2 — 失落之城任务波（23 张）🔓 已登记

M2-W2 波（`src/cards/exp_tlc_w2.rs`）的简化登记：11 张任务卡（1 费传说
法术，`spell_effect: None`，出牌路径将其送入任务区）加 12 张奖励衍生物。
任务机制（M2-W1）本波扩展：TLC_817 第二进度条（`QuestDef::second` /
`Quest::second`——两条都满才离场）、可重复任务（TLC_426 完成后进度清零、
永久奖励标志常驻）、Temporary 手牌标记（回合结束时弃置，真实制造卡在
W4）、两个玩家级奖励标志（`Player::murloc_summon_buff` 由召唤钩子消费、
`Player::deal_exact_2_bonus` 由伤害钩子消费）。W1 占位奖励已换成真实奖励：
TLC_426 设置永久标志、TLC_513 直接召唤两只忍龟、TLC_817 每条各召唤一尊
索莱托斯。与 §14–§14.5 一致，扩展手写卡均不在 RL 池（经典 + 核心
668/659），本表仅作登记追踪，各行在机制落地前保持开放。

| ID | 卡名 | 简化 | 真实机制 |
| --- | --- | --- | --- |
| TLC_229t14 | 阿沙隆·山脊卫士 | 官方 Adapt 战吼（三选一）固定为全体友方随从 +1/+1 | Adapt / 抉择管线 |
| TLC_433t | 泰拉克斯·骨中恐兽 | 官方"恐怖之墓"地点链条（泰拉克斯变形成坟墓、坟墓再复活他）未建模——亡语直接复活 8/8 复制 | 变形 / 地点亡语链条 |
| TLC_446t1 | 安杜菲尔裂隙 | 官方"激活"步骤（消耗英雄血量激活）未建模——纯白板 0/1 | 激活机制 |
| TLC_460t | 起源之石 | 官方"你发现一张卡后，本武器 +1 耐久"未建模——纯白板 0/8 | 发现回放耐久增益 |
| TLC_513t | 暮光大师 | 官方奖励把英雄替换为暮光大师——未建模；奖励直接召唤两只忍龟（TLC_513t2） | ~~英雄替换~~ ✅ 已清偿（M4-W4，PR #159 —— `ReplaceHero` CardEffect 原语兑现"when real"注记；卡本身保持直接召唤忍龟的形状） |
| TLC_602t | 拉托维乌斯·城市之眼 | 官方任务奖励战吼把 4 张奖励牌加入发现池——未建模（无战吼；奖励池在 W4 落地） | 真实发现管线 |
| TLC_817t5 | 索莱托斯·生命之触 | 官方"同时控制两个索莱托斯形态则合体"步骤未建模——t3 与 t4 相互独立 | 合体 / 双子机制 |
| TLC_830t | 肖克·丛林暴君 | 官方战吼的攻击力过滤发现池未建模（无战吼） | 真实发现管线 |

本波新增原语一览：任务第二进度条（双条计数、双条完成才离场、已完成条
在另一条进行时封顶并忽略后续事件）、可重复任务（完成后进度与标记清零、
卡牌留在任务区）、Temporary 手牌标记（回合结束弃置，`World` 新增
sparse-set 存储与访问器）、`murloc_summon_buff`（MinionSummoned 钩子在
种族进度循环之后消费，友方鱼人召唤即带 +1/+1 永久附魔）、
`deal_exact_2_bonus`（DamageDealt 钩子在 Goldrinn 翻倍之后消费，恰好 2 点
敌方伤害再加 2）。简化与既往一致：发现 → 无战吼（TLC_602t/830t）；
Adapt 三选一 → 固定全体 +1/+1（TLC_229t14）；泰拉克斯的"恐怖之墓"地点
链条 → 亡语直接复活 8/8 复制（TLC_433t）；安杜菲尔裂隙的激活步骤 →
纯白板 0/1（TLC_446t1）；起源之石"发现后 +1 耐久" → 纯白板 0/8
（TLC_460t）；英雄替换 → 直接召唤奖励随从（TLC_513t）；TLC_817t5 的
"双形态合体"步骤未建模（t3/t4 相互独立）。

F5 覆盖：`tlc_w2_spirit_of_the_mountain_reward_summoned`、
`tlc_w2_restore_the_wild_everbloom_buffs_after_hero_attack`、
`tlc_w2_golakka_depths_repeatable_murloc_buff`、
`tlc_w2_reanimate_the_terror_tyrax_deathrattle`、
`tlc_w2_escape_the_underfel_temporary_discard`、
`tlc_w2_forbidden_sequence_origin_stone_equipped`、
`tlc_w2_lie_in_wait_master_dusk_ninjas`、
`tlc_w2_enter_the_lost_city_survive_turns`、
`tlc_w2_unleash_the_colossus_bonus_damage`、
`tlc_w2_reach_equilibrium_double_bar`、
`tlc_w2_food_chain_shokk_battlecry`、
`tlc_w2_one_quest_per_player_real_cards`（tests/differential.rs 共 12 个
场景——每张任务卡至少一个，外加 Temporary 原语、两个玩家标志与真实卡的
单人一任务规则）。全量 `cargo test`：433 项中 432 通过——唯一红项是
`tlc_w1_spell_school_lookup_and_progress`：它的两条断言（tests/differential.rs
约 23074/23083 行：四次圣光施放后任务离场、且无衍生物被召唤）钉住了 W1
占位语义，与 W2 真实双条 TLC_817 相矛盾（任务留在任务区、第二条待完成、
并召唤两尊 TLC_817t3）——按波次规范，最小化的两条断言更新待执行。
`cargo clippy --all-targets` 无警告。

### 16. 2025–2026 扩展 M2-W3 — 失落之城 Kindred 波（23 张）🔓 已登记

M2-W3 波（`src/cards/exp_tlc_w3.rs`）的简化登记：23 张 Kindred 卡。机制
本身忠实："Kindred: X——当你本回合早些时候打过一张同类型卡时触发 X"
（随从按种族、法术按 SPELL；卡牌本身永远不算）。激活状态是每玩家
`kindred_played` 列表，每次出牌推入、玩家自己的回合结束时清空；四个解析
点全部落在出牌路径上：费用折扣（TLC_366/600/816，push 之前检查，只数
更早的同型卡 ≥1）、OnPlay（TLC_107/226/243/428/429/440/447/519/815/825/
903，基础结算之后）、战吼修饰（TLC_454/463/482/829——替换型走专用
变体、TLC_482 附加型在战吼之后结算）、抽牌修饰（折进专用抽牌战吼：
TLC_223 的火系过滤走 W1 真实法术学派注册表、TLC_236 精确 1/2/3/4 费
扫描、TLC_432 亡语随从 ≤3 费）。Torga 的牌库扫描忠实：自上而下扫实际
牌库找第一张 Kindred 注册卡，再找其同类型卡；无匹配则不抽——经典|核心
牌库下空抽就是官方行为。与 §14–§15 一致，扩展手写卡均不在 RL 池（经典
+ 核心 668/659），本表仅作登记追踪，各行在机制落地前保持开放。

| ID | 卡名 | 简化 | 真实机制 |
| --- | --- | --- | --- |
| TLC_102/223/243/432/463/482 | 六张双种族 Kindred 卡 | 官方 "races" 第二种族经 `apply_card_keywords` 落到实体上（恐魔先例）、参与部落协同，但 Kindred 计数与 `kindred_type` 注册表只认主种族——双种族卡只以单一部落激活 Kindred，官方语义任一种族都算 | 多种族 Kindred 计数 |
| TLC_815 | 墓生虚空球 | 随机"4 费嘲讽随从"池走 D2 简化——ALL_CARDS 中 4 费随从、排除衍生物；被召随从自身战吼照常结算（尼鲁布虫群卫兵级联是忠实的） | 官方召唤池 |
| TLC_519 | 伏击掠食者 | 固定召唤 TLC_519t 毒液喷吐者（潜行 + 剧毒）——单元素 D2 池，无随机选择 | 随机喷吐者池 |
| TLC_482 | 熔渣之爪 | Kindred 附加结算触发全部友方余烬火花的亡语——包括本回合早些时候从手牌打出的那张——官方"触发它们的亡语"仅指战吼召出的两只 | 按被召火花限定的触发范围 |
| TLC_251 | 起源鳍挑战者 | "你的下一个 Kindred 触发两次"标志只被 OnPlay Kindred 结算消费——费用折扣与战吼修饰不翻倍 | 通用触发双次标志 |
| TLC_107 | 风暴酿造者 | "每当本随从攻击，先对目标造成 3 点伤害"是卡 ID 硬编码的 ResolveAttack 钩子（湖鲟先例），而非通用攻击触发组件 | 通用攻击触发组件 |
| TLC_428 | 热泉滑翔者 | "下一个鱼人"标志为玩家级标志（`next_murloc_discount` 出牌费用时消费、`next_murloc_divine_shield` 出牌时消费），由下一次鱼人出牌消费——滑翔者本人永不消费自己的标志（折扣在战吼设置前已算、圣盾在入场前已加） | 逐实体修饰 |
| TLC_825 | 迅猛龙母兽 | Kindred 目标（造成等同于本随从攻击力的伤害给一个敌方随从）经 PlayCard 目标传入，但 `rl/env.rs` play_targets 只提取战吼目标——RL 视图无法表达 | 浮出面的 Kindred 目标 |

本波新增原语一览：`kindred_played`（每玩家出牌类型列表，出牌路径在
Preparation 消耗块内推入——随从推主种族、法术推 Spell；回合结束清空，
符合"本回合"语义；列表只在出牌路径增长，效果召出的复制品不会再触发，
Conjured Bookkeeper 复制不成环）、`next_kindred_twice`（TLC_251 战吼
设置、`resolve_on_play` 开头读取并清除、效果结算两次）、
`next_murloc_discount` / `next_murloc_divine_shield`（TLC_428 战吼设置
折扣、Kindred 设置圣盾；下一次鱼人出牌时消费）。新 CardEffect 变体 14 个：
GainRush、GainImmuneThisTurn、NextMurlocCostsLess、GiveNextMurlocDivineShield、
SetNextKindredTwice、DrawKindredAndActivator、DrawSpellGiveSpellDamage、
DrawMinionsOfEachCost、DrawDeathrattleMinionCostLE、DestroyLowestAttackEnemy、
TriggerFriendlyCinderDeathrattles、DestroyMinionAndGainItsStats、
DealSelfAttackDamage、SummonRandomMinionCostTaunt——全部与 bincode
CardEffectDe 镜像、序列化往返测试清单、`src/sim/bot.rs` 计分分支同步。
顺带修复一个既有缺口：`resolve_destroy_minion` 此前没有
`EffectTarget::AnyMinion` 分支（静默空操作）——补上后 Ravenous Devilsaur
的基础战吼与 Kindred 变体才真正生效。Stormbrewer 的攻击钩子在 ResolveAttack
处理器内把 3 点伤害压进队列、先于攻击伤害结算（湖鲟先例）；免疫本回合
随回合结束清除（Bestial Wrath 先例）。

F5 覆盖：`tlc_w3_kindred_requires_same_type_earlier_this_turn`、
`tlc_w3_spell_kindred_fires_after_any_spell`、
`tlc_w3_cost_discount_kindred`、`tlc_w3_next_kindred_triggers_twice`、
`tlc_w3_stormbrewer_gains_rush`、`tlc_w3_stormdrake_immune_this_turn`、
`tlc_w3_torga_draws_kindred_and_activator`、
`tlc_w3_kodo_high_attack_replaces_lowest`、
`tlc_w3_razidir_opponent_hand_discard`、
`tlc_w3_slagclaw_triggers_cinder_deathrattles`、
`tlc_w3_devilsaur_gains_destroyed_stats`、`tlc_w3_cryosleep_draws_another`、
`tlc_w3_matriarch_attack_damage`、`tlc_w3_queen_hero_attack`、
`tlc_w3_thrasher_drawn_spell_damage`、`tlc_w3_dread_raptor_costs_zero`、
`tlc_w3_kindred_resets_next_turn`、`tlc_w3_voidbulb_summons_taunt_4cost`、
`tlc_w3_ambush_predators_spitter`、`tlc_w3_hot_spring_glider_murloc_flags`、
`tlc_w3_hybridization_draws_each_cost`、
`tlc_w3_bookkeeper_copy_does_not_loop`（tests/differential.rs 共 22 个
场景——覆盖全部 Kindred 形态：激活判定、费用折扣、双次标志、关键词增益、
Torga 扫描、战吼替换/附加、抽牌修饰、回合重置、随机池与不循环复制）加
`src/cards/kindred.rs` 三个注册表单元测试。全量 `cargo test` 全绿（所有
套件、含全部 `tlc_w1_*`/`tlc_w2_*` 场景——817 通过、1 忽略），`cargo fmt`
干净，`cargo clippy --all-targets` 零警告。

### 17. 2025–2026 扩展 M2-W4a — 失落之城主系列波、第一批（96 张）🔓 已登记

M2-W4a 波（`src/cards/exp_tlc_w4a.rs`）的简化登记：96 张非传说
"失落之城"收藏卡（规则"TLC_* 收藏卡、稀有度非传说、减去 W1–W3 已落地的
22 张"算得 97，但 TLC_249 灼热余烬群 W3 已落地，实际新写 96 张——逐卡
清单见模块头部注释），外加 17 张手写衍生物（11 张来自卡牌数据转储、
6 张转储中缺失、按所服务的卡牌文本手写）与风暴之门支线任务。本波机制
核心大多忠实：
`discovered_this_turn` 每玩家标志（每次发现结算置位、回合结束清除）
驱动储能间斗殴费用 0 与出土文物召唤 4 费随从；地图链是
`map_pending: Option<(Entity, Vec<String>)>`，由发现卡本回合的出牌消费
一次——随机补一张其他选项入手；被诅咒的地下墓穴与血瓣生态园给发现卡
打 Temporary 标记（W2 原语）、洞穴探索者的一次性"下一张临时卡减 2 费"
标志在出牌费用管线里消费；任务出牌标志（风暴之门除外）解锁求知助手。
与 §14–§16 一致，扩展手写卡均不在 RL 池（经典 + 核心 668/659），本表
仅作登记追踪，各行在机制落地前保持开放。

| ID | 卡名 | 简化 | 真实机制 |
| --- | --- | --- | --- |
| TLC_514/434/334/461/449 | 传奇商人 / 古法占卜 / 圣物之戒 / 废品拾荒者 / 血瓣生态园 | D2 发现池：传奇随从、亡灵随从、费用 ≥8 法术、费用等于剩余法力、临时 1 费随从——都是经典\|核心窗口过滤的 ALL_CARDS 抽样，官方是按稀有度加权的三选一、池子为完整赛制池 | 官方发现流程 |
| TLC_435/442/464/824/900 | 地穴地图 / 沉没地图 / 群山地图 / 异象地图 / 蜂巢地图 | 五张扩展发现池：霜符是固定四卡表（W1–W3 霜符卡）、鱼人/高攻野兽/邪能法术是窗口内过滤子集、未打类型随从以本局打过卡牌的主种族为准——同属 D2 抽样 | 官方各池发现 |
| TLC_109 | 矿工遗骸 | 同稀有度发现走静态稀有度表（窗口内收藏卡 + 扩展收藏卡）；被毁卡不在表内（如衍生物）时发现直接落空，无回退 | 官方稀有度池 |
| TLC_435/442/464/515/824/900 | 六张地图卡 | "若你本回合打出发现的卡，发现另一张"简化为随机补一张其他选项入手——`map_pending` 入口由发现卡出牌消费一次，多一张卡、无二次三选一 | 官方再发现链 |
| TLC_518 | 审讯 | 三张巨龟忍者作为普通卡洗入牌库——官方 Summoned-When-Drawn 关键词未实现（翡翠传送门先例）；忍者只在抽到并打出后入场 | Summoned-When-Drawn 管线 |
| TLC_467 | 低语之石 | 获得的邪能法术"以生命值支付法力"经 CostHealth 标记由出牌路径读取——只走一次性出牌支付，无其他血费交互 | 官方血费资源 |
| TLC_444/465 | 加尔瓦顿的故事 / 缠藤者 | 随机增益池是六关键词引擎池（嘲讽/圣盾/剧毒/风怒/扰魔/潜行）——D2 无权重随机 | 官方增益池 |
| TLC_436 | 复生翼手龙 | "以尸体为费用"：尸体计数是简化玩家级资源（友方亡灵死亡 +1、出牌扣 5）——无尸体类型或其他尸体交互 | 官方尸体系统 |
| TLC_252/t | 溶解软泥怪 / 骨骼 | 两张骨骼牌把被毁随从的攻击力复制到骨骼实体上（法术伤害等值）；官方 Health 那一半省去、且骨骼是手里普通 0 费法术 | 官方骨骼（攻击 + 生命） |
| TLC_EVENT_400 | 风暴之门 | 奖励为随机野兽减 3 费——无自定义野兽合成；支线任务本身忠实（"打出 3 个野兽或亡灵"、1 费任务槽、不计入任务出牌标志） | 官方合成野兽 |
| TLC_439 | 潮汐之波 | "敌方随从下回合费用 +2"是施法者侧标志、敌方出牌费用时读取、施法者下一个回合开始时清除——官方窗口（敌方随后那一回合）恰好完整覆盖 | 官方下一回合窗口 |
| TLC_253 | 石化食人魔 | 休眠建模为不能攻击；官方 50% 苏醒掷骰退化为每回合开始确定性 +2/+2（无苏醒翻转） | 官方休眠翻转 |
| TLC_827 | 放牧剑龙 | "回合结束时给手中随机野兽 +1/+1，没有则改牌库随机野兽"简化为回合结束自身 +1 攻击 | 官方先手牌后牌库增益 |
| TLC_221 | 灼热余烬群 | "对一个随从造成 3 点伤害"改为对随机敌方角色造成 3 点（无目标选择） | 官方定向伤害 |
| TLC_483 | 宝库破毁者 | "发现一张卡后其费用 -1"折进发现的入手时刻——发现时一次性降价（持久附魔会活过破毁者离场） | 官方发现后附魔 |

中文小结（同上）：M2-W4a 波（"失落之城"主系列第一批，96 张非传说卡 +
17 张手写衍生物）的新原语大部分忠实：`discovered_this_turn` 每玩家标志
（每次发现结算置位、回合结束清除）驱动储能间斗殴费用 0 与出土文物召唤
4 费随从；地图链是 `map_pending: Option<(Entity, Vec<String>)>`，由发现
卡本回合的出牌消费一次——随机补一张其他选项入手（六张地图卡共用一个
形状）；被诅咒的地下墓穴与血瓣生态园给发现卡打 Temporary 标记（W2
原语）、洞穴探索者的下一次临时卡减 2 费标志在出牌费用管线里消费；
quest_played 标志（除风暴之门外所有任务卡出牌时置位）解锁求知助手。
已登记简化：D2 发现池（十个 DiscoverPool 变体都是经典|核心窗口内过滤的
ALL_CARDS 抽样或固定表；矿工遗骸的稀有度发现走静态稀有度表、表外卡
（如衍生物）被毁则发现直接落空）；六张地图卡"本回合打出则再次发现"
简化为出牌时随机补一张其他选项（无二次三选一）；审讯把三张忍者普通
洗入牌库（官方 Summoned-When-Drawn 关键词未实现，绿翼先例）；低语之石
的血费标记（CostHealth）只走出牌支付分支；加尔瓦顿故事/缠藤者的随机
增益池是六关键词引擎池（嘲讽/圣盾/剧毒/风怒/扰魔/潜行，D2 无权重）；
复生翼手龙尸体机制简化为玩家级尸体计数（友方亡灵死亡 +1、出牌扣 5）；
溶解软泥怪的两张骨骼牌把被毁随从的攻击力复制到骨骼实体上（官方
Attack + Health 两半只做了一半、且是手里普通 0 费法术）；风暴之门奖励
为随机野兽减 3 费（无自定义野兽合成）；潮汐之波"下回合敌方随从费 +2"
是施法者侧标志、敌方出牌费用时读取、施法者下一个回合开始时清除（官方
窗口正好覆盖敌方随后那一回合）；石化食人魔的休眠建模为不能攻击、
官方 50% 苏醒骰退化为每回合开始确定性 +2/+2；放牧剑龙"回合结束时给
手中随机野兽 +1/+1、没有则改牌库"简化为回合结束自身 +1 攻击；余烬蜂群
"对一个随从造成 3 点伤害"改为对随机敌方角色；宝库破毁者的"发现后减 1
费"折进发现入手时刻（一次性降价，持久附魔会活过破毁者离场）。

F5 覆盖：`tlc_w4a_bloodpetal_biome_grants_temporary`、
`tlc_w4a_tunnel_terror_temporary_tokens`、
`tlc_w4a_spelunker_discounts_next_temporary`、
`tlc_w4a_cursed_catacombs_marks_deck_card_temporary`、
`tlc_w4a_storage_scuffle_free_after_discover`、
`tlc_w4a_unearthed_artifacts_escalates_after_discover`、
`tlc_w4a_vault_breaker_discounts_discovered`、
`tlc_w4a_map_card_picks_second`、
`tlc_w4a_questing_assistant_fires_after_quest_played`、
`tlc_w4a_cloud_serpent_copies_hand_elemental`、
`tlc_w4a_curious_explorer_reduces_enemy_hand_cost`、
`tlc_w4a_platysaur_discards_drawn_card_on_death`、
`tlc_w4a_carnassa_shuffles_raptors`、
`tlc_w4a_interrogation_shuffles_ninjas`、
`tlc_w4a_skyscreamer_eggs_summon_four`、
`tlc_w4a_relic_miner_destroys_top_draws_rarity`（tests/differential.rs
共 16 个场景——覆盖本波全部新原语：Temporary 生成器（地点激活发现 /
被诅咒的地下墓穴牌库选取 / 隧道恐魔亡语）与 W2 回合结束弃置生命周期、
洞穴探索者一次性减费、"本回合已发现"标志驱动储能间斗殴/出土文物/宝库
破毁者、地图链（置入、一次性消费、随机补一张）、任务出牌标志、云蛇
手牌复制、敌方手牌减费、甲龙兽的联动弃牌、两种洗入牌库形状与矿工遗骸
的稀有度发现）。全量 `cargo test` 全绿（所有套件、含全部
`tlc_w1_*`/`tlc_w2_*`/`tlc_w3_*` 场景——833 通过、1 忽略），`cargo fmt`
干净，`cargo clippy --all-targets` 零警告。

### 18. 2025–2026 扩展 M2-W4b — 失落之城主系列波、第二批（14 张传说卡 + 1 张衍生物）🔓 已登记

M2-W4b 波（`src/cards/exp_tlc_w4b.rs`）的简化登记：14 张失落之城传说卡
（TLC_100/106/110/228/241/257/452/480/522/624/810/811/836/841）外加
TLC_241t"召唤潮汐舰队！"衍生物。本波机制大多忠实：恩布拉（Endbringer
Umbra）战吼再触发本局已死的至多 5 个友方随从的亡语（友方墓地即"本局
已死"日志）；克罗格（Krog）回合结束把所有敌方随从置为 1/1（写基础值
+ 清伤害 + 剥附魔）；奥普（Opu）的战吼/连击/亡语各对全部敌方随从造成
1 点伤害；纳布利亚（Nablya）为每个受伤友方随从召唤全新基础身材的
复制并给复制突袭（复制剥离战吼/连击组件——本引擎 MinionSummoned
处理器会对效果召唤的随从触发战吼，剥离才能保持复制品全新）；阿凯欧斯
（Archaios）经新增 FriendlyMinionAttacked 触发事件把攻击者的生命值
设为自身的有效生命值（Attacked 事件钉在攻击者身上不可用）；妮莉
（Niri）的打出卡触发对 1 费随从翻倍属性、对 1 费法术双重施放（单个
CardPlayed 触发器、效果按主题卡类型分支——Trigger 组件是单槽）；洛
（Loh）战吼使全场随从费用恒为 5；伊多（Ido）的回合开始效果在存活时
授予 TLC_241t 衍生物（真实 2 费神圣法术，+2/+2 并给予圣盾）。与
§14–§17 一致，扩展手写卡均不在 RL 池（经典 + 核心 668/659），本表仅作
登记追踪，各行在机制落地前保持开放。

| ID | 卡名 | 简化 | 真实机制 |
| --- | --- | --- | --- |
| TLC_100 | 领航者伊莉丝 | 战吼的定制地点铸造退化为对 `Player::starting_deck` 快照的"10 张不同费用"检查并置 `elise_location_crafted` 标志——无定制地点实体、激活或文本 | 官方定制地点 |
| TLC_110 | 城主艾肖 | "任意位置 +2/+2"采用 Grimestreet 形状：手牌/牌库随从改基础数值、场上随从永久附魔；牌库检查本身忠实（当前牌库、空牌库空洞通过） | 官方任意位置增益 |
| TLC_228 | 燃石·布拉玛 | "你的元素造成额外 1 点伤害"是 Goldrinn 同入口的伤害管线钩子（光环近似）：布拉玛玩家所有带元素种族的伤害源 +1，TLC_228 存活时生效——法术来源无种族、自然排除 | 真正的光环 |
| TLC_452 | 泰坦绘师奥斯科 | 泰坦机制未实现——战吼为空（冒烟测试钉住身材） | 官方泰坦技能 |
| TLC_810 | 高阶祭司赫伦 | "他们互殴！"简化为一轮交换：两个召唤复制各按攻击力互打一次，走正常伤害管线（死亡/亡语/圣盾正常结算）；牌库本身不动（`resolve_summon` 复制） | 官方互殴结算 |
| TLC_841 | 昆虫学家托鲁 | 0/1 罐子变形/释放机制未实现——战吼为空（冒烟测试钉住身材） | 官方罐子 |

F5 覆盖：`tlc_w4b_umbra_triggers_five_dead_deathrattles`、
`tlc_w4b_krog_sets_enemy_minions_to_one`、
`tlc_w4b_opo_fan_of_knives_via_battlecry_combo_deathrattle`、
`tlc_w4b_nablya_copies_damaged_minions_with_rush`、
`tlc_w4b_archaios_sets_attacker_health`、
`tlc_w4b_niri_doubles_one_cost_minions`、
`tlc_w4b_niri_casts_one_cost_spells_twice`、
`tlc_w4b_loh_minions_cost_five`、
`tlc_w4b_ido_grants_token_while_alive`、
`tlc_w4b_esho_deck_check_buffs_minions`、
`tlc_w4b_bralma_elementals_deal_extra_damage`、
`tlc_w4b_herenn_summons_two_deathrattle_minions_and_fight`、
`tlc_w4b_elise_checks_starting_deck`、
`tlc_w4b_osk_toru_smoke_pins`（tests/differential.rs 共 14 个场景——
每张传说卡一个：墓地扫描亡语再触发、回合结束 1/1 置位并清伤害/剥附魔、
战吼/连击/亡语三合一扇刀、剥离战吼的全新突袭复制、新增
FriendlyMinionAttacked 触发与 subject==source 守卫、CardPlayed 费用 1
分支（随从翻倍与法术双重施放）、随从费用恒 5 置位、存活才发衍生物及
衍生物 +2/+2 与圣盾、当前牌库种族检查与任意位置增益、元素伤害钩子、
两个亡语随从互殴且牌库不动、开局牌库费用检查、两个简化冒烟钉）。
全量 `cargo test` 全绿（所有套件、含全部 `tlc_w1_*`/`tlc_w2_*`/
`tlc_w3_*`/`tlc_w4a_*` 场景——847 通过、1 忽略），`cargo fmt` 干净，
`cargo clippy --all-targets` 零警告。

### 19. 2025–2026 扩展 M2-W4c — 失落之城迷你系列"魔鬼龙嘉年华"（38 张卡 + 3 张衍生物）🔓 已登记

M2-W4c 波（`src/cards/exp_tlc_w4c.rs`）的简化登记：38 张失落之城迷你系列
卡（DINO_130/131/132/136/137/138/400/401/402/403/404/405/406/407/408/409/
410/411/412/413/414/415/416/417/419/421/422/424/426/427/428/429/430/431/432/
433/434/435）外加 3 张衍生物（DINO_130t 小龙长颈、DINO_136t 暴食迅猛龙、
DINO_410t 基洛）。本波机制大多忠实：四张亲和卡扩展 W3 表（迪亚波罗王
撕裂敌方最左/最右随从、火鳃给其他随从突袭、寒脊剑龙伤害+冻结的
BattlecryModifier、火山实验召唤自身复制）；五张面具引入 SetStats 家族
（蝙蝠面具填满 1/1 复制、恶魔龙面具给冲锋、巨兽面具强制随机敌方随从走
正常攻击管线打它、绵羊面具附加真实"对所有随从造成 2 点伤害"亡语、黑豹
面具给潜行并抽 2）；孵化仪式"你下个回合结束时"是每人两跳计数器（施放
置 2、每个己方回合结束减 1、到 0 生效）；安魂典礼的回合结束死亡是标记
实体清扫 + 真实 DamageDealt 事件（Corruption 模式，亡语正常触发）；路障
拳手护甲钩子接在 grant_armor 咽喉点（全部 16 处护甲获得都走它）；大
德雷克龙用新增 AttackedEnemyMinion 触发事件（以防御者为主题，溅射排除
被攻击的随从）；震地龙复用 W4b 的"任意位置"增益；害羞盘碟商的相邻
手牌减费读取记录的 last_played_hand_index。与 §14–§18 相同，这些手写
扩展卡不在 RL 池（经典 + 核心 668/659），各行为信息性登记：让代码的
简化可追溯回账本。每行保持开启直到对应机制落地。

| ID | 卡牌 | 简化 | 何时为真 |
| --- | --- | --- | --- |
| DINO_407 | 晶透米雷克斯 | "在手牌中时，这是你对手最后打出的随从的 3/4 复制"退化为手牌与场上均为静态 3/4——无手牌动态复制机制 | 官方手牌变形 |
| DINO_409 | 科技龙 | "本局每打出过一张非起始卡牌的牌则费用 -1"未实现（无卡牌来源追踪）——固定 7 费无折扣 | 官方来源追踪 |
| DINO_410 | 基洛之卵 | 裂壳链跳过：亡语直接召唤最终 20/20 嘲讽野兽（DINO_410t 基洛）——无蛋壳衍生物、无 5 次碎裂倒计时 | 官方裂壳链 |
| DINO_414 | 贡舞 | 二选一变形退化为随机变形：随机窗口内随从变形为随机窗口内随从（TransformToMinion 置数值语义——攻击/生命/费用/卡牌 id 重写、攻击次数清零、静态关键字） | 官方二选一 |
| DINO_430 | 野兽语者塔卡 | 战吼/亡语的"发现关联"丢弃：战吼获得随机传奇野兽的身材、亡语随机召唤传奇野兽——两次独立选取，无存储卡牌 | 官方存储卡牌关联 |
| DINO_435 | 火山实验 | ALL 种族亲和检查按主种族约定近似为野兽（全种族卡按 dump 首种族计） | 全种族亲和检查 |
| DINO_131 | 被附身的亡灵巫师 | 召唤的野兽是复制品——牌库不动（赫伦 §18 复制召唤约定） | 牌库拉取 |
| DINO_427 | 化装商人 | "随机获得一张其他职业的面具"是固定 5 面具池的 D2 随机（蝙蝠/恶魔龙/巨兽/绵羊/黑豹——皆中立，无职业过滤损失） | 全卡随机 |
| DINO_434 | 巢穴护士 | 1 费随机随从 / 1 费随机法术为窗口内一费池的 D2 抽取 | 官方无发现随机 |
| DINO_412 | 图腾 | "随机获得一张拥有多种随从类型的随从"是窗口内多重种族随从固定池的 D2 抽取（CORE_TTN_866 神话恐惧——唯一一张，选取确定） | 全多重种族过滤 |
| DINO_415 | 暮光之影故事 | "发现一张 5 费以上亡语随从并召唤、触发其亡语"是窗口内费用 ≥5 亡语随从的 D2 随机（召唤后亡语立即触发） | 官方发现 |
| DINO_424 | 英雄欢迎仪式 | 传说发现是窗口内传说随从的 D2 随机（经典 LEGENDARY_CLASSIC 成员表），置为 10/10 | 官方发现 |
| DINO_426 | 生命仪式 | 3 费发现是窗口内 3 费随从的 D2 随机，召唤为 2/3 复制 | 官方发现 |
| DINO_431 | 腕龙 | "随机 5 费以上嘲讽随从"是窗口内嘲讽随从的 D2 随机 | 全卡随机 |
| DINO_433 | 值日哨岗 | 6/4/2 费嘲讽三连是三次独立 D2 随机召唤 | 三次全卡随机 |

中文小结（同上）：本波大多忠实：五张面具的 SetStats 家族（置数值 +
清伤害 + 剥附魔，蝙蝠面具用目标卡牌 id 填满 7 个 1/1、巨兽面具经
AttackDeclared/ResolveAttack 管线强制攻击并吃反击）、孵化仪式两跳倒计时
（施放回合结束不生效、己方下个回合结束才 +2/+2）、安魂典礼回合结束
清扫（随从 +1 攻击 + 突袭、回合结束以真实伤害事件死亡、亡语正常触发）、
路障拳手护甲咽喉点（+2/+2 永久附魔 + 随机敌方随从立即攻击）、大德雷克龙
溅射排除被攻击者（5/12 攻击 4/4 后其余敌方全灭）、震地龙"任意位置"
+3/+3（手牌与牌库写基础数值、场上不生效）、盘碟商相邻减费（边缘只减
一个邻居、中间两个都减、读取 last_played_hand_index）、盛宴号角连击
判定（最左/最右出牌三只迅猛龙免疫、中间出牌不免疫）、基洛卵亡语直接
出 20/20、灵魂安息+复苏的亡语随从各走真实管线。

F5 覆盖：`tlc_w4c_diabolus_rex_hits_edge_minions`、
`tlc_w4c_firegill_gives_others_rush`、
`tlc_w4c_chillspine_freezes`、
`tlc_w4c_crater_experiment_copies_self`、
`tlc_w4c_bat_mask_sets_and_fills`、
`tlc_w4c_devilsaur_mask_sets_charge`、
`tlc_w4c_behemoth_mask_forces_attack`、
`tlc_w4c_sheep_mask_deathrattle`、
`tlc_w4c_panther_mask_sets_stealth_draws`、
`tlc_w4c_dracorex_aoe_on_attack`、
`tlc_w4c_hatching_ceremony_next_turn_buff`、
`tlc_w4c_hollow_direhorn_corpse_reborn`、
`tlc_w4c_soulrest_turn_end_dies`、
`tlc_w4c_seismopod_wherever_buff`、
`tlc_w4c_saucier_reduces_adjacent`、
`tlc_w4c_horn_of_feasting_outcast`、
`tlc_w4c_basher_armor_trigger`、
`tlc_w4c_tortotem_multi_tribe_pool`（tests/differential.rs 共 18 个场景——
每张亲和卡带激活/未激活双分支、五张面具各一、德雷克龙溅射排除被攻击
随从、孵化两跳倒计时只在己方下个回合结束生效、尸体花费且尸体在触发后
落地、安魂回合结束清扫带亡语触发、任意位置增益触及手牌与牌库、相邻
减费的边缘与中间出牌、连击判定下的免疫仅当回合、护甲触发 +2/+2 与
强制攻击、确定性的多重种族池）。全量 `cargo test` 全绿（所有套件、含
全部 `tlc_w1_*`/`tlc_w2_*`/`tlc_w3_*`/`tlc_w4a_*`/`tlc_w4b_*` 场景——
865 通过、1 忽略），`cargo fmt` 干净，`cargo clippy --all-targets`
零警告。
### 20. 2025–2026 扩展 M3-W2a — "穿越时光"子路线 W2 第一拆（120 张卡 + 11 张衍生物）🔓 已登记

M3-W2a 波（`src/cards/exp_tmw_w2a.rs`）的简化登记：118 张非传说 TIME_*
可收集卡（53 普通 / 38 稀有 / 27 史诗）外加本波实现的 2 张传说卡
（TIME_038 克洛克先生、TIME_063 时间领主诺兹多姆——其效果属于 W2a 形状），
共 120 张，外加它们产生的 11 张衍生物（TIME_006t1 镜像法师、TIME_017t
坦克、TIME_025t 时光碎片、TIME_059t 活悖论、TIME_434t 时空暗影、
TIME_443t 萨格拉尔地狱犬、TIME_610t2 异常暗影、TIME_700t 时序龙、
TIME_704t 高等精灵学徒、TIME_870t 斗兽场猛虎、TIME_873t 斗兽场迅猛龙）。
**M3-W2a 规范中的"114"计数是错的**——`cards/cards.json` 的生成基线里有
118 张非传说 TIME_* 卡；该出入已在 sets.rs 与 exp_tmw_w2a.rs 文件头披露。
与 §14–§19 相同，这些手写扩展卡不在 RL 池（经典 + 核心 668/659），各行为
信息性登记：让代码的简化可追溯回账本。每行保持开启直到对应机制落地。

本波核心机制均为完整原语：**休眠**（新 `core::component::Dormant { turns }`
组件——休眠中不能攻击、不能被指定（validate_attack 的潜行式过滤）、
不受到伤害（DamageDealt 分支提前返回）；每个己方回合开始倒计时、归零
移除组件（苏醒）——卡牌：机械元老（3 回合）、时间领主诺兹多姆（5 回合）、
小翼亡语召唤的随机 2 费随从（2 回合）、守望者囚禁的敌方随从（10,000
回合）；永恒巨蟒在任意随从休眠时减 4 费，走费用管线）；**重放**（W1
机制——17 张重放卡经出牌路径自动回放已记录效果）；**时光碎片**（下表）；
**开放池注册表**（5 张）；**地点**（过去侏儒城/过去汇聚点/过去银月城——
Core Set W8 表示法，激活效果放战吼槽；其 card_type|health|durability 走
expansion_differential_rebalanced，因为生成器早于地点 CardType）。

| ID | 卡牌 | 简化 | 何时为真 |
| --- | --- | --- | --- |
| TIME_025t | 时光碎片 | "抽到时施放"按既有先例（翡翠传送门，§14）简化：可打出的 0 费法术，对你英雄造成 3 点伤害——按普通卡抽入 | 官方抽到时施放 |
| TIME_028 | 命运破坏者 | "从牌库施放时光碎片"是牌库扫描+移除，内联应用碎片自身效果后再 +3/+3（吸血是真实的——自伤被吸回） | 官方从牌库施放时序 |
| TIME_029 | 毁灭迅龙 | 同样的从牌库施放形状，随后召唤自身复制（复制去除战吼） | 官方从牌库施放时序 |
| TIME_030 | 分歧 | "把一张手牌随从分裂成两半"退化为普通复制——分裂卡两半机制不存在 | 官方分裂卡两半 |
| TIME_036 | 皇家线人 | 开放池："获得复制或费用 +2"二选一为随机选取（既有开放池约定） | 官方选择 |
| TIME_039 | 似曾相识 | 开放池发现：对手手牌卡 D2 随机选取（复制入手） | 官方发现 |
| TIME_041 | 未来先知 | 开放池：猜测恒为猜中——直接 +4 生命（无猜测机制） | 官方猜测 |
| TIME_432 | 交织命运 | 开放池：一次合并选择（先三张牌库 id、再三张对手手牌 id）；另一侧随机复制（D2） | 官方双重发现 |
| TIME_876 | 塑形者 | 开放池：读取对手手牌作为变形目标 | 官方目标读取 |
| TIME_063 | 时间领主诺兹多姆 | "你打出新扩展卡后提前 1 回合苏醒"——新扩展检查在封闭卡池无意义；恒在第 5 回合苏醒 | 扩展来源检查 |
| TIME_101 | 错位火法师 | "每当你粉碎一张卡牌"——池内无粉碎机制；触发为无操作 | 粉碎机制 |
| TIME_214 | 波动亡魂 | "每当你将对这张卡造成自然法术伤害，改为 +2/+1"——无"将要受到伤害"拦截钩子；白板 1/4 嘲讽 | 官方拦截 |
| TIME_217 | 风暴鸦 | "每当你将对这张卡造成自然法术伤害，改为召唤随机 5 费随从"——同样未建模的拦截；白板 5/5 | 官方拦截 |
| TIME_021 | 末日预言者 | 连击位置退化为普通战吼（连击位置条件未建模）；免疫到期取当前回合结束（已登记的卡多雷先例） | 连击位置检查 + 到你下个回合结束 |
| TIME_002 | 奥术师维兹南 | "随机职业法术"——职业以职业卡并集近似（RandomPool::ClassSpell） | 按职业过滤 |
| TIME_014 | 多宇宙化身 | "召唤 12 法力值随机随从"——每次选取至多花剩余法力，保证终止 | 官方池遍历 |
| TIME_016 | 霓虹革新 | "过去的一张圣骑士机械"——"过去"即活动窗口；+5/+5 挂在选取时消费的暂存修饰上（D2） | 官方全卡随机 |
| TIME_027 | 超光速齐射 | 分裂伤害是 1 点一发的弹片，永不随法术伤害加成（匹配官方 ImmuneToSpellpower） | 官方带法伤的分裂 |
| TIME_033 | 复苏德鲁伊 | "施放 2 个随机自然法术"——每次选取以随机目标解析法术自身效果 | 官方无目标施放 |
| TIME_038 | 克洛克先生 | 随机传说池 = 活动窗口 + TIME_063（封闭卡池中仅有的另一张传说） | 完整传说池 |
| TIME_043 | PMM 无限者 | "本回合不能攻击英雄"的临时限制是真实的（W1 机制）；指向友方随从的可选选取为 D2 随机 | 官方玩家选择 |
| TIME_044/436/810 | 过去侏儒城/过去汇聚点/过去银月城 | 地点激活的指向性选取为 D2 随机 | 官方玩家选择 |
| TIME_054 | 时间领航员 | "每个玩家的回合结束时"走 per-ID EndTriggers 钩子（Trigger 组件无法看到双方场面） | 双场景触发 |
| TIME_057 | 睿智求真者 | 完整：费用附魔剥离、TurnCostReducer 标记移除；攻击/生命附魔保留 | — |
| TIME_212 | 避雷针 | "对一个友方随从造成 2 点伤害"——友方指向性选取为 D2 随机 | 官方玩家选择 |
| TIME_213 | 原始监督者 | "持有此卡时施放过自然法术"——"持有此卡时"以全场自然法术计数近似 | 手牌来源追踪 |
| TIME_218 | 静电冲击 | "对一个随从造成 1 点伤害"——指向性选取为 D2 随机 | 官方玩家选择 |
| TIME_431 | 琥珀女祭司 | "恢复生命值"——指向性选取为 D2 随机 | 官方玩家选择 |
| TIME_442 | 时光守望者 | "囚禁/唤醒"联动真实但走玩家记录上的（守望者，囚禁目标）实体对——休眠本身是完整原语 | 官方实体关联 |
| TIME_447 | 屏障圣言 | "使一个角色获得圣盾"——指向性选取为 D2 随机 | 官方玩家选择 |
| TIME_448 | 孤独 | "发现 2"退化为一次选取（D2 随机随从） | 官方发现 2 |
| TIME_613 | 冷冻冠军 | 随机传说池 = 活动窗口 + TIME_063 | 完整传说池 |
| TIME_614 | 生命渲染者 | "你的英雄本回合生命值发生变化"以每回合英雄受伤计数器近似 | 完整变化追踪 |
| TIME_615 | 被遗忘的千年 | "改为消耗生命值而非法力"——"本回合"以 CostHealth 标记近似 | 完整回合结束到期 |
| TIME_620 | 不合时宜的死亡 | 奥秘的"打出后下一回合"标记也会在当回合死亡时触发 | 官方回合后过滤 |
| TIME_700 | 时序光环 | "持续 3 回合"——计数器挂在玩家记录上（每个己方回合结束时为正则召唤） | 持续光环 |
| TIME_702 | 潮起潮落 | "持有此卡时打出过随从"——"持有此卡时"以本回合打出任意随从近似 | 手牌来源追踪 |
| TIME_704/704t | 高等精灵导师/学徒 | 教学关联近似：发现的法术像普通发现一样入手；学徒"施放所教法术"战吼未建模 | 官方教学法术关联 |
| TIME_707 | 替代现实 | "过去的随机抉择卡"——池为活动窗口 | 完整过去池 |
| TIME_730 | 卡多雷栽培者 | "发现 2 张野兽"为 D2 随机选取（置入牌库底并 +5/+5） | 官方发现 |
| TIME_770 | 快进 | "选择一张费用 -2"——选取为 D2 随机 | 官方玩家选择 |
| TIME_856 | 奥格萨导师 | 法术伤害 +1——官方 JSON 数据为 +1 而卡面文字写 +2；引擎从数据 | 官方文字 |
| TIME_857 | 改变时间 | "发现两张奥术法术"——第二张为一次选取简化 | 官方发现 2 |
| TIME_860 | 无面谜团 | "选一张"施放的奥秘为 D2 随机 | 官方玩家选择 |

F5 覆盖：`tmw2a_rewind_replays_previous_effect`、
`tmw2a_clocksworth_legendary_pool`、`tmw2a_dormant_sleeps_then_awakens`、
`tmw2a_perennial_serpent_discount`、`tmw2a_flutterwing_dormant_summon`、
`tmw2a_hopper_shuffles_shreds`、`tmw2a_fatebreaker_casts_shred_from_deck`、
`tmw2a_informant_copies_or_raises`、`tmw2a_deja_vu_discovers_enemy_hand_copy`、
`tmw2a_truthseeker_resets_costs`、`tmw2a_causality_reverses_deck`、
`tmw2a_divine_augur_sets_hand_stats`、`tmw2a_rafaam_ladder_draws_distinct_costs`、
`tmw2a_velocity_discounts_per_damage`、`tmw2a_unknown_voyager_transforms`、
`tmw2a_circadiamancer_reduces_each_turn`（`tests/differential.rs` 中 16 个
场景——重放回放形状、D2 传说池、3 回合休眠倒计时与苏醒移除组件及睡眠期
不能攻击/不能指定、任意休眠减费、召唤即休眠的亡语、两张洗入牌库的碎片
以可打出的 0 费法术抽入、命运破坏者的牌库扫描+移除及其真实吸血自疗
（净零）、开放池二选一与对手手牌发现、双方手牌费用重置、整副牌库反转、
置数值神谕者、跳过重复的不同费用抽牌、每伤害减费计数、存活受伤变形、
每回合 TurnCostReducer）。本波还修复了一个既存的 `CardEffectDe` bincode
镜像错位：33 变体段（DestroyAndGainStats..BuffAnotherRandomFriendlyDragon，
索引 81-113）停在 De 索引 194-226，而 113 变体段
（CopyRandomEnemyHandCard..AddRandomOutcastCardNextCheaper）占据 81-193，
导致该区间每个变体静默反序列化为不同效果；镜像已重排对齐，并新增结构
守卫测试（`src/core/effect.rs` 的 `card_effect_de_mirror_order_matches`）
钉住声明顺序。全部 `cargo test` 全绿（所有套件，含全部
`tlc_w1_*`/`tlc_w2_*`/`tlc_w3_*`/`tlc_w4a_*`/`tlc_w4b_*`/`tmw1_*`
场景与 16 个 `tmw2a_*`——888 通过、1 忽略），`cargo fmt` 干净，
`cargo clippy --all-targets` 零警告。

### 21. 2025–2026 扩展 M3-W2b — "穿越时光"子路线 W2 第二拆（25 张传说卡 + 12 张衍生物）🔓 已登记

M3-W2b 波的已登记简化（`src/cards/exp_tmw_w2b.rs`）：25 张"穿越时光"传说卡
（TIME_038 克洛克先生与 TIME_063 时间领主诺兹多姆已在 W2a 落地、PR #152，
不在此列）及其产生的 12 张衍生物（TIME_042t 无限香蕉、TIME_209t 至高王
之锤、TIME_211t1 永恒之井、TIME_211t2 辛艾萨莉、TIME_609t1 游侠队长
奥蕾莉亚、TIME_609t2 游侠新兵温蕾萨、TIME_619t 邦桑迪、TIME_713t 永世
宝箱、TIME_850t 布罗尔·血战士、TIME_850t1 瓦莉拉·血战士、TIME_875t
李亚王、TIME_890t2 卡拉赞圣所——TIME_890t 大法师麦迪文的守护者·埃提耶什
跳过，见 TIME_890 行）。与 §14–§20 相同，这些手写扩展卡不在 RL 池
（经典 + 核心 668/659）中，各行为信息性登记：让代码里的简化对账本可追溯；
每一行在其机制落地前保持未清偿。

本波核心机制均为完整原语：**打出牌日志读取**（`Player::played_minion_ids`
全场随从打出日志 + 重放历史 `last_played`）：TIME_609 游侠将军希尔瓦娜斯
按已打出的姐姐（奥蕾莉亚/温蕾萨）数量向所有敌人重复 2 点伤害；TIME_103
克罗米亡语把最近 `count`（10，重放历史上限）张不同的已打出牌各复制一张
入手；**双重触发光环**（TIME_064 时间领主迪奥斯——`AuraEffect::DoubleTriggers`，
可沉默）：六个规则钩子各自把预捕获效果再解析恰好一次——随从战吼、武器
战吼、亡语、英雄技能、抉择分支、回合结束触发（与"战吼两次"暗黑天赋
叠加时共 3 次而非 4 次）；**地点**（CardType::Location，Core Set W8
表示法：TIME_446 永恒堡垒、TIME_211t1 永恒之井、TIME_211t2 辛艾萨莉、
TIME_890t2 卡拉赞圣所——激活效果放战吼槽、打出回合冷却生效；TIME_446
走 expansion_differential_rebalanced，因为生成器早于地点类型）；**每回合
打出快照**（TIME_714 时间纪元领主只摧毁对手上一回合打出的随从——CardPlayed
钩子维护 minions_played_this_turn_ids，TurnEnded 处理快照进
last_turn_minion_play_ids）；**INFINITY**（TIME_024 穆罗佐德——32 位引擎
没有无限值；打出效果在玩家记录上武装 murozond_infinite_pending，己方下
回合开始钩子把攻击力设为共享常量 INFINITY_ATTACK_CAP（100，W3 复用））；
**手牌换回**（TIME_706 鳍人——Player::hand_swap_snapshot）；**尸体花费**
（TIME_618 胡斯克——Player::corpses）；**托奇追踪**（TIME_861——
Player::toki_pending_spells）；**永恒堡垒恶魔减费标记**
（Player::next_demon_cost_one）。

| ID | 卡牌 | 简化 | 何时为真 |
| --- | --- | --- | --- |
| TIME_005 | 窃时者拉法姆 | Fabled+——"牌库 40 张但有 10 张拉法姆！战吼：若其余均已打出则摧毁敌方英雄"——纯 10/10（无 Fabled 牌库构造、无全打出检查） | Fabled+ 牌库 + 全打出检查 |
| TIME_009 | 未来工匠格尔宾 | "从牌库各召唤一种灵气"简化为一张随机牌库随从——召唤后移除战吼组件（TIME_870 先例），牌库实体消费 | 官方灵气复制链 |
| TIME_013 | 先知沃 | 完整：友方施放法术触发从活动窗口自然法术池发现三选一（发现由引擎自动随机解析——既定约定） | — |
| TIME_020 | 布洛希加 | Fabled——"游戏开始：消失。击杀全部 4 个阿古斯恶魔后重现"——纯 12/12 冲锋 | 官方消失/重现 |
| TIME_024 | 无限穆罗佐德 | "将此随从攻击力设为 INFINITY"——32 位引擎没有无限值；攻击力设为 `INFINITY_ATTACK_CAP`（100） | 无限值 |
| TIME_032 | 时序高龙 | 完整：拥有者抽牌库最高费用的 2 张，对手抽剩余牌库最低费用的 2 张（真实牌库实体；对手所抽卡牌改属入对手手牌） | — |
| TIME_042 | 马鲁克王 | 完整弃牌 + 获得无限香蕉；香蕉的"无限"关键词（"此牌留在手牌"）未建模——纯 1 费法术 | 无限关键词 |
| TIME_064 | 不可阻挡的时间领主迪奥斯 | 完整：双重触发光环使战吼/亡语/英雄技能/回合结束效果各再解析一次；"战吼两次"叠加时共 3 次而非 4 次 | — |
| TIME_103 | 克罗米 | "抽取本局你打出过的牌的复制"受重放历史上限约束（最近 `count` = 10 张不同的已打出牌） | 无界对局历史 |
| TIME_209 | 至高王穆拉丁 | 锤子亡语"将此牌洗入牌库并获得永久 +2 攻击"简化为入手（无 +2 增益、无洗牌库）；装备的锤子本身完整（3/4 风怒） | 官方洗回 + 强化 |
| TIME_211 | 阿莎拉女士 | 完整抉择（选项 0 强化辛艾萨莉、选项 1 永恒之井；另一地点被替换——官方"另一张被摧毁"）；复制友方随从为 D2 随机 | 官方玩家选择 |
| TIME_435 | 永恒者 | 完整阈值（结算时来源的有效生命值）；目标选取为 D2 随机 | 官方玩家选择 |
| TIME_446 | 永恒堡垒 | 地点（card_type\|health\|durability 重平衡——生成器早于地点类型）；恶魔发现由引擎自动解析（D2）；"牌库无随从时下一张减至 1 费"标记真实 | 官方发现 |
| TIME_609 | 游侠将军希尔瓦娜斯 | "若你打出过奥蕾莉亚或温蕾萨则重复"——全场打出日志把效果召唤的复制也算入（现有最佳近似） | 官方打出计数 |
| TIME_618 | 永世收割者胡斯克 | "使你的英雄获得'亡语：消耗至多 20 具尸体，以该生命值复活'"——英雄亡语复活未建模；战吼立即消耗至多 20 具尸体为英雄恢复生命 | 官方复活 |
| TIME_619 | 墓穴之邦桑迪 | 完整抽/复活邦桑迪；祝福抉择直接授予嘲讽/吸血/突袭——官方"+2/+2 与亡语随从费用提高"附带条件简化掉 | 官方祝福附带 |
| TIME_705 | 永世守护者克罗娜 | 完整：牌库底部 5 张费用设为 1（Set(1) 费用修饰），顶部牌不动 | — |
| TIME_706 | 超越时间的鳍人 | "以你的起始手牌替换手牌"——引擎无起手记录；新牌取牌库顶（快照与回合末换回真实） | 起手记录 |
| TIME_713 | 时间海军上将胡克舵 | 完整：为对手召唤 0/8 宝箱；宝箱亡语向"控制者的对手"手牌填满硬币（官方文字——硬币落入胡克舵一方） | — |
| TIME_714 | 时间纪元领主 | 完整：每回合打出快照只摧毁对手上一回合打出的随从（亡语照常触发） | — |
| TIME_850 | 血战士罗戈什 | 完整亡语：手牌血战士 +5/+5 召唤并对 D2 随机敌方攻击；布罗尔/瓦莉拉衍生物亡语未实现（链条止于罗戈什——避免递归） | 官方衍生物链 |
| TIME_852 | 蔚蓝女王辛达苟萨 | 完整：场上另有龙时奥术法术费用 -2（费用管线） | — |
| TIME_861 | 时光重塑者托奇 | 完整：追踪 3 张随机法术；全部打出后入手一张全新托奇（池为 RandomPool::Spell） | — |
| TIME_875 | 加罗娜·半兽人 | 开放池 🔓（已在 POOL_OPEN_CARDS 登记——读取对手手牌）：摧毁手中的李亚王并把敌方最大生命值减半（向上取整；现有伤害保留） | 官方开局隐藏（TIME_875t 为纯 3/3/3） |
| TIME_890 | 圣光麦迪文 | 完整战吼（双方场上其他随从全体沉默 + 摧毁）；"控制卡拉赞时费用为 0"真实（play_cost）；TIME_890t 埃提耶什跳过——卡拉赞"装备埃提耶什时费用为 0"的附带条件无左侧 | 埃提耶什 |
| TIME_609t1 | 游侠队长奥蕾莉亚 | 发现为 D2 随机法术（无选项）；"按姐姐数量重复"并入希尔瓦娜斯的打出日志 | 官方发现 + 重复 |
| TIME_609t2 | 游侠新兵温蕾萨 | "使牌库随从 +1/+1"未建模——纯 2/4（仍计入希尔瓦娜斯的打出日志） | 官方牌库增益 |
| TIME_619t | 邦桑迪 | 祝福关键词（嘲讽/吸血/突袭）直接授予；官方亡语"召唤随机 4 费随从"未建模 | 官方亡语 |
| TIME_042t | 无限香蕉 | "使一个随从 +1/+1（此牌留在手牌）"——无限关键词未建模；纯 1 费法术（指向选取为 D2 随机） | 无限关键词 |
| TIME_209t | 至高王之战锤 | 见 TIME_209——亡语为纯入手 | 官方洗回 + 强化 |
| TIME_875t | 李亚王 | "开局隐藏于敌方牌库。战吼：抽一张牌。洗回此牌"——纯 3/3/3，可被加罗娜的检查摧毁 | 官方隐藏 |
| TIME_850t / TIME_850t1 | 布罗尔 / 瓦莉拉·血战士 | 衍生物亡语未实现（链条止于罗戈什）；嘲讽/扰魔关键词真实 | 官方衍生物链 |
| TIME_890t2 | 卡拉赞圣所 | 激活（两个随机 8 费随从）真实；埃提耶什费用附带跳过（TIME_890t 未实现） | 埃提耶什 |

中文小结（同上）：见上"本波核心机制均为完整原语"段。与 §14–§20 相同，
手写扩展卡不在 RL 池（经典 + 核心 668/659），各行为信息性登记，每一行在
其机制落地前保持未清偿。

F5 覆盖：`tmw2b_sylvanas_repeats_per_alleria`、
`tmw2b_sindragosa_arcane_discount`、`tmw2b_garona_halves_enemy_health`、
`tmw2b_chronogor_highest_lowest_draws`、`tmw2b_krona_bottom_costs_one`、
`tmw2b_epoch_destroys_last_turn_minions`、`tmw2b_toki_pool_and_reget`、
`tmw2b_hooktail_chest_for_opponent`、`tmw2b_muradin_hammer`、
`tmw2b_azshara_choose_one`、`tmw2b_deios_doubling`、
`tmw2b_medivh_silence_destroy_all`、`tmw2b_murozond_infinite_attack`、
`tmw2b_eternus_control`、`tmw2b_chromie_copies_played_cards`、
`tmw2b_simplified_legendary_smoke_pins`（`tests/differential.rs` 中 16 个
场景——按姐姐数量重复、有无第二龙的奥术减费、摧毁李亚王与最大生命减半
且伤害保留、最高/最低抽取与牌库中段不动、底部 5 张设为 1 费而顶部保留、
每回合快照只摧毁上回合随从而其余随从存活、三张追踪法术与入手全新托奇、
对手控制的宝箱填满胡克舵一方手牌、风怒锤子及其亡语回归、两个阿莎拉分支
与地点冷却和临时填充、迪奥斯翻倍及其带光环死亡、双方场沉默且无亡语抽牌、
跨两个回合周期的 INFINITY 上限、生命阈值夺取、已打出牌的复制，以及简化
传说卡的冒烟钉：拉法姆/布洛希加/格尔宾（无战吼召唤）/胡斯克（尸体
花费）/鳍人（换回））。全部 `cargo test` 全绿（所有套件，含全部
`tlc_w*_*`/`tmw1_*`/`tmw2a_*` 场景与 16 个 `tmw2b_*`——908 通过、
1 忽略），`cargo fmt` 干净，`cargo clippy --all-targets` 零警告。

### 22. 2025–2026 扩展 M3-W3 — "时间的终结"迷你系列，收尾波（38 张卡 + 3 张衍生物）🔓 已登记

M3-W3 收尾波的已登记简化（`src/cards/exp_tmw_w3.rs`）：本迷你系列的 38 张
END_ 卡及其产生的 3 张衍生物（END_002t 匕首、END_009t 树人、END_017t 滴答
与嗒啦——END_029t / END_026t 在本波效果形状中无效果，跳过）。与
§14–§21 相同，这些手写扩展卡不在 RL 池（经典 + 核心 668/659）中，各行为
信息性登记：让代码里的简化对账本可追溯；每一行在其机制落地前保持未清偿。

本波复用既有原语——灌注机制（END_000 / END_001 / END_003，含死亡骑士被动）、
同类表（END_015）、黑暗馈赠施加（END_013 / END_027）、真实抉择（END_010）、
尸体资源（END_005）、任务注册表（END_017）、重放历史（END_036）、奥秘机
器（END_024）以及 D2 随机选取（END_013 的发现、END_014 的增益目标、
END_018 的手牌牌、END_020 的召唤、END_027 的龙、END_034 的三次摧毁、
END_037 的龙）——并新增一种触发形状：**敌方回合结束奥秘**
（`SecretTrigger::WhenEnemyTurnEnds`，接入 TurnEnded 处理器，END_024）。

| ID | 卡名 | 简化 | 落地时 |
| --- | --- | --- | --- |
| END_002 | Wicked Blightspawn | 完整亡语：无武器时装备引擎本地的 END_002t 匕首衍生物（1/2——引擎未定义潜行者匕首牌），否则已装备武器 +2 攻击 | 潜行者匕首牌 |
| END_003 | Finality | 完整：抽一张亡灵 + 灌注两次。灌注后的死亡骑士英雄技能被动（"你每回合打出的第一张亡灵 +X 攻击"）为英雄钉扎的 CardPlayed-亡灵触发器，每次灌注重新挂载（`BuffFirstUndeadPlayedEachTurn` 携带增长后的计数） | — |
| END_004 | Remnant of Rage | 完整："本回合每死亡一个随从便减 1 费"——双方玩家的本回合死亡列表相加（友方死亡在己方列表、敌方死亡在对方列表，双方均于各自回合结束清空） | — |
| END_006 | Chronikar | "本回合、下回合、下下回合"近似为三个直到回合结束的 +3 攻击增益——战吼施加当前一个并挂 `chronikar_ticks = 2`；接下来两个回合开始各消耗一个 tick 重新施加 | 官方三回合附魔 |
| END_011 | Acceleration Aura | "接下来 3 个回合"近似为：己方回合开始各获得一个临时法力水晶，持续 3 回合（`acceleration_aura_ticks` 倒计时） | 官方多回合临时水晶 |
| END_012 | Hand of Infinity | "将此武器的攻击力设为 INFINITY"使用共享 `INFINITY_ATTACK_CAP`（100，W2b 先例）；"无法攻击英雄"为按键 ID 的挥击校验（不是 `cant_attack` 组件——英雄仍可攻击随从） | 无界数值 |
| END_017 | Battle at the End Time | 完整：填满再清空的任务序列（`QuestCondition::FillThenEmptyHand`，标记 0 已填满 / 1 已清空——清空半在 CardPlayed 结束时、填满半在新的抽牌/入手钩子于手牌达 MAX_HAND_SIZE 时触发）。注：`add_card_to_hand` 的生成牌路径在临时 EventQueue 上解析钩子，任务完成事件在该稀有路径被丢弃 | — |
| END_018 | Acolyte of Infinity | "将一张随机手牌的费用设为 INFINITY"使用 Cost(100) 附魔（`INFINITY_ATTACK_CAP`）；亡语经链接实体约定恢复记录的牌 | 无界数值 |
| END_022 | Time-Twisted Seer | "受伤时法术伤害 +2"：CardDef 携带 `spell_damage: 2`；`world::total_spell_damage` 在随从未受伤时跳过该加成（按键 ID 跳过；基线对比行不含 `spell_damage`） | — |
| END_024 | Flames of Infinity | 完整：新的敌方回合结束奥秘；"造成 INFINITE 伤害"为单次 `INFINITY_ATTACK_CAP`（100）命中——上限量级的命中足以杀死任何现实随从（平手取场上顺序靠前的随从） | 无界数值 |
| END_025 | Eternal Firebolt | 完整：吸血（关键词）+ 回手为 `eternal_flame_target` 玩家记录——目标死亡时，所有者回合结束加入一张全新 END_025 复制 | — |
| END_030 | Haywire Hornswog | 完整："本局每过载一个法力水晶便减 1 费"——游戏全程 `overload_total` 玩家计数器在过载锁定点累加；费用层读取它 | — |
| END_036 | Morchie | "重放保留 BOTH 结果"光环为单元标记光环（`AuraEffect::RewindKeepsBothOutcomes`，`AuraTarget::AllFriendlyMinions`），使 `engine::rewind` 把每个重放的随机效果解析两次（随机效果变体封闭列表）；战吼发现为对 `REWIND_CARD_IDS` 的 D2 随机选取 | 官方结果追踪 |
| END_037 | Endtime Murozond | 完整：D2 随机龙填满战场并完全治疗英雄；跳过回合为 `skip_next_turn` 玩家标记——下一个 TurnStarted 清除它并立即执行正常回合结束序列（被跳过的玩家不补法力、不抽牌、不进入主阶段；结束序列期间该玩家被设为活跃，使 wrap-up 把行动权交给对手） | — |
| END_008 | Enduring Roach | 完整："使用英雄技能后"触发器挂 HeroPowerUsed 事件；刷新为当前法力上限 10 的补足 | — |
| END_009t | Treant | 完整：2/2 衍生物随游戏全程友方树人死亡计数器缩放（`treants_died_total`，MinionDied 时累加） | — |
| END_017t | Tick and Tock | 完整：8/8 奖励——战吼抽牌直到手牌满、亡语清空对手手牌 | — |
| END_002t | Dagger | 1/2 武器衍生物为引擎本地（见 END_002） | 潜行者匕首牌 |
| END_029t / END_026t | Shade / Fragment 衍生物 | 未实现——在本波效果形状中无效果（2026-08-09 核实） | 官方衍生物 |

中文小结（同上）：见上"本波核心机制均为完整原语"段。与 §14–§21 相同，
手写扩展卡不在 RL 池（经典 + 核心 668/659），各行为信息性登记，每一行在
其机制落地前保持未清偿。

F5 覆盖：`tmw3_*`（`tests/differential.rs` 中 22 个场景）——灌注三件套
（潜行者 + 跨两次灌注的死亡骑士被动）；任务填满再清空与滴答和嗒啦奖励及
空牌库不抽牌；同类授予与（2）减费和两次黑暗馈赠发现；无限之手的 INFINITY
攻击与英雄锁定；无限之仆的 INFINITY 费用设定 + 恢复；敌方回合结束奥秘的
INFINITY 命中；抉择数值两套（含受伤随从死亡）；克罗尼卡三回合增益；
甲虫刷新 + 加速光环倒计时；树人缩放；永恒苦役抽/召分支；苦涩终局的冻结 +
摧毁；万世永存牌的过载 + 霍恩斯沃格减费；终焉预兆的空牌库摧毁；穆罗宗德
跨三次回合结束的跳过；莫尔奇的双结果重放；有翼异变的连击/过载关键词；
匕首装备 + 武器增益 + 时之爪弃牌；幸存者的本回合受伤战吼 + 手牌武器/随从
增益 + 持龙减费；占卜师的受伤法术伤害 + 虚无碎片的随从指向抽牌 + 巫毒
图腾的回合结束授予；以及余烬残渣 / 往昔回声 / 永恒火枪 / 崩碎碾压者批次。
全部 `cargo test` 全绿（所有套件，含全部 `tlc_w*_*`/`tmw1_*`/`tmw2a_*`/
`tmw2b_*` 场景与 22 个 `tmw3_*`——930 通过、1 忽略），`cargo fmt` 干净，
`cargo clippy --all-targets` 零警告。

### 23. 2025–2026 扩展 M4-W1 — 大灾变 Colossal 波（11 张卡 + 31 张衍生物）🔓 已登记

M4-W1 波的已登记简化（`src/cards/exp_cata_w1.rs` + `src/cards/colossal.rs`）：
大灾变扩展的 11 张 CATA_ Colossal 卡及其 31 张身体部件衍生物（4× 维克
方之腿、2× 拉格纳罗斯之手、2× 艾萨拉之触手、2× 艾拉基尔充能之手、2× 辛奈
丝特拉之翼、2× 奥妮克希亚之翼、3× 黑色之血之躯、4× 克洛马图斯之头、2× 弗
尔卡诺斯之羽、6× 玛格玛之躯、2× 乔加尔/加尔之臂）。与 §14–§22 相同，这些
手写扩展卡不在 RL 池（经典 + 核心 668/659）中，各行为信息性登记：让代码
里的简化对账本可追溯；每一行在其机制落地前保持未清偿。

本波的核心机制为完整原语：**Colossal +N**——`ColossalPart{main}` /
`ColossalMain{parts}` 组件（SparseSet 存储）+ 按 ID 键控的 `colossal_parts`
注册表（CardDef 不加任何新字段）。从手牌打出时，部件在主随从的战吼结算
**之后**进入主随从右侧紧邻的位置、按序排列（打出路径的 MinionSummoned
钩子，仅限从手牌打出——效果召唤不携带部件）；7 随从上限生效（满场不召唤；
玛格玛的 +99 为 `fill` 注册项，钳制为填满全部空位）。部件自身的死亡是独
立随从死亡（主随从存活）。主随从死亡时把死亡级联到部件：它们进入**同一**
死亡批次（亡语照常触发、按场上顺序），且每个将死部件在级联时基础生命归零，
使 MinionDied 的补血重检通过——炉石中 Colossal 主随从的死亡无条件带走部件，
批次中途治疗无法挽救。部件自身的"召唤时"效果挂在 `appendage_on_summon`
注册表而非召唤触发器上（友方随从召唤触发器的触发排除被召唤者自身）。

| ID | 卡名 | 简化 | 落地时 |
| --- | --- | --- | --- |
| CATA_139 | Wickerfang | 腿的回合结束 +1/+1 增益复制到主随从（`GainStatsAndCopyToColossalMain`——增益与复制一起结算）；官方"当维克方之腿获得属性后，本随从也获得同样属性"——本波中腿只有自身的回合结束增益，复制点位等价；外部来源给腿的属性增益不会复制 | 官方向任意增益来源的范围 |
| CATA_150 | 拉格纳罗斯，天火 | 完整："在你的回合结束时，触发你的随从的亡语"——按场上顺序触发每个友方场上随从的亡语恰好一次，以该随从为来源（来源自身也被包含——官方文本就是"你的随从"） | — |
| CATA_151 | 艾萨拉，海洋之主 | 完整："你的英雄获得风怒"光环（`AuraEffect::GrantWindfury`，新增 `AuraTarget::FriendlyHero`——可沉默）；触手的召唤时 +2 攻击（见下） | — |
| CATA_153 | 艾拉基尔，风暴之王 | 完整：突袭（关键词钩子）+ 风怒 + "获取 2 张费用等于本随从攻击力的随从。它们费用为（1）"——以来源有效攻击力（2）为费用过滤的窗口内随机选取，费用设为 (1) 的成本修改器（路线图 G5 先例） | — |
| CATA_154 | 辛奈丝特拉 | "你的其他职业法术施放两次"——**职业过滤被去掉**（引擎没有每玩家/每卡职业概念）：玩家施放的**每个**法术都再结算一次，采用泰兰德约定（无显式目标、不发第二个 SpellCast 事件） | 官方职业过滤 |
| CATA_155 | 复生的奥妮克希亚 | "在你的回合中，当你的英雄将要失去生命值时，改为获得等量最大生命值"——伤害重定向在**护甲/武器吸收之前**触发，因此本可被护甲吸收的伤害也会转为最大生命值（官方护甲优先顺序未建模）；增益为永久最大生命值附魔且从不实际造成伤害——不触发受伤触发器、不消耗护甲 | 官方护甲优先顺序 |
| CATA_300 | 黑色之血 | "在你为一个角色恢复生命值后，攻击一个随机敌方随从"——触发器挂在**全局** CharacterHealed 事件类上，因此**任意一方**的任何治疗都会触发（官方仅限己方"你恢复"的范围为近似）；"攻击"为过量伤害直伤模型——以来源攻击力为伤害、溢出伤害带往敌方英雄（非真实攻击动作：无反击、无攻击触发器） | 官方己方范围 + 真实攻击 |
| CATA_432 | 克洛马图斯 | 完整：嘲讽 + 吸血 + 不可指向 + 圣盾；四个头的亡语从主随从移除对应关键词（见下） | — |
| CATA_488 | 弗尔卡诺斯 | 完整："在你的回合结束时，对所有其他随从造成 2 点伤害"（来源自身排除）；羽毛的受伤触发器（见下） | — |
| CATA_550 | 玛格玛 | 完整：Colossal +99 "当有空位时召唤所有剩余部件"——`fill` 注册项把召唤数钳制到空位数；最大空位为 6（主随从占一格），恰等于部件数，故 +99 恒等于填满战场 | — |
| CATA_726 | 乔加尔，主谋 | "你的手臂和士兵改为摧毁敌方牌库中的随从"——友方乔加尔在场上时，每只手臂的摧毁改为命中敌方牌库中一张随机随从（移入墓地）；无乔加尔时手臂摧毁其右侧相邻随从；士兵（官方文本与手臂搭配的两张 1/1）是 W2 的问题，不属本波 | 官方士兵联动 |
| CATA_139t / _t2 / _t3 / _t4 | 维克方之腿 | 完整："在你的回合结束时，获得 +1/+1"并复制到主随从（见 CATA_139） | — |
| CATA_150t / _t1 | 拉格纳罗斯之手 | {0} 定为 2："亡语：对随机敌人造成 2 点伤害" | 传令官升级 |
| CATA_151t / _t1 | 艾萨拉之触手 | {0} 定为 2："召唤时，使你的英雄获得 +2 攻击力（本回合）"（`GainHeroAttack` 本就是直到回合结束） | 传令官升级 |
| CATA_153t / _t1 | 艾拉基尔充能之手 | {0} 定为 1："相邻随从 +1 攻击力"光环（GrantAttack，AdjacentMinions） | 传令官升级 |
| CATA_154t / _t1 | 辛奈丝特拉之翼 | "召唤时，获取一张其他职业的随机法术。它费用减少（{0}）"——{0} 定为 0：减费**去掉**（法术照常入手） | 传令官升级 + 减费 |
| CATA_155t / _t1 | 奥妮克希亚之翼 | "召唤时，获取一张随机的 {0} 费随从。本回合它消耗生命值"——{0} 定为 0；CostHealth 标记挂在入手的牌上（"本回合"范围未清除） | 传令官升级 + 回合范围 |
| CATA_300t1 / _t2 / _t3 | 黑色之血之躯 | 完整：回合结束"为一个随机受伤友方角色恢复 3 点生命值" | — |
| CATA_432t1 | 克洛马图斯之绿头 | 完整：嘲讽；"亡语：从克洛马图斯移除嘲讽" | — |
| CATA_432t2 | 克洛马图斯之红头 | 完整：吸血（吸血关键词钩子）；"亡语：从克洛马图斯移除吸血" | — |
| CATA_432t3 | 克洛马图斯之蓝头 | 完整：不可指向；"亡语：从克洛马图斯移除不可指向" | — |
| CATA_432t4 | 克洛马图斯之铜头 | 完整：圣盾；"亡语：从克洛马图斯移除圣盾" | — |
| CATA_488t / _t2 | 弗尔卡诺斯之羽 | 完整：本随从受到伤害触发器获取一张随机火焰法术并减费（3）（quest.rs 的法术学派表） | — |
| CATA_550t / _t2 / _t3 / _t4 / _t5 / _t6 | 玛格玛之躯 | 完整："亡语：使一个随机友方随从获得 +2 攻击力"——附魔层增益（`GiveRandomFriendlyMinionAttack`；基础数值设定不与光环组合，恶魔仆从先例） | — |
| CATA_726t / _t1 | 乔加尔之臂 / 加尔之臂 | {0} 定为 2："在你的回合结束时，摧毁右侧随从以获得 +2/+2"——最右手臂右侧无随从时不摧毁；乔加尔牌库摧毁（见 CATA_726） | 传令官升级 |

F5 覆盖：`cata_w1_*`（`tests/differential.rs` 中 9 个场景）——
`cata_w1_colossal_summons_parts_adjacent`（部件紧邻主随从、双向链接、按
序）；`cata_w1_colossal_part_positions_ordered`（填充随从 + 维克方的布局，
部件从左到右）；`cata_w1_colossal_part_cap`（近满场只召唤一条腿、满场拒
绝打出）；`cata_w1_magmaw_fills_board`（+99 填满全部空位）；
`cata_w1_main_death_kills_parts`（主随从死亡 → 手随之亡、亡语照常触发）；
`cata_w1_part_death_keeps_main`（部件死亡 → 主随从与其余部件存活）；
`cata_w1_wickerfang_legs_gain_stats_copied`（四条腿的回合结束增益复制到
主随从）；`cata_w1_chromatus_head_removes_keyword`（绿头亡语移除嘲讽、其
余关键词保留）；`cata_w1_colossal_dies_via_aoes`（AOE 单批击杀主随从 + 部
件、每个死亡恰好处理一次）。全部 `cargo test` 全绿（所有套件，含全部
`tlc_w*_*`/`tmw1_*`/`tmw2a_*`/`tmw2b_*`/`tmw3_*` 场景与 9 个 `cata_w1_*`
——938 通过、1 忽略），`cargo fmt` 干净，`cargo clippy --all-targets`
零警告。

### 24. 2025–2026 扩展 M4-W2 —— 大灾变传令官波（13 张卡 + 6 个士兵衍生物 + 1 个 Breezling）🔓 登记

M4-W2 波的简化登记（`src/cards/exp_cata_w2.rs` + `src/cards/herald.rs`）：
大灾变扩展的 13 张传令官卡（官方 15 张减去 W4 的 CATA_190h 死亡之翼与
CATA_497 尤克特拉西安——属 W4 的卡）及其 6 个巨像士兵衍生物，外加仪式
之力的 Breezling 衍生物。与 §14–§23 相同，这些手写扩展卡不在 RL 池中
（经典 + 核心 668/659），行目是信息性的：让代码中的简化在账本上可溯。
每行保持开放，直到其机制落地。

本波的核心机制是一个新原语：**传令官（Herald）**——每玩家
`Player.herald_count`（每次传令官结算 +1，永不重置，W4 的死亡之翼直接读
它）加 id 键控的 `cards::herald` 注册表（`herald_patron` /
`herald_soldier` / `HERALD_CARD_IDS`）。结算传令官关键词先 +1 计数再召唤
该职业主子的巨像士兵；卡面 {0} 即计数值，钉死的"传令官两次升级"档位
（2026-08-09，取自官方卡文 + 官方拉格纳罗斯示例 2 → 4 → 8）把士兵数值
按 ×1（计数 1）、×2（计数 2–3）、×4（计数 4+）缩放。计数在召唤**之前**
递增（本局第一次传令官读"传令官 1"）；每次传令官结算召唤一只**新**士兵
——升级既作用于新召唤士兵的数值，也作用于场上所有友方士兵的实时数值。
没有 `CardEffect::Herald` 变体（方案 B，rewind/kindred 先例）：各游玩路径
在战吼 / 法术 / 武器 / 亡语 / 地标激活的结算点按注册表调用
`resolve_herald`——被反制或法术弯曲的法术结算为空（钩子只活在未被拦截的
分支），随从传令官在其战吼条件处触发（效果召唤的复制品也会传令——官方
文本即"战吼：传令官"）。士兵携带 {0} 的组件（艾拉基尔光环、拉格纳罗斯
亡语、乔加尔回合结束触发器）在召唤时**烘焙**、每次后续传令官**重烘焙**
（光环系统查询时读不到每玩家计数——数值即时更新，主动求值）；被沉默剥
离的组件不再补授。一次性"召唤时"效果（艾萨拉 / 奥妮克希亚 / 辛奈丝特
拉）在结算时读计数。战吼里的关键词不被 Deios 或 BattlecryTwice 暗赐翻倍
（传令官 CardDef 不带战吼组件——关键词只结算一次）。W1 附属物的 {0} 值
**不**随计数升级——官方"传令官升级你的巨像附属物"交互保持范围外（§23
行目保持固定值，由 `cata_w1_*` 场景钉住）。

| ID | 卡牌 | 简化 | 何时真实 |
| --- | --- | --- | --- |
| CATA_156 | 实验性动画 | 完整："传令官。对所有敌方随从造成 4 点伤害"——传令官即游玩路径钩子，伤害是既有原语 | — |
| CATA_158 | 狂乱信徒 | 完整："潜行。亡语：传令官"——亡语路径调钩子（亡语在墓地中仍可读卡牌 id）；CardDef 不带亡语组件 | — |
| CATA_160 | 灼烧掠夺者 | 完整："战吼：传令官。使士兵获得突袭"——突袭在钩子内部、按来源 id 键控，作用于刚召唤的士兵 | — |
| CATA_492 | 暮光圣所 | 完整：地标传令官——"传令官 {0}。抽一张牌"在**激活**时结算（传令官在激活文本里、抽牌之前；抽牌是战吼槽效果） | — |
| CATA_525 | 装甲放血者 | 完整："突袭。战吼：传令官"（突袭走 apply_card_keywords 钩子） | — |
| CATA_530 | 邪能灌注 | 完整："传令官。你的英雄本回合获得吸血"——新 `GrantHeroLifestealThisTurn` 变体（英雄吸血组件 + 每玩家标记，GrantPoisonousThisTurn 惯例，回合末清除） | — |
| CATA_561 | 仪式之力 | 完整："传令官。获得两张 1/1 元素（带突袭）"——两只 CATA_561t Breezling（衍生物不带 {0}——不受传令官缩放） | — |
| CATA_565 | 天墙哨兵 | 完整："嘲讽。战吼：传令官" | — |
| CATA_580 | 灾变战斧 | 完整：武器路径传令官（"战吼：传令官"）在装备后立即结算 | — |
| CATA_722 | 终焉使者 | **中立**主子：传令官 +1 计数但**不**召唤——完整卡库没有中立士兵衍生物 | 中立士兵衍生物 |
| CATA_725 | 暗影誓从 | 完整："战吼：传令官。亡语：为你的英雄恢复 3 点生命值" | — |
| CATA_780 | 执念技师 | 完整："吸血。战吼：传令官"（吸血走 apply_card_keywords 钩子） | — |
| CATA_785 | 暮光仪式 | 完整："传令官。连击：造成 3 点伤害"——连击效果可指向任意角色（官方 "$3 伤害"无目标过滤，从卡库钉死）；连击状态不影响传令官 | — |
| CATA_525t | 艾萨拉士兵 | "召唤时，使你的英雄获得 +{0} 攻击力（本回合）"——{0} 是钉死的档位（基值 2，艾萨拉之触手 §23 值）召唤时读取：+2/+4/+8 | — |
| CATA_565t | 艾拉基尔士兵 | "相邻随从获得 +{0} 攻击力"——光环（基值 1，艾拉基尔充能之手 §23 值）召唤时烘焙、每次传令官重烘焙：+1/+2/+4，实时更新 | 官方实时更新（烘焙以相同数值主动求值） |
| CATA_580t | 拉格纳罗斯士兵 | "亡语：对随机敌人造成 {0} 点伤害"——烘焙 + 重烘焙（基值 2）：2/4/8 | 官方实时更新 |
| CATA_725t | 乔加尔士兵 | "在你的回合结束时，摧毁右侧随从以获得 +{0}/+{0}"——烘焙 + 重烘焙（基值 2，乔加尔之臂 §23 值）：+2/+2、+4/+4、+8/+8；乔加尔牌库摧毁重定向搭 ColossalArmDestroyRight | 官方实时更新 |
| CATA_780t | 奥妮克希亚士兵 | "召唤时，获取一张随机的 {0} 费随从。本回合它消耗生命值"——{0} 是召唤时读取的档位（基值 1 费）：1/2/4 费；CostHealth 标记挂在入手的牌上（"本回合"范围未清除，奥妮克希亚之翼 §23 惯例） | 回合范围 |
| CATA_158t | 辛奈丝特拉士兵 | "召唤时，获取一张其他职业的随机法术。它费用减少（{0}）"——{0} 减费**去掉**（辛奈丝特拉之翼 §23 惯例：引擎的其他职业法术效果无减费参数）；职业过滤去掉（引擎无按卡职业概念，§23） | 减费 + 职业过滤 |
| CATA_561t | Breezling | 完整："突袭"——仪式之力衍生物，无 {0} | — |

F5 覆盖：`cata_w2_*`（`tests/differential.rs` 中 9 个场景）——
`cata_w2_herald_summons_soldier`（打出传令官卡计数到 1 并按基础档位召唤
职业士兵）；`cata_w2_herald_counter_scales_soldier`（第二次传令官召唤
**新**士兵并把场上士兵的光环重烘焙到 ×2 档）；`cata_w2_herald_deathrattle`
（CATA_158 的亡语传令官）；`cata_w2_herald_location`（CATA_492 的激活传
令官 + 抽牌）；`cata_w2_ravager_gives_soldier_rush`（CATA_160 的突袭加成
落在刚召唤的士兵上）；`cata_w2_soldier_azshara_hero_attack`（+2 本回合英
雄攻击 + 邪能灌注英雄吸血本回合范围）；`cata_w2_soldier_ragnaros_deathrattle`
（基值 2 随机敌人伤害）；`cata_w2_soldier_cho_gall_destroys_right`（回合
结束摧毁右侧 +2/+2）；`cata_w2_envoy_neutral_summons_nothing`（中立主子
+1 计数但不召唤）。全部 `cargo test` 全绿（所有套件，含全部
`tlc_w*_*`/`tmw1_*`/`tmw2a_*`/`tmw2b_*`/`tmw3_*`/`cata_w1_*` 场景与 9 个
`cata_w2_*`——954 通过、1 忽略），`cargo fmt` 干净，`cargo clippy
--all-targets` 零警告。

### 25. 2025–2026 扩展 M4-W3 —— 大灾变碎裂波（6 张卡 + 2 个战场衍生物）🔓 登记

M4-W3 波的简化登记（`src/cards/exp_cata_w3.rs`）：大灾变扩展的 6 张碎裂卡
及其 2 个战场衍生物（树妖 CATA_134t3、苍穹巨龙 CATA_479t3）。与 §14–§24
相同，这些手写扩展卡不在 RL 池中（经典 + 核心 668/659），行目是信息性
的：让代码中的简化在账本上可溯。每行保持开放，直到其机制落地。

本波的核心决策是 **D2 简化**（M4-W3 规格 2026-08-09 认可）：官方卡库**没有**
半卡衍生物——完整的分裂/重组管线需要从卡文凭空合成半卡、没有数据锚点，
且 CATA_202 偷取力量卡文直说"(It's already combined)"——合体形态才是可玩
常态。**碎裂卡以合体完整形态打出**：每张卡只带一个效果（或一个新的组合
`CardEffect` 变体），一次结算整段"Shatter. \<合体效果\>"文本——不分裂、无
半卡、无重组。卡库中的 `CATA_xxx t/t2` 半卡"碎裂"衍生物（CATA_134t/134t2、
CATA_306t1/306t2、CATA_479t/479t2、CATA_489t/489t2、CATA_820t/820t2，共
10 个）**不实现**。所有效果都映射到既有原语（亡语附加沿用 M2-W4c 绵羊面
具先例、复制沿用 M2-W4b 的 `SummonCopyOfFriendlyMinion` 惯例、D2 随机池
沿用面具池先例——`pool::SHATTER_POOL`）；只有奥术流的两处 "$" 数值需要
新的 `apply_spell_power` 分支（两处都吃法术伤害加成、维伦双倍两者）。

| ID | 卡牌 | 简化 | 何时真实 |
| --- | --- | --- | --- |
| CATA_134 | 野林环 | 合体形态："召唤两个 2/2 树妖。使你的随从获得'亡语：召唤一个 2/2 树妖'"——`SummonMinionsAndGrantDeathrattleAll`（亡语在召唤**之后**附加，新树妖也吃到；亡语召唤的替身树妖不再继承亡语） | 数据锚定的分裂/重组管线：打出时分裂成半卡、再重组为合体形态 |
| CATA_202 | 偷取力量 | 合体形态："获取一张其他职业的随机碎裂卡。（它已合体）"——`AddRandomShatterCardToHand` 从固定 `SHATTER_POOL`（其余 5 张碎裂卡，均非盗贼）取合体可玩形态 | 职业过滤 + 分裂/重组管线 |
| CATA_306 | 裂痕 | 合体形态："使一个友方随从获得 +2/+3 和精妙。召唤它的一个复制"——`GainStatsElusiveAndSummonCopy`：一次选目标喂三部分；复制为基础卡（无增益、无精妙） | 数据锚定的分裂/重组管线 |
| CATA_479 | 机动演习 | 合体形态："召唤两个 4/2 巨龙。使你的随从获得 +1 攻击力和圣盾"——`SummonMinionsAndGrantFriendlyAttackDivineShield`（增益在召唤后生效，新巨龙也吃到） | 数据锚定的分裂/重组管线 |
| CATA_489 | 奥术流 | 合体形态："造成 $4 点伤害。对所有敌人造成 $2 点伤害"——`DealDamageAndDamageAllEnemies`：主伤害可指向任意角色（官方 "$4 伤害"无目标过滤，§24 钉法），溅射命中所有敌人（含敌方英雄）；两处数值都吃法术伤害加成 | 数据锚定的分裂/重组管线 |
| CATA_820 | 补给线 | 合体形态："抽三张随从牌。使你手牌中的随从获得 +2/+2"——`DrawMinionsAndBuffHandMinions`：每次抽牌从牌库随机选一张随从（同一张不会被抽两次，§20 信鸽使徒惯例），然后所有手牌随从（含刚抽到的）获得 +2/+2 | 数据锚定的分裂/重组管线 |

F5 覆盖：`cata_w3_*`（`tests/differential.rs` 中 6 个场景）——
`cata_w3_wildwood_circle_combined`（两只树妖 + 亡语附加到每个友方随从，
且亡语召唤的替身**不**继承亡语）；`cata_w3_schism_combined`（一次选目标
的 +2/+3 + 精妙增益与基础卡复制）；`cata_w3_flight_maneuvers_combined`
（两只龙族巨龙 + 每个友方随从的 +1 攻击力和圣盾，新巨龙在内）；
`cata_w3_arcane_flow_combined`（奥术师术士在场时主伤害 4+1 命中选定角
色、溅射 2+1 命中敌方英雄与敌方随从）；`cata_w3_supply_run_combined`
（抽 3 张随机随从、牌库剩 2、所有手牌随从 5/4）；
`cata_w3_stolen_power_pool`（从固定池获得一张碎裂卡）。全部 `cargo test`
全绿（所有套件，含全部 `tlc_w*_*`/`tmw1_*`/`tmw2a_*`/`tmw2b_*`/`tmw3_*`/
`cata_w1_*`/`cata_w2_*` 场景与 6 个 `cata_w3_*`——956 通过、1 忽略），
`cargo fmt` 干净，`cargo clippy --all-targets` 零警告。

### 26. 2025–2026 扩展 M4-W4 —— 大灾变收尾波：死亡之翼 + 其余卡（105 张卡 + 21 个衍生物，含 4 张灾变法术）🔓 登记

M4-W4 波的简化登记（`src/cards/exp_cata_w4.rs`）：大灾变子路线图的收尾
波——死亡之翼·灭世者（C4，英雄选择机制）、Ultraxion 与其余约 100 张主系列
卡，外加 21 个衍生物与 4 张数据定义的灾变卡。与 §14–§25 相同，这些手写
扩展卡不在 RL 池中（经典 + 核心 668/659），行目是信息性的：让代码中的
简化在账本上可溯。每行保持开放，直到其机制落地。

本波的头条机制是**完整原语**：**英雄替换**（新 `CardEffect::ReplaceHero
{ card_id }` —— 旧英雄进坟场，打出的实体成为新英雄（取 def 生命、清伤害
与攻击、英雄技能取自卡侧表、毁掉已装备武器、护甲按卡设定），随后结算英雄
战吼；这清偿了 §15 TLC_513t 暮光大师的 "when real" 注记，并保留核心系列
W8 加拉克苏斯流程——含血怒装备）；**选择灾变机制**（`ChoiceKind::Cataclysm`
—— 四张数据定义灾变 CATA_190t10 龙王之怒 / CATA_190t11 倒塌 / CATA_190t12
毁灭 / CATA_190t13 支配，各带独立 `spell_effect`；选择面在每次选取后重出、
去掉已选，直到满足该档次数）；**先锋档缩放**（死亡之翼"选择 {0} 个灾变！
先锋两次以升级"——次数 = herald_number(1, herald_count)：0–1 次为 1、
2–3 次为 2、4+ 为 4，选次互不重复；Deios 翻倍不翻倍战吼）；**无情英雄
技能**（CATA_190p —— 2 费本回合 +5 攻击，由替换原语设定；护甲 12 取官方
数据）；**Ultraxion**（CATA_497 —— 先锋 {0}，随后使死亡之翼费用 -{1}——
先锋钩子把减费累计进 `Player::deathwing_cost_reduction`，由死亡之翼的出牌
费用臂消费；血甲先锋的先锋不减免死亡之翼费用）。

本波的 §26 钉（成文约定）：**支配的传说巨龙池** —— `sets::LEGENDARY_DRAGON_CLASSIC`
（阿莱克丝塔萨、玛里苟斯、奥妮克希亚、死亡之翼）按卡 ID 钉死，因为手写
经典传说 def 先于种族组件、按约定全部无种族（生成基线无 `LEGENDARY_*`
条目，故无门禁风险）；**奈斯皮拉（Nespirah）的纳迦池** ——
`AddRandomNagaCost1` 采样 `HANDWRITTEN_EXPANSION_CARDS`（按决策 D3 关闭
池中不含手写扩展卡，按种族过滤 `ALL_CARDS` 会一无所获）；**深渊公爵的
动态光环** —— 出牌/召唤时按当前弃牌数烘焙、每次友方弃牌再烘焙
（`cards/mod.rs::bake_duke_of_below`）；**沙尘灵气（Sandfury Aura）的双
触发** —— `DoubleTriggers` 光环持续 3 回合（`Player::sandfury_aura_remaining`）；
**吉恩的费旗变换** —— 手牌奇偶检查挂在被标记卡的费旗上；**扭曲怪物
（Twisted Monstrosity）的数据修正** —— 官方数据带嘲讽 + 不可指定（在
cards.json 的 `mechanics` 列表里，不在平铺字段中），手写 def 与生成基线
一致；**四张大灾变地点**（CATA_301 红玉圣地、CATA_477 龙鳞之厅、CATA_527
被魅惑的奈斯皮拉、CATA_584 喷涌火山）——生成器先于地点类型的偏差按
CATA_492（§24）同款登记在 `expansion_differential_rebalanced`。

| ID | 卡 | 简化 | When real |
| --- | --- | --- | --- |
| CATA_190h | 死亡之翼·灭世者 | 完整：英雄替换 + 四灾变选择（先锋档次数 1/2/4，选次不重复）+ 无情英雄技能 + 护甲 12。"先锋两次以升级"读取先锋计数（CATA_497 Ultraxion 与六职业先锋） | — |
| CATA_497 | Ultraxion | 完整：经 W2 §24 钩子先锋 {0}，外加 {1} 死亡之翼费用减免（每次先锋结算累计） | — |
| CATA_131 | 暮色森林树人 | "持有时花费 4 点法力"——持有花费近似挂回合内花费计数 | 手牌来源追踪 |
| CATA_161 | 噩梦（Gruesome Nightmare） | 官方手牌或战场的双重目标集近似为战场一侧 | 官方手牌或战场目标集 |
| CATA_203 | 迦罗娜的最后防线 | 传说过滤器即传说池清单（§26 约定——无稀有度追踪） | 稀有度标记 |
| CATA_304 | 伤者侍从 | 自伤战吼按造成的伤害治疗拥有者（未受伤时的吸血等价） | 官方结算顺序 |
| CATA_470 | 维克多·奈法利安 | 定制"打造"的不死巨龙固定为奈法利安造物衍生物 CATA_470t1 | 打造管线 |
| CATA_978 | 辛达苟萨的胜利 | 溢出 = 伤害减去目标剩余生命 | 官方溢出算法 |
| CATA_621 | 格尔宾的胜利 | 圣骑士灵气池固定为随机圣骑士法术；额外一回合未建模 | 灵气卡池 + 时长 |
| CATA_496 | 诅咒锁链 | "直到他们的回合结束"近似为标准直到回合结束约定 | 官方对手回合末到期 |
| CATA_498 | 拉法姆的最后防线 | （每回合升级！）手牌回合计数器挂在被标记卡上 | 逐手牌计数器 |
| CATA_699 | 恐惧利维坦 | 三次偷取各为目标受等量伤害、来源等量治疗 | 官方偷取算法 |
| CATA_560 | 迎战托维尔人 | 重放的法术对敌方目标结算；重放的随从直接召唤 | 官方重放目标选择 |
| CATA_567 | 升腾 | 变换采样与"死亡时召唤原版"赋予为 D2 近似 | 官方变换池遍历 |
| CATA_581 | 大屠杀 | 升级计数双方场上全部随从 | 官方战场范围 |
| CATA_585 | 火炬 | "以任意溢出伤害回手"近似为死亡判定 | 官方带溢出回手 |
| CATA_591 | 指挥官加尔顿 | 游戏级"改为抽牌"旗标由抽牌步骤钩子读取 | 官方抽牌替代 |
| CATA_610 | 洛戈什的最后防线 | 官方不区分侧的目标近似为友方范围 | 官方任意随从目标 |
| CATA_180 | 战蜥（War'loc） | 一次性"下一个鱼人改付生命"旗标由下一个符合条件的鱼人打出消费（CostHealth 约定） | 官方一次性旗标 |
| CATA_185 | 无面复制者 | 击杀者识别未建模——亡语改为命中一个随机敌方随从 | 官方击杀者链接 |
| CATA_186 | 黏弹破坏者 | 手牌相邻"费用 +1"光环未建模——送出的破坏炸弹 CATA_186t 为空白 2 费法术 | 官方相邻光环 |
| CATA_206 | 扭曲怪物 | 战吼给予两个随机奖励效果；手牌内交换未建模（嘲讽 + 不可指定为真——官方 mechanics 数据） | 官方手牌内交换 |
| CATA_213 | 维拉诺斯 | 牌库快照近似为出牌时的牌库；100 点属性按十张随机牌库随从各 +5/+5 | 官方起始随从快照 |
| CATA_614 | 暗影线人 | （每回合换职业！）职业交换未建模 | 官方每回合职业交换 |
| CATA_721 | 庇护幸存者 | 洗入牌库的卡近似为全新复制 | 官方原卡洗入 |
| CATA_480 | 沙尘灵气 | 完整：双触发光环持续 3 回合 | — |
| CATA_493 | 深渊公爵 | 完整：每弃牌 +2/+2 光环在出牌时烘焙、每次友方弃牌再烘焙 | — |
| CATA_527t2 | 摆脱束缚的奈斯皮拉 | 纳迦池采样 `HANDWRITTEN_EXPANSION_CARDS`（§26 钉——D3 使关闭池不含手写扩展卡） | 全卡池 |
| CATA_464t | 巨龙吐息 | 数值为暂存 `dragon_breath_damage`（来源攻击力） | 官方施放时数值 |
| CATA_470t1 | 奈法利安造物 | 固定"定制不死巨龙"衍生物，持龙时费用 -3 | 打造管线 |
| CATA_478t | 青铜蛮兵 | 按来源属性召唤（复制属性附魔） | 官方属性复制 |

F5 覆盖：`cata_w4_*`（`tests/differential.rs` 16 个场景）——
`cata_w4_deathwing_replaces_hero`（英雄替换：旧英雄进坟场、30 生命 / 护甲
12、无情英雄技能、先锋计数 0 时战吼浮出灾变选择——即 1 次档的单选）、
`cata_w4_deathwing_herald_upgrade_two_distinct_picks`（2–3 档两次不重复
选取、四种灾变效果——龙王之怒的 12/12 后裔、倒塌摧毁最高生命、毁灭 4 伤
清扫）、`cata_w4_deathwing_herald_tier4_enthrall`（4+ 档四次不重复选取、
支配把五张（1）费传说巨龙从 ID 钉死池洗入牌库）、
`cata_w4_deathwing_topple_destroys_highest_health`、
`cata_w4_ultraxion_heralds_and_reduces_deathwing_cost`（先锋 {0} + 累计
死亡之翼费用减免由出牌费用臂消费）、`cata_w4_mossbinding_golems_scale_with_mana`、
`cata_w4_sandfury_aura_doubles_end_turn_effects`、
`cata_w4_stonetalon_striker_transforms_on_dragon`、
`cata_w4_ruby_sanctum_next_heal_deals_damage`、
`cata_w4_chamber_of_aspects_location_activate`、
`cata_w4_duke_of_below_scales_on_discards`、
`cata_w4_maloriak_summons_discarded_minion`、
`cata_w4_nespirah_fel_spell_adds_naga_cost_one`（纳迦池钉）、
`cata_w4_destructive_blaze_survive_summons_and_deathrattle`、
`cata_w4_earthen_drake_end_turn_damage`、
`cata_w4_genn_transforms_and_upgrades_hero_power`。全部 `cargo test` 全绿
（所有套件，含全部 `tlc_w*_*`/`tmw1_*`/`tmw2a_*`/`tmw2b_*`/`tmw3_*`/
`cata_w1_*`/`cata_w2_*`/`cata_w3_*` 场景与 16 个 `cata_w4_*`——976 通过、
1 忽略），`cargo fmt` 干净，`cargo clippy --all-targets` 零警告。

### 27. 2025–2026 扩展 M5-W1 —— 逃脱紫罗兰监狱，破规者波（17 张卡 + 5 个衍生物）🔓 登记

M5-W1 波的简化登记（`src/cards/exp_jail_w1.rs`）：逃脱紫罗兰监狱子路线图
的第一批破规者——开局（Start-of-Game）块（霍格、阿扎莉娜、戈弗雷、大厨
耐斯瑞克、玛格·齐、比阿特丽克斯——经新的 `cards::start_of_game` V1 钩子在
`GameBuilder::build` 中结算，同时清偿 §14 EDR_845 哈缪尔"引擎没有
StartOfGame 事件"的注记）、准备（Prepare）块（凡妮莎、特拉斯塔特、莫拉格
——钉死的关键词：花掉全部剩余法力 → 永久（花费 + 1）费用减免 → 经
CantPlayNextTurn 锁在手牌直到拥有者下个回合开始；每卡一次、每回合一次、
0 法力不可用），以及平铺效果批（薇玛、狱中风暴法师、骷髅钥匙、战钳虫、
活体瘟疫、塔莱娜、卡洛夫）。与 §14–§26 相同，这些手写扩展卡不在 RL 池中
（经典 + 核心 668/659），行目是信息性的：让代码中的简化在账本上可溯。
每行保持开放，直到其机制落地。

本波的头条机制是**完整原语**：**开局钩子**（`cards::start_of_game`——
按 ID 注册表，在 `GameBuilder::build` 中于先后手覆盖之后、起手牌库快照与
洗牌之前结算；牌库按序扫描，先一号玩家后二号玩家，每张注册卡经常规
`trigger::resolve_effect` 管线结算）；**准备**（见上方钉——CantPlayNextTurn
组件沿用 CATA_186t 破坏炸弹先例；减免经 `reduce_hand_card_cost`；每回合
一次守卫与每卡清单挂在 `Player` 上）；**战钳虫的伤害管线召唤**（四个
不同的友方角色在拥有者回合受到真实伤害——护甲吸收的伤害不计，沿用
Emberroot 约定——随后手牌/牌库中的真卡移入战场，非新复制）；**活体瘟疫的
英雄伤害重定向**（瘟疫攻击英雄不入伤害事件；按攻击力数量的噬咒者洗入该
英雄牌库随机位置）；**阿娅的假币机制**（建造器级 `aya_flip` 先后手覆盖 +
三分支抉择战吼：战吼槽 = 玉币、抉择槽 = 污手街硬币、卡侧表 = 卡扎库斯币）；
**玛格·齐的条件英雄技能**（被动技能活在费用管线——玛格的魔法：每回合
第一张随从费用 -2——与打出计数器——齐的威能：每打出第五张随从战吼触发
两次）；**卡洛夫的亡语**（三个 1/1 传说古加尔兄弟）。

本波的 §27 钉（成文约定）：**比阿特丽克斯的组牌选择** ——"构筑牌组时选择
一张 2 费随从"在引擎开局没有落点，改为开局随机采样一张 2 费随从
（ALL_CARDS，排除衍生物）十张加入起始牌库；**玛格·齐双条件角落** ——
牌库既无其他随从也无法术（空牌库边缘）时两个旗标都给，但齐的威能赢得
唯一英雄技能槽；**齐的威能 × 抉择** —— 抉择的第五张随从战吼经选择系统
结算、不翻倍（就绪旗标仍被消费）；**阿娅对称翻转角落** —— 双方牌库都有
阿娅（或都没有）则不翻转；**卡扎库斯币药水池** ——"获得一张随机的 1 费
卡扎库斯药水"改为采样 1 费法术池；**玉傀儡成长** —— 玉币的傀儡固定为
1/1（JAIL_504tj，官方递增傀儡未建模）；**莫拉格的牌库消耗** —— 亡语从
牌库召唤随机恶魔并把该牌库卡移入坟场（满场情况未检查）；**塔莱娜的单英雄
技能槽** ——"获得第二个英雄技能"改为替换为吸血鬼之吻（3 具尸骸代替法力）。

| ID | 卡 | 简化 | When real |
| --- | --- | --- | --- |
| JAIL_384 | 破链者霍格 | 完整：开局复制起始牌库中每张其他传说卡 | — |
| JAIL_430 | 灵魂收割者阿扎莉娜 | 完整：起始生命 40、20+20 牌库构造、抽到手满战吼（经 F-A11 上限） | — |
| JAIL_509 | 背叛者戈弗雷 | 完整：F-A11 焚毁覆盖把过牌存起；拥有者回合开始按最旧优先返还，各费用 -1 | — |
| JAIL_860 | 大厨耐斯瑞克 | 完整：牌库全 ≤3 费检查；回合 5 法力置 10 | — |
| JAIL_800 | 玛格·齐 | 完整：无其他随从 / 无法术检查赋予被动英雄技能；双条件角落由齐的威能赢得唯一英雄技能槽 | — |
| JAIL_397 | 比阿特丽克斯指挥官 | "构筑牌组时选择一张 2 费随从"→ 开局随机 2 费随从 ×10（无组牌界面） | 组牌选择 |
| JAIL_407 | 凡妮莎·头目 | 完整：准备 + 打出后随机战吼随从费用 -2 | — |
| JAIL_721 | 灵魂寄生虫特拉斯塔特 | 完整：准备 + 突袭；召唤恶魔后属性增益 | — |
| JAIL_906 | 莫拉格 | 亡语消耗牌库卡（移入坟场）；满场情况未检查 | 官方牌库处理 |
| JAIL_504 | 莲花大王阿娅 | 完整：`aya_flip` 先后手覆盖 + 三分支抉择假币选择；双方都有阿娅则不翻转 | — |
| JAIL_118 | 逼近的死亡薇玛 | 圣骑士职业过滤器省略——摧毁全部随从（沿用辛尼斯塔拉 §23 先例） | 职业过滤器 |
| JAIL_122 | 狱中风暴法师 | "本局游戏每当你施放法术"→ 存活期间：法术施放钩子检查战场 | 游戏级触发 |
| JAIL_319 | 骷髅钥匙 | 刷新 / 20% 伤害半简化掉：纯发现一张法术 | 刷新管线 |
| JAIL_421 | 战钳虫 | 完整：四个不同友方角色受伤后从手牌或牌库召唤；召唤消费每回合计数（再次跨过四点找不到复制——卡已在场上） | 每次跨过重新触发 |
| JAIL_443 | 活体瘟疫 | 噬咒者的"抽到时造成 2 点伤害"以可打出的 1 费法术实现（抽到即施放简化）；洗入数量 = 瘟疫攻击伤害 | 抽到即施放管线 |
| JAIL_446 | 血医塔莱娜 | "获得第二个英雄技能"→ 替换：英雄技能变为吸血鬼之吻（3 具尸骸代替法力）——单英雄技能槽 | 真正的第二英雄技能槽 |
| JAIL_448 | 破碎者卡洛夫 | 完整：亡语召唤三个 1/1 传说古加尔兄弟 | — |
| JAIL_504t | 玉币 | 傀儡成长固定为 1/1（JAIL_504tj） | 递增玉傀儡 |
| JAIL_504t2 | 污手街硬币 | 完整 | — |
| JAIL_504t3 | 卡扎库斯币 | "获得一张随机的 1 费卡扎库斯药水"→ 1 费法术池 | 卡扎库斯药水池 |
| JAIL_443t | 噬咒者 | 抽到即施放的 2 点伤害以可打出的法术实现（见 JAIL_443） | 抽到即施放管线 |
| JAIL_504tj | 玉傀儡 | 玉币的固定 1/1 衍生物 | 递增玉傀儡 |

F5 覆盖：`jail_w1_*`（`tests/differential.rs` 19 个场景）——
`jail_w1_hogger_duplicates_legendaries`（开局复制牌库中每张其他传说卡）、
`jail_w1_azalina_health_and_deck`（起始生命 40 + 20+20 牌库）、
`jail_w1_aya_always_second`（翻转）+ `jail_w1_aya_battlecry_upgrades_coins`
（三分支硬币选择）、`jail_w1_godfrey_overdraw_returns`（焚毁覆盖 + 返还）、
`jail_w1_nethrek_mana_after_five`（牌库检查 + 回合 5 法力）、
`jail_w1_mugzee_hero_power` + `jail_w1_mugzee_zee_might_doubles_battlecry`
（被动技能 + 第五张翻倍）、`jail_w1_beatrix_ten_copies`（随机 2 费 ×10）、
`jail_w1_vanessa_prepare_after_play`（完整准备钉：全花、花费 +1 减免、锁、
每卡一次、每回合一次、0 法力门、到期、以及她打出即触发自身事件的约定）、
`jail_w1_trastath_demon_stats`（召唤恶魔属性增益）、
`jail_w1_moragg_chain`（连锁亡语 + 新复制召唤不适）、
`jail_w1_vama_destroys_all`、`jail_w1_manastorm_summons_on_spell`（存活期
约定）、`jail_w1_skeleton_key_discovers`、`jail_w1_warptooth_summons_on_four`
（四点伤害召唤）、`jail_w1_living_plague_shuffles_blights`（英雄伤害重定向）、
`jail_w1_thalena_corpses_hero_power`（尸骸费用替换）、
`jail_w1_karov_three_legendaries`。全部 `cargo test` 全绿（所有套件，含
全部既有场景——996 通过、1 忽略），`cargo fmt` 干净，
`cargo clippy --all-targets` 零警告。

### 28. 2025–2026 扩展 M5-W2 —— 逃脱紫罗兰监狱，收尾波（139 张卡：10 传说 + 108 非传说 + 21 个衍生物）🔓 登记

M5-W2 波的简化登记（`src/cards/exp_jail_w2.rs`）：2025–2026 扩展路线图的
最终一波——全部剩余 JAIL 卡（10 传说 + 108 非传说）加本波衍生物，在
`sets.rs` 注册（139 条）。与 §14–§27 相同，这些手写扩展卡不在 RL 池中
（经典 + 核心 668/659），行目是信息性的：让代码中的简化在账本上可溯。
每行保持开放，直到其机制落地。

本波的头条机制是**完整原语**：**微型宝的弹药**（`ChoiceKind::TinyPalAmmo`
——战吼在四种弹药武器 JAIL_458t1–t4 中选择；每次英雄攻击发射已装填的
弹药并经 `weapon_trigger` 表再次弹出选择）；**刀光剑影**（经 M3 倒带机制
重放本回合此前打出的卡——`Player::rewind_turn_start_len`——随后结束回合）；
**艾瑞达·星寻**（整副牌库移入 `Player::void_cards`；随从自身携带的回合开始
触发每次取回两张随机卡——艾瑞达离场则停止）；**深渊之王 / 诡计法杖**
（D2 随机发现池 + `pending_discover_cost_reduction`）；**守望者玛维**（经
`CardPlayed` 随从携带触发给每张打出的随从 +3/+3 并休眠 1 回合）；
**探员穆尔·福尔摩斯**（`ChoiceKind::MurlocHolmes`——读取敌方手牌三张；
嫌疑人记在 `Player::murloc_holmes_suspect`，`CardPlayed` 处理器在对方打出
同名卡时给付 3 枚硬币）；**走私王托格瓦格**（双方手牌合并、洗匀、对半
分发）；**R4T-C4TCH3R**（牌库法术在随机位置复制；亡语抽一张）；**祖拉玛
的牢狱**（地点卡的战吼是 `ChooseHandCard` 弃牌召唤；被解放的祖拉玛
JAIL_887t2 每回合打出一张被弃卡）；**血之克隆**（`ChoiceKind::BloodClone`
——仅在 5 具尸骸可付时弹出发现，选中后花费并召唤）；**元素计数器卡**
（熔金术 / 寒霜碎裂 / 风暴之怒读取 `Player::spells_cast_this_turn`——在
`SpellCast` 处理器中递增，而该事件在法术自身效果结算之后才触发，因此
`>= 3` 检查读到的是另外三张法术；元素衍生物经臂内的来源 id 分支复用父
法术效果，所以元素自身的战吼在召唤时仍会打出伤害）；**被囚的大法师**
（随从携带的死亡计数器在 `MinionDied` 处理器中于死者自身亡语结算之前
递增，读到的恰是"其他"死亡数：四个其他大法师 → 火球术）；以及**十二张
新准备卡**（注册进 `cards::prepare::is_prepare_card`，由 W1 的
`PrepareCardExecuted` 钉驱动）。

本波的 §28 钉（成文约定）：**刀光剑影** ——"尽可能以敌人为目标"省略，
重放沿用原始目标；**深渊之王** —— 组牌选择改为战吼时的 D2 随机三野兽池
（无组牌界面）；**诡计法杖** —— 减免 = 英雄攻击力含已装备武器
（`compute_attacker_damage`，亦覆盖利刃手套的护甲当攻击）；**福尔摩斯**
—— 追踪按回合而非按名字；**托格瓦格** —— 施法者拿第一半向上取整
（"你拿一半"拆分约定）；**解放灵魂** ——"持有时"条件省略（费用 -1 经
`cost.rs`）；**复制族** —— 心灵清道夫与受缚暗影用 `copy_card_to_hand`
在来源属主不同时打上的 `CopiedFromOpponent` 标记近似"从对手复制"；
**虚空之魂** —— 官方"本局游戏首次施放"升级改为按次升级（随机恶魔费用 =
1 + 等级，每施放一次升一级）；**援军光环** —— 三回合从施放回合起连续
（`reinforcement_aura_ticks`；召唤的随从为新复制——芬杰约定）；**掘地求生**
—— 友方随从选择随机化；授予的亡语是新效果变体
`SummonTwoRandomMinionsOfCost`（需同步镜像/机器人更新的偏差）；
**野兽陷阱 / 奥术陷阱** —— 陷阱衍生物按 W1 噬咒者约定做成可打出法术
（抽到即施放特性未建模）；**狱卒鸟** —— 准备的减免是永久手牌费用减免；
**恶魔囚禁** —— -3/-3 半省略（非恶魔目标改为休眠 2 回合）；**紫罗兰惩罚者**
—— 关键词计数读取随从的关键词组件（嘲讽 / 圣盾 / 风怒 / 冲锋 / 潜行 /
剧毒 / 突袭 / 魔免 / 复生 / 吸血），关键词偷取未建模；**孤寂之塔** ——
恶魔属性 = 手牌数，带永久潜行（引擎的不可指定衍生物约定）；敌方无随从
时不攻击（空列表守卫）；**蛛网小精灵** ——"你的回合"限定省略；**被俘的
纳斯雷兹姆** ——"所有"用全局法力怨灵费用光环近似（双方手牌），条件 +5
攻击省略；**顽抗守卫** —— 条件 +5 攻击省略；**"抽到即施放"族** ——
JAIL_386t/879t/881t 为可打出法术；**顺手牵羊** —— 洗入复制直接设费；
**寡妇之咬** —— 升级链封顶三层；**莲花惹事精** —— 重复追加省略；
**紧急手术** —— 每张手牌一个 3/1 吸血护士（JAIL_454t）；**盗贼工具** ——
"偷取"近似为随机敌方手牌卡的新复制；**孱弱食尸鬼** —— 衍生物自定
（官方卡为 HERO_11bpt）。

| ID | 卡 | 简化 | When real |
| --- | --- | --- | --- |
| JAIL_458 | 微型宝 | 完整：弹药选择 + 已装填射击的武器触发（JAIL_458t1–t4） | — |
| JAIL_500 | 刀光剑影 | 倒带重放完整；"尽可能以敌人为目标"省略——重放沿用原始目标 | 敌向重选目标 |
| JAIL_719 | 艾瑞达·星寻 | 完整：整副牌库入虚空；你的每个回合开始取回两张随机卡（艾瑞达离场则停止） | — |
| JAIL_831 | 深渊之王 | 组牌选择 → 战吼时 D2 随机三野兽池，费用 -3 | 组牌选择 |
| JAIL_850 | 守望者玛维 | 完整：每张打出的随从 +3/+3 并休眠 1 回合 | — |
| JAIL_851 | 探员穆尔·福尔摩斯 | 完整：敌方手牌三卡调查 + 对方打出同名卡给付 3 硬币（按回合追踪） | — |
| JAIL_852 | 走私王托格瓦格 | 完整：双方手牌合并、洗匀、对半分发——施法者拿第一半向上取整 | — |
| JAIL_875 | 诡计法杖 | 德鲁伊发现为 D2 随机池；减免 = 英雄攻击力含已装备武器 | — |
| JAIL_882 | R4T-C4TCH3R | 完整：牌库法术在随机位置复制；亡语抽一张 | — |
| JAIL_887 | 祖拉玛的牢狱 | 完整：选择弃牌的地点卡；祖拉玛每回合打出一张被弃卡（回合结束触发） | — |
| JAIL_007 | 下水道小鬼 | "若存活"限定近似——触发不含存活检查 | 存活检查 |
| JAIL_029 | 暴乱者 | "且存活"限定近似——效果在受伤时结算，不经死亡检查 | 存活检查 |
| JAIL_030 | 脱逃艺术家 | "给随机友方随从圣盾"——随机选择 | — |
| JAIL_101 | 紫罗兰惩罚者 | 关键词计数读取关键词组件；关键词偷取未建模 | 关键词偷取 |
| JAIL_123 | 越狱建筑师 | "本回合你施放的下一张法术施放两次"发现 → 随机选择 | 发现 |
| JAIL_200 | 鼠害侵入 | "它拥有你英雄的攻击力"加成省略 | 英雄攻击复制 |
| JAIL_202 | 蛛网小精灵 | "你的回合"限定省略——+1/+1 光环两回合都生效 | 回合限定 |
| JAIL_205 | 鼠贼 | 偷取近似为抽牌机制抽一张敌方牌库随机卡 | 牌库顶偷取 |
| JAIL_206 | 黑暗贿赂 | 选择范围是整只手牌，不限于抽到的三张 | 仅限抽到的选择 |
| JAIL_225 | 顺手牵羊 | 洗入复制直接设费 | 费用 (2) 复制 |
| JAIL_303 | 远古预言者 | 友方随从随机选择 | — |
| JAIL_321 | 花招即兴者 | 奥秘为随机法师职业奥秘，非牌库读取 | 牌库读取 |
| JAIL_326 | 审判 | "把全部随从属性设为随机友方随从"——随机选择 | — |
| JAIL_327 | 援军光环 | 三回合从施放回合起连续；召唤的随从为新复制（芬杰约定） | 每回合一次召唤 |
| JAIL_380 | 走私铁锹 | 抽到的法术必须不在起始牌库中 | — |
| JAIL_386 | 抢夺装备 | 护甲衍生物按 W1 噬咒者约定做成可打出法术（抽到不自动施放） | 抽到即施放管线 |
| JAIL_387 | 释放野兽 | 两只随机 4 费野兽取自封闭池 | — |
| JAIL_395 | 下水道泳者 | 随机 3 费野兽选择；潜行按引擎约定永久 | — |
| JAIL_398 | 大恶魔！ | "若它死亡"升级追加省略 | 死亡链 |
| JAIL_433 | 解放灵魂 | "持有时"省略——费用 -1 经 cost.rs | 持有条件 |
| JAIL_436 | 寡妇之咬 | 升级链封顶三层 | 开放链条 |
| JAIL_442 | 伪装医生 | 魔免已建模；+2/+2 目标随机 | — |
| JAIL_444 | 锯骨者 | +2/+2 目标随机；授予魔免 | — |
| JAIL_451 | 血之克隆 | 完整：5 费发现仅在 5 具尸骸可付时弹出；选中后花费并召唤 | — |
| JAIL_453 | 狱卒鸟 | 完整：准备减免为永久手牌费用减免（W1 钉） | — |
| JAIL_454 | 紧急手术 | 每张手牌一个 3/1 吸血护士（JAIL_454t） | — |
| JAIL_456 | P1CK-P0K3T | +2/+2 目标随机 | — |
| JAIL_459 | 蛛形兽 | 友方随从随机选择；潜行永久 | — |
| JAIL_470 | 莲花惹事精 | "若死亡则再造成 1 点伤害"重复追加省略 | 重复追加 |
| JAIL_474 | 翡翠守卫 | "本局游戏施放过费用 (2)"计数器近似 | 游戏级计数器 |
| JAIL_501 | 开锁贼 | 动态数值（伤害 / 攻击）固定为 1 | 动态缩放 |
| JAIL_502 | 警报机 | "替换自身"追加省略——随从留在场上 | 替换追加 |
| JAIL_507 | 恶意厨师 | 费用归零经手牌费用减免 | — |
| JAIL_511 | 孤寂之塔 | 完整：手牌等属性的希瓦拉渗透者带永久潜行；敌方无随从则不攻击 | — |
| JAIL_513 | 笼中颅骨 | 潜行按引擎约定永久 | — |
| JAIL_515 | 暗影轮射 | 完整：击杀目标后连锁再次施放 | — |
| JAIL_516 | 猩红招募官 | 完整：两张费用 ≤2 的牌库随从 +2/+2 | — |
| JAIL_706 | 盗贼工具 | "偷取"近似为随机敌方手牌卡的新复制 | 真偷取 |
| JAIL_718 | 黑市拍卖师 | 完整：友方法术施放抽牌 | — |
| JAIL_732 | 虚空之魂 | 官方首次施放升级 → 按次等级（随机恶魔费用 = 1 + 等级，有界） | 开放升级 |
| JAIL_733 | 恶毒虚空鳞 | 发现为随机牌库卡；空牌库读取近似 | 牌库读取 |
| JAIL_735 | 紫罗兰暗码 | 随机 7 费随从带嘲讽召唤 | — |
| JAIL_801 | 熔金术 | 完整：3 张其他法术计数器；元素自身战吼召唤时仍造成 4 点伤害 | — |
| JAIL_803 | 寒霜碎裂 | 完整：3 张其他法术计数器；5/5 元素（JAIL_803t）战吼造成 5 | — |
| JAIL_805 | 风暴之怒 | 完整：3 张其他法术计数器；7/7 吸血元素（JAIL_805t）战吼造成 7 | — |
| JAIL_806 | 咒印元帅 | 牌库检查读取牌库法术数 | 牌库读取 |
| JAIL_861 | 恶毒贿赂 | 随机卡选择 | — |
| JAIL_876 | 掘地求生 | 友方随从随机选择；授予的亡语为新效果变体 SummonTwoRandomMinionsOfCost | 指定随从 |
| JAIL_877 | 地下网络 | +2/+2 目标随机 | — |
| JAIL_878 | 看门犬 | 圣盾目标随机 | — |
| JAIL_879 | 野兽陷阱 | 完整：随机 5 费野兽召唤 + 两张陷阱衍生物洗入（按噬咒者约定做成可打出法术） | — |
| JAIL_881 | 奥术陷阱 | 陷阱衍生物按噬咒者约定做成可打出法术，不受法术伤害加成 | 抽到即施放管线 |
| JAIL_890 | 被俘的纳斯雷兹姆 | "所有"用全局法力怨灵费用光环近似（双方手牌）；条件 +5 攻击省略 | 双方读取 |
| JAIL_909 | 迪菲亚小混混 | "本局游戏剩余时间"连击追加简化 | 游戏级追加 |
| JAIL_912 | 预言者 | +3/+3 目标随机 | — |
| JAIL_913 | 顶住！ | 亡语目标为首个友方亡语随从 | — |
| JAIL_940 | 亡语判决 | 目标为首个友方亡语随从，非随机 | 随机触发 |
| JAIL_974 | 被囚的大法师 | 完整：4 个其他死亡计数器对随机敌人施放火球术（6 点） | — |
| JAIL_986 | 狂热伪造者 | 随机可打出法术池 | — |
| JAIL_987 | 低安全翼区 | 门槛为 LockedUntilCardPlayed 标记，任意出牌清除 | — |
| JAIL_997 | 恶魔囚禁 | -3/-3 半省略——非恶魔目标改为休眠 2 回合 | 完整 -3/-3 |
| JAIL_998 | 迪菲亚走私者 | +2/+2 目标随机 | — |
| JAIL_440t | 孱弱食尸鬼 | 衍生物自定（官方卡为 HERO_11bpt） | 官方衍生物 |
| JAIL_458t1–t4 | 微型宝弹药 | 完整：冻结 2 / +2/+2 / 3 点伤害 / 2/1 嘲讽各装填射击 | — |
| JAIL_879t / 881t | 陷阱衍生物 | 按 W1 噬咒者约定做成可打出法术（抽到不自动施放） | 抽到即施放管线 |
| JAIL_887t2 / 887t3 | 祖拉玛 / 虚空低语 | 牢狱亡语链的完整衍生物 | — |

F5 覆盖：`jail_w2_*`（`tests/differential.rs` 18 个场景）——
`jail_w2_slice_and_dice_replays_and_ends_turn`（倒带重放 + 回合结束推进）、
`jail_w2_tiny_pal_ammo_freeze_cycle`（弹药选择、装备射击、冻结、再次弹出）、
`jail_w2_irida_sinseeker_void_deck`（整副牌库入虚空 + 两张取回）、
`jail_w2_king_of_the_underbelly_discover_beast`（野兽发现 + 费用 -3）、
`jail_w2_warden_maiev_dormant_buff`（+3/+3 + 休眠）、
`jail_w2_murloc_holmes_investigates`（调查 + 3 硬币给付）、
`jail_w2_togwaggle_shuffles_hands_together`（合并 + 向上取整对半拆分）、
`jail_w2_staff_of_trickery_discovers_druid`（德鲁伊发现 + 英雄攻击减免）、
`jail_w2_r4t_catcher_copies_deck_spells`（牌库法术复制）、
`jail_w2_zuramats_prison_discard_chain`（选卡弃牌的地点 + 解放祖拉玛的
每回合打出）、`jail_w2_molten_gold_elemental_threshold`（3 张其他法术计数器
+ 元素自身战吼）、`jail_w2_beast_tripwire_summons_and_shuffles`（5 费野兽
+ 两张陷阱衍生物）、`jail_w2_blood_clone_spends_corpses`（尸骸花费 + 召唤）、
`jail_w2_jailbird_prepare_discount`（W2 卡上的准备关键词减免）、
`jail_w2_dig_for_freedom_grants_deathrattle`（授予的
SummonTwoRandomMinionsOfCost 亡语——效果召唤的随从战吼会触发，既有引擎
约定）、`jail_w2_reinforcement_aura_end_turn_summon`（三跳光环）、
`jail_w2_spire_of_solitude_hand_sized_demon`（手牌等属性恶魔 + 无目标守卫）、
`jail_w2_captured_archmage_four_deaths_fireball`（4 个其他死亡 → 火球术）。
全部 `cargo test` 全绿（所有套件，含全部既有场景），`cargo fmt` 干净，
`cargo clippy --all-targets` 零警告。

### 29. MEND W1 —— 大灾变职业套装 W1 波，德鲁伊（8 张卡：MEND_040~046 + MEND_100，+ 2 个衍生物：MEND_046t / MEND_100t）🔓 登记

MEND W1 波（`src/cards/exp_cata_w5.rs`）的简化登记：第一个 MEND_ 波
（2025–2026 扩展总路线图 M5 的跟进）——大灾变德鲁伊职业套装，8 张卡 +
2 个衍生物，注册于 `sets.rs`（10 条手写条目；每张 MEND_ 卡都覆盖一条
生成基线，MEND_044 的地点偏差已在 `expansion_differential_rebalanced`
中登记）。与 §14–§28 相同，这些卡不在 RL 池（经典 + 核心 668/659）内，
所以各行仅为信息性登记：让代码里的简化可追溯到账本。每一行在其机制
落地前保持开启。

该波的头条机制均为完整原语：**"上回合没出随从"**（睿智野语者、心根之石）
直接复用既有的 `last_turn_minion_play_ids` 快照——每回合出牌列表在
CardPlayed 时压入、回合结束时 mem::take 进"上一回合"列表（M3-W2b 机制），
因此该旗标不需要任何新的 Player 状态；**灰烬蠕虫** 通过
`dormant_at_summon` 注册表的 u32::MAX 哨兵进入沉睡——回合开始倒计时跳过
哨兵，MinionSummoned 处理器（所有召唤的唯一漏斗）在己方战场达到 7 个
随从时唤醒哨兵沉睡随从；蠕虫在 6 随从战场出场时自己填满战场并立即苏醒
（与官方一致）；**静谧空地** 是激活效果放在战吼槽的地点（W2 §24 约定），
目标随从选择在 ActivateLocation 动作上浮出；**生命绽放** 走
AllFriendlyCharacters 治疗路径（暗鳞治愈者先例——维伦翻倍、法术伤害不
加成）并召唤两个随机 8 费随从；**播种巨龙** 亡语给手牌加一张随机龙并
减费 (2)。

该波还发现并修复了一个既有的保真缺口：**"对所有角色造成 X 点伤害"**——
地狱烈焰、憎恶、恐惧地狱火与加尔顿男爵——此前在 `resolve_deal_damage`
中是静默 no-op（AllCharacters 分支落进了 no-op 组）。该分支现已接通：
双方英雄 + 全部随从（含潜行——AOE 无视潜行；恶癖不挡 AOE），排除效果
来源（法术本身不是角色，且随从卡的官方文本是"所有其他角色"）。由
`mend_w1_hellfire_damages_all_characters` 钉住（MEND W1 调研中浮出——
法球随机施法抽到了地狱烈焰）。

该波的 §29 钉（已登记约定）：**巴莎娜·符文图腾**——官方雕刻把法术"刻"
进树人衍生物（每个 MEND_046t 2/2 带"战吼：施放 {0}"）；简化形态直接往
手牌加三个白板 2/2 树人 + 最多三张随机自然法术（每张费用 ≤ 剩余 12 费
预算）；**培养之精**——法球（MEND_100t）自身费用固定为 3，"每回合升级"
让施放法术的费用每经过一个持于手牌的己方回合开始 +1（HandTurnCounter
约定，CATA_498），施放该费用的三张随机法术；**静谧空地的沉睡**——目标
获得沉睡（Dormant 1），在"其下一次回合开始"唤醒、与拥有者无关，对照
官方"沉睡至你的下个回合结束"（若中间是对手回合开始则提前唤醒）；**随机
池**——播种巨龙的龙、生命绽放的 8 费随从、巴莎娜雕刻的自然法术与法球
施放的法术都从全卡池采样（`random_minion_of_cost` 约定，无窗口过滤），
对照官方为活跃窗口池。

| ID | 卡牌 | 简化 | 真实形态 |
| --- | --- | --- | --- |
| MEND_040 | 灰烬蠕虫 | 完整：召唤即沉睡哨兵 + 战场满唤醒（蠕虫自己填满战场也会立即唤醒） | — |
| MEND_041 | 睿智野语者 | 完整：上回合无随从旗标读取 `last_turn_minion_play_ids` 快照 | — |
| MEND_042 | 生命绽放 | 完整：全体友方治疗 + 两个随机 8 费随从（池为全卡池） | 活跃窗口池 |
| MEND_043 | 心根之石 | 完整：抽牌 + 护甲的重复读取上回合旗标 | — |
| MEND_044 | 静谧空地 | 完整地点 + 目标 +2 血嘲讽；沉睡为 Dormant 1——目标在其下一次回合开始唤醒、与拥有者无关 | "沉睡至你的下个回合结束" |
| MEND_045 | 播种巨龙 | 完整：亡语加随机龙并减费 (2)（池为全卡池） | 活跃窗口池 |
| MEND_046 | 巴莎娜·符文图腾 | 三个白板 2/2 树人 + 最多三张随机自然法术（每张 ≤ 剩余 12 费预算）加入手牌 | 官方雕刻把法术刻进树人衍生物 |
| MEND_046t | 树人 | 白板 2/2（无"施放 {0}"战吼） | 雕刻法术战吼 |
| MEND_100 | 培养之精 | 完整：3 费法球入手，每经己方回合开始 +1 刻度（HandTurnCounter） | — |
| MEND_100t | 绽放法球 | 法球自身费用固定为 3；升级提升施放法术的费用（每刻度 +1）；施放池为全卡池 | 官方升级语义与活跃窗口池 |

F5 覆盖：`mend_w1_*`（`tests/differential.rs` 12 个场景）——
`mend_w1_ash_worm_awakens_on_full_board`（第 7 个召唤唤醒沉睡蠕虫）、
`mend_w1_ash_worm_played_onto_full_board_awakens_immediately`（蠕虫自己填满
战场）、`mend_w1_ash_worm_stays_dormant_on_partial_board`（未满战场保持
沉睡）、`mend_w1_wizened_wildspeaker_refreshes_when_no_minion_last_turn` /
`mend_w1_wizened_wildspeaker_no_refresh_after_minion_played`（刷新与不刷新）、
`mend_w1_heartroot_stones_repeats_when_no_minion_last_turn` /
`mend_w1_heartroot_stones_single_when_minion_played_last_turn`（双抽 + 6 甲
与单次执行）、`mend_w1_tranquil_clearing_sleeps_target_until_next_turn`
（目标 +2 血嘲讽、沉睡阻止双方攻击、己方下回合开始唤醒、第二次激活摧毁
地点）、`mend_w1_seeding_dragon_deathrattle_gives_dragon_costs_2_less`
（随机龙 + 减费 (2)）、`mend_w1_bashana_runetotem_treants_and_carved_spells`
（三个 2/2 树人 + ≤ 12 费预算内的自然法术）、
`mend_w1_cultivating_sprite_bulb_upgrades_each_turn`（法球的手牌回合刻度与
确定性施法——野性咆哮 + 盾牌格挡 + 地狱烈焰——钉住升级）、
`mend_w1_hellfire_damages_all_characters`（重新接通的 AllCharacters 伤害
分支）。全部 `cargo test` 全绿（所有套件，含全部既有场景），`cargo fmt`
干净，`cargo clippy --all-targets` 零警告。
