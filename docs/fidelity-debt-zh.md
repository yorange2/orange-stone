# 保真债 — 简化卡清单（F4/F5 持续审计账本）

> **现状：`src/cards/` 里有 35 处简化标记**（2026-08-06 审计，修复轮 PR #77；
> W0 接线轮 PR #79 清掉 13 张；W1 种族轮 PR #81 清掉 11 张；W2 触发轮 PR #82 清掉 8 张）。
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
> **2026-08-06 W1 种族轮（PR #81）**：11 张种族卡全部落地——`CardDef.race`
> 字段（Beast/Murloc/Demon，召唤时生效，`EntityView`/Python 绑定暴露 race）、
> 种族条件目标（`FriendlyRace` / `AllOtherFriendlyRace` / `AnyRace`）、种族条件
> 光环（`FriendlyRace` / `OtherFriendlyRace` 目标 + `GrantCharge` 冲锋光环，
> 苔原犀牛）、种族条件触发（`Trigger.race` 字段——鱼人招潮者 / 食腐土狼 /
> 饥饿的秃鹫）、按种族过滤牌库抽牌（感知恶魔）、硬编码 `BEAST_POOL` /
> `DEMON_POOL` 换成字段驱动池（含逐位一致测试，`w1_race_pools_are_field_driven`）。
> 对应 12 个差分场景（`tests/differential.rs` 的 `w1_*`）。
>
> **2026-08-06 W2 触发轮（PR #82）**：8 张触发/奥秘卡全部落地——5 个新触发类：
> `CharacterHealed`（治疗，任何角色被治疗都触发）、`Attacked`（实体攻击——
> 智慧祝福把"本随从攻击时抽牌"挂在目标随从身上）、`CardPlayed`（打出卡牌，
> 友方作用域）、`SecretPlayed`（奥秘打出，双方都触发）、`MinionDied`（任意
> 随从死亡，双方都触发）；以及 3 个摧毁奥秘效果（SI:7 随机一个、吞秘巨蟒 /
> 照明弹全部 + 组合增益/抽牌）。治疗触发只在"真的治疗到"时触发（满血角色
> 不是治疗事件）。对应 8 个差分场景（`tests/differential.rs` 的 `w2_*`）。
>
> **执行计划**：[docs/fidelity-debt-roadmap-zh.md](fidelity-debt-roadmap-zh.md)
> （英文版 `fidelity-debt-roadmap.md`）——按依赖排序的 8 个 wave（W0 接线 …
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
清理，见 §10）；真实债务是 67 张，**W0 已清 13 张、W1 已清 11 张、
W2 已清 8 张，剩 35 张**。

---

## 账本（按缺失机制分组）

### 1. Enrage — 受伤条件增益（4 张）✅ 已解决（W0，PR #79）

4 张全部接线完成：接上已有的 `ThisMinionDamaged` 触发槽（`apply_card_keywords`
按卡 ID 注册，与 Acolyte of Pain 同模式），增益为永久附魔。差分场景：
`w0_gurubashi_berserker_enrage_permanent`、`w0_tauren_warrior_enrage_with_taunt`、
`w0_angry_chicken_enrage_fires_before_death`、`w0_spiteful_smith_buffs_weapon_on_damage`。

### 2. 种族 — 野兽 / 鱼人 / 恶魔（9 张）✅ 已解决（W1，PR #81）

全部落地：`CardDef.race` 字段 + 种族条件目标/光环/触发 + 牌库过滤抽牌 +
字段驱动池（差分场景 `w1_*`，池一致性见 `w1_race_pools_are_field_driven`）。
F-A6 补注的 Starving Buzzard（HUNTER_013）与 Scavenging Hyena（HUNTER_014）
也随本轮离开账本。

### 3. 事件触发 — 召唤 / 治疗 / 死亡 / 奥秘 / 攻击 / 打出（2 张）

触发类已补全（W0 接线 + W2 触发轮）：治疗、攻击（目标身上）、打出卡牌、
奥秘打出、任意随从死亡全部落地；摧毁奥秘效果（随机一个 / 全部 + 组合）也已
落地。剩下两张缺的是非触发机制。（W2 已清：Flesheathing Ghoul、Lightwarden、
Secretkeeper、SI:7 Infiltrator、Eater of Secrets、Blessing of Wisdom、
Questing Adventurer；Flare 见 §9。）

| ID | 卡名 | 现状 | 真实炉石效果 | 缺失机制 |
| --- | --- | --- | --- | --- |
| NEUTRAL_R13 | Alarm-o-Bot | 白板 | 在你的回合开始时，与手牌中一个随机随从交换 | 手牌区交换效果（缺） |
| MAGE_017 | Ethereal Arcanist | 回合结束无条件 +2/+2 | 在你的回合结束时，若你控制一个奥秘，获得 +2/+2 | 条件回合结束（控制奥秘谓词） |

### 4. 条件目标与状态（9 张）

需要**目标/持有者条件谓词**（部分已有：`DamagedEnemyMinion`、`TauntEnemyMinion`；
缺：攻击区间、手牌数、英雄血量、圣盾计数、"生命设为 1"效果）。

| ID | 卡名 | 现状 | 真实炉石效果 | 缺失机制 |
| --- | --- | --- | --- | --- |
| NEUTRAL_R20 | Stampeding Kodo | 白板 | 战吼：摧毁一个攻击力 ≤2 的随机敌方随从 | 攻击 ≤N 谓词 + 随机选取 |
| NEUTRAL_E06 | Big Game Hunter | 白板 | 战吼：摧毁一个攻击力 ≥7 的随从 | 攻击 ≥N 谓词 |
| NEUTRAL_R19 | Twilight Drake | 白板 | 战吼：每有一张手牌便 +1 生命 | 手牌数谓词 |
| NEUTRAL_E05 | Blood Knight | 白板 | 战吼：吸收所有圣盾并获得 +3/+3 | 圣盾计数 + 群体吸收 |
| WARRIOR_021 | Mortal Strike | 造成 4 点伤害 | 造成 4 点伤害；若你 ≤12 生命则改为 6 | 持有者血量谓词 + 条件分支 |
| WARRIOR_023 | Rampage | 给任意友方随从 +3/+3 | 使一个**受伤**的随从 +3/+3 | 受伤谓词（现有只有敌方：`DamagedEnemyMinion`） |
| WARRIOR_022 | Battle Rage | 抽 2 张 | 每有一个受伤的友方角色便抽一张牌 | 受伤计数（双方） |
| PRIEST_018 | Mass Dispel | 沉默一个随机敌方随从 | 沉默所有敌方随从并抽一张牌 | `SilenceMinion`+`AllEnemyMinions` 已有——主要是接线；按 ID 不可达（见审计发现） |
| PALADIN_018 | Repentance | 奥秘：造成伤害 | 奥秘：对手打出随从时，将其生命设为 1 | "生命设为 1"效果（缺；`MinHealthUntilEndOfTurn` 只是临时的）；ID 也错了 |

### 5. 邻位 / 多目标（4 张）

`DestroyAdjacent` 已有；**缺：邻位增益、邻位冻结、交换攻击/生命**
（双随机目标伤害已有——`DealDamageToTwo`，Cleave 在用；Multi-Shot 已忠实但注释
过期，见 §10）。

| ID | 卡名 | 现状 | 真实炉石效果 | 缺失机制 |
| --- | --- | --- | --- | --- |
| MAGE_016 | Cone of Cold | 冻结一个随机敌方随从 | 冻结一个随从及其相邻随从 | 邻位冻结 |
| NEUTRAL_R11 | Sunfury Protector | 白板 | 战吼：使相邻随从获得嘲讽 | 邻位增益 |
| NEUTRAL_R18 | Ancient Mage | 白板 | 战吼：使相邻随从获得法术伤害 +1 | 邻位法强增益 |
| NEUTRAL_R08 | Crazed Alchemist | 白板 | 战吼：交换一个随从的攻击与生命 | 交换效果（缺；现有只有 Set/Double 变体） |

### 6. 费用与武器条件光环（8 张）

费用修正栈（G5）已有；**缺：全随从手牌费用光环、"每回合首个随从"状态、
武器装备时的条件冲锋、按武器攻击减费、武器耐久削减**。

| ID | 卡名 | 现状 | 真实炉石效果 | 缺失机制 |
| --- | --- | --- | --- | --- |
| NEUTRAL_R22 | Mana Wraith | 白板 | 所有随从的法力值消耗 +1 | 手牌费用光环（所有随从） |
| NEUTRAL_C14 | Venture Co. Mercenary | 白板 | 你的随从法力值消耗 +3 | 手牌费用光环（己方随从） |
| NEUTRAL_R24 | Pint-Sized Summoner | 白板 | 你每回合打出的第一个随从法力值消耗 −1 | "每回合首个随从"状态 |
| NEUTRAL_C07 | Southsea Deckhand | 永久冲锋 | 在你装备武器时具有冲锋 | 武器装备谓词（冲锋条件化） |
| NEUTRAL_C13 | Dread Corsair | 只有嘲讽 | 嘲讽；你的武器每有 1 点攻击力便减 1 费 | 按武器攻击减费 |
| NEUTRAL_C09 | Bloodsail Raider | 白板 | 战吼：获得与武器攻击力相等的攻击力 | 武器攻击谓词 |
| NEUTRAL_R03 | Bloodsail Corsair | 白板 | 战吼：从对手的武器上移除 1 点耐久度 | 武器耐久削减（只有 DestroyWeapon） |
| LEGENDARY_021 | Millhouse Manastorm | 无负面 | 战吼：对手下回合法术费用为 0 | 敌方法术 0 费效果（缺） |

### 7. 本回合临时增益（1 张）

| ID | 卡名 | 现状 | 真实炉石效果 | 缺失机制 |
| --- | --- | --- | --- | --- |
| NEUTRAL_R10 | Mana Addict | 白板 | 每当你施放一个法术，**本回合**获得 +2 攻击 | 回合结束自动清除的临时增益（引擎有 `TempDebuff`，缺临时增益） |

### 8. 概率效果（1 张）

| ID | 卡名 | 现状 | 真实炉石效果 | 缺失机制 |
| --- | --- | --- | --- | --- |
| LEGENDARY_022 | Nat Pagle | 回合结束必抽牌 | 你的回合结束时，50% 几率抽一张牌 | 概率效果 |

### 9. 复合与其他（8 张）

（W0 已清：Kul Tiran Chaplain、Emperor Cobra；W2 已清：Flare——摧毁所有
敌方奥秘并抽牌。）

| ID | 卡名 | 现状 | 真实炉石效果 | 缺失机制 |
| --- | --- | --- | --- | --- |
| DRUID_016 | Gift of the Wild | 仅 +2/+2 | 使你的随从获得 +2/+2 和嘲讽 | 双效果群体战吼（增益 + 嘲讽） |
| PALADIN_018 | Righteousness | 按注释不可实现 | 使你的随从获得圣盾 | 群体圣盾（无群体授予）；自身 ID 也错（见审计发现） |
| PALADIN_017 | Holy Wrath | 仅抽牌 | 抽一张牌，造成等同于其法力值消耗的伤害 | 抽牌-伤害链式效果 |
| SHAMAN_018 | Ancestral Healing | 治疗 + 嘲讽（部分） | 将一个随从恢复到满血并获得嘲讽 | 核对 `FullHeal` + 嘲讽接线 |
| PRIEST_018 | Lightwell | 回合结束恢复 3 | 在你的回合开始时，为一个受伤的友方角色恢复 3 点生命 | 受伤友方谓词 + 回合开始（另有 ID 冲突，见审计发现） |
| ROGUE_025 | Pilfer | 随机一张非潜行者卡 | 随机将一张其他职业的卡牌置入你的手牌 | 职业过滤（引擎无职业模型） |
| NEUTRAL_T21e | Ysera Awakens | 伤害包含 Ysera 自身 | 对所有**其他**角色造成 5 点伤害（梦境卡牌） | AllCharacters 排除自身 |
| NEUTRAL_R14 | Arcane Golem | 仅冲锋 | 冲锋；战吼：使你的对手获得一个法力水晶 | 给对手水晶效果 |

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
`SecretPlayed`、`MinionDied`（任意死亡）+ 摧毁奥秘效果（W2）。

**缺**（先补原语，遵守 Review II 的"先 G 后 F4/F5"纪律）：
1. 目标谓词：攻击区间、手牌数、英雄血量、受伤友方、控制奥秘、武器装备；
   "每回合首个随从"状态——§4、§6
2. 效果：生命设为 1、交换攻击/生命、群体圣盾、敌方法术 0 费、
   武器耐久削减、邻位增益/冻结、双随机伤害、本回合临时增益、
   概率效果——§5、§6、§7、§8、§9

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
