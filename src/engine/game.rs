//! GameEngine — the orchestration layer for the action→event→resolve loop.
//!
//! `GameEngine` is a stateless unit struct responsible for:
//! 1. Calling `rules::validate` to check legality
//! 2. Calling `rules::enqueue` to generate initial events
//! 3. Looping `rules::apply_event` until the event queue is empty
//! 4. Returning the full event log (for replay and debugging)
//!
//! # Example
//!
//! ```rust
//! use orange_stone::engine::game::GameEngine;
//! use orange_stone::core::state::GameState;
//! use orange_stone::core::action::Action;
//!
//! let engine = GameEngine::new();
//! let mut state = GameState::new();
//! let result = engine.apply(&mut state, Action::EndTurn);
//! assert!(result.is_ok());
//! ```

use crate::core::action::Action;
use crate::core::event::{Event, EventQueue};
use crate::core::state::{GameState, PendingChoice};
use crate::engine::rules::{self, EngineError};
use crate::engine::secret;

/// The outcome of an engine call (roadmap G6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// The action fully resolved; the event log is the replay record
    Done(Vec<Event>),
    /// The engine needs a player choice to continue — respond with
    /// `Action::Choose { choice_id, option }`
    NeedsChoice {
        /// The pending choice
        choice: PendingChoice,
    },
}

/// Safety valve for a single action's resolution: the maximum number of events
/// one action may resolve before the engine abandons the cascade.
///
/// A normal action resolves well under a thousand events (a full board wipe
/// with deathrattles is a few hundred). A runaway interaction, however, spins
/// forever: the RL stress run over the Standard pool hit a single `step` that
/// burned 6+ minutes of CPU and 23 GB of memory inside one `apply_choices`,
/// growing the event log to hundreds of millions of entries. There is no way
/// to interrupt that from outside the engine — a Python-side watchdog cannot
/// preempt a Rust call — so the budget has to live here.
///
/// Hitting the budget is a **bug**, not a game rule; `GameState::cascade_aborts`
/// counts it so callers (and tests) can notice instead of silently accepting a
/// truncated resolution. Set `ORANGE_STONE_DEBUG_CASCADE=1` to dump the tail of
/// the event log when it trips, which is how the offending interaction gets
/// identified.
pub const MAX_EVENTS_PER_ACTION: usize = 500_000;

/// Companion budget to `MAX_EVENTS_PER_ACTION` for the *choice* dimension: how
/// many prompts one action may surface before the engine abandons the chain.
///
/// The event budget does not cover this shape — a runaway choice chain resolves
/// a small, fast `apply_choices` each round, so neither the event log nor any
/// single effect ever looks abnormal, yet the action never finishes. Choose One
/// / Discover / Choose Thrice chains are a handful of prompts in a real game.
pub const MAX_CHOICES_PER_ACTION: usize = 1_000;

/// Event count at which the diagnostic dump fires. `ORANGE_STONE_DEBUG_CASCADE`
/// lowers it so a cascade can be inspected long before it reaches the hard cap
/// (`=1` uses a 20 000-event default; `=<n>` sets it explicitly).
fn cascade_debug_at() -> usize {
    static CACHE: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *CACHE.get_or_init(|| match std::env::var("ORANGE_STONE_DEBUG_CASCADE") {
        Ok(v) => match v.trim().parse::<usize>() {
            Ok(1) => 20_000,
            Ok(n) if n > 1 => n,
            _ => MAX_EVENTS_PER_ACTION,
        },
        Err(_) => MAX_EVENTS_PER_ACTION,
    })
}

/// Dumps the tail of a runaway event log — the diagnostic that names the
/// looping interaction.
fn dump_cascade_tail(log: &[Event]) {
    eprintln!(
        "orange-stone: cascade reached {} events for one action; tail follows",
        log.len()
    );
    for event in log.iter().rev().take(40).rev() {
        eprintln!("  {event:?}");
    }
}

/// Game engine — stateless, pure logic orchestration.
#[derive(Debug, Default, Clone, Copy)]
pub struct GameEngine;

impl GameEngine {
    /// Creates a new game engine instance.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Validates, enqueues, and resolves a player action, pausing when the
    /// engine needs a player choice (roadmap G6).
    ///
    /// Returns `Resolution::Done(log)` when the action fully resolved, or
    /// `Resolution::NeedsChoice` when a pending choice must be answered with
    /// `Action::Choose` before resolution can continue.
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if validation fails; the state is **not** modified.
    pub fn apply_choices(
        &self,
        state: &mut GameState,
        action: Action,
    ) -> Result<Resolution, EngineError> {
        // 1. Validate (read-only)
        rules::validate(state, action)?;

        // 2. Action → initial events
        let mut queue = EventQueue::new();
        rules::enqueue(state, action, &mut queue)?;

        // 3. Step-driven event loop (roadmap G1/G3): events process within the
        // current step; when the queue drains, the step machine advances
        // (turn-start sequence, end-of-turn sequence, wrap-up, death step).
        let mut log = Vec::new();
        // Budget on resolution ROUNDS, not on logged events: an empty-queue
        // step-machine spin (advance_step keeps reporting progress while
        // enqueuing nothing) logs no events at all, so an event-count budget
        // never notices it.
        let mut iterations = 0_usize;
        loop {
            // A pending choice pauses resolution — unless the choice's own
            // resolution event is already in flight, or events queued before
            // the choice surfaced are still pending (they complete first:
            // equipping a Choose One weapon destroys the old one — its
            // WeaponDestroyed/deathrattle must resolve before the prompt,
            // Barbed Thorn M1-W3; CardPlayed triggers likewise).
            if let Some(choice) = state.pending_choice().cloned() {
                let resolving = matches!(queue.front(), Some(Event::ChoiceResolved { .. }));
                if !resolving && queue.is_empty() {
                    return Ok(Resolution::NeedsChoice { choice });
                }
            }
            iterations += 1;
            if iterations == cascade_debug_at() {
                eprintln!(
                    "orange-stone: resolution passed {iterations} rounds \
                     (turn {}, step {:?}, {} events logged, queue {})",
                    state.turn(),
                    state.step(),
                    log.len(),
                    if queue.is_empty() {
                        "empty"
                    } else {
                        "non-empty"
                    }
                );
                dump_cascade_tail(&log);
            }
            if iterations >= MAX_EVENTS_PER_ACTION {
                // Runaway cascade — abandon it rather than spin forever (see
                // MAX_EVENTS_PER_ACTION). The state keeps whatever resolved so
                // far; the game stays playable and the abort is counted.
                eprintln!(
                    "orange-stone: resolution aborted after {iterations} rounds \
                     ({} events, turn {}, step {:?}); set \
                     ORANGE_STONE_DEBUG_CASCADE=1 to dump the tail",
                    log.len(),
                    state.turn(),
                    state.step()
                );
                state.record_cascade_abort();
                break;
            }
            if let Some(event) = queue.pop_front() {
                // Apply the event (mutates state + may enqueue new events)
                rules::apply_event(state, event, &mut queue)?;
                // Check secret triggers after applying the event
                secret::check_secrets(state, &mut queue, &event);
                log.push(event);
            } else if !rules::advance_step(state, &mut queue) {
                // Main step with an empty queue — waiting for player input
                break;
            }
        }

        Ok(Resolution::Done(log))
    }

    /// Validates, enqueues, and fully resolves a player action with the
    /// default choice policy: pending choices resolve randomly via the
    /// embedded RNG (deterministic — the RL/self-play API).
    ///
    /// Returns the complete event log in resolution order (usable for replay).
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if validation fails; the state is **not** modified.
    pub fn apply(&self, state: &mut GameState, action: Action) -> Result<Vec<Event>, EngineError> {
        let mut log = Vec::new();
        let mut resolution = self.apply_choices(state, action)?;
        let mut choices = 0_usize;
        loop {
            match resolution {
                Resolution::Done(events) => {
                    log.extend(events);
                    return Ok(log);
                }
                Resolution::NeedsChoice { choice } => {
                    choices += 1;
                    if choices > MAX_CHOICES_PER_ACTION {
                        // Runaway choice chain: every resolution surfaces
                        // another prompt, so this action never finishes. Same
                        // safety valve as MAX_EVENTS_PER_ACTION — abandon it
                        // instead of spinning (each round is individually fast
                        // and enqueues few events, so the event budget above
                        // never trips on this shape).
                        eprintln!(
                            "orange-stone: choice chain aborted after {choices} prompts; \
                             kind={:?} repeat={} options={} pool={:?}",
                            choice.kind,
                            choice.repeat,
                            choice.options.len(),
                            choice.pool
                        );
                        state.record_cascade_abort();
                        state.clear_pending_choice();
                        return Ok(log);
                    }
                    // Default policy: random option (deterministic via the RNG)
                    let option = state.rng_mut().next_usize(choice.options.len()) as u8;
                    resolution = self.apply_choices(
                        state,
                        Action::Choose {
                            choice_id: choice.id,
                            option,
                        },
                    )?;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::action::Action;
    use crate::core::component::{Attack, AttacksUsed, CardType, Health};
    use crate::core::player::PlayerId;
    use crate::core::state::GameState;
    use crate::core::zone::Zone;

    fn add_minion_to_board(
        state: &mut GameState,
        player: PlayerId,
        atk: i32,
        hp: i32,
    ) -> crate::core::entity::Entity {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(hp));
        world.set_attack(e, Attack(atk));
        world.set_cost(e, crate::core::component::Cost(0));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, player);
        world.set_attacks_used(e, AttacksUsed(0));
        world.set_zone(e, Zone::Play);
        world.zones_mut().insert(Zone::Play, player, e);
        e
    }

    fn add_minion_to_hand(
        state: &mut GameState,
        player: PlayerId,
        atk: i32,
        hp: i32,
    ) -> crate::core::entity::Entity {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(hp));
        world.set_attack(e, Attack(atk));
        world.set_cost(e, crate::core::component::Cost(3));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, player);
        world.set_attacks_used(e, AttacksUsed(0));
        world.set_zone(e, Zone::Hand);
        world.zones_mut().insert(Zone::Hand, player, e);
        state.make_mut().players[player.index()].current_mana = 10;
        e
    }

    #[test]
    fn play_card_produces_events() {
        let engine = GameEngine::new();
        let mut state = GameState::new();
        let card = add_minion_to_hand(&mut state, PlayerId::Player1, 2, 3);

        let log = engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card,
                    target: None,
                    position: None,
                },
            )
            .unwrap();

        // Should produce CardPlayed and MinionSummoned
        assert_eq!(log.len(), 2);
        assert!(matches!(log[0], Event::CardPlayed { .. }));
        assert!(matches!(log[1], Event::MinionSummoned { .. }));

        // The card should be on the battlefield
        assert_eq!(state.world().zone(card), Some(Zone::Play));
    }

    #[test]
    fn end_turn_switches_player() {
        let engine = GameEngine::new();
        let mut state = GameState::new();

        let log = engine.apply(&mut state, Action::EndTurn).unwrap();

        assert!(matches!(log[0], Event::TurnEnded { .. }));
        assert!(matches!(log[1], Event::TurnStarted { .. }));
        assert_eq!(state.active_player(), PlayerId::Player2);
        assert_eq!(state.turn(), 2);
    }

    #[test]
    fn illegal_action_preserves_state() {
        let engine = GameEngine::new();
        let mut state = GameState::new();

        // Try to attack a non-existent entity
        let result = engine.apply(
            &mut state,
            Action::Attack {
                attacker: crate::core::entity::Entity::new(999, 0),
                defender: crate::core::entity::Entity::new(998, 0),
            },
        );

        assert!(result.is_err());
        // The state should be unchanged
        assert_eq!(state.turn(), 1);
        assert_eq!(state.step(), crate::core::state::Step::Main);
    }

    #[test]
    fn hero_death_ends_game() {
        let engine = GameEngine::new();
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 30, 3);
        let hero = state.player(PlayerId::Player2).hero;

        let log = engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker,
                    defender: hero,
                },
            )
            .unwrap();

        // The last event should be GameOver
        let last_event = log.last().unwrap();
        assert!(matches!(last_event, Event::GameOver { .. }));
        assert_eq!(
            state.step(),
            crate::core::state::Step::GameOver {
                winner: PlayerId::Player1
            }
        );
    }

    #[test]
    fn game_over_rejects_further_actions() {
        let engine = GameEngine::new();
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 30, 3);
        let hero = state.player(PlayerId::Player2).hero;

        engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker,
                    defender: hero,
                },
            )
            .unwrap();

        // The game is over; further actions should be rejected
        let result = engine.apply(&mut state, Action::EndTurn);
        assert_eq!(result, Err(EngineError::GameAlreadyOver));
    }

    #[test]
    fn hero_power_no_definition_rejected() {
        let engine = GameEngine::new();
        let mut state = GameState::new();
        let hero = state.player(PlayerId::Player1).hero;
        // No hero power defined; not enough mana
        let result = engine.apply(&mut state, Action::HeroPower { hero, target: None });
        assert_eq!(result, Err(EngineError::NotEnoughMana));
    }
}
