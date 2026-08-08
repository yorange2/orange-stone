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
- [x] **W2 — Quest cards** (PR #145): all 11 quest cards (one per class) +
  12 reward tokens in `exp_tlc_w2.rs`; TLC_817 dual quest bar
  (`QuestDef::second` — the card leaves the slot only when both bars
  complete), TLC_426 repeatable with the real permanent murloc +1/+1 flag,
  TLC_631 permanent exact-2-damage bonus flag, Everbloom weapon trigger,
  both Sol'etos halves; the Temporary primitive (marker + end-of-turn
  discard) enabling TLC_446; simplifications registered in §15 (en+zh);
  12 `tlc_w2_*` F5 scenarios pin progress→reward sequencing and the
  one-quest-per-player rule with real cards; the W1 spell-school scenario
  updated to the real dual-bar semantics.
- [x] **W3 — Kindred (Q2)** (PR #146): the mechanic verified from the card
  texts — "Kindred: X" activates when a card of the same type was played
  earlier this turn (tribe for minions, card type for spells; self never
  counts). `cards::kindred` registry (KindredType/KindredEffect tables,
  23 cards) + player-level `kindred_played` state (cleared at turn end) +
  `next_kindred_twice` flag; resolution shapes OnPlay / CostDiscount /
  BattlecryModifier + dedicated draw battlecries (TLC_102 Torga scan,
  TLC_223/236/432 drawn-card modifiers); 23 cards + 3 tokens in
  `exp_tlc_w3.rs`; simplifications §16 (en+zh); 22 `tlc_w3_*` F5
  scenarios; fixed the pre-existing `resolve_destroy_minion` AnyMinion
  gap.
- **W4 — Festival of the Devilsaur mini:** miniset cards (mechanics backfilled
  from data); closing wave sweeps tokens/ledger.

## Card inventory

> Backfilled from the M0 data dump (D1). Placeholder grouped by wave.

### W1/W2 — Quests
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

### W3 — Kindred
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

### W4 — remaining cards + mini
- [x] TLC_100 Elise the Navigator
- [x] TLC_101 Undercover Cultist
- [x] TLC_106 Endbringer Umbra
- [x] TLC_109 Relic Miner
- [x] TLC_110 City Chief Esho
- [x] TLC_220 Windswept Pageturner
- [x] TLC_221 Sizzling Swarm
- [x] TLC_222 Flight of the Firehawk
- [x] TLC_224 Mechanized Magma
- [x] TLC_225 Cinderfin
- [x] TLC_227 Lava Flow
- [x] TLC_228 Bralma Searstone
- [x] TLC_230 TREEEES!!!
- [x] TLC_231 Story of Barnabus
- [x] TLC_232 Ravenous Flock
- [x] TLC_233 Hatchery Helper
- [x] TLC_234 Eternal Bloodpetal
- [x] TLC_235 Life Cycle
- [x] TLC_237 Skyscreamer Eggs
- [x] TLC_240 Tyrannogill
- [x] TLC_241 Ido of the Threshfleet
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
- [x] TLC_257 Loh, the Living Legend
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
- [x] TLC_452 Titanographer Osk
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
- [x] TLC_480 Krog, Crater King
- [x] TLC_483 Vault Breaker
- [x] TLC_514 Merchant of Legend
- [x] TLC_515 Cultist Map
- [x] TLC_516 Neferset Weaponsmith
- [x] TLC_517 Knockback
- [x] TLC_518 Interrogation
- [x] TLC_520 Underbrush Tracker
- [x] TLC_521 Eyes in the Sky
- [x] TLC_522 Opu the Unseen
- [x] TLC_601 Shellnado
- [x] TLC_603 Platysaur
- [x] TLC_605 Tar Tyrant
- [x] TLC_606 Latorvian Armorer
- [x] TLC_620 Fortify
- [x] TLC_621 Willful Watcher
- [x] TLC_622 City Defenses
- [x] TLC_623 Stonecarver
- [x] TLC_624 Nablya, the Watcher
- [x] TLC_630 Gorishi Wasp
- [x] TLC_632 Story of Sulfuras
- [x] TLC_633 Bugsquasher
- [x] TLC_810 High Cultist Herenn
- [x] TLC_811 Archaios
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
- [x] TLC_836 Niri of the Crater
- [x] TLC_840 Gorishi Tunneler
- [x] TLC_841 Entomologist Toru
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

## Definition of done

All inventory `- [ ]` → `- [x]`; Q1–Q3 landed with F5 scenarios; the quest
zone is exposed in observations/`legal_actions` (decision D3 of the master
roadmap applies — pool policy decides how much surfaces to RL); `cargo test`
green; sub-roadmap archived with the master file.
