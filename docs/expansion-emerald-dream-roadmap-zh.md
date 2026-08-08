# 扩展子路线图 —— 翡翠梦境 Into the Emerald Dream

> 状态：**活跃**（2026-08-08 创建，2025–2026 扩展主路线图的一部分）。英文镜像：
> `expansion-emerald-dream-roadmap.md`。
> 范围：145 张卡（2025-03-25，补丁 32.0，猎鹰年首发系列）+ **世界之树的余烬**
> 迷你系列（38 张，2025-05-13）。
> 前置：M0 数据管线（`2025-2026-expansions-roadmap-zh.md`）。

## 系列概况

主题：伊瑟拉的翡翠梦境对抗古神的腐化。机制核实（2026-08-08，官方新闻 + 补丁说明）：

- **灌注 Imbue** —— 德鲁伊 / 猎人 / 法师 / 圣骑士 / 牧师 / 萨满。打出带灌注的卡
  （或达成灌注条件）升级职业英雄技能；第二次灌注触发强化形态。
- **黑暗赠礼 Dark Gifts** —— 死亡骑士 / 恶魔猎手 / 潜行者 / 术士 / 战士。
  发现约 10 种强化中的一种作用于随从。
- **全职业 Choose One** —— 与引擎已登记的自动随机简化的第一次正面碰撞。
- **野神 Wild Gods** —— 每职业一张传说。
- ~~**灼烧 Smoldering**~~ —— **已移除（M0.5，2026-08-08，数据核实）**：hearthstonejson
  迷你数据（FIR_*）中无 Smoldering 关键字，无任何"亡语额外触发一次"类文本；迷你 38 张
  卡是黑暗赠礼（7）、战吼/发现等既有机制的延续。
- 已知招牌卡（已核实）：伊瑟拉·翡翠之貌（登录传说）、炎魔·烈焰化身（迷你传说）。

## 引擎原语（本路线图的波）

| # | 原语 | 使用方 | 说明 |
|---|---|---|---|
| P1 | 灌注：英雄技能升级 | 灌注卡（6 职业） | `HeroPowerDef` 按定义静态；需每玩家升级计数 + 阈值事件 + 英雄技能替换组件（与大灾变死亡之翼的选择共用设计——一次设计，两处复用） |
| P2 | 黑暗赠礼：固定发现池 | 5 职业 | 约 10 种强化衍生物；固定池（如 `DREAM_POOL`/`POOL_OPEN_CARDS` 风格），发现简化→随机选择（除非 D2 另有裁决） |
| P3 | 真实抉择结算 | 全职业 | 清偿自动随机欠债：`Resolution::NeedsChoice` 通过 `legal_actions`/绑定暴露（W10 的发现选择机制即现成范式） |
| ~~P4~~ | ~~灼烧：额外亡语~~ | ~~迷你~~ | **已移除（M0.5）**：数据无 Smoldering 关键字；迷你卡用既有原语实现 |
| P5 | 野神 / 各类新 CardEffect | 各卡 | 各波内按卡文本回填 |

## 波计划

每波一个 PR；每张卡带 F5 对拍场景；卡注册进 `sets.rs::ALL_CARDS`；简化带
`(simplified …)` 标记 + `fidelity-debt.md` 行。

- [x] **W0 —— 接线 + 数据（无卡）**：本系列 M0 数据落地（M0.1–M0.5，PR #131–#135：
  schema 字段、生成基线、清单回填）；ID 前缀 + 名称保真测试（M1-W0，PR #136：
  `tests/expansion_fidelity.rs`，EDR_/FIR_ 双向覆盖 183 张）。
- [x] **W1 —— 灌注（P1）**（PR #137）：每玩家灌注计数 + 首次灌注替换为职业灌注
  形态 + 每次灌注数字 +1（等级 = 计数）；6 职业 15 张灌注卡；13 个 F5 场景钉死
  2 次灌注阈值时序与各职业技能结算。5 项 D2 简化登记进 fidelity-debt.md §14。
- [x] **W2 —— 黑暗赠礼（P2）**（PR #138）：10 种赠礼池（`ALL_DARK_GIFTS`）+ 卡级
  赠礼标记 + 每玩家赠礼日志；5 职业 9 张黑暗赠礼卡；15 个 F5 场景（各赠礼结算、
  跨区存续、Combo/尸体/持龙条件、Wallow 同步）。Nightmare Fuel 登记 POOL_OPEN_CARDS；
  7 项 D2 简化登记进 fidelity-debt.md §14.1。
- [x] **W3 —— 全职业 Choose One（P3）**（PR #139）：自动随机欠债已清偿——选择面
  暴露进 `legal_actions`（逐选项 `Action::Choose` + `ChoicePending` 门）与结构化
  视图/绑定；法术/随从/武器三个 surface 点（武器路径新增）+ 入队事件排序修复；
  12 张 EDR 抉择卡 + 6 衍生物；14 个 F5 场景钉死两分支；4 项 D2 简化登记进
  fidelity-debt.md §14.2。
- **W4 —— 野神 + 杂项效果（P5）：** 其余随从/法术；职业传说野神；收尾波清扫
  衍生物/附魔与账本。
- **W5 —— 世界之树的余烬迷你：** 迷你 38 张卡（既有原语：黑暗赠礼补充 7 张等）；
  F5 场景；迷你注册。

## 卡清单

> 由 M0 数据转储回填（D1）。按承载机制的波分组占位；每个 `- [ ]` 一张卡。

### W1 —— 灌注（德鲁伊/猎人/法师/圣骑士/牧师/萨满）
- [x] EDR_226 Exotic Houndmaster
- [x] EDR_227 Umbraclaw
- [x] EDR_231 Aspect's Embrace
- [x] EDR_264 Aegis of Light
- [x] EDR_449 Lunarwing Messenger
- [x] EDR_451 Goldpetal Drake
- [x] EDR_518 Living Garden
- [x] EDR_519 Wisprider
- [x] EDR_800 Flutterwing Guardian
- [x] EDR_845 Hamuul Runetotem
- [x] EDR_852 Bitterbloom Knight
- [x] EDR_860 Resplendent Dreamweaver
- [x] EDR_871 Spirit Gatherer
- [x] EDR_888 Malorne the Waywatcher
- [x] EDR_970 Kaldorei Priestess

### W2 —— 黑暗赠礼（DK/DH/潜行者/术士/战士）
- [x] EDR_102 Treacherous Tormentor
- [x] EDR_456 Darkrider
- [x] EDR_487 Wallow, the Wretched
- [x] EDR_488 Avant-Gardening
- [x] EDR_528 Nightmare Fuel
- [x] EDR_654 Overgrown Horror
- [x] EDR_811 Rite of Atrocity
- [x] EDR_856 Nightmare Lord Xavius
- [x] EDR_882 Jumpscare!

### W3 —— Choose One（全职业）
- [x] EDR_233 Spirits of the Forest
- [x] EDR_257 Lightmender
- [x] EDR_263 Grace of the Greatwolf
- [x] EDR_273 Symbiosis
- [x] EDR_463 Twilight Influence
- [x] EDR_490 Sleep Paralysis
- [x] EDR_525 Barbed Thorn
- [x] EDR_570 Ominous Nightmares
- [x] EDR_813 Morbid Swarm
- [x] EDR_820 Wyvern's Slumber
- [x] EDR_843 Reforestation
- [x] EDR_872 Spark of Life

### W4 —— 野神 + 其余卡
- [ ] EDR_000 Ysera, Emerald Aspect
- [ ] EDR_031 Ohn'ahra
- [ ] EDR_209 Forest Lord Cenarius
- [ ] EDR_238 Merithra
- [ ] EDR_258 Toreth the Unbreaking
- [ ] EDR_259 Ursol
- [ ] EDR_421 Omen
- [ ] EDR_430 Aessina
- [ ] EDR_464 Tyrande
- [ ] EDR_465 Ysondre
- [ ] EDR_471 Tortolla
- [ ] EDR_480 Goldrinn
- [ ] EDR_489 Agamaggan
- [ ] EDR_493 Alara'shi
- [ ] EDR_517 Q'onzu
- [ ] EDR_526 Renferal, the Malignant
- [ ] EDR_527 Ashamane
- [ ] EDR_818 Nythendra
- [ ] EDR_819 Ursoc
- [ ] EDR_844 Naralex, Herald of the Flights
- [ ] EDR_846 Shaladrassil
- [ ] EDR_853 Broll Bearmantle
- [ ] EDR_895 Aviana, Elune's Chosen
> Wild Gods — one per class (elite)
- [ ] EDR_001 Hopeful Dryad
- [ ] EDR_014 Verdant Dreamsaber
- [ ] EDR_060 Ward of Earth
- [ ] EDR_105 Creature of Madness
- [ ] EDR_110 Sporegnasher
- [ ] EDR_230 Beanstalk Brute
- [ ] EDR_232 Typhoon
- [ ] EDR_234 Emerald Bounty
- [ ] EDR_251 Dragonscale Armaments
- [ ] EDR_252 Mark of Ursol
- [ ] EDR_253 Ursine Maul
- [ ] EDR_254 Animated Moonwell
- [ ] EDR_255 Renewing Flames
- [ ] EDR_256 Dreamwarden
- [ ] EDR_260 Illusory Greenwing
- [ ] EDR_261 Amphibian's Spirit
- [ ] EDR_262 Spirit Bond
- [ ] EDR_270 Horn of Plenty
- [ ] EDR_271 Grove Shaper
- [ ] EDR_272 Evergreen Stag
- [ ] EDR_416 Shepherd's Crook
- [ ] EDR_453 Briarspawn Drake
- [ ] EDR_454 Clutch of Corruption
- [ ] EDR_455 Succumb to Madness
- [ ] EDR_457 Brood Keeper
- [ ] EDR_459 Afflicted Devastator
- [ ] EDR_460 Wish of the New Moon
- [ ] EDR_461 Ritual of the New Moon
- [ ] EDR_462 Selenic Drake
- [ ] EDR_468 Eggbasher
- [ ] EDR_469 Slumbering Sprite
- [ ] EDR_470 Barkshield Sentinel
- [ ] EDR_472 Weaver of the Cycle
- [ ] EDR_476 Moonwell
- [ ] EDR_477 Glowroot Lure
- [ ] EDR_481 Mythical Runebear
- [ ] EDR_482 Rotten Apple
- [ ] EDR_483 Fractured Power
- [ ] EDR_484 Scavenging Flytrap
- [ ] EDR_485 Rotheart Dryad
- [ ] EDR_486 Scorching Observer
- [ ] EDR_491 Archdruid of Thorns
- [ ] EDR_492 Mother Duck
- [ ] EDR_494 Hungering Ancient
- [ ] EDR_495 Twisted Treant
- [ ] EDR_520 Forbidden Shrine
- [ ] EDR_521 Tricky Satyr
- [ ] EDR_522 Mimicry
- [ ] EDR_523 Web of Deception
- [ ] EDR_524 Shadowcloaked Assailant
- [ ] EDR_529 Plucky Podling
- [ ] EDR_530 Daydreaming Pixie
- [ ] EDR_531 Siphoning Growth
- [ ] EDR_540 Twisted Webweaver
- [ ] EDR_571 Fae Trickster
- [ ] EDR_572 Tormented Dreadwing
- [ ] EDR_598 Dream Rager
- [ ] EDR_780 Bloodthistle Illusionist
- [ ] EDR_781 Harbinger of the Blighted
- [ ] EDR_804 Divination
- [ ] EDR_810 Hideous Husk
- [ ] EDR_812 Grotesque Runeblade
- [ ] EDR_814 Infested Breath
- [ ] EDR_815 Corpse Flower
- [ ] EDR_816 Monstrous Mosquito
- [ ] EDR_817 Sanguine Infestation
- [ ] EDR_840 Grim Harvest
- [ ] EDR_841 Dreadsoul Corrupter
- [ ] EDR_842 Defiled Spear
- [ ] EDR_847 Dreambound Disciple
- [ ] EDR_848 Photosynthesis
- [ ] EDR_849 Dreambound Raptor
- [ ] EDR_861 Tranquil Treant
- [ ] EDR_873 Envoy of the Glade
- [ ] EDR_874 Stellar Balance
- [ ] EDR_889 Petal Peddler
- [ ] EDR_890 Nightmare Dragonkin
- [ ] EDR_891 Ravenous Felhunter
- [ ] EDR_892 Ferocious Felbat
- [ ] EDR_940 Merry Moonkin
- [ ] EDR_941 Starsurge
- [ ] EDR_942 Curious Cumulus
- [ ] EDR_971 Critter Caretaker
- [ ] EDR_978 Meadowstrider
- [ ] EDR_979 Ancient of Yore
- [ ] EDR_999 Gnawing Greenfin

### W5 —— 世界之树的余烬迷你
- [ ] FIR_777 Spirit of the Kaldorei
- [ ] FIR_778 Avatar of Destruction
- [ ] FIR_900 Cremate
- [ ] FIR_901 Frostburn Matriarch
- [ ] FIR_902 Sigil of Cinder
- [ ] FIR_904 Felfire Blaze
- [ ] FIR_906 Overheat
- [ ] FIR_907 Amirdrassil
- [ ] FIR_908 Charred Chameleon
- [ ] FIR_909 Bursting Shot
- [ ] FIR_910 Scorching Winds
- [ ] FIR_911 Smoldering Grove
- [ ] FIR_913 Inferno Herald
- [ ] FIR_914 Smoldering Strength
- [ ] FIR_916 Smoldering Ascent
- [ ] FIR_918 Light of the New Moon
- [ ] FIR_919 Everburning Phoenix
- [ ] FIR_920 Smoke Bomb
- [ ] FIR_921 Petal Picker
- [ ] FIR_922 Cindersword
- [ ] FIR_923 Flames of the Firelord
- [ ] FIR_924 Shadowflame Stalker
- [ ] FIR_927 Emberscarred Whelp
- [ ] FIR_928 Keeper of Flame
- [ ] FIR_929 Living Flame
- [ ] FIR_939 Shadowflame Suffusion
- [ ] FIR_940 Zaqali Flamemancer
- [ ] FIR_941 Searing Reflection
- [ ] FIR_951 Volcoross
- [ ] FIR_952 Scorchreaver
- [ ] FIR_953 Magma Hound
- [ ] FIR_954 Conflagrate
- [ ] FIR_955 Emberroot Destroyer
- [ ] FIR_956 Dragon Turtle
- [ ] FIR_958 Tindral Sageswift
- [ ] FIR_959 Fyrakk the Blazing
- [ ] FIR_960 Tending Dragonkin
- [ ] FIR_961 Ashleaf Pixie
> note: miniset cards also reuse Imbue/Dark Gift: FIR_900, FIR_901, FIR_920, FIR_921, FIR_922, FIR_924, FIR_939, FIR_956

## 完成定义

清单全部 `- [ ]` → `- [x]`；P1–P3、P5 带 F5 场景落地（P4 灼烧已按 M0.5
数据核实移除）；Choose One 简化已清偿（或按 D2 显式保留并带账本行）；
`cargo test` 全绿；子路线图随主文件移入 `docs/finished/`。
