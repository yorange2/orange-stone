# Battlecry-Target Debt Roadmap — clearing ledger §12 (20 cards)

> Active roadmap (not yet archived). The M1 audit of the engine-mechanics
> roadmap (PR #102) walked every minion battlecry with a target-bearing
> effect and found 20 cards whose `EffectTarget` does not match real HS.
> They are registered in the F4/F5 ledger `docs/fidelity-debt.md` §12, each
> with a `(simplified: …)` comment — the RL pool dropped 391 → 371. This
> roadmap clears them, wave by wave. Chinese mirror:
> `battlecry-target-debt-roadmap-zh.md`.

## Why

The engine-mechanics M1 PR #102 made the *mechanism* work — `PlayCard
{ target }` now threads into minion battlecries and G9 fizzle is honored —
but 20 cards still mis-target. The audit grouped them:

- **Wrong target scope (11)** — the engine can target what HS cannot, or
  the wrong side: Cruel Taskmaster, Ironforge Rifleman, Stormpike
  Commando, Elven Archer, Fire Elemental, SI:7 Agent, Alexstrasza,
  Ironbeak Owl, Spellbreaker, Big Game Hunter, Darkscale Healer.
- **Targeted battlecries modeled as `Self_` (7)** — the chosen target is
  dropped, the effect always hits the minion itself: Temple Enforcer,
  Abusive Sergeant, Dark Iron Dwarf, Youthful/Ancient Brewmaster, Earthen
  Ring Farseer, Voodoo Doctor.
- **Effect-shape debts (2)**: Mad Bomber (`DealDamage AllCharacters` is a
  no-op in `resolve_deal_damage`), Frostwolf Warlord (fixed +1/+1
  self-buff instead of per-other-friendly-minion).

## Principles (same contract as the W0–W12 roadmap)

1. **Primitives before cards**: a wave ships only when the mechanism it
   needs exists and is pinned by an F5 scenario (the M1/G9 mechanism is
   already in place — these waves mostly re-target or add `EffectTarget`
   variants).
2. **One differential scenario per card** (`tests/differential.rs`):
   target sets (both sides where HS allows), G9 fizzle, death-phase
   interplay; SabberStone parity where mirrorable
   (`docs/differential_sabberstone.md`).
3. **Pool flows through the ledger** (fidelity-debt.md Maintenance): when a
   card is fixed, remove its §12 row and its `(simplified: …)` comment,
   then invalidate `~/.cache/orange_stone_debt_ids.txt` — the RL pool
   re-grows automatically. The pool must be 391 again when §12 is empty.
4. One PR per wave; each wave keeps `cargo test` green, fmt/clippy clean,
   benches noise-level.

## Mechanism inventory (as of 2026-08-07)

**Already exists** (verified in code — M1 threading, `select_target` G9
fizzle, resolvers with `explicit`):

| Mechanism | Where | Used by |
| --- | --- | --- |
| `PlayCard { target }` → battlecry (M1) | `Event::MinionSummoned` | all waves |
| G9 fizzle (explicit target left the legal set → no random fallback) | `trigger.rs::select_target` | all waves |
| `EffectTarget::FriendlyMinion` (targeted) | `env.rs::candidates_for_target`, resolvers | Temple Enforcer, Abusive Sergeant, Dark Iron Dwarf, Brewmasters |
| `EffectTarget::AnyMinion` (targeted, both sides) | M1 | Ironbeak Owl, Spellbreaker |
| `EffectTarget::EnemyMinionAttackLE` (enemy-scoped) | `resolve_destroy_minion` | Big Game Hunter (re-target) |
| `GainStatsThisTurn` (until-end-of-turn enchantment) | `trigger.rs::resolve_gain_stats_this_turn` | Abusive Sergeant, Dark Iron Dwarf |
| `GainStatsPerHandCard` (counting pattern) | `trigger.rs` | Frostwolf Warlord (per-friendly-minion variant) |
| `ReturnToHand` targeted resolver | `trigger.rs::resolve_return_to_hand` | Brewmasters (re-target to `FriendlyMinion`) |
| `GameRng` per-game determinism | `sim::rng` | Mad Bomber random pings |

**Missing / to add** (per-group prerequisites):

- `EffectTarget::AnyCharacter` — any character on either side (heroes +
  minions) for targeted single-pick effects: Stormpike Commando, Elven
  Archer, Fire Elemental, SI:7 Agent, Earthen Ring Farseer, Voodoo Doctor.
- `EffectTarget::AnyHero` — either hero (Alexstrasza).
- Friendly-character scope for Darkscale Healer (hero included).
- `resolve_restore_health` takes no `explicit_target` today — it must, for
  Earthen Ring Farseer / Voodoo Doctor.
- Mad Bomber's battlecry: three random 1-damage pings (a new effect
  variant or a card-id special case in the damage resolver).
- `GainStatsPerFriendlyMinion` (Frostwolf Warlord).
- `validate_play_card` / `legal_actions`: no change expected — targets are
  re-validated at resolution (G9); the legal side only needs the new
  `candidates_for_target` arms.

## Waves

### W13 — AnyCharacter / AnyHero target mechanism + first re-targets ✅ done (PR #104)

- Add `EffectTarget::AnyCharacter` and `EffectTarget::AnyHero`; wire
  `candidates_for_target` (legal side) and the resolvers (resolution side,
  G9 included); `play_targets` picks them up automatically.
- Re-target the AnyEnemy → AnyCharacter cards: **Stormpike Commando,
  Elven Archer, Fire Elemental, SI:7 Agent** (combo path included).
- F5 scenarios per card (`w13_*`): friendly-side targets legal, G9 fizzle
  for a target that left the set.
- **Acceptance**: `cargo test` green incl. `w13_*`; clippy clean; ledger
  drops 4 rows; pool 375.

### W14 — enemy-scope corrections ✅ done (PR #105)

- **Ironforge Rifleman**: `AnyEnemy` → `AnyEnemyMinion`.
- **Ironbeak Owl / Spellbreaker**: `AnyEnemyMinion` → `AnyMinion` (silence
  a friendly minion must be legal).
- **Big Game Hunter**: `AnyMinionAttackGE` → enemy-scoped destroy (the
  `EnemyMinionAttackLE` pattern, `≥7` kept).
- **Alexstrasza**: `AnyEnemy` → `AnyHero` (SetAttack on heroes only).
- **Darkscale Healer**: friendly minions → friendly characters incl. the
  hero (heal all friendly characters).
- **Cruel Taskmaster**: `AnyEnemyMinion` → `FriendlyMinion` (damage a
  friendly minion, give it +2 Attack).
- F5 scenarios per card (`w14_*`).
- **Acceptance**: `cargo test` green; ledger drops 7 rows; pool 368.

### W15 — targeted battlecries modeled as Self_ ✅ done (PR #106)

- **Temple Enforcer**: `Self_` → `FriendlyMinion` +3/+3.
- **Abusive Sergeant / Dark Iron Dwarf**: `Self_` → `FriendlyMinion`
  `GainStatsThisTurn` +2 Attack.
- **Youthful / Ancient Brewmaster**: `Self_` → `FriendlyMinion` (return a
  friendly minion to hand — the source no longer returns itself).
- **Earthen Ring Farseer / Voodoo Doctor**: `Self_` → `AnyCharacter`
  restore — requires `resolve_restore_health` to take `explicit_target`
  (W13's `AnyCharacter` re-used).
- F5 scenarios per card (`w15_*`): chosen friendly target buffed/healed/
  bounced; G9 fizzle.
- **Acceptance**: `cargo test` green; ledger drops 7 rows; pool 361.

### W16 — effect-shape debts + pool close-out (one PR)

- **Mad Bomber**: replace the no-op `DealDamage AllCharacters` battlecry
  with three random 1-damage pings across all *other* characters
  (per-game RNG; the source excluded).
- **Frostwolf Warlord**: `GainStats Self_` → +1/+1 for each other friendly
  minion (`GainStatsPerFriendlyMinion`).
- F5 scenarios per card (`w16_*`).
- Close-out: update the stale pool-formula comment in
  `orange-reinforcement/hearthstone_os/decks.py` (cross-repo docs commit —
  it still says "24 债 / 368" from the pre-W8 era; reads 0 / 391 after this
  wave); verify `_load_debt_ids()` returns the empty set with the cache
  invalidated; archive this roadmap to `docs/finished/` (en + zh) and
  update the workspace-root CLAUDE.md pointer.
- **Acceptance**: `cargo test` green; §12 empty; RL pool 391; extractor
  cache re-run clean.

## Wave accounting

| Wave | Scope | Cards | PR |
| --- | --- | --- | --- |
| W13 AnyCharacter/AnyHero mechanism | engine + F5 | 4 | PR #104 ✅ |
| W14 enemy-scope corrections | engine + F5 | 7 | PR #105 ✅ |
| W15 Self_ targeted battlecries | engine + F5 | 7 | PR #106 ✅ |
| W16 effect shapes + close-out | engine + F5 + docs | 2 | one PR (+ cross-repo docs) |

## Out of scope

- Anything beyond the §12 card list (new debts found along the way go to
  the ledger per its maintenance contract).
- RL-side policy/training changes — the pool re-grows automatically via
  the extractor.
- 168-dim observation tensor changes.
