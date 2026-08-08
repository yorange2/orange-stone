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

- **W0 —— 接线 + 数据：** 本系列 M0 数据；清单回填；保真测试。
- **W1 —— 破规者第一批（V1）：** 覆写框架 + 第一批破规者；F5 场景逐个钉死被
  覆写的不变量（覆写前/后）。
- **W2 —— 破规者第二批 + 收尾：** 其余破规者 + 系列其余卡；收尾波清扫衍生物/
  账本。

## 卡清单

> 由 M0 数据转储回填（D1）。按波分组占位。

### W1/W2 —— 破规者
- [ ] JAIL_118 V'ama, Looming Death
- [ ] JAIL_122 Jailhouse Manastorm
- [ ] JAIL_319 The Skeleton Key
- [ ] JAIL_384 Chainbreaker Hogger
- [ ] JAIL_397 Commander Beatrix
- [ ] JAIL_407 Vanessa the Ringleader
- [ ] JAIL_421 Warptooth
- [ ] JAIL_430 Azalina Soulsever
- [ ] JAIL_443 The Living Plague
- [ ] JAIL_446 Blood Doctor Thal'ena
- [ ] JAIL_448 Karov the Broken
- [ ] JAIL_458 Tiny Pal
- [ ] JAIL_500 Slice and Dice
- [ ] JAIL_504 Aya, Lotus Kingpin
- [ ] JAIL_509 Godfrey the Betrayer
- [ ] JAIL_719 Irida Sinseeker
- [ ] JAIL_721 Tras'tath, Soul Parasite
- [ ] JAIL_800 Mug'Zee
- [ ] JAIL_831 King of the Underbelly
- [ ] JAIL_850 Warden Maiev
- [ ] JAIL_851 Inspector Murloc Holmes
- [ ] JAIL_852 Togwaggle, Smuggler King
- [ ] JAIL_860 Chef Neth'rek
- [ ] JAIL_875 Staff of Trickery
- [ ] JAIL_882 R4T-C4TCH3R
- [ ] JAIL_887 Zuramat's Prison
- [ ] JAIL_906 Moragg
> Rulebreaker wave membership verified per-card text during W1/W2 implementation

### W2 —— 其余卡
- [ ] JAIL_007 Sewer Imp
- [ ] JAIL_029 Rioter
- [ ] JAIL_030 Escape Artist
- [ ] JAIL_035 Vigilant Sentry
- [ ] JAIL_101 Violet Punisher
- [ ] JAIL_123 Breakout Architect
- [ ] JAIL_125 Cold Snap
- [ ] JAIL_200 Infest the Scullery
- [ ] JAIL_201 Secret Ingredient
- [ ] JAIL_202 Spiderling
- [ ] JAIL_204 Solitary Prisoner
- [ ] JAIL_205 Rat Burglar
- [ ] JAIL_206 Dark Bribe
- [ ] JAIL_225 Nab
- [ ] JAIL_303 Ancient Augur
- [ ] JAIL_307 Crowd Control
- [ ] JAIL_311 Scrappy Defender
- [ ] JAIL_312 Contraband Wands
- [ ] JAIL_313 Bootleg Alchemist
- [ ] JAIL_315 Mystic Misdirection
- [ ] JAIL_321 Tricksy Improviser
- [ ] JAIL_326 Judgment
- [ ] JAIL_327 Reinforcement Aura
- [ ] JAIL_328 Scarlet Bruiser
- [ ] JAIL_329 Truth Seeker
- [ ] JAIL_330 Dalaran Champion
- [ ] JAIL_376 Ball and Chain
- [ ] JAIL_377 Holy Bola!
- [ ] JAIL_379 Spire Security
- [ ] JAIL_380 Smuggled Shovel
- [ ] JAIL_386 Scramble for Gear
- [ ] JAIL_387 Release the Beasts
- [ ] JAIL_395 Sewer Swimmer
- [ ] JAIL_398 IMPFERNAL!
- [ ] JAIL_399 Imp Gang Stooge
- [ ] JAIL_432 Mind Sweeper
- [ ] JAIL_433 Unshackle Soul
- [ ] JAIL_434 Enthralled Shade
- [ ] JAIL_435 Rampaging Hound
- [ ] JAIL_436 Widow's Bite
- [ ] JAIL_440 Tower of Ghouls
- [ ] JAIL_441 Drink Blood
- [ ] JAIL_442 Disguised Doctor
- [ ] JAIL_444 Sawbones
- [ ] JAIL_445 Bone Flurry
- [ ] JAIL_447 Reckless Detective
- [ ] JAIL_450 Corpse Cannon
- [ ] JAIL_451 Blood Clone
- [ ] JAIL_452 Disguised Detective
- [ ] JAIL_453 Jailbird
- [ ] JAIL_454 Emergency Surgery
- [ ] JAIL_455 Disguised Watchman
- [ ] JAIL_456 P1CK-P0K3T
- [ ] JAIL_457 Hijacked Securitybot
- [ ] JAIL_459 Arachnathid
- [ ] JAIL_460 Concealing Confection
- [ ] JAIL_461 Disguised Executioner
- [ ] JAIL_462 Getaway Hogdriver
- [ ] JAIL_470 Lotus Troublemaker
- [ ] JAIL_474 Jade Guardians
- [ ] JAIL_501 Picklock
- [ ] JAIL_502 Alarm-o-Matic
- [ ] JAIL_503 Blackpaw's Whip
- [ ] JAIL_507 Spiteful Chef
- [ ] JAIL_510 Annihilation
- [ ] JAIL_511 Spire of Solitude
- [ ] JAIL_513 Caged Cranium
- [ ] JAIL_514 The Unseen Atlas
- [ ] JAIL_515 Shadow Rounds
- [ ] JAIL_516 Scarlet Recruiter
- [ ] JAIL_703 Gullible Guard
- [ ] JAIL_706 Thief's Tools
- [ ] JAIL_718 Black Market Auctioneer
- [ ] JAIL_720 Lotus Bookie
- [ ] JAIL_730 Stardust Scythe
- [ ] JAIL_732 Void Soul
- [ ] JAIL_733 Vicious Voidscale
- [ ] JAIL_734 Hellraiser
- [ ] JAIL_735 Code Violet
- [ ] JAIL_801 Molten Gold
- [ ] JAIL_802 Gallagio Goon
- [ ] JAIL_803 Frostshatter
- [ ] JAIL_805 Stormfury
- [ ] JAIL_806 Hexmarshal
- [ ] JAIL_861 Noxious Bribe
- [ ] JAIL_866 Lethal Recipe
- [ ] JAIL_872 Spider Rider
- [ ] JAIL_876 Dig for Freedom
- [ ] JAIL_877 Underbelly Network
- [ ] JAIL_878 Guard Dog
- [ ] JAIL_879 Beast Tripwire
- [ ] JAIL_880 Black Market Overseer
- [ ] JAIL_881 Arcane Tripwire
- [ ] JAIL_883 Activated Golem
- [ ] JAIL_890 Captive Nathrezim
- [ ] JAIL_891 Void Blast
- [ ] JAIL_892 Cosmic Manifestations
- [ ] JAIL_909 Defias Wannabe
- [ ] JAIL_912 Soothsayer
- [ ] JAIL_913 Hold Them Off!
- [ ] JAIL_940 Undeath Sentence
- [ ] JAIL_941 Holy Embrace
- [ ] JAIL_942 Specter of Despair
- [ ] JAIL_974 Captured Archmage
- [ ] JAIL_986 Frantic Forger
- [ ] JAIL_987 Low Security Wing
- [ ] JAIL_997 Demonic Confinement
- [ ] JAIL_998 Defias Smuggler

## 完成定义

清单全部 `- [ ]` → `- [x]`；V1–V2 带 F5 场景落地；每个覆写与 RL 面的交互已
核查（D3）；`cargo test` 全绿；子路线图随主文件归档——即关闭 2025–2026
主路线图（M5）。
