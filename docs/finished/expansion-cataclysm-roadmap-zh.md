# 扩展子路线图 —— 大灾变 Cataclysm

> 状态：**活跃**（2026-08-08 创建，2025–2026 扩展主路线图的一部分）。英文镜像：
> `expansion-cataclysm-roadmap.md`。
> 范围：约 145 张卡（2026-03-17，补丁 35.0，圣甲虫年首发系列）。无迷你——暴雪
> 改发"职业套装"（数据可得时另作跟进波）。前置：M0 数据管线 + 三个 2025
> 子路线图（在此扩展翡翠梦境 P1 的英雄技能机制）。

## 系列概况

主题：死亡之翼获胜的平行时间线。机制核实（2026-08-08，官方新闻 + 补丁说明）：

- **死亡之翼英雄牌** —— "死亡之翼·灭世者"：首张死亡之翼英雄牌；每回合玩家
  选择一个灾变效果，随新关键字缩放。
- **巨像回归** —— 每职业一张传说巨像；巨像 +N 随从连带召唤附属部件（引擎无
  附属部件概念）。
- **先锋 Herald** —— 死翼阵营（死亡骑士 / 恶魔猎手 / 潜行者 / 萨满 / 术士 /
  战士）：召唤随数量缩放的巨像士兵。
- **粉碎 Shatter** —— 龙眠阵营（法师 / 牧师 / 德鲁伊 / 猎人 / 圣骑士）：
  抽到的卡一分为二，可重组为双倍强化的版本。
- 伴随系列的还有新一轮核心系列轮换（引擎侧：无操作——两池共存；切换标准环境
  时在 CLAUDE.md 注明）。

## 引擎原语（本路线图的波）

| # | 原语 | 使用方 | 说明 |
|---|---|---|---|
| C1 | 巨像附属部件 | 巨像随从 | 随从登场连带召唤附属部件、毗邻占位、随本体死亡（部件死亡不杀本体）；新召唤路径 + 战场占位规则 |
| C2 | 先锋缩放 | 先锋卡（6 职业） | 每玩家先锋计数；召唤随计数缩放的巨像士兵 |
| C3 | 粉碎分裂/重组 | 粉碎卡（5 职业） | 抽到的卡变为两半（分别抽取），两半同在时重组——触及抽牌、手牌与打出；年内最大的时序风险 |
| C4 | 死亡之翼英雄选择 | 死亡之翼英雄牌 | 扩展翡翠梦境 P1 英雄技能机制：每回合选择一种灾变效果的结算 |
| C5 | 各类新 CardEffect | 其余卡 | 各波回填 |

## 波计划

每波一个 PR；每张卡带 F5 对拍场景；`sets.rs` 注册；简化带账本行。

- [x] **W0 —— 接线 + 数据**（PR #155）：本系列 M0 数据（M0.1–M0.5，含 M0.3 已合并的职业套装 MEND_ 29 张，按路线图延后为跟进波）；清单回填；per-set dump 保真测试（`cataclysm_dump_fidelity`，CATA_ + MEND_ 前缀，164 张）与生成基线落地。
- [x] **W1 —— 巨像（C1）**（PR #156）： 附属部件组件 + 召唤路径；F5 场景钉死部件占位、部件
  死亡规则与本体死亡级联。
- [x] **W2 —— 先锋（C2）**（PR #157）： 缩放计数 + 巨像士兵池；6 职业先锋卡。
- [x] **W3 —— 粉碎（C3）**（PR #158）： 分裂/重组管线；F5 场景钉死 抽牌-分裂-重组 时序；若该波
  判定完整保真过大，按 D2 登记简化而非上残缺机制。
- [x] **W4 —— 死亡之翼 + 收尾**（PR #159）： C4 基于 P1 机制；其余卡；职业套装跟进推迟；
  收尾波清扫衍生物/账本。

## 卡清单

> 由 M0 数据转储回填（D1）。按波分组占位。

### W1 —— 巨像
- [x] CATA_139 Wickerfang
- [x] CATA_150 Ragnaros, the Great Fire
- [x] CATA_151 Azshara, Ocean Lord
- [x] CATA_153 Al'Akir, Lord of Storms
- [x] CATA_154 Sinestra
- [x] CATA_155 Arisen Onyxia
- [x] CATA_300 The Black Blood
- [x] CATA_432 Chromatus
- [x] CATA_488 Vulcanos
- [x] CATA_550 Magmaw
- [x] CATA_726 Cho'gall, Mastermind

### W2 —— 先锋
- [x] CATA_156 Experimental Animation
- [x] CATA_158 Maniacal Follower
- [x] CATA_160 Scorching Ravager
- [x] CATA_492 Shrine of Twilight
- [x] CATA_525 Armored Bloodletter
- [x] CATA_530 Fel Infusion
- [x] CATA_561 Ritual of Power
- [x] CATA_565 Skywall Sentinel
- [x] CATA_580 Cataclysmic War Axe
- [x] CATA_722 Envoy of the End
- [x] CATA_725 Shadowsworn Disciple
- [x] CATA_780 Obsessive Technician
- [x] CATA_785 Rite of Twilight

### W3 —— 粉碎
- [x] CATA_134 Wildwood Circle
- [x] CATA_202 Stolen Power
- [x] CATA_306 Schism
- [x] CATA_479 Flight Maneuvers
- [x] CATA_489 Arcane Flow
- [x] CATA_820 Supply Run

### W4 —— 死亡之翼 + 其余卡
- [x] CATA_190h Deathwing, Worldbreaker
- [x] CATA_497 Ultraxion
- [x] CATA_111 Darkscale Broodmother
- [x] CATA_130 Crystalspine Cub
- [x] CATA_131 Felwood Treant
- [x] CATA_132 Broodwatcher
- [x] CATA_133 Iridescent Flitterwing
- [x] CATA_135 Mossbinding
- [x] CATA_136 Azshara's Triumph
- [x] CATA_138 Forest's Gift
- [x] CATA_140 Merithra of the Dream
- [x] CATA_161 Gruesome Nightmare
- [x] CATA_180 War'loc
- [x] CATA_185 Faceless Replicator
- [x] CATA_186 Stickybomb Saboteur
- [x] CATA_200 Agent of the Old Ones
- [x] CATA_201 Twilight Mistress
- [x] CATA_203 Garona's Last Stand
- [x] CATA_206 Twisted Monstrosity
- [x] CATA_208 Selfless Protector
- [x] CATA_209 Battlefield Blaster
- [x] CATA_210 Twilight Egg
- [x] CATA_213 Vyranoth
- [x] CATA_215 Daze
- [x] CATA_216 Cleansing Cleric
- [x] CATA_301 Ruby Sanctum
- [x] CATA_302 Mend
- [x] CATA_303 Purifying Breath
- [x] CATA_304 Injured Attendant
- [x] CATA_305 Incensed Matriarch
- [x] CATA_307 Alexstrasza, Guardian of Life
- [x] CATA_308 Medivh's Triumph
- [x] CATA_452 Spellweaver's Brilliance
- [x] CATA_458 Archmage Kalec
- [x] CATA_464 Blackwing Experiment
- [x] CATA_465 Chow Down
- [x] CATA_467 Command Claw
- [x] CATA_469 Chromatic Broodmother
- [x] CATA_470 Victor Nefarius
- [x] CATA_471 Talanji's Last Stand
- [x] CATA_472 Inspiring Maul
- [x] CATA_473 Nozdormu, Bronze Aspect
- [x] CATA_474 Spearheart Sentry
- [x] CATA_475 Scalebreaker Bulwark
- [x] CATA_476 Bronze Keeper
- [x] CATA_477 Chamber of Aspects
- [x] CATA_478 Bronze Redeemer
- [x] CATA_480 Sandfury Aura
- [x] CATA_481 Iso'rath
- [x] CATA_483 Unstable Spellcaster
- [x] CATA_484 Winterspring Whelp
- [x] CATA_485 Sleet Storm
- [x] CATA_487 Raincaller
- [x] CATA_490 Ocular Occultist
- [x] CATA_491 Eldritch Tentacles
- [x] CATA_493 Duke of Below
- [x] CATA_494 Maloriak
- [x] CATA_496 Cursed Chains
- [x] CATA_498 Rafaams' Last Stand
- [x] CATA_499 Disposable Acolytes
- [x] CATA_526 Broxigar's Last Stand
- [x] CATA_527 Nespirah, Enthralled
- [x] CATA_528 Sigil of the Seas
- [x] CATA_529 Ravenous Felfisher
- [x] CATA_533 Flash Flood
- [x] CATA_551 Stonetalon Striker
- [x] CATA_552 Ebonscale Scout
- [x] CATA_553 Ebyssian
- [x] CATA_554 Earthen Roar
- [x] CATA_556 Carrier Whelp
- [x] CATA_557 Sylvanas's Triumph
- [x] CATA_558 Reinforcement Rallier
- [x] CATA_560 Confront the Tol'vir
- [x] CATA_563 Crackling Cloudstrider
- [x] CATA_564 Air Support
- [x] CATA_566 Tol'vir Carver
- [x] CATA_567 Ascendance
- [x] CATA_568 Muradin's Last Stand
- [x] CATA_569 Ceremonial Clash
- [x] CATA_570 Morchok
- [x] CATA_581 Decimation
- [x] CATA_582 Searing Fissure
- [x] CATA_584 Erupting Volcano
- [x] CATA_585 Torch
- [x] CATA_586 Destructive Blaze
- [x] CATA_591 Commander Geddon
- [x] CATA_610 Lo'Gosh's Last Stand
- [x] CATA_612 Frostbitten Imp
- [x] CATA_613 Survivalist
- [x] CATA_614 Shadowed Informant
- [x] CATA_615 Genn, Cursed King
- [x] CATA_616 Gronn Giant
- [x] CATA_621 Gelbin's Triumph
- [x] CATA_697 Malevolent Mutant
- [x] CATA_699 Dread Leviathan
- [x] CATA_720 Warmaster Blackhorn
- [x] CATA_721 Sheltered Survivor
- [x] CATA_723 Drakeadon Mongrel
- [x] CATA_724 Stormbinder
- [x] CATA_786 Chaos Supplicant
- [x] CATA_897 Gemstone Hoarder
- [x] CATA_898 Scaled Lancer
- [x] CATA_978 Sindragosa's Triumph
- [x] CATA_979 Conjuration Specialist
- [x] CATA_999 Earthen Drake
> Class Sets（跟进波，29 张——迷你包位）：**W4 之后仍然推迟** —— MEND_ 职业
> 套装卡不属于 W4 范围（跟进波随 2025–2026 主路线图进入迷你包位时落地；W0
> 数据与 `cataclysm_dump_fidelity` 基线已覆盖它们）。
- [ ] MEND_040 Ash Worm
- [ ] MEND_041 Wizened Wildspeaker
- [ ] MEND_042 Lifebloom
- [ ] MEND_043 Heartroot Stones
- [ ] MEND_044 Tranquil Clearing
- [ ] MEND_045 Seeding Dragon
- [ ] MEND_046 Bashana Runetotem
- [ ] MEND_100 Cultivating Sprite
- [ ] MEND_300 Tame Pet
- [ ] MEND_301 Spiritspeaker
- [ ] MEND_302 Wasteland Vanguard
- [ ] MEND_303 Migrating Elekk
- [ ] MEND_304 Talya Earthstrider
- [ ] MEND_305 Nurturing Nature
- [ ] MEND_307 Roam Free
- [ ] MEND_500 Bursting Leyline
- [ ] MEND_501 Ley Walker
- [ ] MEND_502 Crystallized Leyline
- [ ] MEND_503 Surge Needle
- [ ] MEND_504 Leyline Nexus
- [ ] MEND_505 The Arcanomicon
- [ ] MEND_506 Mystic Runesaber
- [ ] MEND_800 Brash Battlemaster
- [ ] MEND_801 Resilient Savior
- [ ] MEND_802 Convalescence
- [ ] MEND_803 Emboldening Blade
- [ ] MEND_804 Arator the Redeemer
- [ ] MEND_805 Charity
- [ ] MEND_900 Teamwork

## 完成定义

清单全部 `- [ ]` → `- [x]`；C1–C5 带 F5 场景落地（粉碎时序显式钉死或按 D2
登记）；`cargo test` 全绿；子路线图随主文件归档。
