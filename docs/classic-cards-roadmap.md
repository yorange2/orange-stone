# Classic Set Implementation Roadmap

> Last updated: 2026-08-05 (all cards implemented)
> Covers the missing cards from the Classic/Basic (怀旧) set.
> Companion documents: [classic-cards.md](classic-cards.md) (full card list) · [classic-cards-zh.md](classic-cards-zh.md)

## TL;DR

**✅ Done (2026-08-05): all 62 cards are implemented and merged to `main`.**

| Tier | Count | What it is | Status |
|------|-------|-----------|--------|
| 1 | 20 | Docs mark ✅ but missing from code — mostly vanilla/basic effects | ✅ implemented |
| 2 | 29 | Needed new engine mechanics | ✅ implemented |
| 3 | 13 | Random card generation (subject to [pool closure](#pool-closure)) | ✅ implemented |

> Note: Tinkmaster Overspark appears as both 🔧 and ⏸️ in `classic-cards.md`; it is counted once here (Tier 3).
> Note: the numbers in this roadmap are verified against `src/cards/` as of 2026-08-05. The statuses in `classic-cards.md` are known to be stale in both directions (see [Milestone 0](#milestone-0--doc-reconciliation)).
> Note: Combo and Choose One already existed in the engine before this roadmap began (`rules.rs` handles `combo_effect`/`choose_one_effect`); the two corresponding checklist items required no new work.

---

## Pool Closure

The Classic pool must be **closed**: no card implemented under this roadmap may generate or introduce cards from outside the Classic set.

- **Allowed:** random sampling from `ALL_CARDS` (entirely Classic) with class/tribe/rarity filters — e.g. "a random Beast" samples only from Classic Beasts.
- **Allowed:** fixed token pools defined inside the Classic implementation (Dream Cards, Animal Companion's companions, Bananas, Devilsaur/Squirrel, Fireball, Treants, Snakes, …) — they are part of the pool itself.
- **Not allowed:** any mechanism that could reach outside the pool. For this reason, the following opponent-copy cards are **removed from this roadmap**:
  - Mind Vision, Thoughtsteal, Mindgames (copy from the opponent's hand/deck — they would leak outside the pool if other sets are ever supported)
  - Lorewalker Cho (copies a spell the opponent cast)
- The [random card pool framework](#mechanic-checklist) must enforce this constraint: every sampling pool is a filtered subset of the Classic pool.

---

## Tier 1 — Missing implementations, no new mechanics (20)

Every effect these cards need (Taunt, Stealth, Battlecry, Deathrattle, auras, weapons, secrets, card draw) is already supported by the engine. The docs already list them as ✅, so implementing them also fixes doc drift.

> Note: two cards needed tiny effect additions despite the "no engine changes" claim (added 2026-08-05): `EffectTarget::AllEnemies` (all enemy characters — Explosive Trap) and `CardEffect::ReturnToHandAndIncreaseCost` (Freezing Trap). Both are additive variants in `src/core/effect.rs` with resolution in `src/engine/trigger.rs`.

### `src/cards/classic_neutral.rs` (15)

- [x] **Wisp** — 0 cost 1/1, vanilla
- [x] **Elven Archer** — **Battlecry:** Deal 1 damage.
- [x] **Goldshire Footman** — **Taunt**
- [x] **Novice Engineer** — **Battlecry:** Draw a card.
- [x] **River Crocolisk** — 2/3 Beast, vanilla
- [x] **Raid Leader** — aura: your other minions have +1 Attack
- [x] **Shattered Sun Cleric** — **Battlecry:** Give a friendly minion +1/+1.
- [x] **Chillwind Yeti** — 4/5, vanilla
- [x] **Boulderfist Ogre** — 6/7, vanilla
- [x] **Stormwind Champion** — aura: your other minions have +1/+1
- [x] **War Golem** — 7/7, vanilla
- [x] **Dire Wolf Alpha** — aura: adjacent minions have +1 Attack
- [x] **Loot Hoarder** — **Deathrattle:** Draw a card.
- [x] **Stranglethorn Tiger** — **Stealth**
- [x] **Ravenholdt Assassin** — **Stealth**

### `src/cards/classic_warrior.rs` (2)

- [x] **Fiery War Axe** — weapon 3/2
- [x] **Arcanite Reaper** — weapon 5/2

### `src/cards/classic_hunter.rs` (2)

- [x] **Explosive Trap** — **Secret:** When a minion attacks your hero, deal 2 damage to all enemies.
- [x] **Freezing Trap** — **Secret:** When an enemy minion attacks, return it to its owner's hand and it costs (2) more.

### `src/cards/classic_warlock.rs` (1)

- [x] **Siegebreaker** — **Taunt**

---

## Tier 2 — Need new engine mechanics (29)

Organized by class (= by implementation file). Each entry notes the mechanic(s) it requires — build shared mechanics first (see [Mechanic checklist](#mechanic-checklist)).

### `src/cards/classic_neutral.rs` (1)

- [x] **King Mukla** — **Battlecry:** Give your opponent 2 Bananas. *(card generation into opponent's hand; Bananas is an in-pool Classic token — closure-compliant)*

### `src/cards/classic_druid.rs` (3)

- [x] **Cenarius** — **Choose One:** +2/+2 to your minions, or summon two 2/2 Treants. *(Choose One + summon tokens)*
- [x] **Keeper of the Grove** — **Choose One:** Deal 2 damage, or **Silence** a minion. *(Choose One)*
- [x] **Soul of the Forest** — Give your minions "**Deathrattle:** Summon a 2/2 Treant." *(grant deathrattle to a group)*

### `src/cards/classic_hunter.rs` (5)

- [x] **Bestial Wrath** — Give a Beast +2 Attack and **Immune** this turn. *(Immune keyword)*
- [x] **Misdirection** — **Secret:** When an enemy attacks your hero, redirect it to another random character. *(secret + attack redirection)*
- [x] **Snake Trap** — **Secret:** When one of your minions takes damage, summon three 1/1 Snakes. *(secret + damage trigger + summon)*
- [x] **Snipe** — **Secret:** When your opponent plays a minion, deal 4 damage to it. *(secret + "minion played" trigger)*
- [x] **Gladiator's Longbow** — weapon 5/2; your hero is **Immune** while attacking. *(Immune while attacking)*

### `src/cards/classic_mage.rs` (4)

- [x] **Icicle** — **Freeze** a minion; if it is already frozen, deal 2 damage instead of 1. *(conditional freeze — Freeze itself already exists)*
- [x] **Sorcerer's Apprentice** — Your spells cost (1) less. *(cost-reduction aura)*
- [x] **Kirin Tor Mage** — **Battlecry:** Your next Secret costs (0). *(one-shot cost effect)*
- [x] **Spellbender** — **Secret:** When an enemy casts a spell on a minion, summon a 1/3 as the new target. *(secret + spell-target redirection)*

### `src/cards/classic_paladin.rs` (1)

- [x] **Noble Sacrifice** — **Secret:** When an enemy attacks, summon a 2/1 Defender as the new target. *(secret + attack redirection)*

### `src/cards/classic_priest.rs` (2)

- [x] **Shadow Madness** — Take control of an enemy minion with 3 or less Attack until end of turn. *(temporary mind control)*
- [x] **Natalie Seline** — **Battlecry:** Destroy a minion and gain its Health. *(destroy + stat gain)*

### `src/cards/classic_rogue.rs` (8)

- [x] **Shadowstep** — Return a friendly minion to your hand. It costs (2) less. *(bounce + cost reduction — bounce itself exists)*
- [x] **Betrayal** — Deal damage equal to the target's Attack to adjacent minions. *(adjacency damage)*
- [x] **Blade Flurry** — Destroy your weapon and deal its Attack damage to all enemies. *(weapon destruction + AoE)*
- [x] **Patient Assassin** — **Stealth**, **Poison** *(Poison keyword)*
- [x] **Headcrack** — Deal 2 damage to the enemy hero. **Combo:** Return this to your hand. *(Combo keyword)*
- [x] **Perdition's Blade** — weapon 2/2; **Battlecry:** Deal 1 damage. *(weapon with Battlecry)*
- [x] **Master of Disguise** — **Battlecry:** Give a friendly minion **Stealth**. *(grant Stealth)*
- [x] **Kidnapper** — **Combo:** Return an enemy minion to your hand. *(Combo + bounce enemy)*

### `src/cards/classic_shaman.rs` (2)

- [x] **Far Sight** — Draw a card. It costs (3) less. *(draw with cost reduction)*
- [x] **Unbound Elemental** — Whenever you play a card with **Overload**, gain +1/+1. *(Overload trigger)*

### `src/cards/classic_warlock.rs` (2)

- [x] **Corruption** — Choose an enemy minion. At the start of your turn, destroy it. *(delayed start-of-turn destroy)*
- [x] **Summoning Portal** — Your minions cost (2) less, but not less than (1). *(cost-reduction aura with floor)*

### `src/cards/classic_warrior.rs` (1)

- [x] **Commanding Shout** — Your minions can't go below 1 Health this turn. *(minimum-health effect)*

---

## Tier 3 — Deferred: random card generation (13)

These need a random card pool or card-generation framework, subject to [pool closure](#pool-closure): every sampling pool is a filtered subset of `ALL_CARDS` (the runtime registry in `src/cards/sets.rs`), and tokens (Bananas, Dream Cards, Devilsaur/Squirrel, companions, …) are defined as part of the Classic pool.

### `src/cards/classic_neutral.rs` (6)

- [x] **Brightwing** — **Battlecry:** Add a random Legendary minion to your hand. *(sample from the Classic pool with a legendary-rarity filter)*
- [x] **Nozdormu** — 15-second turn timer. *(suggest: implement as a plain 8/8 with a comment — timer is meaningless in sim)*
- [x] **Xavius** — At the end of your turn, add a random Shadow spell to your hand. *(sample from the Classic pool with a shadow filter)*
- [x] **Ysera** — At the end of your turn, draw a Dream Card. *(Dream Cards are an in-pool token set — closure-compliant)*
- [x] **Barrens Stablehand** — **Battlecry:** Summon a random Beast. *(sample from the Classic pool with a beast-tribe filter)*
- [x] **Tinkmaster Overspark** — **Battlecry:** Transform a minion into a 5/5 Devilsaur or a 1/1 Squirrel at random. *(transform + random choice; Devilsaur/Squirrel are fixed in-pool tokens; needs Transform mechanic first — from Tier 2)*

### `src/cards/classic_hunter.rs` (1)

- [x] **Animal Companion** — Summon a random 4/4 Carcass, 4/2 Leokk, or 2/4 Misha. *(the three companions are fixed in-pool tokens — closure-compliant; easy once random summon exists)*

### `src/cards/classic_mage.rs` (2)

- [x] **Tome of Intellect** — Add a random Mage spell to your hand. *(sample from the Classic pool with a mage-class filter)*
- [x] **Archmage Antonidas** — Whenever you cast a spell, add a Fireball to your hand. *(Fireball is a Classic card — fixed generation, closure-compliant)*

### `src/cards/classic_priest.rs` (1)

- [x] **Mind Control** — Take control of an enemy minion. *(not actually random — it is in this tier because permanent mind control needs the mechanic from Shadow Madness; implement right after it)*

### `src/cards/classic_rogue.rs` (1)

- [x] **Pilfer** — Add a random card from another class to your hand. *(sample from the Classic pool with a class filter)*

### `src/cards/classic_warlock.rs` (2)

- [x] **Call of the Void** — Add a random Demon to your hand. *(sample from the Classic pool with a demon-tribe filter)*
- [x] **Bane of Doom** — Deal 2 damage to a character. If it dies, summon a random Demon. *(damage + conditional random summon; Demon pool as above)*

---

## Mechanic checklist

Engine features to build before/while implementing Tier 2 & 3. Roughly ordered by number of cards they unblock:

- [x] **Combo keyword** — unlocks: Headcrack, Kidnapper
- [x] **Choose One** — unlocks: Cenarius, Keeper of the Grove
- [x] **Attack/spell-target redirection** — unlocks: Misdirection, Noble Sacrifice, Spellbender
- [x] **Immune** — unlocks: Bestial Wrath, Gladiator's Longbow
- [x] **Temporary mind control (until end of turn)** — unlocks: Shadow Madness; then permanent control → Mind Control
- [x] **Transform** — unlocks: Tinkmaster Overspark
- [x] **Cost-reduction auras & one-shot cost effects** — unlocks: Sorcerer's Apprentice, Summoning Portal, Kirin Tor Mage, Shadowstep, Far Sight
- [x] **Poison** — unlocks: Patient Assassin
- [x] **Weapon battlecry** — unlocks: Perdition's Blade
- [x] **Weapon destruction + AoE** — unlocks: Blade Flurry
- [x] **Grant deathrattle / grant Stealth** — unlocks: Soul of the Forest, Master of Disguise
- [x] **Adjacency damage** — unlocks: Betrayal
- [x] **Delayed start-of-turn destroy** — unlocks: Corruption
- [x] **Minimum-health effect** — unlocks: Commanding Shout
- [x] **Overload trigger** — unlocks: Unbound Elemental
- [x] **Spell-cast → card-to-hand** — unlocks: Archmage Antonidas, Tome of Intellect
- [x] **Random card pool framework** (class/rarity/type filtering + token pools; **must satisfy [pool closure](#pool-closure)** — sampling pools are Classic subsets, tokens defined in-pool) — unlocks: all of Tier 3

---

## Milestones

### Milestone 0 — Doc reconciliation

`classic-cards.md` / `classic-cards-zh.md` statuses are stale in both directions. Before relying on them:

- [ ] Fix 20 cards marked ✅ that don't exist in code (→ Tier 1 work)
- [ ] Add 11 implemented cards that are absent from the docs: Illidan Stormrage, Ice Block, Azure Drake, Acolyte of Pain, Spellbreaker, Auchenai Soulpriest, Doomguard, Holy Fire, Eater of Secrets, Molten Giant, Mountain Giant
- [ ] Re-mark cards that are actually implemented but flagged 🔧 (e.g. The Black Knight, Captain Greenskin, Harrison Jones, Vaporize, Faceless Manipulator, Vanish, Flare, Houndmaster, Nourish, Starfall, Gadgetzan Auctioneer, and others — verify each against code)
- [ ] Remove duplicate entries in `ALL_CARDS` (src/cards/sets.rs): VAPORIZE, FACELESS_MANIPULATOR, ANCESTRAL_SPIRIT, THE_BLACK_KNIGHT, MILLHOUSE_MANASTORM, NAT_PAGLE, HEROIC_STRIKE
- [x] Update the stats header line in both docs

### Milestone 1 — Tier 1: 20 data-only cards

No engine changes. Pure `CardDef` additions, one commit per class file.

### Milestone 2 — Tier 2: 29 mechanic cards

Build the [mechanic checklist](#mechanic-checklist) items in order, then implement the cards each one unlocks. Group commits by mechanic, not by class, where practical.

### Milestone 3 — Tier 3: 13 random/generation cards

Needs the random card pool framework, subject to [pool closure](#pool-closure). Nozdormu can be done any time as a plain 8/8.

---

## Definition of Done

For every card in this roadmap:

- [x] `CardDef` added to the correct `src/cards/classic_*.rs` file, following existing ordering
- [x] Registered in `ALL_CARDS` in `src/cards/sets.rs`
- [x] Unit test covering the card's effect; `cargo test` passes
- [x] `cargo fmt` and `cargo clippy` clean
- [x] Status flipped to ✅ in both `docs/classic-cards.md` and `docs/classic-cards-zh.md`
- [x] Checkbox ticked in this roadmap
