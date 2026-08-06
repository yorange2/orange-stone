//! Player definitions — PlayerId and Player state.
//!
//! Each player has a `PlayerId` (mounted on entities via a component)
//! and a `Player` struct (stored in `GameState`).
use serde::{Deserialize, Serialize};

/// Player identifier.
///
/// Uses `#[repr(u8)]` so arrays (`[T; 2]`) can be indexed efficiently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PlayerId {
    /// The first player
    Player1 = 0,
    /// The second player
    Player2 = 1,
}

impl PlayerId {
    /// Number of players.
    pub const COUNT: usize = 2;

    /// Returns the `usize` index for array subscripting.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Returns the opponent's `PlayerId`.
    #[must_use]
    pub const fn opponent(self) -> Self {
        match self {
            Self::Player1 => Self::Player2,
            Self::Player2 => Self::Player1,
        }
    }
}

/// Player state — player data that is not at the entity level.
///
/// The hero itself is an entity (`CardType::Hero`) stored in the World.
/// `Player` holds a reference to the hero entity plus state such as mana crystals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// The player ID
    pub id: PlayerId,
    /// Handle to the hero entity
    pub hero: crate::core::entity::Entity,
    /// Total mana crystals (max 10, +1 at the start of each turn)
    pub mana_crystals: i32,
    /// Current available mana (spent when playing cards, refilled at turn start)
    pub current_mana: i32,
    /// The currently equipped weapon entity (`None` means no weapon)
    pub weapon: Option<crate::core::entity::Entity>,
    /// Hero armor
    pub armor: i32,
    /// Fatigue counter (1-based): the damage dealt by the next empty-deck draw
    /// attempt (official HS rule — docs/fatigue-roadmap.md). Starts at 1 (the
    /// first hit deals 1) and increments by 1 after each fatigue hit.
    pub fatigue: u32,
    /// Cards played this turn (for the Combo mechanic)
    pub cards_played_this_turn: u32,
    /// Minions played this turn (Pint-Sized Summoner — the first minion
    /// each turn costs less)
    pub minions_played_this_turn: u8,
    /// Mana locked by overload cards for the next turn (roadmap F1)
    pub overload_locked: i32,
    /// Friendly minions that died this turn (for resurrection effects)
    pub died_this_turn: Vec<crate::core::entity::Entity>,
    /// The next secret costs 0 (Kirin Tor Mage, one-time)
    pub next_secret_free: bool,
    /// All spells cost 0 this turn (Millhouse Manastorm — the opponent's
    /// spells cost 0 next turn); cleared at the owner's turn start
    pub spells_cost_zero: bool,
    /// Enemy minions temporarily controlled (entity, original owner) — returned at end of turn (Shadow Madness)
    pub controlled_this_turn: Vec<(crate::core::entity::Entity, PlayerId)>,
    /// Corrupted enemy minions — destroyed at the owner's turn start (Corruption)
    pub corrupted: Vec<crate::core::entity::Entity>,
    /// Minimum minion health this turn (Commanding Shout, 0 means no limit)
    pub minion_min_health: i32,
}

impl Player {
    /// Create a new player state.
    #[must_use]
    pub const fn new(id: PlayerId, hero: crate::core::entity::Entity, mana_crystals: i32) -> Self {
        Self {
            id,
            hero,
            mana_crystals,
            current_mana: mana_crystals,
            weapon: None,
            armor: 0,
            fatigue: 1,
            cards_played_this_turn: 0,
            minions_played_this_turn: 0,
            overload_locked: 0,
            died_this_turn: Vec::new(),
            next_secret_free: false,
            spells_cost_zero: false,
            controlled_this_turn: Vec::new(),
            corrupted: Vec::new(),
            minion_min_health: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_id_index() {
        assert_eq!(PlayerId::Player1.index(), 0);
        assert_eq!(PlayerId::Player2.index(), 1);
    }

    #[test]
    fn player_id_opponent() {
        assert_eq!(PlayerId::Player1.opponent(), PlayerId::Player2);
        assert_eq!(PlayerId::Player2.opponent(), PlayerId::Player1);
    }
}
