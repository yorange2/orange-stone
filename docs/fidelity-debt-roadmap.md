# Fidelity-Debt Implementation Roadmap — W8–W12 (the 24 §11 cards)

> Active roadmap (not yet archived). Clears the remaining fidelity-debt
> ledger: §11 of `docs/fidelity-debt.md` — the 2026-08-06 registration pass
> found 27 cards with known simplifications but no `(simplified: …)` markers;
> F-A8 (PR #88) resolved 3, leaving **24**. Each card leaves the ledger only
> when its real Hearthstone effect is implemented **and** verified by an F5
> differential test (ledger maintenance contract). Chinese mirror:
> `fidelity-debt-roadmap-zh.md`.

## Principles

1. **Wiring first** (the archived W0–W7 roadmap's rule): waves whose
   mechanisms already exist land before any new-mechanism wave.
2. **One wave = one PR** (W0–W7 precedent: PR #79–#86). A card leaves the
   ledger only with an F5 differential scenario in `tests/differential.rs`
   (`w8_*` … `w12_*`) pinning the real resolution order, plus SabberStone
   parity where the semantics are mirrorable.
3. **RL pool**: every cleared card re-enters the training pool
   (367 → **391** when the ledger empties); per the ledger contract, each
   change also removes the code `(simplified …)` comment and invalidates the
   debt-extractor cache (`~/.cache/orange_stone_debt_ids.txt`).

## Mechanism inventory (verified 2026-08-06)

| Need | Status | Notes |
| --- | --- | --- |
| Enrage (`ThisMinionDamaged` keyword map) | ✅ exists (W0) | Amani Berserker / Raging Worgen / Grommash: wiring only |
| Charge aura (`GrantCharge`, W1 Tundra Rhino) | ✅ exists | Warsong Commander: all-friendly-minions aura, self excluded |
| Heal-draw (`CharacterHealed`, W2) | ✅ exists | Northshire Cleric: trigger wiring |
| Weapon attack effects (`Attacked`, W2 Blessing of Wisdom) | ✅ exists | Truesilver heal; Gorehowl attack drain |
| `SecretPlayed` (W2) | ✅ exists | Eaglehorn Bow +1 durability |
| Race-conditioned target (`FriendlyRace`, W1) | ✅ exists | Bestial Wrath: Beast-only target |
| Choose-One pipeline (`ChoiceKind::ChooseOne`, G6) | ✅ exists | 4 Druid cards: likely def-side branch wiring |
| Discover pipeline (`ChoiceKind::Discover` + pool, trigger.rs) | ✅ exists | Tracking: needs a **deck-top-3** pool source |
| Battlecry summon / discard / shield | partial | Onyxia multi-summon; Deathwing full-hand discard (new effect); Argus shield targets |
| Cost reduction (Sea Giant / Preparation) | partial | board-count cost aura (new); next-spell-cheaper flag (Millhouse W4 precedent) |
| Heal doubling (Prophet Velen) | missing | new aura/modifier |
| Freeze on damage dealt (Water Elemental) | missing | attacker-deals-damage trigger direction (decide in W12) |
| Control (Cabal Shadow Priest) | ✅ exists | `TakeControl` + `EnemyMinionAttackLE` (W3): battlecry wiring |
| Deal-damage-and-draw (Shiv) | ✅ exists | `DealDamageAndDraw` (Holy Wrath path): wiring |
| Combo base branch (Cold Blood) | partial | combo pipeline exists; missing the non-combo +2 branch |

## Waves

### W8 — Trigger wiring (5 cards): Amani Berserker, Raging Worgen, Grommash Hellscream, Warsong Commander, Northshire Cleric

- Enrage wiring via the `ThisMinionDamaged` keyword map (W0 precedent):
  Amani Berserker (2/3 → 5/3 enraged), Raging Worgen (1/3 + Windfury → 3/3
  + Windfury), Grommash (4/9 → 7/9 enraged)
- Warsong Commander: all-other-friendly-minions `GrantCharge` aura
  (Tundra Rhino precedent, self excluded)
- Northshire Cleric: draw a card on friendly `CharacterHealed`

**Acceptance**: 5 `w8_*` differential scenarios (enrage resolution order,
charge aura grant, heal-draw timing vs the heal event); `cargo test` green;
RL pool 367 → 372; §11 loses 5 rows.

### W9 — Weapon & race (4 cards): Truesilver Champion, Gorehowl, Eaglehorn Bow, Bestial Wrath

- Truesilver Champion: heal 2 when the hero attacks with it (`Attacked`
  precedent)
- Gorehowl: the equipped weapon loses 1 attack when the hero attacks a
  minion (enchantment, not durability)
- Eaglehorn Bow: +1 durability whenever a friendly Secret is played
  (`SecretPlayed` precedent)
- Bestial Wrath: `FriendlyRace(Beast)` target for the existing
  `GrantAttackAndImmune` effect

**Acceptance**: 4 `w9_*` scenarios (heal timing on attack, attack drain per
minion hit, durability gain per secret, beast-only targeting); `cargo test`
green; RL pool 372 → 376.

### W10 — Choose-One & Discover (5 cards): Wrath, Druid of the Claw, Ancient of Lore, Ancient of War, Tracking

- Wrath / Druid of the Claw / Ancient of Lore / Ancient of War: wire both
  branches into the existing Choose-One pipeline (audit why the defs
  currently fall back to a fixed branch — likely missing second-branch
  wiring, not a pipeline gap)
- Tracking: Discover over the **top 3 cards of the player's deck** — pick
  one into hand, discard the rest. The Discover pipeline exists but draws
  from a card pool; decide (D1) whether the deck-top-3 is a new pool
  source or a parallel choice

**Acceptance**: 5 `w10_*` scenarios (branch choices, cost/target per branch,
Tracking deck-top-3 semantics incl. the discard); `cargo test` green;
RL pool 376 → 381.

### W11 — Battlecries & cost reduction (6 cards): Onyxia, Defender of Argus, Deathwing, Sea Giant, Preparation, Cold Blood

- Onyxia: summon five 1/1 Whelps (multi-summon — `SummonMinion` loop)
- Defender of Argus: adjacent minions gain +1/+1 **and** Divine Shield
  (target structure + shield grant; W5 adjacency precedent)
- Deathwing: **discard the whole hand** (new effect — `DiscardRandomCard`
  exists, full-hand discard does not) + destroy all other minions
- Sea Giant: costs 1 less per minion on the board (board-count cost aura —
  new; `FirstMinionDiscount` W3 precedent)
- Preparation: your next spell this turn costs 3 less (per-player flag,
  Millhouse `spells_cost_zero` W4 precedent)
- Cold Blood: the base +2 branch wired alongside the existing combo branch

**Acceptance**: 6 `w11_*` scenarios (Whelp count + board fill, Argus
adjacency incl. shield grant order, Deathwing discard-all + board wipe,
Sea Giant cost vs board state, Preparation window, Cold Blood base branch);
`cargo test` green; RL pool 381 → 387.

### W12 — Remaining mechanics (4 cards): Water Elemental, Cabal Shadow Priest, Prophet Velen, Shiv

- Water Elemental: freeze characters damaged **by** this minion — decide
  (D2) between a new source-side trigger or a damage-pipeline check
- Cabal Shadow Priest: battlecry `TakeControl` of an enemy minion with
  attack ≤ 2 (`EnemyMinionAttackLE` predicate, W3)
- Prophet Velen: healing doubled — new heal modifier (decide D3: aura vs
  pipeline hook, mirroring the spell-damage double)
- Shiv: 1 damage + draw 1 (`DealDamageAndDraw` — wiring)

**Acceptance**: 4 `w12_*` scenarios; `cargo test` green; **RL pool 387 →
391 (ledger §11 empty)**; archive this roadmap pair to `docs/finished/`.

## Wave accounting

| Wave | Cards | RL pool |
| --- | --- | --- |
| W8 trigger wiring | 5 | 367 → **372** |
| W9 weapon & race | 4 | 372 → **376** |
| W10 choose-one & discover | 5 | 376 → **381** |
| W11 battlecries & cost | 6 | 381 → **387** |
| W12 remaining mechanics | 4 | 387 → **391 (ledger empty)** |

## Out of scope

- Cards not in §11 with known simplifications: none known — the ledger's
  maintenance contract requires any new `(simplified …)` comment to be
  registered here on the same change.
- 168-dim observation tensor changes (D5 of the fatigue roadmap).
