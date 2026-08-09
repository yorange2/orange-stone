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

- [x] **W0 — wiring + data (no cards)**: M0 data for this set landed (M0.1–M0.5,
  PR #131–#135: schema fields, generated baseline, inventory backfilled);
  ID-prefix + name fidelity tests (M1-W0, PR #136: `tests/expansion_fidelity.rs`,
  EDR_/FIR_ bidirectional coverage over 183 cards).
- [x] **W1 — Imbue (P1)** (PR #137): per-player imbue counter + first-imbue
  hero-power swap to the class's Imbued form + per-imbue number upgrades
  (level = count); the 6 classes' 15 Imbue cards; 13 F5 scenarios pin the
  2x-Imbue threshold sequencing and per-class power resolution. 5 D2
  simplifications registered in fidelity-debt.md §14.
- [x] **W2 — Dark Gifts (P2)** (PR #138): the 10-gift pool (`ALL_DARK_GIFTS`)
  + card-level gift markers + per-player gift log; the 5 classes' 9 Dark Gift
  cards; 15 F5 scenarios (per-gift resolution, cross-zone persistence,
  Combo/corpse/dragon-hand conditions, Wallow sync). Nightmare Fuel registered
  in POOL_OPEN_CARDS; 7 D2 simplifications in fidelity-debt.md §14.1.
- [x] **W3 — Choose One for all classes (P3)** (PR #139): the auto-random debt
  is paid — pending choices surface in `legal_actions` (option-wise
  `Action::Choose`, `ChoicePending` gate) and the structured views/bindings;
  three surface sites (spell/minion/weapon, the weapon path added) with
  queued-event ordering fix; 12 EDR choose-one cards + 6 tokens; 14 F5
  scenarios pin both branches; 4 D2 simplifications in fidelity-debt.md §14.2.
- [x] **W4 — Wild Gods + misc effects (P5)**: W4a (PR #140 — the 86
  non-legendary remaining cards + 9 tokens, 63 effect variants, 3 engine bug
  fixes pinned by F5, 36 D2 simplifications in §14.3, 14 scenarios) + W4b
  (PR #141 — the 23 legendary Wild Gods: choose-thrice, keep-or-top choice,
  per-player attack/double-cast/cost flags, Ursoc's kill-record resurrection;
  23 F5 scenarios, 13 D2 simplifications in §14.4, 2 tokens; Ashamane
  registered in `POOL_OPEN_CARDS`). All 145 main-set cards complete.
- [x] **W5 — Embers of the World Tree miniset** (PR #142): all 38 miniset
  cards + the Broodling token landed on the existing primitives (7 Dark
  Gifts, Discover, Battlecry, imbue conditions, corpses; 30 effect
  variants); 38 F5 scenarios cover every card; 25 D2 simplifications in
  §14.5. **M1 complete (145 + 38).**

## Card inventory

> Backfilled from the M0 data dump (D1). Placeholder grouped by the wave that
> carries each mechanic; every `- [ ]` is one card.

### W1 — Imbue (Druid/Hunter/Mage/Paladin/Priest/Shaman)
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

### W2 — Dark Gifts (DK/DH/Rogue/Warlock/Warrior)
- [x] EDR_102 Treacherous Tormentor
- [x] EDR_456 Darkrider
- [x] EDR_487 Wallow, the Wretched
- [x] EDR_488 Avant-Gardening
- [x] EDR_528 Nightmare Fuel
- [x] EDR_654 Overgrown Horror
- [x] EDR_811 Rite of Atrocity
- [x] EDR_856 Nightmare Lord Xavius
- [x] EDR_882 Jumpscare!

### W3 — Choose One (all classes)
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

### W4 — Wild Gods + remaining cards
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

### W5 — Embers of the World Tree miniset
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

## Definition of done

All inventory `- [ ]` → `- [x]`; P1–P3 and P5 landed with F5 scenarios (P4
Smoldering removed per the M0.5 data check); the Choose-One simplification is
resolved (or explicitly kept with a ledger row per D2); `cargo test` green;
sub-roadmap moves to `docs/finished/` with the master file.
