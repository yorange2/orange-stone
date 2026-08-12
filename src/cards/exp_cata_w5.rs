//! MEND W1 cards (exp_cata_w5) — the Cataclysm class-set W1 wave, the
//! first MEND_ wave (2025–2026 expansions master roadmap M5 follow-up):
//! the Druid class set (8 cards: MEND_040~046 + MEND_100). These are
//! registered in `sets.rs` (HANDWRITTEN_EXPANSION_CARDS) like the other
//! effect waves and override the generated baselines (every MEND_ card
//! has one — the `expansion_differential_gate` compares the stat fields
//! field-by-field; MEND_044's Location divergence is documented in
//! `expansion_differential_rebalanced`).
//!
//! Implementation decisions (per the MEND W1 spec, verified against
//! `cards/cards.json` 2026-08-12; the fidelity rows are
//! fidelity-debt.md §29, en + zh):
//! - **"Didn't play a minion last turn"** (MEND_041 Wizened Wildspeaker,
//!   MEND_043 Heartroot Stones) rides the EXISTING
//!   `last_turn_minion_play_ids` snapshot: the per-turn play list is
//!   pushed at CardPlayed and mem::take'd into the last-turn list at the
//!   owner's turn end (the M3-W2b Chrono-Lord Epoch machinery), so the
//!   flag reads `last_turn_minion_play_ids.is_empty()` with no new
//!   player state.
//! - **Ash Worm** (MEND_040) enters play Dormant via the
//!   `dormant_at_summon` registry with the u32::MAX sentinel: the
//!   turn-start countdown skips the sentinel and the MinionSummoned
//!   handler (the single funnel for ALL summons) awakens sentinel-dormant
//!   minions when the owner's board reaches MAX_BOARD_SIZE. The worm
//!   played onto a 6-minion board fills it itself and awakens
//!   immediately.
//! - **Tranquil Clearing** (MEND_044) is a Location whose battlecry
//!   slot holds the ACTIVATE effect (the W2 §24 convention); the target
//!   minion choice surfaces at the ActivateLocation action. The sleep
//!   rides the existing Dormant component with one turn — it awakens at
//!   the minion's NEXT TURN START regardless of owner, versus the
//!   official "end of your next turn" (§29).
//! - **Bashana Runetotem** (MEND_046): the official carves the spells
//!   INTO the Treant tokens (each MEND_046t carries "Battlecry: Cast
//!   {0}"); the simplified form adds three plain 2/2 Treants plus up to
//!   three random Nature spells — each costing no more than the
//!   remaining 12-Mana budget — directly to the hand (§29).
//! - **Cultivating Sprite** (MEND_100): the Bulb (MEND_100t) is a
//!   3-cost spell whose own cost stays fixed; the upgrade raises the
//!   COST OF THE CAST SPELLS by one per owner turn start in hand (the
//!   HandTurnCounter convention, CATA_498), and the cast spells sample
//!   the full catalog at the given cost (the `random_minion_of_cost`
//!   convention) — the official pool is the active window (§29).
//! - Lifebloom's heal rides the AllFriendlyCharacters path (Darkscale
//!   Healer) with a Velen-only apply_spell_power arm; Seeding Dragon's
//!   random Dragon samples the full catalog for the Dragon race.

use crate::cards::def::CardDef;
use crate::core::component::{CardType, Race};
use crate::core::effect::CardEffect;

/// MEND_040 Ash Worm — 1-cost 6/6 (MINION, Druid, Beast). "Starts
/// Dormant. When your board is full, awaken." — the dormant lands via
/// the `dormant_at_summon` sentinel (u32::MAX); the board-full check in
/// the MinionSummoned handler awakens it (§29).
pub const ASH_WORM: CardDef = CardDef {
    id: "MEND_040",
    name: "Ash Worm",
    card_type: CardType::Minion,
    cost: 1,
    attack: 6,
    health: 6,
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

/// MEND_041 Wizened Wildspeaker — 5-cost 3/7 (MINION, Druid). "Taunt.
/// Battlecry: If you didn't play a minion last turn, refresh 3 Mana
/// Crystals." — the last-turn flag reads the `last_turn_minion_play_ids`
/// snapshot (no new player state, §29).
pub const WIZENED_WILDSPEAKER: CardDef = CardDef {
    id: "MEND_041",
    name: "Wizened Wildspeaker",
    card_type: CardType::Minion,
    cost: 5,
    attack: 3,
    health: 7,
    durability: 0,
    battlecry: Some(CardEffect::RefreshManaIfNoMinionPlayedLastTurn { amount: 3 }),
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

/// MEND_042 Lifebloom — 9-cost (SPELL, Druid). "Restore 8 Health to all
/// friendly characters. Summon two random 8-Cost minions." — the heal
/// rides the AllFriendlyCharacters path (Velen doubles it, Spell Damage
/// does not); the summons sample the full catalog at cost 8.
pub const LIFEBLOOM: CardDef = CardDef {
    id: "MEND_042",
    name: "Lifebloom",
    card_type: CardType::Spell,
    cost: 9,
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
    spell_effect: Some(
        CardEffect::RestoreAllFriendlyAndSummonTwoRandomCostMinions { heal: 8, cost: 8 },
    ),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// MEND_043 Heartroot Stones — 3-cost (SPELL, Druid). "Draw a card and
/// gain 3 Armor. If you didn't play a minion last turn, do it again." —
/// the repeat rides the `last_turn_minion_play_ids` snapshot.
pub const HEARTROOT_STONES: CardDef = CardDef {
    id: "MEND_043",
    name: "Heartroot Stones",
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
    spell_effect: Some(CardEffect::DrawAndGainArmorRepeatIfNoMinionPlayedLastTurn {
        draw: 1,
        armor: 3,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// MEND_044 Tranquil Clearing — 2-cost Location, 2 durability (Druid).
/// "Give a minion +2 Health and Taunt. It falls asleep until the end of
/// your next turn." — the activation effect sits in the battlecry slot
/// (the W2 §24 convention); the target choice surfaces at the
/// ActivateLocation action; the sleep is Dormant with one turn (§29).
pub const TRANQUIL_CLEARING: CardDef = CardDef {
    id: "MEND_044",
    name: "Tranquil Clearing",
    card_type: CardType::Location,
    cost: 2,
    attack: 0,
    health: 0,
    durability: 2,
    battlecry: Some(CardEffect::BuffHealthTauntAndDormant { health: 2 }),
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

/// MEND_045 Seeding Dragon — 4-cost 4/4 (MINION, Druid, Dragon).
/// "Taunt. Deathrattle: Get a random Dragon. It costs (2) less." — the
/// pool is the full catalog's Dragon minions (the `random_minion_of_cost`
/// convention, §29).
pub const SEEDING_DRAGON: CardDef = CardDef {
    id: "MEND_045",
    name: "Seeding Dragon",
    card_type: CardType::Minion,
    cost: 4,
    attack: 4,
    health: 4,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::AddRandomDragonCostReduced { reduction: 2 }),
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

/// MEND_046 Bashana Runetotem — 7-cost 4/4 (MINION, Druid, Legendary).
/// "Battlecry: Get three 2/2 Treants. Carve 12 Mana worth of Nature
/// spells into them." — the carve is simplified: the Treants and up to
/// three random Nature spells (each ≤ the remaining 12-Mana budget) are
/// added to the hand (§29).
pub const BASHANA_RUNETOTEM: CardDef = CardDef {
    id: "MEND_046",
    name: "Bashana Runetotem",
    card_type: CardType::Minion,
    cost: 7,
    attack: 4,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::GetThreeTreantsAndCarveNatureSpells),
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

/// MEND_046t Treant — 1-cost 2/2 (MINION, token). Bashana Runetotem's
/// plain 2/2 Treant (the official token casts the carved Nature spell
/// when played — simplified, §29; the token name is the standard
/// "Treant").
pub const TREANT: CardDef = CardDef {
    id: "MEND_046t",
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

/// MEND_100 Cultivating Sprite — 3-cost 3/3 (MINION, Neutral). "Battlecry:
/// Get a 3-Cost Bulb that casts three random 1-Cost spells. It upgrades
/// each turn." — the Bulb (MEND_100t) rides the AddCardToHand path.
pub const CULTIVATING_SPRITE: CardDef = CardDef {
    id: "MEND_100",
    name: "Cultivating Sprite",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::AddCardToHand {
        card_id: "MEND_100t",
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
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// MEND_100t Blooming Bulb — 3-cost (SPELL, token). "Cast three random
/// spells that cost (1). (Upgrades each turn!)" — the hand-turn counter
/// (bumped at each owner turn start in hand, the CATA_498 convention)
/// raises the cost of the cast spells; the bulb's own cost stays fixed
/// at 3 (§29).
pub const BLOOMING_BULB: CardDef = CardDef {
    id: "MEND_100t",
    name: "Blooming Bulb",
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
    spell_effect: Some(CardEffect::CastRandomSpellsScaledByHandTurns { base: 1, count: 3 }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};
