# 核心系列路线图 —— 实现 281 张 CORE 卡

> 状态：**活跃**（2026-08-07 创建）。英文对照：`core-set-roadmap.md`。
> 范围：`cards/cards.json` 里全部 `CORE_*` 卡 —— 现代核心系列，共 281 张
> （173 随从 / 95 法术 / 8 武器 / 1 英雄 / 1 地点 / 3 附魔衍生物），
> 来自 **42 个原始系列**（卡 ID 中段即来源系列前缀）。

## 为什么

核心系列是经典之后最自然的第二个系列：它是面向全玩家的免费替代系列，大量复用
引擎已有的机制（战吼 84、亡语 23、发现 14、光环 9、奥秘 8、抉择 6、过载 5、连击
3……），也正是 pool-open 注册表（`docs/finished/pool-openness.md`）预言的「第二
系列」——非经典卡进池的那一天，封闭性问题就具体化了。

**本路线图就是卡牌清单。** 里程碑/波次规划、机制缺口与下列池/RL 决策，等清单确认
后再定——每张卡一个复选框。

## 卡牌清单（281）

> 卡名沿用数据源的英文名；中文译名按 `classic-cards-zh.md` 的惯例在卡落地时补。
> 附魔衍生物（ENCHANTMENT）与 HERO/LOCATION 卡会列出，但可能单列波次推迟。

### CORE_AT_* — 冠军的试炼（6 张）

- [ ] `CORE_AT_037` Living Roots — 1 费 法术
- [ ] `CORE_AT_052` Totem Golem — 2 费随从 3/4
- [ ] `CORE_AT_055` Flash Heal — 1 费 法术
- [ ] `CORE_AT_062` Ball of Spiders — 3 费 法术
- [ ] `CORE_AT_064` Bash — 2 费 法术
- [ ] `CORE_AT_123` Chillmaw — 7 费随从 6/6

### CORE_AV_* — 奥特兰克的决裂（2 张）

- [ ] `CORE_AV_107` Glaciate — 6 费 法术
- [ ] `CORE_AV_337` Mountain Bear — 7 费随从 5/6

### CORE_BAR_* — 贫瘠之地的锤炼（7 张）

- [ ] `CORE_BAR_310` Lightshower Elemental — 6 费随从 6/6
- [ ] `CORE_BAR_311` Devouring Plague — 3 费 法术
- [ ] `CORE_BAR_313` Priest of An'she — 5 费随从 5/5
- [ ] `CORE_BAR_541` Runed Orb — 2 费 法术
- [ ] `CORE_BAR_801` Wound Prey — 1 费 法术
- [ ] `CORE_BAR_812` Oasis Ally — 3 费 法术
- [ ] `CORE_BAR_878` Veteran Warmedic — 4 费随从 3/5

### CORE_BOT_* — 砰砰计划（4 张）

- [ ] `CORE_BOT_222` Spirit Bomb — 1 费 法术
- [ ] `CORE_BOT_256` Astromancer — 7 费随从 5/5
- [ ] `CORE_BOT_451` Voltaic Burst — 1 费 法术
- [ ] `CORE_BOT_576` Crazed Chemist — 5 费随从 4/4

### CORE_BRM_* — 黑石山的火焰（1 张）

- [ ] `CORE_BRM_013` Quick Shot — 2 费 法术

### CORE_BT_* — 外域的灰烬（18 张）

- [ ] `CORE_BT_035` Chaos Strike — 2 费 法术
- [ ] `CORE_BT_072` Deep Freeze — 7 费 法术
- [ ] `CORE_BT_120` Warmaul Challenger — 3 费随从 1/10
- [ ] `CORE_BT_156` Imprisoned Vilefiend — 2 费随从 3/5
- [ ] `CORE_BT_187` Kayn Sunfury — 4 费随从 3/5
- [ ] `CORE_BT_201` Augmented Porcupine — 3 费随从 2/4
- [ ] `CORE_BT_292` Hand of A'dal — 2 费 法术
- [ ] `CORE_BT_321` Netherwalker — 2 费随从 2/2
- [ ] `CORE_BT_351` Battlefiend — 1 费随从 1/2
- [ ] `CORE_BT_416` Raging Felscreamer — 4 费随从 4/4
- [ ] `CORE_BT_480` Crimson Sigil Runner — 1 费随从 1/1
- [ ] `CORE_BT_491` Spectral Sight — 2 费 法术
- [ ] `CORE_BT_493` Priestess of Fury — 7 费随从 6/7
- [ ] `CORE_BT_510` Wrathspike Brute — 5 费随从 3/6
- [ ] `CORE_BT_701` Spymistress — 1 费随从 3/1
- [ ] `CORE_BT_781` Bulwark of Azzinoth — 3 费武器 1/4
- [ ] `CORE_BT_801` Eye Beam — 3 费 法术
- [ ] `CORE_BT_921` Aldrachi Warblades — 3 费武器 2/2

### CORE_CATA_* — 大灾变（迷你）（18 张）

- [ ] `CORE_CATA_001` Tichondrius — 9 费随从 9/8
- [ ] `CORE_CATA_002` Calia Menethil — 6 费随从 4/5
- [ ] `CORE_CATA_003` Vindicator Maraad — 3 费随从 2/4
- [ ] `CORE_CATA_004` Rehgar Earthfury — 5 费随从 3/5
- [ ] `CORE_CATA_005` Lord Thorval — 3 费随从 2/4
- [ ] `CORE_CATA_006` Ulfar — 6 费随从 4/3
- [ ] `CORE_CATA_006e` Thornspeakers' Spirit — 附魔衍生物（0 费）
- [ ] `CORE_CATA_007` Consumption — 4 费 法术
- [ ] `CORE_CATA_008` Guardian Gargoyle — 3 费随从 2/4
- [ ] `CORE_CATA_009` Death's Advance — 2 费 法术
- [ ] `CORE_CATA_010` Felscarred Seeker — 3 费随从 2/4
- [ ] `CORE_CATA_011` Ring of Frost — 3 费 法术
- [ ] `CORE_CATA_012` Forbidden Ritual — 3 费 法术
- [ ] `CORE_CATA_013` Slimescale Myrmidon — 3 费随从 2/4
- [ ] `CORE_CATA_014` Crystalweaver — 3 费随从 2/4
- [ ] `CORE_CATA_015` Tricky Geist — 3 费随从 2/4
- [ ] `CORE_CATA_016` Wild Synthesis — 3 费 法术
- [ ] `CORE_CATA_017` Essence Break — 3 费 法术

### CORE_CFM_* — 龙争虎斗加基森（6 张）

- [ ] `CORE_CFM_344` Finja, the Flying Star — 5 费随从 3/5
- [ ] `CORE_CFM_604` Greater Healing Potion — 4 费 法术
- [ ] `CORE_CFM_670` Mayor Noggenfogger — 9 费随从 5/4
- [ ] `CORE_CFM_753` Grimestreet Outfitter — 2 费随从 2/2
- [ ] `CORE_CFM_781` Shaku, the Collector — 3 费随从 2/4
- [ ] `CORE_CFM_790` Dirty Rat — 2 费随从 2/6

### CORE_CS1_* — 基础系列（2 张）

- [ ] `CORE_CS1_112` Holy Nova — 3 费 法术
- [ ] `CORE_CS1_130` Holy Smite — 1 费 法术

### CORE_CS2_* — 基础系列（22 张）

- [ ] `CORE_CS2_004` Power Word: Shield — 1 费 法术
- [ ] `CORE_CS2_009` Mark of the Wild — 2 费 法术
- [ ] `CORE_CS2_023` Arcane Intellect — 3 费 法术
- [ ] `CORE_CS2_024` Frostbolt — 2 费 法术
- [ ] `CORE_CS2_028` Blizzard — 6 费 法术
- [ ] `CORE_CS2_029` Fireball — 4 费 法术
- [ ] `CORE_CS2_032` Flamestrike — 7 费 法术
- [ ] `CORE_CS2_039e` Windfury — 附魔衍生物（0 费）
- [ ] `CORE_CS2_042` Fire Elemental — 6 费随从 6/5
- [ ] `CORE_CS2_053` Far Sight — 3 费 法术
- [ ] `CORE_CS2_062` Hellfire — 3 费 法术
- [ ] `CORE_CS2_072` Backstab — 0 费 法术
- [ ] `CORE_CS2_074` Deadly Poison — 1 费 法术
- [ ] `CORE_CS2_076` Assassinate — 4 费 法术
- [ ] `CORE_CS2_093` Consecration — 3 费 法术
- [ ] `CORE_CS2_094` Hammer of Wrath — 3 费 法术
- [ ] `CORE_CS2_108` Execute — 1 费 法术
- [ ] `CORE_CS2_122` Raid Leader — 3 费随从 2/3
- [ ] `CORE_CS2_179` Sen'jin Shieldmasta — 4 费随从 3/5
- [ ] `CORE_CS2_188` Abusive Sergeant — 1 费随从 1/1
- [ ] `CORE_CS2_189` Elven Archer — 1 费随从 1/1
- [ ] `CORE_CS2_222` Stormwind Champion — 7 费随从 7/7

### CORE_DAL_* — 暗影崛起（2 张）

- [ ] `CORE_DAL_575` Khadgar — 2 费随从 2/2
- [ ] `CORE_DAL_720` Waggle Pick — 4 费武器 4/2

### CORE_DMF_* — 疯狂的暗月马戏团（2 张）

- [ ] `CORE_DMF_067` Prize Vendor — 2 费随从 2/3
- [ ] `CORE_DMF_511` Foxy Fraud — 2 费随从 3/2

### CORE_DRG_* — 巨龙降临（5 张）

- [ ] `CORE_DRG_024` Sky Raider — 1 费随从 1/2
- [ ] `CORE_DRG_079` Evasive Wyrm — 6 费随从 5/4
- [ ] `CORE_DRG_107` Violet Spellwing — 1 费随从 2/1
- [ ] `CORE_DRG_256` Dragonbane — 4 费随从 3/5
- [ ] `CORE_DRG_403` Blowtorch Saboteur — 3 费随从 3/3

### CORE_DS1_* — 基础系列（猎人）（2 张）

- [ ] `CORE_DS1_184` Tracking — 1 费 法术
- [ ] `CORE_DS1_185` Arcane Shot — 1 费 法术

### CORE_EDR_* — 来源系列名待补（EDR）（6 张）

- [ ] `CORE_EDR_001` Babbling Bookcase — 3 费随从 2/4
- [ ] `CORE_EDR_002` Poison Breath — 2 费 法术
- [ ] `CORE_EDR_002e` Deathly Poison — 附魔衍生物（0 费）
- [ ] `CORE_EDR_003` Falric — 3 费随从 2/4
- [ ] `CORE_EDR_004` Raptor Herald — 3 费随从 4/2
- [ ] `CORE_EDR_004_2026` Raptor Herald — 3 费随从 4/2

### CORE_ETC_* — 来源系列名待补（ETC）（2 张）

- [ ] `CORE_ETC_111` Merch Seller — 4 费随从 3/5
- [ ] `CORE_ETC_523` Death Metal Knight — 3 费随从 3/4

### CORE_EX1_* — 经典系列（55 张）

- [ ] `CORE_EX1_002` The Black Knight — 4 费随从 4/4
- [ ] `CORE_EX1_005` Big Game Hunter — 4 费随从 4/2
- [ ] `CORE_EX1_007` Acolyte of Pain — 3 费随从 1/4
- [ ] `CORE_EX1_010` Worgen Infiltrator — 1 费随从 2/1
- [ ] `CORE_EX1_011` Voodoo Doctor — 1 费随从 2/1
- [ ] `CORE_EX1_012` Bloodmage Thalnos — 2 费随从 1/1
- [ ] `CORE_EX1_014` King Mukla — 3 费随从 5/6
- [ ] `CORE_EX1_028` Stranglethorn Tiger — 5 费随从 5/5
- [ ] `CORE_EX1_043` Twilight Drake — 4 费随从 4/1
- [ ] `CORE_EX1_058` Sunfury Protector — 2 费随从 2/3
- [ ] `CORE_EX1_059` Crazed Alchemist — 2 费随从 2/2
- [ ] `CORE_EX1_082` Mad Bomber — 2 费随从 3/2
- [ ] `CORE_EX1_096` Loot Hoarder — 2 费随从 2/1
- [ ] `CORE_EX1_100` Lorewalker Cho — 2 费随从 0/4
- [ ] `CORE_EX1_103` Coldlight Seer — 3 费随从 2/3
- [ ] `CORE_EX1_110` Cairne Bloodhoof — 6 费随从 5/5
- [ ] `CORE_EX1_129` Fan of Knives — 2 费 法术
- [ ] `CORE_EX1_131` Defias Ringleader — 2 费随从 3/2
- [ ] `CORE_EX1_134` SI:7 Agent — 3 费随从 3/3
- [ ] `CORE_EX1_145` Preparation — 0 费 法术
- [ ] `CORE_EX1_154` Wrath — 2 费 法术
- [ ] `CORE_EX1_160` Power of the Wild — 2 费 法术
- [ ] `CORE_EX1_162` Dire Wolf Alpha — 2 费随从 2/2
- [ ] `CORE_EX1_169` Innervate — 0 费 法术
- [ ] `CORE_EX1_189` Brightwing — 2 费随从 3/2
- [ ] `CORE_EX1_193` Psychic Conjurer — 1 费随从 1/2
- [ ] `CORE_EX1_197` Shadow Word: Ruin — 4 费 法术
- [ ] `CORE_EX1_198` Natalie Seline — 7 费随从 7/1
- [ ] `CORE_EX1_238` Lightning Bolt — 1 费 法术
- [ ] `CORE_EX1_246` Hex — 3 费 法术
- [ ] `CORE_EX1_250` Earth Elemental — 5 费随从 7/9
- [ ] `CORE_EX1_259` Lightning Storm — 3 费 法术
- [ ] `CORE_EX1_278` Shiv — 2 费 法术
- [ ] `CORE_EX1_287` Counterspell — 3 费 法术
- [ ] `CORE_EX1_289` Ice Barrier — 3 费 法术
- [ ] `CORE_EX1_302` Mortal Coil — 1 费 法术
- [ ] `CORE_EX1_309` Siphon Soul — 4 费 法术
- [ ] `CORE_EX1_310` Doomguard — 5 费随从 5/7
- [ ] `CORE_EX1_312` Twisting Nether — 8 费 法术
- [ ] `CORE_EX1_319` Flame Imp — 1 费随从 3/2
- [ ] `CORE_EX1_323` Lord Jaraxxus — 英雄（8 费）
- [ ] `CORE_EX1_362` Argent Protector — 2 费随从 3/2
- [ ] `CORE_EX1_383` Tirion Fordring — 8 费随从 8/8
- [ ] `CORE_EX1_391` Slam — 1 费 法术
- [ ] `CORE_EX1_414` Grommash Hellscream — 8 费随从 4/9
- [ ] `CORE_EX1_506` Murloc Tidehunter — 2 费随从 2/1
- [ ] `CORE_EX1_506a` Murloc Scout — 1 费随从 1/1
- [ ] `CORE_EX1_507` Murloc Warleader — 3 费随从 3/3
- [ ] `CORE_EX1_509` Murloc Tidecaller — 1 费随从 1/2
- [ ] `CORE_EX1_559` Archmage Antonidas — 7 费随从 5/7
- [ ] `CORE_EX1_604` Frothing Berserker — 3 费随从 2/4
- [ ] `CORE_EX1_606` Shield Block — 2 费 法术
- [ ] `CORE_EX1_610` Explosive Trap — 2 费 法术
- [ ] `CORE_EX1_611` Freezing Trap — 2 费 法术
- [ ] `CORE_EX1_619` Equality — 2 费 法术

### CORE_GIL_* — 女巫森林（8 张）

- [ ] `CORE_GIL_191t` Imp — 1 费随从 1/1
- [ ] `CORE_GIL_531` Witch's Apprentice — 0 费随从 0/1
- [ ] `CORE_GIL_534` Hench-Clan Thug — 3 费随从 3/3
- [ ] `CORE_GIL_558` Swamp Leech — 1 费随从 2/1
- [ ] `CORE_GIL_577` Rat Trap — 2 费 法术
- [ ] `CORE_GIL_622` Lifedrinker — 4 费随从 3/3
- [ ] `CORE_GIL_623` Witchwood Grizzly — 5 费随从 3/12
- [ ] `CORE_GIL_836` Blazing Invocation — 1 费 法术

### CORE_GVG_* — 地精大战侏儒（5 张）

- [ ] `CORE_GVG_059` Coghammer — 3 费武器 2/3
- [ ] `CORE_GVG_061` Muster for Battle — 3 费 法术
- [ ] `CORE_GVG_085` Annoy-o-Tron — 2 费随从 1/2
- [ ] `CORE_GVG_103` Micro Machine — 2 费随从 1/2
- [ ] `CORE_GVG_114` Sneed's Old Shredder — 7 费随从 5/7

### CORE_ICC_* — 冰封王座的骑士（5 张）

- [ ] `CORE_ICC_038` Righteous Protector — 1 费随从 1/1
- [ ] `CORE_ICC_055` Drain Soul — 2 费 法术
- [ ] `CORE_ICC_210` Shadow Ascendant — 2 费随从 2/3
- [ ] `CORE_ICC_214` Obsidian Statue — 9 费随从 4/8
- [ ] `CORE_ICC_407` Gnomeferatu — 2 费随从 2/3

### CORE_KAR_* — 卡拉赞之夜（5 张）

- [ ] `CORE_KAR_057` Ivory Knight — 4 费随从 4/4
- [ ] `CORE_KAR_061` The Curator — 5 费随从 4/6
- [ ] `CORE_KAR_062` Netherspite Historian — 2 费随从 2/3
- [ ] `CORE_KAR_069` Swashburglar — 1 费随从 1/2
- [ ] `CORE_KAR_077` Silvermoon Portal — 3 费 法术

### CORE_LOE_* — 探险者协会（1 张）

- [ ] `CORE_LOE_039` Gorillabot A-3 — 3 费随从 3/4

### CORE_LOOT_* — 狗头人与地下世界（8 张）

- [ ] `CORE_LOOT_013` Vulgar Homunculus — 2 费随从 2/4
- [ ] `CORE_LOOT_044` Bladed Gauntlet — 2 费武器 0/2
- [ ] `CORE_LOOT_101` Explosive Runes — 3 费 法术
- [ ] `CORE_LOOT_137` Sleepy Dragon — 9 费随从 6/12
- [ ] `CORE_LOOT_309` Oaken Summons — 4 费 法术
- [ ] `CORE_LOOT_368` Voidlord — 9 费随从 3/9
- [ ] `CORE_LOOT_373` Healing Rain — 3 费 法术
- [ ] `CORE_LOOT_413` Plated Beetle — 2 费随从 2/3

### CORE_NEW1_* — 经典补充（7 张）

- [ ] `CORE_NEW1_018` Bloodsail Raider — 2 费随从 2/3
- [ ] `CORE_NEW1_020` Wild Pyromancer — 2 费随从 3/2
- [ ] `CORE_NEW1_021` Doomsayer — 2 费随从 0/7
- [ ] `CORE_NEW1_022` Dread Corsair — 4 费随从 3/3
- [ ] `CORE_NEW1_023` Faerie Dragon — 2 费随从 3/2
- [ ] `CORE_NEW1_027` Southsea Captain — 3 费随从 3/3
- [ ] `CORE_NEW1_031` Animal Companion — 3 费 法术

### CORE_NX2_* — 来源系列名待补（NX2）（1 张）

- [ ] `CORE_NX2_028` Hookfist-3000 — 3 费随从 4/3

### CORE_OG_* — 上古之神的低语（6 张）

- [ ] `CORE_OG_031` Hammer of Twilight — 5 费武器 4/2
- [ ] `CORE_OG_044` Fandral Staghelm — 4 费随从 3/6
- [ ] `CORE_OG_047` Feral Rage — 3 费 法术
- [ ] `CORE_OG_149` Ravaging Ghoul — 3 费随从 3/3
- [ ] `CORE_OG_211` Call of the Wild — 8 费 法术
- [ ] `CORE_OG_218` Bloodhoof Brave — 4 费随从 2/6

### CORE_ONY_* — 奥妮克希亚的巢穴（迷你）（2 张）

- [ ] `CORE_ONY_018` Boomkin — 5 费随从 4/5
- [ ] `CORE_ONY_022` Battle Vicar — 2 费随从 1/3

### CORE_REV_* — 来源系列名待补（REV）（4 张）

- [ ] `CORE_REV_023` Demolition Renovator — 3 费随从 3/3
- [ ] `CORE_REV_308` Maze Guide — 2 费随从 1/1
- [ ] `CORE_REV_946` Steamcleaner — 5 费随从 5/5
- [ ] `CORE_REV_990` Sanguine Depths — 地点（1 费）

### CORE_RLK_* — 巫妖王的进军（17 张）

- [ ] `CORE_RLK_062` Nerubian Swarmguard — 4 费随从 1/3
- [ ] `CORE_RLK_063` Frostwyrm's Fury — 7 费 法术
- [ ] `CORE_RLK_066` Hematurge — 2 费随从 2/3
- [ ] `CORE_RLK_083` Deathchiller — 2 费随从 2/3
- [ ] `CORE_RLK_086` Frostmourne — 6 费武器 4/3
- [ ] `CORE_RLK_087` Asphyxiate — 3 费 法术
- [ ] `CORE_RLK_116` Necrotic Mortician — 2 费随从 2/3
- [ ] `CORE_RLK_118` Tomb Guardians — 4 费 法术
- [ ] `CORE_RLK_121` Acolyte of Death — 3 费随从 2/4
- [ ] `CORE_RLK_505` Marrow Manipulator — 6 费随从 5/5
- [ ] `CORE_RLK_506` Boneguard Commander — 8 费随从 8/8
- [ ] `CORE_RLK_567` Shadow of Demise — 0 费 法术
- [ ] `CORE_RLK_657` Underking — 7 费随从 6/6
- [ ] `CORE_RLK_706` Alexandros Mograine — 7 费随从 7/7
- [ ] `CORE_RLK_712` Blood Tap — 2 费 法术
- [ ] `CORE_RLK_745` Malignant Horror — 4 费随从 2/4
- [ ] `CORE_RLK_814` Crystalsmith Cultist — 1 费随从 1/2

### CORE_SCH_* — 通灵学园（5 张）

- [ ] `CORE_SCH_181` Archwitch Willow — 8 费随从 5/5
- [ ] `CORE_SCH_512` Initiation — 6 费 法术
- [ ] `CORE_SCH_605` Lake Thresher — 5 费随从 4/6
- [ ] `CORE_SCH_713` Cult Neophyte — 2 费随从 3/2
- [ ] `CORE_SCH_717` Keymaster Alabaster — 7 费随从 6/8

### CORE_SW_* — 暴风城下的集结（9 张）

- [ ] `CORE_SW_047` Highlord Fordragon — 6 费随从 5/5
- [ ] `CORE_SW_066` Royal Librarian — 4 费随从 4/4
- [ ] `CORE_SW_068` Mo'arg Forgefiend — 8 费随从 8/8
- [ ] `CORE_SW_072` Rustrot Viper — 3 费随从 3/4
- [ ] `CORE_SW_088` Demonic Assault — 4 费 法术
- [ ] `CORE_SW_108` First Flame — 1 费 法术
- [ ] `CORE_SW_429` Best in Shell — 6 费 法术
- [ ] `CORE_SW_439` Vibrant Squirrel — 1 费随从 2/1
- [ ] `CORE_SW_442` Void Shard — 4 费 法术

### CORE_TID_* — 来源系列名待补（TID）（1 张）

- [ ] `CORE_TID_931` Jackpot! — 2 费 法术

### CORE_TRL_* — 拉斯塔哈的大乱斗（5 张）

- [ ] `CORE_TRL_111` Headhunter's Hatchet — 2 费武器 2/2
- [ ] `CORE_TRL_240` Savage Striker — 2 费随从 2/3
- [ ] `CORE_TRL_307` Flash of Light — 2 费 法术
- [ ] `CORE_TRL_345` Krag'wa, the Frog — 6 费随从 4/6
- [ ] `CORE_TRL_900` Halazzi, the Lynx — 4 费随从 4/2

### CORE_TSC_* — 来源系列名待补（TSC）（2 张）

- [ ] `CORE_TSC_076` Immortalized in Stone — 7 费 法术
- [ ] `CORE_TSC_650` Flipper Friends — 5 费 法术

### CORE_TTN_* — 泰坦诸神（2 张）

- [ ] `CORE_TTN_843` Eredar Deceptor — 4 费随从 3/5
- [ ] `CORE_TTN_866` Mythical Terror — 7 费随从 4/10

### CORE_ULD_* — 奥丹姆奇兵（8 张）

- [ ] `CORE_ULD_133` Crystal Merchant — 2 费随从 1/4
- [ ] `CORE_ULD_152` Pressure Plate — 2 费 法术
- [ ] `CORE_ULD_165` Riftcleaver — 6 费随从 7/5
- [ ] `CORE_ULD_178` Siamat — 7 费随从 7/7
- [ ] `CORE_ULD_191` Beaming Sidekick — 1 费随从 1/2
- [ ] `CORE_ULD_271` Injured Tol'vir — 2 费随从 2/6
- [ ] `CORE_ULD_280` Sahket Sapper — 4 费随从 4/4
- [ ] `CORE_ULD_723` Murmy — 1 费随从 1/1

### CORE_UNG_* — 勇闯安戈洛（7 张）

- [ ] `CORE_UNG_084` Fire Plume Phoenix — 4 费随从 3/4
- [ ] `CORE_UNG_205` Glacial Shard — 1 费随从 2/1
- [ ] `CORE_UNG_809` Fire Fly — 1 费随从 1/2
- [ ] `CORE_UNG_848` Primordial Drake — 8 费随从 4/8
- [ ] `CORE_UNG_912` Jeweled Macaw — 1 费随从 1/2
- [ ] `CORE_UNG_928` Tar Creeper — 3 费随从 1/5
- [ ] `CORE_UNG_952` Spikeridged Steed — 5 费 法术

### CORE_WC_* — 来源系列名待补（WC）（2 张）

- [ ] `CORE_WC_042` Wailing Vapor — 1 费随从 1/3
- [ ] `CORE_WC_701` Felrattler — 3 费随从 3/2

### CORE_WON_* — 来源系列名待补（WON）（6 张）

- [ ] `CORE_WON_096` Dark Peddler — 2 费随从 2/3
- [ ] `CORE_WON_141` Menagerie Mug — 3 费随从 3/3
- [ ] `CORE_WON_145` Avatar of Hearthstone — 9 费随从 5/5
- [ ] `CORE_WON_337` Ironforge Portal — 4 费 法术
- [ ] `CORE_WON_350` I Know a Guy — 1 费 法术
- [ ] `CORE_WON_351` Small-Time Buccaneer — 1 费随从 1/2

### CORE_WW_* — 来源系列名待补（WW）（2 张）

- [ ] `CORE_WW_329` Detonation Juggernaut — 4 费随从 3/4
- [ ] `CORE_WW_374` Corpse Farm — 3 费 法术

### CORE_YOD_* — 来源系列名待补（YOD）（1 张）

- [ ] `CORE_YOD_026` Fiendish Servant — 1 费随从 2/1

### CORE_YOP_* — 来源系列名待补（YOP）（2 张）

- [ ] `CORE_YOP_001` Illidari Studies — 1 费 法术
- [ ] `CORE_YOP_034` Runaway Blackwing — 10 费随从 10/10

## 待定决策（规划里程碑时解决）

1. **池的定义** —— `ALL_CARDS` 目前只含经典卡，池封闭不变量守着它。核心系列把池
   扩成「经典 + 核心」：决定 `ALL_CARDS` 是否仍是唯一来源（把 CORE 加进去），还是
   出现第二个注册表；封闭性测试与 `POOL_OPEN_CARDS` 如何互动（读对手区域的核心卡
   同样开放池）。
2. **与经典的重复** —— 很多 CORE 卡是经典卡的「核心版」重印（如 `CORE_EX1_100`
   游学者周卓 vs `LEGENDARY_024`）。决定两个版本是否共存于池（真实炉石共存），还是
   核心版取代；generated-vs-handwritten 按名比对测试（`cards_generated_match_handwritten`）
   无论哪种都会撞。
3. **新机制清单** —— 从数据看：吸血（8）、可交易（6）、突袭（5）、流放（3）、复生
   （2）、剧毒，以及法术伤害豁免/受法术伤害影响（法术伤害管线的交互），外加地点卡
   类型（CORE_REV_990 猩红之渊）与英雄卡（CORE_EX1_323 贾拉克瑟斯）。每项都需要
   引擎原语 + F5 场景才能进波次。
4. **RL 池** —— 今天 `full_pool()` = 396；加入 CORE 卡会改变训练池大小与
   `include_pool_open` 的语义。重新实测，并决定 RL 池是跟随引擎池还是暂保持经典。
5. **波次形状** —— 42 个来源系列不等于 42 个波次。按机制依赖分组（同 W0~W16）：
   先接线、再原语、然后按系列批量。

## 完成标准

下方每个 `- [ ]` 变为 `- [x]`，每张卡带着 F5 差分场景落地，上述池/RL 决策记录在
本文件，两份文件移入 `docs/finished/` 并更新工作区 `CLAUDE.md`。
