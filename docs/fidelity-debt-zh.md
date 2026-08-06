# 保真债 — 简化卡清单（F4/F5 持续审计账本）

> **现状：`src/cards/` 里已无简化标记——67 张保真债全部清偿（W7 收尾轮 PR #86 清掉最后 3 张）。
> W0 接线轮 PR #79 清掉 13 张；W1 种族轮 PR #80 清掉 11 张；W2 触发轮 PR #81 清掉 8 张；
> W3 谓词轮 PR #82 清掉 9 张；W4 费用/武器轮 PR #83 清掉 8 张；W5 目标结构轮 PR #84 清掉 7 张；
> W6 特殊机制轮 PR #85 清掉 8 张。W7 后账本一度清空、RL 卡池达到全经典构筑池满规模
> （391 张）。**2026-08-06 补登记轮：状态审计发现 27 张已知简化卡没有 `(simplified: …)` 标记——
> 一直静默混在 RL 卡池里。27 张现已全部补上注释并登记（§11）；债务集合从 0 增至 27，
> RL 卡池先降至 361（债务 27），F-A8 过载修复后再升至 367（413 − 24 债 − 21 衍生物 − 硬币，重编 wheel 后实测）。需要重训。F-A8（8 处卡 ID 冲突）已解决（PR #88）。**
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

### 9. 复合与其他（6 张）✅ 已解决（W6，PR #85）

全部落地：群体圣盾（正义）、排除自身的全场伤害（伊瑟拉之醒）、抽牌-按费用
伤害（神圣愤怒）、受伤友方回合开始治疗（光明之泉）、群体增益+嘲讽（自然之力）、
职业过滤抽卡（顺手牵羊——`OtherClass` 池用职业组表过滤，核实已忠实，只清注释）。
差分场景 `w6_*`。

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


### 11. 2026-08-06 补登记 — 既有简化债（27 → 24 张）

2026-08-06 状态审计（`docs/classic-cards-zh.md` 对照代码）发现 27 张已知简化卡
没有 `(simplified: …)` 标记——一直静默混在 RL 卡池里。27 张 def 现已全部补上注释
（债务集合 = 27）；离开账本仍走[维护约定](#维护约定)流程（实现 → F5 差分 →
删注释 → 失效缓存）。

| ID | 卡名 | 简化债 |
| --- | --- | --- |
| CLASSIC_018 | 阿曼尼狂战士 | 无激怒 — 白板 2/3 |
| NEUTRAL_008 | 暴怒的狼人 | 仅风怒，无激怒 |
| WARRIOR_010 | 格罗玛什 | 仅冲锋，无激怒 |
| WARRIOR_008 | 战歌指挥官 | 无冲锋光环 — 白板 2/3 |
| PRIEST_004 | 北郡牧师 | 无治疗抽牌 — 白板 1/3 |
| HUNTER_021 | 狂野怒火 | 作用于任意友方随从（真实为野兽） |
| NEUTRAL_026 | 海巨人 | 无减费 — 白板 8/8 |
| DRUID_004 | 愤怒 | 仅 3 伤分支 |
| DRUID_007 | 利爪德鲁伊 | 固定 4/6 嘲讽，无抉择 |
| DRUID_008 | 知识古树 | 仅抽 2 分支 |
| DRUID_009 | 战争古树 | 白板 5/5，无抉择 |
| HUNTER_003 | 追踪术 | 无发现 — 白板 |
| HUNTER_007 | 鹰角弓 | 无 +1 耐久 |
| PRIEST_011 | 秘教暗影祭司 | 无控制 — 白板 |
| PRIEST_012 | 先知维伦 | 仅法术伤害翻倍，治疗不翻倍 |
| WARRIOR_009 | 血吼 | 无减攻机制 — 白板 7/1 |
| ROGUE_009 | 伺机待发 | 无下个法术减费 — 白板 |
| ROGUE_010 | 冷血 | 仅连击分支，无基础 +2 |
| LEGENDARY_010 | 奥妮克希亚 | 无战吼 — 白板 9/8 |
| LEGENDARY_011 | 死亡之翼 | 无弃手牌 |
| MAGE_007 | 水元素 | 无冻结受伤 — 白板 3/6 |
| PALADIN_006 | 真银圣剑 | 无攻击回血 — 白板 4/2 |
| PALADIN_016 | 阿古斯保护者 | 无战吼圣盾 — 白板 2/2（id 共享，见 F-A8） |
| ROGUE_014 | 毒刃 | 无效果 — 白板 |

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

**缺**（2026-08-06 刷新——§11 登记的 27 张债，按机制分组）：
1. 激怒接线（阿曼尼狂战士、暴怒的狼人、格罗玛什——`ThisMinionDamaged` 已存在，
   纯接线活，W0 先例）
2. 冲锋光环（战歌指挥官）
3. 治疗抽牌触发（北郡牧师）
4. 武器攻击特效（真银圣剑回血、血吼减攻、鹰角弓耐久）
5. 发现（追踪术）；抉择第二分支（愤怒、利爪德鲁伊、知识古树、战争古树）
6. 战吼（奥妮克希亚、阿古斯保护者、死亡之翼弃牌）；受伤冻结（水元素）；
   控制（秘教暗影祭司）；治疗翻倍（先知维伦）；毒刃的 1 伤+抽 1
7. 减费（海巨人、伺机待发）；连击基础分支（冷血）

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
