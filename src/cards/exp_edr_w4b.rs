//! 2025–2026 expansions M1-W4b cards (exp_edr_w4b) — the Emerald Dream (EDR)
//! elite wave: the 23 legendary Wild Gods (one per class + Neutrals) of the
//! W4 "remaining cards" list (the FIR_* miniset is M1-W5 and is NOT
//! implemented here).
//!
//! Mechanics introduced by this wave:
//! - composite Start-of-Game + battlecry effects (Ysera — the +5 maximum
//!   Mana applies at play time, §14.4);
//! - choose-thrice (Cenarius — a repeatable choose-one choice, `repeat` on
//!   PendingChoice);
//! - a per-player attack counter (Omen's "Improves" deathrattle);
//! - a per-player spell double-cast counter (Tyrande);
//! - the damage-pipeline doubling hook (Goldrinn — friendly Beasts deal
//!   double damage, §14.4 aura approximation);
//! - per-player one-time next-card cost flags (Agamaggan's (0), Aviana's
//!   game-long (1));
//! - a per-turn Dragon-play counter (Naralex's first-Dragon discount);
//! - the keep-or-top choice (Q'onzu — `ChoiceKind::QonzuKeepOrTop`);
//! - Ursoc's kill-record resurrection (the battlecry's sync-damage attack
//!   records the killed card IDs for the deathrattle);
//! - Ashamane's enemy-deck hand fill — a pool-open card (registered in
//!   `POOL_OPEN_CARDS`, the Nightmare Fuel precedent).
//!
//! D2 simplifications registered in `docs/finished/fidelity-debt.md` §14.4:
//! Ysera's Start-of-Game timing; Ohn'ahra's "play the top 3 cards" → draw 3;
//! Toreth's three-hit Divine Shield → normal shield; Ursol's 3-turn aura
//! cast → immediate cast; Omen's per-minion "Improves" → per-player counter;
//! Tyrande's double-cast timing; Goldrinn's aura → damage-pipeline hook;
//! Agamaggan's opponent-Health cost → (0); Q'onzu's Discover → random spell;
//! Renferal's trap → random discard; Nythendra's split/reform → beetle
//! summon; Shaladrassil's corruption clause; Aviana's lunar cycle → immediate
//! game-long (1).
//!
//! These consts are handwritten effect-wave implementations of the generated
//! expansion baselines: they never enter `ALL_CARDS` (the sampling pools
//! stay closed, decision D3) but are reachable through the `card_by_id`
//! chain (ALL_CARDS → HANDWRITTEN_EXPANSION_CARDS → EXPANSION_CARDS) and are
//! compared field-by-field against the generated baseline by the
//! `expansion_differential_gate` test (M0.4). Simplifications registered in
//! `docs/finished/fidelity-debt.md` §14.4 (2025–2026 expansions).
//!
//! Card data: `cards/cards.json` (EDR_* ids); card texts verified against
//! the official Emerald Dream set (2026-08-08).

use crate::cards::def::CardDef;
use crate::core::component::CardType;
use crate::core::component::Race;
use crate::core::effect::CardEffect;

// ---------------------------------------------------------------------------
// M1-W4b cards
// ---------------------------------------------------------------------------

/// EDR_000 Ysera, Emerald Aspect — 9/4/12 Neutral Dragon. Start of Game:
/// both players' maximum Mana +5. Battlecry: Gain 3 Mana Crystals.
/// (simplified: the engine has no StartOfGame event — the +5 applies when
/// Ysera is played, §14.4.)
pub const YSERA_EMERALD_ASPECT: CardDef = CardDef {
    id: "EDR_000",
    name: "Ysera, Emerald Aspect",
    card_type: CardType::Minion,
    cost: 9,
    attack: 4,
    health: 12,
    durability: 0,
    battlecry: Some(CardEffect::YseraEmeraldAspect),
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

/// EDR_031 Ohn'ahra — 9/5/11 Shaman Beast. At the end of your turn, play the
/// top 3 cards from your deck. (simplified: "play the top 3" has no pipeline
/// — the effect draws 3 cards instead, §14.4.)
pub const OHNAHRA: CardDef = CardDef {
    id: "EDR_031",
    name: "Ohn'ahra",
    card_type: CardType::Minion,
    cost: 9,
    attack: 5,
    health: 11,
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
    end_turn_effect: Some(CardEffect::DrawCard { count: 3 }),
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_209 Forest Lord Cenarius — 10/5/8 Druid. Choose Thrice: Give your
/// other minions +1/+3; or Summon a 5/5 Ancient with Taunt. (The choice
/// surfaces three times via PendingChoice.repeat — each pick resolves one
/// branch, options may be mixed.)
pub const FOREST_LORD_CENARIUS: CardDef = CardDef {
    id: "EDR_209",
    name: "Forest Lord Cenarius",
    card_type: CardType::Minion,
    cost: 10,
    attack: 5,
    health: 8,
    durability: 0,
    battlecry: Some(CardEffect::GainStatsAllOtherFriendlyMinions {
        attack: 1,
        health: 3,
    }),
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
    choose_one_effect: Some(CardEffect::SummonMinion {
        card_id: "EDR_209t",
    }),
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_238 Merithra — 6/4/4 Shaman. Battlecry: Resurrect all different
/// friendly minions that cost (8) or more. ("Different" is deduped by card
/// ID — first death order wins.)
pub const MERITHRA: CardDef = CardDef {
    id: "EDR_238",
    name: "Merithra",
    card_type: CardType::Minion,
    cost: 6,
    attack: 4,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::ResurrectAllDifferentFriendlyCostGE { cost: 8 }),
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

/// EDR_258 Toreth the Unbreaking — 5/3/4 Paladin. Divine Shield, Taunt. Your
/// Divine Shields take three hits to break. (simplified: the three-hit
/// shield is a new mechanic — Toreth carries a normal Divine Shield, §14.4.)
pub const TORETH_THE_UNBREAKING: CardDef = CardDef {
    id: "EDR_258",
    name: "Toreth the Unbreaking",
    card_type: CardType::Minion,
    cost: 5,
    attack: 3,
    health: 4,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: true,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: true,
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

/// EDR_259 Ursol — 8/9/7 Paladin Beast. Battlecry: Cast the highest Cost
/// spell from your hand as an Aura that lasts 3 turns. (simplified: the
/// aura-cast is a new mechanic — the highest-Cost hand spell is cast
/// immediately instead, §14.4.)
pub const URSOL: CardDef = CardDef {
    id: "EDR_259",
    name: "Ursol",
    card_type: CardType::Minion,
    cost: 8,
    attack: 9,
    health: 7,
    durability: 0,
    battlecry: Some(CardEffect::CastHighestCostSpellFromHand),
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

/// EDR_421 Omen — 10/6/12 Demon Hunter Demon-Beast. Rush, Windfury.
/// Deathrattle: Deal 1 damage to all enemies. (Improves after this attacks!)
/// (registered interpretation: the official per-minion "Improves"
/// enchantment is approximated by a per-player attack counter that adds 1
/// damage per attack, §14.4. The Demon half of the dual tribe and Rush land
/// in `apply_card_keywords`.)
pub const OMEN: CardDef = CardDef {
    id: "EDR_421",
    name: "Omen",
    card_type: CardType::Minion,
    cost: 10,
    attack: 6,
    health: 12,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::OmenDeathrattle),
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Beast),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: true,
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

/// EDR_430 Aessina — 8/6/8 Mage Undead. Battlecry: If 20 friendly minions
/// have died this game, deal 20 damage split among all enemies. (The split
/// is 20 random 1-damage pings; the graveyard IS the death record.)
pub const AESSINA: CardDef = CardDef {
    id: "EDR_430",
    name: "Aessina",
    card_type: CardType::Minion,
    cost: 8,
    attack: 6,
    health: 8,
    durability: 0,
    battlecry: Some(CardEffect::SplitDamageAmongAllEnemiesIfFallen {
        amount: 20,
        threshold: 20,
    }),
    deathrattle: None,
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

/// EDR_464 Tyrande — 7/5/7 Priest. Battlecry: The next 3 spells you play
/// cast twice. (simplified timing: the re-cast fires immediately after the
/// original resolution with no target and fires no second SpellCast event,
/// §14.4.)
pub const TYRANDE: CardDef = CardDef {
    id: "EDR_464",
    name: "Tyrande",
    card_type: CardType::Minion,
    cost: 7,
    attack: 5,
    health: 7,
    durability: 0,
    battlecry: Some(CardEffect::NextSpellsCastTwice { count: 3 }),
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

/// EDR_465 Ysondre — 7/8/5 Warrior Dragon. Taunt. Deathrattle: Summon a
/// random Dragon for each time Ysondre has died this game. (The graveyard
/// IS the death record — the dying instance is already there when the
/// deathrattle resolves, so the count includes this death.)
pub const YSONDRE: CardDef = CardDef {
    id: "EDR_465",
    name: "Ysondre",
    card_type: CardType::Minion,
    cost: 7,
    attack: 8,
    health: 5,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::SummonRandomDragonPerSelfDeath),
    taunt: true,
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

/// EDR_471 Tortolla — 10/1/30 Warrior Beast. Taunt, Elusive. After this
/// takes damage, gain 1 Armor and give this minion +1 Attack. (The armor
/// goes to the owner's hero; the trigger is registered in
/// `apply_card_keywords`.)
pub const TORTOLLA: CardDef = CardDef {
    id: "EDR_471",
    name: "Tortolla",
    card_type: CardType::Minion,
    cost: 10,
    attack: 1,
    health: 30,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: true,
    stealth: false,
    elusive: true,
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

/// EDR_480 Goldrinn — 9/9/9 Hunter Beast. Rush. Friendly Beasts deal double
/// damage. (simplified: the aura is approximated by a damage-pipeline hook —
/// any Beast damage source owned by the Goldrinn player doubles while
/// EDR_480 is on the board, §14.4. Rush lands in `apply_card_keywords`.)
pub const GOLDRINN: CardDef = CardDef {
    id: "EDR_480",
    name: "Goldrinn",
    card_type: CardType::Minion,
    cost: 9,
    attack: 9,
    health: 9,
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

/// EDR_489 Agamaggan — 10/8/9 Warlock Beast. Battlecry: The next card you
/// play costs your OPPONENT'S Health instead of Mana (up to 10).
/// (simplified: the opponent-Health cost needs a cost-pipeline rework — the
/// next card costs (0) instead, §14.4.)
pub const AGAMAGGAN: CardDef = CardDef {
    id: "EDR_489",
    name: "Agamaggan",
    card_type: CardType::Minion,
    cost: 10,
    attack: 8,
    health: 9,
    durability: 0,
    battlecry: Some(CardEffect::NextCardCostsZero),
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

/// EDR_493 Alara'shi — 5/5/5 Demon Hunter Demon-Beast. Battlecry: Transform
/// minions in your hand into random Demons. (They keep their original stats
/// and Cost.) (The Demon half of the dual tribe lands in
/// `apply_card_keywords`.)
pub const ALARASHI: CardDef = CardDef {
    id: "EDR_493",
    name: "Alara'shi",
    card_type: CardType::Minion,
    cost: 5,
    attack: 5,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::TransformHandMinionsToRandomDemons),
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

/// EDR_517 Q'onzu — 3/3/4 Mage Beast. Battlecry: Discover a spell. Choose
/// to keep it or put it on top of your opponent's deck. (simplified: the
/// Discover picks a random spell from the full database — the standing
/// Discover→random debt, §14.4; the keep/top decision surfaces as the
/// QonzuKeepOrTop choice.)
pub const QONZU: CardDef = CardDef {
    id: "EDR_517",
    name: "Q'onzu",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::DiscoverSpellKeepOrTop),
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

/// EDR_526 Renferal, the Malignant — 3/3/3 Rogue Beast. Battlecry: Trap 1
/// random card in your opponent's hand for a turn. (Improved for each time
/// you've played this.) (simplified: the trap is a new mechanic — the enemy
/// discards a random hand card instead, §14.4.)
pub const RENFERAL_THE_MALIGNANT: CardDef = CardDef {
    id: "EDR_526",
    name: "Renferal, the Malignant",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::DiscardRandomEnemyHandCard),
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

/// EDR_527 Ashamane — 9/7/7 Rogue Beast. Battlecry: Fill your hand with
/// copies of cards from your opponent's deck. They cost (3) less.
/// (Pool-open: reads the opponent's actual deck — registered in
/// `POOL_OPEN_CARDS`, the Nightmare Fuel precedent.)
pub const ASHAMANE: CardDef = CardDef {
    id: "EDR_527",
    name: "Ashamane",
    card_type: CardType::Minion,
    cost: 9,
    attack: 7,
    health: 7,
    durability: 0,
    battlecry: Some(CardEffect::FillHandWithEnemyDeckCopies { reduction: 3 }),
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

/// EDR_818 Nythendra — 7/7/7 Death Knight Undead-Dragon. Taunt. Deathrattle:
/// Split into 1/1 Beetles. At the start of your turn, reform with any
/// remaining. (simplified: the split/reform is unmodeled — the deathrattle
/// summons seven 1/1 Beetles, one per point of the 7/7 body, §14.4. The
/// Dragon half of the dual tribe lands in `apply_card_keywords`.)
pub const NYTHENDRA: CardDef = CardDef {
    id: "EDR_818",
    name: "Nythendra",
    card_type: CardType::Minion,
    cost: 7,
    attack: 7,
    health: 7,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::SummonBeetles { count: 7 }),
    taunt: true,
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

/// EDR_819 Ursoc — 9/6/14 Death Knight Beast. Battlecry: Attack ALL other
/// minions. Deathrattle: Resurrect any this killed. (The battlecry's
/// sync-damage attacks record the killed card IDs; the deathrattle
/// resurrects them.)
pub const URSOC: CardDef = CardDef {
    id: "EDR_819",
    name: "Ursoc",
    card_type: CardType::Minion,
    cost: 9,
    attack: 6,
    health: 14,
    durability: 0,
    battlecry: Some(CardEffect::UrsocBattlecry),
    deathrattle: Some(CardEffect::UrsocDeathrattle),
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

/// EDR_844 Naralex, Herald of the Flights — 7/7/7 Neutral. Your first
/// Dragon each turn costs (1). (The per-turn Dragon-play counter lives on
/// Player::dragons_played_this_turn; the discount composes in
/// `cost::play_cost`.)
pub const NARALEX_HERALD_OF_THE_FLIGHTS: CardDef = CardDef {
    id: "EDR_844",
    name: "Naralex, Herald of the Flights",
    card_type: CardType::Minion,
    cost: 7,
    attack: 7,
    health: 7,
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

/// EDR_846 Shaladrassil — 8-cost Neutral spell. Get all 5 Dream cards. If
/// you've played a higher Cost card while holding this, corrupt them!
/// (simplified: the corruption clause is unmodeled — the spell adds the five
/// DREAM_POOL cards to hand, §14.4.)
pub const SHALADRASSIL: CardDef = CardDef {
    id: "EDR_846",
    name: "Shaladrassil",
    card_type: CardType::Spell,
    cost: 8,
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
    spell_effect: Some(CardEffect::AddAllDreamCards),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_853 Broll Bearmantle — 5/3/5 Hunter. After you cast a spell, summon
/// a random Animal Companion. (The trigger is declared via spell_trigger —
/// the FriendlySpellCast registration is automatic.)
pub const BROLL_BEARMANTLE: CardDef = CardDef {
    id: "EDR_853",
    name: "Broll Bearmantle",
    card_type: CardType::Minion,
    cost: 5,
    attack: 3,
    health: 5,
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
    spell_trigger: Some(CardEffect::SummonRandomAnimalCompanion),
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_895 Aviana, Elune's Chosen — 9/7/11 Priest. Battlecry: Start a three
/// turn lunar cycle. When the Full Moon rises, your cards cost (1) this
/// game. (simplified: the lunar-cycle timing is unmodeled — the (1)-for-the-
/// game effect applies immediately, §14.4.)
pub const AVIANA_ELUNES_CHOSEN: CardDef = CardDef {
    id: "EDR_895",
    name: "Aviana, Elune's Chosen",
    card_type: CardType::Minion,
    cost: 9,
    attack: 7,
    health: 11,
    durability: 0,
    battlecry: Some(CardEffect::CardsCostOneThisGame),
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

// ---------------------------------------------------------------------------
// M1-W4b tokens (handwritten only — no generated baselines)
// ---------------------------------------------------------------------------

/// EDR_209t Ancient — the 5/5 Taunt Ancient Forest Lord Cenarius's "Summon
/// a 5/5 Ancient with Taunt" option summons.
pub const CENARIUS_ANCIENT: CardDef = CardDef {
    id: "EDR_209t",
    name: "Ancient",
    card_type: CardType::Minion,
    cost: 5,
    attack: 5,
    health: 5,
    durability: 0,
    battlecry: None,
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

/// EDR_818t Beetle — the 1/1 Beetle Nythendra's simplified deathrattle
/// summons (seven of them, one per point of the 7/7 body — §14.4).
pub const NYTHENDRA_BEETLE: CardDef = CardDef {
    id: "EDR_818t",
    name: "Beetle",
    card_type: CardType::Minion,
    cost: 1,
    attack: 1,
    health: 1,
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
