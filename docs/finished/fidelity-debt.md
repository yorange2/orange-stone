# Fidelity Debt — Simplified Cards (F4/F5 Audit Ledger)

> **Status: the classic-side ledger is EMPTY — `src/cards/` carries no
> `(simplified: …)` markers on classic cards and the RL pool is the full 392
> (415 card ids − 22 derivatives − coin).** The only non-empty section is
> **§14 (2025–2026 expansions)**: the four M1-W1 Emerald Dream simplifications
> registered there are the first entries since §13; they cover the new
> handwritten expansion cards, which are not part of the RL pool.
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

### 14. 2025–2026 expansions M1-W1 — the Emerald Dream imbue wave (5 cards) 🔓 registered

The four registered simplifications of the M1-W1 wave (15 cards + 3 tokens in
`src/cards/exp_edr_w1.rs`, 2025-2026-expansions-roadmap M1). These handwritten
expansion cards are not part of the RL pool (classic + core 668/659), so the
rows are informational: they keep the code's `(simplified: …)` markers
traceable to the ledger. Each row stays open until its mechanism lands in a
later wave (W3 brings the real choice/discover pipeline).

| ID | Card | Simplified | When real |
| --- | --- | --- | --- |
| EDR_845 | Hamuul Runetotem | Nature school check skipped (every friendly spell qualifies); the Start-of-Game part fires on play (the engine has no StartOfGame event) and the every-3-spells trigger fires only while Hamuul is in play (the per-player counter survives him leaving play) | a StartOfGame event + a school check on the spell entity |
| EDR_449p | Blessing of the Moon (Priest skill) | "Choose a playable Priest minion or spell" → a random pick over the PriestCard pool | the real choice mechanism (W3) |
| EDR_445pt3 | Emerald Portal (Paladin skill token) | "Casts When Drawn" not modeled → a playable 0-cost spell that summons a random 1-Cost Dragon when played; the dragon pool spans the expansion baselines (END_022 / CATA_484 / CATA_556) because the active Classic/Core window has no 1-Cost dragons | a cast-when-drawn pipeline |
| EDR_888 | Malorne the Waywatcher | Discover → the existing random simplification over the fixed WILD_GOD_POOL (the 8 EDR Wild Gods) | the real Discover pipeline (W3) |
| EDR_970 | Kaldorei Priestess | "until your next turn" → the TempDebuff UntilEndOfTurn expiry (established engine precedent — Scarlet Subjugator); the debuff expires at the active turn's wrap-up | a next-turn expiry variant |

中文小结（同四行）：Hamuul 的 Nature 学派检查跳过（所有友方法术合格），且
"开局"部分改为打出时触发、每三张法术的触发仅在 Hamuul 在场时生效；牧师
英雄技能"选择一张可用的牧师随从或法术"简化为从 PriestCard 池随机取一张；
圣骑士的翡翠传送门未建模 Casts When Drawn，改为可打出的 0 费法术（打出时
召唤随机 1 费龙，龙池跨扩展基线——当前经典/核心窗口内没有 1 费龙）；
Malorne 的发现沿用既有随机简化（固定 WILD_GOD_POOL 八位荒野之神）；
Kaldorei 祭司的"直到你的下回合"沿用既有 TempDebuff 先例（本回合结束移除，
与 Scarlet Subjugator 一致）。

F5 coverage: `edr_w1_imbue_threshold_sequence`,
`edr_w1_druid_golem_scales_with_level`, `edr_w1_mage_wisp_damage_scales_with_level`,
`edr_w1_paladin_portals_shuffled_into_deck`, `edr_w1_emerald_portal_playable_summons_dragon`,
`edr_w1_priest_moon_random_priest_card_reduced`, `edr_w1_shaman_wind_transforms_to_cost_plus_level`,
`edr_w1_wisprider_imbues_then_triggers`, `edr_w1_dreamweaver_requires_two_imbues`,
`edr_w1_malorne_wild_god_cost_threshold`, `edr_w1_warrior_counts_without_replacement`,
`edr_w1_hamuul_imbues_every_third_spell`, `edr_w1_kaldorei_priestess_debuffs_enemy_attack`
(13 scenarios in `tests/differential.rs`). Full `cargo test` green; `cargo clippy
--all-targets` clean.

### 14.1 2025–2026 expansions M1-W2 — the Emerald Dream dark gifts (9 cards) 🔓 registered

The registered simplifications of the M1-W2 wave (`src/cards/exp_edr_w2.rs` + the
dark-gift engine: `DarkGiftKind`, `ApplyDarkGift`, the per-entity `dark_gifts`
marker, `Player.dark_gifts_given`). As with §14, these handwritten expansion
cards are not in the RL pool, so the rows are informational: they keep the
code's `(simplified: …)` markers traceable to the ledger. Each row stays open
until its mechanism lands in a later wave (W3 brings the real choice/discover
pipeline).

| ID | Card | Simplified | When real |
| --- | --- | --- | --- |
| EDR_102 | Treacherous Tormentor | Discover a Legendary minion → a random pick over the in-window Legendary pool (25 cards), then one random dark gift | the real Discover pipeline (W3) |
| EDR_456 | Darkrider | Discover a Dragon → a random pick over the in-window Dragon pool (9); the holding-Dragon hand condition is enforced (as written) | the real Discover pipeline (W3) |
| EDR_487 | Wallow, the Wretched | The log (`Player.dark_gifts_given`) records gift kinds only, not the source cards; Wallow copies gifts non-retroactively (only gifts given while it is in hand/deck) and never re-logs the copies | a full per-gift source log |
| EDR_488 | Avant-Gardening | Discover a Deathrattle minion → a random pick over the in-window Deathrattle pool (29), then one random dark gift | the real Discover pipeline (W3) |
| EDR_528 | Nightmare Fuel | Discover a copy of a minion from the opponent's deck → a random pick over the actual enemy deck (pool-open card — registered in `POOL_OPEN_CARDS`); the copy is a freshly generated card (no enchantments) | a real Discover pipeline + a copy-that-copies-enchantments pipeline |
| EDR_654 | Overgrown Horror | faithful | — |
| EDR_811 | Rite of Atrocity | Discover an Undead minion → a random pick over the in-window Undead pool (17), then one random dark gift if you spend 2 corpses (exactly as written — no gift without the corpses) | the real Discover pipeline (W3) |
| EDR_856 | Nightmare Lord Xavius | Discover a minion from your own deck → a random pick over the actual friendly deck, then one random dark gift (own deck = in-pool, not pool-open) | the real Discover pipeline (W3) |
| EDR_882 | Jumpscare! | Discover an expensive Demon → a random pick over the in-window Demon-costing-5+ pool (12); the official "shuffle the other two options into your deck" clause is moot under the random Discover simplification and is skipped | the real Discover pipeline (W3) |

Dark-gift simplifications (the ten gifts in `ALL_DARK_GIFTS`): gift 3 ("costs
(2) less, attack -2, attack can't go below 1") skips the attack floor — attack
is simply reduced by 2; gift 6 ("if it has a battlecry, it triggers twice")
skips the has-a-battlecry eligibility filter — the gift re-resolves whatever
effect the minion carries; gift 9 (place on top of the deck with +4/+5) moves
the minion to the deck BEFORE the enchantment lands (move-before-buff), so a
play-zone target's enchantments are wiped by the zone bounce per the engine's
established move_to_zone convention; gift 5 ("when you play this, summon a 2/2
copy") summons the copy through `resolve_summon`, so the copy's battlecry fires
per the engine's summon convention; the gift log is kinds-only and the gifts
ride a per-entity marker (`World::dark_gifts`) that persists across zones, so a
gifted card keeps its gifts deck → hand → play, and Wallow's sync and
Overgrown Horror's discount read the same marker.

中文小结（同上十行 + 赠礼简化）：七张发现卡（Tormentor / Darkrider /
Avant-Gardening / Nightmare Fuel / Rite of Atrocity / Xavius / Jumpscare）的
发现统一简化为随机取一（Discover 管线 W3 落地）；发现池按当前经典/核心窗口
实测 25 张传说 / 9 张龙 / 29 张亡语 / 17 张亡灵 / 12 张费用≥5 恶魔，Nightmare
Fuel 与 Xavius 直接从真实牌库随机取（前者从对手牌库复制——已登记
POOL_OPEN_CARDS——复制品为全新生成卡、不带附魔，后者从己方牌库取走并附赠礼）；
Jumpscare "把另外两张洗回牌库"在随机发现下无意义、不实现；赠礼 3 跳过
"攻击力不低于 1"的下限过滤（直接 -2 攻击）；赠礼 6 跳过"须有战吼"的资格
过滤（无条件二次结算效果）；赠礼 9 先移入牌库再附加 +4/+5（move-before-buff，
场上目标的附魔按既有 move_to_zone 弹回约定被清除）；赠礼 5 的 2/2 复制经
`resolve_summon` 召唤、其战吼按引擎召唤惯例触发；赠礼日志只记种类、Wallow
的同步不回溯且不重复记日志；赠礼以实体标记（`World::dark_gifts`）随身携带、
跨区域保留，Wallow 同步与 Overgrown Horror 减费均读取该标记。

F5 coverage: `edr_w2_treacherous_tormentor_legendary_gift_attack_lifesteal`,
`edr_w2_avant_gardening_deathrattle_gift_cost_discount`,
`edr_w2_jumpscare_demon_gift_shield_windfury`,
`edr_w2_rite_of_atrocity_corpses_gift_charge`,
`edr_w2_rite_of_atrocity_no_corpses_undead_ungifted`,
`edr_w2_nightmare_fuel_combo_gift_health_taunt`,
`edr_w2_nightmare_fuel_without_combo_copy_ungifted`,
`edr_w2_darkrider_holding_dragon_gift_deck_top_buff`,
`edr_w2_darkrider_no_dragon_condition_fizzles`,
`edr_w2_wallow_copies_gift_to_hand_and_deck`,
`edr_w2_xavius_deck_minion_gift_stats_elusive`,
`edr_w2_overgrown_horror_reduces_gifted_hand_minions`,
`edr_w2_gift_summon_copy_on_play_two_two_copy`,
`edr_w2_gift_battlecry_triggers_twice`, `edr_w2_gift_reborn_full_keeps_enchantments`
(15 scenarios in `tests/differential.rs`). Full `cargo test` green; `cargo clippy
--all-targets` clean.

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

## F-A12 — Classic Defias Ringleader (ROGUE_011) combo summoned nothing ✅ resolved (2026-08-08)

The Classic-pool `ROGUE_011` combo_effect referenced `"ROGUE_t"`, a token that
was never defined — `card_by_id` missed and `resolve_summon` silently returned
None, so the Classic combo was a no-op (found 2026-08-08 while the Core Set W6
reprint `CORE_EX1_131` was wired, which defined the real token `EX1_131t`).

Fix applied: `ROGUE_011`'s combo now summons the real `EX1_131t` Defias Bandit
(2/1, 1-cost; already in `ALL_CARDS` via `CORE_DEFIAS_BANDIT`); the stale note
in `core_w6.rs` updated. Pinned by the `classic_defias_bandit_combo` scenario in
`tests/differential.rs` (no combo on the first card, Bandit summoned on the
second). No RL pool impact — no card IDs changed.

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

### 14.2 2025–2026 expansions M1-W3 — the Emerald Dream choose-one wave (12 cards) 🔓 registered

The registered simplifications of the M1-W3 wave (`src/cards/exp_edr_w3.rs` +
the real Choose One pipeline: `PendingChoice`/`ChoiceKind::ChooseOne`, the
per-option `Action::Choose { choice_id, option }` resolution, and the
`choose_one_option_names` labels). The choose-one branches themselves are real
(option 0 = the battlecry slot, option 1 = the choose_one_effect slot,
exposed through `legal_actions` — P3). What stays simplified: the Discover
cards in this wave resolve as random picks, Reforestation's 蓄力-style hold
mechanic is omitted, and Wyvern's Slumber's Dormant is approximated with a
plain token. As with §14/§14.1, these handwritten expansion cards are not in
the RL pool (classic + core 668/659), so the rows are informational: they keep
the code's `(simplified: …)` markers traceable to the ledger. Each row stays
open until its mechanism lands.

| ID | Card | Simplified | When real |
| --- | --- | --- | --- |
| EDR_273 | Symbiosis | Discover a Choose One card from another class → a random pick over the fixed `OTHER_CLASS_CHOOSE_ONE_POOL` table (the 10 non-Druid EDR choose-one cards); the brief's in-window pool formula yields an empty set because every in-window choose-one card is Druid, so the fixed table is the pool | a real Discover pipeline over an other-class Choose One pool |
| EDR_843 | Reforestation | The "hold this for 3 turns to draw both" 蓄力-style mechanic is omitted — each branch draws its card type immediately | the hold mechanic (a future wave) |
| EDR_820 | Wyvern's Slumber | The Dormant Dreadseeds are plain 0/3 can't-attack tokens that never wake (the engine has no Dormant) | a Dormant pipeline |
| EDR_872 | Spark of Life | Both Discover branches → a random pick (a Mage spell from `MAGE_CLASSIC` / a Druid spell from `DRUID_CLASSIC`) | the real Discover pipeline (W3's own Discover still pending) |
| EDR_233t1/t2, EDR_263t, EDR_490t, EDR_813t, EDR_820t | the wave's six tokens | Handwritten consts only (no data rows in the official set / generated baselines — the standard expansion-token registration) | official token data |
| EDR_257, EDR_263, EDR_463, EDR_490, EDR_525, EDR_570, EDR_813 | the other seven choose-one cards | faithful (both branches real) | — |
| EDR_233 | Spirits of the Forest | faithful | — |

中文小结（同上）：M1-W3 抉择波的真抉择管线已落地（分支选择为显式
`Action::Choose`，经 `legal_actions` 暴露，P3）；本波简化为：Symbiosis 的
"发现其他职业抉择牌"沿用随机简化（按固定 `OTHER_CLASS_CHOOSE_ONE_POOL`
表——10 张非德鲁伊 EDR 抉择牌——因设计稿窗口公式在纯德鲁伊窗口下取空集，
故改为固定表，非规范偏离见 W3 报告）；Reforestation 的"蓄力三回合后再抽
另一张"未实现（两分支均立即按类型抽牌）；Wyvern's Slumber 的休眠种子简化为
永不苏醒的 0/3 不可攻击白板；Spark of Life 两分支的发现均简化为随机取一
（法师法术池 / 德鲁伊法术池）；六张衍生物为手写 const（官方数据无此行，
与既有扩展衍生物登记一致）。

F5 coverage: `edr_w3_spirits_of_the_forest_both_branches`,
`edr_w3_lightmender_both_branches`, `edr_w3_grace_of_the_greatwolf_both_branches`,
`edr_w3_symbiosis_adds_other_class_choose_one`,
`edr_w3_twilight_influence_both_branches`, `edr_w3_sleep_paralysis_both_branches`,
`edr_w3_barbed_thorn_poisonous_this_turn`, `edr_w3_barbed_thorn_deathrattle_on_replace`,
`edr_w3_ominous_nightmares_both_branches`, `edr_w3_morbid_swarm_both_branches_and_corpse_gate`,
`edr_w3_wyverns_slumber_both_branches`, `edr_w3_reforestation_draws_by_card_type`,
`edr_w3_spark_of_life_discover_class_spells`, `edr_w3_pending_choice_gate_and_auto_resolve`
(14 scenarios in `tests/differential.rs`). Full `cargo test` green; `cargo clippy
--all-targets` clean.

### 14.3 2025–2026 expansions M1-W4a — the Emerald Dream non-legendary wave (86 cards) 🔓 registered

The registered simplifications of the M1-W4a wave (`src/cards/exp_edr_w4a.rs`,
95 consts = the 86 non-legendary EDR cards of the W4 "remaining cards" list
plus 9 tokens; the 23 elite Wild Gods are M1-W4b, the FIR_* miniset M1-W5).
New mechanism facilities this wave: a per-player spell-cast counter
(`Player::spells_cast_total`, the New Moon upgrade condition — the official
per-card tracking is approximated per-player, and the current cast counts:
the counter only bumps in the after-cast `SpellCast` handler, so the effect
tests `spells_cast_total + 1`); a per-player hero-power-use counter
(`Player::hero_power_uses`) plus a one-time free-hero-power flag (Dreambound
Disciple) and a `HeroPowerDef` on players; per-player pending end-of-turn
timers (`Player::self_damage_pending`/`self_damage_turns` for Rotten Apple,
`crystal_gain_pending`/`crystal_gain_turns` for Fractured Power — the official
"at the end of the next 2 turns" lands at the end of those turns); the
played-minion log (`Player::played_minion_ids`, Twisted Webweaver); a
`HeroAttackedMinion` event feeding the three weapon triggers (Shepherd's
Crook, Emerald Haze, Defiled Spear); and `RandomPool::Murloc`. Two engine
fixes landed during this wave and are worth recording: `resolve_destroy_minion`
previously no-op'd on `EffectTarget::Self_` (a latent bug that made Siphoning
Growth and Divination destroy nothing — a new `EffectTarget::FriendlyMinion`
arm fixes both), and `SummonCopyOfSelf` now strips the copy's battlecry
(Bloodthistle Illusionist's copy used to recurse through the
`MinionSummoned` battlecry dispatch to a full board). Stellar Balance's
faithful spell add resolves to the db's classic `DRUID_011` Moonfire /
`DRUID_006` Starfire — the modern `CORE_EX1_*` ids do not exist in this db,
so `card_by_id` would silently no-op; they are the same cards, pre-rename.
Renewing Flames (EDR_255) re-picks its target per hit over effective health,
so the second 5-damage hit can land on a different enemy than the first —
that is faithful engine behavior, not a simplification. As with §14–§14.2,
these handwritten expansion cards are not in the RL pool (classic + core
668/659), so the rows are informational: they keep the code's
`(simplified: …)` markers traceable to the ledger. Each row stays open until
its mechanism lands.

| ID | Card | Simplified | When real |
| --- | --- | --- | --- |
| EDR_105 | Creature of Madness | Dark Gift Discover → a random 3-cost minion with a Dark Gift (the fixed Discover simplification) | a real Discover pipeline over the Dark Gift pool |
| EDR_234 | Emerald Bounty | The "can't play them for 2 turns" restriction is dropped — it would need per-card play locks | a play-lock mechanism |
| EDR_251 | Dragonscale Armaments | The engine does not track card origins, so the "didn't start there" half becomes a random spell | card-origin tracking |
| EDR_256 | Dreamwarden | The "card in your deck that didn't start there" condition is dropped — always draws the top card and gains +2/+2 | card-origin tracking |
| EDR_260 | Illusory Greenwing | The "Summoned When Drawn" half is dropped — the shuffled 4/5 Dragons enter the hand normally | a summon-when-drawn pipeline |
| EDR_261 | Amphibian's Spirit | The engine stores one Deathrattle per entity, so a target that already has one gets it replaced | multi-deathrattle support |
| EDR_270 | Horn of Plenty | The Nature spell school is not modeled → the pool is any spell; Discover → random | spell schools + Discover |
| EDR_271 | Grove Shaper | The Nature trigger fires on any friendly spell; the treant's deathrattle adds a random spell instead of a copy of the triggering one | spell schools + per-instance memory |
| EDR_416 | Shepherd's Crook | Dormant is not modeled — the summoned Sheep is a 3/3 that never wakes (Dreadseed precedent) | a Dormant pipeline |
| EDR_454 | Clutch of Corruption | The egg cannot remember the chosen Dragon — it hatches a copy of a random friendly Dragon | per-instance card memory |
| EDR_455 | Succumb to Madness | Discover → a random friendly Dragon from the graveyard (the fixed Discover simplification; the death record itself reads the graveyard zone, like Calia Menethil) | a real Discover pipeline |
| EDR_460 | Wish of the New Moon | The per-card New Moon counter is approximated by the player's total spells cast (the current cast counts) | per-card counters |
| EDR_461 | Ritual of the New Moon | Same New Moon approximation (summons 6-cost minions after 3 spells cast) | per-card counters |
| EDR_469 | Slumbering Sprite | Dormant is not modeled — the Sprite is a 3/3 that never wakes; no hero-power-use trigger event exists | Dormant + hero-power events |
| EDR_470 | Barkshield Sentinel | No hero-power-use trigger — the buff fires at end of turn, but only when the hero power was used that turn | a hero-power-use trigger |
| EDR_482 | Rotten Apple | The 2 turns of self-damage tick at the END of each of the next 2 of your turns (`Player::self_damage_pending`) | exact timing per turn-start semantics |
| EDR_483 | Fractured Power | The delayed crystals land at the END of the 2nd of your turns (`Player::crystal_gain_pending`) | exact timing per turn-start semantics |
| EDR_484 | Scavenging Flytrap | The dead minion's enchantments are wiped on the graveyard move, so the Flytrap gains its base Attack | persist final attack into the death event |
| EDR_491 | Archdruid of Thorns | One Deathrattle per entity — gains the deathrattle of the most recently died friendly minion this turn | multi-deathrattle support |
| EDR_494 | Hungering Ancient | The eaten minion's identity cannot be stored per instance — the deathrattle adds a random deck minion to hand | per-instance card memory |
| EDR_520 | Forbidden Shrine | "Cast" is a direct effect resolution — no spell entity or SpellCast event is produced | a cast pipeline for random spells |
| EDR_524 | Shadowcloaked Assailant | When several enemy cards match, one random matching card is shuffled (reads the opponent's hand — registered in `POOL_OPEN_CARDS`) | shuffle-all-matching semantics |
| EDR_529 | Plucky Podling | Transform interception is unmodeled — a plain 1/2 | a transform interception hook |
| EDR_530 | Daydreaming Pixie | The Nature spell school is not modeled → the pool is any spell | spell schools |
| EDR_780 | Bloodthistle Illusionist | The shared-secret-death clause is unmodeled — a plain copy (its battlecry is stripped on the copy to stop summon recursion) | shared-death tracking |
| EDR_781 | Harbinger of the Blighted | No zone-change trigger event — the bounce-to-hand clause is unmodeled (inert 2/3) | a zone-change trigger |
| EDR_810 | Hideous Husk | The leech-steal aura is unmodeled — the Leeches are plain 0/2 tokens | the leech aura |
| EDR_812 | Grotesque Runeblade | Rune tracking is unmodeled — a plain 2/2 weapon | rune tracking |
| EDR_815 | Corpse Flower | No enemy-summons trigger event — the clause is unmodeled (inert 0/5) | an opponent-summons trigger |
| EDR_840 | Grim Harvest | The Dormant Dreadseed is the W3 can't-attack token EDR_820t, which never wakes | a Dormant pipeline |
| EDR_841 | Dreadsoul Corrupter | Same Dreadseed token for both battlecry and deathrattle | a Dormant pipeline |
| EDR_849 | Dreambound Raptor | The official Bonus Effect pool is approximated by a fixed keyword pool — Taunt / Divine Shield / Poisonous / Windfury / Elusive / Stealth | the Bonus Effect list |
| EDR_979 | Ancient of Yore | Dormant is not modeled — the Ancient never wakes; the end-of-turn armor + draw keeps running while it is in play | a Dormant pipeline |
| EDR_271t | Treant (token) | The deathrattle adds a random spell — the triggering spell cannot be remembered per instance | per-instance card memory |
| EDR_416t | Sheep (token) | Dormant is not modeled — a 3/3 that never wakes (handwritten const only, no official data row) | a Dormant pipeline / official token data |
| EDR_454t | Egg (token) | Hatches a copy of a random friendly Dragon instead of the chosen one | per-instance card memory / official token data |

中文小结（同上）：M1-W4a 波（86 张非传说卡 + 9 张衍生物）的新机制设施：
每玩家法术计数（`spells_cast_total`，新月光辉升级条件——官方按牌计数的
近似，当前施放的法术计入阈值：计数器只在施放后的 `SpellCast` 事件里 +1，
效果判定用 `spells_cast_total + 1`）、每玩家英雄技能使用计数与一次性免费
英雄技能（Dreambound Disciple）、回合末定时器（Rotten Apple 的自伤与
Fractured Power 的延迟水晶，均在"之后的第 2 个回合结束时"落地）、已打
随从日志（Twisted Webweaver）、`HeroAttackedMinion` 事件（三把武器触发）
与 `RandomPool::Murloc`。本波登记的两处引擎修复：`resolve_destroy_minion`
此前对 `EffectTarget::Self_` 直接 no-op（潜伏 bug 令 Siphoning Growth 与
Divination 的摧毁效果失效，新增 `FriendlyMinion` 分支一并修复）；
`SummonCopyOfSelf` 现在剥除复制体的战吼（否则 Bloodthistle Illusionist
的复制体会经 `MinionSummoned` 战吼派发递归铺满全场）。Stellar Balance
忠实落地为 db 内的经典 `DRUID_011` 月火术 / `DRUID_006` 星火术（现代
`CORE_EX1_*` id 在本 db 不存在，按原 id 查询会静默无效——它们是改名前的
同一张卡）。Renewing Flames（EDR_255）每次命中按有效生命重新选目标，
第二段 5 点伤害可能落在不同的敌人身上——这是忠实引擎行为而非简化。本波
简化与既往一致：发现 → 随机（EDR_105/EDR_455 按固定发现简化）；自然法术
学派 → 任意法术（EDR_270/EDR_271/EDR_530）；Dormant → 永不苏醒的不可
攻击白板（EDR_416/EDR_469/EDR_979/EDR_840/EDR_841/EDR_416t，沿用 W3
的 EDR_820t 先例）；"非起始于卡组的牌"条件取消（EDR_251/EDR_256）；
"抽到时召唤"取消（EDR_260）；单亡语槽 → 覆盖/取最近死亡者
（EDR_261/EDR_491）；实例身份无法记忆 → 固定/随机替代
（EDR_454/EDR_494/EDR_271t/EDR_454t）；对手召唤/回手触发未建模（惰性
白板 EDR_781/EDR_815）；符文追踪与吸血偷取光环取消（EDR_812/EDR_810）；
"2 回合内不能打出"限制取消（EDR_234）；3 张衍生物为手写 const（官方数据
无此行，与既有扩展衍生物登记一致）。扩展手写卡均不在 RL 池（经典 + 核心
668/659），本表仅作登记追踪，各行在机制落地前保持开放。

F5 coverage: `edr_w4a_deck_top_buffs_and_scans`, `edr_w4a_death_records`,
`edr_w4a_played_minion_logs`, `edr_w4a_spell_tracking`, `edr_w4a_locations`,
`edr_w4a_weapons`, `edr_w4a_rogue_hand_reads`, `edr_w4a_warlock_timers`,
`edr_w4a_dormant_and_vanilla`, `edr_w4a_battlecry_pools`,
`edr_w4a_spell_pools`, `edr_w4a_midrange_keywords_and_end_of_turn_a`,
`edr_w4a_midrange_b`, `edr_w4a_misc_spells_and_battlecries`
(14 scenarios in `tests/differential.rs`). Full `cargo test` green; `cargo
clippy --all-targets` clean.
### 14.4 2025–2026 expansions M1-W4b — the Emerald Dream elite Wild Gods (23 legendary cards) 🔓 registered

The registered simplifications of the M1-W4b wave (`src/cards/exp_edr_w4b.rs`,
23 consts = the 23 elite legendary Wild Gods of the W4 "remaining cards"
list, plus 2 tokens: EDR_209t Ancient and EDR_818t Beetle; the FIR_*
miniset is M1-W5). New mechanism facilities this wave: a per-player attack
counter (`Player::omen_attack_bonus` — Omen's "Improves" deathrattle, the
official per-minion enchantment approximated per-player); a per-player
spell double-cast counter (`Player::next_spells_cast_twice` — Tyrande, the
re-cast fires immediately after the original resolution with no target and
no second `SpellCast` event); a per-player one-time next-card-cost flag
(`Player::next_card_costs_zero` — Agamaggan) and a game-long
cards-cost-(1) flag (`Player::cards_cost_1` — Aviana, applied LAST in the
cost pipeline so the one-time/set-to-value effects above keep their lower
costs only when they land at or below 1); a per-turn Dragon-play counter
(`Player::dragons_played_this_turn` — Naralex's first-Dragon discount); the
choose-thrice machinery (Cenarius — a repeatable choose-one choice,
`repeat` on `PendingChoice`); the keep-or-top choice
(`ChoiceKind::QonzuKeepOrTop` — Q'onzu, which also surfaced an engine bug:
the choice arm never switched the card's PLAYER component, so a card placed
on the enemy deck top was drawn back into the owner's hand — fixed with
`world.set_player`); and Ursoc's kill-record resurrection (the battlecry's
sync-damage attacks record the killed card IDs for the deathrattle).
Ashamane (EDR_527) reads the opponent's deck and is registered in
`POOL_OPEN_CARDS` (the Nightmare Fuel precedent) with its effect added to
the `is_pool_open_effect` matches. Ysondre's random-Dragon summon draws
from the same dragon pool as the engine's other Dragon generators. As with
§14–§14.3, these handwritten expansion cards are not in the RL pool
(classic + core 668/659), so the rows are informational: they keep the
code's `(simplified: …)` markers traceable to the ledger. Each row stays
open until its mechanism lands.

| ID | Card | Simplified | When real |
| --- | --- | --- | --- |
| EDR_000 | Ysera, Emerald Aspect | The Start of Game +5 maximum Mana applies at play time (the engine has no StartOfGame event — the +5 lands with the battlecry) | a StartOfGame event |
| EDR_031 | Ohn'ahra | "Play the top 3 cards from your deck" has no pipeline — the end-of-turn effect draws 3 cards instead | a play-from-deck pipeline |
| EDR_258 | Toreth the Unbreaking | The three-hit Divine Shield is a new mechanic — Toreth carries a normal Divine Shield | multi-hit Divine Shield |
| EDR_259 | Ursol | The "cast as an Aura that lasts 3 turns" is a new mechanic — the highest-Cost hand spell is cast immediately | aura-ized casts |
| EDR_421 | Omen | The official per-minion "Improves" enchantment is approximated by a per-player attack counter (+1 damage per attack) | per-minion enchantments |
| EDR_464 | Tyrande | The doubled cast re-resolves the same effect immediately after the original with no target, firing no second `SpellCast` event | a full double-cast pipeline |
| EDR_480 | Goldrinn | The "friendly Beasts deal double damage" aura is approximated by a damage-pipeline hook while EDR_480 is on the board | an attack-doubling aura |
| EDR_489 | Agamaggan | The opponent's-Health cost needs a cost-pipeline rework — the next card costs (0) instead | opponent-Health costs |
| EDR_517 | Q'onzu | Discover → a random spell from the full database (the standing Discover→random debt); the keep/top decision surfaces as the `QonzuKeepOrTop` choice | a real Discover pipeline |
| EDR_526 | Renferal, the Malignant | The one-turn trap is a new mechanic — the enemy discards a random hand card instead; the "Improved" scaling is unmodeled | a trap mechanic |
| EDR_818 | Nythendra | The split/reform cycle is unmodeled — the deathrattle summons seven 1/1 Beetles, one per point of the 7/7 body | a split/reform mechanic |
| EDR_846 | Shaladrassil | The corruption clause is unmodeled — the spell adds the five DREAM_POOL cards to hand | a corruption mechanic |
| EDR_895 | Aviana, Elune's Chosen | The three-turn lunar-cycle timing is unmodeled — the cards-cost-(1) effect applies immediately for the rest of the game | a lunar-cycle timer |

中文小结（同上）：M1-W4b 波（23 张精英野神传说卡 + 2 张衍生物）的新机制
设施：每玩家攻击计数（`omen_attack_bonus`，Omen 的"Improves"亡语——官方
按随从个体的附魔近似为每玩家计数）、每玩家法术双施计数（`next_spells_cast_twice`，
Tyrande 的加倍施放紧跟在原效果结算后、无目标、不产生第二次 `SpellCast`
事件）、一次性下张卡费用 (0) 标志（`next_card_costs_zero`，Agamaggan）
与本局卡牌费用 (1) 标志（`cards_cost_1`，Aviana，在费用管线最后应用，
保证其上的一次性/设定值效果只有落到 ≤1 时才保留更低费用）、每回合龙类
打出计数（`dragons_played_this_turn`，Naralex 的首张龙优惠）、三选抉择
机制（Cenarius 的 choose-thrice，`PendingChoice` 上的 `repeat`）、保留
或置顶抉择（`ChoiceKind::QonzuKeepOrTop`——顺带修了一个引擎 bug：该抉择
分支此前不切换卡牌的 PLAYER 组件，置入敌方牌库顶的牌会被抽回持有者手牌，
补上 `world.set_player` 修复）以及 Ursoc 的击杀记录复活（战吼的同步伤害
攻击逐目标结算时记录被杀卡牌 id 供亡语复活）。Ashamane（EDR_527）读取
对手牌库，登记进 `POOL_OPEN_CARDS`（沿用 Nightmare Fuel 先例），其效果
也加入了 `is_pool_open_effect` 的匹配。Ysondre 的随机龙召唤与其他龙类
随机生成器共用同一个龙池。本波简化与既往一致：发现 → 随机（EDR_517）；
"打出牌库顶 3 张"→ 抽 3 张（EDR_031）；新机制未建模 → 简化替代
（EDR_258 三破圣盾 → 普通圣盾、EDR_259 光环化施放 → 立即施放、
EDR_421 个体附魔 → 每玩家计数、EDR_526 陷阱 → 随机弃牌、
EDR_818 分裂/重组 → 亡语召唤甲虫、EDR_846 腐化条款、EDR_895 月相周期 →
立即生效）；Start-of-Game 无事件 → 打出场时结算（EDR_000）；费用管线
改造挂账（EDR_489）；双施时序近似（EDR_464）；伤害翻倍光环 → 伤害管线
钩子（EDR_480）。扩展手写卡均不在 RL 池（经典 + 核心 668/659），本表仅作
登记追踪，各行在机制落地前保持开放。

F5 coverage: `edr_w4b_ysera_mana`, `edr_w4b_ohnahra_draws_three`,
`edr_w4b_cenarius_choose_thrice`, `edr_w4b_merithra_resurrects_different`,
`edr_w4b_toreth_divine_shield`, `edr_w4b_ursol_casts_highest_spell`,
`edr_w4b_omen_improves_deathrattle`, `edr_w4b_aessina_split_damage`,
`edr_w4b_tyrande_cast_twice`, `edr_w4b_ysondre_dragon_per_death`,
`edr_w4b_tortolla_armor_attack`, `edr_w4b_goldrinn_double_damage`,
`edr_w4b_agamaggan_next_card_free`, `edr_w4b_alarashi_transforms_demons`,
`edr_w4b_qonzu_keep_or_top`, `edr_w4b_renferal_discards`,
`edr_w4b_ashamane_fills_hand`, `edr_w4b_nythendra_beetles`,
`edr_w4b_ursoc_kill_resurrect`, `edr_w4b_naralex_dragon_discount`,
`edr_w4b_shaladrassil_dream_cards`, `edr_w4b_broll_companion`,
`edr_w4b_aviana_cards_cost_one` (23 scenarios in `tests/differential.rs`).
Full `cargo test` green; `cargo clippy --all-targets` clean.
### 14.5 2025–2026 expansions M1-W5 — the Embers of the World Tree miniset (38 cards + 1 token) 🔓 registered

The registered simplifications of the M1-W5 wave (`src/cards/exp_edr_w5.rs`,
38 consts = the FIR_777~FIR_961 miniset, plus 1 token: FIR_901t Frostburn
Broodling, the 4/4 Taunt Dragon Frostburn Matriarch summons; FIR_918t Light
of the Full Moon — the upgraded form — is NOT implemented, the upgrade step
is registered in the FIR_918 row). The wave is the roadmap's final wave and
reuses the W1–W4 primitives only — no new engine mechanism. New facilities
this wave: a per-player "next turn only" Mana Crystal flag
(`Player::temp_mana_crystal_pending` — Emberscarred Whelp, granted at the
owner's ManaRefill step, the crystal_gain_pending precedent); a
damage-pipeline hook on hero health loss (`fire_emberroot_hook` in rules.rs
— Emberroot Destroyer, since no HeroDamaged trigger event exists, fired at
the two points where the hero's health is actually reduced, on the owner's
turn only); the Everburning Phoenix cost discount (a cost.rs arm reading the
per-turn play counter, which bumps AFTER the discount computes so the
current card is excluded); and Magma Hound's AttackedMinion trigger
(registered in apply_card_keywords, pinned to the attacker; the "and
survives" clause reads effective health at trigger time). FIR_907
Amirdrassil is a Location (the EDR_454/EDR_520 convention: CardType::Location,
health 0, durability 3, effect in the battlecry slot) and its baseline
divergence is registered in `expansion_differential_rebalanced`. As with
§14–§14.4, these handwritten expansion cards are not in the RL pool
(classic + core 668/659), so the rows are informational: they keep the
code's `(simplified: …)` markers traceable to the ledger. Each row stays
open until its mechanism lands.

| ID | Card | Simplified | When real |
| --- | --- | --- | --- |
| FIR_900 | Cremate | Discover → a random minion given a random Dark Gift and costing (2) less | a real Discover pipeline |
| FIR_902 | Sigil of Cinder | The "start of your next turn" timing is unmodeled — the 6 damage resolves immediately as random 1-damage pings | a delayed-secret timing |
| FIR_904 | Felfire Blaze | The Fel-spell filter is unmodeled — fires on ANY friendly spell cast | a spell-school field |
| FIR_906 | Overheat | The Nature-spell filter is unmodeled — any hand spell can be discarded | a spell-school field |
| FIR_907 | Amirdrassil | The "Improves each use!" escalation is unmodeled — every activation resolves the first-tier effect (the Location convention is faithful) | per-use escalations |
| FIR_909 | Bursting Shot | The three random pings may hit the same target | distinct random targets |
| FIR_910 | Scorching Winds | The Fire-spell filter is unmodeled — any hand spell can be discarded | a spell-school field |
| FIR_911 | Smoldering Grove | The "Upgrades each turn, but discards after N" cycle is unmodeled — the spell always plays its first-turn version | per-turn upgrade cycles |
| FIR_913 | Inferno Herald | The Fire-spell filter is unmodeled — fires on ANY friendly spell cast | a spell-school field |
| FIR_914 | Smoldering Strength | The "Upgrades each turn, but discards after N" cycle is unmodeled — the spell always plays its first-turn version | per-turn upgrade cycles |
| FIR_916 | Smoldering Ascent | The "Upgrades each turn, but discards after N" cycle is unmodeled — the spell always plays its first-turn version | per-turn upgrade cycles |
| FIR_918 | Light of the New Moon | The per-play return counter is approximated by the player's spell total (the current cast counts) and the returned card is a fresh copy; the FIR_918t Light of the Full Moon upgrade step is unmodeled | a per-card play counter |
| FIR_919 | Everburning Phoenix | The "at end of turn" deathrattle fires immediately | an end-of-turn deathrattle |
| FIR_920 | Smoke Bomb | Discover → a random Combo/Battlecry/Stealth minion given a random Dark Gift | a real Discover pipeline |
| FIR_924 | Shadowflame Stalker | Discover → a random Demon given a random Dark Gift (the copy IS faithful) | a real Discover pipeline |
| FIR_927 | Emberscarred Whelp | Discover → a random 5-Cost card; the "next turn only" Mana Crystal is granted at the owner's ManaRefill | a real Discover pipeline; until-end-of-next-turn timing |
| FIR_928 | Keeper of Flame | The "destroyed in 3 turns" clause is unmodeled — the +3/+3 buff only | a delayed-destroy mechanic |
| FIR_929 | Living Flame | The Fire-spell filter is unmodeled — the deathrattle draws any card | a spell-school field |
| FIR_939 | Shadowflame Suffusion | Discover → a random Warrior minion given a random Dark Gift (the damage IS faithful) | a real Discover pipeline |
| FIR_941 | Searing Reflection | The tutor is a first-match deck scan — the non-minion cards above the drawn minion stay in the deck | a draw-through tutor |
| FIR_951 | Volcoross | The three-way choose-one is unmodeled — the largest affordable 10/20/30 Corpse option is spent automatically | a real choose-one |
| FIR_952 | Scorchreaver | Discover → a random spell and the Fel-spell filter unmodeled — every hand spell is reduced | a real Discover pipeline; a spell-school field |
| FIR_953 | Magma Hound | The splash trigger fires at attack declaration, before the trade damage resolves (the W2 porcupine convention) — the pings queued at declaration still splash when the Hound dies to the retaliation | an after-attack trigger |
| FIR_955 | Emberroot Destroyer | No HeroDamaged event exists — a damage-pipeline hook fires on any hero health loss on the owner's turn | a HeroDamaged event |
| FIR_959 | Fyrakk the Blazing | The Immune-to-Fire clause is unmodeled and "15 Mana worth of Fire spells" is approximated as 15 random 1-damage pings | a spell-school field; mana-weighted casts |

中文小结（同上）：M1-W5 波（迷你系列"世界之树的余烬"，38 张 + 1 张
衍生物 FIR_901t 冰霜巢母雏龙 4/4 嘲讽龙）沿用 W1–W4 全部原语，不引入新
引擎原语；新设施：每玩家"仅限下回合"法力水晶标志
（`temp_mana_crystal_pending`——烬痕幼龙，在持有者 ManaRefill 步骤结算，
沿用 crystal_gain_pending 先例）、英雄受伤管线钩子（`fire_emberroot_hook`
——余烬根毁灭者，引擎无 HeroDamaged 事件，在英雄血量实际减少的两处
结算、仅持有者回合）、永恒烈焰凤凰的降费（cost.rs 分支读每回合打出
计数，计数在降费计算之后才递增，当前牌不计入）以及岩浆猎犬的
AttackedMinion 触发（apply_card_keywords 注册、钉在攻击者上；在攻击
宣告时触发、溅射先于对撞伤害结算）。FIR_907 阿梅达希尔为地点（沿用 EDR_454/
EDR_520 惯例：Location、血量 0、耐久 3、效果放战吼槽），基线差异登记
于 `expansion_differential_rebalanced`。本波简化与既往一致：发现 → 随机
（FIR_900/920/924/927/939/952）；法术学派过滤未建模（烈焰焚身/炼狱使者
任意友方法术触发、炽热风暴/灼热之戒任意手牌法术可弃、活体烈焰任意抽牌、
炼狱收割者任意法术降费——引擎无法术学派字段）；三张渐燃卡的"每回合
升级、N 回合后弃置"周期未建模（FIR_911/914/916 只打首回合版本）；燃尽
信标的"下回合开始"改为立即结算（FIR_902）；月华之光回手计数用玩家
法术总数近似、回手为全新复制、满月升级步骤未建模（FIR_918，FIR_918t
不实现）；凤凰亡语的"回合结束时"改为立即（FIR_919）；狱火烈焰的
"3 回合后摧毁"未建模（FIR_928）；炽焰倒映的导师效果为牌库首张随从
直取（FIR_941，前置非随从卡留在牌库）；沃克罗斯的 10/20/30 三选改为
自动选最大可承担尸体档（FIR_951）；岩浆猎犬的溅射触发在攻击宣告时
（结算于对撞伤害之前，沿用 W2 豪猪惯例）——宣告时已入队的 1 点溅射
即使猎犬被反击杀死仍会落地（FIR_953）；余烬根毁灭者按持有者回合任意
英雄掉血触发（FIR_955）；
菲莱克之怒"免疫火焰法术"未建模、"15 点法力值的火焰法术"近似为 15 次
随机 1 点伤害（FIR_959）。扩展手写卡均不在 RL 池（经典 + 核心 668/659），
本表仅作登记追踪，各行在机制落地前保持开放。

F5 coverage: `edr_w5_spirit_of_the_kaldorei`, `edr_w5_avatar_of_destruction`,
`edr_w5_cremate`, `edr_w5_frostburn_matriarch`, `edr_w5_sigil_of_cinder`,
`edr_w5_felfire_blaze`, `edr_w5_overheat`, `edr_w5_amirdrassil`,
`edr_w5_charred_chameleon`, `edr_w5_bursting_shot`, `edr_w5_scorching_winds`,
`edr_w5_smoldering_grove`, `edr_w5_inferno_herald`,
`edr_w5_smoldering_strength`, `edr_w5_smoldering_ascent`,
`edr_w5_light_of_the_new_moon`, `edr_w5_everburning_phoenix`,
`edr_w5_smoke_bomb`, `edr_w5_petal_picker`, `edr_w5_cindersword`,
`edr_w5_flames_of_the_firelord`, `edr_w5_shadowflame_stalker`,
`edr_w5_emberscarred_whelp`, `edr_w5_keeper_of_flame`,
`edr_w5_living_flame`, `edr_w5_shadowflame_suffusion`,
`edr_w5_zaqali_flamemancer`, `edr_w5_searing_reflection`,
`edr_w5_volcoross`, `edr_w5_scorchreaver`, `edr_w5_magma_hound`,
`edr_w5_conflagrate`, `edr_w5_emberroot_destroyer`, `edr_w5_dragon_turtle`,
`edr_w5_tindral_sageswift`, `edr_w5_fyrakk_the_blazing`,
`edr_w5_tending_dragonkin`, `edr_w5_ashleaf_pixie` (38 scenarios in
`tests/differential.rs` — every miniset card at least once). The 7 Dark
Gift cards get dedicated scenarios: the gift-granting quartet with their
extra effects — FIR_900 (discount), FIR_924 (copy), FIR_939 (dual
damage + grant) and FIR_920 — plus the holding-gift trio FIR_901
(Frostburn Matriarch), FIR_922 (Cindersword) and FIR_956 (Dragon
Turtle); Petal Picker's imbue-twice condition is exercised both ways.
Full `cargo test` green; `cargo clippy --all-targets` clean.

### 15. 2025–2026 expansions M2-W2 — the Un'Goro quest wave (23 cards) 🔓 registered

The registered simplifications of the M2-W2 wave (`src/cards/exp_tlc_w2.rs`):
the 11 Un'Goro quest cards (1-cost legendary SPELLs with `spell_effect: None`,
routed into the quest zone by the play path) plus their 12 reward tokens. The
quest mechanic itself (M2-W1) is extended this wave: TLC_817's second
progress bar (`QuestDef::second` / `Quest::second` — the card leaves the
quest slot only when both bars complete), the repeatable TLC_426 (progress
resets on completion, the permanent reward flag stays), the `Temporary`
hand-card marker (discarded at the owner's turn end — its W4 creators inject
it in real play), and two player-level reward flags (`Player::murloc_summon_buff`
consumed by the summon hook, `Player::deal_exact_2_bonus` consumed by the
damage hook). The W1 placeholder rewards were replaced with the real ones:
TLC_426 sets the permanent flag, TLC_513 summons the two Ninjas directly,
TLC_817 summons Sol'etos t3/t4 per bar. As with §14–§14.5, these handwritten
expansion cards are not in the RL pool (classic + core 668/659), so the rows
are informational: they keep the code's `(simplified: …)` markers traceable
to the ledger. Each row stays open until its mechanism lands.

| ID | Card | Simplified | When real |
| --- | --- | --- | --- |
| TLC_229t14 | Ashalon, Ridge Guardian | The official Adapt battlecry (choose one of three Adapt options) is fixed to +1/+1 for all friendly minions | an Adapt/choose-one pipeline |
| TLC_433t | Tyrax, Bone Terror | The official "Terror's Grave" location chain (Tyrax transforms into the Grave, which resummons him) is unmodeled — the deathrattle resummons an 8/8 copy directly | a transform/location-deathrattle chain |
| TLC_446t1 | Underfel Rift | The official "activate" step (pay Health to activate) is unmodeled — a 5-cost 0/1 body only | an activate mechanic |
| TLC_460t | The Origin Stone | The official "after you Discover a card, this gains +1 Durability" is unmodeled — a 3-cost 0/8 body only | a discover-replay durability gain |
| TLC_513t | Master Dusk | The official reward replaces the hero with Master Dusk — unmodeled; the reward summons the two Tortollan Ninjas (TLC_513t2) directly | hero replacement |
| TLC_602t | Latorvius, Gaze of the City | The official Quest-Reward battlecry adds a 4-card reward pool to Discover — unmodeled (no battlecry; the pool lands in W4) | a real Discover pipeline |
| TLC_817t5 | Sol'etos, Life's Touch | The official "if you control both Sol'etos forms, combine them" step is unmodeled — the t3 and t4 tokens are independent | a combine/twin mechanic |
| TLC_830t | Shokk, Jungle Tyrant | The official battlecry's attack-filtered Discover pool is unmodeled (no battlecry) | a real Discover pipeline |

中文小结（同上）：M2-W2 波（"失落之城"任务卡，11 张任务 + 12 张奖励
衍生物）新增原语：任务第二进度条（TLC_817 双条，两条都满才离场）、
可重复任务（TLC_426 完成后进度清零、永久 +1/+1 鱼人召唤增益标志常驻）、
Temporary 手牌标记（回合结束时弃置，真实制造卡在 W4）、两个玩家级奖励
标志（`murloc_summon_buff` 由召唤钩子消费、`deal_exact_2_bonus` 由伤害
钩子消费）。W1 占位奖励已换成真实奖励（TLC_426 永久标志、TLC_513 直接
召唤两只忍龟、TLC_817 每条各召唤一尊索莱托斯）。简化与既往一致：发现 →
无战吼（TLC_602t/830t）；Adapt 三选一 → 固定全体 +1/+1（TLC_229t14）；
泰拉克斯的"恐怖之墓"地点链条 → 亡语直接复活 8/8 复制（TLC_433t）；
安杜菲尔裂隙的激活步骤 → 纯白板 0/1（TLC_446t1）；起源之石"发现后
+1 耐久" → 纯白板 0/8（TLC_460t）；英雄替换 → 直接召唤奖励随从
（TLC_513t，官方奖励是把英雄替换成暮光大师）；TLC_817t5 的"双形态
合体"步骤未建模（t3/t4 相互独立）。扩展手写卡均不在 RL 池（经典 + 核心
668/659），本表仅作登记追踪，各行在机制落地前保持开放。

F5 coverage: `tlc_w2_spirit_of_the_mountain_reward_summoned`,
`tlc_w2_restore_the_wild_everbloom_buffs_after_hero_attack`,
`tlc_w2_golakka_depths_repeatable_murloc_buff`,
`tlc_w2_reanimate_the_terror_tyrax_deathrattle`,
`tlc_w2_escape_the_underfel_temporary_discard`,
`tlc_w2_forbidden_sequence_origin_stone_equipped`,
`tlc_w2_lie_in_wait_master_dusk_ninjas`,
`tlc_w2_enter_the_lost_city_survive_turns`,
`tlc_w2_unleash_the_colossus_bonus_damage`,
`tlc_w2_reach_equilibrium_double_bar`, `tlc_w2_food_chain_shokk_battlecry`,
`tlc_w2_one_quest_per_player_real_cards` (12 scenarios in
`tests/differential.rs` — one per quest card, plus the Temporary primitive,
the two player flags and the one-quest-per-player rule with real cards).
Full `cargo test`: 432 of 433 pass — the single red is
`tlc_w1_spell_school_lookup_and_progress`, whose two assertions
(`tests/differential.rs` ~23074/23083: the quest leaves the slot and no
token is summoned after 4 Holy casts) pin the W1 placeholder semantics and
contradict W2's real dual-bar TLC_817 (the quest stays in the slot with the
second bar pending and two TLC_817t3 are summoned) — the minimal
two-assertion update is pending per the wave spec. `cargo clippy
--all-targets` clean.

### 16. 2025–2026 expansions M2-W3 — the Un'Goro Kindred wave (23 cards) 🔓 registered

The registered simplifications of the M2-W3 wave (`src/cards/exp_tlc_w3.rs`):
the 23 Kindred cards. The mechanic itself is faithful: "Kindred: X — X
activates when you played a card of the SAME TYPE earlier THIS TURN" (the
tribe for minions, SPELL for spells; the card itself never counts). The
activation state is a per-player `kindred_played` list pushed by every play
and cleared at the player's own turn end, and the four resolution shapes
land in the play path: cost-time discounts (TLC_366/600/816, checked before
the push), OnPlay effects (TLC_107/226/243/428/429/440/447/519/815/825/903,
after the base resolution), battlecry modifiers (TLC_454/463/482/829 —
replaces resolve via dedicated variants, the TLC_482 add-on after the
battlecry) and the drawn-card modifiers folded into dedicated draw
battlecries (TLC_223's Fire-school filter is the real W1 spell-school
registry; TLC_236's exact 1/2/3/4-Cost scan; TLC_432's Deathrattle ≤3
filter). Torga's deck scan is faithful: top-down over the actual deck for
the first Kindred-registry card, then the first remaining card of its
kindred type; an empty match draws nothing — with a Classic|Core deck that
is the official behavior. As with §14–§15, these handwritten expansion cards
are not in the RL pool (classic + core 668/659), so the rows are
informational: they keep the code's simplifications traceable to the
ledger. Each row stays open until its mechanism lands.

| ID | Card | Simplified | When real |
| --- | --- | --- | --- |
| TLC_102/223/243/432/463/482 | the six dual-tribe Kindred cards | The official "races" second tribe lands on the board entity (`apply_card_keywords`, the Mythical Terror precedent) and feeds tribe synergies, but the Kindred counter and the `kindred_type` registry use the primary race only — a dual-tribe card activates Kindred as one tribe, where the official semantics count it as either | a multi-race kindred counter |
| TLC_815 | Gravedawn Voidbulb | The random "4-Cost minion with Taunt" pool follows the D2 simplification — ALL_CARDS minions of cost 4, token ids excluded; the summoned minion's own battlecry fires (the Nerubian Swarmguard cascade is faithful) | the official summon pool |
| TLC_519 | Ambush Predators | The summon is the fixed TLC_519t Venomous Spitter token (Stealth + Poisonous) — a one-element D2 pool, no random selection | a random Spitter pool |
| TLC_482 | Slagclaw | The Kindred add-on triggers the Deathrattles of ALL friendly Sizzling Cinders — including one played earlier this turn — where the official "trigger their Deathrattles" covers only the two the battlecry summoned | a per-summoned-Cinder trigger scope |
| TLC_251 | Primalfin Challenger | The "your next Kindred triggers twice" flag is consumed by OnPlay Kindred resolutions only — cost-time discounts and battlecry modifiers are not doubled | a universal trigger-twice flag |
| TLC_107 | Stormbrewer | "Whenever this attacks, deal 3 to the target first" is a card-id-keyed ResolveAttack hook (the Lake Thresher precedent) instead of a general attack-trigger component | a general attack-trigger component |
| TLC_428 | Hot Spring Glider | The "next Murloc" flags are player-level (`next_murloc_discount` at play-cost time, `next_murloc_divine_shield` at play) consumed by the next Murloc play — the Glider itself never consumes its own flags (the discount is applied before its battlecry sets it, the shield before it enters play) | per-entity modifiers |
| TLC_825 | Ravasaur Matriarch | The Kindred target (deal damage equal to this Attack to an enemy minion) arrives via the PlayCard target, but `rl/env.rs` play_targets only extracts battlecry targets — the RL view cannot express it | a surfaced kindred target |

中文小结（同上）：M2-W3 波（"失落之城"Kindred 机制，23 张卡）新增原语：
`kindred_played` 每局玩家列表（每次出牌推入类型：法术推 `Spell`、随从推
主种族；回合结束时清空——"本回合早些时候"的官方语义）、
`next_kindred_twice`（TLC_251 战吼设置、下一次 OnPlay Kindred 结算双次并
清除）、`next_murloc_discount` / `next_murloc_divine_shield`（TLC_428：
下一个鱼人出牌费减 1 / 入场即带圣盾，出牌时消费）。四个解析点全部忠实：
费用折扣（TLC_366/600/816，push 之前检查，只数更早的同型卡 ≥1）、OnPlay
（TLC_107/226/243/428/429/440/447/519/815/825/903，基础结算之后——
随从在入场与抉择块之后、法术在基础法术效果之后）、战吼修饰（TLC_454/463/
829 专用变体替换战吼、TLC_482 附加结算）、抽牌修饰（TLC_223 火系过滤走
W1 真实法术学派注册表、TLC_236 精确 1/2/3/4 费扫描、TLC_432 亡语随从
≤3 费）。Torga 的牌库扫描忠实：自上而下扫实际牌库找第一张 Kindred 注册
卡，再找其同型卡；无匹配则不抽——经典|核心牌库下空抽就是官方行为。
简化登记：六张双种族卡（TLC_102/223/243/432/463/482）的第二种族经
`apply_card_keywords` 落到实体上、喂部落协同，但 Kindred 计数只认主种族
（官方语义任一种族都算）；TLC_815 随机 4 费嘲讽池走 D2（ALL_CARDS 4 费
随从、排除衍生物；被召随从自身战吼照常结算）；TLC_519 固定召 TLC_519t
毒液喷吐者（单元素 D2 池）；TLC_482 附加结算触发全部友方余烬火花亡语
（官方"触发它们"仅指战吼召出的两只）；TLC_251 双次标志只被 OnPlay 结算
消费（费用折扣与战吼修饰不翻倍）；TLC_107 攻击钩子按卡 ID 硬编码
（湖鲟先例，非通用攻击触发组件）；TLC_428 的下一个鱼人标志为玩家级标志、
由下一次鱼人出牌消费（滑翔者本人永不消费自己的标志）；TLC_825 的 Kindred
目标经 PlayCard 目标传入，但 `rl/env.rs` play_targets 只提取战吼目标——
RL 视图无法表达。

F5 coverage: `tlc_w3_kindred_requires_same_type_earlier_this_turn`,
`tlc_w3_spell_kindred_fires_after_any_spell`, `tlc_w3_cost_discount_kindred`,
`tlc_w3_next_kindred_triggers_twice`, `tlc_w3_stormbrewer_gains_rush`,
`tlc_w3_stormdrake_immune_this_turn`,
`tlc_w3_torga_draws_kindred_and_activator`,
`tlc_w3_kodo_high_attack_replaces_lowest`,
`tlc_w3_razidir_opponent_hand_discard`,
`tlc_w3_slagclaw_triggers_cinder_deathrattles`,
`tlc_w3_devilsaur_gains_destroyed_stats`, `tlc_w3_cryosleep_draws_another`,
`tlc_w3_matriarch_attack_damage`, `tlc_w3_queen_hero_attack`,
`tlc_w3_thrasher_drawn_spell_damage`, `tlc_w3_dread_raptor_costs_zero`,
`tlc_w3_kindred_resets_next_turn`, `tlc_w3_voidbulb_summons_taunt_4cost`,
`tlc_w3_ambush_predators_spitter`, `tlc_w3_hot_spring_glider_murloc_flags`,
`tlc_w3_hybridization_draws_each_cost`,
`tlc_w3_bookkeeper_copy_does_not_loop` (22 scenarios in
`tests/differential.rs` — every Kindred shape: activation, cost discount,
the twice flag, keyword gains, Torga's scan, battlecry replaces/add-on,
drawn-card modifiers, the turn-end reset, the random pools and the no-loop
bookkeeper copy) plus the three registry unit tests in `src/cards/kindred.rs`.
Full `cargo test` fully green (all suites, incl. every `tlc_w1_*`/`tlc_w2_*`
scenario — 817 passed, 1 ignored), `cargo fmt` clean, `cargo clippy
--all-targets` zero warnings.

### 17. 2025–2026 expansions M2-W4a — the Un'Goro main-set wave, first split (96 cards) 🔓 registered

The registered simplifications of the M2-W4a wave (`src/cards/exp_tlc_w4a.rs`):
the 96 non-legendary Un'Goro collectible cards not landed in W1–W3 (97 by
the rule "TLC_* collectible with rarity != LEGENDARY minus the 22 landed in
W1–W3"; TLC_249 Sizzling Cinder already landed in W3, so 96 new cards get
written — the card-by-card list is in the module header), plus 17
handwritten tokens (11 from the card dump, 6 absent from the dump and
written per the card texts they serve) and the Storm the Gates sidequest.
The faithful core of the wave: the "Discovered this
turn" flag (`Player::discovered_this_turn`, set by every discover
resolution, cleared at the turn end) drives Storage Scuffle's cost (0) and
Unearthed Artifacts' 4-Cost summon; the Map chain is a pending
`(Entity, Vec<String>)` entry consumed once by the discovered card's play;
Cursed Catacombs and Bloodpetal Biome mark the discovered card Temporary
(the W2 primitive) and Spelunker's one-time next-Temporary discount lands
in the play-cost pipeline; the quest-played flag unlocks Questing
Assistant. As with §14–§16, these handwritten expansion cards are not in
the RL pool (classic + core 668/659), so the rows are informational: they
keep the code's simplifications traceable to the ledger. Each row stays
open until its mechanism lands.

| ID | Card | Simplified | When real |
| --- | --- | --- | --- |
| TLC_514/434/334/461/449 | Merchant of Legend / Paleomancy / Relic of Kings / Scrappy Scavenger / Bloodpetal Biome | The D2 discover pools: LegendaryMinion, UndeadMinion, SpellCostGE8, CostEqualRemainingMana and TemporaryOneCostMinion are in-window (Classic\|Core) filtered ALL_CARDS samplings — the official discover is a rarity-weighted three-option pick from the full format pool | the official discover procedure |
| TLC_435/442/464/824/900 | Crypt Map / Submerged Map / Mountain Map / Odd Map / Hive Map | The five in-expansion discover pools: FrostRune is a fixed four-card table (the W1–W3 Frost Rune cards), Murloc / BeastOddAttack / FelSpell in-window filtered subsets, MinionOfUnplayedType keyed on the primary race of the cards played this game — the same D2 sampling | official per-pool discovers |
| TLC_109 | Relic Miner | The same-rarity discover resolves from the static rarity table (in-window collectible + Un'Goro collectible cards); a destroyed top card outside the table (a token etc.) fizzles the discover instead of falling back | the official rarity pool |
| TLC_435/442/464/515/824/900 | the six Map cards | "If you play the discovered card this turn, Discover another" is one random other option added to the hand — a `map_pending` entry consumed by the discovered card's play; one extra card, no second three-option choice | the official re-discover chain |
| TLC_518 | Interrogation | The three Tortollan Ninjas are shuffled into the deck as plain cards — the official Summoned-When-Drawn keyword is not implemented (the Emerald Portal precedent); the ninjas enter play only when drawn and played | a Summoned-When-Drawn pipeline |
| TLC_467 | Whispering Stone | The gotten Fel spells "cost Health instead of Mana" via the CostHealth marker read by the play path — one play-time payment, no other cost-health interactions | the official cost-health resource |
| TLC_444/465 | Story of Galvadon / Stranglevine | The random Bonus Effect pool is the six-keyword engine pool (Taunt / Divine Shield / Poisonous / Windfury / Elusive / Stealth) — a D2 random sample with no weighting | the official bonus-effect pool |
| TLC_436 | Reanimated Pterrordax | "Costs Corpses instead of Mana": the corpse counter is a simplified player-level resource (one per friendly Undead death), 5 spent at play — no corpse types or other corpse interactions | the official corpse system |
| TLC_252/t | Dissolving Ooze / Bones | The two Bone spells copy the destroyed minion's Attack onto the Bone entity (the spell deals damage equal to it); the official Health half is approximated away and the Bones are plain 0-cost spells in hand | the official Bones (Attack + Health) |
| TLC_EVENT_400 | Storm the Gates | The Zombeast reward is a random Beast with a (3) cost reduction — no build-a-beast discover; the sidequest itself is faithful ("Play 3 Beasts or Undead", 1-cost quest slot, excluded from the quest-played flag) | the official Zombeast craft |
| TLC_439 | Wave of Tar | The "Enemy minions cost (2) more NEXT TURN" tax is a caster-side flag read at the enemy's play-cost time and cleared at the caster's next turn start — the official window (the opponent's following turn) is exactly covered | the official next-turn window |
| TLC_253 | Petrified Ogre | Dormant is modeled as can't-attack; the official 50% awaken-or-buff roll is a deterministic +2/+2 at each of the owner's turn starts (no awaken flip) | the official Dormant flip |
| TLC_827 | Grazing Stegodon | The end-of-turn "random Beast in hand +1/+1, else a random Beast in your deck" is simplified to "+1 Attack to itself at the end of your turn" | the official hand-then-deck buff |
| TLC_221 | Sizzling Swarm | "Deal 3 damage to a minion" is resolved as 3 damage to a random enemy character (no target choice) | the official targeted damage |
| TLC_483 | Vault Breaker | "After you Discover a card, reduce its Cost by (1)" is folded into the discover's get — the discovered card's cost is reduced once at discovery time (a persistent enchantment would outlive Vault Breaker's removal) | the official after-discover enchantment |

中文小结（同上）：M2-W4a 波（"失落之城"主系列第一批，96 张非传说卡 +
17 张手写衍生物）的新原语大部分忠实：`discovered_this_turn` 每玩家标志
（每次发现结算置位、回合结束清除）驱动储能间斗殴费用 0 与出土文物召唤
4 费随从；地图链是 `map_pending: Option<(Entity, Vec<String>)>`，由发现
卡本回合的出牌消费一次——随机补一张其他选项入手（六张地图卡共用一个
形状）；被诅咒的地下墓穴与血瓣生态园给发现卡打 Temporary 标记（W2
原语）、洞穴探索者的下一次临时卡减 2 费标志在出牌费用管线里消费；
quest_played 标志（除风暴之门外所有任务卡出牌时置位）解锁求知助手。
已登记简化：D2 发现池（十个 DiscoverPool 变体都是经典|核心窗口内过滤的
ALL_CARDS 抽样或固定表；矿工遗骸的稀有度发现走静态稀有度表、表外卡
（如衍生物）被毁则发现直接落空）；六张地图卡"本回合打出则再次发现"
简化为出牌时随机补一张其他选项（无二次三选一）；审讯把三张忍者普通
洗入牌库（官方 Summoned-When-Drawn 关键词未实现，绿翼先例）；低语之石
的血费标记（CostHealth）只走出牌支付分支；加尔瓦顿故事/缠藤者的随机
增益池是六关键词引擎池（嘲讽/圣盾/剧毒/风怒/扰魔/潜行，D2 无权重）；
复生翼手龙尸体机制简化为玩家级尸体计数（友方亡灵死亡 +1、出牌扣 5）；
溶解软泥怪的两张骨骼牌把被毁随从的攻击力复制到骨骼实体上（官方
Attack + Health 两半只做了一半、且是手里普通 0 费法术）；风暴之门奖励
为随机野兽减 3 费（无自定义野兽合成）；潮汐之波"下回合敌方随从费 +2"
是施法者侧标志、敌方出牌费用时读取、施法者下一个回合开始时清除（官方
窗口正好覆盖敌方随后那一回合）；石化食人魔的休眠建模为不能攻击、
官方 50% 苏醒骰退化为每回合开始确定性 +2/+2；放牧剑龙"回合结束时给
手中随机野兽 +1/+1、没有则改牌库"简化为回合结束自身 +1 攻击；余烬蜂群
"对一个随从造成 3 点伤害"改为对随机敌方角色；宝库破毁者的"发现后减 1
费"折进发现入手时刻（一次性降价，持久附魔会活过破毁者离场）。

F5 coverage: `tlc_w4a_bloodpetal_biome_grants_temporary`,
`tlc_w4a_tunnel_terror_temporary_tokens`,
`tlc_w4a_spelunker_discounts_next_temporary`,
`tlc_w4a_cursed_catacombs_marks_deck_card_temporary`,
`tlc_w4a_storage_scuffle_free_after_discover`,
`tlc_w4a_unearthed_artifacts_escalates_after_discover`,
`tlc_w4a_vault_breaker_discounts_discovered`,
`tlc_w4a_map_card_picks_second`,
`tlc_w4a_questing_assistant_fires_after_quest_played`,
`tlc_w4a_cloud_serpent_copies_hand_elemental`,
`tlc_w4a_curious_explorer_reduces_enemy_hand_cost`,
`tlc_w4a_platysaur_discards_drawn_card_on_death`,
`tlc_w4a_carnassa_shuffles_raptors`, `tlc_w4a_interrogation_shuffles_ninjas`,
`tlc_w4a_skyscreamer_eggs_summon_four`,
`tlc_w4a_relic_miner_destroys_top_draws_rarity` (16 scenarios in
`tests/differential.rs` — every new primitive of the wave: the Temporary
creators (Location activation discover / Cursed Catacombs deck pick /
Tunnel Terror deathrattle) with the W2 discard-at-turn-end lifecycle,
Spelunker's one-time discount, the "Discovered this turn" flag driving
Storage Scuffle / Unearthed Artifacts / Vault Breaker, the Map chain
(armed, consumed once, one other option added), the quest-played flag,
the Cloud Serpent hand copy, the enemy-hand cost reduction, Platysaur's
linked draw, the two shuffle-into-deck shapes and Relic Miner's rarity
discover). Full `cargo test` fully green (all suites, incl. every
`tlc_w1_*`/`tlc_w2_*`/`tlc_w3_*` scenario — 833 passed, 1 ignored),
`cargo fmt` clean, `cargo clippy --all-targets` zero warnings.

### 18. 2025–2026 expansions M2-W4b — the Un'Goro main-set wave, second split (14 legendary cards + 1 token) 🔓 registered

The registered simplifications of the M2-W4b wave (`src/cards/exp_tlc_w4b.rs`):
the 14 Un'Goro legendary cards (TLC_100/106/110/228/241/257/452/480/522/
624/810/811/836/841) plus the TLC_241t "Call the Threshfleet!" token.
Most of the wave is faithful: Endbringer Umbra re-triggers the Deathrattles
of up to five friendly minions that died this game (the friendly graveyard
IS the died-this-game log); Krog's turn-end set makes every enemy minion a
1/1 (base write + damage clear + enchantment strip); Opu's Battlecry /
Combo / Deathrattle each deal 1 to all enemy minions; Nablya summons fresh
base-stat copies of each damaged friendly minion with Rush (the copies
carry no battlecry or combo — the engine's MinionSummoned handler fires
battlecries on effect summons, so the components are stripped to keep the
copy fresh); Archaios sets the attacker's Health to its own effective
Health through a new `FriendlyMinionAttacked` trigger event (the `Attacked`
event is pinned to the attacker and unusable); Niri's played-card trigger
doubles 1-Cost minions and double-casts 1-Cost spells (one CardPlayed
trigger, the effect branches on the subject's card type — the Trigger
component is a single slot); Loh's battlecry sets every minion cost to (5)
for the game; Ido's start-of-turn effect grants the TLC_241t token while
he is alive, and the token is a real 2-Cost Holy spell (+2/+2 and Divine
Shield). As with §14–§17, these handwritten expansion cards are not in the
RL pool (classic + core 668/659), so the rows are informational: they keep
the code's simplifications traceable to the ledger. Each row stays open
until its mechanism lands.

| ID | Card | Simplified | When real |
| --- | --- | --- | --- |
| TLC_100 | Elise the Navigator | The battlecry's custom-Location craft is a deck check against the `Player::starting_deck` snapshot (10 cards of different Costs) that only sets the `elise_location_crafted` marker — no custom Location entity, activation or text exists | the official custom Location |
| TLC_110 | City Chief Esho | "Wherever they are" uses the Grimestreet shape: base-stat buffs for hand and deck minions, permanent enchantments for board minions; the deck check itself is faithful (current deck, vacuous empty-deck pass) | the official wherever-buff |
| TLC_228 | Bralma Searstone | "Your Elementals deal 1 extra damage" is a damage-pipeline hook at the Goldrinn entry point (an aura approximation): any damage source owned by the Bralma player that carries the Elemental race deals +1 while TLC_228 is alive — spell sources carry no race and are naturally excluded | a true aura |
| TLC_452 | Titanographer Osk | The Titan mechanism is not implemented — the battlecry does nothing (smoke-pinned) | the official Titan abilities |
| TLC_810 | High Cultist Herenn | "They fight!" is one exchange: each summoned copy deals damage equal to its Attack to the other once, through the normal damage pipeline (deaths, deathrattles and shields resolve); the deck itself is untouched (copies via `resolve_summon`) | the official fight resolution |
| TLC_841 | Entomologist Toru | The 0/1 Jar transform/release mechanic is not implemented — the battlecry does nothing (smoke-pinned) | the official Jars |

中文小结（同上）：M2-W4b 波（失落之城 14 张传说卡 + TLC_241t 衍生物）
本波大多忠实：恩布拉亡语末日再触发本局已死的至多 5 个友方随从的亡语
（友方墓地即"本局已死"日志）；克罗格回合结束把敌方随从全部置为 1/1
（写基础值 + 清伤害 + 剥附魔）；奥普的战吼/连击/亡语各对全部敌方随从
造成 1 点伤害；纳布利亚为每个受伤友方随从召唤全新基础身材的复制并给
复制突袭（复制剥离战吼/连击组件——本引擎 MinionSummoned 处理器会对
效果召唤的随从触发战吼，剥离才能保持复制品"全新"）；阿凯欧斯经新增
FriendlyMinionAttacked 触发事件把攻击者的生命值设为自身的有效生命值
（Attacked 事件钉在攻击者身上不可用）；妮莉的打出卡触发对 1 费随从
翻倍属性、对 1 费法术双重施放（单个 CardPlayed 触发器、效果按主题卡
类型分支——Trigger 组件是单槽）；洛的战吼使全场随从费用恒为 5；伊多的
回合开始效果在存活时授予 TLC_241t 衍生物（真实 2 费神圣法术，+2/+2 并
给予圣盾）。已登记简化：Elise 的定制地点铸造退化为对开局牌库快照的
"10 张不同费用"检查并置 elise_location_crafted 标志（无地点实体）；
Esho"任意位置"采用 Grimestreet 形状（手牌/牌库改基础数值、场上永久
附魔），牌库检查本身忠实（当前牌库、空牌库空洞通过）；Bralma 的元素
使伤害 +1 是 Goldrinn 同入口的伤害管线钩子（光环近似：Bralma 玩家所有
带元素种族的伤害源 +1，TLC_228 存活时生效；法术来源无种族自然排除）；
Osk（泰坦机制未实现）与 Toru（0/1 罐子变形/释放未实现）战吼为空
（冒烟测试钉住身材）；Herenn"他们互殴！"简化为一轮交换（各自攻击力
互打一次，走正常伤害管线——死亡/亡语/圣盾正常结算），牌库本身不动
（resolve_summon 复制）。

F5 coverage: `tlc_w4b_umbra_triggers_five_dead_deathrattles`,
`tlc_w4b_krog_sets_enemy_minions_to_one`,
`tlc_w4b_opo_fan_of_knives_via_battlecry_combo_deathrattle`,
`tlc_w4b_nablya_copies_damaged_minions_with_rush`,
`tlc_w4b_archaios_sets_attacker_health`,
`tlc_w4b_niri_doubles_one_cost_minions`,
`tlc_w4b_niri_casts_one_cost_spells_twice`,
`tlc_w4b_loh_minions_cost_five`, `tlc_w4b_ido_grants_token_while_alive`,
`tlc_w4b_esho_deck_check_buffs_minions`,
`tlc_w4b_bralma_elementals_deal_extra_damage`,
`tlc_w4b_herenn_summons_two_deathrattle_minions_and_fight`,
`tlc_w4b_elise_checks_starting_deck`,
`tlc_w4b_osk_toru_smoke_pins` (14 scenarios in `tests/differential.rs` —
one per legendary card: the graveyard-scan deathrattle re-trigger, the
turn-end 1/1 set with damage/enchantment clearing, the Battlecry/Combo/
Deathrattle triple Fan of Knives, the fresh Rush copies with battlecries
stripped, the new FriendlyMinionAttacked trigger with the subject==source
guard, the CardPlayed cost-1 branch (minion doubling and spell
double-cast), the minion cost-(5) set, the alive-only token grant and the
token's buff + Divine Shield, the current-deck tribe check with the
wherever-buff, the Elemental damage hook, the two-deathrattle-minions
fight with the untouched deck, the starting-deck cost check, and the two
simplified smoke pins). Full `cargo test` fully green (all suites, incl.
every `tlc_w1_*`/`tlc_w2_*`/`tlc_w3_*`/`tlc_w4a_*` scenario — 847 passed,
1 ignored), `cargo fmt` clean, `cargo clippy --all-targets` zero warnings.

### 19. 2025–2026 expansions M2-W4c — the Un'Goro miniset, Festival of the Devilsaur (38 cards + 3 tokens) 🔓 registered

The registered simplifications of the M2-W4c wave (`src/cards/exp_tlc_w4c.rs`):
the 38 Un'Goro miniset cards (DINO_130/131/132/136/137/138/400/401/402/403/
404/405/406/407/408/409/410/411/412/413/414/415/416/417/419/421/422/424/426/
427/428/429/430/431/432/433/434/435) plus the three tokens (DINO_130t Little
Longneck, DINO_136t Ravenous Raptor, DINO_410t Khelos). Most of the wave is
faithful: the four Kindred cards extend the W3 tables (Diabolus Rex's edge
minion splash, Firegill's give-others-Rush, Chillspine's damage-plus-freeze
BattlecryModifier, Crater Experiment's summon-a-copy); the five Masks
introduce the `SetStats` family (Bat Mask fills the board with 1/1 copies,
Devilsaur Mask grants Charge, Behemoth Mask forces a random enemy minion to
attack through the normal attack pipeline, Sheep Mask attaches a real
DamageAllMinions deathrattle, Panther Mask grants Stealth and draws);
Hatching Ceremony's "end of your NEXT turn" is a two-tick per-player
countdown (armed at 2, decremented at each owner turn end, buff at 0);
Soulrest Ceremony's turn-end die is a marked-entities sweep with real
DamageDealt events (Corruption pattern, so deathrattles fire); Barricade
Basher's armor hook is wired at a `grant_armor` chokepoint (all 16 armor
gain sites route through it); The Great Dracorex fires a new
`AttackedEnemyMinion` trigger (the defender is the subject, so the splash
excludes the attacked minion); Seismopod reuses the W4b wherever-buff;
Skittish Saucier's adjacent-hand discount reads the recorded
`last_played_hand_index`. As with §14–§18, these handwritten expansion
cards are not in the RL pool (classic + core 668/659), so the rows are
informational: they keep the code's simplifications traceable to the
ledger. Each row stays open until its mechanism lands.

| ID | Card | Simplified | When real |
| --- | --- | --- | --- |
| DINO_407 | Mirrex the Crystalline | "While in hand, this is a 3/4 copy of the last minion your opponent played" is a static 3/4 in hand and in play — no hand-dynamic copy mechanic exists | the official hand transform |
| DINO_409 | Techysaurus | The "costs (1) less per card played this game that didn't start in your deck" discount is not implemented (no card-origin tracking) — the card always costs 7 | official origin tracking |
| DINO_410 | The Egg of Khelos | The crack chain is skipped: the deathrattle summons the final 20/20 Taunt Beast (DINO_410t Khelos) directly — no egg tokens, no 5-break countdown | the official egg chain |
| DINO_414 | Tribute Dance | The two-choice transform is a random transform: one random in-window minion transforms into a random in-window minion (the TransformToMinion set semantics — stats, cost, card id, attack usage reset, static keywords) | the official two-choice pick |
| DINO_430 | Beast Speaker Taka | The battlecry/discover-to-deathrattle card link is dropped: the battlecry gains the stats of a random Legendary Beast and the deathrattle summons a random Legendary Beast — independent picks, no stored card | the official stored-card link |
| DINO_435 | Crater Experiment | The ALL-tribe Kindred check is approximated to Beast (the primary-race convention — a card with every tribe counts as its dump-listed first race) | an all-tribes kindred check |
| DINO_131 | Possessed Animancer | The summoned Beast is a copy — the deck is untouched (the Herenn §18 copy-summon convention) | a deck pull |
| DINO_427 | Costume Merchant | "Get a random Mask from another class" is a D2 random draw over the fixed 5-Mask pool (Bat/Devilsaur/Behemoth/Sheep/Panther — all neutral here, so no class filter is lost) | a full-card random |
| DINO_434 | Raptor-Nest Nurse | The 1-Cost random minion / 1-Cost random spell are D2 draws over the in-window one-Cost pools | the official Discover-less random |
| DINO_412 | Tortotem | "Get a random minion with multiple minion types" is a D2 draw over a fixed pool of the in-window multi-tribe minions (CORE_TTN_866 Mythical Terror — the only one, so the draw is deterministic) | the full multi-tribe filter |
| DINO_415 | Story of Umbra | The "discover a Deathrattle minion of cost 5 or more, summon and trigger it" is a D2 random over the in-window Deathrattle minions of cost ≥5 (the summoned minion's deathrattle fires immediately) | the official Discover |
| DINO_424 | Hero's Welcome | The legendary discovery is a D2 random over the in-window legendary minions (the classic LEGENDARY_CLASSIC membership), set to 10/10 | the official Discover |
| DINO_426 | Ritual of Life | The 3-Cost discovery is a D2 random over the in-window 3-Cost minions, summoned as a 2/3 copy | the official Discover |
| DINO_431 | Atlasaurus | The "random Taunt minion of cost 5 or more" is a D2 random over the in-window Taunt minions | a full-card random |
| DINO_433 | Guard Duty | The 6/4/2-Cost Taunt trio is three independent D2 random summons | three full-card randoms |

中文小结（同上）：M2-W4c 波（失落之城迷你系列"魔鬼龙嘉年华"，38 张 +
3 衍生物）本波大多忠实：四张亲和卡扩展 W3 表（迪亚波罗王撕裂边缘随从、
火鳃给其他随从突袭、寒脊剑龙伤害+冻结的 BattlecryModifier、火山实验
召唤自身复制）；五张面具引入 SetStats 家族（蝙蝠面具填满 1/1 复制、
恶魔龙面具给冲锋、巨兽面具强制随机敌方随从走正常攻击管线打它、绵羊
面具附加真实的"对所有随从造成 2 点伤害"亡语、黑豹面具给潜行并抽 2）；
孵化仪式"你下个回合结束时"是每人两跳计数器（施放置 2、每个己方回合
结束减 1、到 0 生效）；安魂典礼的回合结束死亡是标记实体清扫 + 真实
DamageDealt 事件（Corruption 模式，亡语正常触发）；路障拳手护甲钩子
接在 grant_armor 咽喉点（全部 16 处护甲获得都走它）；大德雷克龙用新
AttackedEnemyMinion 触发事件（以防御者为主题，溅射排除被攻击的随从）；
震地龙复用 W4b 的"任意位置"增益；害羞盘碟商的相邻手牌减费读取记录的
last_played_hand_index。已登记简化：Mirrex（手牌动态复制退化为静态
3/4）；Techysaurus（非起始卡牌来源追踪不存在，固定 7 费无折扣）；
基洛之卵（裂壳链跳过，亡语直接召唤 20/20 嘲讽野兽）；贡舞（二选一变形
退化为随机变形，走 TransformToMinion 置数值语义）；野兽语者塔卡（战吼
/亡语的发现关联丢弃——战吼随机传奇野兽的身材、亡语随机召唤传奇野兽，
两次独立选取）；外加代理登记的近似：火山实验 ALL 种族按主种族约定
近似为野兽；被附身的亡灵巫师召唤的是牌库复制（牌库不动）；化装商人
D2 固定 5 面具池（皆中立，无职业过滤损失）；巢穴护士 D2 一费池；
图腾 D2 多重种族池（窗口内仅 CORE_TTN_866 神话恐惧，选取确定）；
暮光之影故事/英雄欢迎仪式/生命仪式/腕龙/值日哨岗为 D2 随机召唤。

F5 coverage: `tlc_w4c_diabolus_rex_hits_edge_minions`,
`tlc_w4c_firegill_gives_others_rush`, `tlc_w4c_chillspine_freezes`,
`tlc_w4c_crater_experiment_copies_self`, `tlc_w4c_bat_mask_sets_and_fills`,
`tlc_w4c_devilsaur_mask_sets_charge`, `tlc_w4c_behemoth_mask_forces_attack`,
`tlc_w4c_sheep_mask_deathrattle`, `tlc_w4c_panther_mask_sets_stealth_draws`,
`tlc_w4c_dracorex_aoe_on_attack`, `tlc_w4c_hatching_ceremony_next_turn_buff`,
`tlc_w4c_hollow_direhorn_corpse_reborn`, `tlc_w4c_soulrest_turn_end_dies`,
`tlc_w4c_seismopod_wherever_buff`, `tlc_w4c_saucier_reduces_adjacent`,
`tlc_w4c_horn_of_feasting_outcast`, `tlc_w4c_basher_armor_trigger`,
`tlc_w4c_tortotem_multi_tribe_pool` (18 scenarios in `tests/differential.rs`
— one per Kindred card with its inactive/active split, each of the five
Masks, the Dracorex splash excluding the attacked minion, the two-tick
hatching countdown buffing only on the owner's NEXT turn end, the corpse
spend with the corpse landing after the trigger, the turn-end soulrest
sweep with deathrattles firing, the wherever-buff touching hand and deck,
the adjacent-hand discount on edge and middle plays, the Outcast
immune-only-for-the-turn raptors, the armor-gain +2/+2 and triggered
attack, and the deterministic multi-tribe pool). Full `cargo test` fully
green (all suites, incl. every `tlc_w1_*`/`tlc_w2_*`/`tlc_w3_*`/
`tlc_w4a_*`/`tlc_w4b_*` scenario — 865 passed, 1 ignored), `cargo fmt`
clean, `cargo clippy --all-targets` zero warnings.
### 20. 2025–2026 expansions M3-W2a — Across the Timeways sub-roadmap W2, first split (120 cards + 11 tokens) 🔓 registered

The registered simplifications of the M3-W2a wave (`src/cards/exp_tmw_w2a.rs`):
the 118 non-legendary TIME_* collectible cards (53 COMMON / 38 RARE / 27 EPIC)
plus the two legendaries implemented in this wave (TIME_038 Mister Clocksworth,
TIME_063 Timelord Nozdormu — their effects are W2a shapes), 120 cards in total,
plus the 11 tokens they produce (TIME_006t1 Mirrored Mage, TIME_017t Tank,
TIME_025t Shred of Time, TIME_059t Living Paradox, TIME_434t Temporal Shadow,
TIME_443t Sargeran Felhound, TIME_610t2 Anomalous Shade, TIME_700t
Chronological Drake, TIME_704t Highborne Pupil, TIME_870t Coliseum Tiger,
TIME_873t Coliseum Crocolisk). **The M3-W2a spec's "114" count is wrong** — the
generated baselines in `cards/cards.json` hold 118 non-legendary TIME_* cards;
the discrepancy is disclosed in the sets.rs and exp_tmw_w2a.rs headers. As
with §14–§19, these handwritten expansion cards are not in the RL pool
(classic + core 668/659), so the rows are informational: they keep the code's
simplifications traceable to the ledger. Each row stays open until its
mechanism lands.

The wave's headline mechanics are FULL primitives: **Dormant** (the new
`core::component::Dormant { turns }` component — while dormant a minion cannot
attack, cannot be targeted (the Stealth-like filter in `validate_attack`) and
takes no damage (the DamageDealt arm early-return); the countdown decrements at
the owner's turn start and the component is removed at 0 (awaken) — cards:
TIME_046 Cyborg Patriarch (Dormant 3), TIME_063 Timelord Nozdormu (Dormant 5),
TIME_058's deathrattle summon (Dormant 2), TIME_442's imprisoned enemy minion
(Dormant 10,000); TIME_022 Perennial Serpent discounts (4) while ANY minion is
dormant, via the cost pipeline); **Rewind** (the W1 machinery — the 17 Rewind
cards replay recorded effects automatically through the play path); **Shred of
Time** (below); the **pool-open** registry (5 cards); and **Locations**
(TIME_044 Past Gnomeregan, TIME_436 Past Conflux, TIME_810 Past Silvermoon —
the Core Set W8 representation, activation effects in the battlecry slot;
their `card_type|health|durability` fields are `expansion_differential_rebalanced`
because the generator predates the Location CardType).

| ID | Card | Simplified | When real |
| --- | --- | --- | --- |
| TIME_025t | Shred of Time | "Casts When Drawn" is simplified per the established cast-when-drawn precedent (EDR_445pt3 Emerald Portal, §14): a playable 0-cost spell dealing 3 damage to your hero — drawn as a normal card | the official draw-time cast |
| TIME_028 | Fatebreaker | "Cast a Shred of Time from your deck" is a deck scan + removal that applies the Shred's own effect inline, then +3/+3 (Lifesteal is real — the self-damage heals right back) | the official cast-from-deck timing |
| TIME_029 | Ruinous Velocidrake | Same cast-from-deck shape, then summons a copy of this (the copy sheds its battlecry) | the official cast-from-deck timing |
| TIME_030 | Divergence | "Split a random minion in your hand into two halves" is approximated by a plain copy — the split-card halves mechanic does not exist | the official split-card halves |
| TIME_036 | Royal Informant | Pool-open: the "get a copy OR increase its Cost by (2)" either/or is a random pick (the established pool-open convention) | the official choice |
| TIME_039 | Deja Vu | Pool-open discover: a D2 random pick over the opponent's hand cards (a copy joins your hand) | the official Discover |
| TIME_041 | Futuristic Forefather | Pool-open: the guess is always right — gain +4 Health (no guess mechanic) | the official guess |
| TIME_432 | Intertwined Fate | Pool-open: one combined choice (three deck ids, then three enemy-hand ids); the other side resolves as a random copy (D2) | the official double Discover |
| TIME_876 | Shapeshifter | Pool-open: reads the opponent's hand for its transform target | the official target read |
| TIME_063 | Timelord Nozdormu | "After you play a card from the newest expansion, awaken 1 turn sooner" — the newest-expansion check has no meaning in the closed pool; always awakens at turn 5 | an expansion-origin check |
| TIME_101 | Misplaced Pyromancer | "Whenever you Shatter a card" — the Shatter mechanic is absent from the pool; the trigger is a no-op | the Shatter mechanic |
| TIME_214 | Flux Revenant | "Whenever you WOULD damage this with a Nature spell, it gains +2/+1 instead" — no would-damage interception hook; a plain 1/4 Taunt | the official interception |
| TIME_217 | Stormrook | "Whenever you WOULD damage this with a Nature spell, summon a random 5-Cost instead" — same unmodeled interception; a plain 5/5 | the official interception |
| TIME_021 | Doomsday Prepper | Outcast is rendered as a plain battlecry (the outcast-position condition is unmodeled); the Immune expiry is the end of the current turn (the registered Kaldorei precedent) | an outcast check + until-your-next-turn expiry |
| TIME_002 | Aeon Wizard | "Random spells from your class" — the class is approximated by the class-card union (RandomPool::ClassSpell) | per-class filtering |
| TIME_014 | Instant Multiverse | "Summon 12 Mana worth of random minions" — each pick costs at most the remaining Mana so the summoning terminates | the official pool walk |
| TIME_016 | Neon Innovation | "A Paladin Mech from the past" — the past is the active window; the +5/+5 rides the stashed modifier consumed at the pick (D2) | the full-card random |
| TIME_027 | Tachyon Barrage | The split damage is one-damage pings that never scale with spell damage (matching the official ImmuneToSpellpower) | the official split with spellpower |
| TIME_033 | Druid of Regrowth | "Cast 2 random Nature spells" — each pick resolves the spell's own effect against random targets | the official targetless casts |
| TIME_038 | Mister Clocksworth | The random Legendary pool spans the active window plus TIME_063 (the closed pool's only other legendary) | the full legendary pool |
| TIME_043 | PMM Infinitizer | The "can't attack heroes this turn" temp restriction is real (the W1 machinery); the targetable friendly-minion pick is a D2 random | the official player choice |
| TIME_044/436/810 | Past Gnomeregan / Past Conflux / Past Silvermoon | Location activations: the targetable pick is a D2 random | the official player choice |
| TIME_054 | Time Skipper | "At the end of each player's turn" is a per-ID EndTriggers hook (a Trigger component cannot see both boards) | a both-boards trigger |
| TIME_057 | Wizened Truthseeker | Full: cost enchantments stripped and TurnCostReducer markers removed; attack/health enchantments stay | — |
| TIME_212 | Lightning Rod | "Deal 2 damage to a friendly minion" — the friendly targetable pick is a D2 random | the official player choice |
| TIME_213 | Primordial Overseer | "If you've cast a Nature spell while holding this" — "while holding this" is approximated by the game-wide Nature counter | hand-origin tracking |
| TIME_218 | Static Shock | "Deal 1 damage to a minion" — the targetable pick is a D2 random | the official player choice |
| TIME_431 | Amber Priestess | "Restore Health to a character" — the targetable pick is a D2 random | the official player choice |
| TIME_442 | Timeway Warden | The imprison/awaken link is real but rides the (warden, imprisoned) entity pairs on the player record — the Dormant application itself is the full primitive | the official entity link |
| TIME_447 | Power Word: Barrier | "Give a character Divine Shield" — the targetable pick is a D2 random | the official player choice |
| TIME_448 | Solitude | "Discover 2" is a one-pick simplification (a D2 random minion) | the official Discover 2 |
| TIME_613 | Cryofrozen Champion | The random Legendary pool spans the active window plus TIME_063 | the full legendary pool |
| TIME_614 | Liferender | "If your hero's Health changed this turn" is approximated by the per-turn hero-damage counter | full change tracking |
| TIME_615 | Forgotten Millennium | "They cost Health instead of Mana" — the "this turn" expiry is approximated by the CostHealth marker | full until-end-of-turn |
| TIME_620 | Untimely Death | The secret's "the turn after being played" marker also fires on a same-turn death | the official turn-after filter |
| TIME_700 | Chronological Aura | "Lasts 3 turns" — the tick counter rides the player record (summons at each own turn end while > 0) | a duration aura |
| TIME_702 | Ebb and Flow | "If you played a minion while holding this" — "while holding this" is approximated by any minion played this turn | hand-origin tracking |
| TIME_704/704t | Highborne Mentor / Highborne Pupil | The teach link is approximated: the discovered spell joins the hand like any Discover; the Pupil's "cast the taught spell" battlecry is unmodeled | the official taught-spell link |
| TIME_707 | Alternate Reality | "Random Choose One cards from the past" — the pool is the active window | the full past pool |
| TIME_730 | Kaldorei Cultivator | "Discover 2 Beasts" is a D2 random pick (bottom of deck with +5/+5) | the official Discover |
| TIME_770 | Fast Forward | "Pick one to have its Cost reduced by (2)" — the pick is a D2 random | the official player choice |
| TIME_856 | Algeth'ar Instructor | Spell Damage +1 — the official JSON data says +1 although the card text says +2; the engine follows the data | the official text |
| TIME_857 | Alter Time | "Discover two Arcane spells" — the second pick is a one-pick simplification | the official Discover 2 |
| TIME_860 | Faceless Enigma | "Pick one" of the two cast Secrets is a D2 random | the official player choice |

中文小结（同上）：M3-W2a 波（"穿越时光"子路线 W2 第一拆，120 张 +
11 衍生物）的本波核心机制都是完整原语：**休眠**（新增
`core::component::Dormant { turns }` 组件——休眠中不能攻击、不能被指定
（潜行式过滤）、不受到伤害（伤害管线提前返回）；每个己方回合开始倒计时、
归零苏醒——卡牌：机械元老（3 回合）、时间领主诺兹多姆（5 回合）、
贫瘠小翼亡语召唤的随机 2 费随从（2 回合）、时光守望者囚禁的敌方随从
（10,000 回合）；永恒巨蟒在任意随从休眠时减 4 费，走费用管线）；
**重放**（W1 机制，17 张重放卡自动回放已记录效果）；**时光碎片**（下方）；
**开放池注册表**（5 张）；**地点**（过去侏儒城/过去汇聚点/过去银月城，
Core Set W8 表示法，激活效果在战吼槽；其 card_type|health|durability 走
expansion_differential_rebalanced，因为生成器早于地点 CardType）。已登记
简化：时光碎片"抽到时施放"按既有先例（翡翠传送门，§14）退化为可打出
的 0 费法术打自己英雄 3（TIME_028/029"从牌库施放"退化为牌库扫描+移除并
内联应用碎片自身效果）；分歧"把一张手牌随从分裂成两半"退化为普通复制；
皇家线人"获得复制或费用+2"二选一为随机选取；幻影先知"猜中"恒为猜中；
交织命运为一次合并选择（另一侧随机复制）；塑形者读取对方手牌；时间领主
诺兹多姆"新扩展卡"检查在封闭卡池无意义，恒在第 5 回合苏醒；错位火法师
的"粉碎"机制池内不存在，触发为无操作；波动亡魂/风暴鸦的"将要受到自然
法术伤害"拦截未建模（分别是白板 1/4 嘲讽与 5/5）；末日预备者的连击位置
退化为普通战吼、免疫到期取当前回合结束（卡多雷先例）；奥术师维兹南
"从过去"= 活动窗口；多宇宙化身按剩余法力逐次选取保证终止；类职业近似为
职业卡并集；传奇池 = 活动窗口 + TIME_063；时光守望者的"囚禁/唤醒"联动
走玩家记录上的实体对，休眠本身是完整原语；孤独的"发现 2"退化为一次选取
（D2 随机随从）；永恒印记的"改为消耗生命值"用 CostHealth 标记近似
"本回合"；海妖导师 +7/+7 的法术伤害取官方 JSON 数据的 +1（卡面文字写
+2，引擎从数据）；其余"指向性选取"一律按既有约定退化为 D2 随机选取。

F5 coverage: `tmw2a_rewind_replays_previous_effect`,
`tmw2a_clocksworth_legendary_pool`, `tmw2a_dormant_sleeps_then_awakens`,
`tmw2a_perennial_serpent_discount`, `tmw2a_flutterwing_dormant_summon`,
`tmw2a_hopper_shuffles_shreds`, `tmw2a_fatebreaker_casts_shred_from_deck`,
`tmw2a_informant_copies_or_raises`, `tmw2a_deja_vu_discovers_enemy_hand_copy`,
`tmw2a_truthseeker_resets_costs`, `tmw2a_causality_reverses_deck`,
`tmw2a_divine_augur_sets_hand_stats`, `tmw2a_rafaam_ladder_draws_distinct_costs`,
`tmw2a_velocity_discounts_per_damage`, `tmw2a_unknown_voyager_transforms`,
`tmw2a_circadiamancer_reduces_each_turn` (16 scenarios in
`tests/differential.rs` — the rewind replay shape, the D2 legendary pool,
the 3-turn Dormant countdown with the awakening removing the component and
the can't-attack/can't-target sleep, the any-dormant cost discount, the
dormant-at-summon deathrattle, the two shuffled Shreds drawn as playable
0-cost spells, the Fatebreaker deck scan + removal with its real
Lifesteal self-heal (net zero), the pool-open either/or and the enemy-hand
discover, the both-hands cost reset, the full-deck reversal, the set-stats
Augur, the distinct-costs draw skipping the duplicate, the per-damage cost
counter, the SurvivedDamage transform, and the per-turn TurnCostReducer).
Also repaired in this wave: a pre-existing `CardEffectDe` bincode mirror
drift — the 33-variant run (DestroyAndGainStats..BuffAnotherRandomFriendlyDragon,
indices 81-113) sat at De indices 194-226 while the 113-variant run
(CopyRandomEnemyHandCard..AddRandomOutcastCardNextCheaper) held 81-193, so
every variant in that range silently deserialized as a different effect;
the mirror was reordered to match and a structural guard test
(`card_effect_de_mirror_order_matches` in `src/core/effect.rs`) now pins
the declaration order. Full `cargo test` fully green (all suites, incl.
every `tlc_w1_*`/`tlc_w2_*`/`tlc_w3_*`/`tlc_w4a_*`/`tlc_w4b_*`/`tmw1_*`
scenario and the 16 `tmw2a_*` — 888 passed, 1 ignored), `cargo fmt` clean,
`cargo clippy --all-targets` zero warnings.

### 21. 2025–2026 expansions M3-W2b — Across the Timeways sub-roadmap W2, second split (25 legendary cards + 12 tokens) 🔓 registered

The registered simplifications of the M3-W2b wave (`src/cards/exp_tmw_w2b.rs`):
the 25 Across-the-Timeways legendaries (TIME_038 Mister Clocksworth and
TIME_063 Timelord Nozdormu shipped in W2a, PR #152 — they are NOT here) plus
the 12 tokens they produce (TIME_042t Infinite Banana, TIME_209t High King's
Hammer, TIME_211t1 The Well of Eternity, TIME_211t2 Zin-Azshari, TIME_609t1
Ranger Captain Alleria, TIME_609t2 Ranger Initiate Vereesa, TIME_619t
Bwonsamdi, TIME_713t Timeless Chest, TIME_850t Broll Blood Fighter, TIME_850t1
Valeera Blood Fighter, TIME_875t King Llane, TIME_890t2 Karazhan the Sanctum —
TIME_890t Atiesh the Greatstaff is skipped, see the TIME_890 row). As with
§14–§20, these handwritten expansion cards are not in the RL pool (classic +
core 668/659), so the rows are informational: they keep the code's
simplifications traceable to the ledger. Each row stays open until its
mechanism lands.

The wave's headline mechanics are FULL primitives: **the played-log reads**
(`Player::played_minion_ids` — the all-time minion play log — and
`last_played`, the rewind history): TIME_609 Ranger General Sylvanas deals 2
damage to all enemies once per played sister (Alleria/Vereesa), TIME_103
Chromie's deathrattle adds a copy of each of the last `count` distinct played
cards (bounded by MAX_REWIND_HISTORY, count 10); **the DoubleTriggers aura**
(TIME_064 Chrono-Lord Deios — `AuraEffect::DoubleTriggers`, silenceable):
six rules sites re-resolve the pre-captured effect exactly once per site —
minion battlecry, weapon battlecry, deathrattle, hero power, choose-one
branch, end-of-turn triggers (a stacked BattlecryTwice dark gift resolves 3
times, never 4); **Locations** (CardType::Location, the Core Set W8
representation: TIME_446 The Eternal Hold, TIME_211t1 The Well of Eternity,
TIME_211t2 Zin-Azshari, TIME_890t2 Karazhan — activation effects in the
battlecry slot, the play-turn cooldown enforced; TIME_446 is
`expansion_differential_rebalanced` because the generator predates the
Location CardType); **the per-turn play snapshot** (TIME_714 Chrono-Lord
Epoch destroys exactly the minions the opponent played last turn — the
CardPlayed hook maintains `minions_played_this_turn_ids`, the TurnEnded
handler snapshots it into `last_turn_minion_play_ids`); **INFINITY**
(TIME_024 Murozond — a 32-bit engine has no unbounded value; the play effect
arms `Player::murozond_infinite_pending`, the start-of-turn hook sets the
Attack to the shared `INFINITY_ATTACK_CAP` (100, W3 reuse)); **the hand
swap-back** (TIME_706 The Fins — `Player::hand_swap_snapshot`); **the
Corpses spend** (TIME_618 Husk — `Player::corpses`); **the Toki tracker**
(TIME_861 — `Player::toki_pending_spells`); and **the Eternal Hold discount
flag** (`Player::next_demon_cost_one`).

| ID | Card | Simplified | When real |
| --- | --- | --- | --- |
| TIME_005 | Timethief Rafaam | Fabled+ — "Your deck size is 40, but has 10 Rafaams! Battlecry: If you played the rest, destroy the enemy hero" — a plain 10/10 (no Fabled deck construction, no played-all check) | the Fabled+ deck + played-all check |
| TIME_009 | Gelbin of Tomorrow | "Put one of each Aura from your deck into the battlefield" is ONE random deck minion — battlecry-free (the TIME_870 precedent removes the Battlecry component after the summon), the deck entity consumed | the official aura copy-chain |
| TIME_013 | Farseer Wo | Full: the FriendlySpellCast trigger surfaces the three-option Discover over the in-window Nature-spell pool (the engine auto-resolves the Discover — the established convention) | — |
| TIME_020 | Broxigar | Fabled — "Start of Game: Disappear. Kill all 4 Demons from Argus to reappear" — a plain 12/12 with Charge | the official disappear/reappear |
| TIME_024 | Murozond, Unbounded | "Set this minion's Attack to INFINITY" — a 32-bit engine has no unbounded value; the Attack is set to `INFINITY_ATTACK_CAP` (100) | an unbounded value |
| TIME_032 | Chronogor | Full: the owner draws the 2 highest-Cost deck cards, the opponent draws the 2 lowest of the remaining deck (the actual deck entities; the opponent-drawn cards are re-owned into the opponent's hand) | — |
| TIME_042 | King Maluk | Full discard + get an Infinite Banana; the Banana's "Infinite" keyword ("this stays in your hand") is unmodeled — a plain 1-Cost spell | the Infinite keyword |
| TIME_064 | Chrono-Lord Deios | Full: the DoubleTriggers aura doubles Battlecries / Deathrattles / Hero Power / end-of-turn effects, one re-resolve per site; a stacked BattlecryTwice dark gift resolves 3 times, never 4 | — |
| TIME_103 | Chromie | "Draw another copy of cards you've played this game" is bounded by the rewind-history cap (the last `count` = 10 distinct played cards) | an unbounded game history |
| TIME_209 | Muradin, High King | The Hammer's deathrattle "Shuffle this into your deck with +2 Attack permanently" is a plain add-to-hand (no +2 buff, no deck shuffle); the equipped Hammer itself is full (3/4 Windfury) | the official shuffle + empower |
| TIME_211 | Lady Azshara | Full Choose One (option 0 empowers Zin-Azshari, option 1 The Well of Eternity; the other location is replaced — the official "the other gets destroyed"); the copied friendly minion is a D2 random pick | the official player choice |
| TIME_435 | Eternus | Full threshold (the source's effective Health at resolution); the target pick is a D2 random | the official player choice |
| TIME_446 | The Eternal Hold | Location (card_type|health|durability rebalanced — the generator predates the Location CardType); the Demon Discover is engine-auto-resolved (D2); the "if your deck has no minions, your next one costs (1)" flag is real | the official Discover |
| TIME_609 | Ranger General Sylvanas | "If you've played Alleria or Vereesa, repeat for each" — the all-time played-minion log counts effect-summoned copies too (the best available approximation) | the official play-count |
| TIME_618 | Husk, Eternal Reaper | "Give your hero 'Deathrattle: Spend up to 20 Corpses to resurrect with that much Health'" — the hero-deathrattle resurrection is unmodeled; the battlecry spends up to 20 Corpses immediately to restore the hero | the official resurrection |
| TIME_619 | Talanji of the Graves | Full draw-or-resurrect Bwonsamdi; the Boon choose-one grants Taunt / Lifesteal / Rush directly — the official "+2/+2 and deathrattle-minions cost more" riders are simplified away | the official boon riders |
| TIME_705 | Krona, Keeper of Eons | Full: the Costs of the bottom 5 deck cards are set to (1) (Set(1) cost modifiers), the top card untouched | — |
| TIME_706 | The Fins Beyond Time | "Replace your hand with your starting hand" — the engine has no opening-hand record; the fresh cards are the top of the deck (the snapshot + swap-back at end of turn are real) | an opening-hand record |
| TIME_713 | Time Adm'ral Hooktail | Full: a 0/8 Chest for the opponent; the Chest's deathrattle fills the CONTROLLER's opponent's hand with Coins (the official text — the coins land with the Hooktail player) | — |
| TIME_714 | Chrono-Lord Epoch | Full: the per-turn play snapshot destroys exactly the minions the opponent played last turn (deathrattles fire) | — |
| TIME_850 | Lo'Gosh, Blood Fighter | Full deathrattle: the hand Blood Fighter is summoned with +5/+5 and attacks a D2-random enemy; the Broll/Valeera token deathrattles are NOT implemented (the chain stops at Lo'Gosh — avoids recursion) | the official token chain |
| TIME_852 | Azure Queen Sindragosa | Full: Arcane spells cost (2) less while another Dragon is on the board (the cost pipeline) | — |
| TIME_861 | Timelooper Toki | Full: 3 random spells are tracked; playing ALL 3 adds a fresh Toki to the hand (the pool is RandomPool::Spell) | — |
| TIME_875 | Garona Halforcen | Pool-open 🔓 (registered in POOL_OPEN_CARDS — reads the opponent's hand): destroys a held King Llane and halves the enemy's max Health (rounded up; current damage preserved) | the official start-of-game hide (TIME_875t is a plain 3/3/3) |
| TIME_890 | Medivh the Hallowed | Full battlecry (Silence + destroy every other minion on BOTH boards); "Costs (0) if you control Karazhan" is real (play_cost); TIME_890t Atiesh is SKIPPED — Karazhan's "Costs (0) if you're wielding Atiesh" rider has no left side | Atiesh the Greatstaff |
| TIME_609t1 | Ranger Captain Alleria | The Discover is a D2 random spell (no options); the "repeat for each sister" rider is folded into Sylvanas' played-log | the official Discover + repeats |
| TIME_609t2 | Ranger Initiate Vereesa | "Give minions in your deck +1/+1" is unmodeled — a plain 2/4 (still counts in Sylvanas' played-log) | the official deck buff |
| TIME_619t | Bwonsamdi | The boon keywords (Taunt / Lifesteal / Rush) are granted directly; the official deathrattle "Summon a random 4-Cost minion" is unmodeled | the official deathrattle |
| TIME_042t | Infinite Banana | "Give a minion +1/+1 (this stays in your hand)" — the Infinite keyword is unmodeled; a plain 1-Cost spell (the targetable pick is a D2 random) | the Infinite keyword |
| TIME_209t | High King's Hammer | See TIME_209 — the deathrattle is a plain add-to-hand | the official shuffle + empower |
| TIME_875t | King Llane | "Start of Game: Hide from Garona in the enemy's deck. Battlecry: Draw a card. Shuffle this back" — a plain 3/3/3 that Garona's check can destroy | the official hide |
| TIME_850t / TIME_850t1 | Broll / Valeera Blood Fighter | The token deathrattles are not implemented (the chain stops at Lo'Gosh); the Taunt / Elusive keywords are real | the official token chain |
| TIME_890t2 | Karazhan the Sanctum | The activation (two random 8-Cost minions) is real; the Atiesh cost rider is skipped (TIME_890t is not implemented) | Atiesh the Greatstaff |

中文小结（同上）：M3-W2b 波（"穿越时光"子路线 W2 第二拆，25 张传说卡 +
12 张衍生物；TIME_038 克洛克先生与 TIME_063 时间领主诺兹多姆已在 W2a
落地、PR #152，不在此列）的本波核心机制均为完整原语：**打出牌日志读取**
（`Player::played_minion_ids` 全场随从打出日志 + 重放历史 `last_played`）：
游侠将军希尔瓦娜斯按已打出的姐姐（奥蕾莉亚 TIME_609t1 / 温蕾萨
TIME_609t2）数量向所有敌人重复 2 点伤害；克罗米亡语把最近 `count`（10，
重放历史上限）张不同的已打出牌各复制一张入手；**双重触发光环**
（时间领主迪奥斯——`AuraEffect::DoubleTriggers`，可沉默）：六个规则钩子
各自把预捕获效果再解析恰好一次（随从战吼、武器战吼、亡语、英雄技能、
抉择分支、回合结束触发；与"战吼两次"暗黑天赋叠加时共 3 次而非 4 次）；
**地点**（CardType::Location，Core Set W8 表示法：永恒堡垒 TIME_446、
永恒之井 TIME_211t1、辛艾萨莉 TIME_211t2、卡拉赞圣所 TIME_890t2——激活
效果放战吼槽、打出回合冷却生效；TIME_446 的 card_type|health|durability
走 expansion_differential_rebalanced，因为生成器早于地点类型）；
**每回合打出快照**（时间纪元领主 TIME_714 只摧毁对手上一回合打出的
随从——CardPlayed 钩子维护 minions_played_this_turn_ids，TurnEnded 处理
快照进 last_turn_minion_play_ids）；**INFINITY**（穆罗佐德 TIME_024——
32 位引擎没有无限值；打出效果在玩家记录上武装 murozond_infinite_pending，
己方下回合开始钩子把攻击力设为共享常量 INFINITY_ATTACK_CAP（100，
W3 复用））；**手牌换回**（鳍人 TIME_706——Player::hand_swap_snapshot）；
**尸体花费**（胡斯克 TIME_618——Player::corpses）；**托奇追踪**
（TIME_861——Player::toki_pending_spells）；**永恒堡垒恶魔减费标记**
（Player::next_demon_cost_one）。

F5 coverage: `tmw2b_sylvanas_repeats_per_alleria`,
`tmw2b_sindragosa_arcane_discount`, `tmw2b_garona_halves_enemy_health`,
`tmw2b_chronogor_highest_lowest_draws`, `tmw2b_krona_bottom_costs_one`,
`tmw2b_epoch_destroys_last_turn_minions`, `tmw2b_toki_pool_and_reget`,
`tmw2b_hooktail_chest_for_opponent`, `tmw2b_muradin_hammer`,
`tmw2b_azshara_choose_one`, `tmw2b_deios_doubling`,
`tmw2b_medivh_silence_destroy_all`, `tmw2b_murozond_infinite_attack`,
`tmw2b_eternus_control`, `tmw2b_chromie_copies_played_cards`,
`tmw2b_simplified_legendary_smoke_pins` (16 scenarios in
`tests/differential.rs` — the per-sister repeat, the Arcane discount with
and without a second Dragon, the King-Llane destroy with the halved max
Health and the preserved damage, the highest/lowest draws with the mid deck
untouched, the bottom-5 cost set with the top card kept, the per-turn
snapshot destroy with the rest of the board surviving, the three tracked
spells and the fresh Toki, the opponent-controlled Chest filling the
Hooktail player's hand, the Windfury Hammer and its deathrattle return,
both Azshara branches with the location cooldown and the Temporary fills,
the Deios doubling and its death with the aura, the both-boards silence
with no deathrattle draws, the INFINITY cap across two turn cycles, the
health-threshold steal, the played-cards copies, and the smoke pins for the
simplified legendaries — Rafaam / Broxigar / Gelbin (battlecry-free
summon) / Husk (Corpses spend) / Fins (swap-back)). Full `cargo test` fully
green (all suites, incl. every `tlc_w*_*`/`tmw1_*`/`tmw2a_*` scenario and
the 16 `tmw2b_*` — 908 passed, 1 ignored), `cargo fmt` clean,
`cargo clippy --all-targets` zero warnings.
