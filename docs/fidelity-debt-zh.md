# 保真债 — 简化卡清单（F4/F5 持续审计账本）

> **现状：`src/cards/` 里有 67 处简化标记**（2026-08-06 审计，修复轮 PR #77）。
> 本账本是 F4 逐效果保真审计的**权威记录**。一张卡**离开账本**的唯一条件：
> 真实炉石效果已实现**且**通过 F5 差分测试验证。不要静默重写卡牌——改动必须
> 同时更新本账本、代码注释和下游简化债提取器（见[维护约定](#维护约定)）。
>
> **2026-08-06 修复轮（PR #77）**：下面的结构性发现（F-A1…F-A7）已全部解决——
> 4 处过期注释清理、Worgen Infiltrator 修复、3 处卡 ID 冲突修复、10 张卡补入
> `ALL_CARDS`（7 个重复条目去重）、Python 提取器重写（PR #31）。剩下的是
> 下面各组的逐机制实现工作与 F5 验证协议。
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
清理，见 §10）；真实债务是 67 张。

---

## 账本（按缺失机制分组）

### 1. Enrage — 受伤条件增益（4 张）

需要**受伤触发的永久增益**（引擎没有受伤触发类；现有触发槽只有
`spell_trigger`/`summon_trigger`/`death_trigger`/`start|end_turn_effect`）。

| ID | 卡名 | 现状 | 真实炉石效果 | 缺失机制 |
| --- | --- | --- | --- | --- |
| NEUTRAL_B19 | Gurubashi Berserker | 白板 5/2/8 | 每当此随从受到伤害，获得 +3 攻击 | 受伤触发 + 永久增益 |
| NEUTRAL_C11 | Tauren Warrior | 只有嘲讽 | 嘲讽；**激怒（Enrage）：**+3 攻击 | 同上 + 受伤条件 |
| NEUTRAL_C15 | Spiteful Smith | 白板 4/6 | **激怒：**你的武器 +2 攻击 | 受伤触发 + 武器光环 |
| NEUTRAL_R02 | Angry Chicken | 白板 1/1 | **激怒：**+5 攻击 | 受伤触发 + 永久增益 |

### 2. 种族 — 野兽 / 鱼人 / 恶魔（9 张）

需要 **`CardDef` 上的种族（tribe/race）字段**（目前没有；`RandomPool::Beast/Demon`
是硬编码 ID 列表），以及种族条件的目标与光环。

| ID | 卡名 | 现状 | 真实炉石效果 | 缺失机制 |
| --- | --- | --- | --- | --- |
| HUNTER_015 | Houndmaster | 战吼：给任意友方随从 +2/+2 | 战吼：使一个友方**野兽** +2/+2 并获得嘲讽 | 种族谓词 + 双效果战吼 |
| HUNTER_016 | Tundra Rhino | 冲锋光环（所有友方） | 你的**野兽**获得冲锋 | 种族条件光环 |
| NEUTRAL_R01 | Coldlight Seer | 战吼：所有友方随从 +2 生命 | 战吼：使所有其他**鱼人** +2 生命 | 种族谓词 |
| NEUTRAL_E02 | Murloc Warleader | 光环：其他友方随从 +2/+1 | 你的其他**鱼人**获得 +2/+1 | 种族条件光环 |
| NEUTRAL_R05 | Murloc Tidecaller | 白板 | 每当你召唤一个**鱼人**，获得 +1 攻击 | 种族谓词 + 召唤触发 |
| NEUTRAL_E03 | Hungry Crab | 白板 | 战吼：摧毁一个**鱼人**并获得 +2/+2 | 种族谓词 + 摧毁增益组合 |
| WARLOCK_020 | Sense Demons | 抽 1 张 | 从牌库抽两张**恶魔** | 种族谓词 + 牌库过滤抽牌 |
| WARLOCK_021 | Demonfire | 造成 2 点伤害 | 对一个随从造成 2 点伤害；若为友方**恶魔**则改为 +2/+2 | 种族谓词 + 条件分支 |
| WARLOCK_T01 | Siegebreaker | 只有嘲讽 | 嘲讽；你的其他**恶魔** +1 攻击 | 种族条件光环 |

### 3. 事件触发 — 召唤 / 治疗 / 死亡 / 奥秘 / 攻击 / 打出（16 张）

`summon_trigger`、`spell_trigger`、`death_trigger` 已存在且被其他卡使用；
**缺：治疗触发、攻击触发、打出卡牌触发、奥秘打出触发**，以及"摧毁奥秘"效果
（奥秘相关卡需要）。

| ID | 卡名 | 现状 | 真实炉石效果 | 缺失机制 |
| --- | --- | --- | --- | --- |
| NEUTRAL_R09 | Knife Juggler | 白板 | 在你召唤一个随从后，对一个随机敌人造成 1 点伤害 | 召唤触发 + 随机目标伤害（机制已有，卡未接线） |
| PALADIN_017 | Sword of Justice | summon_trigger 增益 Self_（触发目标绑定待核） | 每当你召唤一个随从，使其获得 +1/+1 | 核对触发目标绑定；ID 也错了（见审计发现） |
| NEUTRAL_C12 | Flesheathing Ghoul | 白板 | 每当一个随从死亡，获得 +1 攻击 | 死亡触发 + 自身增益（机制已有） |
| NEUTRAL_R04 | Lightwarden | 白板 | 每当一个角色被治疗，获得 +2 攻击 | 治疗触发（缺） |
| NEUTRAL_R06 | Secretkeeper | 白板 | 每当一个**奥秘**被打出，获得 +1/+1 | 奥秘打出触发（缺） |
| NEUTRAL_R25 | SI:7 Infiltrator | 白板 | 战吼：摧毁一个随机的敌方奥秘 | 摧毁奥秘效果（缺） |
| NEUTRAL_R26 | Eater of Secrets | 白板 | 战吼：摧毁所有敌方奥秘并获得 +1/+1 | 摧毁奥秘效果 + 增益 |
| PALADIN_019 | Blessing of Wisdom | 无效果 | 每当目标随从攻击，抽一张牌 | 目标上的攻击触发（缺）+ 实体光环 |
| NEUTRAL_R15 | Demolisher | 白板 | 在你的回合开始时，对一个随机敌人造成 2 点伤害 | 回合开始触发已有；随机目标伤害 |
| NEUTRAL_E04 | Doomsayer | 白板 | 在你的回合开始时，摧毁所有随从 | 回合开始触发 + DestroyMinion(AllMinions) |
| NEUTRAL_R17 | Questing Adventurer | 白板 | 每当你打出一张牌，获得 +1/+1 | 打出卡牌触发（缺） |
| NEUTRAL_R13 | Alarm-o-Bot | 白板 | 在你的回合开始时，与手牌中一个随机随从交换 | 手牌区交换效果（缺） |
| NEUTRAL_R12 | Wild Pyromancer | 白板 | 在你施放一个法术后，对所有随从造成 1 点伤害 | 法术触发 + 全场伤害（机制已有，卡未接线） |
| NEUTRAL_R21 | Young Priestess | 白板 | 在你的回合结束时，使另一个随机友方随从 +1 生命 | 回合结束触发已有；随机友方目标 |
| NEUTRAL_R23 | Master Swordsmith | 白板 | 在你的回合结束时，使另一个随机友方随从 +1 攻击 | 同上 |
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

### 9. 复合与其他（11 张）

| ID | 卡名 | 现状 | 真实炉石效果 | 缺失机制 |
| --- | --- | --- | --- | --- |
| DRUID_016 | Gift of the Wild | 仅 +2/+2 | 使你的随从获得 +2/+2 和嘲讽 | 双效果群体战吼（增益 + 嘲讽） |
| PALADIN_018 | Righteousness | 按注释不可实现 | 使你的随从获得圣盾 | 群体圣盾（无群体授予）；自身 ID 也错（见审计发现） |
| PALADIN_017 | Holy Wrath | 仅抽牌 | 抽一张牌，造成等同于其法力值消耗的伤害 | 抽牌-伤害链式效果 |
| SHAMAN_018 | Ancestral Healing | 治疗 + 嘲讽（部分） | 将一个随从恢复到满血并获得嘲讽 | 核对 `FullHeal` + 嘲讽接线 |
| PRIEST_017 | Kul Tiran Chaplain | 增益自己 | 战吼：使一个友方随从 +2 生命 | 双效果战吼（别处已有——接线） |
| PRIEST_018 | Lightwell | 回合结束恢复 3 | 在你的回合开始时，为一个受伤的友方角色恢复 3 点生命 | 受伤友方谓词 + 回合开始（另有 ID 冲突，见审计发现） |
| ROGUE_025 | Pilfer | 随机一张非潜行者卡 | 随机将一张其他职业的卡牌置入你的手牌 | 职业过滤（引擎无职业模型） |
| NEUTRAL_T21e | Ysera Awakens | 伤害包含 Ysera 自身 | 对所有**其他**角色造成 5 点伤害（梦境卡牌） | AllCharacters 排除自身 |
| HUNTER_017 | Flare | 仅抽牌 | 摧毁所有敌方奥秘并抽一张牌 | 摧毁奥秘效果（同 SI:7） |
| NEUTRAL_R14 | Arcane Golem | 仅冲锋 | 冲锋；战吼：使你的对手获得一个法力水晶 | 给对手水晶效果 |
| NEUTRAL_R16 | Emperor Cobra | 白板 | **剧毒**（受到其伤害的随从被摧毁） | 剧毒机制（完全没有） |

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
潜行 / 扰咒 / 奥秘。

**缺**（先补原语，遵守 Review II 的"先 G 后 F4/F5"纪律）：
1. `CardDef` 种族（tribe/race）字段——解锁 §2 全部
2. 受伤触发效果（Enrage）——§1
3. 触发类：治疗、攻击、打出卡牌、奥秘打出——§3
4. 目标谓词：攻击区间、手牌数、英雄血量、受伤友方、控制奥秘、武器装备；
   "每回合首个随从"状态——§4、§6
5. 效果：生命设为 1、交换攻击/生命、剧毒、群体圣盾、敌方法术 0 费、
   摧毁奥秘、武器耐久削减、邻位增益/冻结、双随机伤害、本回合临时增益、
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
