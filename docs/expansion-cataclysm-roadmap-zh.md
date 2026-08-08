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

- **W0 —— 接线 + 数据：** 本系列 M0 数据；清单回填；保真测试。
- **W1 —— 巨像（C1）：** 附属部件组件 + 召唤路径；F5 场景钉死部件占位、部件
  死亡规则与本体死亡级联。
- **W2 —— 先锋（C2）：** 缩放计数 + 巨像士兵池；6 职业先锋卡。
- **W3 —— 粉碎（C3）：** 分裂/重组管线；F5 场景钉死 抽牌-分裂-重组 时序；若该波
  判定完整保真过大，按 D2 登记简化而非上残缺机制。
- **W4 —— 死亡之翼 + 收尾：** C4 基于 P1 机制；其余卡；职业套装跟进推迟；
  收尾波清扫衍生物/账本。

## 卡清单

> 由 M0 数据转储回填（D1）。按波分组占位。

### W1 —— 巨像
- [ ] CATA_139 Wickerfang
- [ ] CATA_150 Ragnaros, the Great Fire
- [ ] CATA_151 Azshara, Ocean Lord
- [ ] CATA_153 Al'Akir, Lord of Storms
- [ ] CATA_154 Sinestra
- [ ] CATA_155 Arisen Onyxia
- [ ] CATA_300 The Black Blood
- [ ] CATA_432 Chromatus
- [ ] CATA_488 Vulcanos
- [ ] CATA_550 Magmaw
- [ ] CATA_726 Cho'gall, Mastermind

### W2 —— 先锋
- [ ] CATA_156 Experimental Animation
- [ ] CATA_158 Maniacal Follower
- [ ] CATA_160 Scorching Ravager
- [ ] CATA_492 Shrine of Twilight
- [ ] CATA_525 Armored Bloodletter
- [ ] CATA_530 Fel Infusion
- [ ] CATA_561 Ritual of Power
- [ ] CATA_565 Skywall Sentinel
- [ ] CATA_580 Cataclysmic War Axe
- [ ] CATA_722 Envoy of the End
- [ ] CATA_725 Shadowsworn Disciple
- [ ] CATA_780 Obsessive Technician
- [ ] CATA_785 Rite of Twilight

### W3 —— 粉碎
- [ ] CATA_134 Wildwood Circle
- [ ] CATA_202 Stolen Power
- [ ] CATA_306 Schism
- [ ] CATA_479 Flight Maneuvers
- [ ] CATA_489 Arcane Flow
- [ ] CATA_820 Supply Run

### W4 —— 死亡之翼 + 其余卡
- [ ] CATA_190h Deathwing, Worldbreaker
- [ ] CATA_497 Ultraxion
- [ ] CATA_111 Darkscale Broodmother
- [ ] CATA_130 Crystalspine Cub
- [ ] CATA_131 Felwood Treant
- [ ] CATA_132 Broodwatcher
- [ ] CATA_133 Iridescent Flitterwing
- [ ] CATA_135 Mossbinding
- [ ] CATA_136 Azshara's Triumph
- [ ] CATA_138 Forest's Gift
- [ ] CATA_140 Merithra of the Dream
- [ ] CATA_161 Gruesome Nightmare
- [ ] CATA_180 War'loc
- [ ] CATA_185 Faceless Replicator
- [ ] CATA_186 Stickybomb Saboteur
- [ ] CATA_200 Agent of the Old Ones
- [ ] CATA_201 Twilight Mistress
- [ ] CATA_203 Garona's Last Stand
- [ ] CATA_206 Twisted Monstrosity
- [ ] CATA_208 Selfless Protector
- [ ] CATA_209 Battlefield Blaster
- [ ] CATA_210 Twilight Egg
- [ ] CATA_213 Vyranoth
- [ ] CATA_215 Daze
- [ ] CATA_216 Cleansing Cleric
- [ ] CATA_301 Ruby Sanctum
- [ ] CATA_302 Mend
- [ ] CATA_303 Purifying Breath
- [ ] CATA_304 Injured Attendant
- [ ] CATA_305 Incensed Matriarch
- [ ] CATA_307 Alexstrasza, Guardian of Life
- [ ] CATA_308 Medivh's Triumph
- [ ] CATA_452 Spellweaver's Brilliance
- [ ] CATA_458 Archmage Kalec
- [ ] CATA_464 Blackwing Experiment
- [ ] CATA_465 Chow Down
- [ ] CATA_467 Command Claw
- [ ] CATA_469 Chromatic Broodmother
- [ ] CATA_470 Victor Nefarius
- [ ] CATA_471 Talanji's Last Stand
- [ ] CATA_472 Inspiring Maul
- [ ] CATA_473 Nozdormu, Bronze Aspect
- [ ] CATA_474 Spearheart Sentry
- [ ] CATA_475 Scalebreaker Bulwark
- [ ] CATA_476 Bronze Keeper
- [ ] CATA_477 Chamber of Aspects
- [ ] CATA_478 Bronze Redeemer
- [ ] CATA_480 Sandfury Aura
- [ ] CATA_481 Iso'rath
- [ ] CATA_483 Unstable Spellcaster
- [ ] CATA_484 Winterspring Whelp
- [ ] CATA_485 Sleet Storm
- [ ] CATA_487 Raincaller
- [ ] CATA_490 Ocular Occultist
- [ ] CATA_491 Eldritch Tentacles
- [ ] CATA_493 Duke of Below
- [ ] CATA_494 Maloriak
- [ ] CATA_496 Cursed Chains
- [ ] CATA_498 Rafaams' Last Stand
- [ ] CATA_499 Disposable Acolytes
- [ ] CATA_526 Broxigar's Last Stand
- [ ] CATA_527 Nespirah, Enthralled
- [ ] CATA_528 Sigil of the Seas
- [ ] CATA_529 Ravenous Felfisher
- [ ] CATA_533 Flash Flood
- [ ] CATA_551 Stonetalon Striker
- [ ] CATA_552 Ebonscale Scout
- [ ] CATA_553 Ebyssian
- [ ] CATA_554 Earthen Roar
- [ ] CATA_556 Carrier Whelp
- [ ] CATA_557 Sylvanas's Triumph
- [ ] CATA_558 Reinforcement Rallier
- [ ] CATA_560 Confront the Tol'vir
- [ ] CATA_563 Crackling Cloudstrider
- [ ] CATA_564 Air Support
- [ ] CATA_566 Tol'vir Carver
- [ ] CATA_567 Ascendance
- [ ] CATA_568 Muradin's Last Stand
- [ ] CATA_569 Ceremonial Clash
- [ ] CATA_570 Morchok
- [ ] CATA_581 Decimation
- [ ] CATA_582 Searing Fissure
- [ ] CATA_584 Erupting Volcano
- [ ] CATA_585 Torch
- [ ] CATA_586 Destructive Blaze
- [ ] CATA_591 Commander Geddon
- [ ] CATA_610 Lo'Gosh's Last Stand
- [ ] CATA_612 Frostbitten Imp
- [ ] CATA_613 Survivalist
- [ ] CATA_614 Shadowed Informant
- [ ] CATA_615 Genn, Cursed King
- [ ] CATA_616 Gronn Giant
- [ ] CATA_621 Gelbin's Triumph
- [ ] CATA_697 Malevolent Mutant
- [ ] CATA_699 Dread Leviathan
- [ ] CATA_720 Warmaster Blackhorn
- [ ] CATA_721 Sheltered Survivor
- [ ] CATA_723 Drakeadon Mongrel
- [ ] CATA_724 Stormbinder
- [ ] CATA_786 Chaos Supplicant
- [ ] CATA_897 Gemstone Hoarder
- [ ] CATA_898 Scaled Lancer
- [ ] CATA_978 Sindragosa's Triumph
- [ ] CATA_979 Conjuration Specialist
- [ ] CATA_999 Earthen Drake
> Class Sets (follow-up wave, 29 cards — the miniset slot):
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
