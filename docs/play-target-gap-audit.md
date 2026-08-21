# Play-target enumeration gap audit (2026-08-21)

> Trigger: playing **Shadowflame Suffusion (FIR_939)** in the web client offered
> no targeting step, and its 2 damage landed on the enemy hero at random. Pulling
> that thread showed it is not a single-card slip but a **systemic gap in play-time
> target enumeration**. This document is the per-card triage list — what is broken,
> how badly, and in what order to fix it.

## Summary

| | Count |
|---|---|
| Cards the official text targets, but the engine offers no target (**class A**) | **64** |
| ├─ T1 resolution accepts a target (declaration only) — **all fixed** ✅ | 39 |
| └─ T2 resolution ignores the target — **24 fixed, 1 left** (Choose One) | 25 |
| Text looks targeted but is not (**class B**, excluded) | 33 |

The gap spans **Classic through every 2025–2026 expansion**: Classic 24, Emerald Dream 12, Lost City / Un'Goro 10, Timeways 7, Cataclysm 5, Core 3, Violet Hold 3.

## Root cause

`play_targets()` in `src/rl/env.rs` decides "does this effect need a player-chosen
target" with a **`match` whitelist**. Listed variants expand into one legal action
per candidate target; unlisted ones fall through `_ => return Vec::new()` and get a
single untargeted play action.

```rust
let target = match battlecry.0 {
    CardEffect::DealDamage { target, .. } => target,
    // … 44 arms total
    _ => return Vec::new(),          // ← every newer composite effect lands here
};
```

The whitelist has 44 arms; `trigger.rs` has 142 effect variants that accept an
`explicit_target`. Every card wave minted new composite variants
(`DealDamageAndDraw`, `DamageAndDiscoverWarriorWithGift`, …) **without going back
to extend the whitelist**, so new variants default to untargeted. At resolution the
effect has no `explicit` target, so `resolve_deal_damage` and friends fall back to
`select_target(explicit, &candidates, rng)` → a **random pick**; arms written as
`let t = explicit?;` instead **fizzle outright**.

Both degradations are silent: no error, no log line — the player just never gets
the targeting arrow they expect.

## Method (three independent signals)

1. **Static**: parse all 1701 `CardDef`s in `src/cards/*.rs`, extract the
   `battlecry` / `spell_effect` variant, diff against the `play_targets` whitelist
   → 564 cards.
2. **Official text**: pull the English text from `cards/cards.json` (by id, falling
   back to card name — the same convention the engine's differential tests use) and
   look only at the **play-time clause** (spells: the whole text; minions: the
   `Battlecry:` sentence). Keep hits on "a/an minion|character|…" or a bare
   "Deal $N damage." without random/all/each → 97 candidates. Hand-review dropped
   33 Discover/draw/conditional cards → **64 class-A cards**.
3. **Live probe**: for each card, run a mirror game (8 copies + vanilla filler,
   greedy opponent), advance until both sides have minions, play the card and diff
   the board. **All 64 confirmed with zero target options** (`target_id == -1`);
   Control: Fireball (whitelisted) correctly offers targets. (**Only that
   conclusion survives** — the probe's further attempt to classify *how* each
   effect degrades was wrong; see Correction below.)

## Correction (2026-08-21, after this document's first revision)

The first revision split class A into "8 fizzle / 54 random". **That split was
wrong** and is withdrawn.

The instrument was at fault: the probe judged "did the effect do anything" by
diffing the board before and after, and the snapshot recorded only health,
attack and minion counts — **it could not see a Divine Shield popping, a Freeze,
an enchantment, a "set a 1/1 to 1/1" no-op rewrite, or 0 damage from a hero with
0 Attack**. Several perfectly working cards were misread as dead. The clearest
case is Mortal Strike: the probe reported "nothing changed" while the card was in
fact dealing 4 damage to a random Argent Squire and popping its Divine Shield —
the engine already carries a passing `w3_mortal_strike_boosts_at_low_health` test.

Replaced with a **static, checkable** axis: does the resolution arm accept
`explicit_target`? That is what decides the size of the fix, and it does not
depend on snapshot fidelity.

| | Count |
|---|---|
| **T1 resolution already accepts a target** — one `play_target()` arm | 36 |
| **T2 resolution ignores the target** — the target must be plumbed through too | 28 |

(The "fizzle vs random" question becomes moot: both degradations take the choice
away from the player, and the fix is the same either way.)

## Fixed (wave W1, 10 cards)

Every T1 card in Classic and Core, now declaring its target domain, covered by
`tests/play_targeting.rs` — each test asserts both halves of "targetable": the
legal-action list expands per candidate, and a targeted play hits **the chosen**
entity rather than a random one from the same domain.

| Card ID | Name | Effect variant | Official text (play clause) |
|---|---|---|---|
| `DRUID_018` | Savagery | `DealHeroAttackDamage` | Deal damage equal to your hero's Attack to a minion. |
| `HUNTER_021` | Bestial Wrath | `GrantAttackAndImmune` | Give a friendly Beast +2 Attack and Immune this turn. |
| `CLASSIC_009` | Dark Iron Dwarf | `GainStatsThisTurn` | Give a minion +2 Attack this turn. |
| `NEUTRAL_001` | Abusive Sergeant | `GainStatsThisTurn` | Give a minion +2 Attack this turn. |
| `WARLOCK_017` | Siphon Soul | `DestroyAndHeal` | Destroy a minion. Restore #3 Health to your hero. |
| `WARLOCK_021` | Demonfire | `Demonfire` | Deal $2 damage to a minion. If it’s a friendly Demon, give it +2/+2 instead. |
| `WARRIOR_016` | Charge | `GrantCharge` | Give a friendly minion +2 Attack and Charge. |
| `WARRIOR_021` | Mortal Strike | `MortalStrike` | Deal $4 damage. If you have 12 or less Health, deal $6 instead. |
| `CORE_CS2_188` | Abusive Sergeant | `GainStatsThisTurn` | Give a minion +2 Attack this turn. |
| `CORE_TRL_240` | Savage Striker | `DealHeroAttackDamage` | Deal damage to an enemy minion equal to your hero's Attack. |

Declared domains are **the ones the card/resolution already used — no widening**.
Four are narrower than the official text and are logged separately: Savagery and
Siphon Soul (official "a minion", engine enemy-only), Abusive Sergeant / Dark Iron
Dwarf (official "a minion", engine friendly-only), and Mortal Strike (official any
character, engine enemy-only). Domain narrowness is its own class of gap — writing
these tests also turned up **Fireball resolving over `AnyEnemy`** — and deserves a
separate sweep.

## T1 cleared ✅ (wave W2, 26 cards)

W1 landed the 10 Classic + Core cards; W2 finishes the remaining 26 across the
expansions — **T1 is empty**.

| Card ID | Name | Effect variant | Official text (play clause) |
|---|---|---|---|
| `CATA_161` | Gruesome Nightmare | `SetAttackEqualToSource` | Give a minion in your hand or battlefield Attack equal to this minion's Attack |
| `CATA_552` | Ebonscale Scout | `DealDamageEqualSelfAttack` | Deal damage equal to this minion's Attack. (While in hand, play a Dragon to  b |
| `CATA_552t` | Ebonscale Scout | `DealDamageEqualSelfAttack` | Deal damage equal to this minion's Attack. (While in hand, play a Dragon to  b |
| `CATA_564` | Air Support | `GrantMegaWindfuryCantAttackHeroes` | Give a friendly minion Mega-Windfury. |
| `CATA_699` | Dread Leviathan | `StealHealthThreeTimes` | Choose an enemy minion to steal 3  Health from, three times. |
| `EDR_860` | Resplendent Dreamweaver | `DealDamageIfImbuedTwice` | If you've Imbued your Hero Power twice, deal 4 damage to a minion. |
| `EDR_252` | Mark of Ursol | `SetStatsByFriendlyTarget` | Choose a minion. If it's an enemy, set its stats to 1/1. If it's friendly, set |
| `EDR_261` | Amphibian's Spirit | `AmphibianSpiritBuff` | Give a minion +2/+2 and "Deathrattle: Give a friendly minion +2/+2 and this De |
| `EDR_262` | Spirit Bond | `DamageAndSummonWolfIfKilled` | Deal $3 damage to a minion. If it dies, summon a 3/2 Wolf with Rush. |
| `EDR_460` | Wish of the New Moon | `DamageMinionWithMoonLifesteal` | Deal $6 damage to a minion. (Cast 3 spells to gain Lifesteal.) |
| `EDR_523` | Web of Deception | `ReturnFriendlyMinionSummonSpider` | Return a friendly minion to your hand to summon a 4/4 Spider with Stealth. |
| `EDR_531` | Siphoning Growth | `DestroyFriendlyMinionGainArmor` | Destroy a friendly minion to gain 8 Armor. |
| `FIR_908` | Charred Chameleon | `GiveMinionStatsRushIfHeroPowerUsed` | If you've used your Hero Power this turn, give a friendly minion +1/+2 and Rus |
| `FIR_918` | Light of the New Moon | `BuffMinionReturnIfSpellsCast` | Give a minion +3/+3. (Cast 3 spells to return this to your hand when played.) |
| `FIR_939` | Shadowflame Suffusion | `DamageAndDiscoverWarriorWithGift` | Deal $2 damage. Discover a Warrior minion with a Dark Gift. |
| `FIR_954` | Conflagrate | `DamageMinionOwnerDraws` | Deal $5 damage to a minion. Its owner draws a card. |
| `JAIL_998` | Defias Smuggler | `GainStatsAndGrantRush` | Give a friendly minion +2 Attack and Rush. |
| `TLC_230` | TREEEES!!! | `SummonTreantsAttackMinion` | Choose a minion. Summon four 2/2 Treants that attack it. |
| `TLC_252` | Dissolving Ooze | `DestroyFriendlyMinionAddBones` | Destroy a friendly minion. |
| `TLC_441` | Ready the Fleet | `GiveBuffSameType` | Give +1/+2 to a friendly minion and your other minions that share a type with |
| `TLC_606` | Latorvian Armorer | `DealDamageGainArmorIfKilled` | Deal 2 damage to an enemy minion. |
| `TLC_620` | Fortify | `GainArmorDealDamageEqual` | Gain 3 Armor. Deal damage equal to your Armor to an enemy minion. |
| `TLC_823` | Cower in Fear | `DealDamageSetNextBeastDiscount` | Deal $3 damage to a minion. The next Beast you play this turn costs (2) less. |
| `TLC_901` | Fumigate | `DealDamageSameType` | Deal $3 damage to a minion and all others of the same minion type. |
| `TLC_987` | Questing Assistant | `DealDamageIfQuestPlayed` | If you played a Quest this game, deal 3 damage to an enemy minion. |
| `DINO_419` | Herbivore Assistant | `GainStatsAndGrantRush` | Give a friendly Beast +2/+2 and Rush. |

Coverage: one sweep test asserting **every** one of these offers targets (and that
the offers agree with `rules::validate`), plus a "hits the chosen one" test per
domain (Conflagrate, Siphoning Growth, Shadowflame Suffusion, Herbivore Assistant).

Two declarations are deliberately narrower than the resolution's candidate set
(narrower is safe; wider would fizzle at resolution): Air Support and Gruesome
Nightmare scan `Zone::Play(owner)`, which **includes the hero**, while the offer
is the minion subset.

## T2 cleared ✅ (wave W3, 27 cards, 1 left pending Choose One)

T2 = cards whose resolution arm **never received** `explicit_target`: declaring a
domain would not have helped, because the effect picked its own victim. W3
threads the chosen target into every one of those resolution paths.

**One classification correction first**: Icicle, Shiv and Slam landed in T2
because of a **bug in the classifier** — it matched `CardEffect::X` by
indentation and hit the Spell Damage adjustment table (`apply_spell_power`)
instead of the real resolver. Their resolutions always honoured the target;
they only needed the declaration.

| Card ID | Name | Effect variant |
|---|---|---|
| `MAGE_022` | Icicle | `FreezeOrDamage` |
| `ROGUE_014` | Shiv | `DealDamageAndDraw` |
| `WARRIOR_011` | Slam | `DealDamageAndDraw` |

The remaining 24 are genuine T2, in three shapes:

- **Shape A (5)**: the arm already calls `resolve_deal_damage` /
  `resolve_restore_health` but hardcodes `None` — pass `explicit_target`.
- **Shape B (7)**: the arm collects candidates and rolls its own random pick
  (`pick_random` / `rng.next_usize`) — replaced with
  `select_target(explicit_target, ...)`, which keeps the random behaviour when
  no target is supplied.
- **Shape C (12)**: the resolution helper had no target parameter at all — added
  `explicit: Option<Entity>` and threaded it through 11 helpers.

| Card ID | Name | Effect variant |
|---|---|---|
| `MAGE_016` | Cone of Cold | `FreezeAdjacent` |
| `CLASSIC_FM` | Faceless Manipulator | `CopyMinionStats` |
| `PALADIN_017` | Holy Wrath | `DrawAndDamageByCost` |
| `PRIEST_011` | Cabal Shadow Priest | `TakeControlAttackLE` |
| `PRIEST_021` | Natalie Seline | `DestroyAndGainHealth` |
| `PRIEST_022` | Shadow Madness | `TakeControlUntilEndOfTurn` |
| `PRIEST_023` | Mind Control | `TakeControl` |
| `ROGUE_019` | Shadowstep | `ReturnFriendlyToHandAndReduceCost` |
| `ROGUE_020` | Betrayal | `AdjacentDamage` |
| `ROGUE_024` | Master of Disguise | `GrantStealth` |
| `SHAMAN_003` | Rockbiter Weapon | `GainHeroAttack` |
| `WARLOCK_018` | Shadowflame | `DestroyAndAOE` |
| `WARLOCK_024` | Corruption | `Corrupt` |
| `CORE_EX1_198` | Natalie Seline | `DestroyAndGainHealth` |
| `JAIL_101` | Violet Punisher | `VioletPunisher` |
| `JAIL_395` | Sewer Swimmer | `SewerSwimmer` |
| `TLC_221` | Sizzling Swarm | `DealDamageSummonCinders` |
| `TIME_043` | PMM Infinitizer | `SetStatsAndCantAttackHeroesThisTurn` |
| `TIME_427` | Cleansing Lightspawn | `DealDamageEnemyMinionEqualToSourceHealth` |
| `TIME_431` | Amber Priestess | `RestoreHealthEqualToSourceHealth` |
| `TIME_442` | Timeway Warden | `ImprisonEnemyMinion` |
| `TIME_614` | Liferender | `DealDamageEnemyMinionIfHeroHealthChanged` |
| `TIME_858` | Temporal Construct | `DealDamageAndDrawExcess` |
| `TIME_435` | Eternus | `TakeControlEnemyMinionHealthLE` |

### Two holes closed along the way

**1. The enumeration layer had the same silent catch-all.**
`rl::env::candidates_for_target` ended in `_ => Vec::new()`: declare a domain it
does not know and you get **an empty candidate list**, not an error — the card
quietly reverts to an untargeted play. It enumerated 15 of the 31
`EffectTarget` variants. It is now an exhaustive match (no `_` arm) with the 7
missing single-target domains filled in (`EnemyMinionAttackLE`,
`AnyMinionAttackLE`, `EnemyMinionWithRace`, `OtherFriendlyMinion` and the three
Damaged\* variants); the AoE scopes are listed explicitly as "returns nothing".
**This affects the already-merged W1/W2**: cards on filtered domains (Shadow
Word: Death, Hungry Crab) could never produce candidates even with their
variant whitelisted.

**2. Two target domains did not exist and were added to `EffectTarget`:**
- `FriendlyMinionWithDeathrattle` — Sewer Swimmer triggers *a friendly minion's
  Deathrattle*, so only minions that have one should light up.
- `EnemyMinionHealthLESource` — Eternus caps on **its own Health**, a dynamic
  bound, so `candidates_for_target` now takes the `source` entity too.

### One card left

`EDR_813` Morbid Swarm is a **Choose One**: only the second mode (spend 2
Corpses, deal 4 damage to a minion) takes a target. `play_targets` reads the
battlecry component, while Choose One and Combo travel a different path, so this
needs a two-stage "pick the mode, then the target" action design — its own wave.

## Class B — the 33 excluded cards

Their text mentions "a minion / a Beast", but as part of a **Discover, a draw, or a
condition** — no play-time target is required and the current behaviour is correct.

| Card ID | Name | Effect variant | Official text (play clause) |
|---|---|---|---|
| `CORE_BT_321` | Netherwalker | `AddRandomCardToHand` | Discover a Demon. |
| `CORE_EDR_004` | Raptor Herald | `AddRandomCardToHand` | Discover a Beast with a Dark Gift. |
| `CORE_KAR_061` | The Curator | `DrawBeastDragonMurloc` | Draw a Beast, Dragon, and Murloc. |
| `CORE_KAR_062` | Netherspite Historian | `AddRandomCardToHand` | If you're holding a Dragon, Discover a Dragon. |
| `CORE_LOE_039` | Gorillabot A-3 | `AddRandomCardToHand` | If you control another Mech, Discover a Mech. |
| `CORE_RLK_116` | Necrotic Mortician | `AddRandomCardToHand` | If a friendly Undead died after your last turn, Discover an Unholy Rune card. |
| `CORE_CATA_006` | Ulfar | `GrantDeathrattleSummonOwnCost` | Give your other minions "Deathrattle: Summon a minion with this minion's Cost. |
| `CORE_TRL_111` | Headhunter's Hatchet | `BuffWeaponDurabilityIfBeast` | If you control a Beast, gain +1 Durability. |
| `CATA_111` | Darkscale Broodmother | `RefreshManaIfHoldingDragon` | If you're holding a Dragon, refresh 2 Mana Crystals. |
| `CATA_553` | Ebyssian | `SetDragonsHaveRush` | Your Dragons have Rush this game. (While in hand, play a Dragon to become a 12 |
| `CATA_553t` | Ebyssian | `SetDragonsHaveRush` | Your Dragons have Rush this game. (While in hand, play a Dragon to become a 12 |
| `MEND_041` | Wizened Wildspeaker | `RefreshManaIfNoMinionPlayedLastTurn` | If you didn't play a minion last turn, refresh 3 Mana Crystals. |
| `EDR_226` | Exotic Houndmaster | `DrawBeastAndImbue` | Draw a Beast. |
| `EDR_456` | Darkrider | `DiscoverDragonWithDarkGift` | If you're holding a Dragon, Discover a Dragon with a Dark Gift. |
| `EDR_856` | Nightmare Lord Xavius | `DiscoverDeckMinionWithDarkGift` | Discover a minion from your deck. |
| `EDR_490` | Sleep Paralysis | `SummonMultipleMinions` | Choose One - Summon two 3/6 Demons with Taunt that can't attack; or Destroy an |
| `EDR_843` | Reforestation | `DrawCardByType` | Choose One - Draw a spell; or Draw a minion. (Hold this for 3 turns to do both |
| `EDR_455` | Succumb to Madness | `ResurrectRandomFallenDragon` | Discover a friendly Dragon that died this game. Resummon it. |
| `EDR_457` | Brood Keeper | `EquipSwordIfHoldingDragon` | If you're holding a Dragon, equip a 2/2 Sword. |
| `FIR_900` | Cremate | `DiscoverWithDarkGiftCostReduction` | Discover a minion with a Dark Gift. It costs (2) less. |
| `FIR_901` | Frostburn Matriarch | `SummonBroodlingsIfHoldingGift` | If you're holding a minion with a Dark Gift, summon two 4/4 Dragons with Taunt |
| `FIR_922` | Cindersword | `GainWeaponAttackIfHoldingGift` | If you're holding a minion with a Dark Gift, gain +3 Attack. |
| `FIR_924` | Shadowflame Stalker | `DiscoverDemonWithDarkGiftCopy` | Discover a Demon with a Dark Gift. |
| `FIR_941` | Searing Reflection | `DrawMinionSummonDivineShieldCopy` | Draw a minion. Summon an 8/8 copy of it with Divine Shield. |
| `FIR_956` | Dragon Turtle | `GainHeroAttackArmorIfHoldingGift` | If you're holding a minion with a Dark Gift, give your hero +3 Attack this tur |
| `TLC_231` | Story of Barnabus | `DrawMinionBuffArmorIfAttackGE` | Draw a minion. If it has 5 or more Attack, give it +5 Health and gain 5 Armor. |
| `TLC_434` | Paleomancy | `DiscoverPool` | Discover an Undead. Spend 5 Corpses to keep all 3 instead. |
| `TLC_442` | Submerged Map | `DiscoverPool` | Discover a Murloc. If you play it this turn, also pick one of the others. |
| `TLC_464` | Mountain Map | `DiscoverPool` | Discover a minion with a type you haven't played. If you play it this turn, al |
| `TLC_888` | Cloud Serpent | `CopyRandomHandElementalOrDragon` | Get a copy of another Elemental or Dragon in your hand. |
| `TLC_110` | City Chief Esho | `EshoDeckCheckBuffEverywhere` | If every minion in your deck shares a minion type, give your other minions +2/ |
| `TIME_037` | Disciple of the Dove | `DrawMinionAndBuffHandMinionsHealth` | Draw a minion. |
| `TIME_062` | Chronicle Keeper | `GainTauntAndDivineShieldIfHoldingDragon` | If you're holding a Dragon, gain Taunt and Divine Shield. |

## Special cases (they will bite during the fix)

- **`EDR_813` Morbid Swarm** — Choose One; only the second mode (spend 2 Corpses,
  deal 4 damage to a minion) needs a target. `play_targets` only reads the battlecry
  component, so Choose One / Combo need a separate two-stage action design.
- **`CATA_161` Gruesome Nightmare** — the official target domain is "a minion **in
  your hand** or battlefield". Every existing `EffectTarget` is battlefield-only, so
  this needs a new target kind.
- **`CATA_552` Ebonscale Scout / `CATA_552t`** — two forms sharing one effect; fix
  both together.
- **`FIR_939` Shadowflame Suffusion** — the official text is a bare "Deal 2 damage",
  which by Hearthstone convention (Fireball) targets **any character**, while the
  resolution hardcodes `EffectTarget::AnyEnemy`. Fixing this card means widening the
  resolution domain, not just extending the whitelist.
- **Ledger `docs/finished/fidelity-debt.md:933`** claims FIR_939's "damage IS
  faithful" — untrue; correct it with the fix.
- ~~**F-A13 (an independent bug found while clearing T1)**~~ — **fixed
  (2026-08-21)**: the engine enacted "destroy a minion" as lethal damage, so
  **Divine Shield ate a destroy** (Assassinate on an Argent Squire popped the
  shield and left it alive). Now a `Destroyed` marker plus one shared
  `engine::rules::destroy_minion()` covers all 20 destroy sites, off the damage
  pipeline entirely. The `play_targeting` boards still carry no Divine Shield
  minions — that keeps the "did it hit the chosen one" signal clean, which is
  independent of F-A13.

## Recommended fix order

1. ~~Dead cards first~~ — that classification is withdrawn (see Correction).
   Instead **clear T1**: resolution already accepts the target, so a declaration
   plus tests is enough. **T1 is now empty** (W1: 10 Classic + Core, W2: 26 across the
   expansions).
2. ~~**Then chew through T2**~~ — **done (W3)**: 24 cards now thread the chosen
   target through their resolution; only Choose One cards like `EDR_813` remain,
   pending a two-stage action design.
3. ~~**Structural fix so it cannot drift again**~~ — **done (2026-08-21)**: the
   declaration moved out of `play_targets`'s `match` and onto
   `CardEffect::play_target()` (`src/core/play_target.rs`, an **exhaustive match over
   all 866 variants with no `_` arm**). A new effect variant now fails to compile
   until it declares its targeting (verified: adding a probe variant yields
   `error[E0004]: non-exhaustive patterns`). Purely structural, zero behaviour
   change: all 1064 existing tests green, batch throughput 2089–2143 games/s versus
   2089–2138 before — the same noise band.
4. ~~**Guard test**~~ — **done**: `audit_gap_cards_are_still_untargeted` encodes this
   document's 64 class-A ids as an executable ledger. The moment one of them starts
   declaring a target the assertion fails, **forcing the ledger entry to be removed
   in the same commit as the fix** — precisely the code/ledger drift that produced
   this gap in the first place.

## Reproducing

```bash
# Per-card check: does any play action carry a real target_id?
python - <<'EOF'
import orange_stone
e = orange_stone.GameEnv(seed=1, deck=["MAGE_001"]*30, bot="none"); e.reset(1)
for _ in range(6):
    plays = [a for a in e.structured_legal_actions() if a.kind == "play"]
    if plays: print([p.target_id for p in plays]); break
    e.step(next(a.index for a in e.structured_legal_actions() if a.kind == "end_turn"))
EOF
# Fireball (MAGE_001, whitelisted) → real entity ids;
# Shadowflame Suffusion (FIR_939) → all -1
```
