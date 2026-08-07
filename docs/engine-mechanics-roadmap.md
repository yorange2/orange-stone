# Engine-Mechanics Roadmap — battlecry targets & freeze timing

> Active roadmap (not yet archived). Two engine-wide mechanics were left
> untouched during the W8–W12 fidelity-debt execution (PR #97–#101): both
> predate the §11 ledger, affect whole card classes rather than single
> cards, and therefore get their own milestones with F5 differential
> coverage. Chinese mirror: `engine-mechanics-roadmap-zh.md`.

## Why

| Issue | Current state | Real HS | Impact |
| --- | --- | --- | --- |
| Minion battlecry targets dropped | `Event::MinionSummoned` resolves battlecries with `explicit_target = None` (`rules.rs`) — the `PlayCard { target }` is forwarded only for spells | Targeted battlecries (Houndmaster, Argent Protector, …) pick the chosen target and fizzle when it left the legal set (G9) | Every targeted minion battlecry resolves randomly; the RL agent's `PlayCard { target }` is meaningless for them |
| Freeze never blocks an attack | `Event::TurnStarted` clears Freeze for the incoming player's entities at the **start** of their turn (`rules.rs`) — a character frozen during the opponent's turn is thawed before it could ever attack | A frozen character cannot attack on its next turn; it thaws **after** that missed attack opportunity (start of the following turn) | Frost Nova / Cone of Cold / Water Elemental freezes are cosmetic; the freeze check in `AttackDeclared` is dead code in practice |

## Milestones

### M1 — Minion battlecry explicit targets

- Thread the `PlayCard { target }` through the minion play path: the
  `MinionSummoned` event (or the `CardPlayed` handler) must carry the
  explicit target into `resolve_effect` for the battlecry.
- Re-validation stays G9: an explicit target that left the legal candidate
  set at resolution time fizzles the battlecry (no random fallback).
- Audit the affected cards (Houndmaster — friendly Beast, Argent Protector
  — friendly minion, Kul Tiran Chaplain, Blessing-of-Might-style spells
  already work; list the full minion-battlecry target set in the PR) and
  pin each with an F5 scenario.

**Acceptance**: `m1_*` differential scenarios (explicit-target battlecry
hits the chosen target; invalid target fizzles; RL `legal_actions` targets
are honored); `cargo test` green; clippy clean.

### M2 — Freeze timing

- Move the thaw from `Event::TurnStarted` to the turn-end wrap-up: a
  frozen character keeps Freeze through its owner's next turn (its attack
  is blocked by the existing `AttackDeclared` check), then thaws at the
  start of the following turn — official HS semantics.
- Preserve the frozen-interaction contracts: Icicle deals damage to an
  already-frozen character instead of re-freezing; freeze removal must
  still complete before the next turn's attacks.

**Acceptance**: `m2_*` differential scenarios (a character frozen during
the opponent's turn cannot attack on its next turn and thaws afterwards;
hero freeze blocks the hero's attack; Icicle double-freeze interplay);
`cargo test` green; clippy clean.

## Wave accounting

| Milestone | Scope | PR |
| --- | --- | --- |
| M1 battlecry targets | engine + F5 | one PR |
| M2 freeze timing | engine + F5 | one PR |

## Out of scope

- 168-dim observation tensor changes (D5 of the fatigue roadmap).
- RL-side policy/training changes beyond honoring the battlecry target in
  `legal_actions`.
- Any new `(simplified …)` comments discovered during the audits belong in
  the F4/F5 ledger (`docs/fidelity-debt.md`) per its maintenance contract.
