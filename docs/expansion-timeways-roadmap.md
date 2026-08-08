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
- [ ] TIME_000 Semi-Stable Portal
- [ ] TIME_001 Chrono Daggers
- [ ] TIME_002 Aeon Wizard
- [ ] TIME_003 Portal Vanguard
- [ ] TIME_004 Conflux Crasher
- [ ] TIME_008 Bygone Doomspeaker
- [ ] TIME_014 Instant Multiverse
- [ ] TIME_018 Mend the Timeline
- [ ] TIME_033 Druid of Regrowth
- [ ] TIME_034 Stadium Announcer
- [ ] TIME_035 Time Machine
- [ ] TIME_038 Mister Clocksworth
- [ ] TIME_433 Cease to Exist
- [ ] TIME_441 Aeon Rend
- [ ] TIME_602 Wormhole
- [ ] TIME_610 Shadows of Yesterday
- [ ] TIME_005 Timethief Rafaam
- [ ] TIME_009 Gelbin of Tomorrow
- [ ] TIME_020 Broxigar
- [ ] TIME_209 Muradin, High King
- [ ] TIME_211 Lady Azshara
- [ ] TIME_609 Ranger General Sylvanas
- [ ] TIME_619 Talanji of the Graves
- [ ] TIME_850 Lo'Gosh, Blood Fighter
- [ ] TIME_852 Azure Queen Sindragosa
- [ ] TIME_875 Garona Halforcen
- [ ] TIME_890 Medivh the Hallowed
> note: Lady Azshara is also a Choose One card: TIME_211

### W3 — remaining cards + mini
- [ ] TIME_006 Mirror Dimension
- [ ] TIME_013 Farseer Wo
- [ ] TIME_015 Hardlight Protector
- [ ] TIME_016 Neon Innovation
- [ ] TIME_017 Tankgineer
- [ ] TIME_019 Manifested Timeways
- [ ] TIME_021 Doomsday Prepper
- [ ] TIME_022 Perennial Serpent
- [ ] TIME_023 Contingency
- [ ] TIME_024 Murozond, Unbounded
- [ ] TIME_025 Twilight Timehopper
- [ ] TIME_026 Entropic Continuity
- [ ] TIME_027 Tachyon Barrage
- [ ] TIME_028 Fatebreaker
- [ ] TIME_029 Ruinous Velocidrake
- [ ] TIME_030 Divergence
- [ ] TIME_031 RAFAAM LADDER!!
- [ ] TIME_032 Chronogor
- [ ] TIME_036 Royal Informant
- [ ] TIME_037 Disciple of the Dove
- [ ] TIME_039 Deja Vu
- [ ] TIME_040 Fading Memory
- [ ] TIME_041 Futuristic Forefather
- [ ] TIME_042 King Maluk
- [ ] TIME_043 PMM Infinitizer
- [ ] TIME_044 Past Gnomeregan
- [ ] TIME_045 Whelp of the Infinite
- [ ] TIME_046 Cyborg Patriarch
- [ ] TIME_047 Devious Coyote
- [ ] TIME_048 Clockwork Rager
- [ ] TIME_049 Dangerous Variant
- [ ] TIME_050 Sentient Hourglass
- [ ] TIME_051 Soldier of the Infinite
- [ ] TIME_052 Amber Warden
- [ ] TIME_053 Sandmaw
- [ ] TIME_054 Time Skipper
- [ ] TIME_055 Unknown Voyager
- [ ] TIME_056 Whelp of the Bronze
- [ ] TIME_057 Wizened Truthseeker
- [ ] TIME_058 Paltry Flutterwing
- [ ] TIME_059 Living Paradox
- [ ] TIME_060 Quantum Destabilizer
- [ ] TIME_061 Timeless Causality
- [ ] TIME_062 Chronicle Keeper
- [ ] TIME_063 Timelord Nozdormu
- [ ] TIME_064 Chrono-Lord Deios
- [ ] TIME_100 Hourglass Attendant
- [ ] TIME_101 Misplaced Pyromancer
- [ ] TIME_102 Circadiamancer
- [ ] TIME_103 Chromie
- [ ] TIME_212 Lightning Rod
- [ ] TIME_213 Primordial Overseer
- [ ] TIME_214 Flux Revenant
- [ ] TIME_215 Thunderquake
- [ ] TIME_216 Nascent Bolt
- [ ] TIME_217 Stormrook
- [ ] TIME_218 Static Shock
- [ ] TIME_427 Cleansing Lightspawn
- [ ] TIME_428 Yesterloc
- [ ] TIME_429 Divine Augur
- [ ] TIME_431 Amber Priestess
- [ ] TIME_432 Intertwined Fate
- [ ] TIME_434 Temporal Traveler
- [ ] TIME_435 Eternus
- [ ] TIME_436 Past Conflux
- [ ] TIME_442 Timeway Warden
- [ ] TIME_443 Hounds of Fury
- [ ] TIME_444 Time-Lost Glaive
- [ ] TIME_446 The Eternal Hold
- [ ] TIME_447 Power Word: Barrier
- [ ] TIME_448 Solitude
- [ ] TIME_449 Lasting Legacy
- [ ] TIME_600 Precise Shot
- [ ] TIME_601 Arrow Retriever
- [ ] TIME_603 Ticking Timebomb
- [ ] TIME_605 Epoch Stalker
- [ ] TIME_606 Quel'dorei Fletcher
- [ ] TIME_611 Timestop
- [ ] TIME_612 Blood Draw
- [ ] TIME_613 Cryofrozen Champion
- [ ] TIME_614 Liferender
- [ ] TIME_615 Forgotten Millennium
- [ ] TIME_616 Memoriam Manifest
- [ ] TIME_617 Chronochiller
- [ ] TIME_618 Husk, Eternal Reaper
- [ ] TIME_620 Untimely Death
- [ ] TIME_700 Chronological Aura
- [ ] TIME_701 Waveshaping
- [ ] TIME_702 Ebb and Flow
- [ ] TIME_703 Endangered Dodo
- [ ] TIME_704 Highborne Mentor
- [ ] TIME_705 Krona, Keeper of Eons
- [ ] TIME_706 The Fins Beyond Time
- [ ] TIME_707 Alternate Reality
- [ ] TIME_710 Troubled Double
- [ ] TIME_711 Flashback
- [ ] TIME_712 Dethrone
- [ ] TIME_713 Time Adm'ral Hooktail
- [ ] TIME_714 Chrono-Lord Epoch
- [ ] TIME_715 For Glory!
- [ ] TIME_716 Slow Motion
- [ ] TIME_720 Soldier of the Bronze
- [ ] TIME_730 Kaldorei Cultivator
- [ ] TIME_750 Precursory Strike
- [ ] TIME_770 Fast Forward
- [ ] TIME_810 Past Silvermoon
- [ ] TIME_855 Arcane Barrage
- [ ] TIME_856 Algeth'ar Instructor
- [ ] TIME_857 Alter Time
- [ ] TIME_858 Temporal Construct
- [ ] TIME_859 Anomalize
- [ ] TIME_860 Faceless Enigma
- [ ] TIME_861 Timelooper Toki
- [ ] TIME_870 Gladiatorial Combat
- [ ] TIME_871 Heir of Hereafter
- [ ] TIME_872 Undefeated Champion
- [ ] TIME_873 Unleash the Crocolisks
- [ ] TIME_876 Shapeshifter
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
