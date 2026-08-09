# Expansion Roadmap — Across the Timeways (穿越时光之径)

> Status: **active** (created 2026-08-08, part of the 2025–2026 expansions master
> roadmap). Chinese mirror: `expansion-timeways-roadmap-zh.md`.
> Scope: 145 cards (2025-11-04, Patch 34.0) + the **The End of Time** miniset
> (38 cards, 2026-01). Prerequisite: M0 data pipeline + Emerald Dream / Un'Goro
> sub-roadmaps (their P3 choice machinery is reused here).

## The set

Theme: Chromie recruiting heroes across timelines to fight Murozond. Verified
mechanics (2026-08-08, 34.0 patch notes):

- **Rewind** — time-travel keyword: replays the previous card/effect. Patch
  34.0 nerfed Mister Clocksworth from "Rewind x3" to "Rewind x2", confirming
  the keyword stacks (multi-count replay).
- **Fabled** — legendary minions from alternate timelines (a parallel set of
  reimagined legendaries; exact card rules from data).
- Headline card (verified): Mister Clocksworth (Rewind x3 → x2).

## Engine primitives (this roadmap's waves)

| # | Primitive | Used by | Notes |
|---|---|---|---|
| R1 | Effect replay / last-card history | Rewind cards | Per-player history of the last played card (and the one before, for x2); replay re-runs the stored effects — must interact correctly with the death phase, targeting rules and the replay card itself leaving play |
| R2 | Fabled variants | Fabled legendaries | Parallel legendary forms — mostly data/def work once R1's replay machinery exists; misc new CardEffects backfilled |
| R3 | misc new CardEffects | remaining cards | Backfilled per wave |

## Wave plan

One PR per wave; every card lands with an F5 differential scenario; `sets.rs`
registration; simplifications get ledger rows.

- [x] **W0 — wiring + data** (PR #150): M0 data for this set landed
  (M0.1–M0.5); inventory backfilled; the per-set dump fidelity test
  (`across_the_timeways_dump_fidelity`, TIME_ + END_ prefixes, 183 cards)
  landed with the generated baseline.
- [x] **W1 — Rewind (R1)** (PR #151): `Player.last_played` RewindEntry
  history (capped 10) + `cards::rewind` registry (`rewind_count` table
  for all 17 cards, `REWIND_CARD_IDS`) + `engine::rewind` replay
  (pre-push snapshot, count clamped, chronological, rewind card as
  source); play path: own effect → replay → push; 8 `tmw1_*` F5
  scenarios pin ordering, x3 stacking, self-exclusion, snapshot
  semantics, empty-slot clamping and death-phase interplay.
- **W2 — Rewind cards + Fabled (R2):** the set's Rewind cards on R1; Fabled
  legendaries; misc effects.
- **W3 — The End of Time mini:** miniset cards (mechanics backfilled from
  data); closing wave sweeps tokens/ledger.

## Card inventory

> Backfilled from the M0 data dump (D1). Placeholder grouped by wave.

### W1/W2 — Rewind + Fabled
- [x] TIME_000 Semi-Stable Portal
- [x] TIME_001 Chrono Daggers
- [x] TIME_002 Aeon Wizard
- [x] TIME_003 Portal Vanguard
- [x] TIME_004 Conflux Crasher
- [x] TIME_008 Bygone Doomspeaker
- [x] TIME_014 Instant Multiverse
- [x] TIME_018 Mend the Timeline
- [x] TIME_033 Druid of Regrowth
- [x] TIME_034 Stadium Announcer
- [x] TIME_035 Time Machine
- [x] TIME_038 Mister Clocksworth
- [x] TIME_433 Cease to Exist
- [x] TIME_441 Aeon Rend
- [x] TIME_602 Wormhole
- [x] TIME_610 Shadows of Yesterday
- [x] TIME_005 Timethief Rafaam
- [x] TIME_009 Gelbin of Tomorrow
- [x] TIME_020 Broxigar
- [x] TIME_209 Muradin, High King
- [x] TIME_211 Lady Azshara
- [x] TIME_609 Ranger General Sylvanas
- [x] TIME_619 Talanji of the Graves
- [x] TIME_850 Lo'Gosh, Blood Fighter
- [x] TIME_852 Azure Queen Sindragosa
- [x] TIME_875 Garona Halforcen
- [x] TIME_890 Medivh the Hallowed
> note: Lady Azshara is also a Choose One card: TIME_211

### W3 — remaining cards + mini
- [x] TIME_006 Mirror Dimension
- [x] TIME_013 Farseer Wo
- [x] TIME_015 Hardlight Protector
- [x] TIME_016 Neon Innovation
- [x] TIME_017 Tankgineer
- [x] TIME_019 Manifested Timeways
- [x] TIME_021 Doomsday Prepper
- [x] TIME_022 Perennial Serpent
- [x] TIME_023 Contingency
- [x] TIME_024 Murozond, Unbounded
- [x] TIME_025 Twilight Timehopper
- [x] TIME_026 Entropic Continuity
- [x] TIME_027 Tachyon Barrage
- [x] TIME_028 Fatebreaker
- [x] TIME_029 Ruinous Velocidrake
- [x] TIME_030 Divergence
- [x] TIME_031 RAFAAM LADDER!!
- [x] TIME_032 Chronogor
- [x] TIME_036 Royal Informant
- [x] TIME_037 Disciple of the Dove
- [x] TIME_039 Deja Vu
- [x] TIME_040 Fading Memory
- [x] TIME_041 Futuristic Forefather
- [x] TIME_042 King Maluk
- [x] TIME_043 PMM Infinitizer
- [x] TIME_044 Past Gnomeregan
- [x] TIME_045 Whelp of the Infinite
- [x] TIME_046 Cyborg Patriarch
- [x] TIME_047 Devious Coyote
- [x] TIME_048 Clockwork Rager
- [x] TIME_049 Dangerous Variant
- [x] TIME_050 Sentient Hourglass
- [x] TIME_051 Soldier of the Infinite
- [x] TIME_052 Amber Warden
- [x] TIME_053 Sandmaw
- [x] TIME_054 Time Skipper
- [x] TIME_055 Unknown Voyager
- [x] TIME_056 Whelp of the Bronze
- [x] TIME_057 Wizened Truthseeker
- [x] TIME_058 Paltry Flutterwing
- [x] TIME_059 Living Paradox
- [x] TIME_060 Quantum Destabilizer
- [x] TIME_061 Timeless Causality
- [x] TIME_062 Chronicle Keeper
- [x] TIME_063 Timelord Nozdormu
- [x] TIME_064 Chrono-Lord Deios
- [x] TIME_100 Hourglass Attendant
- [x] TIME_101 Misplaced Pyromancer
- [x] TIME_102 Circadiamancer
- [x] TIME_103 Chromie
- [x] TIME_212 Lightning Rod
- [x] TIME_213 Primordial Overseer
- [x] TIME_214 Flux Revenant
- [x] TIME_215 Thunderquake
- [x] TIME_216 Nascent Bolt
- [x] TIME_217 Stormrook
- [x] TIME_218 Static Shock
- [x] TIME_427 Cleansing Lightspawn
- [x] TIME_428 Yesterloc
- [x] TIME_429 Divine Augur
- [x] TIME_431 Amber Priestess
- [x] TIME_432 Intertwined Fate
- [x] TIME_434 Temporal Traveler
- [x] TIME_435 Eternus
- [x] TIME_436 Past Conflux
- [x] TIME_442 Timeway Warden
- [x] TIME_443 Hounds of Fury
- [x] TIME_444 Time-Lost Glaive
- [x] TIME_446 The Eternal Hold
- [x] TIME_447 Power Word: Barrier
- [x] TIME_448 Solitude
- [x] TIME_449 Lasting Legacy
- [x] TIME_600 Precise Shot
- [x] TIME_601 Arrow Retriever
- [x] TIME_603 Ticking Timebomb
- [x] TIME_605 Epoch Stalker
- [x] TIME_606 Quel'dorei Fletcher
- [x] TIME_611 Timestop
- [x] TIME_612 Blood Draw
- [x] TIME_613 Cryofrozen Champion
- [x] TIME_614 Liferender
- [x] TIME_615 Forgotten Millennium
- [x] TIME_616 Memoriam Manifest
- [x] TIME_617 Chronochiller
- [x] TIME_618 Husk, Eternal Reaper
- [x] TIME_620 Untimely Death
- [x] TIME_700 Chronological Aura
- [x] TIME_701 Waveshaping
- [x] TIME_702 Ebb and Flow
- [x] TIME_703 Endangered Dodo
- [x] TIME_704 Highborne Mentor
- [x] TIME_705 Krona, Keeper of Eons
- [x] TIME_706 The Fins Beyond Time
- [x] TIME_707 Alternate Reality
- [x] TIME_710 Troubled Double
- [x] TIME_711 Flashback
- [x] TIME_712 Dethrone
- [x] TIME_713 Time Adm'ral Hooktail
- [x] TIME_714 Chrono-Lord Epoch
- [x] TIME_715 For Glory!
- [x] TIME_716 Slow Motion
- [x] TIME_720 Soldier of the Bronze
- [x] TIME_730 Kaldorei Cultivator
- [x] TIME_750 Precursory Strike
- [x] TIME_770 Fast Forward
- [x] TIME_810 Past Silvermoon
- [x] TIME_855 Arcane Barrage
- [x] TIME_856 Algeth'ar Instructor
- [x] TIME_857 Alter Time
- [x] TIME_858 Temporal Construct
- [x] TIME_859 Anomalize
- [x] TIME_860 Faceless Enigma
- [x] TIME_861 Timelooper Toki
- [x] TIME_870 Gladiatorial Combat
- [x] TIME_871 Heir of Hereafter
- [x] TIME_872 Undefeated Champion
- [x] TIME_873 Unleash the Crocolisks
- [x] TIME_876 Shapeshifter
- [ ] END_000 Eventuality
- [ ] END_001 Jagged Edge of Time
- [ ] END_002 Wicked Blightspawn
- [ ] END_003 Finality
- [ ] END_004 Remnant of Rage
- [ ] END_005 Bygone Echoes
- [ ] END_006 Chronikar
- [ ] END_007 Press the Advantage
- [ ] END_008 Enduring Roach
- [ ] END_009 Splintered Reality
- [ ] END_010 Twilight Timereaver
- [ ] END_011 Acceleration Aura
- [ ] END_012 Hand of Infinity
- [ ] END_013 Brutish Endmaw
- [ ] END_014 Synchronized Spark
- [ ] END_015 Triennium Rex
- [ ] END_016 Chronoclaws
- [ ] END_017 Battle at the End Time
- [ ] END_018 Acolyte of Infinity
- [ ] END_019 Endtime Survivor
- [ ] END_020 Eternal Toil
- [ ] END_021 Dimensional Weaponsmith
- [ ] END_022 Time-Twisted Seer
- [ ] END_023 Bitter End
- [ ] END_024 Flames of Infinity
- [ ] END_025 Eternal Firebolt
- [ ] END_026 Fragment of Nothing
- [ ] END_027 Wings of Eternity
- [ ] END_028 For All Time
- [ ] END_029 Voodoo Totem
- [ ] END_030 Haywire Hornswog
- [ ] END_031 Shade of the End Time
- [ ] END_032 Winged Aberration
- [ ] END_033 Prescient Slitherdrake
- [ ] END_034 Crumblecrusher
- [ ] END_035 Omen of the End
- [ ] END_036 Morchie
- [ ] END_037 Endtime Murozond

## Definition of done

All inventory `- [ ]` → `- [x]`; R1–R3 landed with F5 scenarios (replay
sequencing pinned); `cargo test` green; sub-roadmap archived with the master
file.
