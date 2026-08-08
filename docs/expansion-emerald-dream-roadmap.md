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
- **Smoldering** (miniset) — a marked minion's Deathrattle triggers an extra time.
- Headline cards (verified): Ysera, Emerald Aspect (free legendary),
  Fyrakk, the Blazing (miniset legendary).

## Engine primitives (this roadmap's waves)

| # | Primitive | Used by | Notes |
|---|---|---|---|
| P1 | Imbue: hero-power upgrade | Imbue cards (6 classes) | `HeroPowerDef` is static per-def; needs a per-player upgrade counter + threshold event + hero-power swap component (see also Cataclysm's Deathwing choice — design once, share) |
| P2 | Dark Gifts: fixed Discover pool | 5 classes | ~10 power-up tokens; a fixed pool like `DREAM_POOL`/`POOL_OPEN_CARDS`, Discover simplified → random selection unless D2 rules otherwise |
| P3 | Real Choose One resolution | all classes | Pay the auto-random debt: `Resolution::NeedsChoice` surfaced through `legal_actions`/bindings (the Discover choice machinery from W10 already exists as the pattern) |
| P4 | Smoldering: extra Deathrattle | miniset | A `Smoldering` component; the death phase fires the Deathrattle twice when marked |
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
- **W5 — Embers of the World Tree (P4):** Smoldering component + miniset cards;
  F5 scenarios; miniset registered.

## Card inventory

> Backfilled from the M0 data dump (D1). Placeholder grouped by the wave that
> carries each mechanic; every `- [ ]` is one card.

### W1 — Imbue (Druid/Hunter/Mage/Paladin/Priest/Shaman)
- [ ] (backfill from data)

### W2 — Dark Gifts (DK/DH/Rogue/Warlock/Warrior)
- [ ] (backfill from data)

### W3 — Choose One (all classes)
- [ ] (backfill from data)

### W4 — Wild Gods + remaining cards
- [ ] (backfill from data)

### W5 — Embers of the World Tree miniset (Smoldering)
- [ ] (backfill from data)

## Definition of done

All inventory `- [ ]` → `- [x]`; P1–P5 landed with F5 scenarios; the Choose-One
simplification is resolved (or explicitly kept with a ledger row per D2);
`cargo test` green; sub-roadmap moves to `docs/finished/` with the master file.
