# 扩展子路线图 —— 失落之城安戈洛 The Lost City of Un'Goro

> 状态：**活跃**（2026-08-08 创建，2025–2026 扩展主路线图的一部分）。英文镜像：
> `expansion-ungoro-roadmap.md`。
> 范围：145 张卡（2025-07-08，补丁 33.0）+ **恐龙节** 迷你系列（38 张）。
> 前置：M0 数据管线 + 翡翠梦境子路线图（其 P1/P2 原语在此复用）。

## 系列概况

主题：伊莉斯·逐星重返安戈洛，寻找失落的托托利安城。机制核实（2026-08-08，
33.0 补丁说明）：

- **任务回归** —— 任务卡打入任务槽；按任务进度累积；完成即领奖励。引擎
  **完全没有任务区**——这是全新的区域/组件，是本年度最大的原语。
- **同族 Kindred** —— 逐卡触发关键字（补丁说明示例："Kindred: It costs (1)
  less"；确切触发集按卡文本回填）。
- 背景：145 张卡；任务机制上一次出现在 2020 年前（2017 安戈洛原版），引擎
  无先例可循。

## 引擎原语（本路线图的波）

| # | 原语 | 使用方 | 说明 |
|---|---|---|---|
| Q1 | 任务区 + 进度 + 奖励 | 全部任务卡 | `Zone::Quest`（或每玩家任务槽）、按任务条件的进度事件、完成奖励效果；任务槽互斥（每玩家一任务）与观测里的进度暴露 |
| Q2 | 同族计数 | 同族卡 | 逐卡触发计数（确切条件来自文本——降费、增益或死亡追踪）；与 `cards_played_this_turn` 风格计数交互 |
| Q3 | 各类新 CardEffect | 其余卡 | 各波回填 |

## 波计划

每波一个 PR；每张卡带 F5 对拍场景；`sets.rs` 注册；简化带账本行。

- [x] **W0 —— 接线 + 数据**（PR #143）：本系列 M0 数据（M0.1–M0.5）；清单回填；
  per-set dump 保真测试（`the_lost_city_dump_fidelity`，TLC_ + DINO_ 前缀，
  183 张）与生成基线落地。
- [x] **W1 —— 任务区（Q1）**（PR #144）：`Zone::Quest` 每玩家任务槽 + `Quest`
  组件 + `cards::quest` 注册表（SpellSchool、QuestCondition、全部 11 张任务
  的 `quest_def`）+ `engine::quest` 进度派发（marker 去重、奖励结算；可重复
  任务重置留槽）；出牌路径把任务卡导入任务槽（每玩家一任务——新任务销毁旧
  任务）；接线调用点：出牌/召唤/伤害/回合结束/施法/尸体/发现；落地 7 个
  引擎级 `tlc_w1_*` 冒烟场景（任务卡落地前）。
- [x] **W2 —— 任务卡**（PR #145）：全部 11 张任务卡（每职业一张）+ 12 个
  奖励 token 落 `exp_tlc_w2.rs`；TLC_817 双进度条（`QuestDef::second`——
  两栏全完成才离槽）、TLC_426 可重复任务（真实永久鱼人 +1/+1 旗标）、
  TLC_631 永久"恰好 2 点伤害"旗标、Everbloom 武器触发、Sol'etos 双半；
  Temporary 原语（标记 + 回合末弃置）使 TLC_446 可行；简化登记 §15（en+zh）；
  12 个 `tlc_w2_*` F5 场景钉死 进度→奖励 时序与每玩家一任务（真实卡级）；
  W1 学派场景更新为真实双栏语义。
- [x] **W3 —— 同族（Q2）**（PR #146）：机制按卡文本核实——"Kindred: X"
  在本回合早先打出同类卡时激活（随从按种族、法术按卡类型；自身不算）。
  `cards::kindred` 注册表（KindredType/KindredEffect 表，23 张）+ 玩家级
  `kindred_played` 状态（回合末清空）+ `next_kindred_twice` 旗标；解析
  形状 OnPlay / CostDiscount / BattlecryModifier + 专用抽牌战吼
  （TLC_102 Torga 扫描、TLC_223/236/432 抽牌修饰）；23 卡 + 3 token 落
  `exp_tlc_w3.rs`；简化 §16（en+zh）；22 个 `tlc_w3_*` F5 场景；
  顺带修复既有 `resolve_destroy_minion` 缺 AnyMinion 臂的缺口。
- **W4 —— 恐龙节迷你：** 迷你卡（机制由数据回填）；收尾波清扫衍生物/账本。

## 卡清单

> 由 M0 数据转储回填（D1）。按波分组占位。

### W1/W2 —— 任务
- [x] TLC_229 Spirit of the Mountain
- [x] TLC_239 Restore the Wild
- [x] TLC_426 Dive the Golakka Depths
- [x] TLC_433 Reanimate the Terror
- [x] TLC_446 Escape the Underfel
- [x] TLC_460 The Forbidden Sequence
- [x] TLC_513 Lie in Wait
- [x] TLC_602 Enter the Lost City
- [x] TLC_631 Unleash the Colossus
- [x] TLC_817 Reach Equilibrium
- [x] TLC_830 The Food Chain

### W3 —— 同族
- [x] TLC_102 Torga
- [x] TLC_107 Stormbrewer
- [x] TLC_223 Volcanic Thrasher
- [x] TLC_226 Conjured Bookkeeper
- [x] TLC_236 Hybridization
- [x] TLC_243 Whirling Stormdrake
- [x] TLC_251 Primalfin Challenger
- [x] TLC_366 Pterrorwing Ravager
- [x] TLC_428 Hot Spring Glider
- [x] TLC_429 Steamfin Thief
- [x] TLC_432 Dread Raptor
- [x] TLC_440 Cryosleep
- [x] TLC_447 Caustic Fumes
- [x] TLC_454 Scalehide Kodo
- [x] TLC_463 Razidir
- [x] TLC_482 Slagclaw
- [x] TLC_519 Ambush Predators
- [x] TLC_600 Windpeak Wyrm
- [x] TLC_815 Gravedawn Voidbulb
- [x] TLC_816 Gravedawn Sunbloom
- [x] TLC_825 Ravasaur Matriarch
- [x] TLC_829 Ravenous Devilsaur
- [x] TLC_903 Silithid Queen

### W4 —— 其余卡 + 迷你
- [ ] TLC_100 Elise the Navigator
- [x] TLC_101 Undercover Cultist
- [ ] TLC_106 Endbringer Umbra
- [x] TLC_109 Relic Miner
- [ ] TLC_110 City Chief Esho
- [x] TLC_220 Windswept Pageturner
- [x] TLC_221 Sizzling Swarm
- [x] TLC_222 Flight of the Firehawk
- [x] TLC_224 Mechanized Magma
- [x] TLC_225 Cinderfin
- [x] TLC_227 Lava Flow
- [ ] TLC_228 Bralma Searstone
- [x] TLC_230 TREEEES!!!
- [x] TLC_231 Story of Barnabus
- [x] TLC_232 Ravenous Flock
- [x] TLC_233 Hatchery Helper
- [x] TLC_234 Eternal Bloodpetal
- [x] TLC_235 Life Cycle
- [x] TLC_237 Skyscreamer Eggs
- [x] TLC_240 Tyrannogill
- [ ] TLC_241 Ido of the Threshfleet
- [x] TLC_242 Ancient Stegodon
- [x] TLC_244 Curious Explorer
- [x] TLC_245 Ancient Raptor
- [x] TLC_246 Ancient Pterrordax
- [x] TLC_247 Primal Sabretooth
- [x] TLC_248 Ultragigasaur
- [x] TLC_249 Sizzling Cinder
- [x] TLC_250 Crater Gator
- [x] TLC_252 Dissolving Ooze
- [x] TLC_253 Petrified Ogre
- [x] TLC_254 Tortollan Storyteller
- [x] TLC_255 Crystal Tender
- [x] TLC_256 Marshland Thresher
- [ ] TLC_257 Loh, the Living Legend
- [x] TLC_334 Relic of Kings
- [x] TLC_364 Story of the Waygate
- [x] TLC_365 Storage Scuffle
- [x] TLC_401 Bonechill Stegodon
- [x] TLC_427 Rockskipper
- [x] TLC_430 Creature of the Sacred Cave
- [x] TLC_434 Paleomancy
- [x] TLC_435 Crypt Map
- [x] TLC_436 Reanimated Pterrordax
- [x] TLC_438 Violet Treasuregill
- [x] TLC_439 Wave of Tar
- [x] TLC_441 Ready the Fleet
- [x] TLC_442 Submerged Map
- [x] TLC_443 Reluctant Wrangler
- [x] TLC_444 Story of Galvadon
- [x] TLC_449 Bloodpetal Biome
- [x] TLC_450 Spelunker
- [x] TLC_451 Cursed Catacombs
- [ ] TLC_452 Titanographer Osk
- [x] TLC_461 Scrappy Scavenger
- [x] TLC_462 Unearthed Artifacts
- [x] TLC_464 Mountain Map
- [x] TLC_465 Stranglevine
- [x] TLC_466 Story of Lakkari
- [x] TLC_467 Whispering Stone
- [x] TLC_468 Blob of Tar
- [x] TLC_469 Tunnel Terror
- [x] TLC_477 Threshrider's Blessing
- [x] TLC_478 Axe of the Forefathers
- [x] TLC_479 Deathrot Maw
- [ ] TLC_480 Krog, Crater King
- [x] TLC_483 Vault Breaker
- [x] TLC_514 Merchant of Legend
- [x] TLC_515 Cultist Map
- [x] TLC_516 Neferset Weaponsmith
- [x] TLC_517 Knockback
- [x] TLC_518 Interrogation
- [x] TLC_520 Underbrush Tracker
- [x] TLC_521 Eyes in the Sky
- [ ] TLC_522 Opu the Unseen
- [x] TLC_601 Shellnado
- [x] TLC_603 Platysaur
- [x] TLC_605 Tar Tyrant
- [x] TLC_606 Latorvian Armorer
- [x] TLC_620 Fortify
- [x] TLC_621 Willful Watcher
- [x] TLC_622 City Defenses
- [x] TLC_623 Stonecarver
- [ ] TLC_624 Nablya, the Watcher
- [x] TLC_630 Gorishi Wasp
- [x] TLC_632 Story of Sulfuras
- [x] TLC_633 Bugsquasher
- [ ] TLC_810 High Cultist Herenn
- [ ] TLC_811 Archaios
- [x] TLC_814 Twilight Mender
- [x] TLC_818 Resuscitate
- [x] TLC_819 Gladesong Siren
- [x] TLC_820 Glade Ecologist
- [x] TLC_821 Wilted Shadow
- [x] TLC_822 Dinositter
- [x] TLC_823 Cower in Fear
- [x] TLC_824 Odd Map
- [x] TLC_826 Story of Carnassa
- [x] TLC_827 Grazing Stegodon
- [x] TLC_828 Supreme Dinomancy
- [x] TLC_831 Pterrordax Egg
- [x] TLC_833 Insect Claw
- [x] TLC_835 Story of Amara
- [ ] TLC_836 Niri of the Crater
- [x] TLC_840 Gorishi Tunneler
- [ ] TLC_841 Entomologist Toru
- [x] TLC_888 Cloud Serpent
- [x] TLC_900 Hive Map
- [x] TLC_901 Fumigate
- [x] TLC_902 Infestation
- [x] TLC_987 Questing Assistant
- [ ] DINO_130 Longneck Egg
- [ ] DINO_131 Possessed Animancer
- [ ] DINO_132 Asphyxiodon
- [ ] DINO_136 Horn of Feasting
- [ ] DINO_137 Skittish Saucier
- [ ] DINO_138 Diabolus Rex
- [ ] DINO_400 Barricade Basher
- [ ] DINO_401 The Great Dracorex
- [ ] DINO_402 Bat Mask
- [ ] DINO_403 Devilsaur Mask
- [ ] DINO_404 Firegill
- [ ] DINO_405 Hatching Ceremony
- [ ] DINO_406 Fire Breath
- [ ] DINO_407 Mirrex, the Crystalline
- [ ] DINO_408 Crystal Tusk
- [ ] DINO_409 Techysaurus
- [ ] DINO_410 The Egg of Khelos
- [ ] DINO_411 Holy Eggbearer
- [ ] DINO_412 Tortotem
- [ ] DINO_413 Chillspine Stegodon
- [ ] DINO_414 Tribute Dance
- [ ] DINO_415 Story of Umbra
- [ ] DINO_416 Hollow Direhorn
- [ ] DINO_417 Soulrest Ceremony
- [ ] DINO_419 Herbivore Assistant
- [ ] DINO_421 Seismopod
- [ ] DINO_422 Ankylodon
- [ ] DINO_424 Hero's Welcome
- [ ] DINO_426 Ritual of Life
- [ ] DINO_427 Costume Merchant
- [ ] DINO_428 Behemoth Mask
- [ ] DINO_429 Sheep Mask
- [ ] DINO_430 Beast Speaker Taka
- [ ] DINO_431 Atlasaurus
- [ ] DINO_432 Panther Mask
- [ ] DINO_433 Guard Duty
- [ ] DINO_434 Raptor-Nest Nurse
- [ ] DINO_435 Crater Experiment

## 完成定义

清单全部 `- [ ]` → `- [x]`；Q1–Q3 带 F5 场景落地；任务区暴露进观测/
`legal_actions`（主路线图决策 D3 适用——池策略决定向 RL 暴露多少）；`cargo test`
全绿；子路线图随主文件归档。
