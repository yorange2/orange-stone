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

- **W0 — wiring + data:** M0 data for this set; inventory backfilled; generated
  baseline + fidelity tests.
- **W1 — Quest zone (Q1):** `Zone::Quest` + quest component + progress event
  plumbing + reward resolution; engine-level quest smoke scenarios before any
  quest card lands.
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
- [ ] (backfill from data)

### W3 — Kindred
- [ ] (backfill from data)

### W4 — remaining cards + mini
- [ ] (backfill from data)

## Definition of done

All inventory `- [ ]` → `- [x]`; Q1–Q3 landed with F5 scenarios; the quest
zone is exposed in observations/`legal_actions` (decision D3 of the master
roadmap applies — pool policy decides how much surfaces to RL); `cargo test`
green; sub-roadmap archived with the master file.
