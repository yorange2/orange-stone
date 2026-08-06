# 保真债实现路线图 — 67 张简化卡

> **状态：W3 完成（PR #82）；W4 待做。** 本路线图执行 [architecture-roadmap.md](architecture-roadmap-zh.md)
> 的 F4 持续 / F5 持续条目。[fidelity-debt.md](fidelity-debt-zh.md) 账本是卡名单的
> 权威来源，本文档是执行计划。一张卡只有在"实现 **且** 通过 F5 差分测试验证"后
> 才离开账本——见账本的 [F5 验收](fidelity-debt-zh.md#每张卡的-f5-验收) 与
> [维护约定](fidelity-debt-zh.md#维护约定)。
>
> 2026-08-06 对照引擎核实：67 处标记、触发作用域语义、下面的机制盘点截至
> PR #77 全部为最新。W0（13 张）已在 PR #79 落地，盘点已更新 W0 原语。

## 原则

1. **先原语后卡面**（Review II 纪律）：卡牌 wave 只在其所需机制存在且被测试后
   才出发。
2. **接线优先**：机制已存在的卡（W0）先做——清卡池最快，还能顺带压力测试现有
   触发/光环机制。
3. **每卡一个差分场景**（`tests/differential.rs`）：目标集合、触发时机、死亡
   阶段交互；语义可镜像的加 SabberStone 牌组级对照（`docs/differential_sabberstone.md`）。
4. **卡池走账本**：修好一张卡 = 删账本行 + 删代码里的 `(simplified…)` 注释 +
   使 `~/.cache/orange_stone_debt_ids.txt` 失效——RL 卡池自动回补。
5. 每个 wave 一个 PR；每个 wave 必须保持 `cargo test` 全绿、RL 卡池检查
   （`hearthstone_os` 测试 + M5 冒烟）通过。

## 机制盘点（截至 2026-08-06）

**已存在**（代码核实）：

| 机制 | 位置 | 在用卡 |
| --- | --- | --- |
| `summon_trigger` → `FriendlyMinionSummoned`（仅己方） | `trigger.rs:737` | 饥饿的秃鹫、正义之剑 |
| `spell_trigger` → `FriendlySpellCast`（仅己方） | `trigger.rs:717` | 法力浮龙、巫师学徒 |
| `death_trigger` → `FriendlyMinionDied`（**仅己方随从**） | `trigger.rs:727` | 教派暗影牧师 |
| `start_turn_effect` → `TurnStart` | `trigger.rs:709` | 炎魔之王、末日预言者类 |
| `end_turn_effect` → `TurnEnd` | — | 消失类、光明之泉类 |
| `ThisMinionDamaged`（自身受伤） | `cards/mod.rs` 关键词映射 | 苦痛侍僧 |
| `FriendlyMinionDamaged` | `cards/mod.rs` 关键词映射 | 狂暴的狼人、铸甲师 |
| `BuffWeapon`（攻击+耐久） | `effect.rs` | 升级类效果 |
| `Poison` 组件，按卡 ID 映射 | `cards/mod.rs:136` | 耐心的刺客、帝王眼镜蛇（W0） |
| `FreezeCharacter`、`FullHeal`、`SilenceMinion`+`AllEnemyMinions`、`DealDamageToTwo`、`DestroyAdjacent`、`GrantCharge`、`GrantWindfury` | `effect.rs` | 顺劈斩等 |
| 费用修正栈（G5）含手牌区减费 | `engine::cost` | 冰霜巨人级、海盗减费类 |
| `DealDamageAndDraw`、`SummonMinion`、`ReturnToHand`+… | `effect.rs` | — |
| `ThisMinionDamaged` → 激怒增益（W0） | `cards/mod.rs` 关键词映射 | 4 张激怒卡 |
| `EffectTarget::EventSubject` / `OtherFriendlyMinion`（W0） | `core/effect.rs` | 正义之剑；女祭司 / 武器锻造师 |
| 武器触发注册 + 摧毁后离场（W0） | `trigger.rs` / `rules.rs` | 正义之剑 |
| 法术死亡先于施法后触发（W0） | `rules.rs` SpellCast | 狂野炎术师 |
| `CardDef.race` 字段 + 种族目标/光环/触发（W1） | `def.rs` / `core/effect.rs` / `core/component.rs` | 11 张种族卡 |
| 字段驱动种族池（W1） | `cards/pool.rs` | Barrens Stablehand 等 |
| 触发类补全（W2）：`CharacterHealed` / `Attacked` / `CardPlayed` / `SecretPlayed` / `MinionDied`（任意）+ 摧毁奥秘 | `core/component.rs` / `rules.rs` / `trigger.rs` | 光明之泉侍女、智慧祝福、任务达人、奥秘守护者、食尸鬼、SI:7、吞秘巨蟒、照明弹 |
| 条件谓词（W3）：攻击区间 / 手牌数 / 英雄血量 / 受伤 / 控制奥秘 / 首个随从 / 圣盾吸收 | `core/effect.rs` / `trigger.rs` / `engine/cost.rs` | 科多兽、猎潮驯兽师、暮光幼龙、致死打击、狂暴、战斗怒火、以太奥秘学者、微型召唤师、血骑士 |

**缺失**（按 wave 分组的原语）：

1. 手牌区费用光环（全体随从）、按武器攻击减费、武器耐久削减、敌方法术 0 费、
   给对手水晶 → W4
5. 生命设为 1、交换攻击/生命、邻位增益/冻结、双效果组合（含授嘲讽）→ W5
6. 概率、本回合临时增益、群体圣盾、排除自身的全场伤害、抽牌-按费用伤害、
   职业过滤 → W6
7. 手牌区交换、伤害反射奥秘、1 生命复活奥秘 → W7

---

## Wave 0 — 接线卡：机制已存在（13 张）✅ 完成（PR #79）

预期无需新原语。数张卡带 **先核实** 标记——接线前必须确认既有机制的精确语义；
核实可能暴露出小修（本 wave 可接受）。

| # | ID | 卡名 | 接上的机制 | 差分场景 |
| --- | --- | --- | --- | --- |
| 1 | NEUTRAL_R09 | 飞刀杂耍者 | `summon_trigger` + `DealDamage{AnyEnemy}` | `w0_knife_juggler_throws_after_friendly_summon` |
| 2 | HUNTER_012 | 狂野炎术师 | `spell_trigger` + `DealDamage{AllMinions}` | `w0_wild_pyromancer_aoe_after_spell` + `…_killed_by_the_spell` |
| 3 | NEUTRAL_R15 | 攻城车 | `start_turn_effect` + `DealDamage{AnyEnemy}` | `w0_demolisher_fires_at_turn_start` |
| 4 | NEUTRAL_E04 | 末日预言者 | `start_turn_effect` + `DestroyMinion{AllMinions}` | `w0_doomsayer_destroys_all_minions_including_itself` |
| 5 | EX1_365 | 正义之剑 | `summon_trigger` → 新 `EffectTarget::EventSubject` | `w0_sword_of_justice_buffs_the_summoned_minion` + `…_stops_firing_when_destroyed` |
| 6 | PRIEST_017 | 库尔提拉斯牧师 | 战吼 `GainStats{FriendlyMinion}` | `w0_kul_tiran_chaplain_buffs_a_friendly_minion` |
| 7 | NEUTRAL_R21 | 年轻的女祭司 | `end_turn_effect` → 新 `EffectTarget::OtherFriendlyMinion` | `w0_young_priestess_buffs_another_minion` + `…_alone_does_nothing` |
| 8 | NEUTRAL_R23 | 武器锻造师 | 同上 | `w0_master_swordsmith_buffs_another_minion` |
| 9 | NEUTRAL_B19 | 古拉巴什狂暴者 | `ThisMinionDamaged` + `GainStats{Self_}` | `w0_gurubashi_berserker_enrage_permanent` |
| 10 | NEUTRAL_C11 | 牛头人战士 | 嘲讽 + `ThisMinionDamaged` + `GainStats{Self_}` | `w0_tauren_warrior_enrage_with_taunt` |
| 11 | NEUTRAL_R02 | 愤怒的小鸡 | `ThisMinionDamaged` + `GainStats{Self_}` | `w0_angry_chicken_enrage_fires_before_death` |
| 12 | NEUTRAL_C15 | 恶毒铁匠 | `ThisMinionDamaged` + `BuffWeapon{攻+2}` | `w0_spiteful_smith_buffs_weapon_on_damage` |
| 13 | NEUTRAL_R16 | 帝王眼镜蛇 | ID 加进 `apply_card_keywords` 剧毒映射 | `w0_emperor_cobra_poison_kills_and_divine_shield_absorbs` |

**W0 核实发现**（接线暴露的小引擎修复，均在 PR #79）：
- 正义之剑核实了触发上下文问题：武器实体在装备时注册 CardDef 触发（从手牌
  打出的武器本来就有），且被摧毁的武器离开战场——断剑不再触发。
- `Self_` 绑定对"增益被召唤的随从"是错的；新 `EffectTarget::EventSubject`
  直接解析触发事件的主体（已离场的主体是 no-op）。
- 女祭司/武器锻造师的"另一个"需要真正的 `OtherFriendlyMinion` 目标
  （候选集排除来源）。
- 死亡与施法触发的时序：法术事件现在先结算法术造成的待定死亡，再触发
  `FriendlySpellCast`（HS 语义——被自己的法术杀死的野炎术师不再触发）。

**验收**：16 个差分场景（13 张卡，剑/炎术师/女祭司各两个）；4 张激怒卡的
伤害时序（每次伤害事件触发一次、增益永久）；RL 卡池 +13。

## Wave 1 — 种族字段（11 张）✅ 完成（PR #80）

**原语**（一个 PR 的量）——全部落地：
- `CardDef.race: Option<Race>`（`Beast` / `Murloc` / `Demon`），召唤时生效；
  结构化视图（`EntityView`）/ Python 绑定暴露 race 供 RL 侧用。
- 种族条件目标：`EffectTarget::FriendlyRace`（驯犬者）、`AnyRace`（饥饿的
  螃蟹）、`AllOtherFriendlyRace`（寒光先知）。
- 种族条件光环：`AuraTarget::FriendlyRace`（苔原犀牛——冲锋光环
  `AuraEffect::GrantCharge`）与 `OtherFriendlyRace`（鱼人领军 / 攻城恶魔）。
- 种族条件触发：`Trigger.race` 字段（鱼人招潮者 / 食腐土狼 / 饥饿的秃鹫，
  `apply_card_keywords` 按卡 ID 注册）。
- 按种族过滤牌库抽牌（感知恶魔——`CardEffect::DrawCardByRace`）。
- 硬编码 `BEAST_POOL` / `DEMON_POOL` 换成字段驱动池（池一致性测试
  `w1_race_pools_are_field_driven`：旧表成员全部保留，新增的正是旧表漏掉的
  7 张野兽 / 攻城恶魔）。

| ID | 卡名 | 真实效果 | 场景 |
| --- | --- | --- | --- |
| HUNTER_015 | 驯犬者 | 战吼：使一个友方**野兽** +2/+2 并获得嘲讽 | `w1_houndmaster_buffs_only_a_friendly_beast` |
| HUNTER_016 | 苔原犀牛 | 你的**野兽**获得冲锋 | `w1_tundra_rhino_gives_beasts_charge` |
| NEUTRAL_R01 | 寒光先知 | 战吼：使所有其他**鱼人** +2 生命 | `w1_coldlight_seer_buffs_other_murlocs_only` |
| NEUTRAL_E02 | 鱼人领军 | 你的其他**鱼人** +2/+1 | `w1_murloc_warleader_aura_murloc_only` |
| NEUTRAL_R05 | 鱼人招潮者 | 每当你召唤一个**鱼人**，+1 攻击 | `w1_murloc_tidecaller_gains_attack_on_murloc_summon` |
| NEUTRAL_E03 | 饥饿的螃蟹 | 战吼：摧毁一个**鱼人**，+2/+2 | `w1_hungry_crab_destroys_enemy_murloc_and_buffs` |
| WARLOCK_020 | 感知恶魔 | 从牌库抽两张**恶魔** | `w1_sense_demons_draws_two_demons_from_deck` |
| WARLOCK_021 | 恶魔之火 | 2 点伤害；友方**恶魔**改为 +2/+2 | `w1_demonfire_buffs_friendly_demon_and_damages_others` |
| WARLOCK_T01 | 攻城恶魔 | 嘲讽；你的其他**恶魔** +1 攻击 | `w1_siegebreaker_buffs_other_demons` |
| HUNTER_013 | 食腐土狼 | 每当一个友方**野兽**死亡，+2/+1 | `w1_scavenging_hyena_only_counts_beast_deaths` |
| HUNTER_014 | 饥饿的秃鹫 | 每当你召唤一个**野兽**，抽一张牌 | `w1_starving_buzzard_draws_on_beast_summon` |

**验收**：12 个差分场景（11 张卡 + 池一致性测试）；种族池与硬编码列表逐位一致
（旧成员全保留，新增 = 旧表漏掉的真野兽/恶魔）；RL 卡池 +11（335 → 346）。

## Wave 2 — 触发类补全（8 张）✅ 完成（PR #81）

**原语**——全部落地：
- `TriggerEvent::CharacterHealed`——治疗触发（光明之泉侍女），任何角色被
  治疗都触发；只在**真的治疗到**时触发（满血角色不是治疗事件）。
- `TriggerEvent::Attacked`——实体攻击触发（智慧祝福把"本随从攻击时抽牌"
  挂在目标随从身上，`CardEffect::AttachAttackDraw`）。
- `TriggerEvent::CardPlayed`——打出卡牌触发（任务达人，友方作用域）。
- `TriggerEvent::SecretPlayed`——奥秘打出触发（奥秘守护者，双方都触发）。
- `TriggerEvent::MinionDied`——任意随从死亡变体（食尸鬼，双方都触发）。
- 摧毁奥秘效果：`DestroyRandomEnemySecret`（SI:7 潜行者）、
  `DestroyAllEnemySecretsAndGainStats`（吞秘巨蟒）、
  `DestroyAllEnemySecretsAndDraw`（照明弹）。

| ID | 卡名 | 真实效果 | 场景 |
| --- | --- | --- | --- |
| NEUTRAL_R04 | 光明之泉侍女 | 每当一个角色被治疗，+2 攻击 | `w2_lightwarden_gains_attack_on_real_heals` |
| PALADIN_019 | 智慧祝福 | 每当目标随从攻击，抽一张牌 | `w2_blessing_of_wisdom_draws_on_attacks` |
| NEUTRAL_R17 | 任务达人 | 每当你打出一张牌，+1/+1 | `w2_questing_adventurer_grows_per_played_card` |
| NEUTRAL_R06 | 奥秘守护者 | 每当一个奥秘被打出，+1/+1 | `w2_secretkeeper_grows_on_any_secret` |
| NEUTRAL_R25 | SI:7 潜行者 | 战吼：摧毁一个随机的敌方奥秘 | `w2_si7_destroys_one_enemy_secret` |
| NEUTRAL_R26 | 吞秘巨蟒 | 战吼：摧毁所有敌方奥秘，+1/+1 | `w2_eater_of_secrets_destroys_all_and_buffs` |
| HUNTER_017 | 照明弹 | 摧毁所有敌方奥秘并抽一张牌 | `w2_flare_destroys_all_secrets_and_draws` |
| NEUTRAL_C12 | 食尸鬼 | 每当**任意**随从死亡，+1 攻击 | `w2_flesheathing_ghoul_counts_every_death` |

**验收**：8 个差分场景（逐卡核实 after/whenever 时机）；RL 卡池 +8（346 → 354）。
## Wave 3 — 条件谓词（9 张）✅ 完成（PR #82）

**原语**——全部落地：
- 攻击区间目标：`EffectTarget::EnemyMinionAttackLE`（狂奔科多兽 ≤2）、
  `AnyMinionAttackGE`（猎潮驯兽师 ≥7，双方）。
- 手牌数计数：`CardEffect::GainStatsPerHandCard`（暮光幼龙）。
- 英雄血量阈值：`CardEffect::MortalStrike`（≤12 生命时 6 伤）。
- 受伤目标：`DamagedFriendlyMinion` / `DamagedMinion`（狂暴，双方）。
- 受伤计数：`CardEffect::DrawPerDamagedFriendlyCharacter`（战斗怒火，
  英雄 + 随从都算）。
- 控制奥秘：`CardEffect::GainStatsIfOwnSecret`（以太奥秘学者）。
- "每回合首个随从"状态：`AuraEffect::FirstMinionDiscount` + 每玩家
  `minions_played_this_turn` 计数器（微型召唤师；静默召唤师会移除光环）。
- 圣盾吸收：`CardEffect::AbsorbDivineShields`（血骑士，每盾 +3/+3，
  双方圣盾都吸收）。

| ID | 卡名 | 真实效果 | 场景 |
| --- | --- | --- | --- |
| NEUTRAL_R20 | 狂奔科多兽 | 摧毁一个攻击力 ≤2 的随机敌方随从 | `w3_stampeding_kodo_destroys_low_attack_minion` |
| NEUTRAL_E06 | 猎潮驯兽师 | 摧毁一个攻击力 ≥7 的随从 | `w3_big_game_hunter_destroys_high_attack_minion` |
| NEUTRAL_R19 | 暮光幼龙 | 每有一张手牌便 +1 生命 | `w3_twilight_drake_gains_health_per_hand_card` |
| WARRIOR_021 | 致死打击 | 4 点伤害；若你 ≤12 生命则 6 点 | `w3_mortal_strike_boosts_at_low_health` |
| WARRIOR_023 | 狂暴 | 使一个**受伤**的随从 +3/+3 | `w3_rampage_targets_only_damaged_minions` |
| WARRIOR_022 | 战斗怒火 | 每有一个受伤的友方角色抽一张牌 | `w3_battle_rage_draws_per_damaged_friendly_character` |
| MAGE_017 | 以太奥秘学者 | 回合结束：若你控制一个奥秘，+2/+2 | `w3_ethereal_arcanist_requires_a_secret` |
| NEUTRAL_R24 | 微型召唤师 | 每回合第一个随从费用 −1 | `w3_pint_sized_summoner_discounts_first_minion` |
| NEUTRAL_E05 | 血骑士 | 吸收所有圣盾，+3/+3 | `w3_blood_knight_absorbs_all_divine_shields` |

**验收**：9 个差分场景；RL 卡池 +9（354 → 363）。
## Wave 4 — 费用与武器互动（8 张）

**原语**：
- 手牌区费用光环（"所有随从"/"你的随从"，叠在 G5 修正栈上）、按武器攻击减费、
  武器耐久削减、武器装备谓词（条件冲锋）、敌方法术 0 费、给对手水晶。

| ID | 卡名 | 真实效果 |
| --- | --- | --- |
| NEUTRAL_R22 | 法力怨魂 | 所有随从的法力值消耗 +1 |
| NEUTRAL_C14 | 风险投资公司雇佣兵 | 你的随从法力值消耗 +3 |
| NEUTRAL_C07 | 南海船工 | 装备武器时具有冲锋 |
| NEUTRAL_C13 | 恐怖海盗 | 嘲讽；武器每有 1 攻击力减 1 费 |
| NEUTRAL_C09 | 血帆袭击者 | 战吼：获得与武器攻击力相等的攻击力 |
| NEUTRAL_R03 | 血帆海盗 | 战吼：从对手武器移除 1 点耐久 |
| LEGENDARY_021 | 米米尔隆的头部 | 敌方法术下回合费用为 0 |
| NEUTRAL_R14 | 奥术傀儡 | 冲锋；对手获得一个法力水晶 |

**验收**：8 个差分场景（含与既有光环的费用修正叠加）；RL 卡池 +8。

## Wave 5 — 目标结构与效果组合（7 张）

**原语**：
- `SetHealthTo` 效果（忏悔）、交换攻击/生命效果（疯狂炼金师）、邻位增益/冻结
  目标、双效果组合（`SilenceAllAndDraw`、`FullHealAndTaunt`——或通用链式）、
  `GrantTaunt`。

| ID | 卡名 | 真实效果 |
| --- | --- | --- |
| EX1_349 | 忏悔 | 奥秘：对手随从的生命设为 1 |
| PRIEST_018 | 群体驱散 | 沉默所有敌方随从并抽一张牌 |
| NEUTRAL_R08 | 疯狂炼金师 | 交换一个随从的攻击与生命 |
| MAGE_016 | 冰霜新星锥 | 冻结一个随从及其相邻随从 |
| NEUTRAL_R11 | 日怒保卫者 | 使相邻随从获得嘲讽 |
| NEUTRAL_R18 | 远古法师 | 使相邻随从获得法术伤害 +1 |
| SHAMAN_018 | 先祖治疗 | 将一个随从恢复至满血并获得嘲讽 |

**验收**：7 个差分场景；RL 卡池 +7。

## Wave 6 — 特殊机制（8 张）

**原语**：概率效果（纳特·帕格）、本回合临时增益（法力沸腾者；G4 附魔层已有
`UntilEndOfTurn`）、群体圣盾、排除自身的全场伤害（AllOtherCharacters）、
抽牌-按费用伤害（变数伤害）、随机卡池职业过滤（顺手牵羊）。

| ID | 卡名 | 真实效果 |
| --- | --- | --- |
| LEGENDARY_022 | 纳特·帕格 | 回合结束：50% 几率抽一张牌 |
| NEUTRAL_R10 | 法力沸腾者 | 施法后：本回合 +2 攻击 |
| PALADIN_018 | 正义 | 使你的随从获得圣盾 |
| NEUTRAL_T21e | 伊瑟拉之醒 | 对所有**其他**角色造成 5 点伤害 |
| DRUID_016 | 自然之力 | 你的随从 +2/+2 并获得嘲讽 |
| PALADIN_017 | 神圣愤怒 | 抽一张牌；造成等同于其费用值的伤害 |
| EX1_341 | 光明之泉 | 回合开始：为一个受伤的友方角色恢复 3 点生命 |
| ROGUE_025 | 顺手牵羊 | 随机将一张其他职业的卡牌置入你的手牌 |

**验收**：8 个差分场景；RL 卡池 +8。

## Wave 7 — 收尾：复杂遗留（3 张）

**原语**：手牌区交换（闹钟机器人）、伤害反射奥秘（以眼还眼）、1 生命复活死亡
奥秘（救赎）。

| ID | 卡名 | 真实效果 |
| --- | --- | --- |
| NEUTRAL_R13 | 闹钟机器人 | 回合开始：与手牌中一个随机随从交换 |
| PALADIN_020 | 以眼还眼 | 奥秘：你受到伤害时对敌方英雄造成等量伤害 |
| PALADIN_021 | 救赎 | 奥秘：一个友方随从死亡时以 1 生命复活 |

**验收**：3 个差分场景；账本清空；RL 卡池达到全经典构筑池满规模；终扫 +
完整 SabberStone 对照。

---

## 每 wave 的横向任务

- 使 `~/.cache/orange_stone_debt_ids.txt` 失效；重跑卡池检查
  （`hearthstone_os` 测试 + `tools/orange_stone_m5_smoke.py`）。
- 更新账本：删已修行、删代码注释、记录差分场景编号。
- 热路径改动（触发触发、光环求值）跑 `cargo bench`——每个 wave 的原语不得让
  批量吞吐回退（M4 基线 ~4,200 局/s）。

## Wave 账目

| Wave | 卡数 | 新原语 | 卡池增量 |
| --- | --- | --- | --- |
| W0 接线 ✅ PR #79 | 13 | `EventSubject` / `OtherFriendlyMinion` 目标；武器触发注册 + 摧毁后离场；法术死亡先于施法后触发 | +13 → **334** |
| W1 种族 ✅ PR #80 | 11 | 种族字段 + 目标/光环/触发 + 字段驱动池 | +11 → **346** |
| W2 触发 ✅ PR #81 | 8 | 5 个触发类 + 摧毁奥秘 | +8 → **354** |
| W3 谓词 ✅ PR #82 | 9 | 攻击区间/手牌数/血量/受伤/奥秘/首个随从/圣盾 谓词 | +9 → **363** |
| W4 费用/武器 | 8 | 6+ 原语 | +8 |
| W5 目标结构 | 7 | 5+ 原语 | +7 |
| W6 特殊机制 | 8 | 6 个原语 | +8 |
| W7 收尾 | 3 | 3 个原语 | +3 |
| **合计** | **67** | | **321 → 388** |
