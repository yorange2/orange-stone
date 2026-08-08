# 2025–2026 Expansions Roadmap — implementing the Year of the Raptor + Year of the Scarab sets

> Status: **active** (created 2026-08-08). Chinese mirror: `2025-2026-expansions-roadmap-zh.md`.
> Scope: the five full expansions released between 2025-03 and 2026-07 (~730 cards)
> plus their three minisets (~114 cards) — **Into the Emerald Dream**, **The Lost City
> of Un'Goro**, **Across the Timeways** (2025, Year of the Raptor) and **Cataclysm**,
> **Escape from Violet Hold** (2026, Year of the Scarab). The late-2026 expansion
> (not yet announced as of 2026-08-08) is appended when announced. This is the
> master roadmap; each expansion has its own sub-roadmap (index below).

## Why

The engine currently covers Classic (316) + the modern Core Set (270 real cards).
The 2025–2026 expansions are the next step for three reasons:

1. **The real Hearthstone now.** Any SabberStone differential comparison and any
   "current Standard" training claim needs the actual Standard sets, not a
   2021-era card pool. 2025–2026 is where the live game is.
2. **Mechanic depth.** These five sets introduce mechanics the engine has never
   had: hero-power upgrades (Imbue), the Quest zone (Un'Goro), effect replay
   (Rewind), body-part minions (Colossal), card splitting (Shatter), and rule
   overrides (Rulebreakers). Each is a permanent engine primitive, not a card
   patch — same pay-off shape as the Core Set W-waves.
3. **The debt it pays.** Emerald Dream's "Choose One for every class" collides
   head-on with the registered Choose-One-auto-random simplification — this
   roadmap is the natural place to resolve it for real.

**This roadmap is the architecture and the index.** Card-level inventories live
in the sub-roadmaps, backfilled from the data pipeline (M0) — the checkbox
lists are filled in as data lands, exactly as the Core Set roadmap was written
against its inventory.

## The five expansions (verified against official/news sources, 2026-08-08)

| # | Expansion | Release | Cards | Mechanics (new to the engine in **bold**) |
|---|---|---|---|---|
| 1 | Into the Emerald Dream | 2025-03-25 (32.0) | 145 | **Imbue** (6 classes), **Dark Gifts** (5 classes), Choose One for every class, Wild Gods |
| 1m | Embers of the World Tree (mini) | 2025-05-13 | 38 | **Smoldering** |
| 2 | The Lost City of Un'Goro | 2025-07-08 (33.0) | 145 | **Quests** (return), **Kindred** |
| 2m | Festival of the Devilsaur (mini) | 2025-08/09 | 38 | TBD from data |
| 3 | Across the Timeways | 2025-11-04 (34.0) | 145 | **Rewind**, Fabled legendaries |
| 3m | The End of Time (mini) | 2026-01 | 38 | TBD from data |
| 4 | Cataclysm | 2026-03-17 (35.0) | ~145 | **Colossal** (return), **Herald**, **Shatter**, Deathwing Hero card; no miniset (Class Sets instead) |
| 5 | Escape from Violet Hold | 2026-07-07 (36.0) | ~145 | **Rulebreakers** |

~845 cards total including minisets (exact count after M0 data import; tokens,
enchantments and generated-card backlog follow the Core Set counting rules).

## M0 — data pipeline (prerequisite for every sub-roadmap)

`cards/cards.json` (1190 entries, minimal schema `id/name/cost/attack/health/type/mechanics`)
contains essentially **zero** real 2025–2026 cards — the DREAM/TOY/GIFT/Story
prefixes present are Ysera's dream cards, the Gift heroes and puzzle cards, not
the expansions. M0 is the foundation:

- [x] **Data source (decision D1)** — acquire the real card dumps. Decided
  (2026-08-08): **hearthstonejson dump**
  (`api.hearthstonejson.com/v1/latest/enUS/cards.collectible.json`, community,
  full card DB, the SabberStone ecosystem's source). Sliced per set into
  `cards/data/EMERALD_DREAM.json` (183, 38 mini), `THE_LOST_CITY.json` (183,
  38 mini), `TIME_TRAVEL.json` (183, 38 mini), `CATACLYSM.json` (164, 29 Class
  Sets), `ESCAPEFROM_VIOLET_HOLD.json` (135); every card carries
  id/name/cost/type/attack/health/durability, **race/class**, **text** (effect
  prose), keywords (mechanics) and collectibility. Reproduce via
  `tools/fetch_expansion_sets.py`; dump sha256 pinned in `cards/data/SOURCE.md`.
- [x] **Schema extension** — `cards/cards.json` entries gain optional
  `text`/`race`/`cardClass`/`classes`/`set`/`collectible` fields (M0.2, PR #132:
  764/1190 entries backfilled by ID from the dump; `tools/backfill_card_fields.py`
  reproduces). `build.rs`/`generated.rs`: generated CardDef literals carry real
  races (`Race::Naga` added; "ALL" has no single-tribe mapping and stays None)
  and a new `generated::card_set(id)` registry (`CardSet`: Classic/Core/five
  expansions/Other; unknown/custom IDs default to Classic). Vanilla cards keep
  the generated-const baseline; cards with prose keep hand-written effect files
  (`core_w*.rs` pattern). text/class/collectible stay in the JSON layer.
- [x] **Set registration** (M0.3, PR #133) — `cards/cards.json` gains the
  five expansions (848 cards; `tools/merge_expansion_sets.py` reproduces);
  build.rs emits per-set group consts (`EMERALD_DREAM_CARDS` 183 /
  `THE_LOST_CITY_CARDS` 183 / `TIME_TRAVEL_CARDS` 183 / `CATACLYSM_CARDS` 164 /
  `ESCAPEFROM_VIOLET_HOLD_CARDS` 135) and a flat `EXPANSION_CARDS` slice
  (engine-available via `card_by_id`). **D3 two-state**: `ALL_CARDS` keeps the
  handwritten pool (zero churn across call sites); `pool::in_active_window`
  (Classic|Core — the single cut-over point) filters every sampling pool;
  `is_standard`/`is_expansion` filters in place. Surfaced + documented a real
  Core rebalance: Babbling Bookcase (CORE_EDR_001 2/4 vs EDR_001 3/3).
- [ ] **Validation** — ID-uniqueness and generated-vs-handwritten fidelity tests
  (`core_reprints_match_originals` pattern), plus a `differential` gate that
  every expansion card is either hand-written or matches its generated
  baseline.

## Engine mechanism gaps (what the expansions need vs. what exists)

**Exists** (Core Set W0–W8): Rush/Lifesteal/Reborn/Outcast/Tradeable/spell-power
pipeline, hero replacement (`CardType::Hero`), locations, enchantment tokens,
choose-one (simplified: auto-random), discover (simplified: random), secrets,
auras, quest-free trigger inventory.

**Missing — new engine primitives (each lands with F5 scenarios):**

| Primitive | Needed by | Notes |
|---|---|---|
| Hero-power upgrade/transform + per-turn choice | Imbue (EDR), Deathwing hero (CAT) | `HeroPowerDef` exists but is static; needs a per-game upgrade counter, threshold triggers, and a resolvable "choose effect" hero power |
| Quest zone + quest progress component | Un'Goro | Engine has no quest zone; needs `Zone::Quest` (or per-player quest slot), progress events, reward resolution |
| Effect replay / last-effect history | Rewind (TTW) | Track the last played card's effects (and the card before), replay on activation — sequencing with death phases |
| Colossal body parts | Cataclysm | Attached part entities summoned with the minion, die with it, occupy/are summoned adjacent — new summon path |
| Split-card halves (Shatter) | Cataclysm | A drawn card splits into two half-cards that recombine — touches draw, hand, and play resolution |
| Kindred death-tracking | Un'Goro | "Kindred: X" — per-card counter of triggering events (exact trigger set from card text) |
| Smoldering re-deathrattle | EDR mini | Deathrattle fires an extra time on a marked minion |
| Rule overrides (Rulebreakers) | Violet Hold | Per-card rule exceptions (duplicate legendaries, extra draws, …) — audit each against the rules engine's hardcoded invariants |
| Real choose-one resolution | EDR (all classes) | Pay the registered auto-random simplification: choice surfaces in `legal_actions`/Python bindings |

## Sub-roadmap index and dependency order

```
M0 data pipeline (this file)
 ├─ 01 expansion-emerald-dream-roadmap.md (+zh)   — Imbue, Dark Gifts, Choose One, Smoldering
 ├─ 02 expansion-ungoro-roadmap.md      (+zh)     — Quests, Kindred
 ├─ 03 expansion-timeways-roadmap.md    (+zh)     — Rewind, Fabled
 ├─ 04 expansion-cataclysm-roadmap.md   (+zh)     — Colossal, Herald, Shatter, Deathwing
 └─ 05 expansion-violet-hold-roadmap.md (+zh)     — Rulebreakers
```

Order is release order: each expansion introduces its own primitives, so
sub-roadmaps run sequentially (a later set can reuse earlier primitives; the
reverse never happens). Every sub-roadmap: W0 wiring/data → mechanic waves →
closing wave (tokens, legendary finishing touches, ledger sweep).

## Decisions (resolved — all five recorded 2026-08-08 as part of M0)

- **D1 — Card data source** ✅ **hearthstonejson dump**
  (`api.hearthstonejson.com` collectible dump; landed in M0.1, data under
  `cards/data/`, sha256 in `SOURCE.md`).
- **D2 — Simplification registration** ✅ per the text above: complex expansion
  effects (random Discover pools, Shatter recombination, some Dark Gifts) land
  with `(simplified …)` markers and `fidelity-debt.md` rows like the Core
  waves — unless the wave implements them fully.
- **D3 — RL pool policy** ✅ **real Standard pool**: the training pool
  switches to 2025–2026 + Core. The engine supports both states via pool
  filters; the training switch is a single cut-over at the closing stage
  (like D4-2026-08-08), with observation-encoding extensions (quest progress,
  imbue level, location charges already exist) and a `_load_debt_ids()` glob
  extension for the new wave files.
- **D4 — Wave shape** ✅ **per-expansion waves** (mechanics are set-specific
  and data lands per set).
- **D5 — zh naming** ✅ set names/class names backfilled from official
  localization during M0; card names keep English in `classic-cards-zh.md`
  convention.

## Cross-expansion milestones

- **M0** — data pipeline + validation (D1–D5 recorded). No PR without it.
- **M1** — Emerald Dream + mini complete (Imbue / Dark Gifts / real Choose One).
- **M2** — Un'Goro + mini complete (Quest zone, Kindred).
- **M3** — Timeways + mini complete (Rewind).
- **M4** — Cataclysm complete (Colossal / Herald / Shatter / Deathwing).
- **M5** — Violet Hold complete (Rulebreakers); all five sub-roadmaps archived
  with this master file to `docs/finished/`.

## Definition of done

Every `- [ ]` in every sub-roadmap is `- [x]`; each card lands with an F5
differential scenario (or a `(simplified …)` ledger row where a wave chooses
to register debt); D1–D5 are recorded; the RL pool decision is applied; both
language versions of this file and all five sub-roadmaps move to
`docs/finished/`; workspace `CLAUDE.md` is updated.
