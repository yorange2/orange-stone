# Architecture Roadmap — Findings & Priorities

> Last updated: 2026-08-05
> Records the findings of the architecture review (2026-08-05) — Orange Stone vs RosettaStone (C++) and SabberStone (C#) — and the prioritized work items derived from it.
> Companion document: [architecture-roadmap-zh.md](architecture-roadmap-zh.md)

## TL;DR

The architecture is the right shape for its goal (RL training): typed ECS + generational indices, CoW game state, deterministic event loop, zero-card-code data-driven effects. Compared with RosettaStone/SabberStone the deliberate trade-offs are justified. The real gaps are two: **P0 hot-path data structures** (O(n) event queue, full aura scans) and **P1 design debt** (whole-state CoW copies, secrets implemented by rewriting pending queue events instead of a unified damage pipeline).

| Priority | Item | Where | Effort |
|----------|------|-------|--------|
| P0 | `EventQueue::pop_front` is `Vec::remove(0)` — O(n) per pop | `src/core/event.rs:161` | Small |
| P0 | Aura resolution scans all auras on every query — O(entities × auras) | `src/core/world.rs:609` | Medium |
| P1 | CoW is a whole-`Inner` deep copy, not structural sharing (CLAUDE.md claim vs reality) | `src/core/state.rs:190` | Medium |
| P1 | Every event handler collects `Vec`s to dodge the borrow checker — allocation churn | `src/engine/rules.rs` (throughout) | Medium |
| P1 | Damage resolution is pre-queued in `enqueue`; secrets rewrite pending events — new redirect cards require per-card special-casing | `src/engine/rules.rs:341`, `src/core/event.rs:192` | Large |
| P2 | No player target choice in `Action` — spell/battlecry targets are engine-random | `src/core/action.rs:13` | Large |
| P2 | `py_bind/` and `rl/` do not exist yet; CLAUDE.md module diagram is stale | — | Large |
| P2 | Card database is ~400 hand-written consts; consider `build.rs` generation from official JSON | `src/cards/` | Medium |
| P2 | Code comments are Chinese, violating CLAUDE.md's own "English comments" rule | whole repo | Small |

---

## 1. Position: why this architecture differs from RosettaStone / SabberStone

One-sentence comparison: RosettaStone and SabberStone are **fidelity-first simulators** (full target choice, full trigger sequencing, official JSON card data + per-card power scripts); Orange Stone is an **RL training environment first** (deterministic replay, no player choice, data-only cards, cheap state cloning).

| Dimension | Orange Stone | RosettaStone | SabberStone |
|-----------|-------------|--------------|-------------|
| Entity model | Typed ECS components (SparseSet, Copy) + generational indices | Flat `GameTag → int` tag map | Class hierarchy + tag dictionary |
| Dangling references | Impossible by construction (`Entity{index, generation}`) | Possible, managed by hand | Possible, managed by hand |
| State model | `Arc` Copy-on-Write, O(1) clone | Single mutable state per game | Single mutable state + GC |
| Card implementation | **Pure data**: `CardDef` consts + closed `CardEffect` enum, zero per-card code | JSON + C++ power classes | JSON + C# power classes |
| Trigger system | Ad-hoc component checks inside `apply_event` | Registered trigger table + chain | TriggerManager + TaskQueue + GameStep machine |
| Auras | Computed at query time (always consistent) | Aura manager + enchantments | AuraManager + EnchantmentManager |
| Targeting | None — engine picks targets randomly | Full player choice | Full player choice |
| Randomness | `GameRng` embedded in `GameState`, reproducible | Per-game RNG, not replay-designed | Seeded `System.Random` |
| RL tooling | Battle runner + bots + card coverage tracker | — | — |

### Strengths to keep

- **Safety model**: generational indices + typed components + CoW turn the classic RS/SB bug classes (dangling entity refs during transforms/return-to-hand/deathrattles) into compile-time errors.
- **Determinism first**: stable-FIFO priority event queue (`event.rs` deliberately rejects `BinaryHeap`), embedded RNG, full event log as replay data — neither RS nor SB reaches this.
- **Zero card code**: cards are static data, so they can be statically validated (unique IDs, pool closure in `pool.rs`), coverage-tracked, and tested.
- **Performance baseline**: no GC, no virtual dispatch, Copy components, SoA sparse sets — theoretical ceiling well above SabberStone, above RosettaStone's tag-map lookups.
- **Testing infrastructure**: `GameBuilder` (bypass rules to construct states), `sim/battle.rs` (automated self-play with `CardTracker`) — unique among the three.

---

## 2. Known issues

### P0 — Hot-path data structures

- **Event queue is O(n) per operation** — `src/core/event.rs:161` (`pop_front` uses `Vec::remove(0)`) and `:149` (`push_with_priority` scans + inserts). The event loop is the absolute hot path of the engine; replace with `VecDeque` or a head-indexed implementation (with the same stable-FIFO-within-priority semantics).
- **Aura resolution scans all auras on every query** — `effective_attack` / `effective_health` / `effective_cost` (`src/core/world.rs:609, 640, 672`) each iterate every live aura component. O(entities × auras) per query on the hot path (attack validation, damage resolution, play validation). Correct by construction, but should be indexed (e.g. grouped by owner player + target type) once self-play volume grows.

### P1 — Design debt

- **CoW is a whole-state deep copy** — `Arc::make_mut` clones the entire `Inner` (all sparse sets + zone tables) on first write after a shared clone (`src/core/state.rs:190`). CLAUDE.md claims "shared unchanged parts" (persistent data structure); that is not yet true. Fine at current entity counts; this is the first thing to upgrade for MCTS-style branching.
- **Allocation churn in event handlers** — nearly every `apply_event` branch collects `Vec<Entity>` to satisfy the borrow checker before mutating (`src/engine/rules.rs` throughout). Deterministic and correct, but allocates per event.
- **Secrets rewrite the pending event queue** — attack resolution (attacker damage + retaliation) is pre-queued in `enqueue` (`src/engine/rules.rs:341-369`), and redirect-type secrets (Misdirection, Noble Sacrifice, Spellbender, …) mutate queued events via `EventQueue::redirect_damage` / `replace_damage` / `redirect_damages` (`src/core/event.rs:192-253`). This is the least elegant corner of the engine: each new redirect-type card requires special-casing in `src/engine/secret.rs` instead of plugging into a uniform damage-resolution pipeline.

### P2 — Scope / fidelity gaps (mostly deliberate)

- **No player target choice** — `Action::PlayCard { card }` has no target parameter (`src/core/action.rs:13`); every `EffectTarget` is resolved randomly by the engine. Fine for random self-play, but the agent can never learn to choose *what* to hit (face vs board). Needed before the RL environment is faithful.
- **Choose One auto-random, Combo auto-detected** — `src/engine/rules.rs:570-586` (choose one via RNG) and `:536` (combo from `cards_played_this_turn`). No player decision surface.
- **Documented simplifications** — Overload is only a trigger marker, not mana lock (`src/core/component.rs:380`); Stealth is permanent (`src/core/component.rs:365`). These must stay documented so validation against real Hearthstone doesn't trip on them.
- **`py_bind/` and `rl/` don't exist** — the CLAUDE.md module diagram (Phase 4 target) is ahead of the code. The current `sim/battle.rs` + `sim/bot.rs` are the RL-testing precursor.
- **Card database is hand-written** — ~400 `const CardDef`s across `src/cards/`. Type-checked and fast, but every new set is manual labor and has no mechanical link to official card data.
- **Comment language drift** — code comments are Chinese while CLAUDE.md requires English comments.

---

## 3. Roadmap

### Milestone A — Hot path (small, do first)

- [x] **A1** — `EventQueue`: replace `Vec::remove(0)` / insert with an O(1) pop (head index or `VecDeque`), preserving stable FIFO within priority. *(PR #35: per-priority `VecDeque` buckets)*
- [x] **A2** — Aura indexing: group active auras by (owner, target class) so `effective_attack/health/cost` do not rescan the whole set. *(PR #36: `AuraIndex` bucketed by (owner, effect kind), incrementally maintained)*
- [x] **A3** — Add criterion benchmarks for the event loop and effect resolution to make A1/A2 measurable. *(PR #37: `benches/event_queue.rs` + `benches/effect_resolution.rs`)*

### Milestone B — Damage pipeline unification (design work)

- [x] **B1** — Converge damage resolution into a single pipeline: immune → divine shield → armor → health → death check (`DamageDealt` handler in `rules.rs` + retaliation already partly there). *(PR #39: extracted `queue_death_events`; attacks flow through the pipeline via `ResolveAttack`)*
- [x] **B2** — Attach secret/reactive effects at pipeline points instead of mutating already-queued events; retire `redirect_damage` / `replace_damage` / `redirect_damages` special cases (`src/core/event.rs:192-253`). *(PR #39: unified `redirect_attack` primitive; `redirect_damage`/`replace_damage` deleted; `redirect_damages` kept as the spell-source primitive)*
- [x] **B3** — Re-verify all secret cards (Misdirection, Noble Sacrifice, Explosive Trap, Spellbender, Vaporize, …) against the unified pipeline with the existing battle-runner coverage. *(PR #39: all secret tests green + 3 new pipeline tests)*

### Milestone C — RL environment prep (Phase 4 in CLAUDE.md)

- [x] **C1** — Extend `Action::PlayCard` with an optional target (engine falls back to random when absent), unblocking faithful decision spaces. *(PR #40: `select_target` explicit-first, threaded through 20+ single-target resolvers)*
- [x] **C2** — PyO3 bindings (`py_bind/`) + gym-like environment (`rl/env.rs`). *(PR #42: `GameEnv` reset/step/legal_actions; `py`-feature-gated `GameEnv` class, wheel smoke-tested via maturin)*
- [x] **C3** — Observation space tensorization (`rl/obs.rs`). *(PR #42: fixed 168-dim observation)*
- [x] **C4** — Reward function configuration (`rl/reward.rs`). *(PR #42: sparse win/loss + dense shaping components)*
- [x] **C5** — Batch simulation API over many `GameState`s (rayon). *(PR #41: `BatchSimulator`)*

### Milestone D — Scale

- [x] **D1** — Upgrade CoW from whole-`Inner` clone to structural sharing (segmented arena / persistent vectors) once entity counts justify it. *(PR #44: `SparseSet` segmented pages with `Arc` copy-on-write; clone is O(1))*
- [x] **D2** — `build.rs` generation of `CardDef` consts from official CardDefs JSON (keeps "cards as data" while removing manual labor). *(PR #45: official-format JSON → generated static card consts, verified field-by-field against the hand-written db)*
- [x] **D3** — bincode/rkyv serialization of `GameState` for distributed training. *(PR #43: serde + bincode `to_bytes`/`from_bytes`; `&'static str` card ids re-interned via the static db)*
- [x] **D4** — Restore comment-language discipline (English) to match CLAUDE.md. *(PR #46: whole-repo comment translation)*

---

## 4. Reference points

- RosettaStone: <https://github.com/utilForever/RosettaStone> (C++) — power-as-code reference for sequencing fidelity.
- SabberStone: <https://github.com/HearthSim/SabberStone> (C#) — HearthSim reference implementation; our card-effect semantics are informed by it.
- Comparison maintained in this roadmap only; CLAUDE.md stays the design-intent document.
