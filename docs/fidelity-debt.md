# Fidelity Debt — Simplified Cards (F4/F5 Audit Ledger)

> **Status: 18 simplified-card markers** in `src/cards/` (audit 2026-08-06, fix pass PR #77;
> W0 wiring pass PR #79 cleared 13; W1 race pass PR #80 cleared 11; W2 triggers PR #81 cleared 8;
> W3 predicates PR #82 cleared 9; W4 cost/weapon PR #83 cleared 8).
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
> **Execution plan**: [docs/fidelity-debt-roadmap.md](fidelity-debt-roadmap.md)
> (zh: `fidelity-debt-roadmap-zh.md`) — 8 dependency-ordered waves (W0 wiring …
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
(§10); the genuine debt is 67 cards — **W0 cleared 13, W1 cleared 11,
W2 cleared 8, W3 cleared 9 and W4 cleared 8, leaving 18**.

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

### 3. Event triggers — summon / heal / death / secret / attack / play (2)

Trigger classes are complete (W0 wiring + W2 trigger pass): heal, attack (on a
target), card-played, secret-played and any-minion-died all landed; so did the
destroy-secret effects (one random / all + composition). The two remaining
cards need non-trigger mechanisms. (W2 cleared: Flesheathing Ghoul, Lightwarden,
Secretkeeper, SI:7 Infiltrator, Eater of Secrets, Blessing of Wisdom,
Questing Adventurer; Flare is in §9.)

| ID | Card | Current | Real Hearthstone | Missing mechanism |
| --- | --- | --- | --- | --- |
| NEUTRAL_R13 | Alarm-o-Bot | vanilla | At the start of your turn, swap this with a random hand minion | hand-zone swap effect (missing) |
| MAGE_017 | Ethereal Arcanist | +2/+2 at end of turn, unconditional | At the end of your turn, if you control a Secret, gain +2/+2 | conditional end-turn (owns-secret predicate) |

### 4. Conditional targets & states (9) ✅ resolved (W3, PR #82)

All landed: attack-range (≤2 / ≥7), hand-size, hero-health threshold,
damaged-friendly/any targets, damaged-counting, owns-secret, the "first
minion this turn" state, and divine-shield absorb (`w3_*` scenarios).

### 5. Adjacent / multi-target (4)

`DestroyAdjacent` exists; **missing: adjacent-target buffs, adjacent Freeze,
swap-attack-health** (two-random-target damage exists — `DealDamageToTwo`, already
used by Cleave; Multi-Shot is faithful but its comment is stale, see §10).

| ID | Card | Current | Real Hearthstone | Missing mechanism |
| --- | --- | --- | --- | --- |
| MAGE_016 | Cone of Cold | Freeze one random enemy minion | Freeze a minion and its neighbors | adjacent-target Freeze |
| NEUTRAL_R11 | Sunfury Protector | vanilla | Battlecry: give adjacent minions Taunt | adjacent-target buff |
| NEUTRAL_R18 | Ancient Mage | vanilla | Battlecry: give adjacent minions Spell Damage +1 | adjacent-target spell-damage buff |
| NEUTRAL_R08 | Crazed Alchemist | vanilla | Battlecry: swap a minion's Attack and Health | swap effect (missing; only Set/Double variants) |

### 6. Cost & weapon-condition auras (8) ✅ resolved (W4, PR #83)

All landed: hand-zone cost auras (global / own), weapon-attack cost reduction,
weapon-durability damage, weapon-equipped predicate (conditional Charge),
enemy-spells-cost-0, give-opponent-mana (`w4_*` scenarios).

### 7. This-turn temporary buff (1)

| ID | Card | Current | Real Hearthstone | Missing mechanism |
| --- | --- | --- | --- | --- |
| NEUTRAL_R10 | Mana Addict | vanilla | Whenever you cast a spell, gain +2 Attack **this turn** | this-turn buff with end-of-turn expiry (engine has `TempDebuff`; needs temp-buff) |

### 8. Probability (1)

| ID | Card | Current | Real Hearthstone | Missing mechanism |
| --- | --- | --- | --- | --- |
| LEGENDARY_022 | Nat Pagle | draw at end of turn (always) | 50% chance to draw a card at the end of your turn | probabilistic effect |

### 9. Composite & miscellaneous (8)

(W0 cleared: Kul Tiran Chaplain, Emperor Cobra; W2 cleared: Flare — destroy all
enemy Secrets and draw.)

| ID | Card | Current | Real Hearthstone | Missing mechanism |
| --- | --- | --- | --- | --- |
| DRUID_016 | Gift of the Wild | +2/+2 only | Give your minions +2/+2 and Taunt | two-effect mass battlecry (buff + taunt) |
| PALADIN_018 | Righteousness | not implementable per comment | Give your minions Divine Shield | mass Divine Shield (no mass-grant; wrong ID of its own, see Findings) |
| PALADIN_017 | Holy Wrath | draw only | Draw a card, deal damage equal to its mana cost | draw-then-damage chained effect |
| SHAMAN_018 | Ancestral Healing | heal + Taunt (partial) | Restore a minion to full Health and give it Taunt | verify `FullHeal` + taunt wiring |
| PRIEST_018 | Lightwell | restore 3 at end of turn | At the start of your turn, restore 3 Health to a damaged friendly character | damaged-friendly predicate + start-turn (see also ID collision) |
| ROGUE_025 | Pilfer | random non-Rogue card | Add a random card from another class to your hand | class filter (no class model) |
| NEUTRAL_T21e | Ysera Awakens | damage includes Ysera | Deal 5 damage to all **other** characters (Dream token) | self-exclusion on AllCharacters |
| NEUTRAL_R14 | Arcane Golem | Charge only | Charge; Battlecry: give your opponent a Mana Crystal | give-opponent-mana effect |

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
conditional Charge + enemy-spells-cost-0 + give-opponent-mana (W4).

**Missing** (primitives first, per the Review-II "do G before F4/F5" discipline):
1. effects: set-health-to-1, swap attack/health, mass Divine Shield,
   adjacent-target buff/freeze, two-random damage, this-turn temp buff,
   probabilistic effects — §5, §7, §8, §9

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
