# 出牌目标枚举缺口审计（2026-08-21）

> 起因：网页对局里出 **影焰晕染（FIR_939）** 没有瞄准环节，2 点伤害随机
> 落到了对方英雄头上。顺藤摸瓜发现这不是单卡问题，而是**出牌目标枚举的
> 系统性缺口**。本文是逐卡审计的结论清单，供决定修哪些、按什么顺序修。

## 结论摘要

| | 数量 |
|---|---|
| 官方要玩家指定目标、引擎却不给目标的卡（**A 类**） | **64** |
| ├─ T1 结算已接目标（补声明即可）——**全部修完** ✅ | 39 |
| └─ T2 结算不接目标（要把目标接进结算）——**修完 24，余 1**（抉择卡） | 25 |
| 文本像要目标、实际不需要（**B 类**，已排除） | 33 |

覆盖范围从**经典卡一直到 2025–2026 全部扩展**：经典 24 张、翡翠梦境 12 张、失落之城/安戈洛 10 张、时光之径 7 张、大灾变 5 张、核心 3 张、紫罗兰监狱 3 张。

## 根因

`src/rl/env.rs` 的 `play_targets()` 用一个 **`match` 白名单**逐个列举
"这个效果变体要玩家选目标"，列进去的才会展开成多条"出牌 → 打某个目标"
的合法动作；没列进去的落到 `_ => return Vec::new()`，只生成一条无目标出牌。

```rust
let target = match battlecry.0 {
    CardEffect::DealDamage { target, .. } => target,
    // ……共 44 条
    _ => return Vec::new(),          // ← 新加的复合效果全落这里
};
```

白名单有 44 条，而 `trigger.rs` 里接受 `explicit_target` 的效果变体有 142 个。
每次加卡都新造一个复合效果变体（`DamageAndDrawIfSurvives`、
`DealDamageAndDraw`、`DamageAndDiscoverWarriorWithGift`……），**没人回头补白名单**，
于是新变体默认无目标。到了结算层，`resolve_deal_damage` 等函数拿不到
`explicit`，走 `select_target(explicit, &candidates, rng)` → **随机挑一个**；
如果那条分支写的是 `let t = explicit?;` → **直接哑火**。

两条退化路径都是静默的：没有报错，没有日志，玩家只看到"我明明该能选目标"。

## 方法（三重证据）

1. **静态**：解析 `src/cards/*.rs` 全部 1701 条 `CardDef`，取出 `battlecry` /
   `spell_effect` 的效果变体，与 `play_targets` 白名单求差集 → 564 张卡。
2. **官方文本**：按 `cards/cards.json`（id 优先、卡名回退，与引擎对拍同口径）
   取官方英文文本，只看**出牌时的子句**（法术=全文，随从=`Battlecry:` 那句），
   命中"a/an minion|character|…"或裸的 "Deal $N damage." 且不含 random/all/each
   → 97 张候选。逐条人工复核，剔除发现/抽牌/条件类 33 张 → **A 类 64 张**。
3. **实测探针**：每张卡开一局镜像局（该卡 ×8 + 白板随从填充，对手 greedy），
   推进到双方都有随从时出这张卡，diff 出牌前后的局面。**64 张全部实测确认
   没有任何目标选项**（`target_id == -1`）。（探针**只有这一个结论可信**——
   它进一步判断"效果退化成什么"的那部分是错的，见下方「更正」。）
   对照组 火球术（在白名单里）正常给出目标。

## 更正（2026-08-21，本文首版之后）

首版把 A 类按"哑火 8 张 / 随机 54 张"分类，**这个分类是错的**，已作废。

错在测量工具：探针用"出牌前后 diff 局面"判断效果有没有生效，而快照只记了
血量/攻击力/随从数量——**看不见圣盾被打爆、看不见冻结、看不见附魔、
看不见"把 1/1 设成 1/1"这类等值改写，也看不见英雄攻击力为 0 时的 0 点伤害**。
于是一批正常工作的卡被误判成哑火。最典型的是致死打击：探针说它"零变化"，
实际是 4 点伤害随机打爆了一只白银之手新兵的圣盾——引擎里本来就有一条
`w3_mortal_strike_boosts_at_low_health` 测试证明它工作正常。

替换成一个**静态且可验证**的分类轴：结算分支到底收不收 `explicit_target`。
这决定了修复的工作量，也不依赖任何快照精度。

| | 数量 |
|---|---|
| **T1 结算已接目标** —— 补一条 `play_target()` 声明即可 | 36 |
| **T2 结算不接目标** —— 还要把玩家选的目标接进结算 | 28 |

（"哑火还是随机"这个问题本身也就没有意义了：两种退化都让玩家失去选择权，
修复动作完全一样。）

## 已修复（W1 波，10 张）

经典 + 核心里 T1 的全部卡，已声明目标域并带 `tests/play_targeting.rs` 测试
（每张都验证两件事：合法动作按候选目标展开；带目标出牌命中的是**被选中的
那个**，而不是同域里的随机一个）。

| 卡 ID | 卡名 | 效果变体 | 官方文本（出牌子句） |
|---|---|---|---|
| `DRUID_018` | Savagery | `DealHeroAttackDamage` | Deal damage equal to your hero's Attack to a minion. |
| `HUNTER_021` | Bestial Wrath | `GrantAttackAndImmune` | Give a friendly Beast +2 Attack and Immune this turn. |
| `CLASSIC_009` | Dark Iron Dwarf | `GainStatsThisTurn` | Give a minion +2 Attack this turn. |
| `NEUTRAL_001` | Abusive Sergeant | `GainStatsThisTurn` | Give a minion +2 Attack this turn. |
| `WARLOCK_017` | Siphon Soul | `DestroyAndHeal` | Destroy a minion. Restore #3 Health to your hero. |
| `WARLOCK_021` | Demonfire | `Demonfire` | Deal $2 damage to a minion. If it’s a friendly Demon, give it +2/+2 instead. |
| `WARRIOR_016` | Charge | `GrantCharge` | Give a friendly minion +2 Attack and Charge. |
| `WARRIOR_021` | Mortal Strike | `MortalStrike` | Deal $4 damage. If you have 12 or less Health, deal $6 instead. |
| `CORE_CS2_188` | Abusive Sergeant | `GainStatsThisTurn` | Give a minion +2 Attack this turn. |
| `CORE_TRL_240` | Savage Striker | `DealHeroAttackDamage` | Deal damage to an enemy minion equal to your hero's Attack. |

声明的目标域**取卡面/结算已有的域，不顺带放宽**。其中三张比官方窄，单独记账：
野蛮之击、灵魂虹吸（官方"一个随从"，引擎限敌方随从）、
凌辱/黑铁矮人（官方"一个随从"，引擎限友方随从）。致死打击同理
（官方任意角色，引擎限敌方）。这类"域偏窄"是另一类问题——写测试时还发现
**火球术也是 `AnyEnemy`**——值得单独扫一轮。

## T1 已全部修完 ✅（W2 波，26 张）

W1 修了经典 + 核心 10 张，W2 把各扩展剩下的 26 张一次修完 —— **T1 清空**。

| 卡 ID | 卡名 | 效果变体 | 官方文本（出牌子句） |
|---|---|---|---|
| `CATA_161` | Gruesome Nightmare | `SetAttackEqualToSource` | Give a minion in your hand or battlefield Attack equal to this minion's Attack |
| `CATA_552` | Ebonscale Scout | `DealDamageEqualSelfAttack` | Deal damage equal to this minion's Attack. (While in hand, play a Dragon to  b |
| `CATA_552t` | Ebonscale Scout | `DealDamageEqualSelfAttack` | Deal damage equal to this minion's Attack. (While in hand, play a Dragon to  b |
| `CATA_564` | Air Support | `GrantMegaWindfuryCantAttackHeroes` | Give a friendly minion Mega-Windfury. |
| `CATA_699` | Dread Leviathan | `StealHealthThreeTimes` | Choose an enemy minion to steal 3  Health from, three times. |
| `EDR_860` | Resplendent Dreamweaver | `DealDamageIfImbuedTwice` | If you've Imbued your Hero Power twice, deal 4 damage to a minion. |
| `EDR_252` | Mark of Ursol | `SetStatsByFriendlyTarget` | Choose a minion. If it's an enemy, set its stats to 1/1. If it's friendly, set |
| `EDR_261` | Amphibian's Spirit | `AmphibianSpiritBuff` | Give a minion +2/+2 and "Deathrattle: Give a friendly minion +2/+2 and this De |
| `EDR_262` | Spirit Bond | `DamageAndSummonWolfIfKilled` | Deal $3 damage to a minion. If it dies, summon a 3/2 Wolf with Rush. |
| `EDR_460` | Wish of the New Moon | `DamageMinionWithMoonLifesteal` | Deal $6 damage to a minion. (Cast 3 spells to gain Lifesteal.) |
| `EDR_523` | Web of Deception | `ReturnFriendlyMinionSummonSpider` | Return a friendly minion to your hand to summon a 4/4 Spider with Stealth. |
| `EDR_531` | Siphoning Growth | `DestroyFriendlyMinionGainArmor` | Destroy a friendly minion to gain 8 Armor. |
| `FIR_908` | Charred Chameleon | `GiveMinionStatsRushIfHeroPowerUsed` | If you've used your Hero Power this turn, give a friendly minion +1/+2 and Rus |
| `FIR_918` | Light of the New Moon | `BuffMinionReturnIfSpellsCast` | Give a minion +3/+3. (Cast 3 spells to return this to your hand when played.) |
| `FIR_939` | Shadowflame Suffusion | `DamageAndDiscoverWarriorWithGift` | Deal $2 damage. Discover a Warrior minion with a Dark Gift. |
| `FIR_954` | Conflagrate | `DamageMinionOwnerDraws` | Deal $5 damage to a minion. Its owner draws a card. |
| `JAIL_998` | Defias Smuggler | `GainStatsAndGrantRush` | Give a friendly minion +2 Attack and Rush. |
| `TLC_230` | TREEEES!!! | `SummonTreantsAttackMinion` | Choose a minion. Summon four 2/2 Treants that attack it. |
| `TLC_252` | Dissolving Ooze | `DestroyFriendlyMinionAddBones` | Destroy a friendly minion. |
| `TLC_441` | Ready the Fleet | `GiveBuffSameType` | Give +1/+2 to a friendly minion and your other minions that share a type with |
| `TLC_606` | Latorvian Armorer | `DealDamageGainArmorIfKilled` | Deal 2 damage to an enemy minion. |
| `TLC_620` | Fortify | `GainArmorDealDamageEqual` | Gain 3 Armor. Deal damage equal to your Armor to an enemy minion. |
| `TLC_823` | Cower in Fear | `DealDamageSetNextBeastDiscount` | Deal $3 damage to a minion. The next Beast you play this turn costs (2) less. |
| `TLC_901` | Fumigate | `DealDamageSameType` | Deal $3 damage to a minion and all others of the same minion type. |
| `TLC_987` | Questing Assistant | `DealDamageIfQuestPlayed` | If you played a Quest this game, deal 3 damage to an enemy minion. |
| `DINO_419` | Herbivore Assistant | `GainStatsAndGrantRush` | Give a friendly Beast +2/+2 and Rush. |

覆盖方式：一条 sweep 测试断言这 26 张**每一张**都给出目标（并与 `rules::validate`
一致），外加按目标域各挑一张写"命中被选中那个"的测试（烈焰风暴/虹吸生长/
影焰晕染/食草助手）。

两处声明比结算的候选集窄，是有意的（窄是安全的，宽了会在结算处哑火）：
空中支援与恐怖梦魇的结算扫的是 `Zone::Play(owner)`（**含英雄**），声明只给随从。

## T2 已清空 ✅（W3 波，27 张，余 1 张待抉择机制）

T2 = 结算分支**根本不收** `explicit_target` 的卡：即使声明了目标域，效果也会
自己随便挑。W3 把 `explicit_target` 接进了每一条结算链路。

**先更正一处分类**：冰刺、毒刃、猛击当初被划进 T2 是**分类脚本的 bug** ——
它按缩进找 `CardEffect::X` 分支，结果匹配到了法术伤害加成表（`apply_spell_power`）
而不是真正的结算函数。这三张的结算一直是认目标的，只差一条声明。

| 卡 ID | 卡名 | 效果变体 |
|---|---|---|
| `MAGE_022` | Icicle | `FreezeOrDamage` |
| `ROGUE_014` | Shiv | `DealDamageAndDraw` |
| `WARRIOR_011` | Slam | `DealDamageAndDraw` |

剩下 24 张是真 T2，按结算形态分三类改：

- **A 类（5 张）**：分支已经调用 `resolve_deal_damage`/`resolve_restore_health`，
  只是硬传了 `None` —— 改成传 `explicit_target`。
- **B 类（7 张）**：分支自己收集候选再随机挑（`pick_random` / `rng.next_usize`）
  —— 换成 `select_target(explicit_target, ...)`，没给目标时行为不变（仍是随机）。
- **C 类（12 张）**：结算 helper 的签名里压根没有目标参数 —— 加一个
  `explicit: Option<Entity>` 并一路传下去，共改了 11 个 helper。

| 卡 ID | 卡名 | 效果变体 |
|---|---|---|
| `MAGE_016` | Cone of Cold | `FreezeAdjacent` |
| `CLASSIC_FM` | Faceless Manipulator | `CopyMinionStats` |
| `PALADIN_017` | Holy Wrath | `DrawAndDamageByCost` |
| `PRIEST_011` | Cabal Shadow Priest | `TakeControlAttackLE` |
| `PRIEST_021` | Natalie Seline | `DestroyAndGainHealth` |
| `PRIEST_022` | Shadow Madness | `TakeControlUntilEndOfTurn` |
| `PRIEST_023` | Mind Control | `TakeControl` |
| `ROGUE_019` | Shadowstep | `ReturnFriendlyToHandAndReduceCost` |
| `ROGUE_020` | Betrayal | `AdjacentDamage` |
| `ROGUE_024` | Master of Disguise | `GrantStealth` |
| `SHAMAN_003` | Rockbiter Weapon | `GainHeroAttack` |
| `WARLOCK_018` | Shadowflame | `DestroyAndAOE` |
| `WARLOCK_024` | Corruption | `Corrupt` |
| `CORE_EX1_198` | Natalie Seline | `DestroyAndGainHealth` |
| `JAIL_101` | Violet Punisher | `VioletPunisher` |
| `JAIL_395` | Sewer Swimmer | `SewerSwimmer` |
| `TLC_221` | Sizzling Swarm | `DealDamageSummonCinders` |
| `TIME_043` | PMM Infinitizer | `SetStatsAndCantAttackHeroesThisTurn` |
| `TIME_427` | Cleansing Lightspawn | `DealDamageEnemyMinionEqualToSourceHealth` |
| `TIME_431` | Amber Priestess | `RestoreHealthEqualToSourceHealth` |
| `TIME_442` | Timeway Warden | `ImprisonEnemyMinion` |
| `TIME_614` | Liferender | `DealDamageEnemyMinionIfHeroHealthChanged` |
| `TIME_858` | Temporal Construct | `DealDamageAndDrawExcess` |
| `TIME_435` | Eternus | `TakeControlEnemyMinionHealthLE` |

### 顺带补的两个洞

**1. 枚举层也有同一个静默 catch-all。** `rl::env::candidates_for_target` 的
`match` 结尾是 `_ => Vec::new()`：声明了一个它不认识的目标域，结果不是报错，
而是**静默地给出空候选表** —— 卡又变回无目标出牌。31 个 `EffectTarget` 里它
只枚举了 15 个。现已改成穷尽 match（无 `_` 分支），补齐 7 个单体目标域
（`EnemyMinionAttackLE`、`AnyMinionAttackLE`、`EnemyMinionWithRace`、
`OtherFriendlyMinion`、三个 Damaged\* 系列），AoE 类则显式列出返回空表。
**这直接影响已合并的 W1/W2**：暗影言灵·死、饥饿的螃蟹这类用过滤域的卡，
即便变体在白名单里也一直枚举不出候选。

**2. 两个目标域是缺的，补进了 `EffectTarget`：**
- `FriendlyMinionWithDeathrattle` —— 下水道游泳者"触发一个友方随从的亡语"，
  只有带亡语的随从才该亮起瞄准线。
- `EnemyMinionHealthLESource` —— 永恒者按**自身生命值**卡上限，域是动态的，
  于是 `candidates_for_target` 多收了一个 `source` 参数。

### 还剩 1 张

`EDR_813` 病变虫群是**抉择卡**：只有第二个模式（消耗 2 尸体造成 4 点伤害）
需要目标。`play_targets` 只读 battlecry 组件，抉择/连击是另一条路径，需要
"先选模式、再选目标"的两段式动作设计 —— 单独一波。

## B 类 —— 已排除的 33 张

文本里有"a minion / a Beast"之类措辞，但那是**发现（Discover）、抽牌、
条件判定**，出牌时本来就不需要指定目标，引擎当前行为正确。

| 卡 ID | 卡名 | 效果变体 | 官方文本（出牌子句） |
|---|---|---|---|
| `CORE_BT_321` | Netherwalker（虚无行者） | `AddRandomCardToHand` | Discover a Demon. |
| `CORE_EDR_004` | Raptor Herald（迅猛龙先锋） | `AddRandomCardToHand` | Discover a Beast with a Dark Gift. |
| `CORE_KAR_061` | The Curator（馆长） | `DrawBeastDragonMurloc` | Draw a Beast, Dragon, and Murloc. |
| `CORE_KAR_062` | Netherspite Historian（虚空幽龙史学家） | `AddRandomCardToHand` | If you're holding a Dragon, Discover a Dragon. |
| `CORE_LOE_039` | Gorillabot A-3（A3型机械金刚） | `AddRandomCardToHand` | If you control another Mech, Discover a Mech. |
| `CORE_RLK_116` | Necrotic Mortician（死灵殡葬师） | `AddRandomCardToHand` | If a friendly Undead died after your last turn, Discover an Unholy Rune card. |
| `CORE_CATA_006` | Ulfar（奥尔法） | `GrantDeathrattleSummonOwnCost` | Give your other minions "Deathrattle: Summon a minion with this minion's Cost. |
| `CORE_TRL_111` | Headhunter's Hatchet（猎头者之斧） | `BuffWeaponDurabilityIfBeast` | If you control a Beast, gain +1 Durability. |
| `CATA_111` | Darkscale Broodmother（晦鳞巢母） | `RefreshManaIfHoldingDragon` | If you're holding a Dragon, refresh 2 Mana Crystals. |
| `CATA_553` | Ebyssian（艾比西安） | `SetDragonsHaveRush` | Your Dragons have Rush this game. (While in hand, play a Dragon to become a 12 |
| `CATA_553t` | Ebyssian | `SetDragonsHaveRush` | Your Dragons have Rush this game. (While in hand, play a Dragon to become a 12 |
| `MEND_041` | Wizened Wildspeaker（年迈的荒语者） | `RefreshManaIfNoMinionPlayedLastTurn` | If you didn't play a minion last turn, refresh 3 Mana Crystals. |
| `EDR_226` | Exotic Houndmaster（奇异训犬师） | `DrawBeastAndImbue` | Draw a Beast. |
| `EDR_456` | Darkrider（黑暗的龙骑士） | `DiscoverDragonWithDarkGift` | If you're holding a Dragon, Discover a Dragon with a Dark Gift. |
| `EDR_856` | Nightmare Lord Xavius（梦魇之王萨维斯） | `DiscoverDeckMinionWithDarkGift` | Discover a minion from your deck. |
| `EDR_490` | Sleep Paralysis（麻痹睡眠） | `SummonMultipleMinions` | Choose One - Summon two 3/6 Demons with Taunt that can't attack; or Destroy an |
| `EDR_843` | Reforestation（森林再生） | `DrawCardByType` | Choose One - Draw a spell; or Draw a minion. (Hold this for 3 turns to do both |
| `EDR_455` | Succumb to Madness（屈从疯狂） | `ResurrectRandomFallenDragon` | Discover a friendly Dragon that died this game. Resummon it. |
| `EDR_457` | Brood Keeper（龙巢守护者） | `EquipSwordIfHoldingDragon` | If you're holding a Dragon, equip a 2/2 Sword. |
| `FIR_900` | Cremate（火化） | `DiscoverWithDarkGiftCostReduction` | Discover a minion with a Dark Gift. It costs (2) less. |
| `FIR_901` | Frostburn Matriarch（霜灼巢母） | `SummonBroodlingsIfHoldingGift` | If you're holding a minion with a Dark Gift, summon two 4/4 Dragons with Taunt |
| `FIR_922` | Cindersword（燃薪之剑） | `GainWeaponAttackIfHoldingGift` | If you're holding a minion with a Dark Gift, gain +3 Attack. |
| `FIR_924` | Shadowflame Stalker（影焰猎豹） | `DiscoverDemonWithDarkGiftCopy` | Discover a Demon with a Dark Gift. |
| `FIR_941` | Searing Reflection（烧灼映像） | `DrawMinionSummonDivineShieldCopy` | Draw a minion. Summon an 8/8 copy of it with Divine Shield. |
| `FIR_956` | Dragon Turtle（龙龟） | `GainHeroAttackArmorIfHoldingGift` | If you're holding a minion with a Dark Gift, give your hero +3 Attack this tur |
| `TLC_231` | Story of Barnabus（班纳布斯的故事） | `DrawMinionBuffArmorIfAttackGE` | Draw a minion. If it has 5 or more Attack, give it +5 Health and gain 5 Armor. |
| `TLC_434` | Paleomancy（古生物秘术） | `DiscoverPool` | Discover an Undead. Spend 5 Corpses to keep all 3 instead. |
| `TLC_442` | Submerged Map（淹没的地图） | `DiscoverPool` | Discover a Murloc. If you play it this turn, also pick one of the others. |
| `TLC_464` | Mountain Map（登山地图） | `DiscoverPool` | Discover a minion with a type you haven't played. If you play it this turn, al |
| `TLC_888` | Cloud Serpent（云端翔龙） | `CopyRandomHandElementalOrDragon` | Get a copy of another Elemental or Dragon in your hand. |
| `TLC_110` | City Chief Esho（城市首脑埃舒） | `EshoDeckCheckBuffEverywhere` | If every minion in your deck shares a minion type, give your other minions +2/ |
| `TIME_037` | Disciple of the Dove（白鸽学徒） | `DrawMinionAndBuffHandMinionsHealth` | Draw a minion. |
| `TIME_062` | Chronicle Keeper（史书守护者） | `GainTauntAndDivineShieldIfHoldingDragon` | If you're holding a Dragon, gain Taunt and Divine Shield. |

## 特殊情况（修的时候会撞上）

- **`EDR_813` 蛆虫群**：抉择卡，只有第二个模式（消耗 2 尸体造成 4 点伤害）
  要目标。`play_targets` 只读 battlecry 组件，抉择/连击是另一条路径，
  需要单独设计"选模式后再选目标"的两段式动作。
- **`CATA_161` 恐怖梦魇**：官方文本是"你**手牌或**战场上的一个随从"，
  目标域包含手牌。现有 `EffectTarget` 全是战场域，需要新目标种类。
- **`CATA_552` 黑鳞斥候 / `CATA_552t`**：两个形态同一效果，修一处要同步两处。
- **`FIR_939` 影焰晕染**：官方文本是光秃秃的"造成 2 点伤害"，按炉石惯例
  （火球术）应可指定**任意角色**；而结算写死 `EffectTarget::AnyEnemy`。
  修这张要同时改结算的目标域，不只是补白名单。
- **账本 `docs/finished/fidelity-debt.md:933`** 对 FIR_939 写的是
  "the damage IS faithful" —— 与事实不符，修卡时一并更正。
- ~~**F-A13（修 T1 时发现的独立 bug）**~~ —— **已修复（2026-08-21）**：引擎的
  "消灭随从"曾用致死伤害实现，**圣盾能吃掉一次消灭**（刺杀打白银之手新兵 →
  盾破、随从活着）。现由 `Destroyed` 标记 + 共享的
  `engine::rules::destroy_minion()` 接管全部 20 处消灭点，不再经过伤害管线。
  `tests/play_targeting.rs` 的棋盘仍然不放圣盾随从——那是为了让"是否命中被选中
  目标"的信号干净，与 F-A13 是否修好无关。

## 修复建议

1. ~~先修废卡~~ —— 分类已作废（见「更正」）。改为**先清 T1**：结算已接目标，
   补一条声明 + 测试即可。**T1 已全部清空**（W1 经典 + 核心 10 张，W2 各扩展 26 张）。
2. ~~**再啃 T2**~~ —— **已完成（W3）**：24 张接上了结算链路，只剩 `EDR_813`
   病变虫群等抉择卡的两段式动作设计。
3. ~~**结构性防复发**~~ —— **已完成（2026-08-21）**：目标声明从
   `play_targets` 的 `match` 搬到了 `CardEffect::play_target()`
   （`src/core/play_target.rs`，866 个变体**穷尽 match，无 `_` 分支**）。
   新加效果变体不表态就编译不过（实测：临时加一个变体 →
   `error[E0004]: non-exhaustive patterns`）。纯结构性改动，行为零变化：
   1064 项既有测试全绿，批量对局吞吐 2089~2143 局/s（改动前 2089~2138，同噪声带）。
4. ~~**守卫测试**~~ —— **已完成**：`audit_gap_cards_are_still_untargeted`
   把本文 A 类 64 张的 id 写成可执行账本，任何一张开始声明目标就断言失败，
   **强制修卡的同一个提交里同步删掉清单条目**。旧的内联白名单正是因为
   "代码和清单各走各的"才漂成今天这样。

## 复现

审计脚本与探针留在 `/tmp`（一次性），核心口径是：

```bash
# 单卡验证：出牌动作有没有 target_id != -1
python - <<'EOF'
import orange_stone
e = orange_stone.GameEnv(seed=1, deck=["MAGE_001"]*30, bot="none"); e.reset(1)
for _ in range(6):
    plays = [a for a in e.structured_legal_actions() if a.kind == "play"]
    if plays: print([p.target_id for p in plays]); break
    e.step(next(a.index for a in e.structured_legal_actions() if a.kind == "end_turn"))
EOF
# 火球术（MAGE_001，在白名单里）→ target_id 有真实实体；
# 影焰晕染（FIR_939）→ 全是 -1
```
