//! Action — operations a player can perform.
//!
//! Each Action represents a decision a player can make in the game.
//! Actions are submitted to `GameEngine::apply`, which validates and executes them.

use crate::core::entity::Entity;

/// A player action — a legal operation in the game.
///
/// Phase 1 supports: playing cards (vanilla minions), attacking, and ending the turn.
/// HeroPower returns `EngineError::Unimplemented` in Phase 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Play a card (from hand to the battlefield).
    PlayCard {
        /// The card entity to play
        card: Entity,
        /// Explicit target (the character targeted by a battlecry/spell effect).
        ///
        /// With `Some(target)` the effect applies to that target if it is in the
        /// target set allowed by the effect; with `None` the engine chooses randomly
        /// (self-play fallback). Ignored for multi-target effects (AOE, etc.).
        target: Option<Entity>,
        /// Summon position on the board (0 = leftmost), when playing a minion.
        ///
        /// With `None` the minion is summoned at the rightmost position.
        position: Option<u8>,
    },
    /// Attack an enemy minion/hero with your own minion/hero.
    Attack {
        /// The attacking entity
        attacker: Entity,
        /// The defending entity (enemy minion or enemy hero)
        defender: Entity,
    },
    /// End the current player's turn.
    EndTurn,
    /// Use the hero power.
    HeroPower {
        /// The hero entity using the power
        hero: Entity,
        /// Explicit hero power target (roadmap G6); `None` = engine random
        target: Option<Entity>,
    },
    /// Resolve a pending choice (roadmap G6) — the choice surfaced by
    /// `GameEngine::apply_choices` (Choose One branches, Discover picks, …).
    Choose {
        /// The pending choice's id (echoed back from the request)
        choice_id: u64,
        /// The chosen option index
        option: u8,
    },
    /// Trade a Tradeable hand card (Core Set W2): spend 1 mana, shuffle the
    /// card into the deck, draw a card.
    TradeCard {
        /// The Tradeable card in hand
        card: Entity,
    },
}
