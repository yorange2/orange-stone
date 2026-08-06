# Fidelity Debt — Simplified Cards (F4/F5 Audit Ledger)

> **Status: 67 simplified-card markers** in `src/cards/` (audit 2026-08-06, fix pass PR #77).
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
(§10); the genuine debt is 67 cards.

---

## The ledger (grouped by missing mechanism)

### 1. Enrage — damage-conditional buff (4)

Needs a **damage-triggered permanent buff** (no damage-trigger class exists in the
engine; `spell_trigger`/`summon_trigger`/`death_trigger`/`start|end_turn_effect`
are the only trigger slots).

| ID | Card | Current | Real Hearthstone | Missing mechanism |
| --- | --- | --- | --- | --- |
| NEUTRAL_B19 | Gurubashi Berserker | vanilla 5/2/8 | Whenever this minion takes damage, gain +3 Attack | damage-trigger + permanent buff |
| NEUTRAL_C11 | Tauren Warrior | Taunt only | Taunt; **Enrage:** +3 Attack | same + conditional (damaged) |
| NEUTRAL_C15 | Spiteful Smith | vanilla 4/6 | **Enrage:** your weapon has +2 Attack | damage-trigger + weapon-aura |
| NEUTRAL_R02 | Angry Chicken | vanilla 1/1 | **Enrage:** +5 Attack | damage-trigger + permanent buff |

### 2. Tribes — Beast / Murloc / Demon (9)

Needs a **race/tribe field on `CardDef`** (none exists; `RandomPool::Beast/Demon`
are hardcoded ID lists) plus tribe-conditioned targets and auras.

| ID | Card | Current | Real Hearthstone | Missing mechanism |
| --- | --- | --- | --- | --- |
| HUNTER_015 | Houndmaster | Battlecry: +2/+2 to a friendly minion | Battlecry: give a friendly **Beast** +2/+2 and Taunt | tribe predicate + two-effect battlecry |
| HUNTER_016 | Tundra Rhino | Charge aura (all friendly) | Your **Beasts** have Charge | tribe-conditional aura |
| NEUTRAL_R01 | Coldlight Seer | Battlecry: all friendly minions +2 Health | Battlecry: give all other **Murlocs** +2 Health | tribe predicate |
| NEUTRAL_E02 | Murloc Warleader | aura: other friendly minions +2/+1 | Your other **Murlocs** have +2/+1 | tribe-conditional aura |
| NEUTRAL_R05 | Murloc Tidecaller | vanilla | Whenever you summon a **Murloc**, gain +1 Attack | tribe predicate + summon-trigger |
| NEUTRAL_E03 | Hungry Crab | vanilla | Battlecry: destroy a **Murloc** and gain +2/+2 | tribe predicate + destroy-buff combo |
| WARLOCK_020 | Sense Demons | draw 1 card | Draw two **Demons** from your deck | tribe predicate + deck-filtered draw |
| WARLOCK_021 | Demonfire | deal 2 damage | Deal 2 damage to a minion; if it's a friendly **Demon**, +2/+2 instead | tribe predicate + conditional branch |
| WARLOCK_T01 | Siegebreaker | Taunt only | Taunt; your other **Demons** have +1 Attack | tribe-conditional aura |

### 3. Event triggers — summon / heal / death / secret / attack / play (16)

`summon_trigger`, `spell_trigger`, `death_trigger` exist and are used by other
cards; **missing classes: heal-trigger, attack-trigger, card-played-trigger,
secret-played-trigger**, and a "destroy secret" effect (for the secret cards).

| ID | Card | Current | Real Hearthstone | Missing mechanism |
| --- | --- | --- | --- | --- |
| NEUTRAL_R09 | Knife Juggler | vanilla | After you summon a minion, deal 1 damage to a random enemy | summon-trigger + random-target damage (machinery exists; card not wired) |
| PALADIN_017 | Sword of Justice | summon_trigger buffs Self_ (target semantics unverified) | Whenever you summon a minion, give it +1/+1 | verify trigger target binding; wrong ID too (see Findings) |
| NEUTRAL_C12 | Flesheathing Ghoul | vanilla | Whenever a minion dies, gain +1 Attack | death-trigger + self-buff (machinery exists) |
| NEUTRAL_R04 | Lightwarden | vanilla | Whenever a character is healed, gain +2 Attack | heal-trigger (missing) |
| NEUTRAL_R06 | Secretkeeper | vanilla | Whenever a **Secret** is played, gain +1/+1 | secret-played-trigger (missing) |
| NEUTRAL_R25 | SI:7 Infiltrator | vanilla | Battlecry: destroy a random enemy Secret | destroy-secret effect (missing) |
| NEUTRAL_R26 | Eater of Secrets | vanilla | Battlecry: destroy all enemy Secrets and gain +1/+1 | destroy-secret effect + buff |
| PALADIN_019 | Blessing of Wisdom | no effect | Whenever the target minion attacks, draw a card | attack-trigger on a target (missing) + aura-on-entity |
| NEUTRAL_R15 | Demolisher | vanilla | At the start of your turn, deal 2 damage to a random enemy | start-turn trigger exists; random-target damage |
| NEUTRAL_E04 | Doomsayer | vanilla | At the start of your turn, destroy all minions | start-turn trigger + DestroyMinion(AllMinions) |
| NEUTRAL_R17 | Questing Adventurer | vanilla | Whenever you play a card, gain +1/+1 | card-played-trigger (missing) |
| NEUTRAL_R13 | Alarm-o-Bot | vanilla | At the start of your turn, swap this with a random hand minion | hand-zone swap effect (missing) |
| NEUTRAL_R12 | Wild Pyromancer | vanilla | After you cast a spell, deal 1 damage to ALL minions | spell-trigger + AOE (machinery exists; card not wired) |
| NEUTRAL_R21 | Young Priestess | vanilla | At the end of your turn, give another random friendly minion +1 Health | end-turn trigger exists; random-friendly target |
| NEUTRAL_R23 | Master Swordsmith | vanilla | At the end of your turn, give another random friendly minion +1 Attack | same |
| MAGE_017 | Ethereal Arcanist | +2/+2 at end of turn, unconditional | At the end of your turn, if you control a Secret, gain +2/+2 | conditional end-turn (owns-secret predicate) |

### 4. Conditional targets & states (9)

Needs **condition predicates on targets / owners** (some exist: `DamagedEnemyMinion`,
`TauntEnemyMinion`; missing: attack-range, hand-size, hero-health, divine-shield
count, "set health to 1" effect).

| ID | Card | Current | Real Hearthstone | Missing mechanism |
| --- | --- | --- | --- | --- |
| NEUTRAL_R20 | Stampeding Kodo | vanilla | Battlecry: destroy a random enemy minion with 2 or less Attack | attack ≤ N predicate + random pick |
| NEUTRAL_E06 | Big Game Hunter | vanilla | Battlecry: destroy a minion with 7 or more Attack | attack ≥ N predicate |
| NEUTRAL_R19 | Twilight Drake | vanilla | Battlecry: +1 Health for each card in your hand | hand-size predicate |
| NEUTRAL_E05 | Blood Knight | vanilla | Battlecry: absorb all Divine Shields, gain +3/+3 | divine-shield count + mass absorb |
| WARRIOR_021 | Mortal Strike | deal 4 | Deal 4 damage; if you have 12 or less Health, deal 6 instead | owner-health predicate + conditional branch |
| WARRIOR_023 | Rampage | +3/+3 to any friendly minion | Give a **damaged** minion +3/+3 | damaged predicate (exists for enemies only: `DamagedEnemyMinion`) |
| WARRIOR_022 | Battle Rage | draw 2 | Draw a card for each damaged friendly character | damaged-counting (both sides) |
| PRIEST_018 | Mass Dispel | Silence one random enemy minion | Silence ALL enemy minions, draw a card | `SilenceMinion`+`AllEnemyMinions` exists — mostly card-wiring; unreachable by ID (see Findings) |
| PALADIN_018 | Repentance | Secret: DealDamage | Secret: when your opponent plays a minion, set its Health to 1 | "set health to 1" effect (missing; `MinHealthUntilEndOfTurn` is temporary only); wrong ID too |

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

### 6. Cost & weapon-condition auras (8)

The cost-modifier stack (G5) exists; **missing: hand-zone cost auras on all
minions / "first minion this turn" state / conditional (while weapon equipped)
Charge / weapon-attack cost reduction / weapon-durability damage**.

| ID | Card | Current | Real Hearthstone | Missing mechanism |
| --- | --- | --- | --- | --- |
| NEUTRAL_R22 | Mana Wraith | vanilla | ALL minions cost (1) more | hand-cost aura (all minions) |
| NEUTRAL_C14 | Venture Co. Mercenary | vanilla | Your minions cost (3) more | hand-cost aura (own minions) |
| NEUTRAL_R24 | Pint-Sized Summoner | vanilla | The first minion you play each turn costs (1) less | "first minion this turn" state |
| NEUTRAL_C07 | Southsea Deckhand | always has Charge | Has Charge while you have a weapon equipped | weapon-equipped predicate on Charge |
| NEUTRAL_C13 | Dread Corsair | Taunt only | Taunt; costs (1) less per Attack of your weapon | weapon-attack cost modifier |
| NEUTRAL_C09 | Bloodsail Raider | vanilla | Battlecry: gain Attack equal to your weapon's Attack | weapon-attack predicate |
| NEUTRAL_R03 | Bloodsail Corsair | vanilla | Battlecry: remove 1 Durability from your opponent's weapon | weapon-durability damage (only DestroyWeapon exists) |
| LEGENDARY_021 | Millhouse Manastorm | no drawback | Battlecry: enemy spells cost 0 next turn | enemy-spells-cost-0 effect (missing) |

### 7. This-turn temporary buff (1)

| ID | Card | Current | Real Hearthstone | Missing mechanism |
| --- | --- | --- | --- | --- |
| NEUTRAL_R10 | Mana Addict | vanilla | Whenever you cast a spell, gain +2 Attack **this turn** | this-turn buff with end-of-turn expiry (engine has `TempDebuff`; needs temp-buff) |

### 8. Probability (1)

| ID | Card | Current | Real Hearthstone | Missing mechanism |
| --- | --- | --- | --- | --- |
| LEGENDARY_022 | Nat Pagle | draw at end of turn (always) | 50% chance to draw a card at the end of your turn | probabilistic effect |

### 9. Composite & miscellaneous (11)

| ID | Card | Current | Real Hearthstone | Missing mechanism |
| --- | --- | --- | --- | --- |
| DRUID_016 | Gift of the Wild | +2/+2 only | Give your minions +2/+2 and Taunt | two-effect mass battlecry (buff + taunt) |
| PALADIN_018 | Righteousness | not implementable per comment | Give your minions Divine Shield | mass Divine Shield (no mass-grant; wrong ID of its own, see Findings) |
| PALADIN_017 | Holy Wrath | draw only | Draw a card, deal damage equal to its mana cost | draw-then-damage chained effect |
| SHAMAN_018 | Ancestral Healing | heal + Taunt (partial) | Restore a minion to full Health and give it Taunt | verify `FullHeal` + taunt wiring |
| PRIEST_017 | Kul Tiran Chaplain | buff self | Battlecry: give a friendly minion +2 Health | two-effect battlecry (exists elsewhere — wiring) |
| PRIEST_018 | Lightwell | restore 3 at end of turn | At the start of your turn, restore 3 Health to a damaged friendly character | damaged-friendly predicate + start-turn (see also ID collision) |
| ROGUE_025 | Pilfer | random non-Rogue card | Add a random card from another class to your hand | class filter (no class model) |
| NEUTRAL_T21e | Ysera Awakens | damage includes Ysera | Deal 5 damage to all **other** characters (Dream token) | self-exclusion on AllCharacters |
| HUNTER_017 | Flare | draw only | Destroy all enemy secrets, draw a card | destroy-secret effect (same as SI:7) |
| NEUTRAL_R14 | Arcane Golem | Charge only | Charge; Battlecry: give your opponent a Mana Crystal | give-opponent-mana effect |
| NEUTRAL_R16 | Emperor Cobra | vanilla | **Poison** (destroy any minion damaged by it) | poison mechanic (missing entirely) |

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
`GrantCharge` / Stealth / Elusive / Secrets.

**Missing** (primitives first, per the Review-II "do G before F4/F5" discipline):
1. race/tribe field on `CardDef` — unblocks §2 entirely
2. damage-triggered effects (Enrage) — §1
3. trigger classes: heal, attack, card-played, secret-played — §3
4. target predicates: attack-range, hand-size, hero-health, damaged-friendly,
   owns-secret, weapon-equipped; "first minion this turn" state — §4, §6
5. effects: set-health-to-1, swap attack/health, poison, mass Divine Shield,
   enemy-spells-cost-0, destroy-secret, weapon-durability damage,
   adjacent-target buff/freeze, two-random damage, this-turn temp buff,
   probabilistic effects — §5, §6, §7, §8, §9

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
