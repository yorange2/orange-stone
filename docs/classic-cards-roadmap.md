# Classic Set Implementation Roadmap

> Last updated: 2026-08-05
> Covers the missing cards from the Classic/Basic (怀旧) set.
> Companion documents: [classic-cards.md](classic-cards.md) (full card list) · [classic-cards-zh.md](classic-cards-zh.md)

## TL;DR

**66 unique cards** from the Classic/Basic set are still unimplemented, in three tiers:

| Tier | Count | What it is | New engine mechanics needed |
|------|-------|-----------|-----------------------------|
| 1 | 20 | Docs mark ✅ but missing from code — mostly vanilla/basic effects | None |
| 2 | 30 | Genuinely not implemented, need new engine mechanics | Yes |
| 3 | 16 | Deferred: random/discover/opponent interactions | Random card pool / generation framework |

> Note: Tinkmaster Overspark appears as both 🔧 and ⏸️ in `classic-cards.md`; it is counted once here (Tier 3).
> Note: the numbers in this roadmap are verified against `src/cards/` as of 2026-08-05. The statuses in `classic-cards.md` are known to be stale in both directions (see [Milestone 0](#milestone-0--doc-reconciliation)).

---

## Tier 1 — Missing implementations, no new mechanics (20)

Every effect these cards need (Taunt, Stealth, Battlecry, Deathrattle, auras, weapons, secrets, card draw) is already supported by the engine. The docs already list them as ✅, so implementing them also fixes doc drift.

### `src/cards/classic_neutral.rs` (15)

- [ ] **Wisp** — 0 cost 1/1, vanilla
- [ ] **Elven Archer** — **Battlecry:** Deal 1 damage.
- [ ] **Goldshire Footman** — **Taunt**
- [ ] **Novice Engineer** — **Battlecry:** Draw a card.
- [ ] **River Crocolisk** — 2/3 Beast, vanilla
- [ ] **Raid Leader** — aura: your other minions have +1 Attack
- [ ] **Shattered Sun Cleric** — **Battlecry:** Give a friendly minion +1/+1.
- [ ] **Chillwind Yeti** — 4/5, vanilla
- [ ] **Boulderfist Ogre** — 6/7, vanilla
- [ ] **Stormwind Champion** — aura: your other minions have +1/+1
- [ ] **War Golem** — 7/7, vanilla
- [ ] **Dire Wolf Alpha** — aura: adjacent minions have +1 Attack
- [ ] **Loot Hoarder** — **Deathrattle:** Draw a card.
- [ ] **Stranglethorn Tiger** — **Stealth**
- [ ] **Ravenholdt Assassin** — **Stealth**

### `src/cards/classic_warrior.rs` (2)

- [ ] **Fiery War Axe** — weapon 3/2
- [ ] **Arcanite Reaper** — weapon 5/2

### `src/cards/classic_hunter.rs` (2)

- [ ] **Explosive Trap** — **Secret:** When a minion attacks your hero, deal 2 damage to all enemies.
- [ ] **Freezing Trap** — **Secret:** When an enemy minion attacks, return it to its owner's hand and it costs (2) more.

### `src/cards/classic_warlock.rs` (1)

- [ ] **Siegebreaker** — **Taunt**

---

## Tier 2 — Need new engine mechanics (30)

Organized by class (= by implementation file). Each entry notes the mechanic(s) it requires — build shared mechanics first (see [Mechanic checklist](#mechanic-checklist)).

### `src/cards/classic_neutral.rs` (2)

- [ ] **Lorewalker Cho** — When a player casts a spell, put a copy into the other player's hand. *(spell-cast event → copy card to hand)*
- [ ] **King Mukla** — **Battlecry:** Give your opponent 2 Bananas. *(card generation into opponent's hand; needs Bananas token)*

### `src/cards/classic_druid.rs` (3)

- [ ] **Cenarius** — **Choose One:** +2/+2 to your minions, or summon two 2/2 Treants. *(Choose One + summon tokens)*
- [ ] **Keeper of the Grove** — **Choose One:** Deal 2 damage, or **Silence** a minion. *(Choose One)*
- [ ] **Soul of the Forest** — Give your minions "**Deathrattle:** Summon a 2/2 Treant." *(grant deathrattle to a group)*

### `src/cards/classic_hunter.rs` (5)

- [ ] **Bestial Wrath** — Give a Beast +2 Attack and **Immune** this turn. *(Immune keyword)*
- [ ] **Misdirection** — **Secret:** When an enemy attacks your hero, redirect it to another random character. *(secret + attack redirection)*
- [ ] **Snake Trap** — **Secret:** When one of your minions takes damage, summon three 1/1 Snakes. *(secret + damage trigger + summon)*
- [ ] **Snipe** — **Secret:** When your opponent plays a minion, deal 4 damage to it. *(secret + "minion played" trigger)*
- [ ] **Gladiator's Longbow** — weapon 5/2; your hero is **Immune** while attacking. *(Immune while attacking)*

### `src/cards/classic_mage.rs` (4)

- [ ] **Icicle** — **Freeze** a minion; if it is already frozen, deal 2 damage instead of 1. *(conditional freeze — Freeze itself already exists)*
- [ ] **Sorcerer's Apprentice** — Your spells cost (1) less. *(cost-reduction aura)*
- [ ] **Kirin Tor Mage** — **Battlecry:** Your next Secret costs (0). *(one-shot cost effect)*
- [ ] **Spellbender** — **Secret:** When an enemy casts a spell on a minion, summon a 1/3 as the new target. *(secret + spell-target redirection)*

### `src/cards/classic_paladin.rs` (1)

- [ ] **Noble Sacrifice** — **Secret:** When an enemy attacks, summon a 2/1 Defender as the new target. *(secret + attack redirection)*

### `src/cards/classic_priest.rs` (2)

- [ ] **Shadow Madness** — Take control of an enemy minion with 3 or less Attack until end of turn. *(temporary mind control)*
- [ ] **Natalie Seline** — **Battlecry:** Destroy a minion and gain its Health. *(destroy + stat gain)*

### `src/cards/classic_rogue.rs` (8)

- [ ] **Shadowstep** — Return a friendly minion to your hand. It costs (2) less. *(bounce + cost reduction — bounce itself exists)*
- [ ] **Betrayal** — Deal damage equal to the target's Attack to adjacent minions. *(adjacency damage)*
- [ ] **Blade Flurry** — Destroy your weapon and deal its Attack damage to all enemies. *(weapon destruction + AoE)*
- [ ] **Patient Assassin** — **Stealth**, **Poison** *(Poison keyword)*
- [ ] **Headcrack** — Deal 2 damage to the enemy hero. **Combo:** Return this to your hand. *(Combo keyword)*
- [ ] **Perdition's Blade** — weapon 2/2; **Battlecry:** Deal 1 damage. *(weapon with Battlecry)*
- [ ] **Master of Disguise** — **Battlecry:** Give a friendly minion **Stealth**. *(grant Stealth)*
- [ ] **Kidnapper** — **Combo:** Return an enemy minion to your hand. *(Combo + bounce enemy)*

### `src/cards/classic_shaman.rs` (2)

- [ ] **Far Sight** — Draw a card. It costs (3) less. *(draw with cost reduction)*
- [ ] **Unbound Elemental** — Whenever you play a card with **Overload**, gain +1/+1. *(Overload trigger)*

### `src/cards/classic_warlock.rs` (2)

- [ ] **Corruption** — Choose an enemy minion. At the start of your turn, destroy it. *(delayed start-of-turn destroy)*
- [ ] **Summoning Portal** — Your minions cost (2) less, but not less than (1). *(cost-reduction aura with floor)*

### `src/cards/classic_warrior.rs` (1)

- [ ] **Commanding Shout** — Your minions can't go below 1 Health this turn. *(minimum-health effect)*

---

## Tier 3 — Deferred: random / discover / opponent interactions (16)

These need a random card pool or card-generation framework. The runtime registry (`ALL_CARDS` in `src/cards/sets.rs`) is the natural base for sampling, but class filtering and token pools (Bananas, Dream Cards, Demons, Beasts) need to be defined first.

### `src/cards/classic_neutral.rs` (6)

- [ ] **Brightwing** — **Battlecry:** Add a random Legendary minion to your hand.
- [ ] **Nozdormu** — 15-second turn timer. *(suggest: implement as a plain 8/8 with a comment — timer is meaningless in sim)*
- [ ] **Xavius** — At the end of your turn, add a random Shadow spell to your hand.
- [ ] **Ysera** — At the end of your turn, draw a Dream Card. *(needs the Dream Card pool)*
- [ ] **Barrens Stablehand** — **Battlecry:** Summon a random Beast. *(needs a random Beast pool)*
- [ ] **Tinkmaster Overspark** — **Battlecry:** Transform a minion into a 5/5 Devilsaur or a 1/1 Squirrel at random. *(transform + random choice; needs Transform mechanic first — from Tier 2)*

### `src/cards/classic_hunter.rs` (1)

- [ ] **Animal Companion** — Summon a random 4/4 Carcass, 4/2 Leokk, or 2/4 Misha. *(random among fixed companions — easy once random summon exists)*

### `src/cards/classic_mage.rs` (2)

- [ ] **Tome of Intellect** — Add a random Mage spell to your hand.
- [ ] **Archmage Antonidas** — Whenever you cast a spell, add a Fireball to your hand.

### `src/cards/classic_priest.rs` (4)

- [ ] **Mind Vision** — Copy a random card from your opponent's hand to yours.
- [ ] **Thoughtsteal** — Copy 2 random cards from your opponent's deck.
- [ ] **Mindgames** — Summon a copy of a random minion from your opponent's deck.
- [ ] **Mind Control** — Take control of an enemy minion. *(not actually random — it is in this tier because permanent mind control needs the mechanic from Shadow Madness; implement right after it)*

### `src/cards/classic_rogue.rs` (1)

- [ ] **Pilfer** — Add a random card from another class to your hand. *(needs cross-class sampling)*

### `src/cards/classic_warlock.rs` (2)

- [ ] **Call of the Void** — Add a random Demon to your hand. *(needs a Demon pool)*
- [ ] **Bane of Doom** — Deal 2 damage to a character. If it dies, summon a random Demon. *(damage + conditional random summon)*

---

## Mechanic checklist

Engine features to build before/while implementing Tier 2 & 3. Roughly ordered by number of cards they unblock:

- [ ] **Combo keyword** — unlocks: Headcrack, Kidnapper
- [ ] **Choose One** — unlocks: Cenarius, Keeper of the Grove
- [ ] **Attack/spell-target redirection** — unlocks: Misdirection, Noble Sacrifice, Spellbender
- [ ] **Immune** — unlocks: Bestial Wrath, Gladiator's Longbow
- [ ] **Temporary mind control (until end of turn)** — unlocks: Shadow Madness; then permanent control → Mind Control
- [ ] **Transform** — unlocks: Tinkmaster Overspark
- [ ] **Cost-reduction auras & one-shot cost effects** — unlocks: Sorcerer's Apprentice, Summoning Portal, Kirin Tor Mage, Shadowstep, Far Sight
- [ ] **Poison** — unlocks: Patient Assassin
- [ ] **Weapon battlecry** — unlocks: Perdition's Blade
- [ ] **Weapon destruction + AoE** — unlocks: Blade Flurry
- [ ] **Grant deathrattle / grant Stealth** — unlocks: Soul of the Forest, Master of Disguise
- [ ] **Adjacency damage** — unlocks: Betrayal
- [ ] **Delayed start-of-turn destroy** — unlocks: Corruption
- [ ] **Minimum-health effect** — unlocks: Commanding Shout
- [ ] **Overload trigger** — unlocks: Unbound Elemental
- [ ] **Spell-cast → card-to-hand** — unlocks: Lorewalker Cho, Archmage Antonidas, Tome of Intellect
- [ ] **Random card pool framework** (class/rarity/type filtering + token pools) — unlocks: all of Tier 3

---

## Milestones

### Milestone 0 — Doc reconciliation

`classic-cards.md` / `classic-cards-zh.md` statuses are stale in both directions. Before relying on them:

- [ ] Fix 20 cards marked ✅ that don't exist in code (→ Tier 1 work)
- [ ] Add 11 implemented cards that are absent from the docs: Illidan Stormrage, Ice Block, Azure Drake, Acolyte of Pain, Spellbreaker, Auchenai Soulpriest, Doomguard, Holy Fire, Eater of Secrets, Molten Giant, Mountain Giant
- [ ] Re-mark cards that are actually implemented but flagged 🔧 (e.g. The Black Knight, Captain Greenskin, Harrison Jones, Vaporize, Faceless Manipulator, Vanish, Flare, Houndmaster, Nourish, Starfall, Gadgetzan Auctioneer, and others — verify each against code)
- [ ] Remove duplicate entries in `ALL_CARDS` (src/cards/sets.rs): VAPORIZE, FACELESS_MANIPULATOR, ANCESTRAL_SPIRIT, THE_BLACK_KNIGHT, MILLHOUSE_MANASTORM, NAT_PAGLE, HEROIC_STRIKE
- [ ] Update the stats header line in both docs

### Milestone 1 — Tier 1: 20 data-only cards

No engine changes. Pure `CardDef` additions, one commit per class file.

### Milestone 2 — Tier 2: 30 mechanic cards

Build the [mechanic checklist](#mechanic-checklist) items in order, then implement the cards each one unlocks. Group commits by mechanic, not by class, where practical.

### Milestone 3 — Tier 3: 16 random/generation cards

Needs the random card pool framework. Nozdormu can be done any time as a plain 8/8.

---

## Definition of Done

For every card in this roadmap:

- [ ] `CardDef` added to the correct `src/cards/classic_*.rs` file, following existing ordering
- [ ] Registered in `ALL_CARDS` in `src/cards/sets.rs`
- [ ] Unit test covering the card's effect; `cargo test` passes
- [ ] `cargo fmt` and `cargo clippy` clean
- [ ] Status flipped to ✅ in both `docs/classic-cards.md` and `docs/classic-cards-zh.md`
- [ ] Checkbox ticked in this roadmap
