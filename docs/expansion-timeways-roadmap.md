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

- **W0 — wiring + data:** M0 data for this set; inventory backfilled; fidelity
  tests.
- **W1 — Rewind (R1):** last-card history component + replay resolution; F5
  scenarios pin replay ordering (replay happens after the triggering play
  resolves), x2/x3 stacking, and interplay with death phases.
- **W2 — Rewind cards + Fabled (R2):** the set's Rewind cards on R1; Fabled
  legendaries; misc effects.
- **W3 — The End of Time mini:** miniset cards (mechanics backfilled from
  data); closing wave sweeps tokens/ledger.

## Card inventory

> Backfilled from the M0 data dump (D1). Placeholder grouped by wave.

### W1/W2 — Rewind + Fabled
- [ ] (backfill from data)

### W3 — remaining cards + mini
- [ ] (backfill from data)

## Definition of done

All inventory `- [ ]` → `- [x]`; R1–R3 landed with F5 scenarios (replay
sequencing pinned); `cargo test` green; sub-roadmap archived with the master
file.
