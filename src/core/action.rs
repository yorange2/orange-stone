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
    /// Play a minion card (from hand to the battlefield).
    PlayCard {
        /// The card entity to play
        card: Entity,
        /// Explicit target (the character targeted by a battlecry/spell effect).
        ///
        /// With `Some(target)` the effect applies to that target if it is in the
        /// target set allowed by the effect; with `None` the engine chooses randomly
        /// (self-play fallback). Ignored for multi-target effects (AOE, etc.).
        target: Option<Entity>,
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
    /// Use the hero power (implemented in Phase 2+).
    HeroPower {
        /// The hero entity using the power
        hero: Entity,
    },
}
