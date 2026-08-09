# Expansion Roadmap — Cataclysm (大灾变)

> Status: **active** (created 2026-08-08, part of the 2025–2026 expansions master
> roadmap). Chinese mirror: `expansion-cataclysm-roadmap-zh.md`.
> Scope: ~145 cards (2026-03-17, Patch 35.0, Year of the Scarab's first set).
> No miniset — Blizzard ships "Class Sets" instead (follow-on waves if data
> becomes available). Prerequisite: M0 data pipeline + the three 2025
> sub-roadmaps (Emerald Dream's P1 hero-power machinery is extended here).

## The set

Theme: an alternate timeline where Deathwing won. Verified mechanics
(2026-08-08, official news + patch notes):

- **Deathwing Hero card** — "Deathwing, Worldbreaker": the first Deathwing hero
  card; each turn the player chooses a Cataclysm effect, scaling with a new
  keyword.
- **Colossal returns** — each class gets a legendary Colossal; Colossal +N
  minions summon attached body parts (the engine has no body-part concept).
- **Herald** — Deathwing-allied classes (Death Knight / Demon Hunter / Rogue /
  Shaman / Warlock / Warrior): summons scaling Colossal Soldiers.
- **Shatter** — Dragonflight-allied classes (Mage / Priest / Druid / Hunter /
  Paladin): drawn cards split into two halves that recombine for a doubly
  strong version.
- A new Core Set rotation accompanies the set (engine-side: no-op — both pools
  coexist; note in CLAUDE.md when switching Standard).

## Engine primitives (this roadmap's waves)

| # | Primitive | Used by | Notes |
|---|---|---|---|
| C1 | Colossal body parts | Colossal minions | Attached-part entities summoned with the minion, positioned adjacent, die with it (and part death does not kill the main); new summon path + board occupancy rules |
| C2 | Herald scaling | Herald cards (6 classes) | Per-player Herald counter; summons a Colossal Soldier that scales with the count |
| C3 | Shatter split/recombine | Shatter cards (5 classes) | A drawn card becomes two half-cards (drawn separately), recombining when both halves are held/played — touches draw, hand, play; biggest sequencing risk of the year |
| C4 | Deathwing hero choice | Deathwing hero card | Extends Emerald Dream's P1 hero-power machinery: per-turn choose-a-Cataclysm-effect resolution |
| C5 | misc new CardEffects | remaining cards | Backfilled per wave |

## Wave plan

One PR per wave; every card lands with an F5 differential scenario; `sets.rs`
registration; simplifications get ledger rows.

- [x] **W0 — wiring + data** (PR #155): M0 data for this set landed (M0.1–M0.5, incl. the MEND_ Class Sets merged at M0.3, deferred as a follow-on wave); inventory backfilled; the per-set dump fidelity test (`cataclysm_dump_fidelity`, CATA_ + MEND_ prefixes, 164 cards) landed with the generated baseline; fidelity
  tests.
- [x] **W1 — Colossal (C1)** (PR #156): body-part component + summon path; F5 scenarios pin
  part positioning, part-death rules and main-death cascades.
- [x] **W2 — Herald (C2)** (PR #157): scaling counter + Colossal Soldier pool; the 6
  classes' Herald cards.
- [x] **W3 — Shatter (C3)** (PR #158): split/recombine pipeline; F5 scenarios pin the
  draw-split-recombine sequencing; if the wave judges full fidelity too large,
  register the simplification (D2) instead of shipping a partial mechanic.
- [x] **W4 — Deathwing hero + closing** (PR #159): C4 on P1's machinery; remaining cards;
  Class Sets follow-on deferred; closing wave sweeps tokens/ledger.

## Card inventory

> Backfilled from the M0 data dump (D1). Placeholder grouped by wave.

### W1 — Colossal
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

### W2 — Herald
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

### W3 — Shatter
- [x] CATA_134 Wildwood Circle
- [x] CATA_202 Stolen Power
- [x] CATA_306 Schism
- [x] CATA_479 Flight Maneuvers
- [x] CATA_489 Arcane Flow
- [x] CATA_820 Supply Run

### W4 — Deathwing + remaining cards
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
> Class Sets (follow-up wave, 29 cards — the miniset slot): **still deferred
> after W4** — the MEND_ Class Set cards are NOT part of the W4 scope (the
> follow-on wave lands when the 2025–2026 master roadmap reaches the miniset
> slot; the W0 data and the `cataclysm_dump_fidelity` baseline already cover
> them).
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

## Definition of done

All inventory `- [ ]` → `- [x]`; C1–C5 landed with F5 scenarios (Shatter
sequencing explicitly pinned or registered per D2); `cargo test` green;
sub-roadmap archived with the master file.
