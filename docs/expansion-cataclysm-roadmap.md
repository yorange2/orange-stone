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
- **W2 — Herald (C2):** scaling counter + Colossal Soldier pool; the 6
  classes' Herald cards.
- **W3 — Shatter (C3):** split/recombine pipeline; F5 scenarios pin the
  draw-split-recombine sequencing; if the wave judges full fidelity too large,
  register the simplification (D2) instead of shipping a partial mechanic.
- **W4 — Deathwing hero + closing:** C4 on P1's machinery; remaining cards;
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

### W3 — Shatter
- [ ] CATA_134 Wildwood Circle
- [ ] CATA_202 Stolen Power
- [ ] CATA_306 Schism
- [ ] CATA_479 Flight Maneuvers
- [ ] CATA_489 Arcane Flow
- [ ] CATA_820 Supply Run

### W4 — Deathwing + remaining cards
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

## Definition of done

All inventory `- [ ]` → `- [x]`; C1–C5 landed with F5 scenarios (Shatter
sequencing explicitly pinned or registered per D2); `cargo test` green;
sub-roadmap archived with the master file.
