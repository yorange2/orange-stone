# Differential validation against SabberStone (roadmap F5)

The machine-checkable half of the differential contract lives in
`tests/differential.rs`: each scenario encodes a game state, an action
sequence, and the exact expected outcome, derived from SabberStone (and
RosettaStone) resolution semantics.

This document describes the other half: running the same scenarios through
SabberStone and diffing the outcomes. It requires a .NET SDK and a
SabberStone checkout; run it in CI or on a machine with .NET available.

## Protocol

1. **Export a scenario.** Each scenario in `tests/differential.rs` is
   reproducible from (a) the initial state, (b) the action sequence, and
   (c) the RNG seed. Build a driver binary that, given a scenario id,
   prints a JSON transcript:

   ```json
   {
     "scenario": "scenario_attack_trade_event_sequence",
     "seed": 12345,
     "events": [
       {"event": "attack_declared", "attacker": "P1_0", "defender": "P2_0"},
       {"event": "damage_dealt", "target": "P2_0", "amount": 4},
       {"event": "minion_died", "minion": "P2_0"}
     ],
     "final": {
       "graveyard": ["P2_0"],
       "health": {"P1_0": 3}
     }
   }
   ```

   The transcript is the canonical, reference-agnostic outcome: event
   sequence + final state. Two simulators are "in agreement" iff their
   transcripts for the same (state, actions, seed) are identical.

2. **Run SabberStone.** In a SabberStone checkout (C#), mirror each
   scenario: build the same board with the same cards, apply the same
   action sequence, and emit the same JSON transcript format (a small
   `TranscriptWriter` helper — ~100 lines).

3. **Diff.** Compare the two transcripts field by field. Any divergence is
   a fidelity regression on the Orange Stone side (or a documentation
   error in the golden expectation) — file it against the failing
   scenario.

## Scope

The reference scenarios deliberately stress the resolution primitives that
fidelity depends on:

- turn-start/turn-end step ordering (SabberStone `MAIN_START_TRIGGERS` /
  `MAIN_END_TRIGGERS` / `MAIN_CLEANUP`)
- play-order trigger firing and "whenever" vs "after" timing
- death-phase batching and heal rescue
- enchantment expiry and silence
- counter-secret interception before the spell effect
- choice/choose-one resolution (SabberStone `ChoiceManager`)
- overload mana locking
- target re-validation (fizzle semantics)

When extending the harness, add the scenario to `tests/differential.rs`
first (the golden expectation), then mirror it in the SabberStone driver.
