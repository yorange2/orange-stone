# Fatigue Roadmap — empty-deck draw damage

> Archived 2026-08-06 — all milestones M1–M3 complete (PRs #92–#94); implementation details live in git history. This roadmap closed the registered fidelity debt **F-A10** (`docs/fidelity-debt.md`): empty-deck draws now deal official fatigue damage (1, 2, 3, …), so deck-draining games end with a real winner and `max_steps` / `max_turns` are demoted to backstops. Chinese mirror: `fatigue-roadmap-zh.md` (in `docs/finished/`).

---

## 1. Why

| Pain point | Current state | With fatigue |
| --- | --- | --- |
| Fidelity gap (F-A10) | `draw_card_no_queue` returns `None` on an empty deck (`trigger.rs` — "fatigue in Phase 3+"); no damage, no game progress | Official HS rule: escalating damage per empty-deck draw attempt |
| Stalled games | A game that drains both decks never ends; env caps it at 5000 steps as a forced draw (`rl/env.rs`), battle sims cap via `max_turns` | Games end naturally with a real winner; caps stay only as backstops |
| RL correctness | An agent with a draw-heavy deck (Warlock life tap, Arcane Intellect) gets a game that never terminates and no signal | Fatigue is a real, observable pressure; deck_count=0 in the observation now has consequences |
| SabberStone parity | Differential tests cannot cover exhausted-deck lines | Fatigue semantics match the reference implementation |

## 2. The official rule (what to implement)

Hearthstone fatigue, exactly:

1. A **draw attempt** on an **empty deck** deals damage to the drawing hero equal to the player's fatigue counter, then the counter increments by 1 (first hit = 1 damage, second = 2, …).
2. Damage is applied **immediately, per draw attempt**. A multi-card draw (Arcane Intellect: draw 2) on an empty deck hits twice: 1 then 2.
3. The damage is **ordinary damage**: hero armor absorbs it, lethal damage ends the game (the fatigued player loses), and lethal-prevention secrets (Ice Block) fire on it.
4. Partial draws: if the deck has `k < N` cards, the `k` cards are drawn normally, and fatigue covers only the remaining `N − k` attempts.
5. **No fatigue** for: race-scan draws (Sense Demons draws what exists, no damage for missing cards) and the opening/mulligan draws (deck is full — physically unreachable).

## 3. Current code map

| Piece | Location | Note |
| --- | --- | --- |
| Draw choke point | `src/engine/trigger.rs:848` `draw_card_no_queue` | Single top-deck draw; returns `None` on empty deck — the documented fatigue hook ("fatigue in Phase 3+") |
| Queued draw wrappers | `src/engine/trigger.rs:857` `draw_card_with_reduction` (Far Sight), `:878` `draw_card` | Turn draw + card effects funnel here |
| Effect draws | `src/engine/trigger.rs:79` `DrawCard` (per-card loop — multi-draw stacks naturally), `:621` `ChanceDraw` (Nat Pagle), `:688` `DrawAndDamageByCost` (Holy Wrath), deathrattle draws, Warlock life tap hero power | All route through the choke point |
| Race-scan draw | `src/engine/trigger.rs:886` `resolve_draw_by_race` | Direct zone scan, no choke point — correct to leave untouched (rule 5) |
| Opening/mulligan draws (no queue) | `src/engine/rules.rs:1223` (mulligan replacement), `:1234` (4th card), `src/core/state.rs:288/291` | Deck provably non-empty; signature change must not force a queue here |
| Damage pipeline | `src/engine/rules.rs:907-985` `Event::DamageDealt` → immune → divine shield → armor → health → death check → `GameOver` (Highest priority) | Fatigue must enter here, not subtract health directly |
| Secret on hero lethal | `src/engine/secret.rs:122` (Ice Block) | Fires off the `DamageDealt` event — free once fatigue uses the pipeline |
| Player state | `src/core/player.rs:44` `Player` | Counter home (has `armor`, `mana_crystals`, …) |
| RL surface | `src/rl/views.rs:83` `PlayerView`, `src/py_bind/views.rs` | Add `fatigue` field; **168-dim tensor unchanged** (M0 baseline convention — same reasoning as rl-interface-roadmap D2-a) |
| Game-end backstops | `src/rl/env.rs:223` `max_steps` limit-draw, `src/sim/battle.rs:444` `max_turns` | Stay, as backstops |

## 4. Design decisions

### D1 — Counter location and semantics
`Player.fatigue: u32`, **1-based**: the counter *is* the damage the next empty-deck draw deals, then `counter += 1`. (Rejected: 0-based counter +1 at damage time — the 1-based form reads directly and matches the HS description.) Serde round-trip comes free with the derived `Serialize/Deserialize`.

### D2 — Hook point: the choke point
Fatigue lives in `draw_card_no_queue` — every draw path (turn draw, card effects, hero powers, deathrattles, Nat Pagle, Holy Wrath) funnels through it, so one change covers them all and no future card can forget fatigue. Signature change:

- `draw_card_no_queue(state, player)` → `draw_card_no_queue(state, queue: &mut EventQueue, player)`; on empty deck: push `Event::DamageDealt { source: hero, target: hero, amount: counter }`, then `counter += 1`, return `None` (no `CardDrawn` — "whenever you draw a card" triggers must NOT fire).
- The three opening/mulligan call sites (rule 5 — deck provably full) switch to a small queue-less helper `draw_top_card_no_queue` carrying a `debug_assert!` on the non-empty invariant, so the hot opening path keeps its no-queue shape.

### D3 — Fatigue damage goes through the unified pipeline
`source = the drawing player's hero entity` (heroes have no poison, so the poison check is inert). Consequences, all correct per HS and free: armor absorbs; lethal → `GameOver` via the existing death check; Ice Block fires on the `DamageDealt` event. Matches SabberStone.

### D4 — Race-scan draws keep no-fatigue semantics
`resolve_draw_by_race` untouched; add a code comment documenting why (rule 5). A "draw N minions of a race" effect that finds 0 draws 0 and deals 0 — HS-correct for Sense Demons.

### D5 — RL surfacing: structured views only
`PlayerView.fatigue: u32` (rl + py_bind views) so Python can read the counter. The 168-dim tensor shape is **not** changed (same reasoning as rl-interface-roadmap D2-a: changing the tensor breaks the M0 baseline convention; the RL side builds features from structured views).

### D6 — Backstops stay
`max_steps` / `max_turns` remain as belt-and-braces (armor/heal loops are the only theoretical way to outlive fatigue). The F-A10 ledger entry is updated to "resolved — fatigue implemented; caps demoted to backstops".

## 5. Milestones

### M1 — Core rule (engine) ✅
- [x] `Player.fatigue: u32` (1-based) in `src/core/player.rs`
- [x] `draw_card_no_queue` takes `&mut EventQueue`; empty deck → `DamageDealt` (source = hero, amount = counter), counter += 1, return `None`
- [x] `draw_top_card_no_queue` helper for the 3 opening/mulligan sites (with non-empty `debug_assert!`)
- [x] Tests (new `tests/fatigue.rs` or `tests/gameplay.rs` additions):
  - first empty-deck draw deals 1, second deals 2
  - multi-draw (draw 2) on empty deck deals 1 + 2 = 3
  - turn-draw fatigue (DrawStep on empty deck)
  - armor absorbs fatigue (hero power armor / Shield Block)
  - fatigue lethal → `GameOver`, correct winner
  - draw with non-empty deck behaves as before (no damage, card moves)
  - opening/mulligan draws unaffected (invariant holds)

**Acceptance**: `cargo test` all green (405 baseline); every behavior above pinned by a named test.

### M2 — Coverage & interactions ✅
- [x] Audit pass over every draw path confirming it funnels through the choke point (turn draw, `DrawCard`, `ChanceDraw`, `DrawAndDamageByCost`, deathrattles, life tap, Far Sight, Battle Rage, Flare); any direct `zones().iter(Zone::Deck)` move not covered gets a decision (likely D4-style "no fatigue" or refactor)
- [x] **F-A10 scenario pinned properly**: both decks empty → game ends with a real winner well before `max_steps` (replaces the old "limit draw" expectation with a fatigue-death test)
- [x] Ice Block: fatigue lethal is prevented, game continues
- [x] Determinism: fatigue uses no RNG — same-seed replay remains byte-identical (existing determinism test must pass unchanged)
- [x] SabberStone differential cases for exhausted-deck games (`tests/differential.rs`, if the parity matrix reaches fatigue lines)
- [x] `cargo bench` sanity: fatigue is a cold path (≤ ~30 hits/game), no hot-path regression

**Acceptance**: full test suite green; a two-empty-deck game terminates with a winner in both `GameEnv` and `battle` paths.

### M3 — RL surface + docs ✅
- [x] `PlayerView.fatigue` in `src/rl/views.rs` + `src/py_bind/views.rs`, binding smoke test (tool side)
- [x] `docs/fidelity-debt.md` F-A10 entry marked resolved (keep the caps-as-backstops note)
- [x] `trigger.rs` comment ("fatigue in Phase 3+") replaced by the real semantics
- [x] Archive this roadmap pair to `docs/finished/` (en + zh) and drop the "active" header

**Acceptance**: Python binding exposes the counter and reads it correctly in a fatigue game; docs consistent in both languages.

## 6. Out of scope

- **Hand-size burn** (full-hand draw discards the card) — a separate pre-existing gap; if not already registered in `fidelity-debt.md`'s mechanism inventory, file it there, not here.
- Fatigue counter in the 168-dim observation tensor (D5).
- New cards or engine rework beyond the choke-point change.
