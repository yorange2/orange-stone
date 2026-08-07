# Pool-Open Cards — Permanent Registry

> Canonical registry for **pool-open** cards: cards whose resolution can move a
> card into a zone without that card having been sampled from a pool
> `cards/pool.rs` controls. Chinese mirror: `pool-openness-zh.md`.

## Contract

A card is **pool-open** if resolving it can move a card into a zone without
that card having been sampled from a pool `cards/pool.rs` controls. Today the
only such cards are the four that **read the opponent's actual cards** (their
hand or deck) or **copy a cast spell**:

| Card | Class | Cost | Pool-open because | Since |
| --- | --- | --- | --- | --- |
| *(registry fills as cards land — see `pool-open-cards-roadmap.md` M2/M3)* | | | | |

The Classic pool is **closed** today: `ALL_CARDS` is Classic-only, so anything
the opponent holds is already in the pool, and every other random generator
samples a filtered subset of `ALL_CARDS` or a fixed token pool. This registry
exists so that closure stays auditable the day a second set is supported —
the four cards above are the only ones that can move a card across a pool
boundary.

**Enforcement** (in code, `cargo test`):

1. The zone-reading effect variants (`CopyRandomEnemyHandCard`,
   `CopyRandomEnemyDeckCards`, `SummonRandomEnemyDeckMinion`,
   `CopyCastSpellToOtherPlayerHand`) may appear **only** on cards listed in
   `sets::POOL_OPEN_CARDS` — pinned by `pool_open_effects_require_registry`.
2. `pool_open_registry_is_well_formed`: every registered ID resolves via
   `card_by_id`, no ID is a derivative token, no duplicates.
3. The Lorewalker Cho hook in `cards::apply_card_keywords` debug-asserts that
   its card is registered.

## Maintenance

- **Adding a pool-open card**: register the ID in `sets::POOL_OPEN_CARDS`,
  put a `(pool-open: …)` note in the card's doc block in `src/cards/`, and add
  a row to this table — all in the same change.
- **Not fidelity debt**: pool-open cards are *faithful* implementations. Do
  not use the word "simplified" in their comments — the Python debt extractor
  (`hearthstone_os/decks.py::_load_debt_ids`) keys on "simplified" and must
  not pick them up (see `fidelity-debt.md` Maintenance).
- **RL pool**: `full_pool(include_pool_open=True)` includes them; flipping the
  default to `False` closes the pool without touching the engine.
