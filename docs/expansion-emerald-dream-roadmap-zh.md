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
- [x] **W4 —— 野神 + 杂项效果（P5）**：W4a（PR #140——86 张非传说剩余卡 + 9 衍生物、
  63 个效果变体、3 个引擎 bug 修复经 F5 钉死、36 项 D2 简化登记 §14.3、14 个场景）
  + W4b（PR #141——23 张精英野神传说：三选抉择、保留/置顶抉择、每玩家攻击/双施/
  费用标志、Ursoc 击杀记录复活；23 个 F5 场景、13 项 D2 简化登记 §14.4、2 张
  衍生物；Ashamane 登记 `POOL_OPEN_CARDS`）。主系列 145 张全部完成。
- [x] **W5 —— 世界之树的余烬迷你**（PR #142）：38 张迷你卡 + 1 衍生物全部落地
  （既有原语：黑暗赠礼 7、发现 6、战吼 16、灌注条件、尸体；30 个新效果变体）；
  38 个 F5 场景覆盖每张卡；25 项 D2 简化登记 §14.5。**M1 全部完成（145 + 38）**。

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
- [x] EDR_000 Ysera, Emerald Aspect
- [x] EDR_031 Ohn'ahra
- [x] EDR_209 Forest Lord Cenarius
- [x] EDR_238 Merithra
- [x] EDR_258 Toreth the Unbreaking
- [x] EDR_259 Ursol
- [x] EDR_421 Omen
- [x] EDR_430 Aessina
- [x] EDR_464 Tyrande
- [x] EDR_465 Ysondre
- [x] EDR_471 Tortolla
- [x] EDR_480 Goldrinn
- [x] EDR_489 Agamaggan
- [x] EDR_493 Alara'shi
- [x] EDR_517 Q'onzu
- [x] EDR_526 Renferal, the Malignant
- [x] EDR_527 Ashamane
- [x] EDR_818 Nythendra
- [x] EDR_819 Ursoc
- [x] EDR_844 Naralex, Herald of the Flights
- [x] EDR_846 Shaladrassil
- [x] EDR_853 Broll Bearmantle
- [x] EDR_895 Aviana, Elune's Chosen
> Wild Gods — one per class (elite)
- [x] EDR_001 Hopeful Dryad
- [x] EDR_014 Verdant Dreamsaber
- [x] EDR_060 Ward of Earth
- [x] EDR_105 Creature of Madness
- [x] EDR_110 Sporegnasher
- [x] EDR_230 Beanstalk Brute
- [x] EDR_232 Typhoon
- [x] EDR_234 Emerald Bounty
- [x] EDR_251 Dragonscale Armaments
- [x] EDR_252 Mark of Ursol
- [x] EDR_253 Ursine Maul
- [x] EDR_254 Animated Moonwell
- [x] EDR_255 Renewing Flames
- [x] EDR_256 Dreamwarden
- [x] EDR_260 Illusory Greenwing
- [x] EDR_261 Amphibian's Spirit
- [x] EDR_262 Spirit Bond
- [x] EDR_270 Horn of Plenty
- [x] EDR_271 Grove Shaper
- [x] EDR_272 Evergreen Stag
- [x] EDR_416 Shepherd's Crook
- [x] EDR_453 Briarspawn Drake
- [x] EDR_454 Clutch of Corruption
- [x] EDR_455 Succumb to Madness
- [x] EDR_457 Brood Keeper
- [x] EDR_459 Afflicted Devastator
- [x] EDR_460 Wish of the New Moon
- [x] EDR_461 Ritual of the New Moon
- [x] EDR_462 Selenic Drake
- [x] EDR_468 Eggbasher
- [x] EDR_469 Slumbering Sprite
- [x] EDR_470 Barkshield Sentinel
- [x] EDR_472 Weaver of the Cycle
- [x] EDR_476 Moonwell
- [x] EDR_477 Glowroot Lure
- [x] EDR_481 Mythical Runebear
- [x] EDR_482 Rotten Apple
- [x] EDR_483 Fractured Power
- [x] EDR_484 Scavenging Flytrap
- [x] EDR_485 Rotheart Dryad
- [x] EDR_486 Scorching Observer
- [x] EDR_491 Archdruid of Thorns
- [x] EDR_492 Mother Duck
- [x] EDR_494 Hungering Ancient
- [x] EDR_495 Twisted Treant
- [x] EDR_520 Forbidden Shrine
- [x] EDR_521 Tricky Satyr
- [x] EDR_522 Mimicry
- [x] EDR_523 Web of Deception
- [x] EDR_524 Shadowcloaked Assailant
- [x] EDR_529 Plucky Podling
- [x] EDR_530 Daydreaming Pixie
- [x] EDR_531 Siphoning Growth
- [x] EDR_540 Twisted Webweaver
- [x] EDR_571 Fae Trickster
- [x] EDR_572 Tormented Dreadwing
- [x] EDR_598 Dream Rager
- [x] EDR_780 Bloodthistle Illusionist
- [x] EDR_781 Harbinger of the Blighted
- [x] EDR_804 Divination
- [x] EDR_810 Hideous Husk
- [x] EDR_812 Grotesque Runeblade
- [x] EDR_814 Infested Breath
- [x] EDR_815 Corpse Flower
- [x] EDR_816 Monstrous Mosquito
- [x] EDR_817 Sanguine Infestation
- [x] EDR_840 Grim Harvest
- [x] EDR_841 Dreadsoul Corrupter
- [x] EDR_842 Defiled Spear
- [x] EDR_847 Dreambound Disciple
- [x] EDR_848 Photosynthesis
- [x] EDR_849 Dreambound Raptor
- [x] EDR_861 Tranquil Treant
- [x] EDR_873 Envoy of the Glade
- [x] EDR_874 Stellar Balance
- [x] EDR_889 Petal Peddler
- [x] EDR_890 Nightmare Dragonkin
- [x] EDR_891 Ravenous Felhunter
- [x] EDR_892 Ferocious Felbat
- [x] EDR_940 Merry Moonkin
- [x] EDR_941 Starsurge
- [x] EDR_942 Curious Cumulus
- [x] EDR_971 Critter Caretaker
- [x] EDR_978 Meadowstrider
- [x] EDR_979 Ancient of Yore
- [x] EDR_999 Gnawing Greenfin

### W5 —— 世界之树的余烬迷你
- [x] FIR_777 Spirit of the Kaldorei
- [x] FIR_778 Avatar of Destruction
- [x] FIR_900 Cremate
- [x] FIR_901 Frostburn Matriarch
- [x] FIR_902 Sigil of Cinder
- [x] FIR_904 Felfire Blaze
- [x] FIR_906 Overheat
- [x] FIR_907 Amirdrassil
- [x] FIR_908 Charred Chameleon
- [x] FIR_909 Bursting Shot
- [x] FIR_910 Scorching Winds
- [x] FIR_911 Smoldering Grove
- [x] FIR_913 Inferno Herald
- [x] FIR_914 Smoldering Strength
- [x] FIR_916 Smoldering Ascent
- [x] FIR_918 Light of the New Moon
- [x] FIR_919 Everburning Phoenix
- [x] FIR_920 Smoke Bomb
- [x] FIR_921 Petal Picker
- [x] FIR_922 Cindersword
- [x] FIR_923 Flames of the Firelord
- [x] FIR_924 Shadowflame Stalker
- [x] FIR_927 Emberscarred Whelp
- [x] FIR_928 Keeper of Flame
- [x] FIR_929 Living Flame
- [x] FIR_939 Shadowflame Suffusion
- [x] FIR_940 Zaqali Flamemancer
- [x] FIR_941 Searing Reflection
- [x] FIR_951 Volcoross
- [x] FIR_952 Scorchreaver
- [x] FIR_953 Magma Hound
- [x] FIR_954 Conflagrate
- [x] FIR_955 Emberroot Destroyer
- [x] FIR_956 Dragon Turtle
- [x] FIR_958 Tindral Sageswift
- [x] FIR_959 Fyrakk the Blazing
- [x] FIR_960 Tending Dragonkin
- [x] FIR_961 Ashleaf Pixie
> note: miniset cards also reuse Imbue/Dark Gift: FIR_900, FIR_901, FIR_920, FIR_921, FIR_922, FIR_924, FIR_939, FIR_956

## 完成定义

清单全部 `- [ ]` → `- [x]`；P1–P3、P5 带 F5 场景落地（P4 灼烧已按 M0.5
数据核实移除）；Choose One 简化已清偿（或按 D2 显式保留并带账本行）；
`cargo test` 全绿；子路线图随主文件移入 `docs/finished/`。
