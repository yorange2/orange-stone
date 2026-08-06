# Fidelity-Debt Implementation Roadmap — the 67 simplified cards

> **Status: W6 done (PR #86); W7 next.** This roadmap executes the F4-ongoing /
> F5-ongoing items of [architecture-roadmap.md](architecture-roadmap.md). The
> [fidelity-debt.md](fidelity-debt.md) ledger is the source of truth for the card
> list; this document is the execution plan. A card **leaves the ledger** only when
> implemented **and** verified by an F5 differential test — see the ledger's
> [F5 verification](fidelity-debt.md#f5-verification-per-fix) and
> [Maintenance](fidelity-debt.md#maintenance) sections.
>
> Verified against the engine on 2026-08-06: all 67 markers, trigger-scope
> semantics, and the mechanism inventory below are current as of PR #77.
> W0 (13 cards) landed in PR #79; the inventory below is updated for the W0
> primitives.

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
| `Poison` component, mapped by card ID | `cards/mod.rs:136` | Patient Assassin, Emperor Cobra (W0) |
| `FreezeCharacter`, `FullHeal`, `SilenceMinion`+`AllEnemyMinions`, `DealDamageToTwo`, `DestroyAdjacent`, `GrantCharge`, `GrantWindfury` | `effect.rs` | Cleave, etc. |
| Cost-modifier stack (G5) incl. hand-zone reductions | `engine::cost` | Frost Giant-class, Dread Corsair-adjacent |
| `DealDamageAndDraw`, `SummonMinion`, `ReturnToHand`+… | `effect.rs` | — |
| `ThisMinionDamaged` → Enrage buffs (W0) | `cards/mod.rs` keyword map | 4 Enrage cards |
| `EffectTarget::EventSubject` / `OtherFriendlyMinion` (W0) | `core/effect.rs` | Sword of Justice; Young Priestess / Master Swordsmith |
| Weapon triggers register + destroyed weapons leave play (W0) | `trigger.rs` / `rules.rs` | Sword of Justice |
| Spell-cast deaths resolve before after-cast triggers (W0) | `rules.rs` SpellCast | Wild Pyromancer |

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

## Wave 0 — Wiring: mechanisms already exist (13 cards) ✅ done (PR #79)

No new primitives intended. Several cards carry **verify-first** notes where the
existing mechanism's exact semantics must be confirmed before the card is wired;
a verify may still surface a small engine fix (acceptable in this wave).

| # | ID | Card | Mechanism wired | Scenario(s) |
| --- | --- | --- | --- | --- |
| 1 | NEUTRAL_R09 | Knife Juggler | `summon_trigger` + `DealDamage{AnyEnemy}` | `w0_knife_juggler_throws_after_friendly_summon` |
| 2 | HUNTER_012 | Wild Pyromancer | `spell_trigger` + `DealDamage{AllMinions}` | `w0_wild_pyromancer_aoe_after_spell` + `…_killed_by_the_spell` |
| 3 | NEUTRAL_R15 | Demolisher | `start_turn_effect` + `DealDamage{AnyEnemy}` | `w0_demolisher_fires_at_turn_start` |
| 4 | NEUTRAL_E04 | Doomsayer | `start_turn_effect` + `DestroyMinion{AllMinions}` | `w0_doomsayer_destroys_all_minions_including_itself` |
| 5 | EX1_365 | Sword of Justice | `summon_trigger` → new `EffectTarget::EventSubject` | `w0_sword_of_justice_buffs_the_summoned_minion` + `…_stops_firing_when_destroyed` |
| 6 | PRIEST_017 | Kul Tiran Chaplain | battlecry `GainStats{FriendlyMinion}` | `w0_kul_tiran_chaplain_buffs_a_friendly_minion` |
| 7 | NEUTRAL_R21 | Young Priestess | `end_turn_effect` → new `EffectTarget::OtherFriendlyMinion` | `w0_young_priestess_buffs_another_minion` + `…_alone_does_nothing` |
| 8 | NEUTRAL_R23 | Master Swordsmith | same as above | `w0_master_swordsmith_buffs_another_minion` |
| 9 | NEUTRAL_B19 | Gurubashi Berserker | `ThisMinionDamaged` + `GainStats{Self_}` | `w0_gurubashi_berserker_enrage_permanent` |
| 10 | NEUTRAL_C11 | Tauren Warrior | Taunt + `ThisMinionDamaged` + `GainStats{Self_}` | `w0_tauren_warrior_enrage_with_taunt` |
| 11 | NEUTRAL_R02 | Angry Chicken | `ThisMinionDamaged` + `GainStats{Self_}` | `w0_angry_chicken_enrage_fires_before_death` |
| 12 | NEUTRAL_C15 | Spiteful Smith | `ThisMinionDamaged` + `BuffWeapon{atk+2}` | `w0_spiteful_smith_buffs_weapon_on_damage` |
| 13 | NEUTRAL_R16 | Emperor Cobra | ID added to `apply_card_keywords` poison map | `w0_emperor_cobra_poison_kills_and_divine_shield_absorbs` |

**W0 findings** (small engine fixes the wiring surfaced, all in PR #79):
- Sword of Justice verified the trigger-context question: weapon entities now
  register their CardDef triggers when equipped (played-from-hand weapons
  already did), and a destroyed weapon leaves play — a broken sword stops firing.
- The `Self_` binding was wrong for buff-the-summoned-minion; the new
  `EffectTarget::EventSubject` resolves the trigger event's subject directly
  (a summoned minion that left play is a no-op).
- "Another" for Young Priestess / Master Swordsmith needed a real
  `OtherFriendlyMinion` target (excludes the source from the candidate set).
- Death-vs-after-cast ordering: the spell-cast event now resolves the spell's
  pending deaths before firing `FriendlySpellCast` triggers (HS semantics — a
  Wild Pyromancer killed by its own spell does not fire).

**Acceptance**: 16 differential scenarios (13 cards, sword/pyro/priestess have two
each); the 4 Enrage cards' damage-sequencing (trigger fires once per damage event,
buff persists); RL pool grows by 13.

## Wave 1 — Race/tribe field (11 cards) ✅ done (PR #80)

**Primitives** (one PR-sized unit) — all landed:
- `CardDef.race: Option<Race>` (`Beast` / `Murloc` / `Demon`), applied on spawn;
  race exposed in the structured view (`EntityView`) and the Python bindings.
- Race-conditioned targets: `EffectTarget::FriendlyRace` (Houndmaster),
  `AnyRace` (Hungry Crab), `AllOtherFriendlyRace` (Coldlight Seer).
- Race-conditioned auras: `AuraTarget::FriendlyRace` (Tundra Rhino — a real
  `AuraEffect::GrantCharge` Charge aura) and `OtherFriendlyRace`
  (Murloc Warleader, Siegebreaker).
- Race-conditioned triggers: a `Trigger.race` field (Murloc Tidecaller /
  Scavenging Hyena / Starving Buzzard, registered per card ID in
  `apply_card_keywords`).
- Deck-filtered draw (Sense Demons — `CardEffect::DrawCardByRace`).
- Hardcoded `BEAST_POOL` / `DEMON_POOL` replaced by field-driven pools
  (parity pinned by `w1_race_pools_are_field_driven`: every old member stays;
  the additions are exactly the genuine Beasts/Demons the old lists missed).

| ID | Card | Real effect | Scenario |
| --- | --- | --- | --- |
| HUNTER_015 | Houndmaster | Battlecry: give a friendly **Beast** +2/+2 and Taunt | `w1_houndmaster_buffs_only_a_friendly_beast` |
| HUNTER_016 | Tundra Rhino | Your **Beasts** have Charge | `w1_tundra_rhino_gives_beasts_charge` |
| NEUTRAL_R01 | Coldlight Seer | Battlecry: all other **Murlocs** +2 Health | `w1_coldlight_seer_buffs_other_murlocs_only` |
| NEUTRAL_E02 | Murloc Warleader | Your other **Murlocs** +2/+1 | `w1_murloc_warleader_aura_murloc_only` |
| NEUTRAL_R05 | Murloc Tidecaller | Whenever you summon a **Murloc**, +1 Attack | `w1_murloc_tidecaller_gains_attack_on_murloc_summon` |
| NEUTRAL_E03 | Hungry Crab | Battlecry: destroy a **Murloc**, gain +2/+2 | `w1_hungry_crab_destroys_enemy_murloc_and_buffs` |
| WARLOCK_020 | Sense Demons | Draw two **Demons** from your deck | `w1_sense_demons_draws_two_demons_from_deck` |
| WARLOCK_021 | Demonfire | 2 damage; friendly **Demon** becomes +2/+2 | `w1_demonfire_buffs_friendly_demon_and_damages_others` |
| WARLOCK_T01 | Siegebreaker | Taunt; your other **Demons** +1 Attack | `w1_siegebreaker_buffs_other_demons` |
| HUNTER_013 | Scavenging Hyena | Whenever a friendly **Beast** dies, +2/+1 | `w1_scavenging_hyena_only_counts_beast_deaths` |
| HUNTER_014 | Starving Buzzard | Whenever you summon a **Beast**, draw a card | `w1_starving_buzzard_draws_on_beast_summon` |

**Acceptance**: 12 scenarios (11 cards + the pool-parity test); race pools match
the hardcoded lists bit-for-bit (old members all stay; additions = genuine
Beasts/Demons the old lists missed); RL pool grows by 11 (335 → 346).
## Wave 2 — Trigger classes (8 cards) ✅ done (PR #81)

**Primitives** — all landed:
- `TriggerEvent::CharacterHealed` — heal trigger (Lightwarden); fires for any
  healed character, and only on REAL heals (an undamaged character is not a
  heal event).
- `TriggerEvent::Attacked` — entity-attack trigger (Blessing of Wisdom attaches
  "draw when this minion attacks" to the target via `CardEffect::AttachAttackDraw`).
- `TriggerEvent::CardPlayed` — card-played trigger (Questing Adventurer; friendly scope).
- `TriggerEvent::SecretPlayed` — secret-played trigger (Secretkeeper; both players).
- `TriggerEvent::MinionDied` — any-minion-died variant (Flesheathing Ghoul; both players).
- Destroy-secret effects: `DestroyRandomEnemySecret` (SI:7 Infiltrator),
  `DestroyAllEnemySecretsAndGainStats` (Eater of Secrets),
  `DestroyAllEnemySecretsAndDraw` (Flare).

| ID | Card | Real effect | Scenario |
| --- | --- | --- | --- |
| NEUTRAL_R04 | Lightwarden | Whenever a character is healed, +2 Attack | `w2_lightwarden_gains_attack_on_real_heals` |
| PALADIN_019 | Blessing of Wisdom | Whenever the target minion attacks, draw a card | `w2_blessing_of_wisdom_draws_on_attacks` |
| NEUTRAL_R17 | Questing Adventurer | Whenever you play a card, +1/+1 | `w2_questing_adventurer_grows_per_played_card` |
| NEUTRAL_R06 | Secretkeeper | Whenever a Secret is played, +1/+1 | `w2_secretkeeper_grows_on_any_secret` |
| NEUTRAL_R25 | SI:7 Infiltrator | Battlecry: destroy a random enemy Secret | `w2_si7_destroys_one_enemy_secret` |
| NEUTRAL_R26 | Eater of Secrets | Battlecry: destroy all enemy Secrets, +1/+1 | `w2_eater_of_secrets_destroys_all_and_buffs` |
| HUNTER_017 | Flare | Destroy all enemy Secrets and draw a card | `w2_flare_destroys_all_secrets_and_draws` |
| NEUTRAL_C12 | Flesheathing Ghoul | Whenever a minion dies, +1 Attack | `w2_flesheathing_ghoul_counts_every_death` |

**Acceptance**: 8 differential scenarios (per-card after/whenever timing
verified); RL pool grows by 8 (346 → 354).
## Wave 3 — Conditional predicates (9 cards) ✅ done (PR #82)

**Primitives** — all landed:
- Attack-range targets: `EffectTarget::EnemyMinionAttackLE` (Kodo, ≤2),
  `AnyMinionAttackGE` (Big Game Hunter, ≥7, either side).
- Hand-size counting: `CardEffect::GainStatsPerHandCard` (Twilight Drake).
- Hero-health threshold: `CardEffect::MortalStrike` (6 damage at ≤12 health).
- Damaged targets: `DamagedFriendlyMinion` / `DamagedMinion` (Rampage, either side).
- Damaged-counting: `CardEffect::DrawPerDamagedFriendlyCharacter` (Battle Rage —
  hero and minions count).
- Owns-secret: `CardEffect::GainStatsIfOwnSecret` (Ethereal Arcanist).
- "First minion this turn" state: `AuraEffect::FirstMinionDiscount` + a
  per-player `minions_played_this_turn` counter (Pint-Sized Summoner;
  silencing the summoner removes the aura and the discount).
- Divine-shield absorb: `CardEffect::AbsorbDivineShields` (Blood Knight —
  +3/+3 per shield, both sides).

| ID | Card | Real effect | Scenario |
| --- | --- | --- | --- |
| NEUTRAL_R20 | Stampeding Kodo | Destroy a random enemy minion with 2 or less Attack | `w3_stampeding_kodo_destroys_low_attack_minion` |
| NEUTRAL_E06 | Big Game Hunter | Destroy a minion with 7 or more Attack | `w3_big_game_hunter_destroys_high_attack_minion` |
| NEUTRAL_R19 | Twilight Drake | +1 Health for each card in hand | `w3_twilight_drake_gains_health_per_hand_card` |
| WARRIOR_021 | Mortal Strike | 4 damage; 6 if you have 12 or less Health | `w3_mortal_strike_boosts_at_low_health` |
| WARRIOR_023 | Rampage | Give a damaged minion +3/+3 | `w3_rampage_targets_only_damaged_minions` |
| WARRIOR_022 | Battle Rage | Draw a card for each damaged friendly character | `w3_battle_rage_draws_per_damaged_friendly_character` |
| MAGE_017 | Ethereal Arcanist | End of turn: if you control a Secret, +2/+2 | `w3_ethereal_arcanist_requires_a_secret` |
| NEUTRAL_R24 | Pint-Sized Summoner | The first minion each turn costs (1) less | `w3_pint_sized_summoner_discounts_first_minion` |
| NEUTRAL_E05 | Blood Knight | Destroy all Divine Shields, +3/+3 each | `w3_blood_knight_absorbs_all_divine_shields` |

**Acceptance**: 9 differential scenarios; RL pool grows by 9 (354 → 363).
## Wave 4 — Cost & weapon interactions (8 cards) ✅ done (PR #83)

**Primitives** — all landed:
- Hand-zone cost auras: `AuraEffect::IncreaseMinionCost` (Mana Wraith — ALL
  minions +1, both players) and `IncreaseMinionCostFriendly` (Venture Co. —
  own minions only), stacked on the G5 modifier stack (`effective_cost` scans
  both players' cost-aura buckets).
- Weapon-attack cost reduction: Dread Corsair subtracts the weapon's Attack in
  `play_cost`.
- Weapon-durability damage: `CardEffect::RemoveWeaponDurability` (Bloodsail
  Corsair) — a weapon at 0 durability is destroyed.
- Weapon-equipped predicate: `AuraEffect::ChargeWithWeapon` (Southsea Deckhand)
  — `effective_charge` grants Charge while a weapon is equipped (summoning
  sickness is waived too).
- Enemy-spells-cost-0: `CardEffect::EnemySpellsCostZero` (Millhouse Manastorm)
  — a per-player `spells_cost_zero` flag read by `play_cost`, cleared at turn end.
- Give-opponent-mana: `CardEffect::GiveOpponentManaCrystal` (Arcane Golem —
  an empty crystal).

| ID | Card | Real effect | Scenario |
| --- | --- | --- | --- |
| NEUTRAL_R22 | Mana Wraith | ALL minions cost (1) more | `w4_mana_wraith_increases_all_minion_costs` |
| NEUTRAL_C14 | Venture Co. Mercenary | Your minions cost (3) more | `w4_venture_co_increases_own_minion_costs` |
| NEUTRAL_C07 | Southsea Deckhand | Has Charge while you have a weapon | `w4_southsea_deckhand_charge_with_weapon` |
| NEUTRAL_C13 | Dread Corsair | Taunt; costs (1) less per weapon Attack | `w4_dread_corsair_cost_by_weapon_attack` |
| NEUTRAL_C09 | Bloodsail Raider | Battlecry: gain Attack equal to your weapon's Attack | `w4_bloodsail_raider_gains_weapon_attack` |
| NEUTRAL_R03 | Bloodsail Corsair | Battlecry: remove 1 Durability from the opponent's weapon | `w4_bloodsail_corsair_removes_weapon_durability` + `…_destroys_1_durability_weapon` |
| LEGENDARY_021 | Millhouse Manastorm | Enemy spells cost 0 next turn | `w4_millhouse_makes_enemy_spells_free` |
| NEUTRAL_R14 | Arcane Golem | Charge; Battlecry: give your opponent a Mana Crystal | `w4_arcane_golem_gives_opponent_crystal` |

**Acceptance**: 9 differential scenarios (including stacking with existing
cost auras); RL pool grows by 8 (363 → 371).
## Wave 5 — Target structure & effect composition (7 cards) ✅ done (PR #84)

**Primitives** — all landed:
- `CardEffect::SetPlayedMinionHealth` (Repentance — the secret sets the played
  enemy minion's health to 1; resolved by the secret system with the event
  context, same pattern as Snipe).
- `CardEffect::SilenceAllEnemyMinionsAndDraw` (Mass Dispel — silence + draw
  composition).
- `CardEffect::SwapAttackAndHealth` (Crazed Alchemist — expressed as
  enchantment deltas; silencing a swapped minion reverts to the base stats).
- `CardEffect::FreezeAdjacent` (Cone of Cold — a random enemy minion and its
  left/right neighbors).
- `CardEffect::GrantAdjacentTaunt` (Sunfury Protector) and
  `GrantAdjacentSpellDamage` (Ancient Mage) — adjacent-target buffs.
- `CardEffect::FullHealAndTaunt` (Ancestral Healing — full-heal + taunt).
- New `EffectTarget::AnyMinion` (either side — Crazed Alchemist / Ancestral
  Healing target scope).

| ID | Card | Real effect | Scenario |
| --- | --- | --- | --- |
| EX1_349 | Repentance | Secret: when the opponent plays a minion, set its Health to 1 | `w5_repentance_sets_played_minion_health_to_1` |
| PRIEST_018 | Mass Dispel | Silence all enemy minions, draw a card | `w5_mass_dispel_silences_all_enemy_minions` |
| NEUTRAL_R08 | Crazed Alchemist | Swap a minion's Attack and Health | `w5_crazed_alchemist_swaps_stats` |
| MAGE_016 | Cone of Cold | Freeze a minion and its neighbors | `w5_cone_of_cold_freezes_adjacent` |
| NEUTRAL_R11 | Sunfury Protector | Give adjacent minions Taunt | `w5_sunfury_protector_taunts_adjacent` |
| NEUTRAL_R18 | Ancient Mage | Give adjacent minions Spell Damage +1 | `w5_ancient_mage_gives_adjacent_spell_damage` |
| SHAMAN_018 | Ancestral Healing | Restore a minion to full Health and give it Taunt | `w5_ancestral_healing_full_heals_and_taunts` |

**Acceptance**: 7 differential scenarios; RL pool grows by 7 (371 → 378).
## Wave 6 — Special mechanics (8 cards) ✅ done (PR #86)

**Primitives** — all landed:
- Probability: `CardEffect::ChanceDraw` (Nat Pagle — 50% draw at turn end).
- This-turn temp buff: `CardEffect::GainStatsThisTurn` (Mana Addict — the
  enchantment expires at the end of the turn).
- Mass Divine Shield: `CardEffect::GrantDivineShieldAllFriendly`
  (Righteousness).
- Self-exclusion AOE: `CardEffect::YseraAwakens` (spares Ysera herself).
- Draw-damage-by-cost: `CardEffect::DrawAndDamageByCost` (Holy Wrath).
- Damaged-friendly start-of-turn heal: `CardEffect::RestoreDamagedFriendly`
  (Lightwell — moved from end-of-turn).
- Mass buff+Taunt: `CardEffect::GainStatsAndTauntAllFriendly` (Gift of the Wild).
- Class-filtered draw: Pilfer verified already-faithful — the OtherClass pool
  filters the Rogue class group; only the stale comment was cleaned.

| ID | Card | Real effect | Scenario |
| --- | --- | --- | --- |
| LEGENDARY_022 | Nat Pagle | 50% chance to draw at the end of your turn | `w6_nat_pagle_chance_draw` |
| NEUTRAL_R10 | Mana Addict | After you cast a spell, +2 Attack this turn | `w6_mana_addict_buff_expires_at_turn_end` |
| PALADIN_018 | Righteousness | Give your minions Divine Shield | `w6_righteousness_grants_divine_shields` |
| NEUTRAL_T21e | Ysera Awakens | Deal 5 damage to all other characters | `w6_ysera_awakens_spares_ysera` |
| DRUID_016 | Gift of the Wild | Give your minions +2/+2 and Taunt | `w6_gift_of_the_wild_buffs_and_taunts` |
| PALADIN_017 | Holy Wrath | Draw a card, deal damage equal to its mana cost | `w6_holy_wrath_damages_by_drawn_cost` |
| EX1_341 | Lightwell | Start of turn: restore 3 to a damaged friendly character | `w6_lightwell_heals_at_turn_start` |
| ROGUE_025 | Pilfer | Add a random card from another class to your hand | `w6_pilfer_adds_non_rogue_card` |

**Acceptance**: 8 differential scenarios; RL pool grows by 8 (378 → 388,
including the Pilfer comment cleanup).
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
| W0 wiring ✅ PR #79 | 13 | `EventSubject` / `OtherFriendlyMinion` targets; weapon trigger registration + destroy-leaves-play; spell-cast death-before-after-cast | +13 → **334** |
| W1 race ✅ PR #80 | 11 | race field + targets/auras/triggers + field-driven pools | +11 → **346** |
| W2 triggers ✅ PR #81 | 8 | 5 trigger classes + destroy-secret | +8 → **354** |
| W3 predicates ✅ PR #82 | 9 | attack-range/hand-size/health/damaged/secret/first-minion/shield predicates | +9 → **363** |
| W4 cost/weapon ✅ PR #83 | 8 | cost auras/weapon-attack cost/durability/conditional charge/spells-0/mana gift | +8 → **371** |
| W5 target structure ✅ PR #84 | 7 | set-health/swap/adjacent targets/effect composition | +7 → **378** |
| W6 special mechanics ✅ PR #86 | 8 | probability/temp-buff/mass-shield/self-exclusion/cost-damage/class-filter | +8 → **388** |
| W7 wrap-up | 3 | 3 primitives | +3 |
| **Total** | **67** | | **321 → 388** |
