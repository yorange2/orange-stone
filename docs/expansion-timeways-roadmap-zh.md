# 扩展子路线图 —— 穿越时光之径 Across the Timeways

> 状态：**活跃**（2026-08-08 创建，2025–2026 扩展主路线图的一部分）。英文镜像：
> `expansion-timeways-roadmap.md`。
> 范围：145 张卡（2025-11-04，补丁 34.0）+ **时间的终结** 迷你系列（38 张，
> 2026-01）。前置：M0 数据管线 + 翡翠梦境/安戈洛子路线图（其 P3 选择机制在此复用）。

## 系列概况

主题：克罗米穿越时间线招募英雄对抗穆罗佐德。机制核实（2026-08-08，34.0
补丁说明）：

- **时光倒流 Rewind** —— 时空关键字：重放上一张卡/效果。34.0 补丁把时钟先生从
  "Rewind x3" 削到 "Rewind x2"，证实该关键字可叠加（多次重放）。
- **传说平行线 Fabled** —— 平行时间线的传说随从（一批重新设想的传说；确切卡规
  则由数据确认）。
- 已知招牌卡（已核实）：时钟先生（Rewind x3 → x2）。

## 引擎原语（本路线图的波）

| # | 原语 | 使用方 | 说明 |
|---|---|---|---|
| R1 | 效果重放 / 上张卡历史 | 时光倒流卡 | 每玩家记录上张打出的卡（x2 需更早一张）；重放重新执行所存效果——必须与死亡阶段、目标规则及重放卡自身离场正确交互 |
| R2 | 平行线变体 | 传说平行线 | 平行传说形态——R1 重放机制就绪后多为数据/定义工作；杂项新 CardEffect 回填 |
| R3 | 各类新 CardEffect | 其余卡 | 各波回填 |

## 波计划

每波一个 PR；每张卡带 F5 对拍场景；`sets.rs` 注册；简化带账本行。

- [x] **W0 —— 接线 + 数据**（PR #150）：本系列 M0 数据（M0.1–M0.5）；清单回填；
  per-set dump 保真测试（`across_the_timeways_dump_fidelity`，TIME_ + END_
  前缀，183 张）与生成基线落地。
- [x] **W1 —— 时光倒流（R1）**（PR #151）：`Player.last_played` RewindEntry
  历史（上限 10）+ `cards::rewind` 注册表（全部 17 张的 `rewind_count`
  表、`REWIND_CARD_IDS`）+ `engine::rewind` 重放（push 前快照、计数钳制、
  按序、Rewind 卡为源）；出牌路径：自身效果 → 重放 → push；8 个 `tmw1_*`
  F5 场景钉死时序、x3 叠加、自身排除、快照语义、空槽钳制与死亡阶段交互。
- **W2 —— 时光倒流卡 + 平行线（R2）：** 在 R1 上落系列时光倒流卡；平行线传说；
  杂项效果。
- **W3 —— 时间的终结迷你：** 迷你卡（机制由数据回填）；收尾波清扫衍生物/账本。

## 卡清单

> 由 M0 数据转储回填（D1）。按波分组占位。

### W1/W2 —— 时光倒流 + 平行线
- [ ] TIME_000 Semi-Stable Portal
- [ ] TIME_001 Chrono Daggers
- [ ] TIME_002 Aeon Wizard
- [ ] TIME_003 Portal Vanguard
- [ ] TIME_004 Conflux Crasher
- [ ] TIME_008 Bygone Doomspeaker
- [ ] TIME_014 Instant Multiverse
- [ ] TIME_018 Mend the Timeline
- [ ] TIME_033 Druid of Regrowth
- [ ] TIME_034 Stadium Announcer
- [ ] TIME_035 Time Machine
- [ ] TIME_038 Mister Clocksworth
- [ ] TIME_433 Cease to Exist
- [ ] TIME_441 Aeon Rend
- [ ] TIME_602 Wormhole
- [ ] TIME_610 Shadows of Yesterday
- [ ] TIME_005 Timethief Rafaam
- [ ] TIME_009 Gelbin of Tomorrow
- [ ] TIME_020 Broxigar
- [ ] TIME_209 Muradin, High King
- [ ] TIME_211 Lady Azshara
- [ ] TIME_609 Ranger General Sylvanas
- [ ] TIME_619 Talanji of the Graves
- [ ] TIME_850 Lo'Gosh, Blood Fighter
- [ ] TIME_852 Azure Queen Sindragosa
- [ ] TIME_875 Garona Halforcen
- [ ] TIME_890 Medivh the Hallowed
> note: Lady Azshara is also a Choose One card: TIME_211

### W3 —— 其余卡 + 迷你
- [ ] TIME_006 Mirror Dimension
- [ ] TIME_013 Farseer Wo
- [ ] TIME_015 Hardlight Protector
- [ ] TIME_016 Neon Innovation
- [ ] TIME_017 Tankgineer
- [ ] TIME_019 Manifested Timeways
- [ ] TIME_021 Doomsday Prepper
- [ ] TIME_022 Perennial Serpent
- [ ] TIME_023 Contingency
- [ ] TIME_024 Murozond, Unbounded
- [ ] TIME_025 Twilight Timehopper
- [ ] TIME_026 Entropic Continuity
- [ ] TIME_027 Tachyon Barrage
- [ ] TIME_028 Fatebreaker
- [ ] TIME_029 Ruinous Velocidrake
- [ ] TIME_030 Divergence
- [ ] TIME_031 RAFAAM LADDER!!
- [ ] TIME_032 Chronogor
- [ ] TIME_036 Royal Informant
- [ ] TIME_037 Disciple of the Dove
- [ ] TIME_039 Deja Vu
- [ ] TIME_040 Fading Memory
- [ ] TIME_041 Futuristic Forefather
- [ ] TIME_042 King Maluk
- [ ] TIME_043 PMM Infinitizer
- [ ] TIME_044 Past Gnomeregan
- [ ] TIME_045 Whelp of the Infinite
- [ ] TIME_046 Cyborg Patriarch
- [ ] TIME_047 Devious Coyote
- [ ] TIME_048 Clockwork Rager
- [ ] TIME_049 Dangerous Variant
- [ ] TIME_050 Sentient Hourglass
- [ ] TIME_051 Soldier of the Infinite
- [ ] TIME_052 Amber Warden
- [ ] TIME_053 Sandmaw
- [ ] TIME_054 Time Skipper
- [ ] TIME_055 Unknown Voyager
- [ ] TIME_056 Whelp of the Bronze
- [ ] TIME_057 Wizened Truthseeker
- [ ] TIME_058 Paltry Flutterwing
- [ ] TIME_059 Living Paradox
- [ ] TIME_060 Quantum Destabilizer
- [ ] TIME_061 Timeless Causality
- [ ] TIME_062 Chronicle Keeper
- [ ] TIME_063 Timelord Nozdormu
- [ ] TIME_064 Chrono-Lord Deios
- [ ] TIME_100 Hourglass Attendant
- [ ] TIME_101 Misplaced Pyromancer
- [ ] TIME_102 Circadiamancer
- [ ] TIME_103 Chromie
- [ ] TIME_212 Lightning Rod
- [ ] TIME_213 Primordial Overseer
- [ ] TIME_214 Flux Revenant
- [ ] TIME_215 Thunderquake
- [ ] TIME_216 Nascent Bolt
- [ ] TIME_217 Stormrook
- [ ] TIME_218 Static Shock
- [ ] TIME_427 Cleansing Lightspawn
- [ ] TIME_428 Yesterloc
- [ ] TIME_429 Divine Augur
- [ ] TIME_431 Amber Priestess
- [ ] TIME_432 Intertwined Fate
- [ ] TIME_434 Temporal Traveler
- [ ] TIME_435 Eternus
- [ ] TIME_436 Past Conflux
- [ ] TIME_442 Timeway Warden
- [ ] TIME_443 Hounds of Fury
- [ ] TIME_444 Time-Lost Glaive
- [ ] TIME_446 The Eternal Hold
- [ ] TIME_447 Power Word: Barrier
- [ ] TIME_448 Solitude
- [ ] TIME_449 Lasting Legacy
- [ ] TIME_600 Precise Shot
- [ ] TIME_601 Arrow Retriever
- [ ] TIME_603 Ticking Timebomb
- [ ] TIME_605 Epoch Stalker
- [ ] TIME_606 Quel'dorei Fletcher
- [ ] TIME_611 Timestop
- [ ] TIME_612 Blood Draw
- [ ] TIME_613 Cryofrozen Champion
- [ ] TIME_614 Liferender
- [ ] TIME_615 Forgotten Millennium
- [ ] TIME_616 Memoriam Manifest
- [ ] TIME_617 Chronochiller
- [ ] TIME_618 Husk, Eternal Reaper
- [ ] TIME_620 Untimely Death
- [ ] TIME_700 Chronological Aura
- [ ] TIME_701 Waveshaping
- [ ] TIME_702 Ebb and Flow
- [ ] TIME_703 Endangered Dodo
- [ ] TIME_704 Highborne Mentor
- [ ] TIME_705 Krona, Keeper of Eons
- [ ] TIME_706 The Fins Beyond Time
- [ ] TIME_707 Alternate Reality
- [ ] TIME_710 Troubled Double
- [ ] TIME_711 Flashback
- [ ] TIME_712 Dethrone
- [ ] TIME_713 Time Adm'ral Hooktail
- [ ] TIME_714 Chrono-Lord Epoch
- [ ] TIME_715 For Glory!
- [ ] TIME_716 Slow Motion
- [ ] TIME_720 Soldier of the Bronze
- [ ] TIME_730 Kaldorei Cultivator
- [ ] TIME_750 Precursory Strike
- [ ] TIME_770 Fast Forward
- [ ] TIME_810 Past Silvermoon
- [ ] TIME_855 Arcane Barrage
- [ ] TIME_856 Algeth'ar Instructor
- [ ] TIME_857 Alter Time
- [ ] TIME_858 Temporal Construct
- [ ] TIME_859 Anomalize
- [ ] TIME_860 Faceless Enigma
- [ ] TIME_861 Timelooper Toki
- [ ] TIME_870 Gladiatorial Combat
- [ ] TIME_871 Heir of Hereafter
- [ ] TIME_872 Undefeated Champion
- [ ] TIME_873 Unleash the Crocolisks
- [ ] TIME_876 Shapeshifter
- [ ] END_000 Eventuality
- [ ] END_001 Jagged Edge of Time
- [ ] END_002 Wicked Blightspawn
- [ ] END_003 Finality
- [ ] END_004 Remnant of Rage
- [ ] END_005 Bygone Echoes
- [ ] END_006 Chronikar
- [ ] END_007 Press the Advantage
- [ ] END_008 Enduring Roach
- [ ] END_009 Splintered Reality
- [ ] END_010 Twilight Timereaver
- [ ] END_011 Acceleration Aura
- [ ] END_012 Hand of Infinity
- [ ] END_013 Brutish Endmaw
- [ ] END_014 Synchronized Spark
- [ ] END_015 Triennium Rex
- [ ] END_016 Chronoclaws
- [ ] END_017 Battle at the End Time
- [ ] END_018 Acolyte of Infinity
- [ ] END_019 Endtime Survivor
- [ ] END_020 Eternal Toil
- [ ] END_021 Dimensional Weaponsmith
- [ ] END_022 Time-Twisted Seer
- [ ] END_023 Bitter End
- [ ] END_024 Flames of Infinity
- [ ] END_025 Eternal Firebolt
- [ ] END_026 Fragment of Nothing
- [ ] END_027 Wings of Eternity
- [ ] END_028 For All Time
- [ ] END_029 Voodoo Totem
- [ ] END_030 Haywire Hornswog
- [ ] END_031 Shade of the End Time
- [ ] END_032 Winged Aberration
- [ ] END_033 Prescient Slitherdrake
- [ ] END_034 Crumblecrusher
- [ ] END_035 Omen of the End
- [ ] END_036 Morchie
- [ ] END_037 Endtime Murozond

## 完成定义

清单全部 `- [ ]` → `- [x]`；R1–R3 带 F5 场景落地（重放时序已钉死）；`cargo test`
全绿；子路线图随主文件归档。
