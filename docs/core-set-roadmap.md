# Core Set (核心系列) Roadmap — implementing the 281 CORE cards

> Status: **active** (created 2026-08-07; decisions D1–D5 resolved 2026-08-08,
> wave plan below). Chinese mirror: `core-set-roadmap-zh.md`.
> Scope: all `CORE_*` cards in `cards/cards.json` — the modern Core Set, 281 cards
> (173 minions / 95 spells / 8 weapons / 1 hero / 1 location / 3 enchantment tokens),
> drawn from **42 original sets** (the ID's middle part encodes the source).

## Why

The Core Set is the natural second set after Classic: it is the free-to-all
replacement set, its cards reuse the engine's existing mechanics heavily
(Battlecry 84, Deathrattle 23, Discover 14, Aura 9, Secret 8, Choose One 6,
Overload 5, Combo 3 …), and it is exactly the "second set" the pool-open
registry (`docs/finished/pool-openness.md`) was designed for — the day a
non-Classic card can enter the pool, the closure question becomes concrete.

**This roadmap is the card inventory.** Milestone/wave planning, mechanics
gaps, and the pool/RL decisions below are deliberately left open until the
inventory is confirmed — every card is a checkbox.

## Card inventory (281)

> Card names are English (the data source's names). Chinese translations
> follow the `classic-cards-zh.md` convention when the cards land.
> Tokens (ENCHANTMENT) and the HERO/LOCATION cards are listed but may be
> deferred to a dedicated wave.

### CORE_AT_* — 冠军的试炼（6 张）

- [x] `CORE_AT_037` Living Roots — 1 费 法术
- [x] `CORE_AT_052` Totem Golem — 2 费随从 3/4
- [x] `CORE_AT_055` Flash Heal — 1 费 法术
- [x] `CORE_AT_062` Ball of Spiders — 3 费 法术
- [x] `CORE_AT_064` Bash — 2 费 法术
- [x] `CORE_AT_123` Chillmaw — 7 费随从 6/6

### CORE_AV_* — 奥特兰克的决裂（2 张）

- [x] `CORE_AV_107` Glaciate — 6 费 法术
- [x] `CORE_AV_337` Mountain Bear — 7 费随从 5/6

### CORE_BAR_* — 贫瘠之地的锤炼（7 张）

- [x] `CORE_BAR_310` Lightshower Elemental — 6 费随从 6/6
- [x] `CORE_BAR_311` Devouring Plague — 3 费 法术
- [x] `CORE_BAR_313` Priest of An'she — 5 费随从 5/5
- [x] `CORE_BAR_541` Runed Orb — 2 费 法术
- [x] `CORE_BAR_801` Wound Prey — 1 费 法术
- [x] `CORE_BAR_812` Oasis Ally — 3 费 法术
- [x] `CORE_BAR_878` Veteran Warmedic — 4 费随从 3/5

### CORE_BOT_* — 砰砰计划（4 张）

- [x] `CORE_BOT_222` Spirit Bomb — 1 费 法术
- [x] `CORE_BOT_256` Astromancer — 7 费随从 5/5
- [x] `CORE_BOT_451` Voltaic Burst — 1 费 法术
- [x] `CORE_BOT_576` Crazed Chemist — 5 费随从 4/4

### CORE_BRM_* — 黑石山的火焰（1 张）

- [x] `CORE_BRM_013` Quick Shot — 2 费 法术

### CORE_BT_* — 外域的灰烬（18 张）

- [x] `CORE_BT_035` Chaos Strike — 2 费 法术
- [x] `CORE_BT_072` Deep Freeze — 7 费 法术
- [x] `CORE_BT_120` Warmaul Challenger — 3 费随从 1/10
- [x] `CORE_BT_156` Imprisoned Vilefiend — 2 费随从 3/5
- [x] `CORE_BT_187` Kayn Sunfury — 4 费随从 3/5
- [x] `CORE_BT_201` Augmented Porcupine — 3 费随从 2/4
- [x] `CORE_BT_292` Hand of A'dal — 2 费 法术
- [x] `CORE_BT_321` Netherwalker — 2 费随从 2/2
- [x] `CORE_BT_351` Battlefiend — 1 费随从 1/2
- [x] `CORE_BT_416` Raging Felscreamer — 4 费随从 4/4
- [x] `CORE_BT_480` Crimson Sigil Runner — 1 费随从 1/1
- [x] `CORE_BT_491` Spectral Sight — 2 费 法术
- [x] `CORE_BT_493` Priestess of Fury — 7 费随从 6/7
- [x] `CORE_BT_510` Wrathspike Brute — 5 费随从 3/6
- [x] `CORE_BT_701` Spymistress — 1 费随从 3/1
- [x] `CORE_BT_781` Bulwark of Azzinoth — 3 费武器 1/4
- [x] `CORE_BT_801` Eye Beam — 3 费 法术
- [x] `CORE_BT_921` Aldrachi Warblades — 3 费武器 2/2

### CORE_CATA_* — 大灾变（迷你）（18 张）

- [x] `CORE_CATA_001` Tichondrius — 9 费随从 9/8
- [x] `CORE_CATA_002` Calia Menethil — 6 费随从 4/5
- [ ] `CORE_CATA_003` Vindicator Maraad — 3 费随从 2/4 — ⛔ official placeholder (never released, no card text), not implemented (decision 2026-08-08)
- [x] `CORE_CATA_004` Rehgar Earthfury — 5 费随从 3/5
- [ ] `CORE_CATA_005` Lord Thorval — 3 费随从 2/4 — ⛔ official placeholder (never released, no card text), not implemented (decision 2026-08-08)
- [x] `CORE_CATA_006` Ulfar — 6 费随从 4/3
- [x] `CORE_CATA_006e` Thornspeakers' Spirit — 附魔衍生物（0 费）
- [x] `CORE_CATA_007` Consumption — 4 费 法术
- [ ] `CORE_CATA_008` Guardian Gargoyle — 3 费随从 2/4 — ⛔ official placeholder (never released, no card text), not implemented (decision 2026-08-08)
- [x] `CORE_CATA_009` Death's Advance — 2 费 法术
- [ ] `CORE_CATA_010` Felscarred Seeker — 3 费随从 2/4 — ⛔ official placeholder (never released, no card text), not implemented (decision 2026-08-08)
- [ ] `CORE_CATA_011` Ring of Frost — 3 费 法术 — ⛔ official placeholder (never released, no card text), not implemented (decision 2026-08-08)
- [ ] `CORE_CATA_012` Forbidden Ritual — 3 费 法术 — ⛔ official placeholder (never released, no card text), not implemented (decision 2026-08-08)
- [ ] `CORE_CATA_013` Slimescale Myrmidon — 3 费随从 2/4 — ⛔ official placeholder (never released, no card text), not implemented (decision 2026-08-08)
- [ ] `CORE_CATA_014` Crystalweaver — 3 费随从 2/4 — ⛔ official placeholder (never released, no card text), not implemented (decision 2026-08-08)
- [ ] `CORE_CATA_015` Tricky Geist — 3 费随从 2/4 — ⛔ official placeholder (never released, no card text), not implemented (decision 2026-08-08)
- [ ] `CORE_CATA_016` Wild Synthesis — 3 费 法术 — ⛔ official placeholder (never released, no card text), not implemented (decision 2026-08-08)
- [ ] `CORE_CATA_017` Essence Break — 3 费 法术 — ⛔ official placeholder (never released, no card text), not implemented (decision 2026-08-08)

### CORE_CFM_* — 龙争虎斗加基森（6 张）

- [x] `CORE_CFM_344` Finja, the Flying Star — 5 费随从 3/5
- [x] `CORE_CFM_604` Greater Healing Potion — 4 费 法术
- [x] `CORE_CFM_670` Mayor Noggenfogger — 9 费随从 5/4
- [x] `CORE_CFM_753` Grimestreet Outfitter — 2 费随从 2/2
- [x] `CORE_CFM_781` Shaku, the Collector — 3 费随从 2/4
- [x] `CORE_CFM_790` Dirty Rat — 2 费随从 2/6

### CORE_CS1_* — 基础系列（2 张）

- [x] `CORE_CS1_112` Holy Nova — 3 费 法术
- [x] `CORE_CS1_130` Holy Smite — 1 费 法术

### CORE_CS2_* — 基础系列（22 张）

- [x] `CORE_CS2_004` Power Word: Shield — 1 费 法术
- [x] `CORE_CS2_009` Mark of the Wild — 2 费 法术
- [x] `CORE_CS2_023` Arcane Intellect — 3 费 法术
- [x] `CORE_CS2_024` Frostbolt — 2 费 法术
- [x] `CORE_CS2_028` Blizzard — 6 费 法术
- [x] `CORE_CS2_029` Fireball — 4 费 法术
- [x] `CORE_CS2_032` Flamestrike — 7 费 法术
- [x] `CORE_CS2_039e` Windfury — 附魔衍生物（0 费）
- [x] `CORE_CS2_042` Fire Elemental — 6 费随从 6/5
- [x] `CORE_CS2_053` Far Sight — 3 费 法术
- [x] `CORE_CS2_062` Hellfire — 3 费 法术
- [x] `CORE_CS2_072` Backstab — 0 费 法术
- [x] `CORE_CS2_074` Deadly Poison — 1 费 法术
- [x] `CORE_CS2_076` Assassinate — 4 费 法术
- [x] `CORE_CS2_093` Consecration — 3 费 法术
- [x] `CORE_CS2_094` Hammer of Wrath — 3 费 法术
- [x] `CORE_CS2_108` Execute — 1 费 法术
- [x] `CORE_CS2_122` Raid Leader — 3 费随从 2/3
- [x] `CORE_CS2_179` Sen'jin Shieldmasta — 4 费随从 3/5
- [x] `CORE_CS2_188` Abusive Sergeant — 1 费随从 1/1
- [x] `CORE_CS2_189` Elven Archer — 1 费随从 1/1
- [x] `CORE_CS2_222` Stormwind Champion — 7 费随从 7/7

### CORE_DAL_* — 暗影崛起（2 张）

- [x] `CORE_DAL_575` Khadgar — 2 费随从 2/2
- [x] `CORE_DAL_720` Waggle Pick — 4 费武器 4/2

### CORE_DMF_* — 疯狂的暗月马戏团（2 张）

- [x] `CORE_DMF_067` Prize Vendor — 2 费随从 2/3
- [x] `CORE_DMF_511` Foxy Fraud — 2 费随从 3/2

### CORE_DRG_* — 巨龙降临（5 张）

- [x] `CORE_DRG_024` Sky Raider — 1 费随从 1/2
- [x] `CORE_DRG_079` Evasive Wyrm — 6 费随从 5/4
- [x] `CORE_DRG_107` Violet Spellwing — 1 费随从 2/1
- [x] `CORE_DRG_256` Dragonbane — 4 费随从 3/5
- [x] `CORE_DRG_403` Blowtorch Saboteur — 3 费随从 3/3

### CORE_DS1_* — 基础系列（猎人）（2 张）

- [x] `CORE_DS1_184` Tracking — 1 费 法术
- [x] `CORE_DS1_185` Arcane Shot — 1 费 法术

### CORE_EDR_* — 来源系列名待补（EDR）（6 张）

- [x] `CORE_EDR_001` Babbling Bookcase — 3 费随从 2/4
- [x] `CORE_EDR_002` Poison Breath — 2 费 法术
- [x] `CORE_EDR_002e` Deathly Poison — 附魔衍生物（0 费）
- [x] `CORE_EDR_003` Falric — 3 费随从 2/4
- [x] `CORE_EDR_004` Raptor Herald — 3 费随从 4/2
- [x] `CORE_EDR_004_2026` Raptor Herald — 3 费随从 4/2 (与 `CORE_EDR_004` 同一张卡的另一版本 ID，不单独实现)

### CORE_ETC_* — 来源系列名待补（ETC）（2 张）

- [x] `CORE_ETC_111` Merch Seller — 4 费随从 3/5
- [x] `CORE_ETC_523` Death Metal Knight — 3 费随从 3/4

### CORE_EX1_* — 经典系列（55 张）

- [x] `CORE_EX1_002` The Black Knight — 4 费随从 4/4
- [x] `CORE_EX1_005` Big Game Hunter — 4 费随从 4/2
- [x] `CORE_EX1_007` Acolyte of Pain — 3 费随从 1/4
- [x] `CORE_EX1_010` Worgen Infiltrator — 1 费随从 2/1
- [x] `CORE_EX1_011` Voodoo Doctor — 1 费随从 2/1
- [x] `CORE_EX1_012` Bloodmage Thalnos — 2 费随从 1/1
- [x] `CORE_EX1_014` King Mukla — 3 费随从 5/6
- [x] `CORE_EX1_028` Stranglethorn Tiger — 5 费随从 5/5
- [x] `CORE_EX1_043` Twilight Drake — 4 费随从 4/1
- [x] `CORE_EX1_058` Sunfury Protector — 2 费随从 2/3
- [x] `CORE_EX1_059` Crazed Alchemist — 2 费随从 2/2
- [x] `CORE_EX1_082` Mad Bomber — 2 费随从 3/2
- [x] `CORE_EX1_096` Loot Hoarder — 2 费随从 2/1
- [x] `CORE_EX1_100` Lorewalker Cho — 2 费随从 0/4
- [x] `CORE_EX1_103` Coldlight Seer — 3 费随从 2/3
- [x] `CORE_EX1_110` Cairne Bloodhoof — 6 费随从 5/5
- [x] `CORE_EX1_129` Fan of Knives — 2 费 法术
- [x] `CORE_EX1_131` Defias Ringleader — 2 费随从 3/2
- [x] `CORE_EX1_134` SI:7 Agent — 3 费随从 3/3
- [x] `CORE_EX1_145` Preparation — 0 费 法术
- [x] `CORE_EX1_154` Wrath — 2 费 法术
- [x] `CORE_EX1_160` Power of the Wild — 2 费 法术
- [x] `CORE_EX1_162` Dire Wolf Alpha — 2 费随从 2/2
- [x] `CORE_EX1_169` Innervate — 0 费 法术
- [x] `CORE_EX1_189` Brightwing — 2 费随从 3/2
- [x] `CORE_EX1_193` Psychic Conjurer — 1 费随从 1/2
- [x] `CORE_EX1_197` Shadow Word: Ruin — 4 费 法术
- [x] `CORE_EX1_198` Natalie Seline — 7 费随从 7/1
- [x] `CORE_EX1_238` Lightning Bolt — 1 费 法术
- [x] `CORE_EX1_246` Hex — 3 费 法术
- [x] `CORE_EX1_250` Earth Elemental — 5 费随从 7/9
- [x] `CORE_EX1_259` Lightning Storm — 3 费 法术
- [x] `CORE_EX1_278` Shiv — 2 费 法术
- [x] `CORE_EX1_287` Counterspell — 3 费 法术
- [x] `CORE_EX1_289` Ice Barrier — 3 费 法术
- [x] `CORE_EX1_302` Mortal Coil — 1 费 法术
- [x] `CORE_EX1_309` Siphon Soul — 4 费 法术
- [x] `CORE_EX1_310` Doomguard — 5 费随从 5/7
- [x] `CORE_EX1_312` Twisting Nether — 8 费 法术
- [x] `CORE_EX1_319` Flame Imp — 1 费随从 3/2
- [x] `CORE_EX1_323` Lord Jaraxxus — 英雄（8 费）
- [x] `CORE_EX1_362` Argent Protector — 2 费随从 3/2
- [x] `CORE_EX1_383` Tirion Fordring — 8 费随从 8/8
- [x] `CORE_EX1_391` Slam — 1 费 法术
- [x] `CORE_EX1_414` Grommash Hellscream — 8 费随从 4/9
- [x] `CORE_EX1_506` Murloc Tidehunter — 2 费随从 2/1
- [x] `CORE_EX1_506a` Murloc Scout — 1 费随从 1/1
- [x] `CORE_EX1_507` Murloc Warleader — 3 费随从 3/3
- [x] `CORE_EX1_509` Murloc Tidecaller — 1 费随从 1/2
- [x] `CORE_EX1_559` Archmage Antonidas — 7 费随从 5/7
- [x] `CORE_EX1_604` Frothing Berserker — 3 费随从 2/4
- [x] `CORE_EX1_606` Shield Block — 2 费 法术
- [x] `CORE_EX1_610` Explosive Trap — 2 费 法术
- [x] `CORE_EX1_611` Freezing Trap — 2 费 法术
- [x] `CORE_EX1_619` Equality — 2 费 法术

### CORE_GIL_* — 女巫森林（8 张）

- [x] `CORE_GIL_191t` Imp — 1 费随从 1/1
- [x] `CORE_GIL_531` Witch's Apprentice — 0 费随从 0/1
- [x] `CORE_GIL_534` Hench-Clan Thug — 3 费随从 3/3
- [x] `CORE_GIL_558` Swamp Leech — 1 费随从 2/1
- [x] `CORE_GIL_577` Rat Trap — 2 费 法术
- [x] `CORE_GIL_622` Lifedrinker — 4 费随从 3/3
- [x] `CORE_GIL_623` Witchwood Grizzly — 5 费随从 3/12
- [x] `CORE_GIL_836` Blazing Invocation — 1 费 法术

### CORE_GVG_* — 地精大战侏儒（5 张）

- [x] `CORE_GVG_059` Coghammer — 3 费武器 2/3
- [x] `CORE_GVG_061` Muster for Battle — 3 费 法术
- [x] `CORE_GVG_085` Annoy-o-Tron — 2 费随从 1/2
- [x] `CORE_GVG_103` Micro Machine — 2 费随从 1/2
- [x] `CORE_GVG_114` Sneed's Old Shredder — 7 费随从 5/7

### CORE_ICC_* — 冰封王座的骑士（5 张）

- [x] `CORE_ICC_038` Righteous Protector — 1 费随从 1/1
- [x] `CORE_ICC_055` Drain Soul — 2 费 法术
- [x] `CORE_ICC_210` Shadow Ascendant — 2 费随从 2/3
- [x] `CORE_ICC_214` Obsidian Statue — 9 费随从 4/8
- [x] `CORE_ICC_407` Gnomeferatu — 2 费随从 2/3

### CORE_KAR_* — 卡拉赞之夜（5 张）

- [x] `CORE_KAR_057` Ivory Knight — 4 费随从 4/4
- [x] `CORE_KAR_061` The Curator — 5 费随从 4/6
- [x] `CORE_KAR_062` Netherspite Historian — 2 费随从 2/3
- [x] `CORE_KAR_069` Swashburglar — 1 费随从 1/2
- [x] `CORE_KAR_077` Silvermoon Portal — 3 费 法术

### CORE_LOE_* — 探险者协会（1 张）

- [x] `CORE_LOE_039` Gorillabot A-3 — 3 费随从 3/4

### CORE_LOOT_* — 狗头人与地下世界（8 张）

- [x] `CORE_LOOT_013` Vulgar Homunculus — 2 费随从 2/4
- [x] `CORE_LOOT_044` Bladed Gauntlet — 2 费武器 0/2
- [x] `CORE_LOOT_101` Explosive Runes — 3 费 法术
- [x] `CORE_LOOT_137` Sleepy Dragon — 9 费随从 6/12
- [x] `CORE_LOOT_309` Oaken Summons — 4 费 法术
- [x] `CORE_LOOT_368` Voidlord — 9 费随从 3/9
- [x] `CORE_LOOT_373` Healing Rain — 3 费 法术
- [x] `CORE_LOOT_413` Plated Beetle — 2 费随从 2/3

### CORE_NEW1_* — 经典补充（7 张）

- [x] `CORE_NEW1_018` Bloodsail Raider — 2 费随从 2/3
- [x] `CORE_NEW1_020` Wild Pyromancer — 2 费随从 3/2
- [x] `CORE_NEW1_021` Doomsayer — 2 费随从 0/7
- [x] `CORE_NEW1_022` Dread Corsair — 4 费随从 3/3
- [x] `CORE_NEW1_023` Faerie Dragon — 2 费随从 3/2
- [x] `CORE_NEW1_027` Southsea Captain — 3 费随从 3/3
- [x] `CORE_NEW1_031` Animal Companion — 3 费 法术

### CORE_NX2_* — 来源系列名待补（NX2）（1 张）

- [x] `CORE_NX2_028` Hookfist-3000 — 3 费随从 4/3

### CORE_OG_* — 上古之神的低语（6 张）

- [x] `CORE_OG_031` Hammer of Twilight — 5 费武器 4/2
- [x] `CORE_OG_044` Fandral Staghelm — 4 费随从 3/6
- [x] `CORE_OG_047` Feral Rage — 3 费 法术
- [x] `CORE_OG_149` Ravaging Ghoul — 3 费随从 3/3
- [x] `CORE_OG_211` Call of the Wild — 8 费 法术
- [x] `CORE_OG_218` Bloodhoof Brave — 4 费随从 2/6

### CORE_ONY_* — 奥妮克希亚的巢穴（迷你）（2 张）

- [x] `CORE_ONY_018` Boomkin — 5 费随从 4/5
- [x] `CORE_ONY_022` Battle Vicar — 2 费随从 1/3

### CORE_REV_* — 来源系列名待补（REV）（4 张）

- [x] `CORE_REV_023` Demolition Renovator — 3 费随从 3/3
- [x] `CORE_REV_308` Maze Guide — 2 费随从 1/1
- [x] `CORE_REV_946` Steamcleaner — 5 费随从 5/5
- [x] `CORE_REV_990` Sanguine Depths — 地点（1 费）

### CORE_RLK_* — 巫妖王的进军（17 张）

- [x] `CORE_RLK_062` Nerubian Swarmguard — 4 费随从 1/3
- [x] `CORE_RLK_063` Frostwyrm's Fury — 7 费 法术
- [x] `CORE_RLK_066` Hematurge — 2 费随从 2/3
- [x] `CORE_RLK_083` Deathchiller — 2 费随从 2/3
- [x] `CORE_RLK_086` Frostmourne — 6 费武器 4/3
- [x] `CORE_RLK_087` Asphyxiate — 3 费 法术
- [x] `CORE_RLK_116` Necrotic Mortician — 2 费随从 2/3
- [x] `CORE_RLK_118` Tomb Guardians — 4 费 法术
- [x] `CORE_RLK_121` Acolyte of Death — 3 费随从 2/4
- [x] `CORE_RLK_505` Marrow Manipulator — 6 费随从 5/5
- [x] `CORE_RLK_506` Boneguard Commander — 8 费随从 8/8
- [x] `CORE_RLK_567` Shadow of Demise — 0 费 法术
- [x] `CORE_RLK_657` Underking — 7 费随从 6/6
- [x] `CORE_RLK_706` Alexandros Mograine — 7 费随从 7/7
- [x] `CORE_RLK_712` Blood Tap — 2 费 法术
- [x] `CORE_RLK_745` Malignant Horror — 4 费随从 2/4
- [x] `CORE_RLK_814` Crystalsmith Cultist — 1 费随从 1/2

### CORE_SCH_* — 通灵学园（5 张）

- [x] `CORE_SCH_181` Archwitch Willow — 8 费随从 5/5
- [x] `CORE_SCH_512` Initiation — 6 费 法术
- [x] `CORE_SCH_605` Lake Thresher — 5 费随从 4/6
- [x] `CORE_SCH_713` Cult Neophyte — 2 费随从 3/2
- [x] `CORE_SCH_717` Keymaster Alabaster — 7 费随从 6/8

### CORE_SW_* — 暴风城下的集结（9 张）

- [x] `CORE_SW_047` Highlord Fordragon — 6 费随从 5/5
- [x] `CORE_SW_066` Royal Librarian — 4 费随从 4/4
- [x] `CORE_SW_068` Mo'arg Forgefiend — 8 费随从 8/8
- [x] `CORE_SW_072` Rustrot Viper — 3 费随从 3/4
- [x] `CORE_SW_088` Demonic Assault — 4 费 法术
- [x] `CORE_SW_108` First Flame — 1 费 法术
- [x] `CORE_SW_429` Best in Shell — 6 费 法术
- [x] `CORE_SW_439` Vibrant Squirrel — 1 费随从 2/1
- [x] `CORE_SW_442` Void Shard — 4 费 法术

### CORE_TID_* — 来源系列名待补（TID）（1 张）

- [x] `CORE_TID_931` Jackpot! — 2 费 法术

### CORE_TRL_* — 拉斯塔哈的大乱斗（5 张）

- [x] `CORE_TRL_111` Headhunter's Hatchet — 2 费武器 2/2
- [x] `CORE_TRL_240` Savage Striker — 2 费随从 2/3
- [x] `CORE_TRL_307` Flash of Light — 2 费 法术
- [x] `CORE_TRL_345` Krag'wa, the Frog — 6 费随从 4/6
- [x] `CORE_TRL_900` Halazzi, the Lynx — 4 费随从 4/2

### CORE_TSC_* — 来源系列名待补（TSC）（2 张）

- [x] `CORE_TSC_076` Immortalized in Stone — 7 费 法术
- [x] `CORE_TSC_650` Flipper Friends — 5 费 法术

### CORE_TTN_* — 泰坦诸神（2 张）

- [x] `CORE_TTN_843` Eredar Deceptor — 4 费随从 3/5
- [x] `CORE_TTN_866` Mythical Terror — 7 费随从 4/10

### CORE_ULD_* — 奥丹姆奇兵（8 张）

- [x] `CORE_ULD_133` Crystal Merchant — 2 费随从 1/4
- [x] `CORE_ULD_152` Pressure Plate — 2 费 法术
- [x] `CORE_ULD_165` Riftcleaver — 6 费随从 7/5
- [x] `CORE_ULD_178` Siamat — 7 费随从 7/7
- [x] `CORE_ULD_191` Beaming Sidekick — 1 费随从 1/2
- [x] `CORE_ULD_271` Injured Tol'vir — 2 费随从 2/6
- [x] `CORE_ULD_280` Sahket Sapper — 4 费随从 4/4
- [x] `CORE_ULD_723` Murmy — 1 费随从 1/1

### CORE_UNG_* — 勇闯安戈洛（7 张）

- [x] `CORE_UNG_084` Fire Plume Phoenix — 4 费随从 3/4
- [x] `CORE_UNG_205` Glacial Shard — 1 费随从 2/1
- [x] `CORE_UNG_809` Fire Fly — 1 费随从 1/2
- [x] `CORE_UNG_848` Primordial Drake — 8 费随从 4/8
- [x] `CORE_UNG_912` Jeweled Macaw — 1 费随从 1/2
- [x] `CORE_UNG_928` Tar Creeper — 3 费随从 1/5
- [x] `CORE_UNG_952` Spikeridged Steed — 5 费 法术

### CORE_WC_* — 来源系列名待补（WC）（2 张）

- [x] `CORE_WC_042` Wailing Vapor — 1 费随从 1/3
- [x] `CORE_WC_701` Felrattler — 3 费随从 3/2

### CORE_WON_* — 来源系列名待补（WON）（6 张）

- [x] `CORE_WON_096` Dark Peddler — 2 费随从 2/3
- [x] `CORE_WON_141` Menagerie Mug — 3 费随从 3/3
- [x] `CORE_WON_145` Avatar of Hearthstone — 9 费随从 5/5
- [x] `CORE_WON_337` Ironforge Portal — 4 费 法术
- [x] `CORE_WON_350` I Know a Guy — 1 费 法术
- [x] `CORE_WON_351` Small-Time Buccaneer — 1 费随从 1/2

### CORE_WW_* — 来源系列名待补（WW）（2 张）

- [x] `CORE_WW_329` Detonation Juggernaut — 4 费随从 3/4
- [x] `CORE_WW_374` Corpse Farm — 3 费 法术

### CORE_YOD_* — 来源系列名待补（YOD）（1 张）

- [x] `CORE_YOD_026` Fiendish Servant — 1 费随从 2/1

### CORE_YOP_* — 来源系列名待补（YOP）（2 张）

- [x] `CORE_YOP_001` Illidari Studies — 1 费 法术
- [x] `CORE_YOP_034` Runaway Blackwing — 10 费随从 10/10

## Decisions (resolved 2026-08-08)

The five open decisions were settled in a decision round (2026-08-08);
research basis: `ALL_CARDS` is the single pool source that `GameEnv.
all_card_ids()`/RL `full_pool()` follow, `build.rs` already generates static
consts for all 281 CORE cards, and the engine has no primitives for
RUSH/LIFESTEAL/TRADEABLE/OUTCAST/REBORN/spell-power exemption — POISONOUS /
FREEZE / ENRAGED / OVERLOAD / CHOOSE_ONE / COMBO / DISCOVER / SECRET / AURA
already exist.

1. **Pool definition — `ALL_CARDS` stays the single source.** Implemented
   CORE cards join `ALL_CARDS` one by one (no second registry). Core cards
   that read an opponent zone (Dirty Rat, Gnomeferatu, Shaku, Swashburglar,
   …) must register in `POOL_OPEN_CARDS` (+ `POOL_OPEN_KEYWORD_IDS` where the
   effect is keyword-mapped); the `pool_open_effects_require_registry` test
   enforces it, and the RL pool follows automatically.
2. **Overlap with Classic — both versions coexist** (as in real Hearthstone).
   The 88 reprints (CS1 2 + CS2 22 + DS1 2 + EX1 55 + NEW1 7) and their
   Classic counterparts are all in the pool; the Classic 413-card pool,
   SabberStone reference and training cadence stay untouched. The
   generated-vs-handwritten check switches to ID-suffix matching first
   (`CORE_EX1_100` ↔ generated `EX1_100`), verifying reprint fidelity (W0).
3. **New mechanics inventory — primitives by dependency.** New primitives +
   F5 scenarios: W1 RUSH (5) + LIFESTEAL (8) + REBORN (2); W2 TRADEABLE (6) +
   OUTCAST (3) + ImmuneToSpellpower (3) + AFFECTED_BY_SPELL_POWER (1); the
   `Race` enum grows Dragon / Elemental / Mechanical / Pirate / Totem (W0).
   The HERO card (CORE_EX1_323 Lord Jaraxxus), LOCATION card (CORE_REV_990
   Sanguine Depths, new `CardType::Location`), the 3 ENCHANTMENT cards and
   the Imp token are deferred to the closing wave (W8) with their own
   primitives.
4. **RL pool — follows the engine pool, switched in one go after all cards
   land.** Until then the training pool stays 396. The switch (W9) also
   extends `_load_debt_ids()`'s `classic_*.rs` glob to `core_*.rs` so the
   simplified-debt scan covers Core cards.
5. **Wave shape — wiring first, then primitives, then mechanic batches.**
   See the wave plan below; waves are mechanic-first (not per-set), matching
   W0–W16.

## Wave plan (2026-08-08)

One PR per wave; every card lands with an F5 differential scenario. The
checkbox list above is the authoritative card inventory — check off each card
as its wave lands. Known wrinkle: `CORE_EDR_004` and `CORE_EDR_004_2026` are
the same card under two IDs (re-check during W4a).

- **W0 — wiring (no cards) ✅ (PR #116, 2026-08-08):** generated-vs-
  handwritten matching switches to ID-suffix priority (`CORE_<P>_<n>` ↔
  generated `<P>_<n>` reprint pair); `Race` gains Dragon / Elemental /
  Mechanical / Pirate / Totem (observation encoding extends to 8 tribes,
  dimensionality unchanged). New `core_reprints_match_originals` test guards
  reprint fidelity (90/90 pairs match).
- **W1 — attack-pipeline primitives (15 + F5) ✅ (PR #117, 2026-08-08):**
  RUSH (5) + LIFESTEAL (8) + REBORN (2). New primitives: Rush components +
  SummonedThisTurn hero-attack ban, Lifesteal heal-on-damage (weapons heal via
  the equipped weapon), Reborn fresh-1/1 resurrection, Corpses (friendly
  deaths), forced attack + dual tribes (Vec race storage), Undead tribe, 3 new
  CardEffect variants. `CORE_BT_801` (Eye Beam) and `CORE_BAR_311` (Devouring
  Plague) landed their W1 halves — Outcast / spell-power exemption finish in
  W2.
  `CORE_BT_156 CORE_BT_801 CORE_BT_921 CORE_BAR_311 CORE_DRG_079 CORE_GIL_558
  CORE_ICC_055 CORE_ICC_214 CORE_SW_442 CORE_TTN_866 CORE_RLK_657 CORE_RLK_745
  CORE_TRL_900 CORE_ULD_723 CORE_WC_701`
- **W2 — hand/spell-pipeline primitives (11 + F5) ✅ (PR #118, 2026-08-08):**
  TRADEABLE (6) + OUTCAST (3) + ImmuneToSpellpower (3) +
  AFFECTED_BY_SPELL_POWER (1). New primitives: `Action::TradeCard` (1 mana,
  shuffle back, draw), OutcastPlayed edge marker + `DrawCardOutcast` /
  `OutcastDamage` variants, spell-power exemption, `RestoreRandomFriendly`,
  `DamagePlayedMinionAndExcess` (secret excess to hero), `DestroyEnemyLocation`
  (fizzles until W8), `DealDamage` explicit-target support for AnyMinion.
  Eye Beam / Devouring Plague completed (Outcast / exemption).
  `CORE_EX1_002 CORE_EX1_005 CORE_REV_023 CORE_SW_066 CORE_SW_072 CORE_SW_429
  CORE_BT_480 CORE_BT_491 CORE_BT_801 CORE_BAR_311 CORE_LOOT_101 CORE_LOOT_373
  CORE_BRM_013`
- **W3a/b/c — whiteboard batches (115 cards):** no new primitives; vanilla
  minions/weapons, direct-damage/heal/draw spells, keyword-only cards
  (Taunt/Charge/Divine Shield/Stealth/Elusive/…). Split by set, three PRs.
  - **W3a part 1 ✅ (PR #119, 2026-08-08):** 31 cards + 3 tokens (Frog/Spider
    x2) — faithful effect shapes where the Classic pool simplified (Holy
    Nova heal, Slam/Mortal Coil conditional draws, Shield Block draw, real
    Hex transform, Steed combo); Lorewalker Cho (Core) registered pool-open.
      - **W3a part 2 ✅ (PR #120, 2026-08-08):** the 8 complex cards —
    Noggenfogger target randomization, Khadgar summon doubling, Death
    Metal Knight health payment, Finja/Shaku attack triggers (Shaku
    pool-open), Merch Seller deck-topping, Immortalized in Stone statue
    trio, Runaway Blackwing. **W3a complete (39 cards).**
  - W3a (39): `CORE_AT_055 CORE_AT_062 CORE_AT_064 CORE_CFM_344 CORE_CFM_604
    CORE_CFM_670 CORE_CFM_781 CORE_CS1_112 CORE_CS1_130 CORE_DAL_575
    CORE_DRG_256 CORE_DS1_185 CORE_EDR_002 CORE_ETC_111 CORE_ETC_523
    CORE_EX1_007 CORE_EX1_010 CORE_EX1_028 CORE_EX1_100 CORE_EX1_129
    CORE_EX1_145 CORE_EX1_169 CORE_EX1_197 CORE_EX1_246 CORE_EX1_278
    CORE_EX1_302 CORE_EX1_309 CORE_EX1_312 CORE_EX1_391 CORE_EX1_506a
    CORE_EX1_509 CORE_EX1_559 CORE_EX1_604 CORE_EX1_606 CORE_EX1_619
    CORE_TSC_076 CORE_UNG_928 CORE_UNG_952 CORE_YOP_034`
  - **W3b ✅ (PR #121, 2026-08-08): 27 cards + 5 tokens** — the 11
    CORE_CATA placeholders were removed by decision (official placeholder
    text, never released). New hooks: HeroAttacked / CardDrawn /
    DivineShieldLost trigger events, Dread Corsair cost reduction,
    Quilboar race. `CORE_BAR_801 CORE_BAR_878 CORE_CATA_003 CORE_CATA_004
    CORE_CATA_005 CORE_CATA_007 CORE_CATA_008 CORE_CATA_009 CORE_CATA_010
    CORE_CATA_011 CORE_CATA_012 CORE_CATA_013 CORE_CATA_014 CORE_CATA_015
    CORE_CATA_016 CORE_CATA_017 CORE_GIL_534 CORE_GVG_061 CORE_GVG_085
    CORE_GVG_103 CORE_ICC_038 CORE_ICC_210 CORE_KAR_077 CORE_NEW1_020
    CORE_NEW1_021 CORE_NEW1_022 CORE_NEW1_023 CORE_NEW1_031 CORE_SCH_512
    CORE_SCH_605 CORE_SCH_717 CORE_SW_047 CORE_SW_088 CORE_SW_108 CORE_TID_931
    CORE_TTN_843 CORE_WC_042 CORE_WW_374`
  - **W3c ✅ (PR #122, 2026-08-08): 38 cards + 2 tokens** — faithful
    Classic reprints (Power Word: Shield draw, Backstab undamaged-only),
    15 new effect shapes, engine hooks (Bulwark durability, Bladed Gauntlet
    armor-attack, Small-Time Buccaneer weapon bonus). **W3a/b/c complete
    (115 cards).** `CORE_BOT_222 CORE_BT_035 CORE_BT_292 CORE_BT_351 CORE_BT_493
    CORE_BT_510 CORE_BT_701 CORE_BT_781 CORE_CS2_004 CORE_CS2_009 CORE_CS2_023
    CORE_CS2_029 CORE_CS2_032 CORE_CS2_053 CORE_CS2_062 CORE_CS2_072
    CORE_CS2_074 CORE_CS2_076 CORE_CS2_093 CORE_CS2_094 CORE_CS2_108
    CORE_CS2_179 CORE_LOOT_044 CORE_LOOT_137 CORE_LOOT_309 CORE_NX2_028
    CORE_OG_211 CORE_RLK_063 CORE_RLK_083 CORE_RLK_087 CORE_RLK_118
    CORE_RLK_121 CORE_RLK_567 CORE_RLK_712 CORE_TRL_307 CORE_ULD_133
    CORE_WON_337 CORE_WON_351`
- **W4a/b — battlecry batches (76 cards) ✅ (PR #123/#124, 2026-08-08):** two PRs, split by set.
  - W4a (38): `CORE_BAR_313 CORE_BT_120 CORE_BT_321 CORE_BT_416 CORE_CFM_753
    CORE_CFM_790 CORE_DMF_067 CORE_DMF_511 CORE_EDR_001 CORE_EDR_003
    CORE_EDR_004 CORE_EDR_004_2026 CORE_GIL_531 CORE_GIL_622 CORE_GIL_623
    CORE_GVG_059 CORE_ICC_407 CORE_KAR_057 CORE_KAR_061 CORE_KAR_062
    CORE_KAR_069 CORE_LOE_039 CORE_LOOT_013 CORE_NEW1_018 CORE_ONY_022
    CORE_RLK_062 CORE_RLK_066 CORE_RLK_116 CORE_RLK_505 CORE_RLK_506
    CORE_RLK_706 CORE_RLK_814 CORE_UNG_084 CORE_UNG_205 CORE_UNG_809
    CORE_UNG_848 CORE_UNG_912 CORE_WW_329`
  - W4b (38): `CORE_BOT_256 CORE_CATA_001 CORE_CATA_002 CORE_CATA_006
    CORE_CS2_042 CORE_CS2_188 CORE_CS2_189 CORE_DRG_024 CORE_DRG_403
    CORE_EX1_011 CORE_EX1_014 CORE_EX1_043 CORE_EX1_058 CORE_EX1_059
    CORE_EX1_082 CORE_EX1_103 CORE_EX1_189 CORE_EX1_193 CORE_EX1_198
    CORE_EX1_310 CORE_EX1_319 CORE_EX1_362 CORE_EX1_506 CORE_OG_149
    CORE_REV_308 CORE_REV_946 CORE_SCH_181 CORE_SCH_713 CORE_TRL_111
    CORE_TRL_240 CORE_TRL_345 CORE_ULD_165 CORE_ULD_178 CORE_ULD_191
    CORE_ULD_271 CORE_WON_096 CORE_WON_141 CORE_WON_145`
- **W5 — deathrattle/secret/aura (33 + F5) ✅ (PR #125, 2026-08-08):** `CORE_AT_123 CORE_AV_337
  CORE_BAR_310 CORE_BAR_812 CORE_BT_187 CORE_BT_201 CORE_CS2_122 CORE_CS2_222
  CORE_DAL_720 CORE_DRG_107 CORE_EX1_012 CORE_EX1_096 CORE_EX1_110
  CORE_EX1_162 CORE_EX1_287 CORE_EX1_289 CORE_EX1_383 CORE_EX1_507
  CORE_EX1_610 CORE_EX1_611 CORE_GIL_577 CORE_GVG_114 CORE_LOOT_368
  CORE_LOOT_413 CORE_NEW1_027 CORE_OG_031 CORE_OG_044 CORE_RLK_086 CORE_SW_068
  CORE_SW_439 CORE_ULD_152 CORE_ULD_280 CORE_YOD_026`
- **W6 — discover/choose-one/combo/overload/freeze (23 + 6 tokens + F5) ✅ (PR #126, 2026-08-08):**
  Overload wiring (Totem Golem / Voltaic Burst / Lightning Bolt / Earth
  Elemental / Lightning Storm), Frostbolt (damage AND freeze — the
  Icicle-style FreezeOrDamage miswire fixed), Blizzard (damage all enemy
  minions + freeze), Deep Freeze (freeze an enemy + two 3/6 Water
  Elementals; the token inherits the classic Water Elemental
  freeze-on-damage hook), Glaciate (random 8-Cost minion, frozen —
  Discover simplified); choose-one batch of 6 (engine fix: choose-one
  spells always surface the branch choice — the !combo_active guard
  wrongly resolved the battlecry branch after another card was played);
  combo batch of 3 (Defias Ringleader's 2/1 Defias Bandit token
  EX1_131t added — the NEUTRAL_T04 Novice Engineer miswire fixed);
  Discover batch of 6 (Runed Orb / Tracking / Blazing Invocation /
  I Know a Guy / Illidari Studies — new `next_outcast_discount` player
  field, one-time, consumed on play); Lightning Storm's 2–3 random range
  fixed at 2 (registered simplification); the Otter token carries Rush.
  23 F5 differential scenarios.
  `CORE_AT_037 CORE_AT_052 CORE_AV_107 CORE_BAR_541 CORE_BOT_451 CORE_BOT_576
  CORE_BT_072 CORE_CS2_024 CORE_CS2_028 CORE_DS1_184 CORE_EX1_131
  CORE_EX1_134 CORE_EX1_154 CORE_EX1_160 CORE_EX1_238 CORE_EX1_250
  CORE_EX1_259 CORE_GIL_836 CORE_OG_047 CORE_ONY_018 CORE_TSC_650
  CORE_WON_350 CORE_YOP_001`
- **W7 — enrage finish (2 + F5) ✅ (PR #127, 2026-08-08):** Grommash Hellscream (8/4/9, Charge, Enrage +6) and Bloodhoof Brave (4/2/6, Taunt, Enrage +3) — both ride the read-based Enrage component (fidelity-debt §13). `CORE_EX1_414 CORE_OG_218`
- **W8 — special types (6 + F5) ✅ (PR #128, 2026-08-08):** Lord Jaraxxus (`CardType::Hero` + hero-replacement primitive: 15 health, armor lost, Blood Fury 3/8, INFERNO! hero power summoning a 6/6 Infernal), Sanguine Depths (`CardType::Location`: one per side, one-turn play cooldown, one activation per turn, durability charges), the 3 ENCHANTMENT tokens (`CardType::Enchantment`, never playable) and the Imp token. the deferred HERO (CORE_EX1_323 Lord
  Jaraxxus — hero replacement primitive), LOCATION (CORE_REV_990 Sanguine
  Depths — `CardType::Location` primitive), ENCHANTMENT cards (CORE_CATA_006e
  CORE_CS2_039e CORE_EDR_002e) and the Imp token (CORE_GIL_191t).
- **W9 — RL pool switch (orange-reinforcement PR, no cards):** extend
  `_load_debt_ids()` glob to `core_*.rs`; `full_pool()` now includes the
  landed CORE cards (follows the engine pool per D4); re-measure and record
  the new pool size here; archive both roadmap files to `docs/finished/` and
  update the workspace `CLAUDE.md`.

## Definition of done

Every `- [ ]` below is `- [x]`, each card landed with an F5 differential
scenario, the pool/RL decisions above are recorded here, and both files move
to `docs/finished/` with the workspace `CLAUDE.md` updated.
