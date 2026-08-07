# Classic/Basic Set Card List

> Source: [Hearthstone Wiki - Classic (2014-2021)](https://hearthstone.wiki.gg/wiki/Classic_(2014-2021)), [Basic card list](https://hearthstone.wiki.gg/wiki/Basic_card_list)
> Total: 375 | ✅ fully implemented: 371 | ⬜ pending: 0 | 🔧 needs new mechanic: 0 | ⏸️ skipped: 4 (status column re-audited 2026-08-07 against the code)

Legend: ✅ fully implemented | ⬜ Pending (engine supports) | 🔧 Needs new engine mechanic | ⏸️ Skipped (opponent hand/deck, may introduce other-set cards)

> **2026-08-07 re-audit.** The 24 rows that used to carry `✅ (simplified…)` notes
> were written while fidelity-debt §11 was open. Both §11 and §12 are empty now
> and `src/cards/` carries zero `(simplified: …)` markers, so those notes were
> stale — 17 of them were already faithful. Reading the implementations turned up
> 10 rows that still deviated from real HS; all 10 were fixed the same day and are
> recorded in [fidelity-debt.md](fidelity-debt.md) §13. What changed:
>
> - **Enrage (6)** — Amani Berserker, Raging Worgen, Tauren Warrior, Spiteful
>   Smith, Angry Chicken, Grommash Hellscream. Enrage was a *permanent* stat gain
>   on the `ThisMinionDamaged` trigger, so it stacked per damage instance and
>   survived a full heal. It is now an `Enrage` component resolved on read: a flat
>   bonus while damaged, gone at full Health, and removed by Silence.
>   (Gurubashi Berserker is *not* an Enrage minion — its real text is a permanent
>   stacking buff — so it keeps the trigger.)
> - **Warsong Commander** — was a `GrantCharge` aura on every other friendly
>   minion. Now a summon trigger with a `max_attack` of 3, so only minions
>   summoned with 3 or less Attack get Charge, and the Charge outlives the
>   commander.
> - **Northshire Cleric** — fired on friendly *character* heals. Now fires on the
>   new `MinionHealed` event: any minion, either side, heroes excluded.
> - **Onyxia** — summoned a fixed 5 Whelps. Now 6, which `resolve_summon`'s
>   board-size check turns into "until your side of the battlefield is full".
> - **Prophet Velen** — the spell half was modeled as +1 Spell Damage, which did
>   nothing because Spell Damage was never applied to any damage in the engine
>   (`total_spell_damage` had zero callers). Spell Damage is now wired, and Velen
>   doubles spell and hero-power damage and healing on top of it.
>
> Wiring Spell Damage also fixed every Spell Damage card in the set — Kobold
> Geomancer, Dalaran Mage, Ogre Magi, Archmage, Bloodmage Thalnos, Azure Drake,
> Malygos and Ancient Mage were all effectively vanilla before.
>
> The Text column for Truesilver Champion said "restore 3 Health"; the card and
> the code both restore 2, so the doc was corrected. The Neutral Legendary section
> header said 23 while the table holds 24 rows (header fixed).

---

## Neutral — Basic (Free)

| # | Card | Cost | Atk/HP | Type | Text | Status |
|---|------|------|--------|------|------|--------|
| 1 | Elven Archer | 1 | 1/1 | Minion | **Battlecry:** Deal 1 damage. | ✅ |
| 2 | Goldshire Footman | 1 | 1/2 | Minion | **Taunt** | ✅ |
| 3 | Grimscale Oracle | 1 | 1/1 | Murloc Minion | Your other Murlocs have +1 Attack. | ✅ |
| 4 | Murloc Raider | 1 | 2/1 | Murloc Minion | — | ✅ |
| 5 | Stonetusk Boar | 1 | 1/1 | Beast Minion | **Charge** | ✅ |
| 6 | Voodoo Doctor | 1 | 2/1 | Minion | **Battlecry:** Restore 2 Health. | ✅ |
| 7 | Acidic Swamp Ooze | 2 | 3/2 | Minion | **Battlecry:** Destroy your opponent's weapon. | ✅ |
| 8 | Bloodfen Raptor | 2 | 3/2 | Beast Minion | — | ✅ |
| 9 | Bluegill Warrior | 2 | 2/1 | Murloc Minion | **Charge** | ✅ |
| 10 | Frostwolf Grunt | 2 | 2/2 | Minion | **Taunt** | ✅ |
| 11 | Kobold Geomancer | 2 | 2/2 | Minion | **Spell Damage +1** | ✅ |
| 12 | Murloc Tidehunter | 2 | 2/1 | Murloc Minion | **Battlecry:** Summon a 1/1 Murloc Scout. | ✅ |
| 13 | Novice Engineer | 2 | 1/1 | Minion | **Battlecry:** Draw a card. | ✅ |
| 14 | River Crocolisk | 2 | 2/3 | Beast Minion | — | ✅ |
| 15 | Dalaran Mage | 3 | 1/4 | Minion | **Spell Damage +1** | ✅ |
| 16 | Ironforge Rifleman | 3 | 2/2 | Minion | **Battlecry:** Deal 1 damage. | ✅ |
| 17 | Ironfur Grizzly | 3 | 3/3 | Beast Minion | **Taunt** | ✅ |
| 18 | Magma Rager | 3 | 5/1 | Elemental Minion | — | ✅ |
| 19 | Raid Leader | 3 | 2/3 | Minion | Your other minions have +1 Attack. | ✅ |
| 20 | Razorfen Hunter | 3 | 2/3 | Minion | **Battlecry:** Summon a 1/1 Boar. | ✅ |
| 21 | Shattered Sun Cleric | 3 | 3/2 | Minion | **Battlecry:** Give a friendly minion +1/+1. | ✅ |
| 22 | Silverback Patriarch | 3 | 1/4 | Beast Minion | **Taunt** | ✅ |
| 23 | Wolfrider | 3 | 3/1 | Minion | **Charge** | ✅ |
| 24 | Chillwind Yeti | 4 | 4/5 | Minion | — | ✅ |
| 25 | Dragonling Mechanic | 4 | 2/4 | Minion | **Battlecry:** Summon a 2/1 Mechanical Dragonling. | ✅ |
| 26 | Gnomish Inventor | 4 | 2/4 | Minion | **Battlecry:** Draw a card. | ✅ |
| 27 | Oasis Snapjaw | 4 | 2/7 | Beast Minion | — | ✅ |
| 28 | Ogre Magi | 4 | 4/4 | Minion | **Spell Damage +1** | ✅ |
| 29 | Sen'jin Shieldmasta | 4 | 3/5 | Minion | **Taunt** | ✅ |
| 30 | Stormwind Knight | 4 | 2/5 | Minion | **Charge** | ✅ |
| 31 | Booty Bay Bodyguard | 5 | 5/4 | Minion | **Taunt** | ✅ |
| 32 | Darkscale Healer | 5 | 4/5 | Naga Minion | **Battlecry:** Restore 2 Health to all friendly characters. | ✅ |
| 33 | Frostwolf Warlord | 5 | 4/4 | Minion | **Battlecry:** Gain +1/+1 for each other friendly minion on the battlefield. | ✅ |
| 34 | Gurubashi Berserker | 5 | 2/8 | Minion | Whenever this minion takes damage, gain +3 Attack. | ✅ |
| 35 | Nightblade | 5 | 4/4 | Minion | **Battlecry:** Deal 3 damage to the enemy hero. | ✅ |
| 36 | Stormpike Commando | 5 | 4/2 | Minion | **Battlecry:** Deal 2 damage. | ✅ |
| 37 | Archmage | 6 | 4/7 | Minion | **Spell Damage +1** | ✅ |
| 38 | Boulderfist Ogre | 6 | 6/7 | Minion | — | ✅ |
| 39 | Lord of the Arena | 6 | 6/5 | Minion | **Taunt** | ✅ |
| 40 | Reckless Rocketeer | 6 | 5/2 | Minion | **Charge** | ✅ |
| 41 | Core Hound | 7 | 9/5 | Minion | — | ✅ |
| 42 | Stormwind Champion | 7 | 6/6 | Minion | Your other minions have +1/+1. | ✅ |
| 43 | War Golem | 7 | 7/7 | Minion | — | ✅ |

## Neutral — Classic Common (38)

| # | Card | Cost | Atk/HP | Text | Status |
|---|------|------|--------|------|--------|
| 1 | Wisp | 0 | 1/1 | — | ✅ |
| 2 | Abusive Sergeant | 1 | 2/1 | **Battlecry:** Give a friendly minion +2 Attack this turn. | ✅ |
| 3 | Argent Squire | 1 | 1/1 | **Divine Shield** | ✅ |
| 4 | Leper Gnome | 1 | 2/1 | **Deathrattle:** Deal 2 damage to the enemy hero. | ✅ |
| 5 | Shieldbearer | 1 | 0/4 | **Taunt** | ✅ |
| 6 | Southsea Deckhand | 1 | 2/1 | Has **Charge** while you have a weapon equipped. | ✅ |
| 7 | Worgen Infiltrator | 1 | 2/1 | **Stealth** | ✅ |
| 8 | Young Dragonhawk | 1 | 1/1 | **Windfury** | ✅ |
| 9 | Amani Berserker | 2 | 2/3 | **Enrage:** +3 Attack. | ✅ |
| 10 | Bloodsail Raider | 2 | 2/3 | **Battlecry:** Gain Attack equal to the Attack of your weapon. | ✅ |
| 11 | Dire Wolf Alpha | 2 | 2/2 | Adjacent minions have +1 Attack. | ✅ |
| 12 | Faerie Dragon | 2 | 3/2 | Can't be targeted by spells or Hero Powers. | ✅ |
| 13 | Loot Hoarder | 2 | 2/1 | **Deathrattle:** Draw a card. | ✅ |
| 14 | Mad Bomber | 2 | 3/2 | **Battlecry:** Deal 3 damage randomly split among all other characters. | ✅ |
| 15 | Youthful Brewmaster | 2 | 3/2 | **Battlecry:** Return a friendly minion from the battlefield to your hand. | ✅ |
| 16 | Earthen Ring Farseer | 3 | 3/3 | **Battlecry:** Restore 3 Health. | ✅ |
| 17 | Flesheating Ghoul | 3 | 3/3 | Whenever a minion dies, gain +1 Attack. | ✅ |
| 18 | Harvest Golem | 3 | 2/3 | **Deathrattle:** Summon a 2/1 Damaged Golem. | ✅ |
| 19 | Ironbeak Owl | 2 | 2/1 | **Battlecry:** **Silence** a minion. | ✅ |
| 20 | Jungle Panther | 3 | 4/2 | **Stealth** | ✅ |
| 21 | Raging Worgen | 3 | 3/3 | **Enrage:** **Windfury** and +1 Attack. | ✅ |
| 22 | Scarlet Crusader | 3 | 3/1 | **Divine Shield** | ✅ |
| 23 | Tauren Warrior | 3 | 2/3 | **Taunt** **Enrage:** +3 Attack. | ✅ |
| 24 | Thrallmar Farseer | 3 | 2/3 | **Windfury** | ✅ |
| 25 | Ancient Brewmaster | 4 | 5/4 | **Battlecry:** Return a friendly minion from the battlefield to your hand. | ✅ |
| 26 | Cult Master | 4 | 4/2 | Whenever one of your other minions dies, draw a card. | ✅ |
| 27 | Dark Iron Dwarf | 4 | 4/4 | **Battlecry:** Give a friendly minion +2 Attack this turn. | ✅ |
| 28 | Dread Corsair | 4 | 3/3 | **Taunt.** Costs (1) less per Attack of your weapon. | ✅ |
| 29 | Mogu'shan Warden | 4 | 1/7 | **Taunt** | ✅ |
| 30 | Silvermoon Guardian | 4 | 3/3 | **Divine Shield** | ✅ |
| 31 | Fen Creeper | 5 | 3/6 | **Taunt** | ✅ |
| 32 | Silver Hand Knight | 5 | 4/4 | **Battlecry:** Summon a 2/2 Squire. | ✅ |
| 33 | Spiteful Smith | 5 | 4/6 | **Enrage:** Your weapon has +2 Attack. | ✅ |
| 34 | Stranglethorn Tiger | 5 | 5/5 | **Stealth** | ✅ |
| 35 | Venture Co. Mercenary | 5 | 7/6 | Your minions cost (3) more. | ✅ |
| 36 | Frost Elemental | 6 | 5/5 | **Battlecry:** **Freeze** a character. | ✅ |
| 37 | Priestess of Elune | 6 | 5/4 | **Battlecry:** Restore 4 Health to your hero. | ✅ |
| 38 | Windfury Harpy | 6 | 4/5 | **Windfury** | ✅ |

## Neutral — Classic Rare (35)

| # | Card | Cost | Atk/HP | Text | Status |
|---|------|------|--------|------|--------|
| 1 | Angry Chicken | 1 | 1/1 | **Enrage:** +5 Attack. | ✅ |
| 2 | Bloodsail Corsair | 1 | 1/2 | **Battlecry:** Remove 1 Durability from your opponent's weapon. | ✅ |
| 3 | Lightwarden | 1 | 1/2 | Whenever a character is healed, gain +2 Attack. | ✅ |
| 4 | Murloc Tidecaller | 1 | 1/2 | Whenever you summon a Murloc, gain +1 Attack. | ✅ |
| 5 | Secretkeeper | 1 | 1/2 | Whenever a **Secret** is played, gain +1/+1. | ✅ |
| 6 | Young Priestess | 1 | 2/1 | At the end of your turn, give another random friendly minion +1 Health. | ✅ |
| 7 | Ancient Watcher | 2 | 4/5 | Can't attack. | ✅ |
| 8 | Crazed Alchemist | 2 | 2/2 | **Battlecry:** Swap the Attack and Health of a minion. | ✅ |
| 9 | Knife Juggler | 2 | 3/2 | After you summon a minion, deal 1 damage to a random enemy. | ✅ |
| 10 | Mana Addict | 2 | 1/3 | Whenever you cast a spell, gain +2 Attack this turn. | ✅ |
| 11 | Mana Wraith | 2 | 2/2 | ALL minions cost (1) more. | ✅ |
| 12 | Master Swordsmith | 2 | 1/3 | At the end of your turn, give another random friendly minion +1 Attack. | ✅ |
| 13 | Pint-Sized Summoner | 2 | 2/2 | The first minion you play each turn costs (1) less. | ✅ |
| 14 | Sunfury Protector | 2 | 2/3 | **Battlecry:** Give adjacent minions **Taunt**. | ✅ |
| 15 | Wild Pyromancer | 2 | 3/2 | After you cast a spell, deal 1 damage to ALL minions. | ✅ |
| 16 | Alarm-o-Bot | 3 | 0/3 | At the start of your turn, swap this minion with a random one in your hand. | ✅ |
| 17 | Arcane Golem | 3 | 4/4 | **Charge.** **Battlecry:** Give your opponent a Mana Crystal. | ✅ |
| 18 | Coldlight Seer | 3 | 2/3 | **Battlecry:** Give ALL other Murlocs +2 Health. | ✅ |
| 19 | Demolisher | 3 | 1/4 | At the start of your turn, deal 2 damage to a random enemy. | ✅ |
| 20 | Emperor Cobra | 3 | 2/3 | Destroy any minion damaged by this minion. | ✅ |
| 21 | Imp Master | 3 | 1/5 | At the end of your turn, deal 1 damage to this minion and summon a 1/1 Imp. | ✅ |
| 22 | Injured Blademaster | 3 | 4/7 | **Battlecry:** Deal 4 damage to HIMSELF. | ✅ |
| 23 | Questing Adventurer | 3 | 2/2 | Whenever you play a card, gain +1/+1. | ✅ |
| 24 | Ancient Mage | 4 | 2/5 | **Battlecry:** Give adjacent minions **Spell Damage +1**. | ✅ |
| 25 | Defender of Argus | 4 | 2/3 | **Battlecry:** Give adjacent minions +1/+1 and **Taunt**. | ✅ |
| 26 | SI:7 Infiltrator | 4 | 5/4 | **Battlecry:** Destroy a random enemy **Secret**. | ✅ |
| 27 | Twilight Drake | 4 | 4/1 | **Battlecry:** Gain +1 Health for each card in your hand. | ✅ |
| 28 | Violet Teacher | 4 | 3/5 | Whenever you cast a spell, summon a 1/1 Violet Apprentice. | ✅ |
| 29 | Abomination | 5 | 4/4 | **Taunt.** **Deathrattle:** Deal 2 damage to ALL characters. | ✅ |
| 30 | Stampeding Kodo | 5 | 3/5 | **Battlecry:** Destroy a random enemy minion with 2 or less Attack. | ✅ |
| 31 | Argent Commander | 6 | 4/2 | **Charge.** **Divine Shield** | ✅ |
| 32 | Sunwalker | 6 | 4/5 | **Taunt.** **Divine Shield** | ✅ |
| 33 | Gadgetzan Auctioneer | 5 | 4/4 | Whenever you cast a spell, draw a card. | ✅ |
| 34 | Ravenholdt Assassin | 7 | 7/5 | **Stealth** | ✅ |
| 35 | Arcane Devourer | 8 | 4/8 | Whenever you cast a spell, gain +2/+2. | ✅ |

## Neutral — Classic Epic (9)

| # | Card | Cost | Atk/HP | Text | Status |
|---|------|------|--------|------|--------|
| 1 | Hungry Crab | 1 | 1/2 | **Battlecry:** Destroy a Murloc and gain +2/+2. | ✅ |
| 2 | Doomsayer | 2 | 0/7 | At the start of your turn, destroy ALL minions. | ✅ |
| 3 | Blood Knight | 3 | 3/3 | **Battlecry:** All minions lose **Divine Shield**. Gain +3/+3 for each Shield lost. | ✅ |
| 4 | Murloc Warleader | 3 | 3/3 | Your other Murlocs have +2/+1. | ✅ |
| 5 | Southsea Captain | 3 | 3/3 | Your other Pirates have +1/+1. | ✅ |
| 6 | Big Game Hunter | 5 | 4/2 | **Battlecry:** Destroy a minion with 7 or more Attack. | ✅ |
| 7 | Faceless Manipulator | 5 | 3/3 | **Battlecry:** Choose a minion and become a copy of it. | ✅ |
| 8 | Barrens Stablehand | 7 | 5/5 | **Battlecry:** Summon a random Beast. | ✅ |
| 9 | Sea Giant | 10 | 8/8 | Costs (1) less for each other minion on the battlefield. | ✅ |

## Neutral — Classic Legendary (24)

| # | Card | Cost | Atk/HP | Text | Status |
|---|------|------|--------|------|--------|
| 1 | Bloodmage Thalnos | 2 | 1/1 | **Spell Damage +1.** **Deathrattle:** Draw a card. | ✅ |
| 2 | Brightwing | 2 | 3/2 | **Battlecry:** Add a random Legendary minion to your hand. | ✅ |
| 3 | Lorewalker Cho | 2 | 0/4 | Whenever a player casts a spell, put a copy into the other player's hand. | ⏸️ may introduce cards from other sets |
| 4 | Millhouse Manastorm | 2 | 4/4 | **Battlecry:** Enemy spells cost (0) next turn. | ✅ |
| 5 | Nat Pagle | 2 | 0/4 | At the start of your turn, you have a 50% chance to draw an extra card. | ✅ |
| 6 | King Mukla | 3 | 5/5 | **Battlecry:** Give your opponent 2 Bananas. | ✅ |
| 7 | Tinkmaster Overspark | 3 | 3/3 | **Battlecry:** Transform a minion into a 5/5 Devilsaur or a 1/1 Squirrel at random. | ✅ |
| 8 | The Black Knight | 6 | 4/5 | **Battlecry:** Destroy an enemy minion with **Taunt**. | ✅ |
| 9 | Captain Greenskin | 5 | 5/4 | **Battlecry:** Give your weapon +1/+1. | ✅ |
| 10 | Harrison Jones | 5 | 5/4 | **Battlecry:** Destroy your opponent's weapon and draw cards equal to its Durability. | ✅ |
| 11 | Cairne Bloodhoof | 6 | 4/5 | **Deathrattle:** Summon a 4/5 Baine Bloodhoof. | ✅ |
| 12 | High Inquisitor Whitemane | 6 | 5/7 | **Battlecry:** Summon all friendly minions that died this turn. | ✅ |
| 13 | Hogger | 6 | 4/4 | At the end of your turn, summon a 2/2 Gnoll with **Taunt**. | ✅ |
| 14 | The Beast | 6 | 9/7 | **Deathrattle:** Summon a 3/3 Finkle Einhorn for your opponent. | ✅ |
| 15 | Xavius | 6 | — | — | ✅ |
| 16 | Baron Geddon | 7 | 7/5 | At the end of your turn, deal 2 damage to ALL other characters. | ✅ |
| 17 | Gruul | 8 | 7/7 | At the end of each turn, gain +1/+1. | ✅ |
| 18 | Ragnaros the Firelord | 8 | 8/8 | Can't attack. At the end of your turn, deal 8 damage to a random enemy. | ✅ |
| 19 | Alexstrasza | 9 | 8/8 | **Battlecry:** Set a hero's remaining Health to 15. | ✅ |
| 20 | Malygos | 9 | 4/12 | **Spell Damage +5** | ✅ |
| 21 | Nozdormu | 9 | 8/8 | Players only have 15 seconds to take their turns. | ✅ |
| 22 | Onyxia | 9 | 8/8 | **Battlecry:** Summon 1/1 Whelps until your side of the battlefield is full. | ✅ |
| 23 | Ysera | 9 | 4/12 | At the end of your turn, draw a Dream Card. | ✅ |
| 24 | Deathwing | 10 | 12/12 | **Battlecry:** Destroy all other minions and discard your hand. | ✅ |

---

## Druid

### Druid — Basic (10)

| # | Card | Cost | Atk/HP | Type | Text | Status |
|---|------|------|--------|------|------|--------|
| 1 | Innervate | 0 | — | Nature Spell | Gain 1 Mana Crystal this turn only. | ✅ |
| 2 | Moonfire | 0 | — | Arcane Spell | Deal 1 damage. | ✅ |
| 3 | Claw | 1 | — | Spell | Give your hero +2 Attack this turn. Gain 2 Armor. | ✅ |
| 4 | Mark of the Wild | 2 | — | Nature Spell | Give a minion **Taunt** and +2/+3. | ✅ |
| 5 | Wild Growth | 2 | — | Nature Spell | Gain an empty Mana Crystal. | ✅ |
| 6 | Healing Touch | 3 | — | Nature Spell | Restore 8 Health. | ✅ |
| 7 | Savage Roar | 3 | — | Spell | Give your characters +2 Attack this turn. | ✅ |
| 8 | Swipe | 4 | — | Spell | Deal 4 damage to an enemy and 1 damage to all other enemies. | ✅ |
| 9 | Starfire | 6 | — | Arcane Spell | Deal 5 damage. Draw a card. | ✅ |
| 10 | Ironbark Protector | 8 | 8/8 | Minion | **Taunt** | ✅ |

### Druid — Classic (15)

| # | Card | Cost | Atk/HP | Text | Status |
|---|------|------|--------|------|--------|
| 1 | Savagery | 1 | — | Spell | Deal damage equal to your hero's Attack to a minion. | ✅ |
| 2 | Power of the Wild | 2 | — | Spell | **Choose One** - Give your minions +1/+1; or Summon a 3/2 Panther. | ✅ |
| 3 | Wrath | 2 | — | Spell | **Choose One** - Deal 3 damage to a minion; or Deal 1 damage and draw a card. | ✅ |
| 4 | Mark of Nature | 3 | — | Spell | **Choose One** - Give a minion +4 Attack; or +4 Health and **Taunt**. | ✅ |
| 5 | Soul of the Forest | 4 | — | Spell | Give your minions "**Deathrattle:** Summon a 2/2 Treant." | ✅ |
| 6 | Bite | 4 | — | Spell | Give your hero +4 Attack this turn. Gain 4 Armor. | ✅ |
| 7 | Keeper of the Grove | 4 | 2/4 | Minion | **Choose One** - Deal 2 damage; or **Silence** a minion. | ✅ |
| 8 | Force of Nature | 6 | — | Spell | Summon three 2/2 Treants with **Charge**. | ✅ |
| 9 | Nourish | 5 | — | Spell | **Choose One** - Gain 2 Mana Crystals; or Draw 3 cards. | ✅ |
| 10 | Starfall | 5 | — | Spell | **Choose One** - Deal 5 damage to a minion; or Deal 2 damage to all enemy minions. | ✅ |
| 11 | Druid of the Claw | 5 | 4/4 | Minion | **Choose One** - **Charge**; or +2 Health and **Taunt**. | ✅ |
| 12 | Ancient of Lore | 7 | 5/5 | Minion | **Choose One** - Draw 2 cards; or Restore 5 Health. | ✅ |
| 13 | Ancient of War | 7 | 5/5 | Minion | **Choose One** - +5 Attack; or +5 Health and **Taunt**. | ✅ |
| 14 | Cenarius | 9 | 5/8 | Minion | **Choose One** - Give your other minions +2/+2; or Summon two 2/2 Treants with **Taunt**. | ✅ |
| 15 | Gift of the Wild | 8 | — | Nature Spell | Give your minions +2/+2 and **Taunt**. | ✅ |

---

## Hunter

### Hunter — Basic (10)

| # | Card | Cost | Atk/HP | Type | Text | Status |
|---|------|------|--------|------|------|--------|
| 1 | Arcane Shot | 1 | — | Arcane Spell | Deal 2 damage. | ✅ |
| 2 | Hunter's Mark | 1 | — | Spell | Change a minion's Health to 1. | ✅ |
| 3 | Timber Wolf | 1 | 1/1 | Beast Minion | Your other Beasts have +1 Attack. | ✅ |
| 4 | Tracking | 1 | — | Spell | Discover a card from your deck. | ✅ |
| 5 | Starving Buzzard | 2 | 2/1 | Beast Minion | Whenever you summon a Beast, draw a card. | ✅ |
| 6 | Animal Companion | 3 | — | Spell | Summon a random Beast Companion. | ✅ |
| 7 | Kill Command | 3 | — | Spell | Deal 3 damage. If you control a Beast, deal 5 damage instead. | ✅ |
| 8 | Houndmaster | 4 | 4/3 | Minion | **Battlecry:** Give a friendly Beast +2/+2 and **Taunt**. | ✅ |
| 9 | Multi-Shot | 4 | — | Spell | Deal 3 damage to two random enemy minions. | ✅ |
| 10 | Tundra Rhino | 5 | 2/5 | Beast Minion | Your Beasts have **Charge**. | ✅ |

### Hunter — Classic (15)

| # | Card | Cost | Atk/HP | Text | Status |
|---|------|------|--------|------|--------|
| 1 | Bestial Wrath | 1 | — | Spell | Give a friendly Beast +2 Attack and **Immune** this turn. | ✅ |
| 2 | Flare | 1 | — | Spell | Destroy all enemy **Secrets**. Draw a card. | ✅ |
| 3 | Explosive Trap | 2 | — | Spell | **Secret:** When your hero is attacked, deal 2 damage to all enemies. | ✅ |
| 4 | Freezing Trap | 2 | — | Spell | **Secret:** When an enemy minion attacks, return it to its owner's hand. It costs (2) more. | ✅ |
| 5 | Misdirection | 2 | — | Spell | **Secret:** When a character attacks your hero, instead it attacks another random character. | ✅ |
| 6 | Scavenging Hyena | 2 | 2/2 | Beast Minion | Whenever a friendly Beast dies, gain +2/+1. | ✅ |
| 7 | Snake Trap | 2 | — | Spell | **Secret:** When one of your minions is attacked, summon three 1/1 Snakes. | ✅ |
| 8 | Snipe | 2 | — | Spell | **Secret:** When your opponent plays a minion, deal 4 damage to it. | ✅ |
| 9 | Deadly Shot | 3 | — | Spell | Destroy a random enemy minion. | ✅ |
| 10 | Eaglehorn Bow | 3 | 3/2 | Weapon | Whenever a friendly **Secret** is revealed, gain +1 Durability. | ✅ |
| 11 | Unleash the Hounds | 3 | — | Spell | For each enemy minion, summon a 1/1 Hound with **Charge**. | ✅ |
| 12 | Explosive Shot | 5 | — | Spell | Deal 5 damage to a minion and 2 damage to adjacent ones. | ✅ |
| 13 | Savannah Highmane | 6 | 6/5 | Beast Minion | **Deathrattle:** Summon two 2/2 Hyenas. | ✅ |
| 14 | Gladiator's Longbow | 7 | 5/2 | Weapon | Your hero is **Immune** while attacking. | ✅ |
| 15 | King Krush | 9 | 8/8 | Beast Minion | **Charge** | ✅ |

---

## Mage

### Mage — Basic (10)

| # | Card | Cost | Atk/HP | Type | Text | Status |
|---|------|------|--------|------|------|--------|
| 1 | Arcane Missiles | 1 | — | Arcane Spell | Deal 3 damage randomly split among all enemies. | ✅ |
| 2 | Mirror Image | 1 | — | Spell | Summon two 0/2 minions with **Taunt**. | ✅ |
| 3 | Arcane Explosion | 2 | — | Arcane Spell | Deal 1 damage to all enemy minions. | ✅ |
| 4 | Frostbolt | 2 | — | Frost Spell | Deal 3 damage to a character and **Freeze** it. | ✅ |
| 5 | Arcane Intellect | 3 | — | Arcane Spell | Draw 2 cards. | ✅ |
| 6 | Frost Nova | 3 | — | Frost Spell | **Freeze** all enemy minions. | ✅ |
| 7 | Fireball | 4 | — | Fire Spell | Deal 6 damage. | ✅ |
| 8 | Polymorph | 4 | — | Arcane Spell | Transform a minion into a 1/1 Sheep. | ✅ |
| 9 | Water Elemental | 4 | 3/6 | Elemental Minion | **Freeze** any character damaged by this minion. | ✅ |
| 10 | Flamestrike | 7 | — | Fire Spell | Deal 4 damage to all enemy minions. | ✅ |

### Mage — Classic (15)

| # | Card | Cost | Atk/HP | Text | Status |
|---|------|------|--------|------|--------|
| 1 | Mana Wyrm | 1 | 1/3 | Minion | Whenever you cast a spell, gain +1 Attack. | ✅ |
| 2 | Tome of Intellect | 1 | — | Arcane Spell | Add a random Mage spell to your hand. | ✅ |
| 3 | Icicle | 2 | — | Frost Spell | Deal 2 damage to a minion. If it's **Frozen**, draw a card. | ✅ |
| 4 | Sorcerer's Apprentice | 2 | 3/2 | Minion | Your spells cost (1) less. | ✅ |
| 5 | Cone of Cold | 4 | — | Frost Spell | **Freeze** a minion and its adjacent minions. | ✅ |
| 6 | Counterspell | 3 | — | Spell | **Secret:** When your opponent casts a spell, **Counter** it. | ✅ |
| 7 | Ice Barrier | 3 | — | Spell | **Secret:** When your hero is attacked, gain 8 Armor. | ✅ |
| 8 | Kirin Tor Mage | 3 | 4/3 | Minion | **Battlecry:** The next **Secret** you play this turn costs (0). | ✅ |
| 9 | Mirror Entity | 3 | — | Spell | **Secret:** When your opponent plays a minion, summon a copy of it. | ✅ |
| 10 | Spellbender | 3 | — | Spell | **Secret:** When an enemy casts a spell on a minion, summon a 1/3 as the new target. | ✅ |
| 11 | Vaporize | 3 | — | Spell | **Secret:** When a minion attacks your hero, destroy it. | ✅ |
| 12 | Ethereal Arcanist | 4 | 3/3 | Minion | At the end of your turn, if you control a **Secret**, gain +2/+2. | ✅ |
| 13 | Blizzard | 5 | — | Frost Spell | Deal 2 damage to all enemy minions and **Freeze** them. | ✅ |
| 14 | Archmage Antonidas | 7 | 5/7 | Minion | Whenever you cast a spell, put a 'Fireball' spell into your hand. | ✅ |
| 15 | Pyroblast | 10 | — | Fire Spell | Deal 10 damage. | ✅ |

---

## Paladin

### Paladin — Basic (10)

| # | Card | Cost | Atk/HP | Type | Text | Status |
|---|------|------|--------|------|------|--------|
| 1 | Blessing of Might | 1 | — | Holy Spell | Give a minion +3 Attack. | ✅ |
| 2 | Hand of Protection | 1 | — | Holy Spell | Give a minion **Divine Shield**. | ✅ |
| 3 | Humility | 1 | — | Spell | Change a minion's Attack to 1. | ✅ |
| 4 | Light's Justice | 1 | 1/4 | Weapon | — | ✅ |
| 5 | Holy Light | 2 | — | Holy Spell | Restore 8 Health to your hero. | ✅ |
| 6 | Consecration | 4 | — | Holy Spell | Deal 2 damage to all enemies. | ✅ |
| 7 | Hammer of Wrath | 4 | — | Holy Spell | Deal 3 damage. Draw a card. | ✅ |
| 8 | Blessing of Kings | 4 | — | Holy Spell | Give a minion +4/+4. (+4 Attack/+4 Health) | ✅ |
| 9 | Truesilver Champion | 4 | 4/2 | Weapon | Whenever your hero attacks, restore 2 Health to it. | ✅ |
| 10 | Guardian of Kings | 7 | 5/6 | Minion | **Taunt.** **Battlecry:** Restore 6 Health to your hero. | ✅ |

### Paladin — Classic (15)

| # | Card | Cost | Atk/HP | Text | Status |
|---|------|------|--------|------|--------|
| 1 | Blessing of Wisdom | 1 | — | Holy Spell | Whenever the target minion attacks, draw a card. | ✅ |
| 2 | Eye for an Eye | 1 | — | Spell | **Secret:** When your hero takes damage, deal that much damage to the enemy hero. | ✅ |
| 3 | Noble Sacrifice | 1 | — | Spell | **Secret:** When an enemy attacks, summon a 2/1 Defender as the new target. | ✅ |
| 4 | Redemption | 1 | — | Spell | **Secret:** When a friendly minion dies, return it to life with 1 Health. | ✅ |
| 5 | Repentance | 1 | — | Spell | **Secret:** When your opponent plays a minion, reduce its Health to 1. | ✅ |
| 6 | Argent Protector | 2 | 2/2 | Minion | **Battlecry:** Give a friendly minion **Divine Shield**. | ✅ |
| 7 | Equality | 2 | — | Holy Spell | Change the Health of ALL minions to 1. | ✅ |
| 8 | Aldor Peacekeeper | 3 | 3/3 | Minion | **Battlecry:** Change an enemy minion's Attack to 1. | ✅ |
| 9 | Holy Wrath | 5 | — | Holy Spell | Draw a card and deal damage equal to its Cost. | ✅ |
| 10 | Sword of Justice | 3 | 1/5 | Weapon | Whenever you summon a minion, give it +1/+1 and this loses 1 Durability. | ✅ |
| 11 | Blessed Champion | 5 | — | Holy Spell | Double a minion's Attack. | ✅ |
| 12 | Righteousness | 5 | — | Holy Spell | Give your minions **Divine Shield**. | ✅ |
| 13 | Avenging Wrath | 6 | — | Holy Spell | Deal 8 damage randomly split among all enemies. | ✅ |
| 14 | Lay on Hands | 8 | — | Holy Spell | Restore 8 Health. Draw 3 cards. | ✅ |
| 15 | Tirion Fordring | 8 | 6/6 | Minion | **Divine Shield.** **Taunt.** **Deathrattle:** Equip a 5/3 Ashbringer. | ✅ |

---

## Priest

### Priest — Basic (10)

| # | Card | Cost | Atk/HP | Type | Text | Status |
|---|------|------|--------|------|------|--------|
| 1 | Holy Smite | 1 | — | Holy Spell | Deal 3 damage to a minion. | ✅ |
| 2 | Mind Vision | 1 | — | Shadow Spell | Put a copy of a random card in your opponent's hand into your hand. | ⏸️ opponent hand |
| 3 | Northshire Cleric | 1 | 1/3 | Minion | Whenever a minion is healed, draw a card. | ✅ |
| 4 | Power Word: Shield | 1 | — | Holy Spell | Give a minion +2 Health. Draw a card. | ✅ |
| 5 | Radiance | 1 | — | Holy Spell | Restore 5 Health to your hero. | ✅ |
| 6 | Divine Spirit | 2 | — | Holy Spell | Double a minion's Health. | ✅ |
| 7 | Mind Blast | 2 | — | Shadow Spell | Deal 5 damage to the enemy hero. | ✅ |
| 8 | Shadow Word: Death | 3 | — | Shadow Spell | Destroy a minion with 5 or more Attack. | ✅ |
| 9 | Shadow Word: Pain | 2 | — | Shadow Spell | Destroy a minion with 3 or less Attack. | ✅ |
| 10 | Holy Nova | 5 | — | Holy Spell | Deal 2 damage to all enemy minions. Restore 2 Health to all friendly characters. | ✅ |

### Priest — Classic (15)

| # | Card | Cost | Atk/HP | Text | Status |
|---|------|------|--------|------|--------|
| 1 | Circle of Healing | 0 | — | Holy Spell | Restore 4 Health to ALL minions. | ✅ |
| 2 | Silence | 0 | — | Spell | **Silence** a minion. | ✅ |
| 3 | Inner Fire | 1 | — | Holy Spell | Change a minion's Attack to be equal to its Health. | ✅ |
| 4 | Scarlet Subjugator | 1 | 2/1 | Minion | **Battlecry:** Give an enemy minion -2 Attack until your next turn. | ✅ |
| 5 | Kul Tiran Chaplain | 2 | 2/3 | Minion | **Battlecry:** Give a friendly minion +2 Health. | ✅ |
| 6 | Lightwell | 2 | 0/5 | Minion | At the start of your turn, restore 3 Health to a damaged friendly character. | ✅ |
| 7 | Thoughtsteal | 3 | — | Shadow Spell | Copy 2 cards from your opponent's deck and put them into your hand. | ⏸️ opponent deck |
| 8 | Lightspawn | 4 | 0/5 | Minion | This minion's Attack is always equal to its Health. | ✅ |
| 9 | Shadow Madness | 4 | — | Shadow Spell | Gain control of an enemy minion with 3 or less Attack until end of turn. | ✅ |
| 10 | Mass Dispel | 4 | — | Holy Spell | **Silence** all enemy minions. Draw a card. | ✅ |
| 11 | Mindgames | 4 | — | Shadow Spell | Put a copy of a random minion from your opponent's deck into the battlefield. | ⏸️ opponent deck |
| 12 | Shadow Word: Ruin | 4 | — | Shadow Spell | Destroy all minions with 5 or more Attack. | ✅ |
| 13 | Temple Enforcer | 6 | 6/6 | Minion | **Battlecry:** Give a friendly minion +3 Health. | ✅ |
| 14 | Cabal Shadow Priest | 6 | 4/5 | Minion | **Battlecry:** Take control of an enemy minion that has 2 or less Attack. | ✅ |
| 15 | Natalie Seline | 7 | 7/1 | Minion | **Battlecry:** Destroy a minion and gain its Health. | ✅ |
| 16 | Prophet Velen | 7 | 7/7 | Minion | Double the damage and healing of your spells and Hero Power. | ✅ |
| 17 | Mind Control | 10 | — | Shadow Spell | Take control of an enemy minion. | ✅ |

---

## Rogue

### Rogue — Basic (10)

| # | Card | Cost | Atk/HP | Type | Text | Status |
|---|------|------|--------|------|------|--------|
| 1 | Backstab | 0 | — | Spell | Deal 2 damage to an undamaged minion. | ✅ |
| 2 | Deadly Poison | 1 | — | Nature Spell | Give your weapon +2 Attack. | ✅ |
| 3 | Sinister Strike | 1 | — | Spell | Deal 3 damage to the enemy hero. | ✅ |
| 4 | Fan of Knives | 3 | — | Spell | Deal 1 damage to all enemy minions. Draw a card. | ✅ |
| 5 | Sap | 2 | — | Spell | Return an enemy minion to your opponent's hand. | ✅ |
| 6 | Shiv | 2 | — | Spell | Deal 1 damage. Draw a card. | ✅ |
| 7 | Assassin's Blade | 5 | 3/4 | Weapon | — | ✅ |
| 8 | Assassinate | 5 | — | Spell | Destroy an enemy minion. | ✅ |
| 9 | Sprint | 7 | — | Spell | Draw 4 cards. | ✅ |
| 10 | Vanish | 6 | — | Spell | Return all minions to their owner's hand. | ✅ |

### Rogue — Classic (15)

| # | Card | Cost | Atk/HP | Text | Status |
|---|------|------|--------|------|--------|
| 1 | Preparation | 0 | — | Spell | The next spell you cast this turn costs (3) less. | ✅ |
| 2 | Shadowstep | 0 | — | Spell | Return a friendly minion to your hand. It costs (2) less. | ✅ |
| 3 | Cold Blood | 1 | — | Spell | Give a minion +2 Attack. **Combo:** +4 Attack instead. | ✅ |
| 4 | Pilfer | 1 | — | Spell | Add a random card from another class to your hand. | ✅ |
| 5 | Betrayal | 2 | — | Spell | Force an enemy minion to deal its damage to the minions next to it. | ✅ |
| 6 | Blade Flurry | 2 | — | Spell | Destroy your weapon and deal its damage to all enemy minions. | ✅ |
| 7 | Defias Ringleader | 2 | 2/2 | Minion | **Combo:** Summon a 2/1 Defias Bandit. | ✅ |
| 8 | Eviscerate | 2 | — | Spell | Deal 2 damage. **Combo:** Deal 4 damage instead. | ✅ |
| 9 | Patient Assassin | 2 | 1/1 | Minion | **Stealth.** Destroy any minion damaged by this minion. | ✅ |
| 10 | Edwin VanCleef | 3 | 2/2 | Minion | **Combo:** Gain +2/+2 for each other card you've played this turn. | ✅ |
| 11 | Headcrack | 3 | — | Spell | Deal 2 damage to the enemy hero. **Combo:** Return this to your hand next turn. | ✅ |
| 12 | Perdition's Blade | 3 | 2/2 | Weapon | **Battlecry:** Deal 1 damage. **Combo:** Deal 2 instead. | ✅ |
| 13 | SI:7 Agent | 3 | 3/3 | Minion | **Combo:** Deal 2 damage. | ✅ |
| 14 | Master of Disguise | 4 | 4/4 | Minion | **Battlecry:** Give a friendly minion **Stealth** until your next turn. | ✅ |
| 15 | Kidnapper | 6 | 5/3 | Minion | **Combo:** Return a minion to its owner's hand. | ✅ |

---

## Shaman

### Shaman — Basic (10)

| # | Card | Cost | Atk/HP | Type | Text | Status |
|---|------|------|--------|------|------|--------|
| 1 | Ancestral Healing | 0 | — | Nature Spell | Restore a minion to full Health and give it **Taunt**. | ✅ |
| 2 | Totemic Might | 0 | — | Spell | Give your Totems +2 Health. | ✅ |
| 3 | Frost Shock | 1 | — | Frost Spell | Deal 1 damage to an enemy character and **Freeze** it. | ✅ |
| 4 | Flametongue Totem | 2 | 0/3 | Totem Minion | Adjacent minions have +2 Attack. | ✅ |
| 5 | Rockbiter Weapon | 2 | — | Nature Spell | Give a friendly character +3 Attack this turn. | ✅ |
| 6 | Windfury | 2 | — | Nature Spell | Give a minion **Windfury**. | ✅ |
| 7 | Hex | 3 | — | Nature Spell | Transform a minion into a 0/1 Frog with **Taunt**. | ✅ |
| 8 | Windspeaker | 4 | 3/3 | Minion | **Battlecry:** Give a friendly minion **Windfury**. | ✅ |
| 9 | Bloodlust | 5 | — | Spell | Give your minions +3 Attack this turn. | ✅ |
| 10 | Fire Elemental | 6 | 6/5 | Elemental Minion | **Battlecry:** Deal 4 damage. | ✅ |

### Shaman — Classic (15)

| # | Card | Cost | Atk/HP | Text | Status |
|---|------|------|--------|------|--------|
| 1 | Dust Devil | 1 | 3/1 | Minion | **Windfury.** **Overload:** (2) | ✅ |
| 2 | Earth Shock | 1 | — | Nature Spell | **Silence** a minion, then deal 1 damage to it. | ✅ |
| 3 | Forked Lightning | 1 | — | Spell | Deal 2 damage to 2 random enemy minions. **Overload:** (2) | ✅ |
| 4 | Lightning Bolt | 1 | — | Nature Spell | Deal 3 damage. **Overload:** (1) | ✅ |
| 5 | Ancestral Spirit | 2 | — | Nature Spell | Give a minion "**Deathrattle:** Resummon this minion." | ✅ |
| 6 | Stormforged Axe | 2 | 2/3 | Weapon | **Overload:** (1) | ✅ |
| 7 | Far Sight | 3 | — | Spell | Draw a card. That card costs (3) less. | ✅ |
| 8 | Feral Spirit | 3 | — | Spell | Summon two 2/3 Spirit Wolves with **Taunt.** **Overload:** (2) | ✅ |
| 9 | Lava Burst | 3 | — | Fire Spell | Deal 5 damage. **Overload:** (2) | ✅ |
| 10 | Lightning Storm | 3 | — | Nature Spell | Deal 2-3 damage to all enemy minions. **Overload:** (2) | ✅ |
| 11 | Mana Tide Totem | 3 | 0/3 | Totem Minion | At the end of your turn, draw a card. | ✅ |
| 12 | Unbound Elemental | 3 | 2/4 | Elemental Minion | Whenever you play a card with **Overload**, gain +1/+1. | ✅ |
| 13 | Doomhammer | 5 | 2/8 | Weapon | **Windfury.** **Overload:** (2) | ✅ |
| 14 | Earth Elemental | 5 | 7/8 | Elemental Minion | **Taunt.** **Overload:** (3) | ✅ |
| 15 | Al'Akir the Windlord | 8 | 3/5 | Minion | **Windfury, Charge, Divine Shield, Taunt** | ✅ |

---

## Warlock

### Warlock — Basic (10)

| # | Card | Cost | Atk/HP | Type | Text | Status |
|---|------|------|--------|------|------|--------|
| 1 | Corruption | 1 | — | Shadow Spell | Choose an enemy minion. At the start of your turn, destroy it. | ✅ |
| 2 | Mortal Coil | 1 | — | Shadow Spell | Deal 1 damage to a minion. If it dies, draw a card. | ✅ |
| 3 | Soulfire | 1 | — | Fire Spell | Deal 4 damage. Discard a random card. | ✅ |
| 4 | Voidwalker | 1 | 1/3 | Demon Minion | **Taunt** | ✅ |
| 5 | Felstalker | 2 | 4/3 | Demon Minion | **Battlecry:** Discard a random card. | ✅ |
| 6 | Drain Life | 3 | — | Shadow Spell | Deal 2 damage. Restore 2 Health to your hero. | ✅ |
| 7 | Hellfire | 4 | — | Fire Spell | Deal 3 damage to ALL characters. | ✅ |
| 8 | Shadow Bolt | 3 | — | Shadow Spell | Deal 4 damage to a minion. | ✅ |
| 9 | Dread Infernal | 6 | 6/6 | Demon Minion | **Battlecry:** Deal 1 damage to ALL other characters. | ✅ |
| 10 | Siegebreaker | 7 | 5/8 | Demon Minion | **Taunt.** Your other Demons have +1 Attack. | ✅ |

### Warlock — Classic (15)

| # | Card | Cost | Atk/HP | Text | Status |
|---|------|------|--------|------|--------|
| 1 | Blood Imp | 1 | 0/1 | Demon Minion | **Stealth.** At the end of your turn, give another random friendly minion +1 Health. | ✅ |
| 2 | Call of the Void | 1 | — | Shadow Spell | Add a random Demon to your hand. | ✅ |
| 3 | Flame Imp | 1 | 3/2 | Demon Minion | **Battlecry:** Deal 3 damage to your hero. | ✅ |
| 4 | Demonfire | 2 | — | Shadow Spell | Deal 2 damage to a minion. If it's a friendly Demon, give it +2/+2 instead. | ✅ |
| 5 | Felguard | 3 | 3/5 | Demon Minion | **Taunt.** **Battlecry:** Destroy one of your Mana Crystals. | ✅ |
| 6 | Sense Demons | 3 | — | Spell | Draw 2 Demons from your deck. | ✅ |
| 7 | Void Terror | 3 | 3/3 | Demon Minion | **Battlecry:** Destroy the minions on both sides of this minion and gain their Attack and Health. | ✅ |
| 8 | Pit Lord | 4 | 5/6 | Demon Minion | **Battlecry:** Deal 5 damage to your hero. | ✅ |
| 9 | Shadowflame | 4 | — | Shadow Spell | Destroy a friendly minion and deal its Attack damage to all enemy minions. | ✅ |
| 10 | Siphon Soul | 6 | — | Shadow Spell | Destroy a minion. Restore 3 Health to your hero. | ✅ |
| 11 | Summoning Portal | 4 | 0/4 | Minion | Your minions cost (2) less, but not less than (1). | ✅ |
| 12 | Bane of Doom | 5 | — | Shadow Spell | Deal 2 damage to a character. If that kills it, summon a random Demon. | ✅ |
| 13 | Lord Jaraxxus | 9 | 3/15 | Demon Minion | **Battlecry:** Destroy your hero and replace it with Lord Jaraxxus. Equip a 3/8 Blood Fury. | ✅ |
| 14 | Twisting Nether | 8 | — | Shadow Spell | Destroy all minions. | ✅ |

---

## Warrior

### Warrior — Basic (10)

| # | Card | Cost | Atk/HP | Type | Text | Status |
|---|------|------|--------|------|------|--------|
| 1 | Execute | 1 | — | Spell | Destroy a damaged enemy minion. | ✅ |
| 2 | Whirlwind | 1 | — | Spell | Deal 1 damage to ALL minions. | ✅ |
| 3 | Cleave | 2 | — | Spell | Deal 2 damage to two random enemy minions. | ✅ |
| 4 | Fiery War Axe | 2 | 3/2 | Weapon | — | ✅ |
| 5 | Heroic Strike | 2 | — | Spell | Give your hero +4 Attack this turn. | ✅ |
| 6 | Shield Block | 3 | — | Spell | Gain 5 Armor. Draw a card. | ✅ |
| 7 | Charge | 3 | — | Spell | Give a friendly minion +2 Attack and **Charge**. | ✅ |
| 8 | Warsong Commander | 3 | 2/3 | Minion | Whenever you summon a minion with 3 or less Attack, give it **Charge**. | ✅ |
| 9 | Kor'kron Elite | 4 | 4/3 | Minion | **Charge** | ✅ |
| 10 | Arcanite Reaper | 5 | 5/2 | Weapon | — | ✅ |

### Warrior — Classic (15)

| # | Card | Cost | Atk/HP | Text | Status |
|---|------|------|--------|------|--------|
| 1 | Inner Rage | 0 | — | Spell | Deal 1 damage to a minion and give it +2 Attack. | ✅ |
| 2 | Shield Slam | 1 | — | Spell | Deal 1 damage to a minion for each Armor you have. | ✅ |
| 3 | Slam | 2 | — | Spell | Deal 2 damage to a minion. If it survives, draw a card. | ✅ |
| 4 | Upgrade! | 1 | — | Spell | If you have a weapon, give it +1/+1. Otherwise equip a 1/3 weapon. | ✅ |
| 5 | Armorsmith | 2 | 1/4 | Minion | Whenever a friendly minion takes damage, gain 1 Armor. | ✅ |
| 6 | Battle Rage | 2 | — | Spell | Draw a card for each damaged friendly character. | ✅ |
| 7 | Commanding Shout | 2 | — | Spell | Your minions can't be reduced below 1 Health this turn. Draw a card. | ✅ |
| 8 | Cruel Taskmaster | 2 | 2/2 | Minion | **Battlecry:** Deal 1 damage to a minion and give it +2 Attack. | ✅ |
| 9 | Rampage | 2 | — | Spell | Give a damaged minion +3/+3. | ✅ |
| 10 | Frothing Berserker | 3 | 2/4 | Minion | Whenever a minion takes damage, gain +1 Attack. | ✅ |
| 11 | Arathi Weaponsmith | 4 | 3/3 | Minion | **Battlecry:** Equip a 2/2 Battle Axe. | ✅ |
| 12 | Mortal Strike | 4 | — | Spell | Deal 4 damage. If you have 12 or less Health, deal 6 instead. | ✅ |
| 13 | Brawl | 5 | — | Spell | Destroy all minions except one. (chosen randomly) | ✅ |
| 14 | Gorehowl | 7 | 7/1 | Weapon | Attacking a minion costs 1 Attack instead of 1 Durability. | ✅ |
| 15 | Grommash Hellscream | 8 | 4/9 | Minion | **Charge.** **Enrage:** +6 Attack. | ✅ |

---

## Statistics

| Category | Total | ✅ fully | ⬜ | 🔧 | ⏸️ |
|----------|-------|---------|-----|------|------|
| Neutral Basic | 43 | 43 | 0 | 0 | 0 |
| Neutral Common | 38 | 38 | 0 | 0 | 0 |
| Neutral Rare | 35 | 35 | 0 | 0 | 0 |
| Neutral Epic | 9 | 9 | 0 | 0 | 0 |
| Neutral Legendary | 24 | 23 | 0 | 0 | 1 |
| Druid | 25 | 25 | 0 | 0 | 0 |
| Hunter | 25 | 25 | 0 | 0 | 0 |
| Mage | 25 | 25 | 0 | 0 | 0 |
| Paladin | 25 | 25 | 0 | 0 | 0 |
| Priest | 27 | 24 | 0 | 0 | 3 |
| Rogue | 25 | 25 | 0 | 0 | 0 |
| Shaman | 25 | 25 | 0 | 0 | 0 |
| Warlock | 24 | 24 | 0 | 0 | 0 |
| Warrior | 25 | 25 | 0 | 0 | 0 |
| **Grand Total** | **375** | **371** | **0** | **0** | **4** |
