# Pool-Open Cards Roadmap — implementing the 4 skipped copy cards (marked)

> Status: **active** (created 2026-08-07). Chinese mirror: `pool-open-cards-roadmap-zh.md`.
> Scope: the 4 cards `docs/classic-cards.md` marks ⏸️ — Mind Vision, Thoughtsteal,
> Mindgames, Lorewalker Cho. They were removed from the classic-cards roadmap
> under the [Pool Closure](finished/classic-cards-roadmap.md#pool-closure) rule.
> This roadmap implements them faithfully **and introduces a permanent marker**
> so the pool-closure contract stays auditable once other sets exist.

## Why they were skipped, and why the reason no longer blocks us

The classic-cards roadmap required the card pool to be **closed**: no implemented
card may generate cards from outside the Classic set. Every random generator in
the engine satisfies this by construction — `cards/pool.rs` samples filtered
subsets of `ALL_CARDS` plus fixed token pools.

These 4 cards cannot satisfy it *by construction*, because they do not sample a
pool at all — they read the **opponent's actual cards**:

| Card | Class | Cost | Text | Reads |
| --- | --- | --- | --- | --- |
| Mind Vision | Priest | 1 | Put a copy of a random card in your opponent's hand into your hand. | enemy hand |
| Thoughtsteal | Priest | 3 | Copy 2 cards from your opponent's deck and put them into your hand. | enemy deck |
| Mindgames | Priest | 4 | Put a copy of a random minion from your opponent's deck into the battlefield. | enemy deck |
| Lorewalker Cho | Neutral Legendary | 2 | Whenever a player casts a spell, put a copy into the other player's hand. | the cast spell |

**Today there is no leak**: `ALL_CARDS` is Classic-only, so anything in the
opponent's hand/deck is already in the pool. The risk is *future*: the day a
second set is supported, these four are the only cards that can move a card
across a pool boundary. That is exactly what the marker is for — the closure
invariant becomes a checked property with a known, enumerated exception list,
instead of an unwritten assumption.

**Non-goal:** supporting multiple sets. This roadmap only makes the exception
visible and machine-readable.

---

## The marker

### Contract

A card is **pool-open** if resolving it can move a card into a zone without that
card having been sampled from a pool `cards/pool.rs` controls.

Three parts, one source of truth:

1. **Registry (canonical, machine-readable)** — `sets::POOL_OPEN_CARDS: &[&str]`,
   a card-ID list next to `ALL_CARDS`.
2. **Code comment (human)** — each card's doc block carries `(pool-open: reads
   the opponent's deck)` etc. Distinct keyword from the fidelity ledger's
   `(simplified: …)` on purpose: these cards are **not** debt, they are faithful;
   the Python debt extractor keys on "simplified" and must not pick them up.
3. **Docs** — a new permanent registry `docs/pool-openness.md` (+ `-zh`), and a
   new 🔓 legend row in `classic-cards.md` / `-zh.md` replacing their ⏸️ status.

### Enforced invariant (M0 test)

The four zone-reading effect variants added in M1 may appear **only** on cards
listed in `POOL_OPEN_CARDS`, and every registry ID must resolve via
`card_by_id`. A future card that reads an opponent zone therefore cannot be
added without either registering it or failing `cargo test`.

### Rejected alternative

A `CardDef.pool_open: bool` field. `CardDef` is constructed with ~415 exhaustive
struct literals and has no `Default`, so one field costs an edit in every card
literal for zero extra expressiveness over an ID registry.

### D1 (decided) — the RL training pool keeps these cards

`hearthstone_os/decks.py::full_pool()` currently drops the coin, derivative
tokens (`id` ends with `t`) and fidelity-debt IDs → 392 cards.

**Decided: keep them (pool 392 → 396), with the marker exposed so the choice is
one flag away.** The pool is closed today, so inclusion is sound; the cards are
faithful, so excluding them would only cost training diversity. M0 ships
`GameEnv.pool_open_card_ids()` and `full_pool(include_pool_open: bool = True)`;
flipping the default is a one-line change if a second set ever lands.

The flag is not decoration — it is the escape hatch that makes D1 reversible.
Whoever adds a second set flips the default to `False` in the same change,
and the pool drops back to a closed 392 without touching the engine.

---

## Milestones

One PR per milestone. Each keeps `cargo test` green, `cargo fmt` / `cargo clippy`
clean, and `cargo bench` at noise level. F5 differential scenarios go in
`tests/differential.rs` prefixed `po_*` (following the `w0_*` convention).

### M0 — Marking mechanism (no card behaviour yet)

- [x] `sets::POOL_OPEN_CARDS` registry (empty at first, filled by M2/M3).
- [x] `docs/pool-openness.md` + `-zh.md`: the contract above, the card table, and
      a maintenance section mirroring `fidelity-debt.md`'s (adding a pool-open
      card requires a registry row + comment + doc row on the same change).
- [x] Cross-reference line in `fidelity-debt.md` / `-zh.md` Maintenance:
      pool-open ≠ simplified; do not use the word "simplified" for these.
- [x] `py_bind`: `GameEnv.pool_open_card_ids()` static method (mirrors
      `all_card_ids()`).
- [x] Rust test `pool_open_registry_is_well_formed`: every ID resolves; no ID is
      a token; registry has no duplicates.
- [x] RL side (separate PR in orange-reinforcement): `full_pool(include_pool_open=True)`
      + a test that the flag changes the pool size by exactly `len(registry)`.

### M1 — Engine primitives

- [x] **Effect variants** (`core/effect.rs`) — note each needs an arm in *both*
      `CardEffect` and the serde mirror `CardEffectDe`, plus the `From` impl:
      - `CopyRandomEnemyHandCard { count }` — Mind Vision.
      - `CopyRandomEnemyDeckCards { count }` — Thoughtsteal; sampling **without
        replacement** over deck entities (two copies of the same card are two
        distinct entities and may both be picked; the same entity may not).
      - `SummonRandomEnemyDeckMinion { fallback_card_id }` — Mindgames.
      - `CopyCastSpellToOtherPlayerHand` — Lorewalker Cho.
- [x] **Copy helper** (`engine/trigger.rs`): `copy_card_to_hand(state, src_entity,
      to_player)` — resolves `card_id(src)` through `card_by_id` and reuses
      `add_card_to_hand`. Copies the **base card definition**, not in-zone
      enchantments (documented nuance; matches Classic-era behaviour and keeps
      copies indistinguishable from freshly generated cards).
- [x] **Global spell trigger**: `TriggerEvent::AnySpellCast` + a global arm in
      `rules.rs::trigger_applies` (next to `SecretPlayed` / `MinionDied`), and
      `Event::SpellCast` must thread the spell entity as `subject` —
      `fire_triggers(..., player, Some(spell), None)`. Existing
      `FriendlySpellCast` triggers carry no `race` / `max_attack`, so passing a
      subject where `None` was passed before is behaviour-neutral; pin that with
      a regression scenario.
- [x] **Registration hook**: Cho registers its trigger by card ID in
      `cards::apply_card_keywords` (the existing "special keywords by ID" hook —
      same reason as M0's rejected alternative: no new `CardDef` field).
- [x] Determinism: every pick goes through `state.rng_mut()`; add a replay test
      (same seed + same actions → identical copies).

### M2 — The three Priest spells

- [x] `PRIEST_024` **Mind Vision** (1 mana). Empty enemy hand → no card, spell
      still consumed.
- [x] `PRIEST_025` **Thoughtsteal** (3 mana). Deck with 1 card → 1 copy; empty
      deck → 0 copies; no fatigue (nothing is drawn).
- [x] `PRIEST_026` **Mindgames** (4 mana) + `PRIEST_026t` **Shadow of Nothing**
      (0/1 minion token, `t` suffix keeps it out of the RL pool). Real HS: the
      opponent's deck is **not** modified; a deck with no minions summons Shadow
      of Nothing; a full board (7) summons nothing — `resolve_summon`'s board cap
      already handles this.
- [x] Registry + `(pool-open: …)` comments + `docs/pool-openness.md` rows.
- [x] F5 scenarios `po_mind_vision_*`, `po_thoughtsteal_*`, `po_mindgames_*`
      (one per edge case above).

### M3 — Lorewalker Cho

- [ ] `LEGENDARY_024` **Lorewalker Cho** (2 mana 0/4 neutral legendary).
- [ ] Direction rule: the copy goes to the **caster's opponent**, not to Cho's
      owner. Resolution reads the subject spell's owner
      (`world.player(subject)`) and hands the copy to `.opponent()` — so Cho
      feeds the enemy when its own controller casts.
- [ ] Scenarios: (a) Cho's owner casts → enemy gains the copy; (b) enemy casts →
      Cho's owner gains it; (c) two Chos (one per side) → each fires once,
      copies do not chain (a copy put into hand is not a cast); (d) a spell that
      kills Cho still copies iff Cho was alive when the trigger fired — pin
      whichever way the SabberStone reference resolves it and document it;
      (e) Cho silenced → no copies.
- [ ] Registry + comment + doc rows.

### M4 — Pool, docs and RL sync

- [ ] `classic-cards.md` / `-zh.md`: 4 rows ⏸️ → ✅ 🔓, legend gains 🔓, the
      per-class table and totals updated (Priest 24 → 27 implemented, Neutral
      Legendary 23 → 24, total 371 → 375 implemented / 4 → 0 skipped).
- [ ] `ALL_CARDS` 415 → 420 IDs (4 collectible + 1 token); `battle_1000.rs`
      prints the new count.
- [ ] Rebuild the wheel with the venv interpreter
      (`maturin build --release --interpreter …/.venv/bin/python`), reinstall,
      re-measure `full_pool()` → expect **396**.
- [ ] `tools/orange_stone_m5_smoke.py` pool stress pass (every card playable).
- [ ] Throughput check: `cargo bench` + the ~970 games/s baseline. Cho's card
      generation lengthens games; report the delta rather than assuming none.
- [ ] Register this roadmap in the workspace `CLAUDE.md` and, on completion,
      move both files to `docs/finished/`.
- [ ] **No retrain required.** 392 → 396 is a 1% pool change with no engine
      semantics change for existing cards, unlike PR #108. If a retrain is run
      anyway, it must use the same `--pool full` 30k × 3-seed protocol so the
      numbers stay comparable.

### M5 — Hand cap and burn (D2: decided, in scope)

The engine has **no 10-card hand limit and no burn** (`add_card_to_hand` and
`draw_card_no_queue` append unconditionally). This is a pre-existing gap the
fatigue roadmap explicitly deferred to the ledger, and it is not currently
registered anywhere.

Cho and Thoughtsteal make overfull hands routine rather than rare, and
`rl/obs.rs` truncates the hand at `MAX_HAND = 10` — cards past the tenth become
invisible to the agent while still being playable, i.e. a silent
observation/action mismatch.

- [ ] Register the gap in `fidelity-debt.md`'s mechanism inventory as F-A11
      (this is genuine debt, unlike the pool-open marker).
- [ ] Implement: hand caps at 10; a **drawn** card over the cap is destroyed
      (burn, goes to graveyard, still counts as drawn for deck depletion); a
      **generated** card over the cap is never created.
- [ ] Scenarios: draw at 10 burns; Mind Vision at 10 does nothing; Thoughtsteal
      at 9 copies exactly 1; Mindgames is unaffected (it summons, never fills a
      hand); Cho at 10 does nothing on either side; fatigue interplay unchanged.
- [ ] Clear F-A11 from the ledger the same way any debt row leaves it:
      implementation + F5 scenario + row removal (no `(simplified: …)` comment
      is involved — F-A11 is a mechanism-inventory gap, not a per-card marker).
- [ ] `rl/obs.rs` needs no change: with the cap in place `MAX_HAND = 10` stops
      being a truncation and becomes an exact bound. Pin that with an assertion
      in the obs test — a hand can never exceed `MAX_HAND`.

**D2 (decided): in scope here, as M5.** Cho and Thoughtsteal are what make the
missing cap bite, so the fix ships with the cards that expose it rather than
landing an observation/action mismatch and filing a follow-up. M5 is a
prerequisite for closing this roadmap, not an optional tail.

---

## Risks

| Risk | Mitigation |
| --- | --- |
| Cho generates cards on both sides every spell → longer games, lower throughput | measure in M4; if games/s regresses materially, report it before merging |
| Copies bypass deck-building rules (>2 of a card, off-class cards in hand) | correct per HS — pin it in a scenario so it is not "fixed" later |
| `Event::SpellCast` now threads a subject — silent behaviour change for existing spell triggers | regression scenario over the existing `FriendlySpellCast` cards |
| Mindgames summoning into a full board / empty-minion deck | explicit scenarios; reuse `resolve_summon`'s cap |
| The `(pool-open: …)` comment gets read as fidelity debt by the Python extractor | extractor keys on "simplified"; M0 adds the cross-reference note and a test |

## Definition of done

`POOL_OPEN_CARDS` holds exactly the 4 IDs; the closure test forbids unregistered
zone-reading effects; `classic-cards.md` shows 0 ⏸️; the RL pool measures 396;
`docs/pool-openness.md` (+ zh) is the standing registry; both roadmap files are
archived to `docs/finished/`.
