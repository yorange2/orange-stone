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

- **W0 — wiring + data:** M0 data for this set; inventory backfilled; fidelity
  tests.
- **W1 — Rulebreakers batch 1 (V1):** the override framework + the first
  batch of Rulebreakers; F5 scenarios pin each overridden invariant
  (before/after the override).
- **W2 — Rulebreakers batch 2 + closing:** remaining Rulebreakers + the rest
  of the set; closing wave sweeps tokens/ledger.

## Card inventory

> Backfilled from the M0 data dump (D1). Placeholder grouped by wave.

### W1/W2 — Rulebreakers
- [ ] (backfill from data)

### W2 — remaining cards
- [ ] (backfill from data)

## Definition of done

All inventory `- [ ]` → `- [x]`; V1–V2 landed with F5 scenarios; each
override's interaction with the RL surfaces is checked (D3); `cargo test`
green; sub-roadmap archived with the master file — which closes the
2025–2026 master roadmap (M5).
