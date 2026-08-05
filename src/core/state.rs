//! GameState — immutable game state + Copy-on-Write.
//!
//! `GameState` wraps the actual game data in an `Arc<Inner>`.
//! Cloning is an O(1) reference-count bump, so branching (e.g. MCTS search) is nearly free.
//! The first mutation clones via `Arc::make_mut` (only when the reference is shared);
//! when not shared, mutations happen in place.
//!
//! # Example
//!
//! ```rust
//! use orange_stone::core::state::GameState;
//!
//! let mut parent = GameState::new();
//! let mut branch = parent.clone();  // Arc refcount bump
//! // Mutating branch triggers CoW automatically; parent is unaffected
//! ```
use serde::{Deserialize, Serialize};

use crate::core::component::{Attack, AttacksUsed, CardType, Cost, Health};
use crate::core::player::{Player, PlayerId};
use crate::core::world::World;
use crate::core::zone::Zone;
use crate::sim::rng::GameRng;
use std::sync::Arc;

/// Game resolution step — the GameStep state machine (RS/SB analogue, roadmap G1).
///
/// Events process within the current step; when the event queue drains,
/// `rules::advance_step` moves the machine to the next step. The turn start
/// sequence runs StartTriggers → ManaRefill → DrawStep → Main (start-of-turn
/// secrets/triggers fire before the mana refill and the draw); the end
/// sequence runs EndTriggers → WrapUp (end-of-turn effects resolve at full
/// strength, then "until end of turn" effects expire). The Death step batch-
/// processes pending deaths (Milestone G3 marks them; G1 enters it for queued
/// `MinionDied` events).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    /// Start-of-turn triggers for the active player (fire before the mana refill and the draw)
    StartTriggers,
    /// Gain a mana crystal and refill mana (overload locks apply here, Milestone F1)
    ManaRefill,
    /// Draw the turn's card (the first player skips this on turn 1)
    DrawStep,
    /// Main step — the player can play cards, attack, and end the turn
    Main,
    /// End-of-turn triggers ("at the end of your turn" effects fire here)
    EndTriggers,
    /// Turn wrap-up: expire "until end of turn" effects and clear per-turn state
    WrapUp,
    /// Death step: batch-process pending deaths
    Death,
    /// Game over
    GameOver {
        /// The winner
        winner: PlayerId,
    },
}

/// Internal data of the game state — shared via `Arc`.
///
/// Contains the full World, player info, and game metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inner {
    /// The ECS World (entities and components)
    pub world: World,
    /// The two players' state
    pub players: [Player; 2],
    /// Current turn number (counts from 1, incremented on each TurnStarted)
    pub turn: u32,
    /// Current game phase
    pub step: Step,
    /// The player whose turn it is
    pub active_player: PlayerId,
    /// Random number generator (reproducible)
    pub rng: GameRng,
}

/// Immutable game state with Copy-on-Write support.
///
/// Clone is O(1). Mutation triggers CoW through the internal `Arc::make_mut`:
/// - single reference → mutate in place
/// - shared by multiple references → clone Inner, then mutate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    inner: Arc<Inner>,
}

impl GameState {
    /// Create a new initial game state.
    ///
    /// Contains both players and their hero entities (30 HP, 0 Attack).
    /// The first player starts their opening turn with 1 mana crystal; the
    /// second player starts at 0 (their turn-1 refill runs via the step machine).
    /// The deck starts empty (filled via GameBuilder).
    /// The game starts at Player1's turn 1, step Main.
    /// The RNG seed is fixed at 12345.
    #[must_use]
    pub fn new() -> Self {
        let mut world = World::new();

        // Create hero entities
        let hero1 = world.spawn();
        world.set_health(hero1, Health(30));
        world.set_attack(hero1, Attack(0));
        world.set_cost(hero1, Cost(0));
        world.set_card_type(hero1, CardType::Hero);
        world.set_player(hero1, PlayerId::Player1);
        world.set_attacks_used(hero1, AttacksUsed(0));
        world.set_zone(hero1, Zone::Play);
        world
            .zones_mut()
            .insert(Zone::Play, PlayerId::Player1, hero1);

        let hero2 = world.spawn();
        world.set_health(hero2, Health(30));
        world.set_attack(hero2, Attack(0));
        world.set_cost(hero2, Cost(0));
        world.set_card_type(hero2, CardType::Hero);
        world.set_player(hero2, PlayerId::Player2);
        world.set_attacks_used(hero2, AttacksUsed(0));
        world.set_zone(hero2, Zone::Play);
        world
            .zones_mut()
            .insert(Zone::Play, PlayerId::Player2, hero2);

        let inner = Inner {
            world,
            players: [
                // The first player's opening turn starts with 1 mana crystal
                // (their turn-1 refill; the step machine's ManaRefill step only
                // runs for turns entered via a TurnStarted event).
                Player::new(PlayerId::Player1, hero1, 1),
                Player::new(PlayerId::Player2, hero2, 0),
            ],
            turn: 1,
            step: Step::Main,
            active_player: PlayerId::Player1,
            rng: GameRng::new(12345),
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    /// Get a read-only reference to the World (shared access, no overhead).
    #[must_use]
    pub fn world(&self) -> &World {
        &self.inner.world
    }

    /// Get the state of the given player.
    #[must_use]
    pub fn player(&self, id: PlayerId) -> &Player {
        &self.inner.players[id.index()]
    }

    /// Get the current game phase.
    #[must_use]
    pub fn step(&self) -> Step {
        self.inner.step
    }

    /// Set the game phase.
    /// Requires a mutable reference obtained via `make_mut` (called by `GameEngine` or `GameBuilder`).
    pub fn set_step(&mut self, step: Step) {
        let inner = self.make_mut();
        inner.step = step;
    }

    /// Get the current active player.
    #[must_use]
    pub fn active_player(&self) -> PlayerId {
        self.inner.active_player
    }

    /// Set the current active player.
    pub fn set_active_player(&mut self, player: PlayerId) {
        self.make_mut().active_player = player;
    }

    /// Get the current turn number.
    #[must_use]
    pub fn turn(&self) -> u32 {
        self.inner.turn
    }

    /// Set the current turn number.
    pub fn set_turn(&mut self, turn: u32) {
        self.make_mut().turn = turn;
    }

    /// Get a read-only reference to the RNG.
    #[must_use]
    pub fn rng(&self) -> &GameRng {
        &self.inner.rng
    }

    /// Get a mutable reference to the RNG (triggers CoW).
    #[must_use]
    pub fn rng_mut(&mut self) -> &mut GameRng {
        &mut self.make_mut().rng
    }

    /// Get a mutable reference to Inner, triggering CoW.
    ///
    /// If the `Arc` is shared (`strong_count > 1`), `Arc::make_mut` clones the entire
    /// `Inner` and returns an exclusive reference to the clone.
    /// If there is only one reference, the in-place data is returned directly.
    ///
    /// This method is the core of CoW. Callers should:
    /// - batch multiple changes for one event handling under a single `make_mut` call
    /// - finish all read-only operations before calling `make_mut` (to avoid borrow conflicts)
    #[must_use]
    pub fn make_mut(&mut self) -> &mut Inner {
        Arc::make_mut(&mut self.inner)
    }

    /// Get a mutable reference to the World (convenience method, triggers CoW).
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.make_mut().world
    }

    /// Returns the current Inner's Arc reference count (for tests and debugging).
    #[cfg(test)]
    #[must_use]
    fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Get a read-only reference to the shared Inner (for internal comparisons and tests).
    #[cfg(test)]
    fn inner_ref(&self) -> &Inner {
        &self.inner
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    /// Serialize to compact binary (bincode) — state transfer / checkpoints for distributed training.
    ///
    /// After deserialization the state is fully equivalent to the original (including the RNG state
    /// and event log), and the game can continue with identical results.
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(&self.inner)
    }

    /// Restore a game state by deserializing from bincode bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        Ok(Self {
            inner: Arc::new(bincode::deserialize(bytes)?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_has_two_heroes() {
        let state = GameState::new();
        let world = state.world();
        let hero1 = state.player(PlayerId::Player1).hero;
        let hero2 = state.player(PlayerId::Player2).hero;
        assert_eq!(world.health(hero1), Some(Health(30)));
        assert_eq!(world.health(hero2), Some(Health(30)));
        assert_eq!(world.attack(hero1), Some(Attack(0)));
        assert_eq!(world.card_type(hero1), Some(CardType::Hero));
    }

    #[test]
    fn new_state_initial_values() {
        let state = GameState::new();
        assert_eq!(state.turn(), 1);
        assert_eq!(state.step(), Step::Main);
        assert_eq!(state.active_player(), PlayerId::Player1);
    }

    #[test]
    fn clone_is_independent() {
        let a = GameState::new();
        let mut b = a.clone();

        // Both share the same Arc; refcount is 2 for each
        assert_eq!(a.ref_count(), 2);
        assert_eq!(b.ref_count(), 2);

        // Mutate b
        b.set_turn(5);
        assert_eq!(b.turn(), 5);
        // a is unaffected
        assert_eq!(a.turn(), 1);

        // Mutate b again
        b.set_turn(10);
        assert_eq!(b.turn(), 10);
    }

    #[test]
    fn clone_then_modify_parent_still_independent() {
        let mut parent = GameState::new();
        let mut child = parent.clone();

        parent.set_turn(3);
        child.set_turn(7);

        assert_eq!(parent.turn(), 3);
        assert_eq!(child.turn(), 7);
    }

    #[test]
    fn siblings_diverge_independently() {
        let mut parent = GameState::new();
        let mut sibling_a = parent.clone();
        let mut sibling_b = parent.clone();

        sibling_a.set_turn(2);
        sibling_b.set_turn(3);
        parent.set_turn(4);

        assert_eq!(sibling_a.turn(), 2);
        assert_eq!(sibling_b.turn(), 3);
        assert_eq!(parent.turn(), 4);
    }

    #[test]
    fn second_mutation_no_reclone() {
        // With an exclusive reference there must be no re-clone
        let mut state = GameState::new();
        assert_eq!(state.ref_count(), 1);

        // First mutation
        let inner_ptr_before: *const Inner = state.inner_ref();
        state.set_turn(2);
        let inner_ptr_after: *const Inner = state.inner_ref();
        // With an exclusive reference, Arc::make_mut must not allocate new memory
        assert_eq!(inner_ptr_before, inner_ptr_after);

        // Second mutation
        state.set_turn(3);
        let inner_ptr_final: *const Inner = state.inner_ref();
        assert_eq!(inner_ptr_after, inner_ptr_final);
    }

    #[test]
    fn world_mutation_through_state() {
        let mut state = GameState::new();
        let hero = state.player(PlayerId::Player1).hero;
        state.world_mut().set_health(hero, Health(25));
        assert_eq!(state.world().health(hero), Some(Health(25)));
    }

    #[test]
    fn clone_preserves_world_snapshot() {
        let mut state = GameState::new();
        let hero = state.player(PlayerId::Player1).hero;
        state.world_mut().set_health(hero, Health(20));

        // Clone a snapshot
        let snapshot = state.clone();

        // Keep mutating the original state
        state.world_mut().set_health(hero, Health(10));

        // The snapshot is unaffected
        assert_eq!(state.world().health(hero), Some(Health(10)));
        assert_eq!(snapshot.world().health(hero), Some(Health(20)));
    }
}

#[cfg(test)]
mod serialization_tests {
    use super::*;
    use crate::core::action::Action;
    use crate::core::component::{Attack, Health};
    use crate::engine::game::GameEngine;
    use crate::sim::game::GameBuilder;

    /// Build a complex state: minions, auras, armor, mana, weapon.
    fn complex_state() -> GameState {
        use crate::cards::def::{BLOODFEN_RAPTOR, GLADIATORS_LONGBOW, STORMWIND_CHAMPION};
        let mut builder = GameBuilder::new();
        builder.set_mana(PlayerId::Player1, 5, 5);
        builder.set_mana(PlayerId::Player2, 4, 4);
        builder.hero_health(PlayerId::Player1, 25);
        builder.add_minion_to_board(PlayerId::Player1, &STORMWIND_CHAMPION);
        builder.add_minion_to_board(PlayerId::Player1, &BLOODFEN_RAPTOR);
        builder.add_minion_to_board(PlayerId::Player2, &BLOODFEN_RAPTOR);
        builder.add_minion_to_hand(PlayerId::Player1, &BLOODFEN_RAPTOR);
        builder.equip_weapon(PlayerId::Player1, &GLADIATORS_LONGBOW);
        builder.build()
    }

    #[test]
    fn roundtrip_preserves_state() {
        let state = complex_state();
        let bytes = state.to_bytes().expect("serialize");
        let restored = GameState::from_bytes(&bytes).expect("deserialize");

        // Metadata
        assert_eq!(restored.turn(), state.turn());
        assert_eq!(restored.step(), state.step());
        assert_eq!(restored.active_player(), state.active_player());
        // Player state
        for pid in [PlayerId::Player1, PlayerId::Player2] {
            let a = state.player(pid);
            let b = restored.player(pid);
            assert_eq!(a.mana_crystals, b.mana_crystals);
            assert_eq!(a.current_mana, b.current_mana);
            assert_eq!(a.armor, b.armor);
            assert_eq!(a.weapon.is_some(), b.weapon.is_some());
        }
        // World state: hero health, minions, auras
        let wa = state.world();
        let wb = restored.world();
        assert_eq!(
            wb.health(restored.player(PlayerId::Player1).hero),
            wa.health(state.player(PlayerId::Player1).hero)
        );
        // The aura minion (Stormwind Champion) is still on the board and its effect is queryable
        let champion = wb
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .find(|&e| wb.card_id(e).is_some_and(|c| c.0 == "NEUTRAL_T10"))
            .expect("champion on board");
        assert!(wb.aura(champion).is_some());
        // The aura effect still applies (the index is serialized with the state)
        let raptor = wb
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .find(|&e| wb.card_id(e).is_some_and(|c| c.0 == "CLASSIC_001"))
            .expect("raptor on board");
        assert!(
            wb.effective_attack(raptor).map_or(0, |a| a.0) > 3,
            "aura buff must survive serialization"
        );
    }

    #[test]
    fn roundtrip_continues_identically() {
        // Continue the game after serialization; the event log must match the original state exactly
        let engine = GameEngine::new();
        let mut a = complex_state();
        let mut b = GameState::from_bytes(&a.to_bytes().unwrap()).unwrap();

        let hero = a.player(PlayerId::Player1).hero;
        let defender = a
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player2)
            .find(|&e| a.world().card_type(e) == Some(crate::core::component::CardType::Minion))
            .expect("enemy minion");

        let log_a = engine
            .apply(
                &mut a,
                Action::Attack {
                    attacker: hero,
                    defender,
                },
            )
            .unwrap();
        let log_b = engine
            .apply(
                &mut b,
                Action::Attack {
                    attacker: hero,
                    defender,
                },
            )
            .unwrap();
        assert_eq!(log_a, log_b, "identical event logs after restore");
        assert_eq!(a.turn(), b.turn());
        assert_eq!(
            a.world().health(defender),
            b.world().health(defender),
            "damage identical after restore"
        );
    }

    #[test]
    fn corrupted_bytes_rejected() {
        let state = complex_state();
        let mut bytes = state.to_bytes().unwrap();
        // Corrupt the data (the first length field)
        bytes[0] ^= 0xFF;
        assert!(
            GameState::from_bytes(&bytes).is_err(),
            "corrupt data must fail"
        );
    }

    #[test]
    fn rng_state_preserved() {
        // Identical subsequent random call sequences
        let a = GameState::new();
        let b = GameState::from_bytes(&a.to_bytes().unwrap()).unwrap();
        let mut a = a;
        let mut b = b;
        for _ in 0..10 {
            assert_eq!(a.rng_mut().next_u32(), b.rng_mut().next_u32());
        }
    }

    // silence unused import warning safety
    #[allow(dead_code)]
    fn _unused(_: Health, _: Attack) {}
}

#[cfg(test)]
mod cow_sharing_tests {
    use super::*;
    use crate::cards::def::BLOODFEN_RAPTOR;
    use crate::core::component::Health;
    use crate::sim::game::GameBuilder;

    #[test]
    fn game_state_clone_shares_world_pages() {
        // After cloning GameState, the World's component pages are shared via Arc (structural sharing, not deep copy)
        let mut builder = GameBuilder::new();
        for _ in 0..10 {
            builder.add_minion_to_board(PlayerId::Player1, &BLOODFEN_RAPTOR);
        }
        let a = builder.build();
        let mut b = a.clone();

        // Both share the World data — mutating b does not affect a (copy-on-write)
        let hero = b.player(PlayerId::Player1).hero;
        {
            let inner = b.make_mut();
            inner.world.set_health(hero, Health(20));
        }
        assert_eq!(
            a.world().health(hero),
            Some(Health(30)),
            "clone must not see the write"
        );
        assert_eq!(b.world().health(hero), Some(Health(20)));
        // a still has all 10 minions
        let minions: Vec<_> = a
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .filter(|&e| a.world().card_type(e) == Some(crate::core::component::CardType::Minion))
            .collect();
        assert_eq!(minions.len(), 10);
    }
}
