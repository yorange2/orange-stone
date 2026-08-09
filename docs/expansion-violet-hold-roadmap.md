# Expansion Roadmap — Escape from Violet Hold (逃离紫罗兰监狱)

> Status: **active** (created 2026-08-08, part of the 2025–2026 expansions master
> roadmap). Chinese mirror: `expansion-violet-hold-roadmap-zh.md`.
> Scope: ~145 cards (2026-07-07, Patch 36.0, Year of the Scarab's second set).
> Prerequisite: M0 data pipeline + the four earlier sub-roadmaps (all shared
> primitives available).

## The set

Theme: Vanessa VanCleef's crew breaking out of Dalaran's maximum-security
prison. Verified mechanics (2026-08-08, 36.0 patch notes):

- **Rulebreakers** — legendary minions that bend core Hearthstone rules
  (examples cited: duplicate Legendaries, extra draws). These are per-card
  rule exceptions to the rules engine's hardcoded invariants (deck limits,
  draw rates, legendary uniqueness, …).

## Engine primitives (this roadmap's waves)

| # | Primitive | Used by | Notes |
|---|---|---|---|
| V1 | Rule-override framework | Rulebreaker cards | Audit each Rulebreaker against the engine's hardcoded invariants; implement as scoped per-card exceptions (a component + the affected rule reads it), not a generic rules-scripting layer; RL observation/legal-action surfaces re-checked per override (decision D3) |
| V2 | misc new CardEffects | remaining cards | Backfilled per wave |

## Wave plan

One PR per wave; every card lands with an F5 differential scenario; `sets.rs`
registration; simplifications get ledger rows.

- [x] **W0 — wiring + data** (PR #160): M0 data for this set; inventory backfilled; fidelity
  tests.
- [x] **W1 — Rulebreakers batch 1 (V1)** (PR #161): the override framework + the first
  batch of Rulebreakers; F5 scenarios pin each overridden invariant
  (before/after the override).
- **W2 — Rulebreakers batch 2 + closing:** remaining Rulebreakers + the rest
  of the set; closing wave sweeps tokens/ledger.

## Card inventory

> Backfilled from the M0 data dump (D1). Placeholder grouped by wave.

### W1/W2 — Rulebreakers
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
- [ ] JAIL_458 Tiny Pal
- [ ] JAIL_500 Slice and Dice
- [x] JAIL_504 Aya, Lotus Kingpin
- [x] JAIL_509 Godfrey the Betrayer
- [x] JAIL_719 Irida Sinseeker
- [x] JAIL_721 Tras'tath, Soul Parasite
- [x] JAIL_800 Mug'Zee
- [ ] JAIL_831 King of the Underbelly
- [ ] JAIL_850 Warden Maiev
- [ ] JAIL_851 Inspector Murloc Holmes
- [ ] JAIL_852 Togwaggle, Smuggler King
- [ ] JAIL_860 Chef Neth'rek
- [ ] JAIL_875 Staff of Trickery
- [ ] JAIL_882 R4T-C4TCH3R
- [ ] JAIL_887 Zuramat's Prison
- [x] JAIL_906 Moragg
> Rulebreaker wave membership verified per-card text during W1/W2 implementation

### W2 — remaining cards
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

## Definition of done

All inventory `- [ ]` → `- [x]`; V1–V2 landed with F5 scenarios; each
override's interaction with the RL surfaces is checked (D3); `cargo test`
green; sub-roadmap archived with the master file — which closes the
2025–2026 master roadmap (M5).
