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

- **W0 — wiring + data:** M0 data for this set; inventory backfilled; fidelity
  tests.
- **W1 — Colossal (C1):** body-part component + summon path; F5 scenarios pin
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
- [ ] (backfill from data)

### W2 — Herald
- [ ] (backfill from data)

### W3 — Shatter
- [ ] (backfill from data)

### W4 — Deathwing + remaining cards
- [ ] (backfill from data)

## Definition of done

All inventory `- [ ]` → `- [x]`; C1–C5 landed with F5 scenarios (Shatter
sequencing explicitly pinned or registered per D2); `cargo test` green;
sub-roadmap archived with the master file.
