//! Zone system — where cards are located in the game.
//!
//! Hearthstone has 5 zones: Deck, Hand, Play,
//! Graveyard, and SetAside.
//! Each zone maintains an ordered entity list (order matters in hand and on the battlefield).
use serde::{Deserialize, Serialize};

use crate::core::player::PlayerId;

/// Zone type.
///
/// An entity can be in only one zone at a time. Zone transfers go through `World::move_to_zone`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Zone {
    /// Deck — where cards start; draw logic arrives in Phase 2+
    Deck,
    /// Hand — where playable cards are held
    Hand,
    /// Play — the zone where minions and heroes are
    Play,
    /// Graveyard — where dead minions/used spells go
    Graveyard,
    /// SetAside — temporarily removed from the game (used for secrets in Phase 2+)
    SetAside,
}

/// Zone move error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneError {
    /// The entity has been destroyed (generation mismatch)
    EntityGone,
    /// The PlayerId component needed to determine the owner is missing
    MissingPlayer,
    /// The current Zone component is missing (should not happen; indicates inconsistent state)
    MissingZone,
}

/// Entity lists for all zones.
///
/// Deck, hand, play, and graveyard are maintained per player; SetAside is shared.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Zones {
    deck: [Vec<crate::core::entity::Entity>; 2],
    hand: [Vec<crate::core::entity::Entity>; 2],
    play: [Vec<crate::core::entity::Entity>; 2],
    graveyard: [Vec<crate::core::entity::Entity>; 2],
    set_aside: Vec<crate::core::entity::Entity>,
}

impl Zones {
    /// Create an empty zone table.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            deck: [const { Vec::new() }, const { Vec::new() }],
            hand: [const { Vec::new() }, const { Vec::new() }],
            play: [const { Vec::new() }, const { Vec::new() }],
            graveyard: [const { Vec::new() }, const { Vec::new() }],
            set_aside: Vec::new(),
        }
    }

    /// Get a mutable reference to a zone.
    fn vec_mut(&mut self, zone: Zone, player: PlayerId) -> &mut Vec<crate::core::entity::Entity> {
        match zone {
            Zone::Deck => &mut self.deck[player.index()],
            Zone::Hand => &mut self.hand[player.index()],
            Zone::Play => &mut self.play[player.index()],
            Zone::Graveyard => &mut self.graveyard[player.index()],
            Zone::SetAside => &mut self.set_aside,
        }
    }

    /// Get a read-only reference to a zone.
    fn vec_ref(&self, zone: Zone, player: PlayerId) -> &[crate::core::entity::Entity] {
        match zone {
            Zone::Deck => &self.deck[player.index()],
            Zone::Hand => &self.hand[player.index()],
            Zone::Play => &self.play[player.index()],
            Zone::Graveyard => &self.graveyard[player.index()],
            Zone::SetAside => &self.set_aside,
        }
    }

    /// Insert an entity at the end of the given zone.
    pub fn insert(&mut self, zone: Zone, player: PlayerId, entity: crate::core::entity::Entity) {
        self.vec_mut(zone, player).push(entity);
    }

    /// Remove an entity from the given zone (preserving order).
    ///
    /// Uses `Vec::remove` (not swap-remove) to preserve hand/battlefield order.
    pub fn remove(&mut self, zone: Zone, player: PlayerId, entity: crate::core::entity::Entity) {
        let vec = self.vec_mut(zone, player);
        if let Some(pos) = vec.iter().position(|&e| e == entity) {
            vec.remove(pos);
        }
    }

    /// Try to remove an entity from all zones (used for despawn).
    ///
    /// Returns `true` if the entity was actually removed.
    pub fn remove_from_all(&mut self, entity: crate::core::entity::Entity) -> bool {
        // Use indices to avoid borrow-checker issues (multiple mutable borrows cannot coexist in one array literal)
        let mut removed = false;
        for player_idx in 0..2usize {
            if let Some(pos) = self.deck[player_idx].iter().position(|&e| e == entity) {
                self.deck[player_idx].remove(pos);
                removed = true;
                break;
            }
            if let Some(pos) = self.hand[player_idx].iter().position(|&e| e == entity) {
                self.hand[player_idx].remove(pos);
                removed = true;
                break;
            }
            if let Some(pos) = self.play[player_idx].iter().position(|&e| e == entity) {
                self.play[player_idx].remove(pos);
                removed = true;
                break;
            }
            if let Some(pos) = self.graveyard[player_idx].iter().position(|&e| e == entity) {
                self.graveyard[player_idx].remove(pos);
                removed = true;
                break;
            }
        }
        if !removed {
            if let Some(pos) = self.set_aside.iter().position(|&e| e == entity) {
                self.set_aside.remove(pos);
                removed = true;
            }
        }
        removed
    }

    /// Iterate over all entities in the given zone (in order).
    pub fn iter(
        &self,
        zone: Zone,
        player: PlayerId,
    ) -> impl Iterator<Item = crate::core::entity::Entity> + '_ {
        self.vec_ref(zone, player).iter().copied()
    }

    /// Returns the number of entities in the given zone.
    #[must_use]
    pub fn len(&self, zone: Zone, player: PlayerId) -> usize {
        self.vec_ref(zone, player).len()
    }

    /// Returns `true` if the given zone is empty.
    #[must_use]
    pub fn is_empty(&self, zone: Zone, player: PlayerId) -> bool {
        self.len(zone, player) == 0
    }

    /// Returns all entities in the given zone (an ordered copy).
    #[must_use]
    pub fn entities(&self, zone: Zone, player: PlayerId) -> Vec<crate::core::entity::Entity> {
        self.vec_ref(zone, player).to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity::Entity;

    fn e(idx: u32) -> Entity {
        Entity::new(idx, 0)
    }

    #[test]
    fn insert_and_iter_hand() {
        let mut zones = Zones::new();
        zones.insert(Zone::Hand, PlayerId::Player1, e(1));
        zones.insert(Zone::Hand, PlayerId::Player1, e(2));
        let hand: Vec<_> = zones.iter(Zone::Hand, PlayerId::Player1).collect();
        assert_eq!(hand, vec![e(1), e(2)]);
    }

    #[test]
    fn remove_preserves_order() {
        let mut zones = Zones::new();
        zones.insert(Zone::Play, PlayerId::Player1, e(10));
        zones.insert(Zone::Play, PlayerId::Player1, e(20));
        zones.insert(Zone::Play, PlayerId::Player1, e(30));
        zones.remove(Zone::Play, PlayerId::Player1, e(20));
        let play: Vec<_> = zones.iter(Zone::Play, PlayerId::Player1).collect();
        assert_eq!(play, vec![e(10), e(30)]);
    }

    #[test]
    fn zones_are_per_player() {
        let mut zones = Zones::new();
        zones.insert(Zone::Hand, PlayerId::Player1, e(1));
        zones.insert(Zone::Hand, PlayerId::Player2, e(2));
        assert!(zones.iter(Zone::Hand, PlayerId::Player1).eq([e(1)]));
        assert!(zones.iter(Zone::Hand, PlayerId::Player2).eq([e(2)]));
    }

    #[test]
    fn remove_from_all_finds_entity() {
        let mut zones = Zones::new();
        zones.insert(Zone::Play, PlayerId::Player1, e(42));
        assert!(zones.remove_from_all(e(42)));
        assert!(zones.is_empty(Zone::Play, PlayerId::Player1));
    }

    #[test]
    fn remove_from_all_missing() {
        let mut zones = Zones::new();
        assert!(!zones.remove_from_all(e(99)));
    }
}
