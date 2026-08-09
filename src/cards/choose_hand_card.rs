//! Choose-a-card-in-hand dispatch (2025–2026 expansions M4-W4 — the
//! Cataclysm closing wave, §26): the nine W4 cards whose battlecry/location
//! text reads "Choose a card in your hand" (CATA_200 Agent of the Old Ones,
//! CATA_209 Battlefield Blaster, CATA_477 Chamber of Aspects, CATA_490
//! Ocular Occultist, CATA_563 Crackling Cloudstrider, CATA_566 Tol'vir
//! Carver, CATA_697 Malevolent Mutant, CATA_721 Sheltered Survivor,
//! CATA_979 Conjuration Specialist).
//!
//! # Shape
//!
//! `CardEffect::ChooseHandCard` resolves by storing the pending
//! [`ChooseHandCardKind`] on the player record and surfacing a
//! `ChoiceKind::ChooseHandCard` pending choice whose options are the
//! player's hand card ids (filtered per card — the official texts restrict
//! the choice to spells / minions / Fel spells / spells costing (4) or
//! less). The ChoiceResolved handler picks the entity and calls
//! `engine::trigger::resolve_choose_hand_card`, which applies the pending
//! kind. The player field rides between the effect resolution and the
//! choice resolution (a choice may surface across engine steps); a stale
//! pending kind is overwritten by the next ChooseHandCard resolution and
//! is inert otherwise.
//!
//! Registered approximations (§26): CATA_566's per-turn reduction reuses
//! the `TurnCostReducer` marker (the Circadiamancer convention); CATA_721
//! shuffles a COPY into the deck (the picked entity itself is removed —
//! the official "shuffle it into your deck" keeps the same entity, which
//! the engine approximates by a fresh copy); CATA_979's "split" is two
//! random same-Cost spells (the §26 random simplification).

use crate::core::player::PlayerId;
use crate::core::state::GameState;
use crate::core::zone::Zone;
use serde::{Deserialize, Serialize};

/// The pending action applied to the card picked from a
/// `ChoiceKind::ChooseHandCard` choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChooseHandCardKind {
    /// CATA_200 Agent of the Old Ones — "Choose a card in your hand to
    /// transform into a Coin." Any card qualifies; the picked card
    /// transforms into THE_COIN (the GAME_005 def).
    TransformToCoin,
    /// CATA_209 Battlefield Blaster — "Choose a spell in your hand to
    /// give Spell Damage +1." Spells only; the picked spell gains
    /// Spell Damage +1 (stacking).
    GrantSpellDamage1,
    /// CATA_477 Chamber of Aspects (location) — "Choose a minion in your
    /// hand. Give it +2/+2." Minions only; a permanent +2/+2 enchantment.
    GrantStats {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// CATA_490 Ocular Occultist — "Choose a card in your hand to
    /// discard." Any card qualifies; the picked card is discarded (the
    /// discard funnel: FriendlyDiscarded fires, CATA_499 re-bakes).
    Discard,
    /// CATA_563 Crackling Cloudstrider — "Choose a spell in your hand
    /// that costs (4) or less to absorb. Deathrattle: Cast it." The
    /// picked spell entity rides `Player::absorbed_spell`; the
    /// deathrattle casts it (CastAbsorbedSpell).
    AbsorbSpell,
    /// CATA_566 Tol'vir Carver — "Choose a card in your hand. At the
    /// start of your turns, reduce its Cost by (1)." Any card qualifies;
    /// the picked card gets the `TurnCostReducer(1)` marker (reduced by
    /// (1) at each owner's turn start — the §26 Circadiamancer shape).
    ReduceCostEachTurn,
    /// CATA_697 Malevolent Mutant — "Choose a Fel spell in your hand.
    /// Get a copy of it." Fel spells only; a copy enters the hand.
    GetCopy,
    /// CATA_721 Sheltered Survivor — "Choose a card in your hand to
    /// shuffle into your deck. Draw a card." Any card qualifies; the
    /// picked card is removed and a fresh copy is shuffled into the
    /// deck, then a card is drawn.
    ShuffleIntoDeckDraw,
    /// CATA_979 Conjuration Specialist — "Choose a spell in your hand.
    /// Split it into two random spells of the same Cost." Spells only;
    /// the picked card is removed and two random same-Cost spells are
    /// added to the hand.
    SplitIntoTwoSameCost,
}

/// Builds the option labels for a `ChoiceKind::ChooseHandCard` choice —
/// the hand card ids the given kind may pick, in hand order (each option
/// doubles as the pool entry the ChoiceResolved handler resolves).
#[must_use]
pub fn options(state: &GameState, player: PlayerId, kind: ChooseHandCardKind) -> Vec<String> {
    use crate::core::component::CardType;
    state
        .world()
        .zones()
        .iter(Zone::Hand, player)
        .filter(|&e| match kind {
            ChooseHandCardKind::GrantSpellDamage1
            | ChooseHandCardKind::AbsorbSpell
            | ChooseHandCardKind::GetCopy
            | ChooseHandCardKind::SplitIntoTwoSameCost => {
                state.world().card_type(e) == Some(CardType::Spell)
            }
            ChooseHandCardKind::GrantStats { .. } => {
                state.world().card_type(e) == Some(CardType::Minion)
            }
            ChooseHandCardKind::TransformToCoin
            | ChooseHandCardKind::Discard
            | ChooseHandCardKind::ReduceCostEachTurn
            | ChooseHandCardKind::ShuffleIntoDeckDraw => true,
        })
        .filter(|&e| match kind {
            ChooseHandCardKind::AbsorbSpell => {
                state.world().effective_cost(e).is_some_and(|c| c.0 <= 4)
            }
            ChooseHandCardKind::GetCopy => state
                .world()
                .card_id(e)
                .and_then(|c| crate::cards::quest::spell_school(c.0))
                .is_some_and(|s| s == crate::cards::quest::SpellSchool::Fel),
            _ => true,
        })
        .filter_map(|e| state.world().card_id(e).map(|c| c.0.to_string()))
        .collect()
}
