# Fidelity-Debt Implementation Roadmap — the 67 simplified cards

> **Status: W0 in progress.** This roadmap executes the F4-ongoing / F5-ongoing
> items of [architecture-roadmap.md](architecture-roadmap.md). The
> [fidelity-debt.md](fidelity-debt.md) ledger is the source of truth for the card
> list; this document is the execution plan. A card **leaves the ledger** only when
> implemented **and** verified by an F5 differential test — see the ledger's
> [F5 verification](fidelity-debt.md#f5-verification-per-fix) and
> [Maintenance](fidelity-debt.md#maintenance) sections.
>
> Verified against the engine on 2026-08-06: all 67 markers, trigger-scope
> semantics, and the mechanism inventory below are current as of PR #77.

## Principles

1. **Primitives before cards** (Review-II discipline): a card wave ships only when
   the mechanism it needs exists and is tested.
2. **Wiring first**: cards whose mechanism already exists (W0) are done before any
   new primitive — they clean the pool fastest and stress-test the existing
   trigger/aura machinery.
3. **One differential scenario per card** (`tests/differential.rs`): target sets,
   trigger timing, death-phase interplay; SabberStone deck-level parity where
   mirrorable (`docs/differential_sabberstone.md`).
4. **Pool flows through the ledger**: when a card is fixed, remove its ledger row
   and its `(simplified…)` comment, then invalidate
   `~/.cache/orange_stone_debt_ids.txt` — the RL pool re-grows automatically.
5. One PR per wave; each wave must keep `cargo test` green and the RL pool checks
   (`hearthstone_os` tests + M5 smoke) passing.

## Mechanism inventory (as of 2026-08-06)

**Already exists** (verified in code):

| Mechanism | Where | Used by |
| --- | --- | --- |
| `summon_trigger` → `FriendlyMinionSummoned` (owner-only) | `trigger.rs:737` | Starving Buzzard, Sword of Justice |
| `spell_trigger` → `FriendlySpellCast` (owner-only) | `trigger.rs:717` | Mana Wyrm, Sorcerer's Apprentice |
| `death_trigger` → `FriendlyMinionDied` (**own minions only**) | `trigger.rs:727` | Cult Master |
| `start_turn_effect` → `TurnStart` | `trigger.rs:709` | Ragnaros, Doomsayer-style |
| `end_turn_effect` → `TurnEnd` | — | Vanish-class, Lightwell |
| `ThisMinionDamaged` (self) | `cards/mod.rs` keyword map | Acolyte of Pain |
| `FriendlyMinionDamaged` | `cards/mod.rs` keyword map | Frothing Berserker, Armorsmith |
| `BuffWeapon` (attack+durability) | `effect.rs` | upgrade-style effects |
| `Poison` component, mapped by card ID | `cards/mod.rs:136` | Patient Assassin |
| `FreezeCharacter`, `FullHeal`, `SilenceMinion`+`AllEnemyMinions`, `DealDamageToTwo`, `DestroyAdjacent`, `GrantCharge`, `GrantWindfury` | `effect.rs` | Cleave, etc. |
| Cost-modifier stack (G5) incl. hand-zone reductions | `engine::cost` | Frost Giant-class, Dread Corsair-adjacent |
| `DealDamageAndDraw`, `SummonMinion`, `ReturnToHand`+… | `effect.rs` | — |

**Missing** (per-group prerequisites):

1. race/tribe field on `CardDef` + race predicates (targets, auras, trigger events)
   → W1
2. trigger classes: heal, attack (on a target), card-played, secret-played,
   any-minion-died → W2
3. target/state predicates: attack-range, hand-size, hero-health, damaged-friendly,
   damaged-count, owns-secret, "first minion this turn", weapon-equipped →
   W3; divine-shield absorb → W3
4. hand-zone cost auras (all minions), weapon-attack cost modifier,
   weapon-durability damage, enemy-spells-cost-0, give-opponent-mana →
   W4
5. set-health-to-1, swap attack/health, adjacent-target buff/freeze,
   two-effect composition (incl. a taunt-grant) → W5
6. probability, this-turn temp buff, mass Divine Shield, self-exclusion AOE,
   draw-then-damage-by-cost, class filter → W6
7. hand-zone swap, damage-reflection secret, 1-Health resummon secret → W7

---

## Wave 0 — Wiring: mechanisms already exist (13 cards)

No new primitives intended. Several cards carry **verify-first** notes where the
existing mechanism's exact semantics must be confirmed before the card is wired;
a verify may still surface a small engine fix (acceptable in this wave).

| # | ID | Card | Mechanism to wire | Note |
| --- | --- | --- | --- | --- |
| 1 | NEUTRAL_R09 | Knife Juggler | `summon_trigger` + `DealDamage{AnyEnemy}` | random target picked at resolution ✓ |
| 2 | HUNTER_012 | Wild Pyromancer | `spell_trigger` + `DealDamage{AllMinions}` | fires after *your* spell ✓ |
| 3 | NEUTRAL_R15 | Demolisher | `start_turn_effect` + `DealDamage{AnyEnemy}` | random enemy ✓ |
| 4 | NEUTRAL_E04 | Doomsayer | `start_turn_effect` + `DestroyMinion{AllMinions}` | includes self — verify destroy-all ordering |
| 5 | EX1_365 | Sword of Justice | `summon_trigger` buffing the **summoned minion** | verify `Self_` binding in trigger context; likely small target fix |
| 6 | PRIEST_017 | Kul Tiran Chaplain | battlecry `GainStats{FriendlyMinion}` | same target family as Shattered Sun Cleric — verify why it was simplified |
| 7 | NEUTRAL_R21 | Young Priestess | `end_turn_effect` + `GainStats{FriendlyMinion}` | verify "another" (self-exclusion) — may need `OtherFriendlyMinion` target |
| 8 | NEUTRAL_R23 | Master Swordsmith | same as above | same verify |
| 9 | NEUTRAL_B19 | Gurubashi Berserker | `ThisMinionDamaged` + `GainStats{Self_}` | Enrage = damage-triggered permanent buff |
| 10 | NEUTRAL_C11 | Tauren Warrior | Taunt + `ThisMinionDamaged` + `GainStats{Self_}` | Enrage |
| 11 | NEUTRAL_R02 | Angry Chicken | `ThisMinionDamaged` + `GainStats{Self_}` | Enrage |
| 12 | NEUTRAL_C15 | Spiteful Smith | `ThisMinionDamaged` + `BuffWeapon{atk+2}` | Enrage on weapon |
| 13 | NEUTRAL_R16 | Emperor Cobra | add ID to `apply_card_keywords` poison map | Poison component exists |

**Acceptance**: 13 differential scenarios; the 4 Enrage cards' damage-sequencing
(trigger fires once per damage event, buff persists); RL pool grows by 13.

## Wave 1 — Race/tribe field (11 cards)

**Primitives** (one PR-sized unit):
- `CardDef.race: Option<Race>` (`Beast` / `Murloc` / `Demon`), applied on spawn;
  race exposed in the structured view (`EntityView`) for the RL side.
- Race-conditioned targets: `FriendlyBeast`, `AnyMurloc`, `FriendlyDemon`,
  `OtherMurlocs` / `OtherDemons` aura targets.
- Race-conditioned trigger events: `FriendlyMurlocSummoned`, `FriendlyBeastDied`.
- Deck-filtered draw (Sense Demons) — draw from the deck filtered by race.
- Replace hardcoded `BEAST_POOL` / `DEMON_POOL` (`cards/pool.rs`) with
  field-driven pools (keep behavior, delete ID lists).

| ID | Card | Real effect |
| --- | --- | --- |
| HUNTER_015 | Houndmaster | Battlecry: give a friendly **Beast** +2/+2 and Taunt |
| HUNTER_016 | Tundra Rhino | Your **Beasts** have Charge |
| NEUTRAL_R01 | Coldlight Seer | Battlecry: all other **Murlocs** +2 Health |
| NEUTRAL_E02 | Murloc Warleader | Your other **Murlocs** +2/+1 |
| NEUTRAL_R05 | Murloc Tidecaller | Whenever you summon a **Murloc**, +1 Attack |
| NEUTRAL_E03 | Hungry Crab | Battlecry: destroy a **Murloc**, gain +2/+2 |
| WARLOCK_020 | Sense Demons | Draw two **Demons** from your deck |
| WARLOCK_021 | Demonfire | 2 damage; friendly **Demon** gets +2/+2 instead |
| WARLOCK_T01 | Siegebreaker | Taunt; your other **Demons** +1 Attack |
| HUNTER_013 | Scavenging Hyena | Whenever a friendly **Beast** dies, +2/+1 |
| HUNTER_014 | Starving Buzzard | Whenever you summon a **Beast**, draw |

**Acceptance**: 11 differential scenarios; race pools verified identical to the
hardcoded lists; RL pool grows by 11.

## Wave 2 — Trigger classes (8 cards)

**Primitives**:
- `heal_trigger` → `FriendlyCharacterHealed` (Lightwarden)
- `attack_trigger` on a target entity → `Attacked` (Blessing of Wisdom)
- `played_trigger` → `CardPlayed` (Questing Adventurer)
- `secret_played_trigger` → `SecretPlayed` (Secretkeeper)
- `any-minion-died` variant of `FriendlyMinionDied` (Flesheathing Ghoul)
- `DestroySecrets` effect (SI:7 — one random enemy secret; Eater of Secrets /
  Flare — all enemy secrets; Flare also draws → composition, see W5)

| ID | Card | Real effect |
| --- | --- | --- |
| NEUTRAL_R04 | Lightwarden | Whenever a character is healed, +2 Attack |
| PALADIN_019 | Blessing of Wisdom | Whenever the target minion attacks, draw |
| NEUTRAL_R17 | Questing Adventurer | Whenever you play a card, +1/+1 |
| NEUTRAL_R06 | Secretkeeper | Whenever a Secret is played, +1/+1 |
| NEUTRAL_R25 | SI:7 Infiltrator | Battlecry: destroy a random enemy Secret |
| NEUTRAL_R26 | Eater of Secrets | Battlecry: destroy all enemy Secrets, +1/+1 |
| HUNTER_017 | Flare | Destroy all enemy secrets, draw a card |
| NEUTRAL_C12 | Flesheathing Ghoul | Whenever **any** minion dies, +1 Attack |

**Acceptance**: 8 differential scenarios (trigger timing: "after" vs "whenever"
per card); RL pool grows by 8.

## Wave 3 — Conditional predicates (9 cards)

**Primitives** (extend `EffectTarget` / add effect conditions):
- attack-range predicates (≤2, ≥7), hand-size count, hero-health threshold,
  damaged-friendly target + damaged-count, owns-secret, "first minion this turn"
  state, divine-shield absorb-and-buff.

| ID | Card | Real effect |
| --- | --- | --- |
| NEUTRAL_R20 | Stampeding Kodo | Destroy a random enemy minion with ≤2 Attack |
| NEUTRAL_E06 | Big Game Hunter | Destroy a minion with ≥7 Attack |
| NEUTRAL_R19 | Twilight Drake | +1 Health per card in hand |
| WARRIOR_021 | Mortal Strike | 4 damage; 6 if you have ≤12 Health |
| WARRIOR_023 | Rampage | Give a **damaged** minion +3/+3 |
| WARRIOR_022 | Battle Rage | Draw per damaged friendly character |
| MAGE_017 | Ethereal Arcanist | End of turn: if you control a Secret, +2/+2 |
| NEUTRAL_R24 | Pint-Sized Summoner | First minion each turn costs (1) less |
| NEUTRAL_E05 | Blood Knight | Absorb all Divine Shields, +3/+3 |

**Acceptance**: 9 differential scenarios; RL pool grows by 9.

## Wave 4 — Cost & weapon interactions (8 cards)

**Primitives**:
- hand-zone cost aura targeting "all minions" / "your minions" (on top of the G5
  modifier stack), weapon-attack cost modifier, weapon-durability damage,
  weapon-equipped predicate (conditional Charge), enemy-spells-cost-0,
  give-opponent-mana.

| ID | Card | Real effect |
| --- | --- | --- |
| NEUTRAL_R22 | Mana Wraith | ALL minions cost (1) more |
| NEUTRAL_C14 | Venture Co. Mercenary | Your minions cost (3) more |
| NEUTRAL_C07 | Southsea Deckhand | Charge while you have a weapon |
| NEUTRAL_C13 | Dread Corsair | Taunt; costs (1) less per weapon Attack |
| NEUTRAL_C09 | Bloodsail Raider | Battlecry: gain Attack equal to weapon's |
| NEUTRAL_R03 | Bloodsail Corsair | Battlecry: remove 1 durability from enemy weapon |
| LEGENDARY_021 | Millhouse Manastorm | Enemy spells cost 0 next turn |
| NEUTRAL_R14 | Arcane Golem | Charge; opponent gains a Mana Crystal |

**Acceptance**: 8 differential scenarios (incl. cost-modifier stacking vs.
existing auras); RL pool grows by 8.

## Wave 5 — Target structure & effect composition (7 cards)

**Primitives**:
- `SetHealthTo` effect (Repentance), swap attack/health effect (Crazed Alchemist),
  adjacent-target buff/freeze targets, two-effect composition
  (`SilenceAllAndDraw`, `FullHealAndTaunt` — or a generic chain), `GrantTaunt`.

| ID | Card | Real effect |
| --- | --- | --- |
| EX1_349 | Repentance | Secret: opponent's minion's Health set to 1 |
| PRIEST_018 | Mass Dispel | Silence ALL enemy minions, draw a card |
| NEUTRAL_R08 | Crazed Alchemist | Swap a minion's Attack and Health |
| MAGE_016 | Cone of Cold | Freeze a minion and its neighbors |
| NEUTRAL_R11 | Sunfury Protector | Give adjacent minions Taunt |
| NEUTRAL_R18 | Ancient Mage | Give adjacent minions Spell Damage +1 |
| SHAMAN_018 | Ancestral Healing | Full-heal a minion and give it Taunt |

**Acceptance**: 7 differential scenarios; RL pool grows by 7.

## Wave 6 — Special mechanics (8 cards)

**Primitives**: probabilistic effect (Nat Pagle), this-turn temporary buff with
end-of-turn expiry (Mana Addict; the G4 enchantment layer already has
`UntilEndOfTurn`), mass Divine Shield, self-exclusion AOE (AllOtherCharacters),
draw-then-damage-equal-to-cost (variable-amount damage), class filter for random
card pools (Pilfer).

| ID | Card | Real effect |
| --- | --- | --- |
| LEGENDARY_022 | Nat Pagle | End of turn: 50% chance to draw |
| NEUTRAL_R10 | Mana Addict | After spell: +2 Attack this turn |
| PALADIN_018 | Righteousness | Give your minions Divine Shield |
| NEUTRAL_T21e | Ysera Awakens | Deal 5 damage to all **other** characters |
| DRUID_016 | Gift of the Wild | Your minions +2/+2 and Taunt |
| PALADIN_017 | Holy Wrath | Draw; deal damage equal to its cost |
| EX1_341 | Lightwell | Start of turn: heal 3 to a damaged friendly character |
| ROGUE_025 | Pilfer | Add a random card from another class to hand |

**Acceptance**: 8 differential scenarios; RL pool grows by 8.

## Wave 7 — Wrap-up: complex leftovers (3 cards)

**Primitives**: hand-zone swap (Alarm-o-Bot), damage-reflection secret
(Eye for an Eye), resummon-with-1-Health death secret (Redemption).

| ID | Card | Real effect |
| --- | --- | --- |
| NEUTRAL_R13 | Alarm-o-Bot | Start of turn: swap with a random hand minion |
| PALADIN_020 | Eye for an Eye | Secret: reflect hero damage to the enemy hero |
| PALADIN_021 | Redemption | Secret: resummon a dead minion with 1 Health |

**Acceptance**: 3 differential scenarios; ledger empty; RL pool at full Classic
constructible size; final sweep + full SabberStone parity run.

---

## Cross-cutting tasks (each wave)

- Invalidate `~/.cache/orange_stone_debt_ids.txt`; re-run the pool checks
  (`hearthstone_os` tests + `tools/orange_stone_m5_smoke.py`).
- Update the ledger: remove the fixed rows, drop the code comments, note the
  differential scenario numbers.
- `cargo bench` on any hot-path touch (trigger firing, aura evaluation) — the
  per-wave primitives must not regress batch throughput (M4 baseline ~4,200 局/s).

## Wave accounting

| Wave | Cards | New primitives | Pool growth |
| --- | --- | --- | --- |
| W0 wiring | 13 | — (verify-first) | +13 |
| W1 race | 11 | race field + predicates + pools | +11 |
| W2 triggers | 8 | 4 trigger classes + destroy-secret | +8 |
| W3 predicates | 9 | 6+ predicates | +9 |
| W4 cost/weapon | 8 | 6+ primitives | +8 |
| W5 target structure | 7 | 5+ primitives | +7 |
| W6 special mechanics | 8 | 6 primitives | +8 |
| W7 wrap-up | 3 | 3 primitives | +3 |
| **Total** | **67** | | **321 → 388** |
