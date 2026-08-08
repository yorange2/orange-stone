//! Kindred definitions (2025–2026 expansions M2-W3 — the Un'Goro Kindred
//! mechanic): the static half of the 27 Kindred cards, keyed by card id.
//!
//! "Kindred: X" — X activates when the player played a card of the SAME
//! TYPE earlier THIS TURN. The type is the tribe for minions (all 27
//! Kindred minions have tribes: Beast/Elemental/Murloc/Undead/Demon/Dragon
//! — the W4c DINO_435 Crater Experiment's official ALL tribe lands as
//! Beast, the §19 approximation) and the card type (SPELL) for spells. The
//! Kindred card itself never counts ("another card of the same type").
//!
//! The registry is the id-keyed analogue of `CardDef` fields — the kindred
//! data (type, effect shape) deliberately does NOT live in the ~800
//! `CardDef` literals; the play path looks it up via `kindred_type` /
//! `kindred_effect` instead (mirror of `cards::quest`, `apply_card_keywords`,
//! imbue). The 23 W3 Kindred cards plus the 4 W4c DINO Kindred cards
//! (Diabolus Rex, Firegill, Chillspine Stegodon, Crater Experiment) are
//! the registry.

use serde::{Deserialize, Serialize};

use crate::core::component::{CardType, Race};
use crate::core::effect::CardEffect;
use crate::core::entity::Entity;
use crate::core::event::EventQueue;
use crate::core::player::PlayerId;
use crate::core::state::GameState;

/// Kindred type — what "same type" means for the activation check: the
/// tribe for minions, `Spell` for spells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KindredType {
    /// A spell card
    Spell,
    /// A minion of the given tribe
    Minion(Race),
}

/// Kindred resolution shapes (2025–2026 expansions M2-W3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KindredEffect {
    /// Resolved when the card is played — after it enters play / its base
    /// resolution — if the condition holds (TLC_251's "triggers twice"
    /// resolves it twice).
    OnPlay {
        /// The effect resolved on play
        effect: CardEffect,
    },
    /// The card costs `amount` less when the condition holds at play time.
    /// Checked BEFORE the play path pushes the card's own type, so the
    /// current card does not count itself (>= 1 earlier same-type card).
    CostDiscount {
        /// Discount amount
        amount: i32,
    },
    /// Battlecry modifier: `replace: true` — resolve this effect INSTEAD
    /// of the battlecry (a different target set / a different hand);
    /// `replace: false` — resolve it in addition (add-on), after the
    /// battlecry.
    BattlecryModifier {
        /// The modifier effect
        effect: CardEffect,
        /// Whether it replaces the base battlecry
        replace: bool,
    },
}

/// All Kindred card ids (2025–2026 expansions M2-W3 + M2-W4c) — the 27
/// cards with a `kindred_type` entry (the 23 W3 cards plus the 4 W4c DINO
/// cards). Torga's battlecry (TLC_102) scans the deck for the first of
/// these, then draws a same-type card to activate it.
pub const KINDRED_CARD_IDS: &[&str] = &[
    "TLC_102", "TLC_107", "TLC_223", "TLC_226", "TLC_236", "TLC_243", "TLC_251", "TLC_366",
    "TLC_428", "TLC_429", "TLC_432", "TLC_440", "TLC_447", "TLC_454", "TLC_463", "TLC_482",
    "TLC_519", "TLC_600", "TLC_815", "TLC_816", "TLC_825", "TLC_829", "TLC_903", "DINO_138",
    "DINO_404", "DINO_413", "DINO_435",
];

/// The Kindred type of a card definition — spells count as `Spell`, minions
/// as their tribe (the CardDef's single race; the six multi-race W3 cards
/// count as their primary race, see fidelity-debt §16). `None` for weapons,
/// heroes and race-less minions (a race-less minion can never activate a
/// Kindred check, matching the official semantics).
#[must_use]
pub fn played_type_of(def: &crate::cards::def::CardDef) -> Option<KindredType> {
    match def.card_type {
        CardType::Spell => Some(KindredType::Spell),
        CardType::Minion => def.race.map(KindredType::Minion),
        _ => None,
    }
}

/// Kindred type of a card, from the official card dump (extracted
/// 2026-08-09); `None` for every other card.
///
/// The 23 collectible Kindred cards of the W3 wave plus the 4 W4c DINO
/// Kindred cards. For the multi-race cards the dump lists two tribes; the
/// engine's CardDef models one (the primary race — the one the Kindred
/// text keys on; the second tribe lands via `apply_card_keywords`, the
/// Mythical Terror precedent). DINO_435's ALL tribe lands as Beast (the
/// §19 approximation).
#[must_use]
pub fn kindred_type(card_id: &str) -> Option<KindredType> {
    match card_id {
        "TLC_102" => Some(KindredType::Minion(Race::Beast)), // Torga
        "TLC_107" => Some(KindredType::Minion(Race::Elemental)), // Stormbrewer
        "TLC_223" => Some(KindredType::Minion(Race::Elemental)), // Volcanic Thrasher
        "TLC_226" => Some(KindredType::Minion(Race::Elemental)), // Conjured Bookkeeper
        "TLC_236" => Some(KindredType::Spell),               // Hybridization
        "TLC_243" => Some(KindredType::Minion(Race::Elemental)), // Whirling Stormdrake
        "TLC_251" => Some(KindredType::Minion(Race::Murloc)), // Primalfin Challenger
        "TLC_366" => Some(KindredType::Minion(Race::Beast)), // Pterrorwing Ravager
        "TLC_428" => Some(KindredType::Minion(Race::Murloc)), // Hot Spring Glider
        "TLC_429" => Some(KindredType::Minion(Race::Murloc)), // Steamfin Thief
        "TLC_432" => Some(KindredType::Minion(Race::Undead)), // Dread Raptor
        "TLC_440" => Some(KindredType::Spell),               // Cryosleep
        "TLC_447" => Some(KindredType::Spell),               // Caustic Fumes
        "TLC_454" => Some(KindredType::Minion(Race::Beast)), // Scalehide Kodo
        "TLC_463" => Some(KindredType::Minion(Race::Demon)), // Razidir
        "TLC_482" => Some(KindredType::Minion(Race::Elemental)), // Slagclaw
        "TLC_519" => Some(KindredType::Spell),               // Ambush Predators
        "TLC_600" => Some(KindredType::Minion(Race::Dragon)), // Windpeak Wyrm
        "TLC_815" => Some(KindredType::Spell),               // Gravedawn Voidbulb
        "TLC_816" => Some(KindredType::Spell),               // Gravedawn Sunbloom
        "TLC_825" => Some(KindredType::Minion(Race::Beast)), // Ravasaur Matriarch
        "TLC_829" => Some(KindredType::Minion(Race::Beast)), // Ravenous Devilsaur
        "TLC_903" => Some(KindredType::Minion(Race::Beast)), // Silithid Queen
        // M2-W4c — the Festival of the Devilsaur Kindred cards (the
        // primary race of the multi-race cards, matching the CardDef race)
        "DINO_138" => Some(KindredType::Minion(Race::Demon)), // Diabolus Rex
        "DINO_404" => Some(KindredType::Minion(Race::Elemental)), // Firegill
        "DINO_413" => Some(KindredType::Minion(Race::Elemental)), // Chillspine Stegodon
        "DINO_435" => Some(KindredType::Minion(Race::Beast)), // Crater Experiment
        _ => None,
    }
}

/// Kindred effect of a card (2025–2026 expansions M2-W3); `None` for the
/// five Kindred cards whose effect is folded elsewhere — TLC_102 (the
/// special `DrawKindredAndActivator` battlecry), TLC_223/236/432 (the
/// kindred modification is folded into their dedicated draw battlecries,
/// activation checked inside at >= 2), TLC_251 (a player flag only).
#[must_use]
pub fn kindred_effect(card_id: &str) -> Option<&'static KindredEffect> {
    match card_id {
        "TLC_107" => Some(&KindredEffect::OnPlay {
            effect: CardEffect::GainRush {
                target: crate::core::effect::EffectTarget::Self_,
            },
        }),
        "TLC_226" => Some(&KindredEffect::OnPlay {
            effect: CardEffect::SummonCopyOfSelf,
        }),
        "TLC_243" => Some(&KindredEffect::OnPlay {
            effect: CardEffect::GainImmuneThisTurn {
                target: crate::core::effect::EffectTarget::Self_,
            },
        }),
        "TLC_366" => Some(&KindredEffect::CostDiscount { amount: 2 }),
        "TLC_428" => Some(&KindredEffect::OnPlay {
            effect: CardEffect::GiveNextMurlocDivineShield,
        }),
        "TLC_429" => Some(&KindredEffect::OnPlay {
            effect: CardEffect::SummonMultipleMinions {
                card_id: "TLC_429t",
                count: 2,
            },
        }),
        "TLC_440" => Some(&KindredEffect::OnPlay {
            effect: CardEffect::DrawCard { count: 1 },
        }),
        "TLC_447" => Some(&KindredEffect::OnPlay {
            effect: CardEffect::DamageAllMinions { damage: 2 },
        }),
        "TLC_454" => Some(&KindredEffect::BattlecryModifier {
            effect: CardEffect::DestroyHighestAttackEnemy,
            replace: true,
        }),
        "TLC_463" => Some(&KindredEffect::BattlecryModifier {
            effect: CardEffect::DiscardRandomEnemyHandCard,
            replace: true,
        }),
        "TLC_482" => Some(&KindredEffect::BattlecryModifier {
            effect: CardEffect::TriggerFriendlyCinderDeathrattles,
            replace: false,
        }),
        "TLC_519" => Some(&KindredEffect::OnPlay {
            effect: CardEffect::SummonMinion {
                card_id: "TLC_519t",
            },
        }),
        "TLC_600" => Some(&KindredEffect::CostDiscount { amount: 3 }),
        "TLC_815" => Some(&KindredEffect::OnPlay {
            effect: CardEffect::SummonRandomMinionCostTaunt { cost: 4 },
        }),
        "TLC_816" => Some(&KindredEffect::CostDiscount { amount: 2 }),
        "TLC_825" => Some(&KindredEffect::OnPlay {
            effect: CardEffect::DealSelfAttackDamage {
                target: crate::core::effect::EffectTarget::AnyEnemyMinion,
            },
        }),
        "TLC_829" => Some(&KindredEffect::BattlecryModifier {
            effect: CardEffect::DestroyMinionAndGainItsStats {
                target: crate::core::effect::EffectTarget::AnyMinion,
            },
            replace: true,
        }),
        "TLC_903" => Some(&KindredEffect::OnPlay {
            effect: CardEffect::GainHeroAttack {
                attack: 5,
                armor: 0,
            },
        }),
        // M2-W4c — the Festival of the Devilsaur Kindred effects:
        // - Diabolus Rex (DINO_138): deal 6 to the left- and right-most
        //   enemy minions;
        // - Firegill (DINO_404): give the owner's OTHER minions Rush;
        // - Chillspine Stegodon (DINO_413): the Kindred and Freeze REPLACES
        //   the damage-only battlecry — the freeze lands on the SAME two
        //   minions the damage hit;
        // - Crater Experiment (DINO_435): summon a copy of this.
        "DINO_138" => Some(&KindredEffect::OnPlay {
            effect: CardEffect::DealDamageToLeftRightEnemyMinions { amount: 6 },
        }),
        "DINO_404" => Some(&KindredEffect::OnPlay {
            effect: CardEffect::GiveOtherFriendlyMinionsRush,
        }),
        "DINO_413" => Some(&KindredEffect::BattlecryModifier {
            effect: CardEffect::DealDamageToTwoAndFreeze { amount: 2 },
            replace: true,
        }),
        "DINO_435" => Some(&KindredEffect::OnPlay {
            effect: CardEffect::SummonCopyOfSelf,
        }),
        _ => None,
    }
}

/// Whether the Kindred activation condition holds: the player played at
/// least `required` cards of `my_type` this turn. The play path pushes a
/// played card's type BEFORE the card's OnPlay/battlecry-kindred checks
/// (so those pass `required = 2` — the card itself plus an earlier
/// same-type card — the self-exclusion is purely ordering-based) and
/// AFTER the cost pipeline (so cost-time checks pass `required = 1`).
#[must_use]
pub fn kindred_active(
    state: &GameState,
    player: PlayerId,
    my_type: KindredType,
    required: u32,
) -> bool {
    let count = state
        .player(player)
        .kindred_played
        .iter()
        .filter(|&&t| t == my_type)
        .count();
    count as u32 >= required
}

/// The Kindred cost discount (M2-W3) for a card about to be played — the
/// amount the activation condition earns (TLC_366/600/816), 0 when
/// inactive. Called from the cost pipeline BEFORE the play path pushes the
/// card's own type, so the check counts earlier same-type cards only
/// (>= 1).
#[must_use]
pub fn kindred_cost_discount(state: &GameState, card: Entity, player: PlayerId) -> i32 {
    let Some(card_id) = state.world().card_id(card) else {
        return 0;
    };
    let Some(KindredEffect::CostDiscount { amount }) = kindred_effect(card_id.0) else {
        return 0;
    };
    let Some(my_type) = kindred_type(card_id.0) else {
        return 0;
    };
    if kindred_active(state, player, my_type, 1) {
        *amount
    } else {
        0
    }
}

/// Resolves a played card's Kindred OnPlay effect (M2-W3) when the
/// activation condition holds, applying TLC_251's "triggers twice" flag.
///
/// Called from the play path — for minions in the CardPlayed handler after
/// the card enters play (after the base resolution, before the enqueued
/// MinionSummoned battlecry — W3 decision: none of the OnPlay minions'
/// kindred effects interact with their battlecries, see §16), for spells
/// after the base spell effect — AFTER the played-type push, so the
/// activation check counts the card itself plus any earlier same-type card
/// (>= 2; a card never counts itself — the push is the only way a card
/// enters the list, so effect-summoned copies of a Kindred card do NOT
/// re-fire, breaking the Conjured Bookkeeper copy loop).
pub fn resolve_on_play(
    state: &mut GameState,
    queue: &mut EventQueue,
    card: Entity,
    player: PlayerId,
) {
    let Some(card_id) = state.world().card_id(card) else {
        return;
    };
    let Some(KindredEffect::OnPlay { effect }) = kindred_effect(card_id.0) else {
        return;
    };
    let Some(my_type) = kindred_type(card_id.0) else {
        return;
    };
    if !kindred_active(state, player, my_type, 2) {
        return;
    }
    let twice = state.player(player).next_kindred_twice;
    if twice {
        state.make_mut().players[player.index()].next_kindred_twice = false;
    }
    crate::engine::trigger::resolve_effect(state, queue, card, player, *effect, None, None);
    if twice {
        crate::engine::trigger::resolve_effect(state, queue, card, player, *effect, None, None);
    }
}

/// Applies a Kindred battlecry modifier (M2-W3) to a battlecry about to
/// resolve: returns `(effect, extra)` — `effect` is what the battlecry
/// resolves (the modifier's effect instead of the base when active and
/// `replace`, the base otherwise), `extra` is the add-on effect to resolve
/// after the battlecry (Some only when active and the modifier is an
/// add-on). The activation check counts the played list (>= 2 — the
/// card's own push happened at the CardPlayed path, before the enqueued
/// MinionSummoned battlecry).
#[must_use]
pub fn apply_battlecry_modifier(
    state: &GameState,
    card: Entity,
    player: PlayerId,
    base: CardEffect,
) -> (CardEffect, Option<CardEffect>) {
    let Some(card_id) = state.world().card_id(card) else {
        return (base, None);
    };
    let Some(KindredEffect::BattlecryModifier { effect, replace }) = kindred_effect(card_id.0)
    else {
        return (base, None);
    };
    let Some(my_type) = kindred_type(card_id.0) else {
        return (base, None);
    };
    if !kindred_active(state, player, my_type, 2) {
        return (base, None);
    }
    if *replace {
        (*effect, None)
    } else {
        (base, Some(*effect))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::def::card_by_id;

    /// The 27 Kindred cards (23 W3 + 4 W4c DINO, from the official card
    /// dump) — every card with Kindred text has a `kindred_type` entry,
    /// and the registry count matches `KINDRED_CARD_IDS`.
    #[test]
    fn kindred_type_table_is_complete() {
        for id in KINDRED_CARD_IDS {
            assert!(kindred_type(id).is_some(), "missing kindred_type for {id}");
        }
        assert_eq!(KINDRED_CARD_IDS.len(), 27);
    }

    /// The registry's type agrees with the CardDef the play path pushes
    /// (spell → Spell; minion → its CardDef race — the primary race of the
    /// multi-race cards).
    #[test]
    fn kindred_type_matches_the_card_defs() {
        for &id in KINDRED_CARD_IDS {
            let def = card_by_id(id).unwrap_or_else(|| panic!("no CardDef for {id}"));
            assert_eq!(
                played_type_of(def),
                kindred_type(id),
                "kindred type mismatch for {id}"
            );
        }
    }

    /// `kindred_effect` covers every Kindred card except the five whose
    /// effect is folded into a dedicated battlecry variant or a player
    /// flag (TLC_102 special battlecry, TLC_223/236/432 drawn-card
    /// modifiers, TLC_251 flag) — 22 of the 27 cards carry a registry
    /// effect.
    #[test]
    fn kindred_effect_table_is_complete() {
        let registered = KINDRED_CARD_IDS
            .iter()
            .filter(|id| kindred_effect(id).is_some())
            .count();
        assert_eq!(registered, 22, "22 of the 27 cards carry a registry effect");
        for id in ["TLC_102", "TLC_223", "TLC_236", "TLC_251", "TLC_432"] {
            assert!(kindred_effect(id).is_none(), "{id} has a registry entry");
        }
    }
}
