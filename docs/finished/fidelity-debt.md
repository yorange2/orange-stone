# Fidelity Debt — Simplified Cards (F4/F5 Audit Ledger)

> **Status: the ledger is EMPTY — `src/cards/` carries no `(simplified: …)` markers
> and the RL pool is the full 392 (415 card ids − 22 derivatives − coin).**
>
> How it got here: the original 67 fidelity-debt cards were cleared in W0–W7
> (W0 wiring PR #79 cleared 13; W1 race PR #80 cleared 11; W2 triggers PR #81 cleared 8;
> W3 predicates PR #82 cleared 9; W4 cost/weapon PR #83 cleared 8; W5 target structure PR #84
> cleared 7; W6 special mechanics PR #85 cleared 8; W7 wrap-up PR #86 closed the last 3).
> The 2026-08-06 registration pass then found 27 more cards implemented with known
> simplifications but **without** markers — silently in the RL pool — and registered them as
> §11 (24 after the F-A8 overload fix); §11 was cleared by W8–W12 (PRs #97–#101).
> The engine-mechanics M1 battlecry-target audit registered 20 cards as §12, cleared by
> W13–W16 (PRs #104–#107). The 2026-08-07 classic-cards status re-audit found 10 more
> deviating cards and fixed them the same day as §13 (those never entered the debt set,
> so the pool stayed at 392 throughout). F-A8 (8 duplicate card IDs) resolved (PR #88).
> This ledger is the canonical record of the F4 per-effect fidelity audit backlog.
> A card **leaves the ledger** only when its real Hearthstone effect is implemented
> **and** verified by an F5 differential test. Do not reimplement a card silently —
> update this ledger, the code comment, and the downstream debt extractor on the
> same change (see [Maintenance](#maintenance)).
>
> **2026-08-06 fix pass (PR #77)**: the structural findings below (F-A1…F-A7) are
> resolved — 4 stale comments cleaned, Worgen Infiltrator fixed, 3 card-ID
> collisions fixed, 10 cards added to `ALL_CARDS` (7 duplicated entries deduped),
> and the Python extractor rewritten (PR #31). What remains is the per-mechanism
> implementation work in the groups below and the F5 verification protocol.
>
> **2026-08-06 W0 wiring pass (PR #79)**: all 13 wiring cards landed (mechanisms
> existed; only wiring was missing) — see the W0 wave in the roadmap. Four small
> primitives came along: `EffectTarget::EventSubject` (event subject as buff
> target — Sword of Justice), `EffectTarget::OtherFriendlyMinion` ("another"
> friendly minion — Young Priestess / Master Swordsmith), weapon entities now
> register triggers and leave play when destroyed (a broken Sword of Justice
> stops firing), and spell-cast deaths resolve before "after you cast" triggers
> (a Wild Pyromancer killed by its own spell does not fire). 16 scenarios in
> `tests/differential.rs` (`w0_*`).
>
> **2026-08-06 W1 race pass (PR #80)**: all 11 race cards landed — a `CardDef.race`
> field (Beast/Murloc/Demon, applied on spawn, exposed in `EntityView`/Python
> bindings), race-conditioned targets (`FriendlyRace` / `AllOtherFriendlyRace` /
> `AnyRace`), race-conditioned auras (`FriendlyRace` / `OtherFriendlyRace`
> targets + a `GrantCharge` Charge aura — Tundra Rhino), race-conditioned
> triggers (`Trigger.race` — Murloc Tidecaller / Scavenging Hyena / Starving
> Buzzard), race-filtered deck draw (Sense Demons), and the hardcoded
> `BEAST_POOL` / `DEMON_POOL` replaced by field-driven pools (parity pinned by
> `w1_race_pools_are_field_driven`). 12 scenarios in `tests/differential.rs`
> (`w1_*`).
>
> **2026-08-06 W2 trigger pass (PR #81)**: all 8 trigger/secret cards landed —
> five new trigger classes: `CharacterHealed` (any healed character),
> `Attacked` (the entity attacks — Blessing of Wisdom attaches
> "draw when this minion attacks" to the target), `CardPlayed` (friendly
> scope), `SecretPlayed` (both players), `MinionDied` (any minion, both
> players); plus three destroy-secret effects (SI:7 — one random; Eater of
> Secrets / Flare — all, with gain-stats / draw composition). Heal triggers
> fire only on real heals (an undamaged character is not a heal event).
> 8 scenarios in `tests/differential.rs` (`w2_*`).
>
> **2026-08-06 W3 predicate pass (PR #82)**: all 9 conditional cards landed —
> attack-range targets (`EnemyMinionAttackLE` Kodo, `AnyMinionAttackGE` Big
> Game Hunter), hand-size counting (`GainStatsPerHandCard` Twilight Drake),
> hero-health threshold (`MortalStrike`), damaged-friendly/any targets
> (`DamagedFriendlyMinion` / `DamagedMinion` Rampage), damaged-counting
> (`DrawPerDamagedFriendlyCharacter` Battle Rage), owns-secret
> (`GainStatsIfOwnSecret` Ethereal Arcanist), the "first minion this turn"
> state (`AuraEffect::FirstMinionDiscount` + a per-player
> `minions_played_this_turn` counter — Pint-Sized Summoner), and divine-shield
> absorb (`AbsorbDivineShields` Blood Knight — +3/+3 per shield, both sides).
> 9 scenarios in `tests/differential.rs` (`w3_*`).
>
> **2026-08-06 W4 cost/weapon pass (PR #83)**: all 8 cost & weapon cards
> landed — hand-zone cost auras (`IncreaseMinionCost` Mana Wraith — both
> players; `IncreaseMinionCostFriendly` Venture Co. — own minions only; both
> stack on the G5 modifier stack), weapon-attack cost reduction (Dread Corsair
> in `play_cost`), weapon-durability damage (Bloodsail Corsair — a weapon at 0
> durability is destroyed), the weapon-equipped predicate (`ChargeWithWeapon`
> aura — Southsea Deckhand; `effective_charge` checks for an equipped weapon),
> enemy-spells-cost-0 (Millhouse Manastorm — a per-player `spells_cost_zero`
> flag, cleared at turn end), and give-opponent-mana (Arcane Golem — an empty
> crystal). 9 scenarios in `tests/differential.rs` (`w4_*`).
>
> **2026-08-06 W5 target-structure pass (PR #84)**: all 7 cards landed —
> `SetPlayedMinionHealth` (Repentance), `SilenceAllEnemyMinionsAndDraw`
> (Mass Dispel), `SwapAttackAndHealth` (Crazed Alchemist), `FreezeAdjacent`
> (Cone of Cold), `GrantAdjacentTaunt` (Sunfury Protector) /
> `GrantAdjacentSpellDamage` (Ancient Mage), `FullHealAndTaunt` (Ancestral
> Healing). 7 scenarios (`w5_*`).
>
> **2026-08-06 W6 special-mechanics pass (PR #85)**: all 8 cards landed —
> probability (`ChanceDraw` Nat Pagle), this-turn temp buff
> (`GainStatsThisTurn` Mana Addict), mass Divine Shield
> (`GrantDivineShieldAllFriendly` Righteousness), self-exclusion AOE
> (`YseraAwakens` — spares Ysera), draw-damage-by-cost (`DrawAndDamageByCost`
> Holy Wrath), damaged-friendly start-of-turn heal (`RestoreDamagedFriendly`
> Lightwell), mass buff+Taunt (`GainStatsAndTauntAllFriendly` Gift of the
> Wild). Pilfer was *claimed* verified already-faithful in W6, but the
> OtherClass filter (`!ROGUE_CLASSIC`) also admitted every neutral card;
> **corrected 2026-08-06** — the pool is now exactly the other eight classes'
> class cards (`is_other_class_card`, pinned by a `pool.rs` unit test +
> strengthened `w6_pilfer_adds_other_class_card`). 8 scenarios (`w6_*`).
>
> **2026-08-06 W7 wrap-up pass (PR #86)**: the last 3 cards landed — hand-zone
> swap (`SwapWithHandMinion` Alarm-o-Bot), the damage-reflection secret
> (`ReflectDamage` Eye for an Eye — new `SecretTrigger::WhenFriendlyHeroDamaged`),
> and the 1-Health resummon secret (`ResurrectDiedMinion` Redemption).
> 3 scenarios (`w7_*`). **The ledger was EMPTY and the RL pool was the full 391-card
> classic pool — until the 2026-08-06 registration pass re-opened it with 27
> pre-existing simplifications (§11); the pool is now 391 (413 − 0 debt − 21 tokens − coin,
> W8 clears 5: Amani Berserker / Raging Worgen / Grommash / Warsong Commander / Northshire Cleric;
> W9 clears 4: Truesilver Champion / Gorehowl / Eaglehorn Bow / Bestial Wrath;
> W10 clears 5: Wrath / Druid of the Claw / Ancient of Lore / Ancient of War / Tracking;
> W11 clears 5: Onyxia / Deathwing / Sea Giant / Preparation / Cold Blood — Defender of Argus
> was fixed in the same wave but was never registered in §11, so the pool gains 5, not 6;
> W12 clears the last 5: Water Elemental / Cabal Shadow Priest / Prophet Velen / Shiv /
> Argent Protector (the roadmap's wave accounting swapped Argent for the unregistered
> Argus; the ledger is now EMPTY and the pool is the full 391-card classic pool).**
>
> **Execution plan**: [docs/fidelity-debt-roadmap.md](../finished/fidelity-debt-roadmap.md)
> (zh: `finished/fidelity-debt-roadmap-zh.md`) — 8 dependency-ordered waves (W0 wiring …
> W7 wrap-up) covering all 67 cards; a card is done when its ledger row, its code
> comment, and its differential scenario all land together.

**Source of truth**: the `(simplified: ...)` doc comments on card consts in
`src/cards/classic_*.rs`. The Python side
(`orange-reinforcement/hearthstone_os/decks.py::_load_debt_ids`) extracts this set
by parsing those comments and excludes it from the RL training pool — so comment
wording changes ripple into the training pool (invalidate
`~/.cache/orange_stone_debt_ids.txt` after any edit here).

The 67 markers are 67 unique card IDs — the 3 pre-fix ID collisions
(Sword of Justice / Repentance / Lightwell) were resolved to their official IDs
(EX1_365 / EX1_349 / EX1_341), so Mass Dispel is now reachable. All 67 are in
`ALL_CARDS` (413 unique entries after the 10-card addition and the 7-entry dedup,
PR #77). 4 markers were stale comments on already-faithful cards and are cleaned
(§10); the genuine debt is 67 cards — **all 67 cleared (W0 13 + W1 11 +
W2 8 + W3 9 + W4 8 + W5 7 + W6 8 + W7 3); the ledger was EMPTY after the
2026-08-06 W7 wrap-up PR #86, then re-opened with 27 registered simplifications
(§11) on the same day**.

---

## The ledger (grouped by missing mechanism)

### 1. Enrage — damage-conditional buff (4) ✅ resolved (W0, PR #79)

All 4 wired to the existing `ThisMinionDamaged` trigger slot (registered per card
ID in `apply_card_keywords`, same pattern as Acolyte of Pain); the buff is a
permanent enchantment. Scenarios: `w0_gurubashi_berserker_enrage_permanent`,
`w0_tauren_warrior_enrage_with_taunt`, `w0_angry_chicken_enrage_fires_before_death`,
`w0_spiteful_smith_buffs_weapon_on_damage`.

### 2. Tribes — Beast / Murloc / Demon (9) ✅ resolved (W1, PR #80)

All landed: `CardDef.race` + race-conditioned targets/auras/triggers +
race-filtered deck draw + field-driven pools (`w1_*` scenarios; pool parity
pinned by `w1_race_pools_are_field_driven`). The F-A6-annotated Starving
Buzzard (HUNTER_013) and Scavenging Hyena (HUNTER_014) also left the ledger. 

### 3. Event triggers — summon / heal / death / secret / attack / play (2) ✅ resolved (W7, PR #86)

Alarm-o-Bot (hand-zone swap) landed in W7; Ethereal Arcanist (owns-secret
predicate) landed in W3 — this section is empty
(`w7_alarm_o_bot_swaps_with_hand_minion`).

### 4. Conditional targets & states (9) ✅ resolved (W3, PR #82)

All landed: attack-range (≤2 / ≥7), hand-size, hero-health threshold,
damaged-friendly/any targets, damaged-counting, owns-secret, the "first
minion this turn" state, and divine-shield absorb (`w3_*` scenarios).

### 5. Adjacent / multi-target (4) ✅ resolved (W5, PR #84)

All landed: adjacent-target Freeze (Cone of Cold), adjacent-target buffs
(Sunfury Protector Taunt / Ancient Mage Spell Damage), swap attack/health
(Crazed Alchemist) (`w5_*` scenarios).

### 6. Cost & weapon-condition auras (8) ✅ resolved (W4, PR #83)

All landed: hand-zone cost auras (global / own), weapon-attack cost reduction,
weapon-durability damage, weapon-equipped predicate (conditional Charge),
enemy-spells-cost-0, give-opponent-mana (`w4_*` scenarios).

### 7. This-turn temporary buff (1) ✅ resolved (W6, PR #85)

`GainStatsThisTurn` — the enchantment expires at the end of the turn
(`w6_mana_addict_buff_expires_at_turn_end`).

### 8. Probability (1) ✅ resolved (W6, PR #85)

`ChanceDraw` — a percentage draw at the end of the turn
(`w6_nat_pagle_chance_draw`).

### 9. Composite & miscellaneous (8) ✅ resolved (W5 PR #84 + W6 PR #85)

W5 cleared: Holy Wrath (draw-damage-by-cost), Ancestral Healing
(full-heal + Taunt). W6 cleared: Righteousness (mass Divine Shield), Ysera
Awakens (self-exclusion AOE), Lightwell (damaged-friendly start-of-turn
heal), Gift of the Wild (mass buff+Taunt), Pilfer (class-filtered draw —
W6 marked it verified already-faithful; the 2026-08-06 correction narrowed
the pool from "any non-Rogue" to the other eight classes' class cards).
Scenarios `w5_*` / `w6_*`.

### 10. Resolved — marked simplified but already faithful (4, cleaned in PR #77)

Comments predated the implementing PRs (PR #72 Stealth for the first three;
Multi-Shot's `DealDamageToTwo` lands with the two-random-target effect). The defs
already carried the real effect; the stale "simplified" wording has been dropped,
which also removes them from the Python debt set (4 cards re-enter the RL pool).

| ID | Card | What was fixed |
| --- | --- | --- |
| NEUTRAL_C10 | Jungle Panther | comment cleaned — `stealth: true` in def |
| NEUTRAL_T14 | Stranglethorn Tiger | comment cleaned — `stealth: true` in def |
| NEUTRAL_T15 | Ravenholdt Assassin | comment cleaned — `stealth: true` in def |
| HUNTER_012 | Multi-Shot | comment cleaned — `DealDamageToTwo` is the real effect |

---


### 11. Pre-existing simplifications registered 2026-08-06 (27 → 24 → 19 → 15 → 10 → 5 → 0 cards) ✅ EMPTY

The 2026-08-06 status audit (`docs/classic-cards-zh.md` vs. the code) found 27
cards with known simplifications that carried no `(simplified: …)` marker — they
were silently in the RL pool. All 27 defs now have comments (debt set = 27);
cards leave the ledger only via the [Maintenance](#maintenance) flow
(implement → F5 differential → drop comment → invalidate cache). W8 (roadmap
PR #97) cleared 5: Amani Berserker / Raging Worgen / Grommash Hellscream /
Warsong Commander / Northshire Cleric; W9 (PR #98) cleared 4: Truesilver
Champion / Gorehowl / Eaglehorn Bow / Bestial Wrath; W10 (PR #99) cleared 5:
Wrath / Druid of the Claw / Ancient of Lore / Ancient of War / Tracking; W11
(PR #100) cleared 5: Onyxia / Deathwing / Sea Giant / Preparation / Cold Blood;
W12 (PR #101) cleared the last 5: Water Elemental / Cabal Shadow Priest /
Prophet Velen / Shiv / Argent Protector — **the ledger is EMPTY and the RL
pool is the full 391-card classic pool**.

### 12. Battlecry target sets — M1 audit (20 → 16 → 9 → 2 → 0 cards) ✅ EMPTY

The engine-mechanics roadmap M1 audit (minion battlecry explicit targets)
walked every minion battlecry with a target-bearing effect and checked the
engine's `EffectTarget` against real HS. 20 cards did not match and were
registered here per the maintenance contract (each carried a
`(simplified: …)` comment in `src/cards/`; the RL pool dropped to 371 until
they were cleared). The wrong-scope rows were cleared in W13 (PR #104) and
W14 (PR #105), the `Self_`-modeled rows in W15 (PR #106), and the
effect-shape rows in W16 (PR #107). §12 is now empty — the RL pool is back
to the full 392 (415 card ids − 22 derivatives − 1 coin; the roadmap's 391
was measured on a stale 414-card wheel, see PR #104).

*(20 rows registered 2026-08-07 by the engine-mechanics M1 audit; all
cleared 2026-08-07 by the battlecry-target-debt roadmap W13–W16)*

### 13. classic-cards status re-audit — 2026-08-07 (10 cards) ✅ EMPTY

Re-auditing `docs/classic-cards.md` against the code found 10 cards whose
implementations still deviated from real Hearthstone. None carried a
`(simplified: …)` marker and none were registered here, so none had ever been
excluded from the RL pool — the pool stayed at 392 throughout. All 10 were
fixed the same day rather than registered as debt.

| ID | Card | Was | Now |
| --- | --- | --- | --- |
| CLASSIC_018 | Amani Berserker | permanent stacking +3 | conditional Enrage |
| NEUTRAL_008 | Raging Worgen | permanent stacking +1 & Windfury | conditional Enrage |
| NEUTRAL_C11 | Tauren Warrior | permanent stacking +3 | conditional Enrage |
| NEUTRAL_C15 | Spiteful Smith | permanent weapon +2 | conditional Enrage |
| NEUTRAL_R02 | Angry Chicken | permanent stacking +5 | conditional Enrage |
| WARRIOR_010 | Grommash Hellscream | permanent stacking +6 | conditional Enrage |
| WARRIOR_008 | Warsong Commander | Charge aura on all other friendlies | on-summon trigger, Attack ≤ 3 |
| PRIEST_004 | Northshire Cleric | friendly *character* heals | any minion, either side |
| LEGENDARY_010 | Onyxia | fixed 5 Whelps | fills the board (6 from empty) |
| PRIEST_012 | Prophet Velen | +1 Spell Damage (a no-op) | doubles spell/hero-power damage & healing |

Engine work behind those fixes:

- **`Enrage` component** (`core/component.rs`) carrying an attack bonus, an
  optional Windfury, and an optional weapon-attack bonus. It is resolved on read
  in `World::effective_attack` and `World::max_attacks` rather than written into
  an enchantment — that is what makes it non-stacking, makes it end the moment
  the minion is healed to full, and lets Silence strip it. Gurubashi Berserker
  keeps the old `ThisMinionDamaged` trigger because its real text genuinely is a
  permanent stacking buff. `compute_attacker_damage` now reads the weapon through
  `effective_attack` so Spiteful Smith's bonus reaches the hero's swing.
- **`Trigger.max_attack`** — an attack ceiling on a trigger's event subject,
  alongside the existing `race` condition. Warsong Commander uses it; the Charge
  is granted once, to that minion, so it outlives the commander.
- **`TriggerEvent::MinionHealed`** — replaces `FriendlyCharacterHealed` (which
  only Northshire Cleric used). Global scope, minions only.
- **Spell Damage pipeline.** `World::total_spell_damage` existed but had **zero
  callers**: Spell Damage was stored on the board and never applied to any
  damage, so every Spell Damage card in the set was effectively vanilla
  (Kobold Geomancer, Dalaran Mage, Ogre Magi, Archmage, Bloodmage Thalnos,
  Azure Drake, Malygos, Ancient Mage), and Velen's "+1 Spell Damage" rebalance
  did nothing at all. `trigger::apply_spell_power` now rewrites damage and
  healing amounts for effects whose source is a spell or hero power — Spell
  Damage adds first, then Velen doubles, matching HS (Mind Blast 5 → 6 with a
  Geomancer → 10 with Velen → 12 with both). Attacks, battlecries, and
  deathrattles are untouched, which also narrowed Velen's heal doubling: it used
  to double *every* heal the owner produced, including a Voodoo Doctor battlecry.

F5 coverage: `w8_amani_berserker_enrage`, `enrage_does_not_stack_and_ends_at_full_health`,
`enrage_is_removed_by_silence`, `w8_raging_worgen_enrage_and_windfury`,
`w0_spiteful_smith_buffs_weapon_while_damaged`, `w8_grommash_hellscream_enrage`,
`w8_warsong_commander_charges_only_small_summons`,
`warsong_charge_outlives_the_commander`,
`w8_northshire_cleric_draws_on_any_minion_heal`,
`northshire_cleric_ignores_a_heal_that_restores_nothing`,
`w11_onyxia_fills_the_board_with_whelps`, `spell_damage_boosts_spell_damage`,
`spell_damage_does_not_boost_battlecry_damage`,
`prophet_velen_doubles_spell_damage_after_spell_damage`,
`prophet_velen_leaves_minion_effects_alone`. Full `cargo test` green (488);
`cargo clippy --all-targets` clean; `cargo bench` shows no change on the touched
paths (`effective_stats/aura_board_14_minions`, `effect_resolution/*`, all
p > 0.05).

## Findings from the 2026-08-06 audit pass (all resolved in PR #77 / PR #31)

These were F4 work items discovered while compiling the ledger. All structural
findings are resolved; what remains is the per-mechanism implementation work in the
groups above plus the F5 verification protocol.

- ✅ **F-A1 — 4 stale comments** (§10 above). Resolved: "(simplified…)" dropped from
  the comments; the Python extractor stops excluding the cards (cache invalidated);
  the RL pool gains 4 cards (3 stealth minions + Multi-Shot).
- ✅ **F-A2 — Worgen Infiltrator (NEUTRAL_C08)**: was still vanilla though Stealth
  exists. Resolved: hand-written def with `stealth: true`.
- ✅ **F-A3 — Card-ID collisions (3)**:
  - `SWORD_OF_JUSTICE` used `"PALADIN_017"` (= Holy Wrath) → now EX1_365
  - `REPENTANCE` used `"PALADIN_018"` (= Righteousness) → now EX1_349
  - `LIGHTWELL` used `"PRIEST_018"` (= Mass Dispel) → now EX1_341; **Mass Dispel is
    now reachable** (previously `card_by_id("PRIEST_018")` resolved to Lightwell).
- ✅ **F-A4 — 10 cards absent from `ALL_CARDS`** (Houndmaster, Tundra Rhino, Flare,
  Sword of Justice, Repentance, Blessing of Wisdom, Scavenging Hyena, Starving
  Buzzard, Eye for an Eye, Redemption). Resolved as *omission*: all 10 added to
  `ALL_CARDS` (413 unique entries). The RL pool is unaffected — the 6 simplified
  cards and the 4 newly-documented ones (F-A6) are all excluded by the debt set.
- ✅ **F-A5 — `ALL_CARDS` had 7 duplicated const entries** (410 entries, 403 unique).
  Resolved: deduped to 413 unique entries (403 + 10 additions).
- ✅ **F-A6 — Undocumented simplifications**: Starving Buzzard (HUNTER_013) and
  Scavenging Hyena (HUNTER_014) were tribe-simplified ("summon a minion" / "a
  friendly minion dies" vs. real "**Beast**") without a marker; Eye for an Eye and
  Redemption (PALADIN_020/021) are secret triggers with no effect wired. Resolved:
  all four now carry explicit `(simplified: …)` comments, so they are documented
  debt and stay out of the RL pool.
- ✅ **F-A7 — Python debt extractor was misattributing cards** (fixed in PR #31):
  `hearthstone_os/decks.py::_load_debt_ids` split the source on `pub const` blocks,
  so each `simplified` comment was matched to the *preceding* const's ID. The RL
  training pool therefore contained ~12 real simplified cards (Nat Pagle, Multi-Shot,
  Secretkeeper, Mass Dispel, Rampage, Cone of Cold, Ancestral Healing, …) and
  excluded ~15 clean ones; the 321-pool size was coincidental. Rewritten to resolve
  the comment to the const directly below it. Cache `~/.cache/orange_stone_debt_ids.txt`
  invalidated; M5's training numbers were produced on the pre-fix pool and will
  drift once re-trained.


## F-A8 — 8 duplicate card IDs + mis-wired Overload map ✅ resolved (PR #88, 2026-08-06)

F-A3 fixed 3 ID collisions; this pass found 8 more. `card_by_id` resolves the
first match, so one card of each pair is unreachable by ID, and the Overload
wiring (`cards/mod.rs::apply_card_keywords`, keyed by raw IDs) is mis-wired:

| ID | Card A | Card B |
| --- | --- | --- |
| MAGE_016 | Cone of Cold | Mana Wyrm |
| PALADIN_016 | Argent Protector | Blessed Champion |
| PRIEST_016 | Radiance | Divine Spirit |
| PRIEST_017 | Kul Tiran Chaplain | Inner Fire |
| SHAMAN_016 | Forked Lightning | Windfury |
| SHAMAN_017 | Lava Burst | Windspeaker |
| SHAMAN_018 | Stormforged Axe | Ancestral Healing |
| SHAMAN_019 | Earth Elemental | Ancestral Spirit |

Overload consequences today: Lightning Bolt / Lightning Storm / Feral Spirit /
Dust Devil / Lava Burst / Earth Elemental receive the correct amount *by luck*
(their IDs happen to line up with the match), Forked Lightning gets 1 instead of
2 (registered §11), Windfury / Windspeaker / Ancestral Spirit wrongly gain
Overload, Stormforged Axe / Doomhammer get none (registered §11). The match's
comments are stale (naming Feral Spirit / Forked Lightning / Lightning Storm /
Totem Golem — the last one is not in the card set at all).

Fix applied (PR #88): one card of each pair renumbered to its real HS ID
(Mana Wyrm CS2_027, Blessed Champion CS2_089, Divine Spirit CS2_235, Inner Fire
CS1_129, Windfury CS2_039, Windspeaker CS2_041, Ancestral Healing CS2_003,
Ancestral Spirit CS2_289 — no duplicate IDs remain); the Overload match is
re-wired by actual IDs with correct amounts (Forked Lightning 2, Stormforged
Axe 1, Doomhammer 2 — phantom Windfury / Windspeaker / Ancestral Spirit entries
gone with the renumber); F5 differential scenes `f8_*` pin the new behavior;
§11 rows for the three now-fixed overload cards removed (24 remain). The RL
side references none of the touched IDs — no RL deck config changes. Full
`cargo test` green (402 tests).

## F-A9 — first player missing the turn-1 draw ✅ resolved (official rule, 2026-08-06)

Official Hearthstone rule (Blizzard, *Opening Moves: Mulligans*): the first
player draws the 4th card as their turn 1 starts; the second player draws on
their own first turn too. `Step::DrawStep` carried a turn-1 guard that skipped
the first player's draw (`src/engine/rules.rs`), and a unit test
(`first_player_does_not_draw_on_turn_one`) pinned the wrong behavior; the
shipped opening (`sim::battle::build_game_state`) dealt the first player only
`hand_size` cards, so P1 opened at 3 instead of the official 4 — inflating the
second player's advantage (parity: P1 winrate ~23% vs the simplified engine's
~34%, which already drew the turn-1 card).

Fix applied: `build_game_state` deals `hand_size + 1` to the first player
(opening + turn-1 draw); the DrawStep turn-1 guard is removed (a state entering
DrawStep on turn 1 draws normally); the `begin_game`/mulligan flow draws the
4th card when the opening finishes (both mulligans resolved). Tests re-pinned:
`first_player_draws_on_turn_one`, `second_player_draws_on_first_turn_first_player_from_turn_two`,
the mulligan-completion hand size, and the env opening-shape tests (default
4/3, `hand_size 4` → 5/4, with coin → 4/5). Parity re-measured (48 seeds):
P1 winrate os 23% → 46% (简版 33%), same-seed agreement 58%. Full
`cargo test` green (405 tests).

## F-A10 — env step-limit draw never fired for EndTurn-only stalls ✅ resolved (2026-08-06)

The engine has no fatigue yet (`trigger.rs` — "fatigue in Phase 3+"), so a game
that drains both decks stalls forever; the env bounds it with `max_steps`
(default 5000, ends the episode in a draw). But the limit check sat in an
`else if` chain *after* the EndTurn branch — in a stall state every action is
EndTurn, so the check was unreachable and the episode ran forever
(`test_batched_matches_single_per_seed`, seed 3). Second flaw: the structured
observation's `done` flag was derived from `state.step() == GameOver`, while a
limit draw leaves the step machine in Main — so even a correctly-flagged env
never surfaced `done` to Python.

Fix applied: the limit check now runs before the EndTurn branch
(`rl/env.rs::step`); `GameEnv::is_done()` returns the env's episode flag
instead of re-deriving it from the step; `rl::views::observation` takes the
done flag as a parameter (the state alone cannot express a limit draw).
Pinned by `step_limit_ends_end_turn_stall_in_draw`. Both `GameEnv` and
`BatchEnv` structured observations now report limit draws as done.

**Fatigue closure (2026-08-06, roadmap PR #94)**: the underlying gap is
closed — the engine implements official fatigue (an empty-deck draw attempt
deals 1, 2, 3, … damage to the drawing hero;
`docs/finished/fatigue-roadmap.md`). Deck-draining games now end with a real
winner; `max_steps` / `max_turns` are demoted to backstops (only an
armor/heal loop can outlive fatigue).

## F-A11 — no hand-size cap, no burn ✅ resolved (2026-08-07, pool-open-cards roadmap M5)

The engine has no 10-card hand limit: `add_card_to_hand` and
`draw_card_no_queue` append unconditionally (the fatigue roadmap explicitly
deferred this gap here, `docs/finished/fatigue-roadmap.md` — "Hand-size burn
… a separate pre-existing gap"). The pool-open cards (Lorewalker Cho,
Thoughtsteal, Mind Vision) make overfull hands routine rather than rare, and
`rl/obs.rs` truncates the hand at `MAX_HAND = 10` — cards past the tenth
become invisible to the agent while still being playable, a silent
observation/action mismatch.

Fix applied: hands cap at 10. A **drawn** card over the cap is burned
(destroyed — sent to the graveyard, still counts as drawn for deck
depletion); a **generated** card over the cap is never created
(`add_card_to_hand` refuses). All generation paths (Pilfer, Antonidas,
Cho, Thoughtsteal, Mind Vision, random pools) route through the same two
functions, so the cap is central. Pinned by `po_*` hand-cap scenarios in
`tests/differential.rs`.

## Mechanism inventory (what the engine has vs. what's missing)

**Exists** (so the corresponding cards are mostly *wiring* work):
`summon_trigger` / `spell_trigger` / `death_trigger` / `start|end_turn_effect` /
`aura` (`AuraTarget` incl. OtherFriendlyMinions) / cost-modifier stack (G5) /
`SilenceMinion`+`AllEnemyMinions` / `FreezeCharacter` / `FullHeal` / `DestroyMinion`
targets incl. `DamagedEnemyMinion` / `DealDamageToTwo` / `DestroyAdjacent` /
`GrantCharge` / Stealth / Elusive / Secrets / `ThisMinionDamaged` (Enrage —
W0 wired all 4) / Poison component (W0 — Emperor Cobra) /
`EffectTarget::EventSubject` + `OtherFriendlyMinion` (W0) / weapon triggers
register + destroyed weapons leave play (W0) / spell-cast deaths resolve before
"after you cast" triggers (W0) / `CardDef.race` + race-conditioned targets
(`FriendlyRace`/`AllOtherFriendlyRace`/`AnyRace`) + race-conditioned auras
(`GrantCharge` Charge aura) + `Trigger.race` + field-driven race pools (W1) /
trigger classes complete: `CharacterHealed`, `Attacked`, `CardPlayed`,
`SecretPlayed`, `MinionDied` (any) + destroy-secret effects (W2) /
hand-zone cost auras (`IncreaseMinionCost` / `IncreaseMinionCostFriendly`) +
weapon-attack cost reduction + weapon-durability damage + `ChargeWithWeapon`
conditional Charge + enemy-spells-cost-0 + give-opponent-mana (W4) /
set-health-to-1 (Repentance), swap attack/health (Crazed Alchemist),
adjacent-target buff/freeze (Sunfury Protector / Ancient Mage / Cone of Cold),
two-effect composition (Mass Dispel, Ancestral Healing) (W5).

**Missing** (2026-08-06 refresh — the 27 registered debts in §11, grouped;
W8 cleared groups 1–3, W9 cleared group 4):
1. ~~Enrage wiring (Amani Berserker, Raging Worgen, Grommash — `ThisMinionDamaged`
   exists, pure wiring per W0)~~ ✅ W8
2. ~~Charge aura (Warsong Commander)~~ ✅ W8
3. ~~Heal-draw trigger (Northshire Cleric)~~ ✅ W8
4. ~~Weapon attack effects (Truesilver heal, Gorehowl attack-loss, Eaglehorn
   Durability)~~ ✅ W9
5. ~~Discover (Tracking); Choose One second branches (Wrath, Druid of the Claw,
   Ancient of Lore, Ancient of War)~~ ✅ W10
6. ~~Battlecries (Onyxia, Deathwing hand discard + all-other-destroy);
   freeze-on-damage (Water Elemental); control (Cabal Shadow Priest); healing
   doubling (Prophet Velen); Shiv's 1-damage draw-1; Argent Protector
   battlecry Divine Shield~~ ✅ W11/W12 (W11 also fixed Defender of Argus's
   adjacent buff, an unregistered silent misimplementation)
7. ~~Cost reduction (Sea Giant, Preparation); Combo base branch (Cold Blood)~~ ✅ W11

## F5 verification per fix

Every card that leaves this ledger must land with:
1. a scenario in `tests/differential.rs` pinning the exact real-HS sequencing
   (target sets, trigger timing, death-phase interplay), and
2. where the semantics can be mirrored, a deck-level parity check against
   SabberStone following `docs/differential_sabberstone.md`.

## Maintenance

- **Adding cards**: a new `(simplified …)` comment in `src/cards/` requires a row
  in this ledger (and vice versa). The RL training pool excludes this set
  (`hearthstone_os/decks.py`), so pool membership changes go through the ledger.
- **Fixing cards**: implement → F5 differential test → remove the ledger row →
  drop the code comment → invalidate `~/.cache/orange_stone_debt_ids.txt`
  (the Python extractor caches the parsed set).
- **Comment wording**: the Python extractor keys on the word "simplified" in the
  card's doc block. Use "simplified" only for genuine debt; use "(verified)" or
  plain text once faithful.
- **Pool-open ≠ debt**: the four pool-open cards (Mind Vision, Thoughtsteal,
  Mindgames, Lorewalker Cho) carry `(pool-open: …)` comments, not
  `(simplified: …)` — they are faithful implementations that read the
  opponent's cards, not simplifications. The extractor keys on "simplified",
  so they stay in the RL pool; their registry lives in `pool-openness.md`.
