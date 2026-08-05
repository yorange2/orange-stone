//! Rule engine — validation, event enqueueing, event application.
//!
//! Three core functions:
//! - `validate()` — read-only check of an action's legality in the current state
//! - `enqueue()` — converts an action into initial events and enqueues them
//! - `apply_event()` — applies a single event, possibly enqueueing new events
//!
//! All functions are pure-functional in style, interacting with state via a `GameState` parameter.

use crate::core::action::Action;
use crate::core::component::{
    Attack, AttacksUsed, CardType, Cost, Durability, Health, HeroPowerUsed, Secret,
};
use crate::core::component::{TriggerEvent, TriggerTiming};
use crate::core::entity::Entity;
use crate::core::event::{Event, EventQueue, Priority};
use crate::core::player::PlayerId;
use crate::core::small_list::SmallList;
use crate::core::state::{GameState, Step};
use crate::core::zone::Zone;
use crate::engine::trigger;

/// Engine error — why an action could not be executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineError {
    /// Not your turn
    NotYourTurn,
    /// Trying to play the opponent's card
    NotYourCard,
    /// Card is not in hand
    CardNotInHand,
    /// Not a minion or weapon (PlayCard can only play minions or weapons)
    NotPlayable,
    /// Board is full (7 minion limit)
    BoardFull,
    /// Invalid target (attacking own side or target does not exist)
    InvalidTarget,
    /// Attacker is not on the board
    NotOnBoard,
    /// Attack count exhausted this turn
    AttacksExhausted,
    /// Entity has been destroyed (stale handle)
    EntityGone(Entity),
    /// Game is already over
    GameAlreadyOver,
    /// Not enough mana
    NotEnoughMana,
    /// Must attack a taunt minion first
    MustAttackTaunt,
    /// Hero power already used this turn
    HeroPowerAlreadyUsed,
    /// Feature not yet implemented (Phase 2+)
    Unimplemented,
}

/// Maximum number of minions on the battlefield.
pub const MAX_BOARD_SIZE: usize = 7;

/// Validates an action's legality in the current state (read-only).
///
/// Returns `Ok(())` or `Err(EngineError)`.
pub fn validate(state: &GameState, action: Action) -> Result<(), EngineError> {
    // Reject all actions once the game is over
    if matches!(state.step(), Step::GameOver { .. }) {
        return Err(EngineError::GameAlreadyOver);
    }

    match action {
        Action::PlayCard { card, target: _ } => validate_play_card(state, card),
        Action::Attack { attacker, defender } => validate_attack(state, attacker, defender),
        Action::EndTurn => validate_end_turn(state),
        Action::HeroPower { hero } => validate_hero_power(state, hero),
    }
}

/// Validates a play-card action.
fn validate_play_card(state: &GameState, card: Entity) -> Result<(), EngineError> {
    let world = state.world();
    let active = state.active_player();

    // Check entity liveness and component presence
    check_entity(world, card)?;

    let card_player = world.player(card).ok_or(EngineError::EntityGone(card))?;
    if card_player != active {
        return Err(EngineError::NotYourCard);
    }

    // Must be a minion, weapon, or spell
    let card_type = world.card_type(card).ok_or(EngineError::NotPlayable)?;
    if card_type != CardType::Minion
        && card_type != CardType::Weapon
        && card_type != CardType::Spell
    {
        return Err(EngineError::NotPlayable);
    }

    // Must be in hand
    let zone = world.zone(card).ok_or(EngineError::CardNotInHand)?;
    if zone != Zone::Hand {
        return Err(EngineError::CardNotInHand);
    }

    // Check mana (using effective cost: aura reduction + Kirin Tor Mage's one-time free secret)
    let cost = effective_play_cost(state, card, active);
    if cost.0 > state.player(active).current_mana {
        return Err(EngineError::NotEnoughMana);
    }

    // Check board size limit (weapons do not occupy a minion slot)
    if card_type == CardType::Minion {
        let board_count = count_board_minions(world, active);
        if board_count >= MAX_BOARD_SIZE {
            return Err(EngineError::BoardFull);
        }
    }

    Ok(())
}

/// Computes the effective cost of playing a card: base cost - aura reduction;
/// secret cards additionally benefit from Kirin Tor Mage's one-time free discount.
fn effective_play_cost(state: &GameState, card: Entity, player: PlayerId) -> Cost {
    let mut cost = state.world().effective_cost(card).unwrap_or_default();
    let is_secret = state
        .world()
        .card_id(card)
        .and_then(|cid| crate::cards::def::card_by_id(cid.0))
        .is_some_and(|def| def.secret.is_some());
    if is_secret && state.player(player).next_secret_free {
        cost = Cost(0);
    }
    cost
}

/// Validates an attack action.
fn validate_attack(
    state: &GameState,
    attacker: Entity,
    defender: Entity,
) -> Result<(), EngineError> {
    let world = state.world();
    let active = state.active_player();

    // Check phase
    if state.step() != Step::Main {
        return Err(EngineError::InvalidTarget);
    }

    // Check entity liveness
    check_entity(world, attacker)?;
    check_entity(world, defender)?;

    // Attacker must be a friendly character (minion or hero) on the board
    let attacker_player = world
        .player(attacker)
        .ok_or(EngineError::EntityGone(attacker))?;
    if attacker_player != active {
        return Err(EngineError::InvalidTarget);
    }
    let attacker_type = world.card_type(attacker).ok_or(EngineError::NotOnBoard)?;
    match attacker_type {
        CardType::Minion => {}
        CardType::Hero => {
            // Hero attacks require a weapon or a temporary attack bonus
            let has_weapon = state.player(attacker_player).weapon.is_some();
            let has_temp_attack = state.player(attacker_player).temp_attack_bonus > 0;
            if !has_weapon && !has_temp_attack {
                return Err(EngineError::InvalidTarget);
            }
        }
        _ => return Err(EngineError::NotOnBoard),
    }
    let attacker_zone = world.zone(attacker).ok_or(EngineError::NotOnBoard)?;
    if attacker_zone != Zone::Play {
        return Err(EngineError::NotOnBoard);
    }

    // Check attack count (accounting for windfury)
    let max_atks = world.max_attacks(attacker);
    if world
        .attacks_used(attacker)
        .is_some_and(|a| a.is_exhausted_with(max_atks))
    {
        return Err(EngineError::AttacksExhausted);
    }

    // Minions that cannot attack (e.g. Ragnaros)
    if world.cant_attack(attacker).is_some() {
        return Err(EngineError::InvalidTarget);
    }

    // Attack must be > 0 (considering weapon and auras)
    let total_atk = compute_attacker_damage(state, attacker);
    if total_atk <= 0 {
        return Err(EngineError::InvalidTarget);
    }

    // Defender must be an enemy target (own hero or own minion is not allowed)
    let defender_player = world.player(defender).ok_or(EngineError::InvalidTarget)?;
    if defender_player == active {
        return Err(EngineError::InvalidTarget);
    }

    // Defender must be a minion or hero on the battlefield
    let defender_type = world
        .card_type(defender)
        .ok_or(EngineError::InvalidTarget)?;
    let defender_zone = world.zone(defender).ok_or(EngineError::InvalidTarget)?;
    if defender_zone != Zone::Play {
        return Err(EngineError::InvalidTarget);
    }
    match defender_type {
        CardType::Minion | CardType::Hero => {}
        _ => return Err(EngineError::InvalidTarget),
    }

    // Taunt check: if the enemy board has taunt minions, a taunt must be attacked
    let enemy = active.opponent();
    let has_taunt = world
        .zones()
        .iter(Zone::Play, enemy)
        .any(|e| world.taunt(e).is_some());
    if has_taunt {
        // defender must be a taunt minion
        if world.taunt(defender).is_none() {
            return Err(EngineError::MustAttackTaunt);
        }
    }

    // Stealth check: cannot attack enemy stealthed characters
    if world.stealth(defender).is_some() && defender_player != active {
        return Err(EngineError::InvalidTarget);
    }

    Ok(())
}

/// Validates an end-turn action.
fn validate_end_turn(state: &GameState) -> Result<(), EngineError> {
    if state.step() != Step::Main {
        return Err(EngineError::NotYourTurn);
    }
    Ok(())
}

/// Validates a hero power use.
fn validate_hero_power(state: &GameState, hero: Entity) -> Result<(), EngineError> {
    let world = state.world();
    let active = state.active_player();

    // Check entity liveness
    check_entity(world, hero)?;

    // Must be a hero
    if world.card_type(hero) != Some(CardType::Hero) {
        return Err(EngineError::InvalidTarget);
    }

    // Must be own hero
    let hero_player = world.player(hero).ok_or(EngineError::EntityGone(hero))?;
    if hero_player != active {
        return Err(EngineError::NotYourTurn);
    }

    // Phase check
    if state.step() != Step::Main {
        return Err(EngineError::NotYourTurn);
    }

    // Check whether already used this turn
    if world.hero_power_used(hero).is_some_and(|u| u.0) {
        return Err(EngineError::HeroPowerAlreadyUsed);
    }

    // Check mana
    let hero_power = world.hero_power(hero);
    let cost = hero_power.map(|hp| hp.cost).unwrap_or(2);
    if cost > state.player(active).current_mana {
        return Err(EngineError::NotEnoughMana);
    }

    Ok(())
}

/// Computes the attacker's total damage (base attack + auras + weapon bonus).
fn compute_attacker_damage(state: &GameState, attacker: Entity) -> i32 {
    let world = state.world();
    let base = world.effective_attack(attacker).unwrap_or(Attack(0));
    let weapon_bonus = if world.card_type(attacker) == Some(CardType::Hero) {
        world
            .player(attacker)
            .and_then(|pid| state.player(pid).weapon)
            .and_then(|w| world.attack(w))
            .unwrap_or(Attack(0))
            .0
    } else {
        0
    };
    base.0 + weapon_bonus
}

/// Checks whether an entity is alive, returning `EntityGone` otherwise.
fn check_entity(world: &crate::core::world::World, entity: Entity) -> Result<(), EngineError> {
    if world.is_alive(entity) {
        Ok(())
    } else {
        Err(EngineError::EntityGone(entity))
    }
}

/// Counts the minions on a player's battlefield.
fn count_board_minions(world: &crate::core::world::World, player: PlayerId) -> usize {
    world
        .zones()
        .iter(Zone::Play, player)
        .filter(|&e| world.card_type(e) == Some(CardType::Minion))
        .count()
}

// ============================================================
// Event enqueueing
// ============================================================

/// Generates initial events from an action and enqueues them (read-only).
pub fn enqueue(
    state: &GameState,
    action: Action,
    queue: &mut EventQueue,
) -> Result<(), EngineError> {
    match action {
        Action::PlayCard { card, target } => {
            let player = state.active_player();
            queue.push(Event::CardPlayed {
                player,
                card,
                target,
            });
            let card_type = state.world().card_type(card);
            if card_type == Some(CardType::Minion) {
                queue.push(Event::MinionSummoned {
                    player,
                    minion: card,
                });
            }
        }
        Action::Attack { attacker, defender } => {
            let world = state.world();
            queue.push(Event::AttackDeclared { attacker, defender });
            // The attack resolution is enqueued as a single pipeline event:
            // the damage value is computed when `ResolveAttack` is processed,
            // but the attacker's damage must be fixed at enqueue time — the
            // weapon may be destroyed in `AttackDeclared`, so the attack
            // damage must include the weapon bonus.
            let attacker_total_atk = compute_attacker_damage(state, attacker);
            // Retaliation immunity (Gladiator's Longbow: the hero is immune while attacking, no retaliation)
            let retaliation_immune = world.card_type(attacker) == Some(CardType::Hero)
                && world.player(attacker).is_some_and(|pid| {
                    state.player(pid).weapon.is_some_and(|w| {
                        world.card_id(w).is_some_and(|c| {
                            c.0 == crate::cards::classic_hunter::GLADIATORS_LONGBOW_ID
                        })
                    })
                });
            queue.push(Event::ResolveAttack {
                attacker,
                defender,
                attacker_damage: attacker_total_atk,
                retaliation_immune,
            });
        }
        Action::EndTurn => {
            // Only the TurnEnded event is enqueued here: the step machine runs
            // the end sequence (EndTriggers → WrapUp) and enqueues the next
            // player's TurnStarted itself at the wrap-up boundary.
            let active = state.active_player();
            queue.push(Event::TurnEnded { player: active });
        }
        Action::HeroPower { hero } => {
            let active = state.active_player();
            queue.push(Event::HeroPowerActivated {
                player: active,
                hero,
            });
        }
    }
    Ok(())
}

// ============================================================
// Event application (mutating state)
// ============================================================

/// Death check — the last step of the damage pipeline (`DamageDealt`).
///
/// When the target's effective health is ≤ 0: heroes enqueue a game-over
/// (highest priority), minions enqueue `MinionDied`.
fn queue_death_events(
    state: &GameState,
    queue: &mut EventQueue,
    target: Entity,
    card_type: Option<CardType>,
) {
    let effective_hp = state.world().effective_health(target);
    if !effective_hp.is_some_and(|h| h.is_dead()) {
        return;
    }
    match card_type {
        Some(CardType::Hero) => {
            let loser = state.world().player(target);
            if let Some(player) = loser {
                let winner = player.opponent();
                queue.push_with_priority(Event::GameOver { winner }, Priority::Highest);
            }
        }
        Some(CardType::Minion) => {
            queue.push(Event::MinionDied { minion: target });
        }
        _ => {}
    }
}

/// Applies a single event, possibly enqueueing new events.
///
/// This is the core of the event loop. Each event is processed exactly once;
/// if an event produces new events, they are added to the queue and processed in turn.
pub fn apply_event(
    state: &mut GameState,
    event: Event,
    queue: &mut EventQueue,
) -> Result<(), EngineError> {
    match event {
        Event::TurnStarted { player } => {
            // Corruption: destroy corrupted minions at the start of your turn
            // (mem::take swaps in an empty Vec — zero allocation, unlike drain().collect())
            let inner = state.make_mut();
            let corrupted = std::mem::take(&mut inner.players[player.index()].corrupted);
            for entity in corrupted {
                if !state.world().is_alive(entity)
                    || state.world().card_type(entity) != Some(CardType::Minion)
                {
                    continue;
                }
                let hp = state.world().health(entity).unwrap_or(Health(1));
                queue.push(Event::DamageDealt {
                    source: entity,
                    target: entity,
                    amount: hp.0.max(1),
                });
            }

            // First collect entities whose attack counts need resetting (requires a read-only
            // borrow). Only board characters hold attack state, so restrict to Zone::Play —
            // this also bounds the list (hero + up to 7 minions + weapon) for the stack buffer.
            let player_entities: SmallList<Entity> = state
                .world()
                .iter_player()
                .filter(|(e, pid)| **pid == player && state.world().zone(*e) == Some(Zone::Play))
                .map(|(e, _)| e)
                .collect();
            let new_turn = state.turn() + 1;

            // Then perform all modifications step by step
            {
                let world = state.world_mut();
                for &entity in player_entities.iter() {
                    world.set_attacks_used(entity, AttacksUsed(0));
                    // Reset the hero-power-used flag
                    world.set_hero_power_used(entity, HeroPowerUsed(false));
                    // Clear freeze
                    world.remove_freeze(entity);
                }
            }
            state.set_active_player(player);
            state.set_turn(new_turn);
            // Enter the start-of-turn sequence. The mana refill and the draw are
            // NOT done here: the step machine runs StartTriggers (start-of-turn
            // secrets already fired on this event via check_secrets, start-turn
            // card effects fire next) → ManaRefill → DrawStep → Main, so that
            // start-of-turn triggers resolve before the refill and the draw.
            state.set_step(Step::StartTriggers);
        }
        Event::TurnEnded { player: _ } => {
            // End-of-turn effects fire in the EndTriggers step — before the
            // wrap-up cleanup — so effects resolve at full strength and deaths
            // they cause are processed before "until end of turn" buffs expire.
            state.set_step(Step::EndTriggers);
        }
        Event::CardPlayed {
            player,
            card,
            target,
        } => {
            // Deduct mana (using effective cost: aura reduction + Kirin Tor Mage's one-time free secret)
            let cost = effective_play_cost(state, card, player);
            let card_type = state.world().card_type(card);
            {
                let inner = state.make_mut();
                let p = &mut inner.players[player.index()];
                p.current_mana -= cost.0;
                p.cards_played_this_turn += 1;
            }
            // Detect combo: another card was played this turn (cards_played > 1 because it was just incremented)
            let combo_active = state.player(player).cards_played_this_turn > 1;
            if card_type == Some(CardType::Spell) {
                // Secret cards: attach the Secret component and move to the
                // SetAside zone (revealed when the trigger condition is met).
                // Secret effects are stored in the battlecry slot (consistent
                // with the spell card convention).
                let secret_trigger = state
                    .world()
                    .card_id(card)
                    .and_then(|cid| crate::cards::def::card_by_id(cid.0))
                    .and_then(|def| def.secret);
                if let Some(trigger) = secret_trigger {
                    // Kirin Tor Mage's one-time free secret is consumed
                    let secret_effect = state.world().battlecry(card).map(|b| b.0);
                    let inner = state.make_mut();
                    inner.players[player.index()].next_secret_free = false;
                    if let Some(effect) = secret_effect {
                        inner.world.set_secret(card, Secret { trigger, effect });
                    }
                    state
                        .world_mut()
                        .move_to_zone(card, Zone::SetAside)
                        .map_err(|_| EngineError::EntityGone(card))?;
                    queue.push(Event::SpellCast {
                        player,
                        spell: card,
                    });
                } else {
                    // Spell card: resolve the effect (with choose-one random pick and combo), then move to the graveyard
                    let chosen_effect = if combo_active {
                        // Combo: prefer combo_effect
                        state
                            .world()
                            .combo_effect(card)
                            .map(|c| c.0)
                            .or_else(|| state.world().battlecry(card).map(|b| b.0))
                    } else if state.world().choose_one_effect(card).is_some() {
                        // Choose one: pick randomly
                        let has_main = state.world().battlecry(card).is_some();
                        let has_alt = state.world().choose_one_effect(card).is_some();
                        if has_main && has_alt {
                            if state.rng_mut().next_usize(2) == 0 {
                                state.world().battlecry(card).map(|b| b.0)
                            } else {
                                state.world().choose_one_effect(card).map(|c| c.0)
                            }
                        } else {
                            state
                                .world()
                                .battlecry(card)
                                .map(|b| b.0)
                                .or_else(|| state.world().choose_one_effect(card).map(|c| c.0))
                        }
                    } else {
                        state.world().battlecry(card).map(|b| b.0)
                    };
                    // Combo bounce-back (Headcrack): the card stays in hand after the effect resolves instead of going to the graveyard
                    let returns_to_hand = matches!(
                        chosen_effect,
                        Some(crate::core::effect::CardEffect::DealDamageAndReturnToHand { .. })
                    );
                    if let Some(effect) = chosen_effect {
                        trigger::resolve_effect(state, queue, card, player, effect, target);
                    }
                    // Push the spell-cast event
                    queue.push(Event::SpellCast {
                        player,
                        spell: card,
                    });
                    if !returns_to_hand {
                        state
                            .world_mut()
                            .move_to_zone(card, Zone::Graveyard)
                            .map_err(|_| EngineError::EntityGone(card))?;
                    }
                }
            } else if card_type == Some(CardType::Weapon) {
                // Weapon card: destroy the old weapon first, then equip the new one
                let old_weapon = state.player(player).weapon;
                if let Some(old) = old_weapon {
                    let inner = state.make_mut();
                    inner.players[player.index()].weapon = None;
                    queue.push(Event::WeaponDestroyed {
                        player,
                        weapon: old,
                    });
                }
                // Equip the new weapon: move the card to the battlefield as a weapon entity
                state
                    .world_mut()
                    .move_to_zone(card, Zone::Play)
                    .map_err(|_| EngineError::EntityGone(card))?;
                let inner = state.make_mut();
                inner.players[player.index()].weapon = Some(card);
                queue.push(Event::WeaponEquipped {
                    player,
                    weapon: card,
                });
                // Weapon battlecry (combo-aware): resolved after equipping, e.g. Perdition's Blade
                let weapon_effect = if combo_active {
                    state
                        .world()
                        .combo_effect(card)
                        .map(|c| c.0)
                        .or_else(|| state.world().battlecry(card).map(|b| b.0))
                } else {
                    state.world().battlecry(card).map(|b| b.0)
                };
                if let Some(effect) = weapon_effect {
                    trigger::resolve_effect(state, queue, card, player, effect, None);
                }
            } else {
                // Move the card from hand to the battlefield
                state
                    .world_mut()
                    .move_to_zone(card, Zone::Play)
                    .map_err(|_| EngineError::EntityGone(card))?;
            }
            // Overload triggers: when a card with overload is played, registered
            // FriendlyOverloadPlayed triggers fire (Unbound Elemental)
            if state.world().overload(card).is_some() {
                fire_triggers(
                    state,
                    queue,
                    TriggerEvent::FriendlyOverloadPlayed,
                    player,
                    None,
                    None,
                );
            }
        }
        Event::MinionSummoned { player, minion } => {
            // Summoning sickness: minions without charge cannot attack this turn
            if state.world().charge(minion).is_none() {
                state.world_mut().set_attacks_used(minion, AttacksUsed(1));
            }
            // Check battlecry component (combo-aware)
            let combo_active = state.player(player).cards_played_this_turn > 1;
            let chosen_effect = if combo_active {
                state
                    .world()
                    .combo_effect(minion)
                    .map(|c| c.0)
                    .or_else(|| state.world().battlecry(minion).map(|b| b.0))
            } else {
                state.world().battlecry(minion).map(|b| b.0)
            };
            if let Some(effect) = chosen_effect {
                trigger::resolve_effect(state, queue, minion, player, effect, None);
            }
            // Summon triggers: registered FriendlyMinionSummoned triggers fire in
            // play order (the summoned minion itself is excluded)
            fire_triggers(
                state,
                queue,
                TriggerEvent::FriendlyMinionSummoned,
                player,
                None,
                Some(minion),
            );
        }
        Event::AttackDeclared { attacker, .. } => {
            // Freeze check: frozen characters cannot attack
            if state.world().freeze(attacker).is_some() {
                return Err(EngineError::InvalidTarget);
            }

            // Read attacker type and weapon info first (read-only borrow)
            let is_hero = state.world().card_type(attacker) == Some(CardType::Hero);
            let attacker_player = state.world().player(attacker);
            let weapon_info: Option<(PlayerId, Entity)> = if is_hero {
                attacker_player.and_then(|pid| state.player(pid).weapon.map(|w| (pid, w)))
            } else {
                None
            };

            // Mark the attacker as having used an attack
            {
                let world = state.world_mut();
                let used = world.attacks_used(attacker).unwrap_or(AttacksUsed(0));
                world.set_attacks_used(attacker, AttacksUsed(used.0 + 1));
            }

            // Decrement weapon durability by 1
            if let Some((player, weapon)) = weapon_info {
                let dur = state.world().durability(weapon).unwrap_or(Durability(0));
                let new_dur = Durability(dur.0 - 1);
                state.world_mut().set_durability(weapon, new_dur);
                if new_dur.0 <= 0 {
                    // Destroy the weapon
                    let inner = state.make_mut();
                    inner.players[player.index()].weapon = None;
                    queue.push(Event::WeaponDestroyed { player, weapon });
                }
            }
        }
        Event::ResolveAttack {
            attacker,
            defender,
            attacker_damage,
            retaliation_immune,
        } => {
            // Enqueue the attack damage (the value was fixed at enqueue time)
            queue.push(Event::DamageDealt {
                source: attacker,
                target: defender,
                amount: attacker_damage,
            });
            // Defender retaliation: attack is computed from the current state
            // at resolution time — after an attack is redirected by a secret,
            // the new defender's retaliation applies automatically, without
            // per-card special handling.
            if !retaliation_immune && state.world().card_type(defender) == Some(CardType::Minion) {
                let atk = state
                    .world()
                    .effective_attack(defender)
                    .unwrap_or(Attack(0));
                if atk.0 > 0 {
                    queue.push(Event::DamageDealt {
                        source: defender,
                        target: attacker,
                        amount: atk.0,
                    });
                }
            }
        }
        Event::DamageDealt {
            target,
            amount,
            source,
        } => {
            // Unified damage pipeline: immune → divine shield → armor → health → death check
            // Immune: damage is completely ignored (the attack is still consumed)
            if state.world().immune(target).is_some() {
                return Ok(());
            }
            // Divine shield absorbs: if the target has a divine shield, remove it and zero the damage
            if state.world().divine_shield(target).is_some() {
                state.world_mut().remove_divine_shield(target);
                return Ok(());
            }

            // Get the target's card type
            let card_type = state.world().card_type(target);

            // Heroes lose armor first; remaining damage leaks through to health
            if card_type == Some(CardType::Hero) {
                let target_player = state.world().player(target);
                if let Some(pid) = target_player {
                    let armor = state.player(pid).armor;
                    if armor > 0 {
                        let absorbed = amount.min(armor);
                        let remaining = amount - absorbed;
                        let inner = state.make_mut();
                        inner.players[pid.index()].armor -= absorbed;
                        if remaining <= 0 {
                            // All damage absorbed by armor
                            return Ok(());
                        }
                        // Remaining damage continues through to health
                        let current_health = state.world().health(target);
                        let Some(hp) = current_health else {
                            return Ok(());
                        };
                        let new_hp = Health(hp.0 - remaining);
                        state.world_mut().set_health(target, new_hp);
                        queue_death_events(state, queue, target, card_type);
                        return Ok(());
                    }
                }
            }

            // No armor or non-hero: deduct health directly
            let current_health = state.world().health(target);
            let Some(hp) = current_health else {
                // Target does not exist (dead or removed); skip
                return Ok(());
            };
            let mut new_hp = Health(hp.0 - amount);
            // Poison: a poisonous source destroys minions it damages (divine shield already handled above)
            if state.world().poison(source).is_some()
                && card_type == Some(CardType::Minion)
                && amount > 0
            {
                new_hp = Health(0);
            }
            // Commanding Shout: minions' health cannot drop below 1 this turn
            if card_type == Some(CardType::Minion)
                && state
                    .world()
                    .player(target)
                    .is_some_and(|pid| state.player(pid).minion_min_health > 0)
            {
                let min_hp = state
                    .world()
                    .player(target)
                    .map(|pid| state.player(pid).minion_min_health)
                    .unwrap_or(0);
                new_hp = Health(new_hp.0.max(min_hp));
            }

            // Mutate state via CoW
            state.world_mut().set_health(target, new_hp);

            // Damage triggers (roadmap G2): "whenever this minion takes damage"
            // (Acolyte of Pain) and "whenever a friendly minion takes damage"
            // (Frothing Berserker, Armorsmith) fire after the damage is applied
            // and before the death check — matching HS (Acolyte draws even if it
            // dies to the same damage).
            if amount > 0 && card_type == Some(CardType::Minion) {
                if let Some(owner) = state.world().player(target) {
                    fire_triggers(
                        state,
                        queue,
                        TriggerEvent::ThisMinionDamaged,
                        owner,
                        Some(target),
                        None,
                    );
                    fire_triggers(
                        state,
                        queue,
                        TriggerEvent::FriendlyMinionDamaged,
                        owner,
                        None,
                        None,
                    );
                }
            }

            // Death check (using effective health to account for aura bonuses)
            queue_death_events(state, queue, target, card_type);
        }
        Event::MinionDied { minion } => {
            // Check the deathrattle effect first (the entity is still on the board)
            let deathrattle_effect = state.world().deathrattle(minion);
            let owner = state.world().player(minion);

            // If there is a deathrattle effect, enqueue it (processed later, preserving the current entity state)
            if let (Some(dr), Some(owner)) = (deathrattle_effect, owner) {
                trigger::resolve_effect(state, queue, minion, owner, dr.0, None);
            }

            // Death triggers: registered FriendlyMinionDied triggers fire in play
            // order (the dead minion itself is excluded)
            if let Some(owner) = owner {
                fire_triggers(
                    state,
                    queue,
                    TriggerEvent::FriendlyMinionDied,
                    owner,
                    None,
                    Some(minion),
                );
            }

            // Move the minion to the graveyard (entity and components kept for replay and Phase 2+ graveyard effects)
            state
                .world_mut()
                .move_to_zone(minion, Zone::Graveyard)
                .map_err(|_| EngineError::EntityGone(minion))?;
            // Record the death for resurrection effects
            if let Some(owner) = owner {
                let inner = state.make_mut();
                inner.players[owner.index()].died_this_turn.push(minion);
            }
        }
        Event::CardDrawn { .. } => {
            // Notification event — the card was already moved to hand in draw_card
        }
        Event::HeroPowerActivated { player, hero } => {
            // Deduct mana
            let cost = state
                .world()
                .hero_power(hero)
                .map(|hp| hp.cost)
                .unwrap_or(2);
            {
                let inner = state.make_mut();
                let p = &mut inner.players[player.index()];
                p.current_mana = (p.current_mana - cost).max(0);
            }
            // Mark as used
            state
                .world_mut()
                .set_hero_power_used(hero, HeroPowerUsed(true));
            // Resolve the hero power effect
            if let Some(hp_def) = state.world().hero_power(hero) {
                let effect = hp_def.effect;
                trigger::resolve_effect(state, queue, hero, player, effect, None);
            }
        }
        Event::WeaponEquipped { .. } => {
            // Notification event — the weapon was already created in resolve_equip_weapon
        }
        Event::WeaponDestroyed { .. } => {
            // Notification event — the weapon was already removed in AttackDeclared or on equip
        }
        Event::SecretRevealed { .. } => {
            // Notification event — the secret effect is resolved through the trigger system
        }
        Event::SpellCast { player, spell: _ } => {
            // Spell triggers: registered FriendlySpellCast triggers fire in play order
            fire_triggers(
                state,
                queue,
                TriggerEvent::FriendlySpellCast,
                player,
                None,
                None,
            );
        }
        Event::GameOver { winner } => {
            state.set_step(Step::GameOver { winner });
        }
    }
    Ok(())
}

// ============================================================
// Step state machine (roadmap G1)
// ============================================================

/// Advances the step state machine at an event-queue boundary.
///
/// Steps are entered by state: the turn-start sequence runs StartTriggers →
/// ManaRefill → DrawStep → Main (start-of-turn card effects fire before the
/// mana refill and the draw); the end sequence runs EndTriggers → WrapUp →
/// next player's TurnStarted. Returns `false` when the machine is waiting for
/// player input (Main step with an empty queue) or the game is over — the
/// engine loop should stop then.
pub fn advance_step(state: &mut GameState, queue: &mut EventQueue) -> bool {
    match state.step() {
        Step::GameOver { .. } => false,
        Step::StartTriggers => {
            // Start-of-turn triggers (CardDef::start_turn_effect) fire before
            // the mana refill and the draw
            let active = state.active_player();
            fire_triggers(state, queue, TriggerEvent::TurnStart, active, None, None);
            state.set_step(Step::ManaRefill);
            true
        }
        Step::ManaRefill => {
            // Mana crystal growth and refill (overload mana locks apply here in F1)
            let active = state.active_player();
            let inner = state.make_mut();
            let p = &mut inner.players[active.index()];
            p.mana_crystals = (p.mana_crystals + 1).min(10);
            p.current_mana = p.mana_crystals;
            p.cards_played_this_turn = 0;
            state.set_step(Step::DrawStep);
            true
        }
        Step::DrawStep => {
            // The first player does not draw on turn 1 (the initial state never
            // runs a DrawStep for it; the guard covers constructed states too).
            let active = state.active_player();
            if !(state.turn() == 1 && active == PlayerId::Player1) {
                trigger::draw_card(state, queue, active);
            }
            state.set_step(Step::Main);
            true
        }
        Step::EndTriggers => {
            // End-of-turn triggers fire before the wrap-up cleanup
            let active = state.active_player();
            fire_triggers(state, queue, TriggerEvent::TurnEnd, active, None, None);
            state.set_step(Step::WrapUp);
            true
        }
        Step::WrapUp => {
            // Expire "until end of turn" effects, then start the opponent's turn
            wrap_up_turn(state);
            queue.push(Event::TurnStarted {
                player: state.active_player().opponent(),
            });
            state.set_step(Step::StartTriggers);
            true
        }
        Step::Death => {
            // The death step's batch processing happens in the event loop
            // (queued MinionDied events; Milestone G3 replaces them with marked
            // pending deaths). Back to the main step.
            state.set_step(Step::Main);
            true
        }
        Step::Main => false,
    }
}

/// Fires registered triggers for the given event (roadmap G2).
///
/// Replaces the ad-hoc per-class trigger scans with per-entity trigger
/// registration (the unified `Trigger` component). Triggers fire in **play
/// order** (summon order on each board), with the **current player's triggers
/// first** and the opponent's second — HS's simultaneous-trigger precedence.
/// "Whenever" triggers fire before "after" triggers (the HS whenever/after
/// timing classification).
///
/// `event_owner` is the player the event involves (the spell caster, the
/// minion's owner, the turn player); only triggers whose scope includes them
/// fire. `subject` pins the event to one entity (`ThisMinionDamaged` — the
/// damaged minion). `exclude` skips one entity (the summoned/died minion).
pub fn fire_triggers(
    state: &mut GameState,
    queue: &mut EventQueue,
    event: TriggerEvent,
    event_owner: PlayerId,
    subject: Option<Entity>,
    exclude: Option<Entity>,
) {
    let active = state.active_player();
    for timing in [TriggerTiming::Whenever, TriggerTiming::After] {
        for player in [active, active.opponent()] {
            let triggers: SmallList<(Entity, crate::core::effect::CardEffect), 8> = state
                .world()
                .zones()
                .iter(Zone::Play, player)
                .filter(|&e| {
                    Some(e) != exclude
                        && state.world().is_alive(e)
                        && state.world().trigger(e).is_some_and(|t| {
                            t.event == event
                                && t.timing == timing
                                && trigger_applies(event, player, event_owner, subject, e)
                        })
                })
                .map(|e| {
                    let t = state
                        .world()
                        .trigger(e)
                        .expect("filter guarantees a trigger");
                    (e, t.effect)
                })
                .collect();
            for (source, effect) in triggers {
                trigger::resolve_effect(state, queue, source, player, effect, None);
            }
        }
    }
}

/// Whether a trigger owned by `trigger_player` fires for the event.
///
/// Most trigger classes are friendly-scoped (the trigger's owner must be the
/// event's player); `ThisMinionDamaged` is pinned to the damaged minion itself.
fn trigger_applies(
    event: TriggerEvent,
    trigger_player: PlayerId,
    event_owner: PlayerId,
    subject: Option<Entity>,
    entity: Entity,
) -> bool {
    match event {
        TriggerEvent::ThisMinionDamaged => Some(entity) == subject,
        _ => trigger_player == event_owner,
    }
}

/// Turn wrap-up: expires "until end of turn" effects and clears per-turn state.
///
/// Runs after the EndTriggers step so that end-of-turn effects resolve at full
/// strength and deaths they cause are processed before buffs expire.
fn wrap_up_turn(state: &mut GameState) {
    let player = state.active_player();
    // Clear temporary attack bonus
    {
        let inner = state.make_mut();
        let p = &mut inner.players[player.index()];
        let hero = p.hero;
        let bonus = p.temp_attack_bonus;
        if bonus > 0 {
            let cur_atk = inner.world.attack(hero).unwrap_or(Attack(0));
            inner
                .world
                .set_attack(hero, Attack((cur_atk.0 - bonus).max(0)));
            p.temp_attack_bonus = 0;
        }
    }
    // Clear temporary attack debuffs and death records
    {
        let inner = state.make_mut();
        let p = &mut inner.players[player.index()];
        p.died_this_turn.clear();
        // Clear temporary attack debuffs on all entities (stack-buffered snapshot)
        let debuff_entities: SmallList<Entity> = inner
            .world
            .iter_temp_attack_debuff()
            .map(|(e, _)| e)
            .collect();
        for &e in debuff_entities.iter() {
            inner.world.remove_temp_attack_debuff(e);
        }
        // Clear temporary immunity on all entities (Bestial Wrath — until end of turn)
        let immune_entities: SmallList<Entity> =
            inner.world.iter_immune().map(|(e, _)| e).collect();
        for &e in immune_entities.iter() {
            inner.world.remove_immune(e);
        }
    }
    // Return temporarily controlled minions (Shadow Madness — until end of turn)
    // (mem::take — zero allocation)
    let inner = state.make_mut();
    let controlled = std::mem::take(&mut inner.players[player.index()].controlled_this_turn);
    for (entity, original_owner) in controlled {
        if state.world().is_alive(entity) {
            trigger::transfer_minion(state, entity, original_owner);
        }
    }
    // Clear the commanding-shout minimum health effect (until end of turn)
    {
        let inner = state.make_mut();
        inner.players[player.index()].minion_min_health = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::{Attack, CardType, Health, Trigger};
    use crate::core::entity::Entity;
    use crate::core::state::GameState;
    use crate::core::zone::Zone;

    /// Helper: create a fully-componented minion on the battlefield
    fn add_minion_to_board(state: &mut GameState, player: PlayerId, atk: i32, hp: i32) -> Entity {
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

    /// Helper: add a minion to hand
    fn add_minion_to_hand(state: &mut GameState, player: PlayerId, atk: i32, hp: i32) -> Entity {
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
        // Give enough mana for testing
        state.make_mut().players[player.index()].current_mana = 10;
        e
    }

    #[test]
    fn validate_play_card_legal() {
        let mut state = GameState::new();
        let card = add_minion_to_hand(&mut state, PlayerId::Player1, 2, 3);
        let result = validate(&state, Action::PlayCard { card, target: None });
        assert!(result.is_ok(), "valid play should succeed: {result:?}");
    }

    #[test]
    fn validate_play_card_not_your_card() {
        let mut state = GameState::new();
        let card = add_minion_to_hand(&mut state, PlayerId::Player2, 2, 3);
        let result = validate(&state, Action::PlayCard { card, target: None });
        assert_eq!(result, Err(EngineError::NotYourCard));
    }

    #[test]
    fn validate_play_card_not_in_hand() {
        let mut state = GameState::new();
        let card = add_minion_to_board(&mut state, PlayerId::Player1, 2, 3);
        // On the battlefield, not in hand
        let result = validate(&state, Action::PlayCard { card, target: None });
        assert_eq!(result, Err(EngineError::CardNotInHand));
    }

    #[test]
    fn validate_play_card_board_full() {
        let mut state = GameState::new();
        // Fill the board with 7 minions
        for _ in 0..7 {
            add_minion_to_board(&mut state, PlayerId::Player1, 1, 1);
        }
        let card = add_minion_to_hand(&mut state, PlayerId::Player1, 2, 3);
        let result = validate(&state, Action::PlayCard { card, target: None });
        assert_eq!(result, Err(EngineError::BoardFull));
    }

    #[test]
    fn validate_attack_legal() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 3);
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 2, 2);
        let result = validate(&state, Action::Attack { attacker, defender });
        assert!(result.is_ok(), "valid attack should succeed: {result:?}");
    }

    #[test]
    fn validate_attack_own_minion() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 3);
        let defender = add_minion_to_board(&mut state, PlayerId::Player1, 2, 2);
        let result = validate(&state, Action::Attack { attacker, defender });
        assert_eq!(result, Err(EngineError::InvalidTarget));
    }

    #[test]
    fn validate_attack_hero_legal() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 3);
        let defender = state.player(PlayerId::Player2).hero;
        let result = validate(&state, Action::Attack { attacker, defender });
        assert!(result.is_ok(), "attacking hero should be valid: {result:?}");
    }

    #[test]
    fn validate_attack_twice() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 3);
        state.world_mut().set_attacks_used(attacker, AttacksUsed(1));
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 2, 2);
        let result = validate(&state, Action::Attack { attacker, defender });
        assert_eq!(result, Err(EngineError::AttacksExhausted));
    }

    #[test]
    fn validate_stale_entity() {
        let state = GameState::new();
        let entity = Entity::new(999, 0); // Non-existent entity
        let result = validate(
            &state,
            Action::PlayCard {
                card: entity,
                target: None,
            },
        );
        assert_eq!(result, Err(EngineError::EntityGone(entity)));
    }

    #[test]
    fn apply_trade_deals_damage_both_ways() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 5);
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 2, 3);

        let mut queue = EventQueue::new();
        enqueue(&state, Action::Attack { attacker, defender }, &mut queue).unwrap();

        // Process all events
        while let Some(event) = queue.pop_front() {
            apply_event(&mut state, event, &mut queue).unwrap();
        }

        // defender takes 3 damage: 3 → 0, dies
        assert_eq!(
            state.world().health(defender),
            Some(Health(0)),
            "defender should have taken 3 damage"
        );
        assert_eq!(
            state.world().zone(defender),
            Some(Zone::Graveyard),
            "defender should be dead in graveyard"
        );

        // attacker takes 2 damage: 5 → 3, survives
        assert_eq!(
            state.world().health(attacker),
            Some(Health(3)),
            "attacker should have taken 2 damage"
        );
        assert_eq!(
            state.world().attacks_used(attacker),
            Some(AttacksUsed(1)),
            "attacker should have used attack"
        );
    }

    #[test]
    fn damage_hero() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 5, 3);
        let hero = state.player(PlayerId::Player2).hero;

        let mut queue = EventQueue::new();
        enqueue(
            &state,
            Action::Attack {
                attacker,
                defender: hero,
            },
            &mut queue,
        )
        .unwrap();

        while let Some(event) = queue.pop_front() {
            apply_event(&mut state, event, &mut queue).unwrap();
        }

        assert_eq!(state.world().health(hero), Some(Health(25)));
        // The attacker should not take damage from the hero (heroes do not retaliate)
        assert_eq!(state.world().health(attacker), Some(Health(3)));
    }

    #[test]
    fn game_over_on_hero_death() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 30, 3);
        let hero = state.player(PlayerId::Player2).hero;

        let mut queue = EventQueue::new();
        enqueue(
            &state,
            Action::Attack {
                attacker,
                defender: hero,
            },
            &mut queue,
        )
        .unwrap();

        while let Some(event) = queue.pop_front() {
            apply_event(&mut state, event, &mut queue).unwrap();
        }

        assert_eq!(
            state.step(),
            Step::GameOver {
                winner: PlayerId::Player1
            }
        );
    }

    #[test]
    fn attacker_dies_still_deals_damage() {
        // The attacker dies in the trade, but damage is still dealt (simultaneous resolution)
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 4, 1);
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 5, 10);

        let mut queue = EventQueue::new();
        enqueue(&state, Action::Attack { attacker, defender }, &mut queue).unwrap();

        while let Some(event) = queue.pop_front() {
            apply_event(&mut state, event, &mut queue).unwrap();
        }

        // defender takes 4 damage: 10 → 6
        assert_eq!(state.world().health(defender), Some(Health(6)));
        // attacker takes 5 damage: 1 → -4 → dies
        assert_eq!(state.world().health(attacker), Some(Health(-4)));
        assert_eq!(state.world().zone(attacker), Some(Zone::Graveyard));
        assert_eq!(state.world().zone(defender), Some(Zone::Play)); // defender survives
    }

    #[test]
    fn reset_attacks_on_turn_start() {
        let mut state = GameState::new();
        let minion = add_minion_to_board(&mut state, PlayerId::Player1, 2, 3);
        state.world_mut().set_attacks_used(minion, AttacksUsed(1));

        let mut queue = EventQueue::new();
        queue.push(Event::TurnStarted {
            player: PlayerId::Player1,
        });

        // Apply events
        while let Some(event) = queue.pop_front() {
            apply_event(&mut state, event, &mut queue).unwrap();
        }

        assert_eq!(state.world().attacks_used(minion), Some(AttacksUsed(0)));
    }

    // ============================================================
    // G1 — step state machine
    // ============================================================

    #[test]
    fn first_player_turn_one_has_one_mana() {
        // The first player's opening turn starts with 1 mana crystal (their
        // turn-1 refill is built into GameState::new).
        let state = GameState::new();
        assert_eq!(state.player(PlayerId::Player1).mana_crystals, 1);
        assert_eq!(state.player(PlayerId::Player1).current_mana, 1);
        assert_eq!(state.player(PlayerId::Player2).mana_crystals, 0);
    }

    #[test]
    fn mana_refill_runs_on_turn_start() {
        // Player2's turn 1 (turn counter 2) goes through the ManaRefill step: 0 → 1.
        let mut state = GameState::new();
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        assert_eq!(state.player(PlayerId::Player2).mana_crystals, 1);
        assert_eq!(state.player(PlayerId::Player2).current_mana, 1);
        // The machine returns to Main, waiting for input
        assert_eq!(state.step(), Step::Main);
    }

    #[test]
    fn first_player_does_not_draw_on_turn_one() {
        // The first player does not draw on turn 1; the second player draws on
        // their first turn; the first player draws from turn 2 on.
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_deck(PlayerId::Player1, &BLOODFEN_RAPTOR);
        builder.add_minion_to_deck(PlayerId::Player2, &BLOODFEN_RAPTOR);
        let mut state = builder.build();
        let engine = crate::engine::game::GameEngine::new();

        // End Player1's opening turn: Player2 draws, Player1's deck is untouched
        engine.apply(&mut state, Action::EndTurn).unwrap();
        assert_eq!(
            state.world().zones().len(Zone::Deck, PlayerId::Player1),
            1,
            "first player must not draw on turn 1"
        );
        assert_eq!(state.world().zones().len(Zone::Deck, PlayerId::Player2), 0);

        // End Player2's turn: Player1 draws on turn 2
        engine.apply(&mut state, Action::EndTurn).unwrap();
        assert_eq!(state.world().zones().len(Zone::Deck, PlayerId::Player1), 0);
        assert_eq!(state.world().zones().len(Zone::Hand, PlayerId::Player1), 1);
    }

    #[test]
    fn start_of_turn_effect_fires_before_draw() {
        // A start-of-turn effect (CardDef::start_turn_effect, wired in G1)
        // resolves in the StartTriggers step — before the mana refill and the
        // turn's draw both happened afterwards.
        let mut state = GameState::new();
        let minion = add_minion_to_board(&mut state, PlayerId::Player2, 1, 2);
        state.world_mut().set_trigger(
            minion,
            Trigger {
                event: TriggerEvent::TurnStart,
                timing: TriggerTiming::Whenever,
                effect: crate::core::effect::CardEffect::GainStats {
                    attack: 1,
                    health: 1,
                    target: crate::core::effect::EffectTarget::Self_,
                },
            },
        );
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        // The start effect resolved
        assert_eq!(state.world().attack(minion), Some(Attack(2)));
        assert_eq!(state.world().health(minion), Some(Health(3)));
    }

    #[test]
    fn end_of_turn_effect_fires_before_wrap_up() {
        // End-of-turn effects must resolve at full strength, before "until end
        // of turn" buffs expire. The hero has a temporary attack bonus of 5 and
        // an end-of-turn effect dealing the hero's attack as damage: the enemy
        // minion must take the full 5 (if wrap-up ran first, it would take 0).
        let mut state = GameState::new();
        // Heroic-Strike-style temporary bonus: attack 5 + temp bonus 5
        {
            let inner = state.make_mut();
            let p = &mut inner.players[0];
            p.temp_attack_bonus = 5;
            inner.world.set_attack(p.hero, Attack(5));
        }
        let hero = state.player(PlayerId::Player1).hero;
        state.world_mut().set_trigger(
            hero,
            Trigger {
                event: TriggerEvent::TurnEnd,
                timing: TriggerTiming::Whenever,
                effect: crate::core::effect::CardEffect::DealHeroAttackDamage {
                    target: crate::core::effect::EffectTarget::AnyEnemy,
                },
            },
        );
        let enemy_minion = add_minion_to_board(&mut state, PlayerId::Player2, 1, 2);
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();

        // The effect fired with the full attack (5 damage kills the 2-HP minion)
        assert_eq!(
            state.world().zone(enemy_minion),
            Some(Zone::Graveyard),
            "end-of-turn effect must resolve before the temporary buff expires"
        );
        // The temp bonus expired afterwards
        assert_eq!(state.player(PlayerId::Player1).temp_attack_bonus, 0);
        assert_eq!(
            state.world().attack(hero),
            Some(Attack(0)),
            "temporary attack bonus must expire at wrap-up"
        );
    }

    #[test]
    fn turn_start_secret_fires_before_draw() {
        // A start-of-turn secret (OnFriendlyTurnStart) fires when the
        // TurnStarted event is processed — before the ManaRefill and DrawStep.
        // Its discard effect must run against the empty hand, leaving exactly
        // the turn's drawn card (if the draw happened first, the discard would
        // remove it).
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::core::component::{CardId, Secret, SecretTrigger};
        use crate::core::effect::CardEffect;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_deck(PlayerId::Player2, &BLOODFEN_RAPTOR);
        let mut state = builder.build();
        // A Player2 secret that discards a random card on their turn start
        let secret_entity = {
            let world = state.world_mut();
            let e = world.spawn();
            world.set_card_id(e, CardId("CLASSIC_001"));
            world.set_card_type(e, CardType::Spell);
            world.set_cost(e, crate::core::component::Cost(0));
            world.set_player(e, PlayerId::Player2);
            world.set_secret(
                e,
                Secret {
                    trigger: SecretTrigger::OnFriendlyTurnStart,
                    effect: CardEffect::DiscardRandomCard,
                },
            );
            world.set_zone(e, Zone::SetAside);
            world
                .zones_mut()
                .insert(Zone::SetAside, PlayerId::Player2, e);
            e
        };
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();

        // The secret fired (revealed) and the draw still happened afterwards
        assert_eq!(state.world().zone(secret_entity), Some(Zone::Graveyard));
        assert_eq!(
            state.world().zones().len(Zone::Hand, PlayerId::Player2),
            1,
            "turn-start secret must fire before the draw"
        );
    }

    #[test]
    fn death_step_processes_kills_and_returns_to_main() {
        // Combat deaths surface while in the Main step: the machine enters the
        // Death step, batch-processes the MinionDied events, and returns to Main.
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 4, 5);
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 1, 2);
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(&mut state, Action::Attack { attacker, defender })
            .unwrap();
        assert_eq!(state.world().zone(defender), Some(Zone::Graveyard));
        assert_eq!(state.step(), Step::Main);
    }

    // ============================================================
    // G2 — registered triggers
    // ============================================================

    #[test]
    fn acolyte_of_pain_draws_when_damaged() {
        // Acolyte of Pain is a damage trigger, not a deathrattle (the old
        // mis-modeling is removed): it draws when it takes damage.
        use crate::cards::def::ACOLYTE_OF_PAIN;
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_board(PlayerId::Player1, &ACOLYTE_OF_PAIN);
        builder.add_minion_to_deck(PlayerId::Player1, &BLOODFEN_RAPTOR);
        let mut state = builder.build();
        // P2's attacker damages the Acolyte on P2's turn
        let attacker = add_minion_to_board(&mut state, PlayerId::Player2, 2, 3);
        let acolyte = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .find(|&e| {
                state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| c.0 == "NEUTRAL_004")
            })
            .expect("acolyte on board");
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker,
                    defender: acolyte,
                },
            )
            .unwrap();
        assert_eq!(
            state.world().zones().len(Zone::Hand, PlayerId::Player1),
            1,
            "Acolyte of Pain must draw when damaged"
        );
        assert_eq!(state.world().health(acolyte), Some(Health(1)));
    }

    #[test]
    fn acolyte_of_pain_draws_before_dying_to_the_damage() {
        // HS semantics: the draw fires when the damage is applied, before the
        // death check — a lethal hit still draws.
        use crate::cards::def::ACOLYTE_OF_PAIN;
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_board(PlayerId::Player1, &ACOLYTE_OF_PAIN);
        builder.add_minion_to_deck(PlayerId::Player1, &BLOODFEN_RAPTOR);
        let mut state = builder.build();
        let acolyte = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .find(|&e| {
                state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| c.0 == "NEUTRAL_004")
            })
            .expect("acolyte on board");
        let attacker = add_minion_to_board(&mut state, PlayerId::Player2, 5, 3);
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker,
                    defender: acolyte,
                },
            )
            .unwrap();
        assert_eq!(
            state.world().zones().len(Zone::Hand, PlayerId::Player1),
            1,
            "Acolyte draws even when the damage is lethal"
        );
        assert_eq!(state.world().zone(acolyte), Some(Zone::Graveyard));
    }

    #[test]
    fn frothing_berserker_gains_attack_when_friendly_minion_damaged() {
        use crate::cards::def::FROTHING_BERSERKER;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_board(PlayerId::Player1, &FROTHING_BERSERKER);
        let mut state = builder.build();
        let frothing = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .find(|&e| {
                state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| c.0 == "WARRIOR_007")
            })
            .expect("frothing on board");
        let friendly_minion = add_minion_to_board(&mut state, PlayerId::Player1, 1, 3);
        let attacker = add_minion_to_board(&mut state, PlayerId::Player2, 1, 3);
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker,
                    defender: friendly_minion,
                },
            )
            .unwrap();
        assert_eq!(
            state.world().attack(frothing),
            Some(Attack(3)),
            "Frothing Berserker gains +1 attack when a friendly minion takes damage"
        );
    }

    #[test]
    fn armorsmith_gains_armor_when_friendly_minion_damaged() {
        use crate::cards::def::ARMORSMITH;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_board(PlayerId::Player1, &ARMORSMITH);
        let mut state = builder.build();
        let friendly_minion = add_minion_to_board(&mut state, PlayerId::Player1, 1, 3);
        let attacker = add_minion_to_board(&mut state, PlayerId::Player2, 1, 3);
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker,
                    defender: friendly_minion,
                },
            )
            .unwrap();
        assert_eq!(
            state.player(PlayerId::Player1).armor,
            1,
            "Armorsmith grants 1 armor when a friendly minion takes damage"
        );
    }

    #[test]
    fn triggers_fire_in_play_order() {
        // Two friendly-summon triggers with order-observable effects: the first
        // played (A draws) must fire before the second (B discards) — the drawn
        // card is discarded, leaving the hand empty.
        use crate::core::effect::CardEffect;
        let mut state = GameState::new();
        let a = add_minion_to_board(&mut state, PlayerId::Player1, 1, 1);
        let b = add_minion_to_board(&mut state, PlayerId::Player1, 1, 1);
        state.world_mut().set_trigger(
            a,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                effect: CardEffect::DrawCard { count: 1 },
            },
        );
        state.world_mut().set_trigger(
            b,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                effect: CardEffect::DiscardRandomCard,
            },
        );
        // A deck with one card so A's draw has something to draw
        {
            let world = state.world_mut();
            let e = world.spawn();
            world.set_card_type(e, CardType::Minion);
            world.set_cost(e, crate::core::component::Cost(0));
            world.set_player(e, PlayerId::Player1);
            world.set_zone(e, Zone::Deck);
            world.zones_mut().insert(Zone::Deck, PlayerId::Player1, e);
        }
        // Play a third minion from hand to trigger FriendlyMinionSummoned
        let hand_minion = add_minion_to_hand(&mut state, PlayerId::Player1, 1, 1);
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: hand_minion,
                    target: None,
                },
            )
            .unwrap();
        assert_eq!(
            state.world().zones().len(Zone::Hand, PlayerId::Player1),
            0,
            "triggers must fire in play order: A draws first, B discards it"
        );
    }

    #[test]
    fn whenever_triggers_fire_before_after() {
        // HS timing classification: "whenever" triggers fire before "after"
        // triggers, even when the "after" trigger was played first.
        use crate::core::effect::CardEffect;
        let mut state = GameState::new();
        // B is played first but is an "after" trigger; A is a "whenever" trigger
        let b = add_minion_to_board(&mut state, PlayerId::Player1, 1, 1);
        let a = add_minion_to_board(&mut state, PlayerId::Player1, 1, 1);
        state.world_mut().set_trigger(
            b,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::After,
                effect: CardEffect::DiscardRandomCard,
            },
        );
        state.world_mut().set_trigger(
            a,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                effect: CardEffect::DrawCard { count: 1 },
            },
        );
        {
            let world = state.world_mut();
            let e = world.spawn();
            world.set_card_type(e, CardType::Minion);
            world.set_cost(e, crate::core::component::Cost(0));
            world.set_player(e, PlayerId::Player1);
            world.set_zone(e, Zone::Deck);
            world.zones_mut().insert(Zone::Deck, PlayerId::Player1, e);
        }
        let hand_minion = add_minion_to_hand(&mut state, PlayerId::Player1, 1, 1);
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: hand_minion,
                    target: None,
                },
            )
            .unwrap();
        // Whenever (A) drew, then After (B) discarded it
        assert_eq!(
            state.world().zones().len(Zone::Hand, PlayerId::Player1),
            0,
            "whenever triggers must fire before after triggers"
        );
    }
}
