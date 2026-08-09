//! M3-W3 cards (exp_tmw_w3) — Across the Timeways sub-roadmap W3, the
//! closing wave: the full **The End of Time** miniset (38 END_ cards,
//! 2026-01), plus the two tokens they produce (END_009t Treant, END_017t
//! Tick and Tock). The set is reuse-heavy — most machinery landed in W1
//! (Rewind), W2a/W2b (the Across-the-Timeways cards) and M1 (imbue,
//! dark gifts, quests).
//!
//! Implementation decisions (per the M3-W3 spec, verified against
//! `cards/data/TIME_TRAVEL.json` and `/tmp/hs_full.json` 2026-08-09; the
//! fidelity rows are fidelity-debt.md §22, en + zh):
//! - **Imbue trio** (END_000 / END_001 / END_003): the existing
//!   `ImbueHeroPower` machinery. END_000p (Rogue, cost 1) and END_003p
//!   (Death Knight, cost 0) get NO CardDefs — `resolve_imbue` extends to
//!   the two new classes: Rogue replaces with a cost-1 power that gets a
//!   random other-class minion costing the imbue level less and Rewinds
//!   (the rewind resolves inside the Rogue arm — hero powers never hook
//!   the rewind play path, §22); Death Knight replaces with a cost-0
//!   passive whose hero-pinned CardPlayed-Undead trigger (re-attached on
//!   every imbue, so the buff scales with the level) buffs the first
//!   Undead played each turn.
//! - **INFINITY** (END_012 Hand of Infinity, END_018 Acolyte of
//!   Infinity, END_024 Flames of Infinity): the W2b precedent — a
//!   32-bit engine has no unbounded value, so the effects use
//!   `INFINITY_ATTACK_CAP` (100, `exp_tmw_w2b`, shared). END_012 sets an
//!   UntilEndOfTurn attack-delta enchantment on the weapon; END_018
//!   records the affected hand card in `Player::hand_card_infinity`
//!   (cost layer reports the cap; the deathrattle restores); END_024's
//!   secret deals the cap as damage (a kill in all but the most extreme
//!   boards, §22).
//! - **END_012's "Can't attack heroes"** rides the weapon: the attack
//!   validation rejects a hero swing at the enemy HERO while END_012 is
//!   equipped (the hero may still attack enemy minions).
//! - **END_017 quest** (Battle at the End Time): the "Fill your hand,
//!   then empty it" SEQUENCE is a new `QuestCondition::FillThenEmptyHand`
//!   with two progress markers (filled once / emptied); the reward
//!   summons END_017t Tick and Tock (5-cost 8/8 Dragon — battlecry draws
//!   until the hand is full, deathrattle empties the opponent's hand).
//! - **END_015 Triennium Rex** registers the kindred table (Beast
//!   kindred, OnPlay = the deathrattle effect) and carries the same
//!   effect as its deathrattle.
//! - **END_010 Twilight Timereaver** is a choose-one MINION: option 0
//!   rides the battlecry slot, option 1 the choose-one slot; "all other
//!   minions" = every minion on both boards except the source.
//! - **END_036 Morchie**: the "Rewinds keep BOTH outcomes" aura is a
//!   documented simplification (§22) — a unit-marker aura
//!   (`AuraEffect::RewindKeepsBothOutcomes`, `AuraTarget::AllFriendlyMinions`)
//!   that makes `engine::rewind` resolve each replayed random-outcome
//!   effect twice (a closed list of the random-outcome variants, §22).
//!   The battlecry (discover a Rewind card) reuses the existing
//!   `AddRandomRewindCardToHand` (the real Discover is a D2 random pick).
//! - **END_006 Chronikar**: the "this turn, next turn, and the turn
//!   after" buff is a `chronikar_ticks` counter — the battlecry applies
//!   the current-turn +3 and arms 2; each of the next two turn starts
//!   decrements and re-applies (the exact official "3 turns" is
//!   approximated as 3 buffs, §22).
//! - **END_037 Endtime Murozond**: the skip is the `skip_next_turn`
//!   player flag — the next TurnStarted clears it and immediately runs
//!   the normal turn-end sequence.
//! - **Stats** follow the official JSON data (`/tmp/hs_full.json`
//!   2026-08-09); they match the generated baselines (the differential
//!   tripwire compares every handwritten card with a baseline).
//! - **END_022 Time-Twisted Seer** ("Spell Damage +2 while damaged"):
//!   the CardDef carries `spell_damage: 2`; `world::total_spell_damage`
//!   skips the bonus while the minion is undamaged (id-keyed, so the
//!   baseline compare sees 2 — the differential rebalance row marks
//!   `spell_damage` as excluded).
//! - **END_025 Eternal Firebolt** (Lifesteal, return-to-hand if the
//!   target dies): the return is the `eternal_flame_target` player
//!   record — the target is recorded at cast; the owner's turn end adds
//!   a fresh END_025 copy if the target died (§22).
//! - **END_030 Haywire Hornswog** ("costs (1) less for each Mana Crystal
//!   you've Overloaded this game"): a game-long `overload_total` player
//!   counter bumped at the overload lock site; the cost layer reads it.
//! - **END_004 / END_033** cost reductions read existing state
//!   (died-this-turn minions / holding a Dragon, TIME_852 convention).
//! - **END_005 Bygone Echoes** (summon a random 4-Cost minion; spend 4
//!   Corpses for another; Outcast for a third): one effect resolution —
//!   the corpse spend and the Outcast check ride the same call.
//! - **D2 random picks**: every pick that would be a player choice in
//!   the real game (END_013's discover, END_014's buff target, END_018's
//!   hand card, END_020's summon, END_027's dragon, END_034's three
//!   destroys, END_037's dragons) is the established D2 random.
//! - **END_029t / END_026t tokens are NOT implemented** (verified
//!   2026-08-09): the Voodoo Totem's Shade token and the Fragment token
//!   have no effects in this wave's shapes.
//!
//! The 38 cards and 2 tokens register in
//! `sets::HANDWRITTEN_EXPANSION_CARDS` (the M3-W3 block).

use crate::cards::def::CardDef;
use crate::core::component::AuraEffect;
use crate::core::component::AuraTarget;
use crate::core::component::CardType;
use crate::core::component::Race;
use crate::core::component::SecretTrigger;
use crate::core::effect::CardEffect;
use crate::core::effect::EffectTarget;
use crate::core::effect::RandomPool;

/// END_000 Eventuality — 2-cost (SPELL). "Deal 2 damage. Imbue your Hero
/// Power." (the unqualified damage text follows the Elven Archer
/// AnyCharacter convention; the imbue rides the existing machinery).
pub const EVENTUALITY: CardDef = CardDef {
    id: "END_000",
    name: "Eventuality",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: Some(CardEffect::DealDamageAndImbue {
        amount: 2,
        target: EffectTarget::AnyCharacter,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_001 Jagged Edge of Time — 3-cost 3/2 (WEAPON). "Battlecry: Imbue
/// your Hero Power."
pub const JAGGED_EDGE_OF_TIME: CardDef = CardDef {
    id: "END_001",
    name: "Jagged Edge of Time",
    card_type: CardType::Weapon,
    cost: 3,
    attack: 3,
    health: 0,
    durability: 2,
    battlecry: Some(CardEffect::ImbueHeroPower),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_002 Wicked Blightspawn — 4-cost 4/3 (MINION, Undead). "Reborn.
/// Deathrattle: Equip a 1/2 Dagger. If you already have a weapon
/// equipped, give it +2 Attack instead." (Reborn is applied by
/// `apply_card_keywords`.)
pub const WICKED_BLIGHTSPAWN: CardDef = CardDef {
    id: "END_002",
    name: "Wicked Blightspawn",
    card_type: CardType::Minion,
    cost: 4,
    attack: 4,
    health: 3,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::EquipDaggerOrBuffWeapon),
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Undead),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_003 Finality — 3-cost (SPELL). "Draw an Undead. Imbue your Hero
/// Power twice."
pub const FINALITY: CardDef = CardDef {
    id: "END_003",
    name: "Finality",
    card_type: CardType::Spell,
    cost: 3,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: Some(CardEffect::DrawUndeadAndImbueTwice),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_004 Remnant of Rage — 7-cost 5/4 (MINION, Demon). "Costs (1) less
/// for each minion that died this turn. Battlecry: Draw 2 cards." (The
/// cost reduction reads `died_this_turn` in the cost layer.)
pub const REMNANT_OF_RAGE: CardDef = CardDef {
    id: "END_004",
    name: "Remnant of Rage",
    card_type: CardType::Minion,
    cost: 7,
    attack: 5,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::DrawCard { count: 2 }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Demon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_005 Bygone Echoes — 5-cost (SPELL, Shadow). "Summon a random
/// 4-Cost minion. Spend 4 Corpses to summon another. Outcast: And
/// another."
pub const BYGONE_ECHOES: CardDef = CardDef {
    id: "END_005",
    name: "Bygone Echoes",
    card_type: CardType::Spell,
    cost: 5,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: Some(CardEffect::BygoneEchoesSummon),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_006 Chronikar — 5-cost 3/5 (MINION, Dragon). "Battlecry: Give your
/// hero +3 Attack this turn, next turn, and the turn after." (The
/// battlecry applies the current buff and arms the two-turn
/// `chronikar_ticks` counter; the start-of-turn effect re-applies.)
pub const CHRONIKAR: CardDef = CardDef {
    id: "END_006",
    name: "Chronikar",
    card_type: CardType::Minion,
    cost: 5,
    attack: 3,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::ChronikarHeroAttackBuff),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: Some(CardEffect::ChronikarRebuff),
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_007 Press the Advantage — 2-cost (SPELL). "Deal 1 damage. Give
/// your hero +1 Attack this turn. Draw 1 card. Gain 1 Armor."
pub const PRESS_THE_ADVANTAGE: CardDef = CardDef {
    id: "END_007",
    name: "Press the Advantage",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: Some(CardEffect::PressTheAdvantage),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_008 Enduring Roach — 3-cost 3/4 (MINION, Beast). "After you use
/// your Hero Power, refresh 2 Mana Crystals." (The HeroPowerUsed trigger
/// is attached by `apply_card_keywords`.)
pub const ENDURING_ROACH: CardDef = CardDef {
    id: "END_008",
    name: "Enduring Roach",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 4,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Beast),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_009 Splintered Reality — 4-cost (SPELL, Nature). "Summon two 2/2
/// Treants. They gain +1/+1 for each friendly Treant that died this
/// game." (The per-player `treants_died_total` counter.)
pub const SPLINTERED_REALITY: CardDef = CardDef {
    id: "END_009",
    name: "Splintered Reality",
    card_type: CardType::Spell,
    cost: 4,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: Some(CardEffect::SummonTwoTreantsScaling),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_010 Twilight Timereaver — 6-cost 5/5 (MINION, Dragon). "Choose One
/// — Set the Attack of all other minions to 1; or Health to 1." (Option
/// 0 rides the battlecry slot, option 1 the choose-one slot.)
pub const TWILIGHT_TIMEREAVER: CardDef = CardDef {
    id: "END_010",
    name: "Twilight Timereaver",
    card_type: CardType::Minion,
    cost: 6,
    attack: 5,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::SetAllOtherMinionsAttack { attack: 1 }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: Some(CardEffect::SetAllOtherMinionsHealth { health: 1 }),
    combo_effect: None,
    attack_equals_health: false,
};

/// END_011 Acceleration Aura — 2-cost (SPELL, Holy). "At the start of
/// your turn, gain a temporary Mana Crystal. Lasts 3 turns." (The
/// `acceleration_aura_ticks` counter arms 3; the ManaRefill step grants
/// one temporary mana per owner turn start while it is positive.)
pub const ACCELERATION_AURA: CardDef = CardDef {
    id: "END_011",
    name: "Acceleration Aura",
    card_type: CardType::Spell,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: Some(CardEffect::ArmAccelerationAura),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_012 Hand of Infinity — 3-cost 4/2 (WEAPON). "Can't attack heroes.
/// Battlecry: Set this weapon's Attack to INFINITY this turn!" (The
/// "can't attack heroes" restriction is enforced at attack validation
/// while the weapon is equipped; the INFINITY attack is an
/// UntilEndOfTurn delta enchantment on the weapon, §22.)
pub const HAND_OF_INFINITY: CardDef = CardDef {
    id: "END_012",
    name: "Hand of Infinity",
    card_type: CardType::Weapon,
    cost: 3,
    attack: 4,
    health: 0,
    durability: 2,
    battlecry: Some(CardEffect::SetWeaponAttackInfinityThisTurn),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_013 Brutish Endmaw — 3-cost 3/3 (MINION, Beast). "Battlecry:
/// Discover a 1-Cost minion with a Dark Gift." (The real Discover is a
/// D2 random pick from the 1-cost minion pool, the established
/// simplification.)
pub const BRUTISH_ENDMAW: CardDef = CardDef {
    id: "END_013",
    name: "Brutish Endmaw",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::DiscoverWithDarkGift {
        pool: RandomPool::OneCostMinion,
    }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Beast),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_014 Synchronized Spark — 4-cost (SPELL). "Deal 3 damage to an
/// enemy. If it dies, give a random friendly minion +3/+3." (The
/// predicted-death convention of DealDamageGainArmorIfKilled.)
pub const SYNCHRONIZED_SPARK: CardDef = CardDef {
    id: "END_014",
    name: "Synchronized Spark",
    card_type: CardType::Spell,
    cost: 4,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: Some(CardEffect::DamageAndBuffFriendlyIfKilled {
        amount: 3,
        attack: 3,
        health: 3,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_015 Triennium Rex — 5-cost 5/5 (MINION, Beast). "Kindred and
/// Deathrattle: Get a random Deathrattle minion. It costs (2) less."
/// (Registers the kindred table — Beast kindred, OnPlay effect — and
/// carries the same effect as its deathrattle.)
pub const TRIENNIUM_REX: CardDef = CardDef {
    id: "END_015",
    name: "Triennium Rex",
    card_type: CardType::Minion,
    cost: 5,
    attack: 5,
    health: 5,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::AddRandomDeathrattleMinionCostsLess),
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Beast),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_016 Chronoclaws — 4-cost 4/3 (WEAPON). "After your hero attacks,
/// discard your highest Cost card." (The HeroAttacked trigger is
/// attached to the weapon by `apply_card_keywords`.)
pub const CHRONOCLAWS: CardDef = CardDef {
    id: "END_016",
    name: "Chronoclaws",
    card_type: CardType::Weapon,
    cost: 4,
    attack: 4,
    health: 0,
    durability: 3,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_017 Battle at the End Time — 1-cost (SPELL). "Quest: Fill your
/// hand, then empty it. Reward: Tick and Tock." (The condition is the
/// FillThenEmptyHand SEQUENCE — quest registry entry; quest cards record
/// no rewind history and carry no play effect.)
pub const BATTLE_AT_THE_END_TIME: CardDef = CardDef {
    id: "END_017",
    name: "Battle at the End Time",
    card_type: CardType::Spell,
    cost: 1,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_018 Acolyte of Infinity — 3-cost 5/5 (MINION, Dragon). "Battlecry:
/// Set the Cost of a random card in your hand to INFINITY! Deathrattle:
/// Change it back." (The affected card is recorded in
/// `Player::hand_card_infinity`; the cost layer reports the INFINITY
/// cap, §22.)
pub const ACOLYTE_OF_INFINITY: CardDef = CardDef {
    id: "END_018",
    name: "Acolyte of Infinity",
    card_type: CardType::Minion,
    cost: 3,
    attack: 5,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::SetRandomHandCardCostInfinity),
    deathrattle: Some(CardEffect::RestoreInfinityHandCardCost),
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_019 Endtime Survivor — 5-cost 5/6 (MINION). "Taunt. Battlecry: If
/// your hero took damage this turn, gain +3/+3."
pub const ENDTIME_SURVIVOR: CardDef = CardDef {
    id: "END_019",
    name: "Endtime Survivor",
    card_type: CardType::Minion,
    cost: 5,
    attack: 5,
    health: 6,
    durability: 0,
    battlecry: Some(CardEffect::GainStatsIfHeroDamagedThisTurn {
        attack: 3,
        health: 3,
    }),
    deathrattle: None,
    taunt: true,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_020 Eternal Toil — 1-cost (SPELL, Shadow). "Deal 1 damage to a
/// minion. If it survives, draw a card. If it dies, summon a random
/// 1-Cost minion." (The predicted-death convention.)
pub const ETERNAL_TOIL: CardDef = CardDef {
    id: "END_020",
    name: "Eternal Toil",
    card_type: CardType::Spell,
    cost: 1,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: Some(CardEffect::DamageMinionDrawIfSurvivesSummonIfDies { amount: 1 }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_021 Dimensional Weaponsmith — 3-cost 2/5 (MINION, Dragon).
/// "Battlecry: Give all minions and weapons in your hand +2 Attack."
pub const DIMENSIONAL_WEAPONSMITH: CardDef = CardDef {
    id: "END_021",
    name: "Dimensional Weaponsmith",
    card_type: CardType::Minion,
    cost: 3,
    attack: 2,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::BuffHandMinionsAndWeapons { attack: 2 }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_022 Time-Twisted Seer — 1-cost 1/3 (MINION, Dragon). "Has Spell
/// Damage +2 while damaged." (The CardDef carries `spell_damage: 2`;
/// `world::total_spell_damage` skips the bonus while the minion is
/// undamaged — id-keyed; the differential rebalance row marks
/// `spell_damage` excluded so the tripwire still compares it.)
pub const TIME_TWISTED_SEER: CardDef = CardDef {
    id: "END_022",
    name: "Time-Twisted Seer",
    card_type: CardType::Minion,
    cost: 1,
    attack: 1,
    health: 3,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 2,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_023 Bitter End — 5-cost (SPELL, Frost). "Freeze a minion and its
/// neighbors. Destroy any that are damaged." (The neighbors are the
/// adjacent board slots; the frozen-but-undamaged middle survives, per
/// the official "destroy any that are damaged".)
pub const BITTER_END: CardDef = CardDef {
    id: "END_023",
    name: "Bitter End",
    card_type: CardType::Spell,
    cost: 5,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: Some(CardEffect::FreezeMinionAndNeighborsDestroyDamaged),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_024 Flames of Infinity — 3-cost (SPELL, Fire). "Secret: When your
/// enemy's turn ends, deal INFINITE damage to their highest Health
/// minion." (The secret trigger is the new WhenEnemyTurnEnds; the
/// damage is the INFINITY cap, §22.)
pub const FLAMES_OF_INFINITY: CardDef = CardDef {
    id: "END_024",
    name: "Flames of Infinity",
    card_type: CardType::Spell,
    cost: 3,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: Some(CardEffect::InfiniteDamageToHighestHealthEnemyMinion),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: Some(SecretTrigger::WhenEnemyTurnEnds),
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_025 Eternal Firebolt — 3-cost (SPELL, Fire). "Lifesteal. Deal 3
/// damage to a minion. If it dies, return this to your hand at the end
/// of your turn." (Lifesteal is applied by `apply_card_keywords`; the
/// return is the `eternal_flame_target` record — the owner's turn end
/// adds a fresh copy when the target died, §22.)
pub const ETERNAL_FIREBOLT: CardDef = CardDef {
    id: "END_025",
    name: "Eternal Firebolt",
    card_type: CardType::Spell,
    cost: 3,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: Some(CardEffect::DamageMinionEternalFirebolt { amount: 3 }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_026 Fragment of Nothing — 4-cost 3/6 (MINION, Elemental). "After
/// you cast a spell on a minion, draw a card." (The
/// FriendlySpellCastOnMinion trigger is attached by
/// `apply_card_keywords` — `spell_trigger` hardcodes FriendlySpellCast,
/// so the registration is the established per-card trigger path.)
pub const FRAGMENT_OF_NOTHING: CardDef = CardDef {
    id: "END_026",
    name: "Fragment of Nothing",
    card_type: CardType::Minion,
    cost: 4,
    attack: 3,
    health: 6,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Elemental),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_027 Wings of Eternity — 1-cost (SPELL). "Discover a Dragon from
/// the past with a Dark Gift." (The real Discover is a D2 random pick
/// from the Dragon pool; the "from the past" pool is approximated by the
/// active window, the TIME_861 convention.)
pub const WINGS_OF_ETERNITY: CardDef = CardDef {
    id: "END_027",
    name: "Wings of Eternity",
    card_type: CardType::Spell,
    cost: 1,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: Some(CardEffect::DiscoverWithDarkGift {
        pool: RandomPool::Dragon,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_028 For All Time — 4-cost (SPELL). "Destroy all minions with 4 or
/// less Attack. Overload: (2)." (The Overload attaches via the overload
/// table in `apply_card_keywords`.)
pub const FOR_ALL_TIME: CardDef = CardDef {
    id: "END_028",
    name: "For All Time",
    card_type: CardType::Spell,
    cost: 4,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: Some(CardEffect::DestroyAllMinionsWith4OrLessAttack),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_029 Voodoo Totem — 2-cost 0/4 (MINION, Totem). "At the end of your
/// turn, get a random Shadow spell."
pub const VOODOO_TOTEM: CardDef = CardDef {
    id: "END_029",
    name: "Voodoo Totem",
    card_type: CardType::Minion,
    cost: 2,
    attack: 0,
    health: 4,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Totem),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: Some(CardEffect::AddRandomShadowSpell),
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_030 Haywire Hornswog — 6-cost 4/6 (MINION, Dragon). "Elusive,
/// Taunt. Costs (1) less for each Mana Crystal you've Overloaded this
/// game." (The game-long `overload_total` counter, read by the cost
/// layer.)
pub const HAYWIRE_HORNSWOG: CardDef = CardDef {
    id: "END_030",
    name: "Haywire Hornswog",
    card_type: CardType::Minion,
    cost: 6,
    attack: 4,
    health: 6,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: true,
    stealth: false,
    elusive: true,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_031 Shade of the End Time — 3-cost 3/3 (MINION, Undead). "Stealth.
/// Spell Damage +1."
pub const SHADE_OF_THE_END_TIME: CardDef = CardDef {
    id: "END_031",
    name: "Shade of the End Time",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 3,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: true,
    elusive: false,
    race: Some(Race::Undead),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 1,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_032 Winged Aberration — 4-cost 4/4 (MINION, Dragon). "Rush. Combo:
/// Overload for (2) to gain Immune this turn and Windfury." (Rush is
/// applied by `apply_card_keywords`; the combo overloads then grants the
/// two keywords, §22.)
pub const WINGED_ABERRATION: CardDef = CardDef {
    id: "END_032",
    name: "Winged Aberration",
    card_type: CardType::Minion,
    cost: 4,
    attack: 4,
    health: 4,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: Some(CardEffect::OverloadForAndGainImmuneWindfury { overload: 2 }),
    attack_equals_health: false,
};

/// END_033 Prescient Slitherdrake — 7-cost 5/8 (MINION, Dragon). "Elusive.
/// Costs (3) less if you're holding another Dragon." (The cost layer
/// reads the hand, TIME_852 convention.)
pub const PRESCIENT_SLITHERDRAKE: CardDef = CardDef {
    id: "END_033",
    name: "Prescient Slitherdrake",
    card_type: CardType::Minion,
    cost: 7,
    attack: 5,
    health: 8,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: true,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_034 Crumblecrusher — 8-cost 8/6 (MINION, Dragon). "Battlecry:
/// Destroy a random enemy minion, location, and weapon."
pub const CRUMBLECRUSHER: CardDef = CardDef {
    id: "END_034",
    name: "Crumblecrusher",
    card_type: CardType::Minion,
    cost: 8,
    attack: 8,
    health: 6,
    durability: 0,
    battlecry: Some(CardEffect::DestroyRandomEnemyMinionLocationWeapon),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_035 Omen of the End — 5-cost 5/5 (MINION, Dragon). "Battlecry: If
/// your deck is empty, destroy the top 5 cards of the enemy deck."
pub const OMEN_OF_THE_END: CardDef = CardDef {
    id: "END_035",
    name: "Omen of the End",
    card_type: CardType::Minion,
    cost: 5,
    attack: 5,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::DestroyTopFiveEnemyDeckIfOwnEmpty),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_036 Morchie — 4-cost 3/6 (MINION). "Your Rewinds keep BOTH
/// potential outcomes. Battlecry: Discover a Rewind card from any
/// class." (The aura is the documented §22 simplification — a
/// unit-marker `RewindKeepsBothOutcomes` aura consulted by
/// `engine::rewind`; the discover is the existing
/// `AddRandomRewindCardToHand`, D2.)
pub const MORCHIE: CardDef = CardDef {
    id: "END_036",
    name: "Morchie",
    card_type: CardType::Minion,
    cost: 4,
    attack: 3,
    health: 6,
    durability: 0,
    battlecry: Some(CardEffect::AddRandomRewindCardToHand),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: Some((
        AuraEffect::RewindKeepsBothOutcomes,
        AuraTarget::AllFriendlyMinions,
    )),
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_037 Endtime Murozond — 9-cost 4/6 (MINION). "Battlecry: Fill your
/// board with random Dragons. Fully heal your hero. Skip your next
/// turn." (The skip is the `skip_next_turn` player flag, consumed at the
/// owner's next TurnStarted; the dragons are D2 random picks from the
/// Dragon pool.)
pub const ENDTIME_MUROZOND: CardDef = CardDef {
    id: "END_037",
    name: "Endtime Murozond",
    card_type: CardType::Minion,
    cost: 9,
    attack: 4,
    health: 6,
    durability: 0,
    battlecry: Some(CardEffect::FillBoardRandomDragonsHealHeroSkipNextTurn),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_009t Treant — 1-cost 2/2 (MINION, the Splintered Reality token).
/// The +1/+1 scaling rides the summoning spell's enchantment.
pub const TREANT: CardDef = CardDef {
    id: "END_009t",
    name: "Treant",
    card_type: CardType::Minion,
    cost: 1,
    attack: 2,
    health: 2,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_017t Tick and Tock — 5-cost 8/8 (MINION, Dragon; the Battle at the
/// End Time quest reward). "Battlecry: Draw until your hand is full.
/// Deathrattle: Empty the opponent's hand."
pub const TICK_AND_TOCK: CardDef = CardDef {
    id: "END_017t",
    name: "Tick and Tock",
    card_type: CardType::Minion,
    cost: 5,
    attack: 8,
    health: 8,
    durability: 0,
    battlecry: Some(CardEffect::DrawUntilHandFull),
    deathrattle: Some(CardEffect::EmptyOpponentHand),
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// END_002t Dagger — 1/2 (WEAPON, the Wicked Blightspawn token). The
/// card text's "Equip a 1/2 Dagger" (END_002's deathrattle) — the engine
/// defines no Rogue dagger card, so the dagger is a small handwritten
/// token like the other W3 tokens (no generated baseline).
pub const DAGGER: CardDef = CardDef {
    id: "END_002t",
    name: "Dagger",
    card_type: CardType::Weapon,
    cost: 1,
    attack: 1,
    health: 0,
    durability: 2,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};
