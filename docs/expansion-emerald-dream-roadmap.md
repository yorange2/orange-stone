# Expansion Roadmap — Into the Emerald Dream (翡翠梦境)

> Status: **active** (created 2026-08-08, part of the 2025–2026 expansions master
> roadmap). Chinese mirror: `expansion-emerald-dream-roadmap-zh.md`.
> Scope: 145 cards (2025-03-25, Patch 32.0, Year of the Raptor's first set)
> + the **Embers of the World Tree** miniset (38 cards, 2025-05-13).
> Prerequisite: M0 data pipeline (`2025-2026-expansions-roadmap.md`).

## The set

Theme: Ysera's Emerald Dream vs. the Old Gods' corruption. Verified mechanics
(2026-08-08, official news + patch notes):

- **Imbue** — Druid / Hunter / Mage / Paladin / Priest / Shaman. Playing cards
  with Imbue (or meeting their Imbue condition) upgrades the class hero power;
  the second Imbue triggers the powered-up form.
- **Dark Gifts** — Death Knight / Demon Hunter / Rogue / Warlock / Warrior.
  Discover one of ~10 power-ups applied to a minion.
- **Choose One** for every class — the first real collision with the engine's
  registered auto-random simplification.
- **Wild Gods** — one legendary per class.
- ~~**Smoldering**~~ — **removed (M0.5, 2026-08-08, data check)**: the
  hearthstonejson miniset data (FIR_*) carries no Smoldering keyword and no
  "Deathrattle triggers an extra time" text; the 38 miniset cards continue
  existing mechanics (7 Dark Gifts, Battlecry/Discover, …).
- Headline cards (verified): Ysera, Emerald Aspect (free legendary),
  Fyrakk, the Blazing (miniset legendary).

## Engine primitives (this roadmap's waves)

| # | Primitive | Used by | Notes |
|---|---|---|---|
| P1 | Imbue: hero-power upgrade | Imbue cards (6 classes) | `HeroPowerDef` is static per-def; needs a per-player upgrade counter + threshold event + hero-power swap component (see also Cataclysm's Deathwing choice — design once, share) |
| P2 | Dark Gifts: fixed Discover pool | 5 classes | ~10 power-up tokens; a fixed pool like `DREAM_POOL`/`POOL_OPEN_CARDS`, Discover simplified → random selection unless D2 rules otherwise |
| P3 | Real Choose One resolution | all classes | Pay the auto-random debt: `Resolution::NeedsChoice` surfaced through `legal_actions`/bindings (the Discover choice machinery from W10 already exists as the pattern) |
| ~~P4~~ | ~~Smoldering: extra Deathrattle~~ | ~~miniset~~ | **removed (M0.5)**: no Smoldering keyword in the data; the miniset uses existing primitives |
| P5 | Wild Gods / misc new CardEffects | various | Backfilled from card text during each wave |

## Wave plan

One PR per wave; every card lands with an F5 differential scenario; cards
register in `sets.rs::ALL_CARDS`; simplifications carry `(simplified …)` markers
+ `fidelity-debt.md` rows.

- **W0 — wiring + data (no cards):** M0 data for this set lands (schema fields,
  generated baseline); the card inventory below is backfilled from the dump;
  ID-prefix + name fidelity tests for this set's generated cards.
- **W1 — Imbue (P1):** hero-power upgrade counter + threshold trigger +
  hero-power swap; the 6 classes' Imbue cards (W2+ in EDR terms); F5 scenarios
  pin the 2x-Imbue threshold sequencing.
- **W2 — Dark Gifts (P2):** the 10-gift pool + Discover-application on the 5
  classes' Dark Gift cards.
- **W3 — Choose One for all classes (P3):** real choice resolution; the W10
  Discover choice pattern generalized; EDR choose-one cards playable with
  explicit branches in `legal_actions`.
- **W4 — Wild Gods + misc effects (P5):** remaining minions/spells; the
  legendary Wild Gods; closing wave sweeps tokens/enchantments and the ledger.
- **W5 — Embers of the World Tree miniset:** the 38 miniset cards (existing
  primitives — 7 more Dark Gifts, Battlecry/Discover, …); F5 scenarios;
  miniset registered.

## Card inventory

> Backfilled from the M0 data dump (D1). Placeholder grouped by the wave that
> carries each mechanic; every `- [ ]` is one card.

### W1 — Imbue (Druid/Hunter/Mage/Paladin/Priest/Shaman)
- [ ] EDR_226 Exotic Houndmaster
- [ ] EDR_227 Umbraclaw
- [ ] EDR_231 Aspect's Embrace
- [ ] EDR_264 Aegis of Light
- [ ] EDR_449 Lunarwing Messenger
- [ ] EDR_451 Goldpetal Drake
- [ ] EDR_518 Living Garden
- [ ] EDR_519 Wisprider
- [ ] EDR_800 Flutterwing Guardian
- [ ] EDR_845 Hamuul Runetotem
- [ ] EDR_852 Bitterbloom Knight
- [ ] EDR_860 Resplendent Dreamweaver
- [ ] EDR_871 Spirit Gatherer
- [ ] EDR_888 Malorne the Waywatcher
- [ ] EDR_970 Kaldorei Priestess

### W2 — Dark Gifts (DK/DH/Rogue/Warlock/Warrior)
- [ ] EDR_102 Treacherous Tormentor
- [ ] EDR_456 Darkrider
- [ ] EDR_487 Wallow, the Wretched
- [ ] EDR_488 Avant-Gardening
- [ ] EDR_528 Nightmare Fuel
- [ ] EDR_654 Overgrown Horror
- [ ] EDR_811 Rite of Atrocity
- [ ] EDR_856 Nightmare Lord Xavius
- [ ] EDR_882 Jumpscare!

### W3 — Choose One (all classes)
- [ ] EDR_233 Spirits of the Forest
- [ ] EDR_257 Lightmender
- [ ] EDR_263 Grace of the Greatwolf
- [ ] EDR_273 Symbiosis
- [ ] EDR_463 Twilight Influence
- [ ] EDR_490 Sleep Paralysis
- [ ] EDR_525 Barbed Thorn
- [ ] EDR_570 Ominous Nightmares
- [ ] EDR_813 Morbid Swarm
- [ ] EDR_820 Wyvern's Slumber
- [ ] EDR_843 Reforestation
- [ ] EDR_872 Spark of Life

### W4 — Wild Gods + remaining cards
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

### W5 — Embers of the World Tree miniset
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

## Definition of done

All inventory `- [ ]` → `- [x]`; P1–P3 and P5 landed with F5 scenarios (P4
Smoldering removed per the M0.5 data check); the Choose-One simplification is
resolved (or explicitly kept with a ledger row per D2); `cargo test` green;
sub-roadmap moves to `docs/finished/` with the master file.
