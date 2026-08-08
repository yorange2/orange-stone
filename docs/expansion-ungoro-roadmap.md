# Expansion Roadmap — The Lost City of Un'Goro (失落之城安戈洛)

> Status: **active** (created 2026-08-08, part of the 2025–2026 expansions master
> roadmap). Chinese mirror: `expansion-ungoro-roadmap-zh.md`.
> Scope: 145 cards (2025-07-08, Patch 33.0) + the **Festival of the Devilsaur**
> miniset (38 cards). Prerequisite: M0 data pipeline + Emerald Dream sub-roadmap
> (its P1/P2 primitives are reused here).

## The set

Theme: Elise Starseeker's return to Un'Goro, hunting a lost Tortollan city.
Verified mechanics (2026-08-08, 33.0 patch notes):

- **Quests return** — quest cards played into a quest slot; progress tracked
  per quest; reward on completion. The engine has **no quest zone at all** —
  this is a new zone/component, the biggest primitive of the year.
- **Kindred** — a per-card trigger keyword (example from patch notes:
  "Kindred: It costs (1) less"; exact trigger set per card text, backfilled
  from data).
- Headline context: 145-card set; the quest mechanic was last in the game
  pre-2020 (Un'Goro 2017 original), so no engine precedent exists.

## Engine primitives (this roadmap's waves)

| # | Primitive | Used by | Notes |
|---|---|---|---|
| Q1 | Quest zone + progress + reward | all quest cards | `Zone::Quest` (or per-player quest slot), progress events per quest condition, reward effect on completion; quest-slot conflicts (one quest per player) and counter-exposure in observations |
| Q2 | Kindred counter | Kindred cards | Per-card trigger counter (exact condition from text — costs, buffs, or death-tracking); interacts with `cards_played_this_turn`-style per-player counters |
| Q3 | misc new CardEffects | remaining cards | Backfilled per wave |

## Wave plan

One PR per wave; every card lands with an F5 differential scenario; `sets.rs`
registration; simplifications get ledger rows.

- [x] **W0 — wiring + data** (PR #143): M0 data for this set landed
  (M0.1–M0.5); inventory backfilled; the per-set dump fidelity test
  (`the_lost_city_dump_fidelity`, TLC_ + DINO_ prefixes, 183 cards) landed
  with the generated baseline.
- [x] **W1 — Quest zone (Q1)** (PR #144): `Zone::Quest` per-player quest slot
  + `Quest` component + `cards::quest` registry (SpellSchool, QuestCondition,
  `quest_def` for all 11 quests) + `engine::quest` progress dispatch with
  marker dedup and reward resolution (repeatable quests reset and stay);
  play path diverts quest cards to the slot (one quest per player — new quest
  destroys the old); call sites for minion plays/summons/damage/turn-end/
  spell-cast/corpse-spend/discover; 7 engine-level `tlc_w1_*` smoke scenarios
  landed before any quest card.
- **W2 — Quest cards:** the set's quest cards on top of W1 (one per class +
  neutral where applicable); F5 scenarios pin progress→reward sequencing and
  the one-quest-per-player rule.
- **W3 — Kindred (Q2):** counter primitive + Kindred cards; F5 scenarios for
  the trigger conditions from text.
- **W4 — Festival of the Devilsaur mini:** miniset cards (mechanics backfilled
  from data); closing wave sweeps tokens/ledger.

## Card inventory

> Backfilled from the M0 data dump (D1). Placeholder grouped by wave.

### W1/W2 — Quests
- [ ] TLC_229 Spirit of the Mountain
- [ ] TLC_239 Restore the Wild
- [ ] TLC_426 Dive the Golakka Depths
- [ ] TLC_433 Reanimate the Terror
- [ ] TLC_446 Escape the Underfel
- [ ] TLC_460 The Forbidden Sequence
- [ ] TLC_513 Lie in Wait
- [ ] TLC_602 Enter the Lost City
- [ ] TLC_631 Unleash the Colossus
- [ ] TLC_817 Reach Equilibrium
- [ ] TLC_830 The Food Chain

### W3 — Kindred
- [ ] TLC_102 Torga
- [ ] TLC_107 Stormbrewer
- [ ] TLC_223 Volcanic Thrasher
- [ ] TLC_226 Conjured Bookkeeper
- [ ] TLC_236 Hybridization
- [ ] TLC_243 Whirling Stormdrake
- [ ] TLC_251 Primalfin Challenger
- [ ] TLC_366 Pterrorwing Ravager
- [ ] TLC_428 Hot Spring Glider
- [ ] TLC_429 Steamfin Thief
- [ ] TLC_432 Dread Raptor
- [ ] TLC_440 Cryosleep
- [ ] TLC_447 Caustic Fumes
- [ ] TLC_454 Scalehide Kodo
- [ ] TLC_463 Razidir
- [ ] TLC_482 Slagclaw
- [ ] TLC_519 Ambush Predators
- [ ] TLC_600 Windpeak Wyrm
- [ ] TLC_815 Gravedawn Voidbulb
- [ ] TLC_816 Gravedawn Sunbloom
- [ ] TLC_825 Ravasaur Matriarch
- [ ] TLC_829 Ravenous Devilsaur
- [ ] TLC_903 Silithid Queen

### W4 — remaining cards + mini
- [ ] TLC_100 Elise the Navigator
- [ ] TLC_101 Undercover Cultist
- [ ] TLC_106 Endbringer Umbra
- [ ] TLC_109 Relic Miner
- [ ] TLC_110 City Chief Esho
- [ ] TLC_220 Windswept Pageturner
- [ ] TLC_221 Sizzling Swarm
- [ ] TLC_222 Flight of the Firehawk
- [ ] TLC_224 Mechanized Magma
- [ ] TLC_225 Cinderfin
- [ ] TLC_227 Lava Flow
- [ ] TLC_228 Bralma Searstone
- [ ] TLC_230 TREEEES!!!
- [ ] TLC_231 Story of Barnabus
- [ ] TLC_232 Ravenous Flock
- [ ] TLC_233 Hatchery Helper
- [ ] TLC_234 Eternal Bloodpetal
- [ ] TLC_235 Life Cycle
- [ ] TLC_237 Skyscreamer Eggs
- [ ] TLC_240 Tyrannogill
- [ ] TLC_241 Ido of the Threshfleet
- [ ] TLC_242 Ancient Stegodon
- [ ] TLC_244 Curious Explorer
- [ ] TLC_245 Ancient Raptor
- [ ] TLC_246 Ancient Pterrordax
- [ ] TLC_247 Primal Sabretooth
- [ ] TLC_248 Ultragigasaur
- [ ] TLC_249 Sizzling Cinder
- [ ] TLC_250 Crater Gator
- [ ] TLC_252 Dissolving Ooze
- [ ] TLC_253 Petrified Ogre
- [ ] TLC_254 Tortollan Storyteller
- [ ] TLC_255 Crystal Tender
- [ ] TLC_256 Marshland Thresher
- [ ] TLC_257 Loh, the Living Legend
- [ ] TLC_334 Relic of Kings
- [ ] TLC_364 Story of the Waygate
- [ ] TLC_365 Storage Scuffle
- [ ] TLC_401 Bonechill Stegodon
- [ ] TLC_427 Rockskipper
- [ ] TLC_430 Creature of the Sacred Cave
- [ ] TLC_434 Paleomancy
- [ ] TLC_435 Crypt Map
- [ ] TLC_436 Reanimated Pterrordax
- [ ] TLC_438 Violet Treasuregill
- [ ] TLC_439 Wave of Tar
- [ ] TLC_441 Ready the Fleet
- [ ] TLC_442 Submerged Map
- [ ] TLC_443 Reluctant Wrangler
- [ ] TLC_444 Story of Galvadon
- [ ] TLC_449 Bloodpetal Biome
- [ ] TLC_450 Spelunker
- [ ] TLC_451 Cursed Catacombs
- [ ] TLC_452 Titanographer Osk
- [ ] TLC_461 Scrappy Scavenger
- [ ] TLC_462 Unearthed Artifacts
- [ ] TLC_464 Mountain Map
- [ ] TLC_465 Stranglevine
- [ ] TLC_466 Story of Lakkari
- [ ] TLC_467 Whispering Stone
- [ ] TLC_468 Blob of Tar
- [ ] TLC_469 Tunnel Terror
- [ ] TLC_477 Threshrider's Blessing
- [ ] TLC_478 Axe of the Forefathers
- [ ] TLC_479 Deathrot Maw
- [ ] TLC_480 Krog, Crater King
- [ ] TLC_483 Vault Breaker
- [ ] TLC_514 Merchant of Legend
- [ ] TLC_515 Cultist Map
- [ ] TLC_516 Neferset Weaponsmith
- [ ] TLC_517 Knockback
- [ ] TLC_518 Interrogation
- [ ] TLC_520 Underbrush Tracker
- [ ] TLC_521 Eyes in the Sky
- [ ] TLC_522 Opu the Unseen
- [ ] TLC_601 Shellnado
- [ ] TLC_603 Platysaur
- [ ] TLC_605 Tar Tyrant
- [ ] TLC_606 Latorvian Armorer
- [ ] TLC_620 Fortify
- [ ] TLC_621 Willful Watcher
- [ ] TLC_622 City Defenses
- [ ] TLC_623 Stonecarver
- [ ] TLC_624 Nablya, the Watcher
- [ ] TLC_630 Gorishi Wasp
- [ ] TLC_632 Story of Sulfuras
- [ ] TLC_633 Bugsquasher
- [ ] TLC_810 High Cultist Herenn
- [ ] TLC_811 Archaios
- [ ] TLC_814 Twilight Mender
- [ ] TLC_818 Resuscitate
- [ ] TLC_819 Gladesong Siren
- [ ] TLC_820 Glade Ecologist
- [ ] TLC_821 Wilted Shadow
- [ ] TLC_822 Dinositter
- [ ] TLC_823 Cower in Fear
- [ ] TLC_824 Odd Map
- [ ] TLC_826 Story of Carnassa
- [ ] TLC_827 Grazing Stegodon
- [ ] TLC_828 Supreme Dinomancy
- [ ] TLC_831 Pterrordax Egg
- [ ] TLC_833 Insect Claw
- [ ] TLC_835 Story of Amara
- [ ] TLC_836 Niri of the Crater
- [ ] TLC_840 Gorishi Tunneler
- [ ] TLC_841 Entomologist Toru
- [ ] TLC_888 Cloud Serpent
- [ ] TLC_900 Hive Map
- [ ] TLC_901 Fumigate
- [ ] TLC_902 Infestation
- [ ] TLC_987 Questing Assistant
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

## Definition of done

All inventory `- [ ]` → `- [x]`; Q1–Q3 landed with F5 scenarios; the quest
zone is exposed in observations/`legal_actions` (decision D3 of the master
roadmap applies — pool policy decides how much surfaces to RL); `cargo test`
green; sub-roadmap archived with the master file.
