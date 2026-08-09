# 扩展子路线图 —— 逃离紫罗兰监狱 Escape from Violet Hold

> 状态：**活跃**（2026-08-08 创建，2025–2026 扩展主路线图的一部分）。英文镜像：
> `expansion-violet-hold-roadmap.md`。
> 范围：约 145 张卡（2026-07-07，补丁 36.0，圣甲虫年第二系列）。
> 前置：M0 数据管线 + 前四个子路线图（全部共享原语可用）。

## 系列概况

主题：瓦妮莎·范克里夫的团队越狱达拉然最高戒备监狱。机制核实（2026-08-08，
36.0 补丁说明）：

- **破规者 Rulebreakers** —— 打破炉石核心规则的传说随从（示例：重复传说、
  额外抽牌）。这些是针对规则引擎硬编码不变量（卡组上限、抽牌速率、传说唯一性…）
  的逐卡规则例外。

## 引擎原语（本路线图的波）

| # | 原语 | 使用方 | 说明 |
|---|---|---|---|
| V1 | 规则覆写框架 | 破规者卡 | 逐卡对照引擎硬编码不变量；以受限的逐卡例外实现（组件 + 相关规则读取它），不做通用规则脚本层；每个覆写都要复查 RL 观测/合法动作面（决策 D3） |
| V2 | 各类新 CardEffect | 其余卡 | 各波回填 |

## 波计划

每波一个 PR；每张卡带 F5 对拍场景；`sets.rs` 注册；简化带账本行。

- [x] **W0 —— 接线 + 数据**（PR #160）： 本系列 M0 数据；清单回填；保真测试。
- [x] **W1 —— 破规者第一批（V1）**（PR #161）： 覆写框架 + 第一批破规者；F5 场景逐个钉死被
  覆写的不变量（覆写前/后）。
- [x] **W2 —— 破规者第二批 + 收尾**（PR #162）： 其余破规者 + 系列其余卡；收尾波清扫衍生物/
  账本。

## 卡清单

> 由 M0 数据转储回填（D1）。按波分组占位。

### W1/W2 —— 破规者
- [x] JAIL_118 V'ama, Looming Death
- [x] JAIL_122 Jailhouse Manastorm
- [x] JAIL_319 The Skeleton Key
- [x] JAIL_384 Chainbreaker Hogger
- [x] JAIL_397 Commander Beatrix
- [x] JAIL_407 Vanessa the Ringleader
- [x] JAIL_421 Warptooth
- [x] JAIL_430 Azalina Soulsever
- [x] JAIL_443 The Living Plague
- [x] JAIL_446 Blood Doctor Thal'ena
- [x] JAIL_448 Karov the Broken
- [x] JAIL_458 Tiny Pal
- [x] JAIL_500 Slice and Dice
- [x] JAIL_504 Aya, Lotus Kingpin
- [x] JAIL_509 Godfrey the Betrayer
- [x] JAIL_719 Irida Sinseeker
- [x] JAIL_721 Tras'tath, Soul Parasite
- [x] JAIL_800 Mug'Zee
- [x] JAIL_831 King of the Underbelly
- [x] JAIL_850 Warden Maiev
- [x] JAIL_851 Inspector Murloc Holmes
- [x] JAIL_852 Togwaggle, Smuggler King
- [x] JAIL_860 Chef Neth'rek
- [x] JAIL_875 Staff of Trickery
- [x] JAIL_882 R4T-C4TCH3R
- [x] JAIL_887 Zuramat's Prison
- [x] JAIL_906 Moragg
> Rulebreaker wave membership verified per-card text during W1/W2 implementation

### W2 —— 其余卡
- [x] JAIL_007 Sewer Imp
- [x] JAIL_029 Rioter
- [x] JAIL_030 Escape Artist
- [x] JAIL_035 Vigilant Sentry
- [x] JAIL_101 Violet Punisher
- [x] JAIL_123 Breakout Architect
- [x] JAIL_125 Cold Snap
- [x] JAIL_200 Infest the Scullery
- [x] JAIL_201 Secret Ingredient
- [x] JAIL_202 Spiderling
- [x] JAIL_204 Solitary Prisoner
- [x] JAIL_205 Rat Burglar
- [x] JAIL_206 Dark Bribe
- [x] JAIL_225 Nab
- [x] JAIL_303 Ancient Augur
- [x] JAIL_307 Crowd Control
- [x] JAIL_311 Scrappy Defender
- [x] JAIL_312 Contraband Wands
- [x] JAIL_313 Bootleg Alchemist
- [x] JAIL_315 Mystic Misdirection
- [x] JAIL_321 Tricksy Improviser
- [x] JAIL_326 Judgment
- [x] JAIL_327 Reinforcement Aura
- [x] JAIL_328 Scarlet Bruiser
- [x] JAIL_329 Truth Seeker
- [x] JAIL_330 Dalaran Champion
- [x] JAIL_376 Ball and Chain
- [x] JAIL_377 Holy Bola!
- [x] JAIL_379 Spire Security
- [x] JAIL_380 Smuggled Shovel
- [x] JAIL_386 Scramble for Gear
- [x] JAIL_387 Release the Beasts
- [x] JAIL_395 Sewer Swimmer
- [x] JAIL_398 IMPFERNAL!
- [x] JAIL_399 Imp Gang Stooge
- [x] JAIL_432 Mind Sweeper
- [x] JAIL_433 Unshackle Soul
- [x] JAIL_434 Enthralled Shade
- [x] JAIL_435 Rampaging Hound
- [x] JAIL_436 Widow's Bite
- [x] JAIL_440 Tower of Ghouls
- [x] JAIL_441 Drink Blood
- [x] JAIL_442 Disguised Doctor
- [x] JAIL_444 Sawbones
- [x] JAIL_445 Bone Flurry
- [x] JAIL_447 Reckless Detective
- [x] JAIL_450 Corpse Cannon
- [x] JAIL_451 Blood Clone
- [x] JAIL_452 Disguised Detective
- [x] JAIL_453 Jailbird
- [x] JAIL_454 Emergency Surgery
- [x] JAIL_455 Disguised Watchman
- [x] JAIL_456 P1CK-P0K3T
- [x] JAIL_457 Hijacked Securitybot
- [x] JAIL_459 Arachnathid
- [x] JAIL_460 Concealing Confection
- [x] JAIL_461 Disguised Executioner
- [x] JAIL_462 Getaway Hogdriver
- [x] JAIL_470 Lotus Troublemaker
- [x] JAIL_474 Jade Guardians
- [x] JAIL_501 Picklock
- [x] JAIL_502 Alarm-o-Matic
- [x] JAIL_503 Blackpaw's Whip
- [x] JAIL_507 Spiteful Chef
- [x] JAIL_510 Annihilation
- [x] JAIL_511 Spire of Solitude
- [x] JAIL_513 Caged Cranium
- [x] JAIL_514 The Unseen Atlas
- [x] JAIL_515 Shadow Rounds
- [x] JAIL_516 Scarlet Recruiter
- [x] JAIL_703 Gullible Guard
- [x] JAIL_706 Thief's Tools
- [x] JAIL_718 Black Market Auctioneer
- [x] JAIL_720 Lotus Bookie
- [x] JAIL_730 Stardust Scythe
- [x] JAIL_732 Void Soul
- [x] JAIL_733 Vicious Voidscale
- [x] JAIL_734 Hellraiser
- [x] JAIL_735 Code Violet
- [x] JAIL_801 Molten Gold
- [x] JAIL_802 Gallagio Goon
- [x] JAIL_803 Frostshatter
- [x] JAIL_805 Stormfury
- [x] JAIL_806 Hexmarshal
- [x] JAIL_861 Noxious Bribe
- [x] JAIL_866 Lethal Recipe
- [x] JAIL_872 Spider Rider
- [x] JAIL_876 Dig for Freedom
- [x] JAIL_877 Underbelly Network
- [x] JAIL_878 Guard Dog
- [x] JAIL_879 Beast Tripwire
- [x] JAIL_880 Black Market Overseer
- [x] JAIL_881 Arcane Tripwire
- [x] JAIL_883 Activated Golem
- [x] JAIL_890 Captive Nathrezim
- [x] JAIL_891 Void Blast
- [x] JAIL_892 Cosmic Manifestations
- [x] JAIL_909 Defias Wannabe
- [x] JAIL_912 Soothsayer
- [x] JAIL_913 Hold Them Off!
- [x] JAIL_940 Undeath Sentence
- [x] JAIL_941 Holy Embrace
- [x] JAIL_942 Specter of Despair
- [x] JAIL_974 Captured Archmage
- [x] JAIL_986 Frantic Forger
- [x] JAIL_987 Low Security Wing
- [x] JAIL_997 Demonic Confinement
- [x] JAIL_998 Defias Smuggler

## 完成定义

清单全部 `- [ ]` → `- [x]`；V1–V2 带 F5 场景落地；每个覆写与 RL 面的交互已
核查（D3）；`cargo test` 全绿；子路线图随主文件归档——即关闭 2025–2026
主路线图（M5）。
