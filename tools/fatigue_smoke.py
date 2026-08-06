#!/usr/bin/env python3
"""Binding smoke test — the Python binding exposes the fatigue counter, and a
deck-draining game ends with a real winner (fatigue roadmap M3 acceptance).

A tiny deck (deck_size=1) is emptied by the opening deal, so every turn draw
fatigues: 1, 2, 3, … damage per attempt. P1's turn-1 draw is consumed at
construction (opening semantics — no fatigue), so P2's DrawStep fatigues
first and P2 dies first; the survivor (P1) ends with a fatigue counter of 8.

Run from the repo root with the RL venv python:
    orange-reinforcement/.venv/bin/python tools/fatigue_smoke.py
"""

import orange_stone


def main():
    env = orange_stone.GameEnv(
        seed=7, perspective=0, deck_size=1, deck=None, bot="none"
    )
    obs = env.structured_observation()
    assert obs.me.fatigue == 1, (
        f"initial fatigue counter should be 1 (1-based), got {obs.me.fatigue}"
    )

    steps = 0
    while not obs.done and steps < 100:
        actions = env.structured_legal_actions()
        idx = next((a.index for a in actions if a.kind == "end_turn"), None)
        assert idx is not None, "EndTurn should always be legal"
        env.step(idx)
        obs = env.structured_observation()
        steps += 1

    assert obs.done, "the deck-draining game must terminate (fatigue)"
    assert obs.winner != 0, "there must be a real winner, not a limit draw"
    # The second player (P2) died first — the survivor (P1) took 8 hits
    assert obs.me.fatigue >= 8, (
        f"the survivor's fatigue counter should have climbed, got {obs.me.fatigue}"
    )
    print(
        f"OK: fatigue binding smoke — {steps} steps, winner={obs.winner}, "
        f"my_fatigue={obs.me.fatigue}"
    )


if __name__ == "__main__":
    main()
