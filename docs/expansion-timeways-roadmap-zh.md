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
- [x] **W2 —— 时光倒流卡 + 平行线（R2）**（PR #152、#153）：在 R1 上落系列
  时光倒流卡；平行线传说；杂项效果。
- [x] **W3 —— 时间的终结迷你**（PR #154）：迷你卡（机制由数据回填）；
  收尾波清扫衍生物/账本。

## 卡清单

> 由 M0 数据转储回填（D1）。按波分组占位。

### W1/W2 —— 时光倒流 + 平行线
- [x] TIME_000 Semi-Stable Portal
- [x] TIME_001 Chrono Daggers
- [x] TIME_002 Aeon Wizard
- [x] TIME_003 Portal Vanguard
- [x] TIME_004 Conflux Crasher
- [x] TIME_008 Bygone Doomspeaker
- [x] TIME_014 Instant Multiverse
- [x] TIME_018 Mend the Timeline
- [x] TIME_033 Druid of Regrowth
- [x] TIME_034 Stadium Announcer
- [x] TIME_035 Time Machine
- [x] TIME_038 Mister Clocksworth
- [x] TIME_433 Cease to Exist
- [x] TIME_441 Aeon Rend
- [x] TIME_602 Wormhole
- [x] TIME_610 Shadows of Yesterday
- [x] TIME_005 Timethief Rafaam
- [x] TIME_009 Gelbin of Tomorrow
- [x] TIME_020 Broxigar
- [x] TIME_209 Muradin, High King
- [x] TIME_211 Lady Azshara
- [x] TIME_609 Ranger General Sylvanas
- [x] TIME_619 Talanji of the Graves
- [x] TIME_850 Lo'Gosh, Blood Fighter
- [x] TIME_852 Azure Queen Sindragosa
- [x] TIME_875 Garona Halforcen
- [x] TIME_890 Medivh the Hallowed
> note: Lady Azshara is also a Choose One card: TIME_211

### W3 —— 其余卡 + 迷你
- [x] TIME_006 Mirror Dimension
- [x] TIME_013 Farseer Wo
- [x] TIME_015 Hardlight Protector
- [x] TIME_016 Neon Innovation
- [x] TIME_017 Tankgineer
- [x] TIME_019 Manifested Timeways
- [x] TIME_021 Doomsday Prepper
- [x] TIME_022 Perennial Serpent
- [x] TIME_023 Contingency
- [x] TIME_024 Murozond, Unbounded
- [x] TIME_025 Twilight Timehopper
- [x] TIME_026 Entropic Continuity
- [x] TIME_027 Tachyon Barrage
- [x] TIME_028 Fatebreaker
- [x] TIME_029 Ruinous Velocidrake
- [x] TIME_030 Divergence
- [x] TIME_031 RAFAAM LADDER!!
- [x] TIME_032 Chronogor
- [x] TIME_036 Royal Informant
- [x] TIME_037 Disciple of the Dove
- [x] TIME_039 Deja Vu
- [x] TIME_040 Fading Memory
- [x] TIME_041 Futuristic Forefather
- [x] TIME_042 King Maluk
- [x] TIME_043 PMM Infinitizer
- [x] TIME_044 Past Gnomeregan
- [x] TIME_045 Whelp of the Infinite
- [x] TIME_046 Cyborg Patriarch
- [x] TIME_047 Devious Coyote
- [x] TIME_048 Clockwork Rager
- [x] TIME_049 Dangerous Variant
- [x] TIME_050 Sentient Hourglass
- [x] TIME_051 Soldier of the Infinite
- [x] TIME_052 Amber Warden
- [x] TIME_053 Sandmaw
- [x] TIME_054 Time Skipper
- [x] TIME_055 Unknown Voyager
- [x] TIME_056 Whelp of the Bronze
- [x] TIME_057 Wizened Truthseeker
- [x] TIME_058 Paltry Flutterwing
- [x] TIME_059 Living Paradox
- [x] TIME_060 Quantum Destabilizer
- [x] TIME_061 Timeless Causality
- [x] TIME_062 Chronicle Keeper
- [x] TIME_063 Timelord Nozdormu
- [x] TIME_064 Chrono-Lord Deios
- [x] TIME_100 Hourglass Attendant
- [x] TIME_101 Misplaced Pyromancer
- [x] TIME_102 Circadiamancer
- [x] TIME_103 Chromie
- [x] TIME_212 Lightning Rod
- [x] TIME_213 Primordial Overseer
- [x] TIME_214 Flux Revenant
- [x] TIME_215 Thunderquake
- [x] TIME_216 Nascent Bolt
- [x] TIME_217 Stormrook
- [x] TIME_218 Static Shock
- [x] TIME_427 Cleansing Lightspawn
- [x] TIME_428 Yesterloc
- [x] TIME_429 Divine Augur
- [x] TIME_431 Amber Priestess
- [x] TIME_432 Intertwined Fate
- [x] TIME_434 Temporal Traveler
- [x] TIME_435 Eternus
- [x] TIME_436 Past Conflux
- [x] TIME_442 Timeway Warden
- [x] TIME_443 Hounds of Fury
- [x] TIME_444 Time-Lost Glaive
- [x] TIME_446 The Eternal Hold
- [x] TIME_447 Power Word: Barrier
- [x] TIME_448 Solitude
- [x] TIME_449 Lasting Legacy
- [x] TIME_600 Precise Shot
- [x] TIME_601 Arrow Retriever
- [x] TIME_603 Ticking Timebomb
- [x] TIME_605 Epoch Stalker
- [x] TIME_606 Quel'dorei Fletcher
- [x] TIME_611 Timestop
- [x] TIME_612 Blood Draw
- [x] TIME_613 Cryofrozen Champion
- [x] TIME_614 Liferender
- [x] TIME_615 Forgotten Millennium
- [x] TIME_616 Memoriam Manifest
- [x] TIME_617 Chronochiller
- [x] TIME_618 Husk, Eternal Reaper
- [x] TIME_620 Untimely Death
- [x] TIME_700 Chronological Aura
- [x] TIME_701 Waveshaping
- [x] TIME_702 Ebb and Flow
- [x] TIME_703 Endangered Dodo
- [x] TIME_704 Highborne Mentor
- [x] TIME_705 Krona, Keeper of Eons
- [x] TIME_706 The Fins Beyond Time
- [x] TIME_707 Alternate Reality
- [x] TIME_710 Troubled Double
- [x] TIME_711 Flashback
- [x] TIME_712 Dethrone
- [x] TIME_713 Time Adm'ral Hooktail
- [x] TIME_714 Chrono-Lord Epoch
- [x] TIME_715 For Glory!
- [x] TIME_716 Slow Motion
- [x] TIME_720 Soldier of the Bronze
- [x] TIME_730 Kaldorei Cultivator
- [x] TIME_750 Precursory Strike
- [x] TIME_770 Fast Forward
- [x] TIME_810 Past Silvermoon
- [x] TIME_855 Arcane Barrage
- [x] TIME_856 Algeth'ar Instructor
- [x] TIME_857 Alter Time
- [x] TIME_858 Temporal Construct
- [x] TIME_859 Anomalize
- [x] TIME_860 Faceless Enigma
- [x] TIME_861 Timelooper Toki
- [x] TIME_870 Gladiatorial Combat
- [x] TIME_871 Heir of Hereafter
- [x] TIME_872 Undefeated Champion
- [x] TIME_873 Unleash the Crocolisks
- [x] TIME_876 Shapeshifter
- [x] END_000 Eventuality
- [x] END_001 Jagged Edge of Time
- [x] END_002 Wicked Blightspawn
- [x] END_003 Finality
- [x] END_004 Remnant of Rage
- [x] END_005 Bygone Echoes
- [x] END_006 Chronikar
- [x] END_007 Press the Advantage
- [x] END_008 Enduring Roach
- [x] END_009 Splintered Reality
- [x] END_010 Twilight Timereaver
- [x] END_011 Acceleration Aura
- [x] END_012 Hand of Infinity
- [x] END_013 Brutish Endmaw
- [x] END_014 Synchronized Spark
- [x] END_015 Triennium Rex
- [x] END_016 Chronoclaws
- [x] END_017 Battle at the End Time
- [x] END_018 Acolyte of Infinity
- [x] END_019 Endtime Survivor
- [x] END_020 Eternal Toil
- [x] END_021 Dimensional Weaponsmith
- [x] END_022 Time-Twisted Seer
- [x] END_023 Bitter End
- [x] END_024 Flames of Infinity
- [x] END_025 Eternal Firebolt
- [x] END_026 Fragment of Nothing
- [x] END_027 Wings of Eternity
- [x] END_028 For All Time
- [x] END_029 Voodoo Totem
- [x] END_030 Haywire Hornswog
- [x] END_031 Shade of the End Time
- [x] END_032 Winged Aberration
- [x] END_033 Prescient Slitherdrake
- [x] END_034 Crumblecrusher
- [x] END_035 Omen of the End
- [x] END_036 Morchie
- [x] END_037 Endtime Murozond

## 完成定义

清单全部 `- [ ]` → `- [x]`；R1–R3 带 F5 场景落地（重放时序已钉死）；`cargo test`
全绿；子路线图随主文件归档。
