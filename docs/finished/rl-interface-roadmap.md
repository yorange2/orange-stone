# RL-Interface Roadmap — external RL-facing interface work

> This roadmap tracks orange-stone's **external RL-facing interface work** (Python binding API, batching, card views, and fidelity), driven by the orange-reinforcement integration project (RL-side doc: `orange-reinforcement/docs/finished/roadmap.md`). The engine's internal architecture and fidelity milestones (A–G, F) live in [architecture-roadmap.md](../architecture-roadmap.md); this document is its RL-interface supplement — it closes the G2–G8 gaps raised by the integration project, and carries the M4 batch bindings and the M5 fidelity-debt payoff.
>
> Status verified: 2026-08-05 (M1 baseline; code cross-checked).

---

## 1. Why orange-stone for RL

orange-reinforcement's previous attempt at a real engine was the RosettaStone C++ binding (`rosetta/`), with three showstoppers that orange-stone solves (details in the RL-side doc §1):

| RosettaStone pain point | orange-stone capability |
| --- | --- |
| `Game` not cloneable (copy/move constructors `= delete`) — no whole-turn search | CoW GameState clones cheaply; search/rollback is natural |
| Global static RNG — parallel sampling forced multiprocessing | Per-game `GameRng`; `sim/batch.rs` is rayon thread-parallel with per-game reproducibility |
| AGPL-3.0 (license contamination) | MIT |

## 2. RL-facing surface at M0 baseline

| Existing | Location | Notes |
| --- | --- | --- |
| RL environment | `src/rl/env.rs` | `GameEnv`: single agent vs built-in bot; full legal-action enumeration; `max_steps=5000` anti-infinite-loop |
| Observation encoding | `src/rl/obs.rs` | Fixed **168-dim** (hero/mana/deck-count/hand 10 slots×6/both boards 7 slots×7), normalized. **Basic keywords only** (taunt/divine-shield/windfury/charge/stealth); no battlecry/deathrattle/trigger/card text |
| Reward | `src/rl/reward.rs` | Configurable (win/loss/draw + per-step shaping); default sparse win+1/loss−1/draw 0 |
| Python binding | `src/py_bind/mod.rs` | `GameEnv(seed, perspective=0, deck_size=30)` → `reset` / `observation` / `legal_actions` (`[(index, string description)]`) / `step` (→ `(obs, reward, done, winner)`) |
| Batch simulation | `src/sim/batch.rs` | `BatchSimulator`: rayon parallel bot-vs-bot, per-game RNG reproducible; **bot-driven only, no RL batch step** |
| Built-in bots | `src/sim/battle.rs` | `BotType::Greedy` / `Smart`, only these two |

## 3. Binding gaps (raised by RL integration, G2–G8)

| # | Gap | Status | Impact |
| --- | --- | --- | --- |
| G2 | No custom decks | Deck randomly drawn from full card pool (`deck_size` cards, no duplicates within a deck) | Parity/training need fixed pool ×2 mirrors (simplified-engine convention) |
| G3 | No structured observation/action | Only 168-dim vector + `(index, string description)` | RL feature engineering (251-dim v6) needs structured fields: action type/hand index/attacker/target/trade outcome |
| G4 | Both sides not externally controllable | `GameEnv` fixed "agent vs built-in bot"; bot takes over after EndTurn | arena (bot vs bot), future self-play need a no-bot mode |
| G5 | clone not exposed | CoW cheap on the Rust side, but py_bind has no clone | Whole-turn beam search cannot be rolled back from Python (killed rosetta) |
| G6 | Fixed mulligan rules | Both sides 3 cards; no second-player 4 cards + coin | Misaligned with simplified-engine convention; distorted first-player advantage |
| G7 | Different reward convention | Sparse win/loss +1/−1 | Simplified engine gives "loss 0~−1 scaled by opponent health"; training curves not comparable |
| G8 | No RL batch step | `batch.rs` bot-driven only | Training parallel sampling would need hand-rolled multiprocessing |

(G1 binding install, G9 pool misalignment, and G10 rule-semantics adaptation belong to the RL side — see `orange-reinforcement/docs/finished/roadmap.md` §2.3.)

## 4. Milestones

### M1 — Interface completion (≈1–2 weeks) ✅ Done (2026-08-05)

Closing G2–G8 one by one, each change with a Rust unit test + binding-layer test:

- [x] **Custom decks** (G2): `EnvConfig`/py_bind accept an explicit deck list (fixed pool ×2 mirrors), random mode kept (`GameEnv(seed, deck=[...])`, unknown IDs raise ValueError)
- [x] **Structured Action/Observation views** (G3): PyO3 exports structured fields in the shape of the rosetta bindings (`EntityView/PlayerView/Observation`); string descriptions kept for play.py (`structured_observation()` / `structured_legal_actions()`)
- [x] **Both-sides-controllable mode** (G4): `BotType::None`; the current player becomes externally steppable after EndTurn; shared by arena and training (`GameEnv(seed, bot="none")`)
- [x] **Expose clone()** (G5): py_bind gains `clone()` (`GameEnv: Clone` already derived on the Rust side — just a passthrough)
- [x] **Configurable mulligan** (G6): hand_size, second-player-coin toggle (second player 4 cards + coin; coin not drawn from deck)
- [x] **Parameterized reward convention** (G7): the simplified engine's `final_reward` (win +1 / draw 0 / loss 0~−1 scaled by remaining health) exported as an optional reward config (`terminal_reward="sparse"|"health_scaled"`)
- [x] ~~(Optional) card-text dims in `obs.rs`~~: **not done** — the G3 structured views already carry card_id/name/all keywords; the RL-side v7 features build from structured views per decision D2-a, no need to touch the 168-dim tensor (changing it would break the M0 baseline convention)

**Acceptance**: every item has a Rust test in `tests/`; the Python side has matching smoke tests. **Met**: `cargo test` all 321 pass; RL-side `tools/orange_stone_m1_smoke.py` all six sections pass; M0 smoke regression (deterministic 8 seeds) passes.

**M1 results** (2026-08-05, merged per-PR): 6 PRs **#64** (G2 decks), **#65** (G3 views), **#66** (G4 no-bot), **#67** (G5 clone), **#68** (G6 mulligan), **#69** (G7 reward); the RL side landed 6 matching smoke PRs (each appended a section to `orange_stone_m1_smoke.py`). Two latent bugs fixed along the way: small-deck draw skipped cards (indexed by original deck length), and fixed-deck openings were seed-independent (shuffle/mulligan moved to the runner RNG).

### M4 — Batching and performance (engine part, ≈1 week) ✅ Done (2026-08-06)

Releasing the GIL was the prerequisite for RL-side threaded parallel training:

- [x] **GIL release** (**#71**): hot methods wrapped in `py.allow_threads` — pure-Rust engine + per-game RNG is thread-safe; previously the GIL serialized threads completely
- [x] **Batched observation/action tensorization**: `orange_stone.BatchEnv` (drives N games per call, structured observations from the **current actor's perspective** — batch training drops the RL-side dual-instance lockstep; `reset_one` reopens a single game), `battle_batch` (rayon batching, direct exposure of `sim/batch.rs`)
- [x] Throughput: `battle_batch` ~4,200 games/s (≈9.2× rosetta's 460; the ≥10× target was not fully met but same order of magnitude)

**Acceptance**: RL-side `test_batched.py` — deterministic policy, BatchedEnv matches single-game Env winners across 12 games + `battle_batch` single/batched consistency; benchmark table in the RL-side README.

**Note**: engine batching is at ~4,200 games/s, but training throughput remains capped by Python feature/forward GIL serialization (an RL-side matter; further gains need batching features and the forward pass, or GPU batched inference).

### M5 — Pool expansion and fidelity (engine part, in sync with milestone F) ✅ Done (2026-08-06)

- [x] **Card-view/pool additions** (**#72**/**#73**/**#74**/**#75**): stealth cards (`CardDef.stealth`), Elusive mechanic (`CardDef.elusive` + spell-target enumeration/resolution exclusion), card-text view fields (card_type + effect magnitude), `all_card_ids()`
- [x] **In sync with milestone F**: F1–F5 done internally (`tests/differential.rs`); **external SabberStone comparison passing** (**#75**: dotnet-driven mirrored attack-trade scenarios, both simulators agree — see `docs/differential_sabberstone.md`)
- [x] **Fidelity-debt payoff** (F4/F5 ongoing-audit items) ✅ **all cleared (2026-08-06)**: the 67 simplified-card markers are recorded in `docs/fidelity-debt.md` (audit ledger; empty after the payoff); the execution plan `docs/finished/fidelity-debt-roadmap.md` (8 dependency-ordered waves: W0 wiring 13 → W1 race 11 → W2 triggers 8 → W3 predicates 9 → W4 cost weapons 8 → W5 target structure 7 → W6 special mechanics 8 → W7 wrap-up 3) fully landed (PR #79–#86); a card **leaves the ledger** only when implemented **and** verified by an F5 differential test (`tests/differential.rs` now has 72 `w0_*`–`w7_*` scenarios); the F4/F5 ongoing-audit mechanism stays

**Out-of-ledger fixes along the way**:
- **#77 structural findings**: stale comments, Worgen Infiltrator stealth added, 3 card-ID conflicts, 10 cards added to ALL_CARDS, 7 duplicate entries — ALL_CARDS is now **413 unique entries**
- **Pilfer**: the OtherClass pool filter was "any non-Rogue" (counting neutral cards); narrowed to the class cards of the other 8 classes

**RL-side follow-up**: with the debt cleared, the RL training pool grew to full classic constructed scale, **391 cards**; the `decks.py::_load_debt_ids` pool bug (simplified marker recorded against the previous card's ID) was fixed in RL-side PR #31. Details in `orange-reinforcement/docs/finished/roadmap.md` §3 M5.

## 5. Remaining risks (engine side)

| Risk | Mitigation |
| --- | --- |
| Individual cards may still carry simplifications (F4 ongoing audit) | Training pools only use **cards implemented and passing differential tests**; new simplifications get registered in `docs/fidelity-debt.md` per its maintenance conventions |
| Performance-regression on hot paths (`sim/`, `rl/`) | `cargo bench` required for changes; do not let parity/training throughput regress |

## 6. Related documents

- RL-side integration roadmap: `orange-reinforcement/docs/finished/roadmap.md` (M0/M2/M3/M6, decisions D1–D5, risks)
- Architecture roadmap: `docs/finished/architecture-roadmap.md` (milestone G precedes F; F4/F5 carry the fidelity debt; this doc's M5 records the RL-side landing)
- Fidelity-debt ledger: `docs/fidelity-debt.md` · execution plan (archived): `docs/finished/fidelity-debt-roadmap.md`
- External comparison: `docs/differential_sabberstone.md` (SabberStone differential-verification protocol)
