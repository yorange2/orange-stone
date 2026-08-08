//! 2025–2026 expansions M1-W2 cards (exp_edr_w2) — the Emerald Dream (EDR)
//! dark-gift wave: 9 dark-gift cards.
//!
//! The dark-gift mechanic (see `trigger.rs` `apply_dark_gift` and the
//! `DarkGiftKind` component): the W2 cards Discover (simplified to a random
//! pick, matching the W1 Discover simplification) a qualifying minion and
//! give it one of the ten dark gifts — card-level upgrades that persist
//! across zones (hand / deck / play). The gift marker rides the `World`
//! `dark_gifts` component; the static effects (enchantments + keyword
//! components) are applied at grant time, and the behavioral gifts
//! (SummonCopyOnPlay, BattlecryTwice, RebornFull) are resolved by the
//! `engine/rules.rs` hooks.
//!
//! These consts are handwritten effect-wave implementations of the generated
//! expansion baselines: they never enter `ALL_CARDS` (the sampling pools
//! stay closed, decision D3) but are reachable through the `card_by_id`
//! chain (ALL_CARDS → HANDWRITTEN_EXPANSION_CARDS → EXPANSION_CARDS) and are
//! compared field-by-field against the generated baseline by the
//! `expansion_differential_gate` test (M0.4). Simplifications registered in
//! `docs/finished/fidelity-debt.md` §14 (2025–2026 expansions).
//!
//! Card data: `cards/cards.json` (EDR_* ids); card texts verified against
//! the official Emerald Dream set (2026-08-08).

use crate::cards::def::CardDef;
use crate::core::component::CardType;
use crate::core::component::Race;
use crate::core::effect::CardEffect;
use crate::core::effect::RandomPool;

/// EDR_102 Treacherous Tormentor — 4/5/4 neutral Demon. Battlecry: Discover
/// a Legendary minion with a Dark Gift. (simplified: Discover is a random
/// pick over the in-window Legendary pool, then a random dark gift —
/// fidelity-debt §14.)
pub const TREACHEROUS_TORMENTOR: CardDef = CardDef {
    id: "EDR_102",
    name: "Treacherous Tormentor",
    card_type: CardType::Minion,
    cost: 4,
    attack: 5,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::DiscoverWithDarkGift {
        pool: RandomPool::Legendary,
    }),
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

/// EDR_456 Darkrider — 1/1/1 Warrior Demon. Battlecry: If you're holding a
/// Dragon, Discover a Dragon with a Dark Gift. (simplified: Discover is a
/// random pick over the in-window Dragon pool, then a random dark gift —
/// fidelity-debt §14.)
pub const DARKRIDER: CardDef = CardDef {
    id: "EDR_456",
    name: "Darkrider",
    card_type: CardType::Minion,
    cost: 1,
    attack: 1,
    health: 1,
    durability: 0,
    battlecry: Some(CardEffect::DiscoverDragonWithDarkGift),
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

/// EDR_487 Wallow, the Wretched — 7/6/6 Warlock. While this is in your hand
/// or deck, it gains a copy of every Dark Gift given to your minions.
/// (simplified: the per-player gift log records kinds only, not targets —
/// the sync applies the same gifts to every friendly Wallow in the hand or
/// deck at the moment a gift is given; gifts granted before Wallow enters
/// the hand or deck are not retroactively copied — fidelity-debt §14.)
pub const WALLOW_THE_WRETCHED: CardDef = CardDef {
    id: "EDR_487",
    name: "Wallow, the Wretched",
    card_type: CardType::Minion,
    cost: 7,
    attack: 6,
    health: 6,
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

/// EDR_488 Avant-Gardening — 2-mana Warlock spell. Discover a Deathrattle
/// minion with a Dark Gift. (simplified: Discover is a random pick over the
/// in-window Deathrattle-minion pool — a Deathrattle effect or a death
/// trigger — then a random dark gift — fidelity-debt §14.)
pub const AVANT_GARDENING: CardDef = CardDef {
    id: "EDR_488",
    name: "Avant-Gardening",
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
    spell_effect: Some(CardEffect::DiscoverWithDarkGift {
        pool: RandomPool::DeathrattleMinion,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_528 Nightmare Fuel — 1-mana Rogue spell. Discover a copy of a minion
/// in your opponent's deck. Combo: With a Dark Gift. (**pool-open**: reads
/// the opponent's deck — registered in `sets::POOL_OPEN_CARDS`; Discover is
/// simplified to a random pick over the actual enemy-deck minions, and the
/// Combo branch gives the copy a random dark gift — fidelity-debt §14.)
pub const NIGHTMARE_FUEL: CardDef = CardDef {
    id: "EDR_528",
    name: "Nightmare Fuel",
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
    spell_effect: Some(CardEffect::DiscoverEnemyDeckMinionCopy { with_gift: false }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: Some(CardEffect::DiscoverEnemyDeckMinionCopy { with_gift: true }),
    attack_equals_health: false,
};

/// EDR_654 Overgrown Horror — 4/3/5 Warlock. Taunt. Battlecry: Reduce the
/// Cost of minions in your hand with Dark Gifts by (2).
pub const OVERGROWN_HORROR: CardDef = CardDef {
    id: "EDR_654",
    name: "Overgrown Horror",
    card_type: CardType::Minion,
    cost: 4,
    attack: 3,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::ReduceHandMinionGiftCost),
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

/// EDR_811 Rite of Atrocity — 1-mana Death Knight spell. Discover an Undead.
/// Spend 2 Corpses to give it a Dark Gift. (simplified: Discover is a random
/// pick over the in-window Undead pool — fidelity-debt §14.)
pub const RITE_OF_ATROCITY: CardDef = CardDef {
    id: "EDR_811",
    name: "Rite of Atrocity",
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
    spell_effect: Some(CardEffect::DiscoverUndeadWithCorpseGift { corpses: 2 }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// EDR_856 Nightmare Lord Xavius — 4/4/4 neutral Demon. Battlecry: Discover
/// a minion from your deck. Give it a Dark Gift. (simplified: Discover is a
/// random pick over the player's own deck — in-pool, not pool-open — then a
/// random dark gift — fidelity-debt §14.)
pub const NIGHTMARE_LORD_XAVIUS: CardDef = CardDef {
    id: "EDR_856",
    name: "Nightmare Lord Xavius",
    card_type: CardType::Minion,
    cost: 4,
    attack: 4,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::DiscoverDeckMinionWithDarkGift),
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

/// EDR_882 Jumpscare! — 2-mana Demon Hunter spell. Discover a Demon that
/// costs (5) or more with a Dark Gift. Shuffle the other two into your deck.
/// (simplified: Discover is a random pick over the in-window Demon pool
/// costing (5)+, then a random dark gift; the "shuffle the other two into
/// your deck" clause is moot under the random simplification — fidelity-debt
/// §14.)
pub const JUMPSCARE: CardDef = CardDef {
    id: "EDR_882",
    name: "Jumpscare!",
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
    spell_effect: Some(CardEffect::DiscoverWithDarkGift {
        pool: RandomPool::DemonCost5Plus,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};
