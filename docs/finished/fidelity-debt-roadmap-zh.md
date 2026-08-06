# 保真债实现路线图 — 67 张简化卡

> **状态：全部完成（W7 收尾 PR #86）。** 本路线图执行 [architecture-roadmap.md](../architecture-roadmap-zh.md)
> 的 F4 持续 / F5 持续条目。[fidelity-debt.md](../fidelity-debt-zh.md) 账本是卡名单的
> 权威来源，本文档是执行计划。一张卡只有在"实现 **且** 通过 F5 差分测试验证"后
> 才离开账本——见账本的 [F5 验收](../fidelity-debt-zh.md#每张卡的-f5-验收) 与
> [维护约定](../fidelity-debt-zh.md#维护约定)。
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
| 费用/武器（W4）：手牌区费用光环 / 按武器攻击减费 / 武器耐久削减 / `ChargeWithWeapon` / 敌方法术 0 费 / 给对手水晶 | `core/component.rs` / `core/world.rs` / `engine/cost.rs` / `trigger.rs` | 法力怨魂、雇佣兵、船工、恐怖海盗、血帆袭击者、血帆海盗、米米尔隆、奥术傀儡 |
| 目标结构/组合（W5）：生命设为 1 / 交换攻击生命 / 邻位增益冻结 / 双效果组合 | `core/effect.rs` / `trigger.rs` / `secret.rs` | 忏悔、群体驱散、疯狂炼金师、冰锥术、日怒保卫者、远古法师、先祖治疗 |
| 特殊机制（W6）：概率 / 本回合临时增益 / 群体圣盾 / 排除自身全场伤害 / 抽牌按费用伤害 / 职业过滤 | `core/effect.rs` / `trigger.rs` / `cards/mod.rs` | 纳特·帕格、法力沸腾者、正义、伊瑟拉之醒、神圣愤怒、光明之泉、自然之力、顺手牵羊 |

**缺失**（按 wave 分组的原语）：

1. 手牌区交换、伤害反射奥秘、1 生命复活奥秘 → W7

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
## Wave 4 — 费用与武器互动（8 张）✅ 完成（PR #83）

**原语**——全部落地：
- 手牌区费用光环：`AuraEffect::IncreaseMinionCost`（法力怨魂——所有随从 +1，
  双方手牌都生效）与 `IncreaseMinionCostFriendly`（风险投资公司雇佣兵——只加
  自己的），叠在 G5 修正栈上（`effective_cost` 同时扫双方费用光环桶）。
- 按武器攻击减费：恐怖海盗在 `play_cost` 中减去武器攻击力。
- 武器耐久削减：`CardEffect::RemoveWeaponDurability`（血帆海盗），耐久归零
  即摧毁。
- 武器装备谓词：`AuraEffect::ChargeWithWeapon`（南海船工）——
  `effective_charge` 在武器在场时授予冲锋（召唤失调也被豁免）。
- 敌方法术 0 费：`CardEffect::EnemySpellsCostZero`（米米尔隆）——玩家级
  `spells_cost_zero` 标志，`play_cost` 读取、回合结束清除。
- 给对手水晶：`CardEffect::GiveOpponentManaCrystal`（奥术傀儡——空水晶）。

| ID | 卡名 | 真实效果 | 场景 |
| --- | --- | --- | --- |
| NEUTRAL_R22 | 法力怨魂 | 所有随从的法力值消耗 +1 | `w4_mana_wraith_increases_all_minion_costs` |
| NEUTRAL_C14 | 风险投资公司雇佣兵 | 你的随从法力值消耗 +3 | `w4_venture_co_increases_own_minion_costs` |
| NEUTRAL_C07 | 南海船工 | 装备武器时具有冲锋 | `w4_southsea_deckhand_charge_with_weapon` |
| NEUTRAL_C13 | 恐怖海盗 | 嘲讽；武器每有 1 攻击力减 1 费 | `w4_dread_corsair_cost_by_weapon_attack` |
| NEUTRAL_C09 | 血帆袭击者 | 战吼：获得与武器攻击力相等的攻击力 | `w4_bloodsail_raider_gains_weapon_attack` |
| NEUTRAL_R03 | 血帆海盗 | 战吼：从对手武器移除 1 点耐久 | `w4_bloodsail_corsair_removes_weapon_durability` + `…_destroys_1_durability_weapon` |
| LEGENDARY_021 | 米米尔隆的头部 | 敌方法术下回合费用为 0 | `w4_millhouse_makes_enemy_spells_free` |
| NEUTRAL_R14 | 奥术傀儡 | 冲锋；对手获得一个法力水晶 | `w4_arcane_golem_gives_opponent_crystal` |

**验收**：9 个差分场景（含与既有光环的费用修正叠加）；RL 卡池 +8（363 → 371）。
## Wave 5 — 目标结构与效果组合（7 张）✅ 完成（PR #84）

**原语**——全部落地：
- `CardEffect::SetPlayedMinionHealth`（忏悔——奥秘把打出的敌方随从生命设为 1，
  由奥秘系统带事件上下文解析，同 Snipe 模式）。
- `CardEffect::SilenceAllEnemyMinionsAndDraw`（群体驱散——沉默 + 抽牌组合）。
- `CardEffect::SwapAttackAndHealth`（疯狂炼金师——用附魔差量表达交换，
  交换后沉默会回到基础值）。
- `CardEffect::FreezeAdjacent`（冰锥术——随机敌方随从 + 左右邻位冻结）。
- `CardEffect::GrantAdjacentTaunt`（日怒保卫者）与
  `GrantAdjacentSpellDamage`（远古法师）——邻位增益目标。
- `CardEffect::FullHealAndTaunt`（先祖治疗——满血 + 嘲讽组合）。
- 新增 `EffectTarget::AnyMinion`（任意随从，疯狂炼金师 / 先祖治疗的目标域）。

| ID | 卡名 | 真实效果 | 场景 |
| --- | --- | --- | --- |
| EX1_349 | 忏悔 | 奥秘：对手随从的生命设为 1 | `w5_repentance_sets_played_minion_health_to_1` |
| PRIEST_018 | 群体驱散 | 沉默所有敌方随从并抽一张牌 | `w5_mass_dispel_silences_all_enemy_minions` |
| NEUTRAL_R08 | 疯狂炼金师 | 交换一个随从的攻击与生命 | `w5_crazed_alchemist_swaps_stats` |
| MAGE_016 | 冰霜新星锥 | 冻结一个随从及其相邻随从 | `w5_cone_of_cold_freezes_adjacent` |
| NEUTRAL_R11 | 日怒保卫者 | 使相邻随从获得嘲讽 | `w5_sunfury_protector_taunts_adjacent` |
| NEUTRAL_R18 | 远古法师 | 使相邻随从获得法术伤害 +1 | `w5_ancient_mage_gives_adjacent_spell_damage` |
| SHAMAN_018 | 先祖治疗 | 将一个随从恢复至满血并获得嘲讽 | `w5_ancestral_healing_full_heals_and_taunts` |

**验收**：7 个差分场景；RL 卡池 +7（371 → 378）。
## Wave 6 — 特殊机制（8 张）✅ 完成（PR #85）

**原语**——全部落地：
- 概率效果：`CardEffect::ChanceDraw`（纳特·帕格——回合结束 50% 抽牌）。
- 本回合临时增益：`CardEffect::GainStatsThisTurn`（法力沸腾者——附魔层
  `UntilEndOfTurn` 到期）。
- 群体圣盾：`CardEffect::GrantDivineShieldAllFriendly`（正义）。
- 排除自身的全场伤害：`CardEffect::YseraAwakens`（伊瑟拉之醒——按卡 ID
  放过伊瑟拉本体）。
- 抽牌-按费用伤害：`CardEffect::DrawAndDamageByCost`（神圣愤怒——抽牌后
  造成等同于其费用的伤害）。
- 受伤友方回合开始治疗：`CardEffect::RestoreDamagedFriendly`（光明之泉——
  从回合结束改为回合开始）。
- 群体增益+嘲讽：`CardEffect::GainStatsAndTauntAllFriendly`（自然之力）。
- 职业过滤：顺手牵羊的 `OtherClass` 池在 W6 曾被判定"已忠实"，但实际
  过滤器（`!ROGUE_CLASSIC`）把全部中立卡也算进去了。**2026-08-06 修正**：
  卡池改为恰好是另外 8 个职业的职业卡（`pool.rs` 的 `is_other_class_card`，
  由 `other_class_pool_is_class_cards_of_other_classes` 单元测试与加强后的
  `w6_pilfer_adds_other_class_card` 场景钉住）。

| ID | 卡名 | 真实效果 | 场景 |
| --- | --- | --- | --- |
| LEGENDARY_022 | 纳特·帕格 | 回合结束：50% 几率抽一张牌 | `w6_nat_pagle_chance_draw` |
| NEUTRAL_R10 | 法力沸腾者 | 施法后：本回合 +2 攻击 | `w6_mana_addict_buff_expires_at_turn_end` |
| PALADIN_018 | 正义 | 使你的随从获得圣盾 | `w6_righteousness_grants_divine_shields` |
| NEUTRAL_T21e | 伊瑟拉之醒 | 对所有**其他**角色造成 5 点伤害 | `w6_ysera_awakens_spares_ysera` |
| DRUID_016 | 自然之力 | 你的随从 +2/+2 并获得嘲讽 | `w6_gift_of_the_wild_buffs_and_taunts` |
| PALADIN_017 | 神圣愤怒 | 抽一张牌；造成等同于其费用值的伤害 | `w6_holy_wrath_damages_by_drawn_cost` |
| EX1_341 | 光明之泉 | 回合开始：为一个受伤的友方角色恢复 3 点生命 | `w6_lightwell_heals_at_turn_start` |
| ROGUE_025 | 顺手牵羊 | 随机将一张其他职业的卡牌置入你的手牌 | `w6_pilfer_adds_other_class_card` |

**验收**：8 个差分场景；RL 卡池 +8（378 → 388，含顺手牵羊注释清理）。
## Wave 7 — 收尾：复杂遗留（3 张）✅ 完成（PR #86）

**原语**——全部落地：
- 手牌区交换：`CardEffect::SwapWithHandMinion`（闹钟机器人——回合开始与手牌
  随机随从互换，换入随从落在原位置并带召唤失调）。
- 伤害反射奥秘：`ReflectDamage`（以眼还眼——新
  `SecretTrigger::WhenFriendlyHeroDamaged`，英雄受伤时对敌方英雄造成等量伤害，
  带伤害事件上下文）。
- 1 生命复活奥秘：`CardEffect::ResurrectDiedMinion`（救赎——友方随从死亡时
  以 1 生命复活，带死亡事件上下文）。

| ID | 卡名 | 真实效果 | 场景 |
| --- | --- | --- | --- |
| NEUTRAL_R13 | 闹钟机器人 | 回合开始：与手牌中一个随机随从交换 | `w7_alarm_o_bot_swaps_with_hand_minion` |
| PALADIN_020 | 以眼还眼 | 奥秘：你受到伤害时对敌方英雄造成等量伤害 | `w7_eye_for_an_eye_reflects_damage` |
| PALADIN_021 | 救赎 | 奥秘：一个友方随从死亡时以 1 生命复活 | `w7_redemption_resummons_with_1_health` |

**验收**：3 个差分场景；**账本清空；RL 卡池达到全经典构筑池满规模（391 张）**；
终扫 + 完整 SabberStone 对照。
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
| W4 费用/武器 ✅ PR #83 | 8 | 费用光环/武器攻击减费/耐久削减/条件冲锋/法术0费/给水晶 | +8 → **371** |
| W5 目标结构 ✅ PR #84 | 7 | 生命设为1/交换攻防/邻位目标/双效果组合 | +7 → **378** |
| W6 特殊机制 ✅ PR #85 | 8 | 概率/临时增益/群体圣盾/排除自身/按费用伤害/职业过滤 | +8 → **388** |
| W7 收尾 ✅ PR #86 | 3 | 手牌区交换/伤害反射/1 生命复活 | +3 → **391（账本清空）** |
| **合计** | **67 ✅ 全部完成** | | **321 → 391（全经典构筑池满规模）** |

---

# 保真债实现路线图 — W8~W12（§11 的 24 张卡）

> 已归档（2026-08-06）——全部 wave W8~W12 完成（PR #97–#101）；账本 §11
> （`docs/fidelity-debt.md`）已清空，RL 池回到完整 391 张经典卡池。清偿保真债账本的剩余部分：`docs/fidelity-debt.md`
> §11 —— 2026-08-06 补登记发现 27 张已知简化卡没有 `(simplified: …)` 标记；
> F-A8（PR #88）解决 3 张，W8（PR #97）解决 5 张，剩 **19 张**。每张卡只有在真实炉石效果实现**并且**
> 通过 F5 差分测试验证后才离开账本（账本维护约定）。英文镜像：
> `fidelity-debt-roadmap.md`。

## 原则

1. **先接线**（已归档 W0~W7 路线图的规则）：机制已存在的 wave 先于任何新机制
   wave 落地。
2. **一个 wave = 一个 PR**（W0~W7 先例：PR #79–#86）。卡离开账本必须带
   `tests/differential.rs` 里的 F5 差分场景（`w8_*` … `w12_*`）钉死真实结算
   顺序，语义可镜像的部分按 SabberStone 做对照。
3. **RL 卡池**：每张清偿的卡重新进入训练卡池（账本清空时 367 → **391**）；
   按账本约定，每次改动同时删除代码里的 `(simplified …)` 注释并使提取器缓存
   失效（`~/.cache/orange_stone_debt_ids.txt`）。

## 机制清单（2026-08-06 核实）

| 需求 | 现状 | 说明 |
| --- | --- | --- |
| 激怒（`ThisMinionDamaged` 关键词表） | ✅ 已有（W0） | 阿曼尼狂战士 / 暴怒的狼人 / 格罗玛什：纯接线 |
| 冲锋光环（`GrantCharge`，W1 苔原犀牛） | ✅ 已有 | 战歌指挥官：友方随从全量光环，排除自身 |
| 治疗抽牌（`CharacterHealed`，W2） | ✅ 已有 | 北郡牧师：触发接线 |
| 武器攻击特效（`Attacked`，W2 智慧祝福） | ✅ 已有 | 真银圣剑回血；血吼减攻 |
| `SecretPlayed`（W2） | ✅ 已有 | 鹰角弓 +1 耐久 |
| 种族条件目标（`FriendlyRace`，W1） | ✅ 已有 | 狂野怒火：野兽限定 |
| 抉择管道（`ChoiceKind::ChooseOne`，G6） | ✅ 已有 | 4 张德鲁伊卡疑似 def 侧缺第二分支 |
| 发现管道（`ChoiceKind::Discover` + 卡池，trigger.rs） | ✅ 已有 | 追踪术需"牌库顶 3 张"的池来源 |
| 战吼召唤 / 弃牌 / 圣盾 | 部分 | 奥妮克希亚多张召唤；死亡之翼全弃（新效果）；阿古斯圣盾目标结构 |
| 减费（海巨人 / 伺机待发） | 部分 | 按场上随从数减费光环（新）；下个法术减费标志（米尔豪斯 W4 先例） |
| 治疗翻倍（先知维伦） | 缺 | 新光环/修饰器 |
| 受伤冻结（水元素） | 缺 | "造成伤害时冻结"的触发方向（W12 定夺） |
| 控制（秘教暗影祭司） | ✅ 已有 | `TakeControl` + `EnemyMinionAttackLE`（W3）：战吼接线 |
| 伤害+抽牌（毒刃） | ✅ 已有 | `DealDamageAndDraw`（神圣愤怒路径）：接线 |
| 连击基础分支（冷血） | 部分 | 连击管道已有；缺非连击 +2 分支 |

## Wave

### W8 — 触发接线（5 张）：阿曼尼狂战士、暴怒的狼人、格罗玛什、战歌指挥官、北郡牧师 ✅（PR #97）

- 经 `ThisMinionDamaged` 关键词表接线激怒（W0 先例）：阿曼尼狂战士
  （2/3 → 激怒 5/3）、暴怒的狼人（3/3，激怒：+1 攻击**并且**获得风怒——官方
  卡牌数据基准是 3/3 而非最初草稿写的 1/3；原常驻风怒已移除，风怒属于激怒）、
  格罗玛什（4/9 → 激怒 10/9，非 7/9）
- 战歌指挥官：其他友方随从全量 `GrantCharge` 光环（苔原犀牛先例，排除自身）
- 北郡牧师：友方 `CharacterHealed` 时抽一张（治疗汇聚点新增
  `FriendlyCharacterHealed` 事件）

**验收**：✅ 5 个 `w8_*` 差分场景（激怒结算顺序、冲锋光环授予、治疗抽牌时机相对
治疗事件）；`cargo test` 全绿（422）；RL 池 367 → **372**；§11 删 5 行。

### W9 — 武器与种族（4 张）：真银圣剑、血吼、鹰角弓、狂野怒火 ✅（PR #98）

- 真银圣剑：英雄持剑攻击时治疗 2（`Attacked` 先例——武器触发挂在武器实体上，
  `trigger_applies` 把攻击事件钉扎到攻击者或其英雄装备的武器）
- 血吼：英雄攻击随从时武器失去 1 点攻击（新 `AttackedMinion` 事件；rules.rs
  经 `GOREHOWL_ID` 对随从命中跳过耐久扣减；打脸照常扣耐久）
- 鹰角弓：每有友方奥秘**揭示** +1 耐久（两个揭示点统一的新
  `FriendlySecretRevealed` 事件——真实卡牌是"揭示"触发而非"打出"；路线图初稿
  建议 `SecretPlayed` 先例，最终按真实语义用揭示）
- 狂野怒火：既有 `GrantAttackAndImmune` 效果的目标改为 `FriendlyRace(Beast)`
  （非野兽目标使法术哑火，G9 重新校验）

**验收**：✅ 4 个 `w9_*` 场景（攻击时治疗时机、逐次攻击减攻、每揭示加耐久、
野兽限定目标）；`cargo test` 全绿（426）；RL 池 372 → **376**。

### W10 — 抉择与发现（5 张）：愤怒、利爪德鲁伊、知识古树、战争古树、追踪术 ✅（PR #99）

- 愤怒 / 利爪德鲁伊 / 知识古树 / 战争古树：把两个分支接进既有抉择管道
  （审计证实了诊断——管道选项映射（battlecry = 分支 0，`choose_one_effect` =
  分支 1）本就完整，只是 def 从未接第二分支）。利爪德鲁伊基准回到 4/4、战争
  古树基准回到 5/5；`GrantCharge` 与 `GainStatsAndTaunt` 新增 `Self_` 分支
- 追踪术：从**玩家牌库顶 3 张**中发现——选 1 入手，弃掉其余。**D1 已决**：
  新池来源（牌库顶 3 张）以并行 Discover 选择呈现并携带新 `discard_rest`
  标记——被选卡牌的**既有实体**移入手牌，其余两张弃掉

**验收**：✅ 5 个 `w10_*` 场景（各分支选择、分支的费用/目标、追踪术牌库顶 3 张
语义含弃牌）；`cargo test` 全绿（431）；RL 池 376 → **381**。

### W11 — 战吼与减费（6 张）：奥妮克希亚、阿古斯保护者、死亡之翼、海巨人、伺机待发、冷血 ✅（PR #100）

- 奥妮克希亚：召唤五只 1/1 幼龙（`SummonMinion` 循环；新 `EX1_170t` 衍生物，
  按 `t` 后缀规则自动排除出 RL 池）
- 阿古斯保护者：相邻随从 +1/+1 **并且**获得圣盾（新 `GrantAdjacentStatsAndDivineShield`，
  W5 邻位解析器；原 def 是错误的自增益战吼——从未登记的静默误实现，故无账本行）
- 死亡之翼：**弃掉整个手牌**（新 `DiscardHand`）+ 摧毁所有其他随从
  （新 `DestroyAllOtherMinionsAndDiscardHand`——旧的 `AllMinions` 效果会连
  死亡之翼自己一起毁掉）
- 海巨人：场上每有一个随从费用 -1（场面计数规则在 `cost::play_cost` 合成——
  G5 单一费用合成点，弯刀海盗先例）
- 伺机待发：本回合下一个法术费用 -3（每玩家 `next_spell_discount` 标志，
  首个法术消耗、回合末清除——米尔豪斯 `spells_cost_zero` W4 先例）
- 冷血：基础 +2 分支与既有连击分支一起接线

**验收**：✅ 6 个 `w11_*` 场景（幼龙数量与满场、阿古斯邻位含圣盾授予顺序、死亡之翼
全弃 + 清场、海巨人费用随场面、伺机待发窗口、冷血基础分支）；`cargo test` 全绿（437）；
RL 池 381 → **386**（路线图原写 387 假设阿古斯已登记——实际没有，故池只增 5 而非 6）。

### W12 — 剩余机制（5 张）：水元素、秘教暗影祭司、先知维伦、毒刃、银色保卫者 ✅（PR #101）

- 水元素：冻结受到**该随从**伤害的角色——**D2 已决**：`Event::DamageDealt`
  伤害管线检查（在圣盾吸收之前应用——盾被击破但目标仍冻结，符合炉石）
- 秘教暗影祭司：战吼 `TakeControl` 一个攻击 ≤ 2 的敌方随从
  （新 `TakeControlAttackLE` 效果，永久控制）
- 先知维伦：治疗翻倍——**D3 已决**：`resolve_restore_health` 管线钩子
  （维伦在场时翻倍；法术伤害部分维持已登记的 +1 重平衡）
- 毒刃：1 伤 + 抽 1（`DealDamageAndDraw` —— 接线）
- 银色保卫者：战吼圣盾（新 `GrantDivineShield` 效果）——路线图 wave 核算把
  这张已登记卡错算成了未登记的阿古斯保护者；清偿它后账本清空

**验收**：✅ 5 个 `w12_*` 场景；`cargo test` 全绿（442）；**RL 池 386 → 391
（账本 §11 清空）**；✅ 本路线图对已归档到 `docs/finished/`。

## Wave 核算

| Wave | 卡数 | RL 池 |
| --- | --- | --- |
| W8 触发接线 ✅（PR #97） | 5 | 367 → **372** |
| W9 武器与种族 ✅（PR #98） | 4 | 372 → **376** |
| W10 抉择与发现 ✅（PR #99） | 5 | 376 → **381** |
| W11 战吼与减费 ✅（PR #100） | 6 | 381 → **386** |
| W12 剩余机制 ✅（PR #101） | 5 | 386 → **391（账本清空）** |

## 超出范围

- §11 之外的已知简化卡：无——账本维护约定要求任何新增 `(simplified …)` 注释
  在同一改动里登记到本账本。
- 168 维观测张量改动（疲劳路线图 D5）。
