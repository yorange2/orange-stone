# Architecture Roadmap — Findings & Priorities

> Last updated: 2026-08-05
> Records the findings of the architecture review — Orange Stone vs RosettaStone (C++) and SabberStone (C#) — and the prioritized work items derived from it.
> Review I (2026-08-05): hot-path + scope findings (Milestones A–F). Review II (2026-08-05): fidelity architecture — the sequencing and stat-modifier model vs RS/SB (Milestone G).
> Companion document: [architecture-roadmap-zh.md](architecture-roadmap-zh.md)

## TL;DR

The architecture is the right shape for its goal: typed ECS + generational indices, CoW game state, deterministic event loop, zero-card-code data-driven effects.

**Fidelity policy (2026-08-05): absolute fidelity to Hearthstone is a hard requirement.** Every card effect, sequencing rule, and targeting rule must match real Hearthstone semantics; RL-training ergonomics (determinism, replay, cheap cloning, tensorized I/O) are engineering properties of the same engine, not a license to simplify game rules. The previously "deliberate" simplifications (Overload mana lock, permanent Stealth, Choose One auto-random, single-target destroy hitting all matches, …) are now fidelity debt tracked in Milestone F.

Status: milestones A–D (hot path, damage pipeline, RL prep, scale) are complete (PRs #35–#46). Milestone E is complete (PR #51: E1 allocation churn — the E2/E3 items are subsumed by Milestone G; E4 remains an input-data task). Remaining work: Milestone G (fidelity engine architecture) and Milestone F (card-level fidelity — blocked on G).

**Review II conclusion (2026-08-05):** the flat priority event queue + ad-hoc component scans + direct stat writes cannot express Hearthstone's resolution semantics. RS/SB reach fidelity through a **GameStep state machine** (start-triggers → mana → draw → action → end-triggers → death phase → wrap-up), **registered triggers** with play-order / player-precedence rules, a **death-phase batch model**, and an **enchantment layer** for stat/cost modifiers. Milestone G adds those primitives; Milestone F then audits cards against them.

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

> Status of every item above is annotated in §2 (resolved via PRs #35–#46; the rest are tracked in Milestones E and F).
| P2 | Code comments are Chinese, violating CLAUDE.md's own "English comments" rule | whole repo | Small |

---

## 1. Position: why this architecture differs from RosettaStone / SabberStone

One-sentence comparison: RosettaStone and SabberStone are **fidelity-first simulators** (full target choice, full trigger sequencing, official JSON card data + per-card power scripts); Orange Stone is an **RL training environment first** (deterministic replay, no player choice, data-only cards, cheap state cloning).

### Fidelity policy — absolute fidelity is a hard requirement

**Orange Stone must be absolutely faithful to Hearthstone.** Card effects, trigger/sequencing rules, targeting rules, and resource mechanics must match the real game — the fidelity-first posture of RosettaStone/SabberStone is the correctness bar, and RL ergonomics (determinism, replay, cheap cloning, tensorized I/O) are engineering properties of the same engine rather than a license to simplify rules. Every documented simplification is therefore **debt**, not a trade-off: it is tracked in Milestone F and must be eliminated. The enforcement mechanism is differential validation against SabberStone/RosettaStone outcomes (F5) plus a per-effect fidelity audit (F4).

| Dimension | Orange Stone | RosettaStone | SabberStone |
|-----------|-------------|--------------|-------------|
| Entity model | Typed ECS components (SparseSet, Copy) + generational indices | Flat `GameTag → int` tag map | Class hierarchy + tag dictionary |
| Dangling references | Impossible by construction (`Entity{index, generation}`) | Possible, managed by hand | Possible, managed by hand |
| State model | `Arc` Copy-on-Write, O(1) clone | Single mutable state per game | Single mutable state + GC |
| Card implementation | **Pure data**: `CardDef` consts + closed `CardEffect` enum, zero per-card code | JSON + C++ power classes | JSON + C# power classes |
| Trigger system | Ad-hoc component checks inside `apply_event` | Registered trigger table + chain | TriggerManager + TaskQueue + GameStep machine |
| Resolution model | Flat priority event queue + per-event handler | GameStep state machine (Main/Death/Final steps) | GameStep state machine (BeginStep … FinalWrapUp) |
| Auras | Computed at query time (always consistent) | Aura manager + enchantments | AuraManager + EnchantmentManager |
| Stat/cost modifiers | Direct writes into base components (no enchantment layer) | Enchantment objects on GameTag values | EnchantmentInfo + EnchantmentManager |
| Targeting | Explicit-first `select_target` + random fallback (PR #40) | Full player choice | Full player choice |
| Game setup & choices | No mulligan/coin; deck draws random index | Choice system (Mulligan/General/HeroPower/TaskList) | Choice + ChoiceManager (MULLIGAN/GENERAL/HERO_POWER/TASK_LIST) |
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

- ✅ **RESOLVED (A1, PR #35)** — The event queue was O(n) per operation (`src/core/event.rs:161` `pop_front` used `Vec::remove(0)`; `:149` `push_with_priority` scanned + inserted). Now three per-priority `VecDeque` buckets: O(1) push/pop, stable FIFO within priority preserved.
- ✅ **RESOLVED (A2, PR #36)** — `effective_attack` / `effective_health` / `effective_cost` (`src/core/world.rs`) each iterated every live aura component — O(entities × auras) per query. Now `AuraIndex` buckets active sources by (owner, effect kind), maintained incrementally by the aura/zone/player mutators; queries scan only the relevant buckets, lock-free, `World` stays `Send + Sync`.

### P1 — Design debt

- ✅ **RESOLVED (D1, PR #44)** — CoW was a whole-`Inner` deep copy (`src/core/state.rs:190`): `Arc::make_mut` cloned every sparse set + zone table. Now `SparseSet` is a segmented arena (fixed-size pages, `Arc`-shared page table and pages): clone is O(1), first write copies only the touched page — structural sharing as CLAUDE.md claimed.
- ✅ **RESOLVED (E1, PR #51)** — Allocation churn in event handlers: nearly every `apply_event` branch collected `Vec<Entity>` to satisfy the borrow checker before mutating (`src/engine/rules.rs`, `src/engine/trigger.rs` throughout). Now the hot path is allocation-free: the drain-style collects use `std::mem::take` (reuses the existing buffer), and the scan-then-mutate snapshot lists use `SmallList` (`src/core/small_list.rs`) — a stack-array list with heap spill — covering all event-handler, trigger, secret, and aura target/trigger lists (bounded by board size in practice). `select_target` takes a slice view; attack resets at turn start are restricted to `Zone::Play` (only board characters hold attack state).
- ✅ **RESOLVED (B2, PR #39)** — Secrets rewrote the pending event queue: attack damage + retaliation were pre-queued in `enqueue`, and redirect secrets mutated queued events via `redirect_damage` / `replace_damage` / `redirect_damages` (`src/core/event.rs:192-253`). Attacks now resolve through a single `ResolveAttack` pipeline event with fresh-state retaliation; `redirect_damage` / `replace_damage` deleted, Misdirection/Noble Sacrifice use the uniform `redirect_attack` primitive. ⚠️ One leftover: `redirect_damages` survives for Spellbender (spell-source redirect) — see E2.

### P2 — Scope / fidelity gaps (mostly deliberate)

- ✅ **RESOLVED (C1, PR #40)** — `Action::PlayCard { card }` had no target (`src/core/action.rs:13`); every `EffectTarget` was engine-random. Now `PlayCard { card, target }` with `select_target` (explicit-first, random fallback), threaded through 20+ single-target resolvers — the agent can choose face vs board.
- ❌ **OPEN (E3)** — Choose One auto-random, Combo auto-detected — `src/engine/rules.rs` (choose one via RNG; combo from `cards_played_this_turn`). No player decision surface.
- ❌ **OPEN (F1, F2)** — Documented simplifications, now fidelity debt: Overload is only a trigger marker, not mana lock (`src/core/component.rs`); Stealth is permanent and its "cannot be targeted by single-target effects" claim is not enforced in target resolution (`src/core/component.rs`, `src/engine/rules.rs:229`). Per the fidelity policy these must be implemented, not merely documented.
- ✅ **RESOLVED (C2/C3/C4, PR #42)** — `py_bind/` and `rl/` did not exist; the CLAUDE.md module diagram (Phase 4 target) was ahead of the code. Now `rl/` (env, obs, reward) + PyO3 `GameEnv` (`py` feature, abi3) exist and the wheel is smoke-tested; `sim/battle.rs` + `sim/bot.rs` remain the self-play precursor.
- ⚠️ **PARTIAL (D2, PR #45)** — Card database was ~400 hand-written `const CardDef`s with no mechanical link to official data. The `build.rs` generation pipeline now exists (official-format JSON → static card consts, verified against the hand-written db), but the repo carries only a 4-card sample — the full official DB is not vendored yet (see E4).
- ✅ **RESOLVED (D4, PR #46)** — Code comments were Chinese while CLAUDE.md requires English. Whole-repo translation done (~2,100 comment lines); verified code-byte-identical via a comment-strip diff.

### Fidelity architecture — sequencing & modifier model (review II, 2026-08-05)

Review II compares the engine's *resolution machinery* against RS/SB. The flat event queue + ad-hoc component scans + direct component writes cannot express Hearthstone's sequencing; each item is tracked in Milestone G.

- ✅ **RESOLVED (G1, PR #52)** — No game-step machine. Now a `Step` machine (RS/SB GameStep analogue, `src/core/state.rs`): turn start runs StartTriggers → ManaRefill → DrawStep → Main; turn end runs EndTriggers → WrapUp → opponent's TurnStarted; a Death step is entered when pending deaths surface in Main (G3 upgrades it to marked pending deaths). `TurnStarted` no longer refills mana and draws inline — the refill moved to the ManaRefill step and the draw to DrawStep, so start-of-turn secrets/triggers (`OnFriendlyTurnStart`, `CardDef::start_turn_effect` — now wired) fire *before* the draw; the first player does not draw on turn 1 (DrawStep guard + the initial state never runs it); `TurnEnded` only marks EndTriggers — end-of-turn effects resolve at full strength before the wrap-up cleanup expires temporary buffs; the first player's opening turn now starts with 1 mana crystal.
- ✅ **RESOLVED (G2, PR #53)** — Trigger detection was ad-hoc component scans in sparse-set index order. Now a unified `Trigger` component (`TriggerEvent` + `TriggerTiming`, the RS `ITrigger` / SB `TriggerManager` analogue): fires in play order (zone-table order), current player before opponent, explicit "whenever" vs "after" timing, zone-scoped validity; `start_turn_effect` is wired. The take-damage trigger class is added: Acolyte of Pain is now `ThisMinionDamaged` draw (deathrattle mis-modeling removed), Frothing Berserker / Armorsmith are `FriendlyMinionDamaged` (+1 attack / +1 armor; `GainArmor::FriendlyHero` resolver arm added). The per-class trigger components (`EndTurnEffect`/`StartTurnEffect`/`SpellTrigger`/`DeathTrigger`/`SummonTrigger`/`OverloadTrigger`) are retired; silence/effect-clear go through `remove_trigger`.
- ❌ **OPEN (G3)** — No death-phase batching. `MinionDied` is enqueued per-death at damage time and the handler moves the minion to the graveyard unconditionally (`rules.rs:884-922`), so a minion healed/buffed above 0 before its death is processed cannot survive; death-trigger zone state is wrong (the "dead" minion is still on board when "whenever a minion dies" fires).
- ✅ **RESOLVED (G4, PR #55)** — Buffs wrote directly into base Attack/Health, cost modifiers into base Cost. Now the enchantment layer: base components hold printed values; `Enchantment` (attack/health/cost deltas + `Permanent`/`UntilEndOfTurn` expiry) and an accumulated `Damage` component; effective = base + Σ enchantments + auras − damage. Buffs/debuffs/cost/double effects attach enchantments; auras stay query-time. Silence strips enchantments only (base + damage kept); until-end-of-turn enchantments expire at wrap-up; leaving the battlefield clears enchantments and damage (bounces return at full health); cost enchantments are re-applied by bounce effects (Shadowstep's −2 persists in hand); copy (Faceless semantics) is expressed as an enchantment. `TempAttackDebuff` and `temp_attack_bonus` are retired; `SparseSet` relaxed to `Clone`. ⚠️ Leftover: set-to-value (`SetAttack`/`SetAttackToHealth`) still writes base — deferred to G5's modifier stack.
- ✅ **RESOLVED (G5, PR #56)** — Cost was base component + ad-hoc aura reduction (`effective_play_cost`, `rules.rs:121`, duplicated between validation and mana deduction). Now the single composition is `engine::cost::play_cost` (validation, deduction, bots); `World::effective_cost` is the modifier stack: base + enchantment deltas → set-to-value / floor modifiers (`CostModifier`: Set / Min) → hand-only aura reductions → floored at 0. The ManaRefill step is the F1 overload-lock site.
- ✅ **RESOLVED (G6, PR #57)** — No choice surface beyond `PlayCard { card, target }`; Choose One was engine-random. Now the choice system: `GameEngine::apply_choices` returns `Resolution::{Done, NeedsChoice}` — a pending `PendingChoice` (`ChoiceKind`: ChooseOne / Discover / Mulligan, with option labels and the Discover pool) pauses resolution and the player answers with `Action::Choose { choice_id, option }`; `GameEngine::apply` keeps the default policy (random — RNG-deterministic, bots/self-play unchanged). Choose One (spells and minions — Cenarius, Keeper of the Grove), Discover (`AddRandomCardToHand` pools as options), summon position (`PlayCard.position`, 0 = leftmost), and hero power targets (`HeroPower.target`) are all unlocked; `Event::ChoiceResolved` carries the resolution.
- ✅ **RESOLVED (G7, PR #58)** — No opening flow: `GameState::new` started with an empty deck and draws took a random deck index. Now: `GameBuilder::build` shuffles (Fisher–Yates, seed-deterministic); draws take the **top card** of the ordered deck (random-index draw retired); `GameState::begin_game` deals the starting hands (3 cards to the first player, 4 + The Coin to the second) and surfaces the mulligan as a G6 pending choice (`ChoiceKind::Mulligan` — replaced cards return to the deck, reshuffle, redraw; the Coin is not mulliganable); the first player's turn-1 no-draw landed with G1. New token card `THE_COIN` (`GainManaThisTurn` — +1 mana this turn, no permanent crystal).
- ✅ **RESOLVED (G8, PR #59)** — Secrets matched events post-hoc (`secret.rs:26`): `WhenEnemySpellCast` fired after the spell's effect had already resolved, so counter secrets could not pre-empt. Now play-boundary interception (`secret::intercept_counter_secrets`): Counterspell negates the spell before its effect resolves (this also fixed a latent bug — Counterspell never registered a `Secret` component, so it never triggered at all; `Secret::effect` is now optional); Spellbender summons its 1/3 token first and the spell's single-target effect is redirected to it (AOE spells are unaffected, matching HS). The `redirect_damages` queue mutation is retired (E2).
- ✅ **RESOLVED (G9, PR #60)** — Play targets were only membership-checked against the candidate set at resolve time with a random fallback (`trigger.rs:34`). Now `select_target` re-validates at resolution: an explicit target no longer in the legal candidate set (fresh stealth, target removed, untargetable) makes the effect **fizzle** (no random fallback — HS semantics); with no explicit target, the pick happens at resolution against the current state. Stealth's single-target exclusion (the F2 target side) is enforced: stealthed characters cannot be explicitly targeted — the effect whiffs.

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

### Milestone E — Remaining debt (follow-ups)

- [x] **E1** — Kill the per-event allocation churn (P1): replace the borrow-checker `Vec` collects in `rules.rs`/`trigger.rs` with borrowing splits, small-vector reuse, or arena allocation on the hot path. *(PR #51: `mem::take` for drain sites + `SmallList` stack-buffer lists)*
- [ ] **E2** — Retire the last queue-mutation primitive `redirect_damages` (B2 leftover): route Spellbender through the damage pipeline (e.g. damage-source interception at `DamageDealt`) instead of rewriting pending spell-damage events. *(⏩ subsumed by G8: step-boundary interception replaces queue mutation)*
- [ ] **E3** — Choose One / Combo decision surface (P2): expose the choice in `Action` so the agent decides instead of engine-random / auto-detected combo. *(⏩ superseded by G6: the Choice system covers it)*
- [ ] **E4** — Complete D2: vendor the full official CardDefs JSON and regenerate the static card consts (hand-written effect fields stay on top of the generated statics).

### Milestone G — Fidelity engine architecture (prerequisite for F)

Review II conclusion: these primitives are how RS/SB reach fidelity — the GameStep machine, registered triggers, death-phase batching, and the enchantment layer. F's card-level fixes are only meaningful on top of them; do G before F4/F5.

- [x] **G1** — Game-step machine: model HS resolution steps (start triggers → mana → draw → action → end triggers → death phase → wrap-up) after the RS/SB `GameStep` machine; the priority event queue becomes the event stream *within* a step, and steps are entered from state (pending deaths → death step). Fixes turn-start/end ordering, temp-effect expiry, first-turn draw. *(PR #52: `Step` machine + `advance_step`; the Death step batches at the event level, G3 upgrades it to marking)*
- [x] **G2** — Registered triggers: replace the ad-hoc `iter_*_trigger` scans with per-entity trigger registration (RS `ITrigger` / SB `TriggerManager`): fire in play order, active player first, explicit "whenever" vs "after" timing, zone-scoped validity. Wire `start_turn_effect` (declared but never triggered); add the missing trigger classes (take-damage triggers — unblocks Acolyte of Pain, Frothing Berserker, Armorsmith). *(PR #53: unified `Trigger` component + `fire_triggers`)*
- [x] **G3** — Death-phase batching: pending-death marking (health ≤ 0 = dead but still on board), deaths processed in play order within one step, `MinionDied` re-checks health at processing time (healed/buffed above 0 before its death is processed → survives), death triggers see the minion already removed. *(PR #54: `pending_deaths` + death-step precedence with interrupted-step return)*
- [ ] **G4** — Enchantment layer: stats become base + enchantment deltas (+ damage) instead of direct component writes; buff/debuff/cost effects attach enchantments, auras stay query-time. Enables correct Silence (strips enchantments only), transform, copy (Faceless), and "until end of turn" expiry.
- [x] **G5** — Cost manager: cost = base + modifier stack with HS rules (floor 0, "cannot cost less than", set-to, frozen cost); retire the ad-hoc `effective_play_cost` combination. Overload (F1) locks mana at the mana-refill step. *(PR #56: `engine::cost::play_cost` + `CostModifier` stack; the frozen-cost class and the F1 lock land with F1)*
- [x] **G6** — Choice system: a `Choice` object surfaced in `Action` (SB ChoiceType: Mulligan / General / HeroPower / TaskList) covering Choose One, Discover, opponent choices, summon position, and hero power targets. *(absorbs E3 and F3.)* *(PR #57: `apply_choices` pause protocol + `PendingChoice`/`Action::Choose`; the Mulligan class is used by G7)*
- [x] **G7** — Opening flow: shuffle at game start, starting hands (3/4 + coin), mulligan, ordered deck + top draw (retire random-index draws), first-player no-draw on turn 1. *(PR #58: build-time shuffle + top draw + `begin_game` hands/coin/mulligan choice)*
- [x] **G8** — Secret interception: secrets trigger at step boundaries — counter secrets (Counterspell, Spellbender) before the effect resolves, post-effect secrets keep their trigger point. Retire `redirect_damages` (E2). *(PR #59: play-boundary interception + optional `Secret::effect`)*
- [x] **G9** — Target legality at resolution: re-validate play targets at resolve time per HS rules (Stealth gained, target removed, "cannot be targeted"); exact re-pick/fizzle semantics; enforce Stealth's single-target exclusion (the targeting half of F2). *(PR #60: `select_target` fizzle semantics)*

### Milestone F — Absolute fidelity (hard requirement)

Card-level fixes; the engine primitives they depend on land in Milestone G.

- [ ] **F1** — Overload mana lock: implement real Hearthstone semantics (per-card overload amounts lock the owner's mana on the next turn) instead of the trigger-only marker. *(unblocked by G1/G5: the lock applies at the mana-refill step)*
- [ ] **F2** — Stealth full fidelity: remove Stealth when the character attacks, and exclude stealthed characters from single-target effects (currently only attacks are blocked). *(targeting half lands with G9)*
- [ ] **F3** — Choose One player choice: expose the choice in `Action` (engine-random selection is not faithful). *(⏩ superseded by G6)*
- [ ] **F4** — Fidelity audit: systematic per-`CardEffect` pass against real Hearthstone semantics; fix known divergences (e.g. single-target destroy currently destroys *all* matching enemies; damaged-enemy destroy likewise; verify target sets, damage sequencing, aura stacking). *(blocked by G2–G5: per-effect fixes are only meaningful once the sequencing/enchantment primitives exist)*
- [ ] **F5** — Differential validation: conformance tests comparing game outcomes / event sequences against SabberStone (and/or RosettaStone) as reference implementations, so fidelity regressions are caught mechanically. *(depends on G1–G9: differential runs compare step-level sequencing)*

---

## 4. Reference points

- RosettaStone: <https://github.com/utilForever/RosettaStone> (C++) — power-as-code reference for sequencing fidelity.
- SabberStone: <https://github.com/HearthSim/SabberStone> (C#) — HearthSim reference implementation; our card-effect semantics are informed by it.
- Comparison maintained in this roadmap only; CLAUDE.md stays the design-intent document.
