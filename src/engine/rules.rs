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
    Attack, AttacksUsed, CardType, Durability, EnchantmentExpiry, Health, HeroPowerUsed, Secret,
    TriggerEvent, TriggerTiming,
};
use crate::core::entity::Entity;
use crate::core::event::{Event, EventQueue, Priority};
use crate::core::player::PlayerId;
use crate::core::small_list::SmallList;
use crate::core::state::{ChoiceKind, GameState, Step};
use crate::core::zone::Zone;
use crate::engine::secret;
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
    /// No choice is pending (roadmap G6)
    NoPendingChoice,
    /// The choice id does not match the pending choice, or the option is out of range
    InvalidChoice,
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
        Action::PlayCard {
            card,
            target: _,
            position: _,
        } => validate_play_card(state, card),
        Action::Attack { attacker, defender } => validate_attack(state, attacker, defender),
        Action::EndTurn => validate_end_turn(state),
        Action::HeroPower { hero, target: _ } => validate_hero_power(state, hero),
        Action::Choose { choice_id, option } => validate_choose(state, choice_id, option),
        Action::TradeCard { card } => validate_trade_card(state, card),
    }
}

/// Validates a Tradeable trade (Core Set W2): the card must be a Tradeable
/// hand card and the player must afford the 1-mana trade.
fn validate_trade_card(state: &GameState, card: Entity) -> Result<(), EngineError> {
    let world = state.world();
    let active = state.active_player();
    check_entity(world, card)?;
    let card_player = world.player(card).ok_or(EngineError::EntityGone(card))?;
    if card_player != active {
        return Err(EngineError::NotYourCard);
    }
    if world.zone(card) != Some(Zone::Hand) {
        return Err(EngineError::CardNotInHand);
    }
    if world.tradeable(card).is_none() {
        return Err(EngineError::InvalidTarget);
    }
    if state.player(active).current_mana < 1 {
        return Err(EngineError::NotEnoughMana);
    }
    Ok(())
}

/// Validates a choice resolution (roadmap G6): a choice must be pending and
/// the id/option must match.
fn validate_choose(state: &GameState, choice_id: u64, option: u8) -> Result<(), EngineError> {
    let pending = state.pending_choice().ok_or(EngineError::NoPendingChoice)?;
    if pending.id != choice_id {
        return Err(EngineError::InvalidChoice);
    }
    if option as usize >= pending.options.len() {
        return Err(EngineError::InvalidChoice);
    }
    Ok(())
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

    // Check mana (single cost composition — roadmap G5)
    let cost = crate::engine::cost::play_cost(state, card, active);
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
            // Hero attacks require a weapon or a temporary attack enchantment
            // (Heroic Strike-style buffs make the hero able to attack)
            let has_weapon = state.player(attacker_player).weapon.is_some();
            let has_attack = state
                .world()
                .effective_attack(attacker)
                .is_some_and(|a| a.0 > 0);
            if !has_weapon && !has_attack {
                return Err(EngineError::InvalidTarget);
            }
        }
        _ => return Err(EngineError::NotOnBoard),
    }
    let attacker_zone = world.zone(attacker).ok_or(EngineError::NotOnBoard)?;
    if attacker_zone != Zone::Play {
        return Err(EngineError::NotOnBoard);
    }

    // Freeze (engine-mechanics roadmap M2): a frozen character cannot attack.
    // With the thaw moved to the turn-end wrap-up the freeze is actually
    // present during the owner's next turn, so legal actions must exclude it.
    if world.freeze(attacker).is_some() {
        return Err(EngineError::InvalidTarget);
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

    // Rush (Core Set W1): a minion with Rush cannot attack the enemy HERO
    // on the turn it was summoned (it may attack enemy minions). The
    // SummonedThisTurn marker is cleared at the start of the next turn.
    if defender_type == CardType::Hero
        && attacker_type == CardType::Minion
        && world.rush(attacker).is_some()
        && world.summoned_this_turn(attacker).is_some()
    {
        return Err(EngineError::InvalidTarget);
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
/// `pub(crate)` — the forced-attack effect (Mythical Terror, Core Set W1)
/// enqueues attacks from the trigger resolver.
pub(crate) fn compute_attacker_damage(state: &GameState, attacker: Entity) -> i32 {
    let world = state.world();
    let base = world.effective_attack(attacker).unwrap_or(Attack(0));
    // `effective_attack` rather than the raw component so weapon enchantments
    // and Spiteful Smith's Enrage (+2 Attack to your weapon while damaged)
    // reach the hero's swing. Auras never apply to weapons.
    let weapon_bonus = if world.card_type(attacker) == Some(CardType::Hero) {
        world
            .player(attacker)
            .and_then(|pid| state.player(pid).weapon)
            .and_then(|w| world.effective_attack(w))
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
        Action::PlayCard {
            card,
            target,
            position,
        } => {
            let player = state.active_player();
            queue.push(Event::CardPlayed {
                player,
                card,
                target,
                position,
            });
            let card_type = state.world().card_type(card);
            if card_type == Some(CardType::Minion) {
                queue.push(Event::MinionSummoned {
                    player,
                    minion: card,
                    target,
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
        Action::TradeCard { card } => {
            // Tradeable (Core Set W2): the state changes run in the
            // event handler (`TradeCardExecuted`); enqueue is read-only.
            queue.push(Event::TradeCardExecuted { card });
        }
        Action::HeroPower { hero, target } => {
            let active = state.active_player();
            queue.push(Event::HeroPowerActivated {
                player: active,
                hero,
                target,
            });
        }
        Action::Choose { choice_id, option } => {
            queue.push(Event::ChoiceResolved { choice_id, option });
        }
    }
    Ok(())
}

// ============================================================
// Event application (mutating state)
// ============================================================

/// Lifesteal (Core Set W1) — a Lifesteal damage source heals its owner's
/// hero for the damage dealt. Only characters with damage accumulate a heal
/// (overhealing triggers nothing, matching the heal pipeline); a healed hero
/// fires `CharacterHealed` triggers (Lightwarden sees lifesteal heals).
fn resolve_lifesteal_heal(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    amount: i32,
) {
    // The source itself carries Lifesteal (minion, spell entity, weapon-hit
    // source), or the source is a hero whose equipped weapon has it
    // (Aldrachi Warblades — hero attacks heal; Core Set W1).
    let source_heals = state.world().lifesteal(source).is_some()
        || (state.world().card_type(source) == Some(CardType::Hero)
            && state
                .world()
                .player(source)
                .and_then(|p| state.player(p).weapon)
                .is_some_and(|w| state.world().lifesteal(w).is_some()));
    if !source_heals {
        return;
    }
    let Some(owner) = state.world().player(source) else {
        return;
    };
    let hero = state.player(owner).hero;
    let dmg = state
        .world()
        .damage(hero)
        .unwrap_or(crate::core::component::Damage(0))
        .0;
    if dmg <= 0 {
        return;
    }
    let new_dmg = (dmg - amount).max(0);
    if new_dmg > 0 {
        state
            .world_mut()
            .set_damage(hero, crate::core::component::Damage(new_dmg));
    } else {
        state.world_mut().remove_damage(hero);
    }
    fire_triggers(
        state,
        queue,
        TriggerEvent::CharacterHealed,
        owner,
        Some(hero),
        None,
    );
}

/// Death check — the last step of the damage pipeline (`DamageDealt`).
///
/// When the target's effective health is ≤ 0: heroes enqueue a game-over
/// (highest priority); minions are marked pending death (roadmap G3) — they
/// stay on the battlefield until the death step processes them, so healing can
/// rescue them before their death resolves.
fn queue_death_events(
    state: &mut GameState,
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
            let inner = state.make_mut();
            inner.pending_deaths.push(target);
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
            // Freeze timing (engine-mechanics roadmap M2): entities frozen at
            // the START of this turn (i.e. during the opponent's turn) keep
            // Freeze through the whole turn — the AttackDeclared check blocks
            // their attacks — and thaw in the turn-end wrap-up. Entities
            // frozen DURING this turn (by this player's own actions, e.g.
            // Icicle) are not in the snapshot and stay frozen into the next
            // turn, matching HS. The snapshot is taken before the attack
            // resets below.
            let frozen_at_start: SmallList<Entity> = player_entities
                .iter()
                .copied()
                .filter(|&e| state.world().freeze(e).is_some())
                .collect();

            // Then perform all modifications step by step
            {
                let world = state.world_mut();
                for &entity in player_entities.iter() {
                    world.set_attacks_used(entity, AttacksUsed(0));
                    // Reset the hero-power-used flag
                    world.set_hero_power_used(entity, HeroPowerUsed(false));
                    // SummonedThisTurn expires: a Rush minion may attack the
                    // hero from this turn on (Core Set W1)
                    world.remove_summoned_this_turn(entity);
                }
            }
            {
                let inner = state.make_mut();
                inner.players[player.index()].frozen_at_turn_start =
                    frozen_at_start.iter().copied().collect();
                // Core Set W3a — the healed-this-turn marker expires at turn start
                inner.players[player.index()].healed_this_turn = false;
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
            position,
        } => {
            // Deduct mana (single cost composition — roadmap G5). Death
            // Metal Knight (Core Set W3a) pays Health instead of Mana when
            // the hero was healed this turn.
            let cost = crate::engine::cost::play_cost(state, card, player);
            let card_type = state.world().card_type(card);
            let is_minion = state.world().card_type(card) == Some(CardType::Minion);
            let pay_health = state.player(player).healed_this_turn
                && state
                    .world()
                    .card_id(card)
                    .is_some_and(|c| c.0 == "CORE_ETC_523");
            if pay_health {
                let hero = state.player(player).hero;
                queue.push(Event::DamageDealt {
                    source: hero,
                    target: hero,
                    amount: cost.0,
                });
            }
            {
                let inner = state.make_mut();
                let p = &mut inner.players[player.index()];
                if !pay_health {
                    p.current_mana -= cost.0;
                }
                p.cards_played_this_turn += 1;
                if is_minion {
                    p.minions_played_this_turn += 1;
                }
                // Preparation (W11): the discount is one-time — the first
                // spell played consumes it (its cost already included it)
                if card_type == Some(CardType::Spell) {
                    p.next_spell_discount = 0;
                }
            }
            // Outcast (Core Set W2): a card played from the leftmost or
            // rightmost hand position carries the Outcast marker — read
            // BEFORE the card leaves the hand. A one-card hand counts as
            // both edges (official rule).
            let hand_len = state.world().zones().len(Zone::Hand, player);
            let hand_index = state
                .world()
                .zones()
                .iter(Zone::Hand, player)
                .position(|e| e == card);
            if hand_index == Some(0) || hand_index == Some(hand_len - 1) {
                state
                    .world_mut()
                    .set_outcast_played(card, crate::core::component::OutcastPlayed);
            }

            // Detect combo: another card was played this turn (cards_played > 1 because it was just incremented)
            let combo_active = state.player(player).cards_played_this_turn > 1;
            if card_type == Some(CardType::Spell) {
                // A played spell leaves the hand immediately (HS: the card is
                // in play while its effect resolves). The hand-size cap
                // (F-A11) counts it outside the hand, so a cast at 9 cards
                // can still fit generated copies / draws; the spell then
                // moves on to SetAside (secrets) or the graveyard below.
                state
                    .world_mut()
                    .move_to_zone(card, Zone::Play)
                    .expect("spell must leave the hand when played");
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
                    // The secret is registered even when it has no effect
                    // (Counterspell — the reveal itself negates the spell)
                    inner.world.set_secret(
                        card,
                        Secret {
                            trigger,
                            effect: secret_effect,
                        },
                    );
                    state
                        .world_mut()
                        .move_to_zone(card, Zone::SetAside)
                        .map_err(|_| EngineError::EntityGone(card))?;
                    // Secret-played triggers (Secretkeeper — whenever a Secret
                    // is played; both players' secrets count)
                    fire_triggers(
                        state,
                        queue,
                        TriggerEvent::SecretPlayed,
                        player,
                        Some(card),
                        None,
                    );
                    // After-cast triggers fire at Lowest priority — after the
                    // spell's damage and the deaths it caused have resolved
                    // (HS: deaths process before "after you cast" triggers)
                    queue.push_with_priority(
                        Event::SpellCast {
                            player,
                            spell: card,
                        },
                        Priority::Lowest,
                    );
                } else {
                    // Counter-secret interception (roadmap G8): WhenEnemySpellCast
                    // secrets fire BEFORE the spell's effect resolves.
                    let interception =
                        secret::intercept_counter_secrets(state, queue, card, player);
                    match interception {
                        secret::Interception::Countered => {
                            // The spell is negated but still cast and discarded
                            // After-cast triggers fire at Lowest priority — after the
                            // spell's damage and the deaths it caused have resolved
                            // (HS: deaths process before "after you cast" triggers)
                            queue.push_with_priority(
                                Event::SpellCast {
                                    player,
                                    spell: card,
                                },
                                Priority::Lowest,
                            );
                            state
                                .world_mut()
                                .move_to_zone(card, Zone::Graveyard)
                                .map_err(|_| EngineError::EntityGone(card))?;
                        }
                        secret::Interception::Spellbent(token) => {
                            // Spellbender: the spell's single-target effect is
                            // redirected to the 1/3 token
                            let chosen_effect = if combo_active {
                                state
                                    .world()
                                    .combo_effect(card)
                                    .map(|c| c.0)
                                    .or_else(|| state.world().battlecry(card).map(|b| b.0))
                            } else {
                                state.world().battlecry(card).map(|b| b.0)
                            };
                            if let Some(effect) = chosen_effect {
                                trigger::resolve_effect(
                                    state,
                                    queue,
                                    card,
                                    player,
                                    effect,
                                    Some(token),
                                    None,
                                );
                            }
                            // After-cast triggers fire at Lowest priority — after the
                            // spell's damage and the deaths it caused have resolved
                            // (HS: deaths process before "after you cast" triggers)
                            queue.push_with_priority(
                                Event::SpellCast {
                                    player,
                                    spell: card,
                                },
                                Priority::Lowest,
                            );
                            state
                                .world_mut()
                                .move_to_zone(card, Zone::Graveyard)
                                .map_err(|_| EngineError::EntityGone(card))?;
                        }
                        secret::Interception::None => {
                            if state.world().choose_one_effect(card).is_some() && !combo_active {
                                // Choose One spell (roadmap G6): the branch choice
                                // surfaces as a pending choice; the effect, the
                                // SpellCast event, and the graveyard move resolve in
                                // ChoiceResolved. The default policy
                                // (GameEngine::apply) resolves it randomly via the
                                // embedded RNG, preserving the historical behavior.
                                state.set_pending_choice(
                                    ChoiceKind::ChooseOne,
                                    card,
                                    vec![
                                        String::from("First option"),
                                        String::from("Second option"),
                                    ],
                                    Vec::new(),
                                );
                            } else {
                                // Spell card: resolve the effect (combo-aware), then move to the graveyard
                                let chosen_effect = if combo_active {
                                    // Combo: prefer combo_effect
                                    state
                                        .world()
                                        .combo_effect(card)
                                        .map(|c| c.0)
                                        .or_else(|| state.world().battlecry(card).map(|b| b.0))
                                } else {
                                    state.world().battlecry(card).map(|b| b.0)
                                };
                                // Combo bounce-back (Headcrack): the card stays in hand after the effect resolves instead of going to the graveyard
                                let returns_to_hand = matches!(
                                    chosen_effect,
                                    Some(crate::core::effect::CardEffect::DealDamageAndReturnToHand { .. })
                                );
                                if let Some(effect) = chosen_effect {
                                    trigger::resolve_effect(
                                        state, queue, card, player, effect, target, None,
                                    );
                                }
                                // Push the spell-cast event
                                // After-cast triggers fire at Lowest priority — after the
                                // spell's damage and the deaths it caused have resolved
                                // (HS: deaths process before "after you cast" triggers)
                                queue.push_with_priority(
                                    Event::SpellCast {
                                        player,
                                        spell: card,
                                    },
                                    Priority::Lowest,
                                );
                                if returns_to_hand {
                                    // Headcrack bounce-back: the spell went to
                                    // Zone::Play at cast time (F-A11 hand cap)
                                    // and returns to the hand here.
                                    state
                                        .world_mut()
                                        .move_to_zone(card, Zone::Hand)
                                        .map_err(|_| EngineError::EntityGone(card))?;
                                } else {
                                    state
                                        .world_mut()
                                        .move_to_zone(card, Zone::Graveyard)
                                        .map_err(|_| EngineError::EntityGone(card))?;
                                }
                            }
                        }
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
                    trigger::resolve_effect(state, queue, card, player, effect, None, None);
                }
            } else {
                // Move the card from hand to the battlefield (summon position,
                // roadmap G6 — 0 = leftmost)
                state
                    .world_mut()
                    .move_to_zone(card, Zone::Play)
                    .map_err(|_| EngineError::EntityGone(card))?;
                if let Some(position) = position {
                    let world = state.world_mut();
                    world.zones_mut().remove(Zone::Play, player, card);
                    world
                        .zones_mut()
                        .insert_at(Zone::Play, player, card, position as usize);
                }
                // Choose One minions (Cenarius, Keeper of the Grove): the branch
                // choice surfaces as a pending choice — MinionSummoned skips the
                // battlecry and ChoiceResolved resolves the chosen branch (G6).
                if state.world().choose_one_effect(card).is_some() {
                    state.set_pending_choice(
                        ChoiceKind::ChooseOne,
                        card,
                        vec![String::from("First option"), String::from("Second option")],
                        Vec::new(),
                    );
                }
            }
            // Overload (roadmap F1): playing an overload card locks mana for the
            // owner's next turn (applied at the ManaRefill step)
            if let Some(overload) = state.world().overload(card) {
                let inner = state.make_mut();
                inner.players[player.index()].overload_locked += overload.0;
                // Registered FriendlyOverloadPlayed triggers fire (Unbound Elemental)
                fire_triggers(
                    state,
                    queue,
                    TriggerEvent::FriendlyOverloadPlayed,
                    player,
                    None,
                    None,
                );
            }
            // Card-played triggers (Questing Adventurer — whenever YOU play a
            // card): fire after the card fully resolved (effect included).
            fire_triggers(
                state,
                queue,
                TriggerEvent::CardPlayed,
                player,
                Some(card),
                None,
            );
        }
        Event::MinionSummoned {
            player,
            minion,
            target,
        } => {
            // Summoning sickness: minions without charge cannot attack this
            // turn (a Charge aura — Tundra Rhino — counts as charge). Rush
            // (Core Set W1) also skips sickness — it can attack enemy MINIONS
            // the turn it arrives — but is tracked by SummonedThisTurn so the
            // hero-attack ban (validate_attack) can be enforced until the
            // next turn.
            if !state.world().effective_charge(minion) {
                state
                    .world_mut()
                    .set_summoned_this_turn(minion, crate::core::component::SummonedThisTurn);
                if state.world().rush(minion).is_none() {
                    state.world_mut().set_attacks_used(minion, AttacksUsed(1));
                }
            }
            // Check battlecry component (combo-aware). Choose One minions
            // resolve their branch through the choice system (roadmap G6).
            if state.world().choose_one_effect(minion).is_none() {
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
                    // Explicit battlecry target (engine-mechanics roadmap M1):
                    // forwarded from Action::PlayCard; re-validation stays G9 —
                    // a target that left the legal candidate set fizzles.
                    trigger::resolve_effect(state, queue, minion, player, effect, target, None);
                }
            }
            // Summon triggers: registered FriendlyMinionSummoned triggers fire in
            // play order (the summoned minion itself is excluded). The summoned
            // minion is the event subject — Sword of Justice buffs it via
            // EffectTarget::EventSubject.
            fire_triggers(
                state,
                queue,
                TriggerEvent::FriendlyMinionSummoned,
                player,
                Some(minion),
                Some(minion),
            );
        }
        Event::AttackDeclared { attacker, defender } => {
            // Mayor Noggenfogger (Core Set W3a): all targets are chosen
            // randomly — the declared defender is replaced by a random enemy
            // character (hero or minion) when a Noggenfogger is on the
            // attacking player's board.
            let defender = if state
                .world()
                .zones()
                .iter(
                    Zone::Play,
                    state
                        .world()
                        .player(attacker)
                        .unwrap_or(state.active_player()),
                )
                .any(|e| {
                    state
                        .world()
                        .card_id(e)
                        .is_some_and(|c| c.0 == "CORE_CFM_670")
                }) {
                let attacker_player = state
                    .world()
                    .player(attacker)
                    .unwrap_or(state.active_player());
                let enemies: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Play, attacker_player.opponent())
                    .collect();
                if enemies.is_empty() {
                    defender
                } else {
                    let idx = state.rng_mut().next_usize(enemies.len());
                    enemies[idx]
                }
            } else {
                defender
            };
            // Freeze check: frozen characters cannot attack
            if state.world().freeze(attacker).is_some() {
                return Err(EngineError::InvalidTarget);
            }

            // Attack triggers (Blessing of Wisdom — the buffed minion draws
            // when IT attacks): pinned to the declared attacker
            fire_triggers(
                state,
                queue,
                TriggerEvent::Attacked,
                state
                    .world()
                    .player(attacker)
                    .unwrap_or(state.active_player()),
                Some(attacker),
                None,
            );
            // Minion-hit attack triggers (Gorehowl — the weapon loses 1
            // Attack when the hero attacks a minion)
            if state.world().card_type(defender) == Some(CardType::Minion) {
                fire_triggers(
                    state,
                    queue,
                    TriggerEvent::AttackedMinion,
                    state
                        .world()
                        .player(attacker)
                        .unwrap_or(state.active_player()),
                    Some(attacker),
                    None,
                );
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
                // Stealth (roadmap F2): attacking breaks the attacker's stealth
                world.remove_stealth(attacker);
            }

            // Decrement weapon durability by 1 — except Gorehowl attacking a
            // minion: the attack loss (triggered above) replaces the
            // durability loss
            if let Some((player, weapon)) = weapon_info {
                let gorehowl_minion_hit = state
                    .world()
                    .card_id(weapon)
                    .is_some_and(|c| c.0 == crate::cards::classic_warrior::GOREHOWL_ID)
                    && state.world().card_type(defender) == Some(CardType::Minion);
                if !gorehowl_minion_hit {
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
            // Water Elemental (W12, D2 — damage-pipeline check): freeze any
            // character damaged by this minion. Applied before the divine
            // shield absorption — HS freezes even when the shield absorbs.
            if state
                .world()
                .card_id(source)
                .is_some_and(|c| c.0 == crate::cards::classic_mage::WATER_ELEMENTAL_ID)
            {
                state
                    .world_mut()
                    .set_freeze(target, crate::core::component::Freeze);
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
                            // All damage absorbed by armor — the damage was
                            // dealt, so Lifesteal still heals the full amount
                            resolve_lifesteal_heal(state, queue, source, amount);
                            return Ok(());
                        }
                        // Remaining damage continues through to health — accumulate
                        // it on the damage component (roadmap G4)
                        let current_health = state.world().effective_health(target);
                        let Some(_hp) = current_health else {
                            return Ok(());
                        };
                        let damage_old = state
                            .world()
                            .damage(target)
                            .unwrap_or(crate::core::component::Damage(0))
                            .0;
                        let new_damage = damage_old + remaining;
                        state
                            .world_mut()
                            .set_damage(target, crate::core::component::Damage(new_damage));
                        queue_death_events(state, queue, target, card_type);
                        return Ok(());
                    }
                }
            }

            // No armor or non-hero: deduct health directly via the damage component
            let Some(cur) = state.world().effective_health(target) else {
                // Target does not exist (dead or removed); skip
                return Ok(());
            };
            let damage_old = state
                .world()
                .damage(target)
                .unwrap_or(crate::core::component::Damage(0))
                .0;
            // cur + damage_old = base + Σ health enchantments + aura (undamaged total)
            let mut new_damage = damage_old + amount;
            // Poison: a poisonous source destroys minions it damages (divine shield already handled above)
            if state.world().poison(source).is_some()
                && card_type == Some(CardType::Minion)
                && amount > 0
            {
                new_damage = cur.0 + damage_old;
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
                new_damage = new_damage.min(cur.0 + damage_old - min_hp);
            }

            // Mutate state via CoW
            state
                .world_mut()
                .set_damage(target, crate::core::component::Damage(new_damage));

            // Lifesteal (Core Set W1): damage dealt by a Lifesteal source
            // heals the source's owner hero for the damage dealt. Weapon,
            // minion and spell damage all count; the divine-shield/immune
            // branches returned early (no damage was dealt), and the
            // fully-armor-absorbed hero branch healed above.
            resolve_lifesteal_heal(state, queue, source, amount);

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
            // Death batching (roadmap G3): re-check the health — a minion healed
            // above 0 before its death was processed survives (the event is a no-op).
            let effective_hp = state.world().effective_health(minion);
            if !effective_hp.is_some_and(|h| h.is_dead()) {
                return Ok(());
            }
            let owner = state.world().player(minion);

            // Reborn (Core Set W1): the first death resurrects the minion as
            // a fresh 1/1 instead of dying — all buffs cleared, base stats
            // 1/1, Reborn spent, summoning sickness applied (a Rush/Charge
            // minion keeps its attack availability and can attack minions
            // again). The resurrection counts as a summon: on-summon
            // triggers fire (Sword of Justice), battlecries do NOT re-fire.
            if state.world().reborn(minion).is_some() {
                // A Reborn death still counts as a death for corpse purposes
                // (Core Set W1 — the Death-Knight corpse resource; the
                // resurrection itself produces none beyond this one).
                if let Some(owner) = owner {
                    let inner = state.make_mut();
                    inner.players[owner.index()].corpses += 1;
                }
                state.world_mut().remove_reborn(minion);
                {
                    let world = state.world_mut();
                    world.remove_enchantments(minion);
                    world.set_attack(minion, Attack(1));
                    world.set_health(minion, Health(1));
                    world.remove_damage(minion);
                }
                if !state.world().effective_charge(minion) {
                    state
                        .world_mut()
                        .set_summoned_this_turn(minion, crate::core::component::SummonedThisTurn);
                    if state.world().rush(minion).is_none() {
                        state.world_mut().set_attacks_used(minion, AttacksUsed(1));
                    }
                }
                if let Some(owner) = owner {
                    fire_triggers(
                        state,
                        queue,
                        TriggerEvent::FriendlyMinionSummoned,
                        owner,
                        Some(minion),
                        Some(minion),
                    );
                }
                return Ok(());
            }

            // Move the minion to the graveyard FIRST (entity and components kept
            // for replay and graveyard effects) — death triggers must see the
            // dead minion already removed from the battlefield.
            state
                .world_mut()
                .move_to_zone(minion, Zone::Graveyard)
                .map_err(|_| EngineError::EntityGone(minion))?;

            // Deathrattle effect (the entity is in the graveyard, as in HS)
            if let (Some(dr), Some(owner)) = (state.world().deathrattle(minion), owner) {
                trigger::resolve_effect(state, queue, minion, owner, dr.0, None, None);
            }

            // Death triggers: registered FriendlyMinionDied triggers fire in play
            // order (the dead minion itself is excluded). The dead minion is the
            // event subject — race-conditioned triggers (Scavenging Hyena — a
            // friendly Beast died) check its race.
            if let Some(owner) = owner {
                fire_triggers(
                    state,
                    queue,
                    TriggerEvent::FriendlyMinionDied,
                    owner,
                    Some(minion),
                    Some(minion),
                );
                // Any-minion-died triggers (Flesheathing Ghoul) fire for both
                // players' deaths
                fire_triggers(
                    state,
                    queue,
                    TriggerEvent::MinionDied,
                    owner,
                    Some(minion),
                    None,
                );
            }

            // Record the death for resurrection effects; the owner also
            // gains a corpse (Core Set W1 — Malignant Horror's end-of-turn
            // effect spends them; any friendly death produces one)
            if let Some(owner) = owner {
                let inner = state.make_mut();
                inner.players[owner.index()].died_this_turn.push(minion);
                inner.players[owner.index()].corpses += 1;
            }
        }
        Event::CardDrawn { .. } => {
            // Notification event — the card was already moved to hand in draw_card
        }
        Event::TradeCardExecuted { card } => {
            // Tradeable (Core Set W2): spend 1 mana, shuffle the card back
            // into the deck at a random position, draw a card.
            let active = state.active_player();
            {
                let inner = state.make_mut();
                inner.players[active.index()].current_mana =
                    (inner.players[active.index()].current_mana - 1).max(0);
            }
            let deck_count = state.world().zones().len(Zone::Deck, active);
            let position = if deck_count > 0 {
                state.rng_mut().next_usize(deck_count + 1)
            } else {
                0
            };
            // Shuffle the card into the deck at a random position (the
            // official trade semantics). Manually remove + insert: a
            // move_to_zone would append at the deck bottom, and a second
            // insert_at on top of it would duplicate the card.
            state
                .world_mut()
                .zones_mut()
                .remove(Zone::Hand, active, card);
            state.world_mut().set_zone(card, Zone::Deck);
            state
                .world_mut()
                .zones_mut()
                .insert_at(Zone::Deck, active, card, position);
            // draw_card pushes its own CardDrawn event for the drawn card
            trigger::draw_card(state, queue, active);
        }
        Event::HeroPowerActivated {
            player,
            hero,
            target,
        } => {
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
            // Resolve the hero power effect (explicit target, roadmap G6)
            if let Some(hp_def) = state.world().hero_power(hero) {
                let effect = hp_def.effect;
                trigger::resolve_effect(state, queue, hero, player, effect, target, None);
            }
        }
        Event::WeaponEquipped { .. } => {
            // Notification event — the weapon was already created in resolve_equip_weapon
        }
        Event::WeaponDestroyed { weapon, .. } => {
            // The weapon leaves play when destroyed — it must stop firing its
            // triggers (Sword of Justice's summon trigger). The player's weapon
            // pointer was already cleared by the destroying action.
            let _ = state.world_mut().move_to_zone(weapon, Zone::Graveyard);
        }
        Event::SecretRevealed { .. } => {
            // Notification event — the secret effect is resolved through the trigger system
        }
        Event::SpellCast { player, spell } => {
            // HS: deaths caused by the spell resolve BEFORE "after you cast"
            // triggers fire (a Wild Pyromancer killed by the spell must not
            // fire). When the spell left pending deaths, defer the trigger
            // firing: enqueue the death batch (Normal — processes first) and
            // re-push this event at Lowest; the second pass sees an empty
            // pending-death batch and fires the triggers.
            if !state.pending_deaths().is_empty() && process_pending_deaths(state, queue) {
                queue.push_with_priority(Event::SpellCast { player, spell }, Priority::Lowest);
                return Ok(());
            }
            // Spell triggers: registered FriendlySpellCast triggers fire in play order
            fire_triggers(
                state,
                queue,
                TriggerEvent::FriendlySpellCast,
                player,
                None,
                None,
            );
            // Global spell trigger (pool-open M1 — Lorewalker Cho): fires for
            // either player, with the spell entity as the subject so the
            // effect can hand the copy to the caster's opponent. The subject
            // is behaviour-neutral for FriendlySpellCast triggers (none of
            // them carry race/max_attack conditions) — pinned by
            // `po_spellcast_subject_is_behaviour_neutral`.
            fire_triggers(
                state,
                queue,
                TriggerEvent::AnySpellCast,
                player,
                Some(spell),
                None,
            );
        }
        Event::ChoiceResolved { choice_id, option } => {
            // Resolve the pending choice (roadmap G6). The choice was validated
            // and the pending entry is consumed here.
            let inner = state.make_mut();
            let Some(pending) = inner.pending_choice.take() else {
                return Ok(());
            };
            if pending.id != choice_id {
                inner.pending_choice = Some(pending);
                return Err(EngineError::InvalidChoice);
            }
            match pending.kind {
                ChoiceKind::ChooseOne => {
                    let card = pending.card;
                    let player = state.world().player(card).unwrap_or(state.active_player());
                    let card_type = state.world().card_type(card);
                    let chosen_effect = if option == 0 {
                        state.world().battlecry(card).map(|b| b.0)
                    } else {
                        state.world().choose_one_effect(card).map(|c| c.0)
                    };
                    let returns_to_hand = matches!(
                        chosen_effect,
                        Some(crate::core::effect::CardEffect::DealDamageAndReturnToHand { .. })
                    );
                    if let Some(effect) = chosen_effect {
                        trigger::resolve_effect(state, queue, card, player, effect, None, None);
                    }
                    if card_type == Some(CardType::Spell) {
                        // Spells: the SpellCast event and graveyard move complete
                        // the play that was deferred when the choice surfaced.
                        // After-cast triggers fire at Lowest priority — after the
                        // spell's damage and the deaths it caused have resolved
                        // (HS: deaths process before "after you cast" triggers)
                        queue.push_with_priority(
                            Event::SpellCast {
                                player,
                                spell: card,
                            },
                            Priority::Lowest,
                        );
                        if !returns_to_hand {
                            let _ = state.world_mut().move_to_zone(card, Zone::Graveyard);
                        }
                    }
                }
                ChoiceKind::Discover => {
                    let player = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    if pending.discard_rest {
                        // Tracking (W10): the pool IS the deck's top cards —
                        // the picked card's existing entity moves to hand and
                        // the unpicked ones are discarded.
                        let deck: Vec<Entity> = state
                            .world()
                            .zones()
                            .iter(Zone::Deck, player)
                            .take(3)
                            .collect();
                        let Some(picked) = deck.get(option as usize).copied() else {
                            return Ok(());
                        };
                        let _ = state.world_mut().move_to_zone(picked, Zone::Hand);
                        for &e in &deck {
                            if e != picked {
                                let _ = state.world_mut().move_to_zone(e, Zone::Graveyard);
                            }
                        }
                    } else if let Some(card_id) = pending.pool.get(option as usize) {
                        // Add the picked pool card to the owner's hand
                        if let Some(card_def) = crate::cards::def::card_by_id(card_id) {
                            trigger::add_card_to_hand(state, player, card_def);
                        }
                    }
                }
                ChoiceKind::Mulligan => {
                    // Opening mulligan (roadmap G7): "Keep all" (option 0) or
                    // replace the chosen starting card — the card returns to the
                    // deck, the deck is reshuffled, and a new card is drawn from
                    // the top. The Coin is not mulliganable.
                    let owner = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    if option > 0 {
                        let mulliganable: SmallList<Entity> = state
                            .world()
                            .zones()
                            .iter(Zone::Hand, owner)
                            .filter(|&e| state.world().card_id(e).is_none_or(|c| c.0 != "GAME_005"))
                            .collect();
                        if let Some(card) = mulliganable.get((option - 1) as usize).copied() {
                            let _ = state.world_mut().move_to_zone(card, Zone::Deck);
                            state.shuffle_decks();
                            crate::engine::trigger::draw_top_card_no_queue(state, owner);
                        }
                    }
                    state.make_mut().mulliganed[owner.index()] = true;
                    // Surface the opponent's mulligan; when both are resolved the
                    // opening is finished and the first player draws the 4th card
                    // as their turn 1 starts (official rule).
                    let next = owner.opponent();
                    if !state.make_mut().mulliganed[next.index()] {
                        state.surface_mulligan(next);
                    } else {
                        trigger::draw_top_card_no_queue(state, PlayerId::Player1);
                    }
                }
            }
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
/// next player's TurnStarted. Pending deaths (roadmap G3) enter the death
/// step from any boundary and return to the interrupted step afterwards.
/// Returns `false` when the machine is waiting for player input (Main step
/// with an empty queue) or the game is over — the engine loop should stop then.
pub fn advance_step(state: &mut GameState, queue: &mut EventQueue) -> bool {
    // The death phase takes precedence at any step boundary (HS: deaths resolve
    // before anything else proceeds). The interrupted step is saved so the
    // machine returns to it after the pending deaths are processed.
    if !state.pending_deaths().is_empty() && state.step() != Step::Death {
        state.make_mut().return_step = state.step();
        state.set_step(Step::Death);
        return true;
    }
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
            // Mana crystal growth and refill. The overload mana lock (roadmap
            // F1) applies HERE: after the refill, mana locked by overload cards
            // played last turn is subtracted from current_mana.
            let active = state.active_player();
            let inner = state.make_mut();
            let p = &mut inner.players[active.index()];
            p.mana_crystals = (p.mana_crystals + 1).min(10);
            let locked = p.overload_locked;
            p.overload_locked = 0;
            p.current_mana = (p.mana_crystals - locked).max(0);
            p.cards_played_this_turn = 0;
            p.minions_played_this_turn = 0;
            state.set_step(Step::DrawStep);
            true
        }
        Step::DrawStep => {
            // The turn's draw. Every player draws at the start of every turn,
            // including the first player's turn 1 (official rule) — the shipped
            // opening (sim::battle::build_game_state) deals that 4th card during
            // construction, so a DrawStep on turn 1 only appears in constructed
            // states, where the draw runs normally (no turn-1 skip).
            let active = state.active_player();
            trigger::draw_card(state, queue, active);
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
            // Process the pending deaths (batch): enqueue MinionDied in play
            // order, current player first. Stay in the death step while more
            // deaths surface (deathrattles can kill); return to the interrupted
            // step when the batch drains.
            if process_pending_deaths(state, queue) {
                true
            } else {
                let return_step = state.make_mut().return_step;
                state.set_step(return_step);
                true
            }
        }
        Step::Main => false,
    }
}

/// Enqueues `MinionDied` for all pending deaths, in play order with the current
/// player's minions first (HS death-phase resolution order). Returns whether
/// any death was enqueued.
///
/// The `MinionDied` handler re-checks health, so a minion healed after being
/// marked survives (roadmap G3).
fn process_pending_deaths(state: &mut GameState, queue: &mut EventQueue) -> bool {
    let active = state.active_player();
    let inner = state.make_mut();
    let pending = std::mem::take(&mut inner.pending_deaths);
    let mut any = false;
    for player in [active, active.opponent()] {
        // Play order: the zone table holds the board in summon order
        for entity in inner.world.zones().iter(Zone::Play, player) {
            if pending.contains(&entity) {
                queue.push(Event::MinionDied { minion: entity });
                any = true;
            }
        }
    }
    any
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
                                && trigger_applies(state, event, player, event_owner, subject, e, t)
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
                trigger::resolve_effect(state, queue, source, player, effect, None, subject);
            }
        }
    }
}

/// Whether a trigger owned by `trigger_player` fires for the event.
///
/// Most trigger classes are friendly-scoped (the trigger's owner must be the
/// event's player); `ThisMinionDamaged` is pinned to the damaged minion itself.
/// A race-conditioned trigger (fidelity-debt W1 — Murloc Tidecaller, Starving
/// Buzzard, Scavenging Hyena) additionally requires the event subject to have
/// the trigger's race.
fn trigger_applies(
    state: &GameState,
    event: TriggerEvent,
    trigger_player: PlayerId,
    event_owner: PlayerId,
    subject: Option<Entity>,
    entity: Entity,
    trigger: crate::core::component::Trigger,
) -> bool {
    match event {
        // Pinned to the entity the event happened to
        TriggerEvent::ThisMinionDamaged => {
            if Some(entity) != subject {
                return false;
            }
        }
        // Pinned to the attacker itself (Blessing of Wisdom), or to the
        // weapon the attacking hero has equipped (Truesilver Champion,
        // Gorehowl — weapon-attack effects ride the weapon entity)
        TriggerEvent::Attacked | TriggerEvent::AttackedMinion => {
            let pinned = Some(entity) == subject
                || subject.is_some_and(|s| {
                    state.world().card_type(s) == Some(CardType::Hero)
                        && state
                            .world()
                            .player(s)
                            .and_then(|pid| state.player(pid).weapon)
                            == Some(entity)
                });
            if !pinned {
                return false;
            }
        }
        // Global classes — fire regardless of who owns the event
        // (Lightwarden: any character healed; Northshire Cleric: any minion
        // healed; Secretkeeper: any Secret played; Flesheating Ghoul: any
        // minion died; Lorewalker Cho: any player's spell cast)
        TriggerEvent::CharacterHealed
        | TriggerEvent::MinionHealed
        | TriggerEvent::SecretPlayed
        | TriggerEvent::MinionDied
        | TriggerEvent::AnySpellCast => {}
        _ => {
            if trigger_player != event_owner {
                return false;
            }
        }
    }
    if let Some(race) = trigger.race {
        // The subject must exist and carry the required race (a dead subject
        // keeps its components in the graveyard, so its race is still readable)
        if !subject.is_some_and(|s| state.world().has_race(s, race)) {
            return false;
        }
    }
    if let Some(max_attack) = trigger.max_attack {
        // Warsong Commander — the summoned minion must have at most this much
        // Attack. Read at fire time, so a minion summoned small and buffed
        // afterwards still qualifies (and vice versa), matching HS.
        if !subject.is_some_and(|s| {
            state
                .world()
                .effective_attack(s)
                .is_some_and(|a| a.0 <= max_attack)
        }) {
            return false;
        }
    }
    true
}

/// Turn wrap-up: expires "until end of turn" effects and clears per-turn state.
///
/// Runs after the EndTriggers step so that end-of-turn effects resolve at full
/// strength and deaths they cause are processed before buffs expire.
fn wrap_up_turn(state: &mut GameState) {
    let player = state.active_player();
    // Freeze timing (engine-mechanics roadmap M2): the entities this player
    // had frozen at the start of their turn — their attacks were blocked by
    // the AttackDeclared check — thaw here, in the turn-end wrap-up, so they
    // are unfrozen for the opponent's turn (HS: thaw after the missed attack
    // opportunity). Dead entities are skipped; the snapshot is drained.
    {
        let inner = state.make_mut();
        let frozen = std::mem::take(&mut inner.players[player.index()].frozen_at_turn_start);
        for e in frozen {
            if inner.world.is_alive(e) {
                inner.world.remove_freeze(e);
            }
        }
    }
    // Expire "until end of turn" enchantments (temporary attack buffs/debuffs)
    // and clear per-turn state
    {
        let inner = state.make_mut();
        let p = &mut inner.players[player.index()];
        p.died_this_turn.clear();
        // Millhouse Manastorm's zero-spell-cost window lasts one turn
        p.spells_cost_zero = false;
        // Preparation's next-spell discount also expires at the turn end
        p.next_spell_discount = 0;
        // Stack-buffered snapshot of entities holding expiring enchantments
        let expiring: SmallList<Entity> = inner
            .world
            .iter_enchantments()
            .filter(|(_, list)| {
                list.iter()
                    .any(|e| e.expiry == EnchantmentExpiry::UntilEndOfTurn)
            })
            .map(|(e, _)| e)
            .collect();
        for &e in expiring.iter() {
            inner
                .world
                .retain_enchantments(e, |ench| ench.expiry != EnchantmentExpiry::UntilEndOfTurn);
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
    use crate::core::component::{
        Attack, CardType, Cost, CostModifier, CostModifierKind, Damage, Enchantment,
        EnchantmentExpiry, Health, Trigger,
    };
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
        let result = validate(
            &state,
            Action::PlayCard {
                card,
                target: None,
                position: None,
            },
        );
        assert!(result.is_ok(), "valid play should succeed: {result:?}");
    }

    #[test]
    fn validate_play_card_not_your_card() {
        let mut state = GameState::new();
        let card = add_minion_to_hand(&mut state, PlayerId::Player2, 2, 3);
        let result = validate(
            &state,
            Action::PlayCard {
                card,
                target: None,
                position: None,
            },
        );
        assert_eq!(result, Err(EngineError::NotYourCard));
    }

    #[test]
    fn validate_play_card_not_in_hand() {
        let mut state = GameState::new();
        let card = add_minion_to_board(&mut state, PlayerId::Player1, 2, 3);
        // On the battlefield, not in hand
        let result = validate(
            &state,
            Action::PlayCard {
                card,
                target: None,
                position: None,
            },
        );
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
        let result = validate(
            &state,
            Action::PlayCard {
                card,
                target: None,
                position: None,
            },
        );
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
                position: None,
            },
        );
        assert_eq!(result, Err(EngineError::EntityGone(entity)));
    }

    #[test]
    fn apply_trade_deals_damage_both_ways() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 5);
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 2, 3);

        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(&mut state, Action::Attack { attacker, defender })
            .unwrap();

        // defender takes 3 damage and dies (damage is cleared when it moves to
        // the graveyard, so the dead minion reads its base health)
        assert_eq!(
            state.world().zone(defender),
            Some(Zone::Graveyard),
            "defender should be dead in graveyard"
        );

        // attacker takes 2 damage: 5 → 3, survives
        assert_eq!(
            state.world().effective_health(attacker),
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

        assert_eq!(state.world().effective_health(hero), Some(Health(25)));
        // The attacker should not take damage from the hero (heroes do not retaliate)
        assert_eq!(state.world().effective_health(attacker), Some(Health(3)));
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

        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(&mut state, Action::Attack { attacker, defender })
            .unwrap();

        // defender takes 4 damage: 10 → 6
        assert_eq!(state.world().effective_health(defender), Some(Health(6)));
        // attacker takes 5 damage: 1 → -4 → dies
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
    fn first_player_draws_on_turn_one() {
        // Official rule: the first player draws the 4th card as their turn 1
        // starts. The shipped opening (sim::battle::build_game_state) deals it
        // during construction; a state entering DrawStep on turn 1 (e.g. a
        // constructed state) must draw as well — there is no turn-1 skip.
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_deck(PlayerId::Player1, &BLOODFEN_RAPTOR);
        let mut state = builder.build();
        assert_eq!(state.turn(), 1);
        state.set_step(Step::DrawStep);
        let mut queue = crate::core::event::EventQueue::new();
        assert!(advance_step(&mut state, &mut queue));
        assert_eq!(state.step(), Step::Main);
        assert_eq!(state.world().zones().len(Zone::Hand, PlayerId::Player1), 1);
        assert_eq!(state.world().zones().len(Zone::Deck, PlayerId::Player1), 0);
    }

    #[test]
    fn second_player_draws_on_first_turn_first_player_from_turn_two() {
        // Step-machine draw schedule: the second player draws on their first
        // turn (entered via TurnStarted); the first player draws from turn 2 on
        // (its turn-1 card is dealt during the opening construction).
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_deck(PlayerId::Player1, &BLOODFEN_RAPTOR);
        builder.add_minion_to_deck(PlayerId::Player2, &BLOODFEN_RAPTOR);
        let mut state = builder.build();
        let engine = crate::engine::game::GameEngine::new();

        // End Player1's opening turn: Player2 draws on its first turn
        engine.apply(&mut state, Action::EndTurn).unwrap();
        assert_eq!(state.world().zones().len(Zone::Deck, PlayerId::Player1), 1);
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
                race: None,
                max_attack: None,
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
        assert_eq!(state.world().effective_attack(minion), Some(Attack(2)));
        assert_eq!(state.world().effective_health(minion), Some(Health(3)));
    }

    #[test]
    fn end_of_turn_effect_fires_before_wrap_up() {
        // End-of-turn effects must resolve at full strength, before "until end
        // of turn" buffs expire. The hero has a temporary attack bonus of 5 and
        // an end-of-turn effect dealing the hero's attack as damage: the enemy
        // minion must take the full 5 (if wrap-up ran first, it would take 0).
        let mut state = GameState::new();
        // Heroic-Strike-style temporary bonus: a +5 until-end-of-turn attack enchantment
        let hero = state.player(PlayerId::Player1).hero;
        state.world_mut().add_enchantment(
            hero,
            crate::core::component::Enchantment {
                attack: 5,
                health: 0,
                cost: 0,
                expiry: crate::core::component::EnchantmentExpiry::UntilEndOfTurn,
            },
        );
        state.world_mut().set_trigger(
            hero,
            Trigger {
                event: TriggerEvent::TurnEnd,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
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
        // The temp enchantment expired afterwards — the hero is back to base attack
        assert_eq!(
            state.world().effective_attack(hero),
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
                    effect: Some(CardEffect::DiscardRandomCard),
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
        assert_eq!(state.world().effective_health(acolyte), Some(Health(1)));
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
            state.world().effective_attack(frothing),
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
                race: None,
                max_attack: None,
                effect: CardEffect::DrawCard { count: 1 },
            },
        );
        state.world_mut().set_trigger(
            b,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
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
                    position: None,
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
                race: None,
                max_attack: None,
                effect: CardEffect::DiscardRandomCard,
            },
        );
        state.world_mut().set_trigger(
            a,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
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
                    position: None,
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

    // ============================================================
    // G3 — death batching
    // ============================================================

    /// Plays a custom P1 minion whose battlecry deals `amount` damage to all
    /// enemy minions, returning the battlecry minion.
    fn play_aoe_battlecry(state: &mut GameState, amount: i32) -> Entity {
        use crate::core::component::Battlecry;
        use crate::core::effect::CardEffect;
        let aoe = {
            let world = state.world_mut();
            let e = world.spawn();
            world.set_health(e, Health(2));
            world.set_attack(e, Attack(1));
            world.set_cost(e, crate::core::component::Cost(1));
            world.set_card_type(e, CardType::Minion);
            world.set_player(e, PlayerId::Player1);
            world.set_attacks_used(e, AttacksUsed(0));
            world.set_battlecry(
                e,
                Battlecry(CardEffect::DealDamage {
                    amount,
                    target: crate::core::effect::EffectTarget::AllEnemyMinions,
                }),
            );
            world.set_zone(e, Zone::Hand);
            world.zones_mut().insert(Zone::Hand, PlayerId::Player1, e);
            e
        };
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                state,
                Action::PlayCard {
                    card: aoe,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        aoe
    }

    #[test]
    fn healed_pending_death_survives() {
        // Death batching (G3): a minion marked dead stays on the battlefield
        // until the death step processes it. B (played first, deathrattle heals
        // all friendly minions) dies before A; its deathrattle heals A above 0,
        // so A's death is re-checked and A survives.
        use crate::core::component::Deathrattle;
        use crate::core::effect::CardEffect;
        let mut state = GameState::new();
        // B (healer) is played first, A (victim) second — B's death processes first
        let b = add_minion_to_board(&mut state, PlayerId::Player2, 0, 1);
        state.world_mut().set_deathrattle(
            b,
            Deathrattle(CardEffect::RestoreHealth {
                amount: 10,
                target: crate::core::effect::EffectTarget::AllFriendlyMinions,
            }),
        );
        let a = add_minion_to_board(&mut state, PlayerId::Player2, 0, 1);
        // P1 plays an AOE battlecry dealing 1 damage to both
        play_aoe_battlecry(&mut state, 1);

        assert_eq!(
            state.world().zone(a),
            Some(Zone::Play),
            "A must survive: healed above 0 before its death was processed"
        );
        // Damage dropped A to 0; the heal restored it to full (heals cap at
        // full health in the damage model)
        assert_eq!(state.world().effective_health(a), Some(Health(1)));
        assert_eq!(state.world().zone(b), Some(Zone::Graveyard));
        assert_eq!(state.step(), Step::Main);
    }

    #[test]
    fn deaths_process_in_play_order() {
        // The death phase processes deaths in play order: A (played first,
        // deathrattle draws) dies before B (deathrattle discards), so the drawn
        // card is discarded — if B processed first, the hand would keep it.
        use crate::core::component::Deathrattle;
        use crate::core::effect::CardEffect;
        let mut state = GameState::new();
        let a = add_minion_to_board(&mut state, PlayerId::Player2, 0, 1);
        state
            .world_mut()
            .set_deathrattle(a, Deathrattle(CardEffect::DrawCard { count: 1 }));
        let b = add_minion_to_board(&mut state, PlayerId::Player2, 0, 1);
        state
            .world_mut()
            .set_deathrattle(b, Deathrattle(CardEffect::DiscardRandomCard));
        // P2's deck with one card for A's draw
        {
            let world = state.world_mut();
            let e = world.spawn();
            world.set_card_type(e, CardType::Minion);
            world.set_cost(e, crate::core::component::Cost(0));
            world.set_player(e, PlayerId::Player2);
            world.set_zone(e, Zone::Deck);
            world.zones_mut().insert(Zone::Deck, PlayerId::Player2, e);
        }
        play_aoe_battlecry(&mut state, 1);

        assert_eq!(
            state.world().zones().len(Zone::Hand, PlayerId::Player2),
            0,
            "deaths must process in play order: A's draw is discarded by B"
        );
        assert_eq!(state.world().zone(a), Some(Zone::Graveyard));
        assert_eq!(state.world().zone(b), Some(Zone::Graveyard));
    }

    #[test]
    fn deathrattle_resolves_after_minion_removed() {
        // Death triggers see the dead minion already removed: a minion with a
        // FriendlyMinionDied trigger on ITSELF does not fire (the dead minion is
        // excluded), and the deathrattle resolves after the move to the graveyard.
        use crate::core::component::Deathrattle;
        use crate::core::effect::CardEffect;
        let mut state = GameState::new();
        let victim = add_minion_to_board(&mut state, PlayerId::Player2, 0, 1);
        state.world_mut().set_deathrattle(
            victim,
            Deathrattle(CardEffect::SummonMinion {
                card_id: "CLASSIC_001",
            }),
        );
        let hand_before = state.world().zones().len(Zone::Hand, PlayerId::Player1);
        let _ = hand_before;
        play_aoe_battlecry(&mut state, 1);
        // The deathrattle fired (a minion was summoned) — and the dead minion is gone
        assert_eq!(state.world().zone(victim), Some(Zone::Graveyard));
        assert_eq!(
            state
                .world()
                .zones()
                .iter(Zone::Play, PlayerId::Player2)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .count(),
            1,
            "deathrattle summon must resolve after the minion is removed"
        );
    }

    // ============================================================
    // G4 — enchantment layer
    // ============================================================

    /// Plays a custom P1 minion whose battlecry is `effect`, returning it.
    fn play_battlecry_minion(
        state: &mut GameState,
        effect: crate::core::effect::CardEffect,
    ) -> Entity {
        use crate::core::component::Battlecry;
        let e = {
            let world = state.world_mut();
            let e = world.spawn();
            world.set_health(e, Health(2));
            world.set_attack(e, Attack(1));
            world.set_cost(e, crate::core::component::Cost(1));
            world.set_card_type(e, CardType::Minion);
            world.set_player(e, PlayerId::Player1);
            world.set_attacks_used(e, AttacksUsed(0));
            world.set_battlecry(e, Battlecry(effect));
            world.set_zone(e, Zone::Hand);
            world.zones_mut().insert(Zone::Hand, PlayerId::Player1, e);
            e
        };
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                state,
                Action::PlayCard {
                    card: e,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        e
    }

    #[test]
    fn buffs_and_damage_are_enchantments_and_damage_component() {
        // G4: base stats stay untouched; buffs attach enchantments, damage
        // accumulates on the damage component.
        let mut state = GameState::new();
        let minion = add_minion_to_board(&mut state, PlayerId::Player1, 2, 5);
        state.world_mut().add_enchantment(
            minion,
            Enchantment {
                attack: 3,
                health: 2,
                cost: 0,
                expiry: EnchantmentExpiry::Permanent,
            },
        );
        state.world_mut().set_damage(minion, Damage(2));
        // Base values untouched
        assert_eq!(state.world().attack(minion), Some(Attack(2)));
        assert_eq!(state.world().health(minion), Some(Health(5)));
        // Effective values: base + enchantments − damage
        assert_eq!(state.world().effective_attack(minion), Some(Attack(5)));
        assert_eq!(state.world().effective_health(minion), Some(Health(5)));
    }

    #[test]
    fn silence_strips_enchantments_but_keeps_damage() {
        // G4: silence removes enchantments, keeping base stats and accumulated damage
        let mut state = GameState::new();
        let target = add_minion_to_board(&mut state, PlayerId::Player2, 2, 5);
        state.world_mut().add_enchantment(
            target,
            Enchantment {
                attack: 3,
                health: 3,
                cost: 0,
                expiry: EnchantmentExpiry::Permanent,
            },
        );
        state.world_mut().set_damage(target, Damage(2));
        assert_eq!(state.world().effective_attack(target), Some(Attack(5)));
        assert_eq!(state.world().effective_health(target), Some(Health(6)));
        // Silence via a custom battlecry
        play_battlecry_minion(
            &mut state,
            crate::core::effect::CardEffect::SilenceMinion {
                target: crate::core::effect::EffectTarget::AnyEnemyMinion,
            },
        );
        assert_eq!(
            state.world().effective_attack(target),
            Some(Attack(2)),
            "silence must strip enchantments, revealing the base attack"
        );
        assert_eq!(
            state.world().effective_health(target),
            Some(Health(3)),
            "silence keeps the accumulated damage (5 − 2)"
        );
        assert!(state.world().enchantments(target).is_none());
    }

    #[test]
    fn until_end_of_turn_enchantments_expire_at_wrap_up() {
        // G4: "until end of turn" enchantments expire at wrap-up; permanent
        // enchantments survive it.
        let mut state = GameState::new();
        let minion = add_minion_to_board(&mut state, PlayerId::Player1, 2, 3);
        state.world_mut().add_enchantment(
            minion,
            Enchantment {
                attack: 3,
                health: 0,
                cost: 0,
                expiry: EnchantmentExpiry::UntilEndOfTurn,
            },
        );
        state.world_mut().add_enchantment(
            minion,
            Enchantment {
                attack: 1,
                health: 0,
                cost: 0,
                expiry: EnchantmentExpiry::Permanent,
            },
        );
        assert_eq!(state.world().effective_attack(minion), Some(Attack(6)));
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        assert_eq!(
            state.world().effective_attack(minion),
            Some(Attack(3)),
            "the until-end-of-turn enchantment must expire at wrap-up"
        );
    }

    #[test]
    fn leaving_play_clears_enchantments_and_damage() {
        // G4: a minion leaving the battlefield loses enchantments and damage —
        // bounced minions return at full health with base stats.
        let mut state = GameState::new();
        let minion = add_minion_to_board(&mut state, PlayerId::Player1, 2, 5);
        state.world_mut().add_enchantment(
            minion,
            Enchantment {
                attack: 2,
                health: 3,
                cost: 0,
                expiry: EnchantmentExpiry::Permanent,
            },
        );
        state.world_mut().set_damage(minion, Damage(2));
        assert_eq!(state.world().effective_attack(minion), Some(Attack(4)));
        state
            .world_mut()
            .move_to_zone(minion, Zone::Hand)
            .expect("bounce");
        assert_eq!(state.world().effective_attack(minion), Some(Attack(2)));
        assert_eq!(state.world().effective_health(minion), Some(Health(5)));
        assert!(state.world().enchantments(minion).is_none());
        assert!(state.world().damage(minion).is_none());
    }

    #[test]
    fn shadowstep_cost_enchantment_applies_after_bounce() {
        // G4 end-to-end: Shadowstep bounces a buffed, damaged minion — the
        // bounce clears enchantments and damage, then the bounce effect
        // re-applies its cost reduction as a cost enchantment.
        use crate::cards::def::SHADOWSTEP;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_board(PlayerId::Player1, &crate::cards::def::BLOODFEN_RAPTOR);
        builder.add_minion_to_hand(PlayerId::Player1, &SHADOWSTEP);
        let mut state = builder.build();
        let minion = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .find(|&e| {
                state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| c.0 == "CLASSIC_001")
            })
            .expect("raptor on board");
        let shadowstep = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .find(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "ROGUE_019"))
            .expect("shadowstep in hand");
        // Buff +2/+2 and damage the minion
        state.world_mut().add_enchantment(
            minion,
            Enchantment {
                attack: 2,
                health: 2,
                cost: 0,
                expiry: EnchantmentExpiry::Permanent,
            },
        );
        state.world_mut().set_damage(minion, Damage(1));
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: shadowstep,
                    target: Some(minion),
                    position: None,
                },
            )
            .unwrap();
        assert_eq!(state.world().zone(minion), Some(Zone::Hand));
        assert_eq!(
            state.world().effective_attack(minion),
            Some(Attack(3)),
            "bounced minion must lose its buffs (raptor base attack)"
        );
        assert_eq!(
            state.world().effective_health(minion),
            Some(Health(2)),
            "bounced minion must be at full health (raptor base health)"
        );
        assert_eq!(
            state.world().effective_cost(minion),
            Some(crate::core::component::Cost(0)),
            "Shadowstep's cost reduction persists in hand (3 − 2)"
        );
    }

    // ============================================================
    // G5 — cost manager
    // ============================================================

    #[test]
    fn cost_modifier_stack_set_and_floor() {
        // G5: the cost stack composes base + enchantment deltas, then applies
        // set-to-value and floor modifiers.
        let mut state = GameState::new();
        let card = add_minion_to_hand(&mut state, PlayerId::Player1, 2, 3); // cost 3
        assert_eq!(state.world().effective_cost(card), Some(Cost(3)));
        // −2 enchantment → 1
        state.world_mut().add_enchantment(
            card,
            Enchantment {
                attack: 0,
                health: 0,
                cost: -2,
                expiry: EnchantmentExpiry::Permanent,
            },
        );
        assert_eq!(state.world().effective_cost(card), Some(Cost(1)));
        // Min(3) floor raises it back to 3
        state.world_mut().add_cost_modifier(
            card,
            CostModifier {
                kind: CostModifierKind::Min(3),
            },
        );
        assert_eq!(state.world().effective_cost(card), Some(Cost(3)));
        // Set(5) overrides the composed value
        state.world_mut().add_cost_modifier(
            card,
            CostModifier {
                kind: CostModifierKind::Set(5),
            },
        );
        assert_eq!(state.world().effective_cost(card), Some(Cost(5)));
        // Removing the stack restores the enchantment-composed cost
        state.world_mut().remove_cost_modifiers(card);
        assert_eq!(state.world().effective_cost(card), Some(Cost(1)));
    }

    #[test]
    fn cost_cannot_go_below_zero() {
        let mut state = GameState::new();
        let card = add_minion_to_hand(&mut state, PlayerId::Player1, 2, 3); // cost 3
        state.world_mut().add_enchantment(
            card,
            Enchantment {
                attack: 0,
                health: 0,
                cost: -10,
                expiry: EnchantmentExpiry::Permanent,
            },
        );
        assert_eq!(
            state.world().effective_cost(card),
            Some(Cost(0)),
            "costs cannot go below 0"
        );
    }

    #[test]
    fn play_cost_composes_player_level_modifiers() {
        // G5: cost::play_cost is the single composition — Kirin Tor Mage's
        // one-time free secret applies at the player level.
        use crate::cards::def::EXPLOSIVE_TRAP;
        use crate::cards::def::KIRIN_TOR_MAGE;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &EXPLOSIVE_TRAP);
        builder.add_minion_to_hand(PlayerId::Player1, &KIRIN_TOR_MAGE);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let mut state = builder.build();
        let trap = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .find(|&e| {
                state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| c.0 == "HUNTER_T01")
            })
            .expect("trap in hand");
        assert_eq!(
            crate::engine::cost::play_cost(&state, trap, PlayerId::Player1),
            Cost(2)
        );
        // Kirin Tor's one-time free secret (played first)
        let kirin = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .find(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "MAGE_021"))
            .expect("kirin in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: kirin,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        assert!(state.player(PlayerId::Player1).next_secret_free);
        assert_eq!(
            crate::engine::cost::play_cost(&state, trap, PlayerId::Player1),
            Cost(0),
            "the next secret costs 0"
        );
    }

    // ============================================================
    // G6 — choice system
    // ============================================================

    #[test]
    fn choose_one_choice_surfaces_and_resolves_branch() {
        // G6: a Choose One card surfaces a pending choice via apply_choices;
        // Action::Choose resolves the chosen branch.
        use crate::cards::def::CENARIUS;
        use crate::engine::game::Resolution;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &CENARIUS);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let mut state = builder.build();
        let cenarius = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("cenarius in hand");
        let engine = crate::engine::game::GameEngine::new();
        let resolution = engine
            .apply_choices(
                &mut state,
                Action::PlayCard {
                    card: cenarius,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        let Resolution::NeedsChoice { choice } = resolution else {
            panic!("a choose-one card must surface a choice");
        };
        assert_eq!(choice.kind, ChoiceKind::ChooseOne);
        assert_eq!(choice.card, cenarius);
        assert_eq!(choice.options.len(), 2);
        // Resolve the second branch (summon two treants)
        let resolution = engine
            .apply_choices(
                &mut state,
                Action::Choose {
                    choice_id: choice.id,
                    option: 1,
                },
            )
            .unwrap();
        assert!(matches!(resolution, Resolution::Done(_)));
        let minions: SmallList<Entity> = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
            .collect();
        assert_eq!(
            minions.len(),
            3,
            "the second branch summons two treants next to Cenarius"
        );
    }

    #[test]
    fn choose_one_invalid_choice_rejected() {
        use crate::cards::def::CENARIUS;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &CENARIUS);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let mut state = builder.build();
        let cenarius = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("cenarius in hand");
        let engine = crate::engine::game::GameEngine::new();
        let _ = engine
            .apply_choices(
                &mut state,
                Action::PlayCard {
                    card: cenarius,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        // A stale choice id is rejected
        let err = engine
            .apply_choices(
                &mut state,
                Action::Choose {
                    choice_id: 999,
                    option: 0,
                },
            )
            .unwrap_err();
        assert_eq!(err, EngineError::InvalidChoice);
    }

    #[test]
    fn discover_choice_picks_a_pool_card() {
        // G6: an AddRandomCardToHand effect surfaces a Discover choice with the
        // pool as options; the chosen card is added to hand.
        use crate::engine::game::Resolution;
        let mut state = GameState::new();
        let source = add_minion_to_board(&mut state, PlayerId::Player1, 2, 2);
        let dream_pool = crate::cards::pool::pool_cards(crate::core::effect::RandomPool::Dream);
        assert!(!dream_pool.is_empty());
        let chosen = dream_pool[0];
        // Simulate the discover flow: set the pending choice as the resolver would
        let choice_id = state.set_pending_choice(
            ChoiceKind::Discover,
            source,
            dream_pool
                .iter()
                .map(|c| format!("{} ({})", c.name, c.id))
                .collect(),
            dream_pool.iter().map(|c| c.id.to_string()).collect(),
        );
        let engine = crate::engine::game::GameEngine::new();
        let resolution = engine
            .apply_choices(
                &mut state,
                Action::Choose {
                    choice_id,
                    option: 0,
                },
            )
            .unwrap();
        assert!(matches!(resolution, Resolution::Done(_)));
        let hand: SmallList<Entity> = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .collect();
        assert_eq!(hand.len(), 1);
        assert_eq!(
            state.world().card_id(hand[0]),
            Some(crate::core::component::CardId(chosen.id)),
            "the chosen pool card is added to hand"
        );
    }

    #[test]
    fn summon_position_places_minion() {
        // G6: PlayCard's position picks the board slot (0 = leftmost)
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_board(PlayerId::Player1, &BLOODFEN_RAPTOR);
        builder.add_minion_to_hand(PlayerId::Player1, &BLOODFEN_RAPTOR);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let mut state = builder.build();
        let existing = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .next()
            .expect("board minion");
        let card = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("hand minion");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card,
                    target: None,
                    position: Some(0),
                },
            )
            .unwrap();
        let order: SmallList<Entity> = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .collect();
        assert_eq!(order[0], card, "position 0 places the minion leftmost");
        assert_eq!(order[1], existing);
    }

    #[test]
    fn hero_power_explicit_target() {
        // G6: Action::HeroPower carries an explicit target (engine-random otherwise)
        use crate::core::effect::CardEffect;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.set_hero_power(
            PlayerId::Player1,
            2,
            CardEffect::DealDamage {
                amount: 2,
                target: crate::core::effect::EffectTarget::AnyEnemy,
            },
        );
        builder.set_mana(PlayerId::Player1, 10, 10);
        let mut state = builder.build();
        let hero = state.player(PlayerId::Player1).hero;
        let enemy_minion = add_minion_to_board(&mut state, PlayerId::Player2, 2, 4);
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::HeroPower {
                    hero,
                    target: Some(enemy_minion),
                },
            )
            .unwrap();
        assert_eq!(
            state.world().effective_health(enemy_minion),
            Some(Health(2)),
            "the explicit hero power target takes the damage"
        );
    }

    // ============================================================
    // G7 — opening flow
    // ============================================================

    /// Spawns a deck card with a known identity for the player (no shuffle).
    fn add_raw_deck_card(state: &mut GameState, player: PlayerId, card_id: &'static str) -> Entity {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_card_id(e, crate::core::component::CardId(card_id));
        world.set_card_type(e, CardType::Minion);
        world.set_attack(e, Attack(1));
        world.set_health(e, Health(1));
        world.set_cost(e, crate::core::component::Cost(1));
        world.set_player(e, player);
        world.set_zone(e, Zone::Deck);
        world.zones_mut().insert(Zone::Deck, player, e);
        e
    }

    #[test]
    fn top_draw_uses_deck_order() {
        // G7: draws take the top card of the ordered deck (the random element
        // moved to the game-start shuffle).
        let mut state = GameState::new();
        add_raw_deck_card(&mut state, PlayerId::Player2, "NEUTRAL_001");
        add_raw_deck_card(&mut state, PlayerId::Player2, "NEUTRAL_002");
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        // P2 drew the top card (first spawned)
        let hand: SmallList<Entity> = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player2)
            .collect();
        assert_eq!(hand.len(), 1);
        assert_eq!(
            state.world().card_id(hand[0]),
            Some(crate::core::component::CardId("NEUTRAL_001"))
        );
    }

    #[test]
    fn begin_game_deals_starting_hands_and_coin() {
        // G7: 3 cards to the first player, 4 + The Coin to the second
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        for _ in 0..10 {
            builder.add_minion_to_deck(PlayerId::Player1, &BLOODFEN_RAPTOR);
            builder.add_minion_to_deck(PlayerId::Player2, &BLOODFEN_RAPTOR);
        }
        let mut state = builder.build();
        assert!(state.pending_choice().is_none());
        state.begin_game();
        assert_eq!(state.world().zones().len(Zone::Hand, PlayerId::Player1), 3);
        assert_eq!(state.world().zones().len(Zone::Hand, PlayerId::Player2), 5);
        let coin = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player2)
            .find(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "GAME_005"));
        assert!(coin.is_some(), "the second player gets The Coin");
        // P1's mulligan surfaces as a pending choice
        let choice = state.pending_choice().expect("mulligan pending");
        assert_eq!(choice.kind, ChoiceKind::Mulligan);
        assert_eq!(choice.options.len(), 4, "3 replace options + keep all");
    }

    #[test]
    fn mulligan_replaces_a_card() {
        // G7: resolving the mulligan replaces the chosen card — the hand keeps
        // its size and the replaced card returns to the deck.
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        for _ in 0..10 {
            builder.add_minion_to_deck(PlayerId::Player1, &BLOODFEN_RAPTOR);
            builder.add_minion_to_deck(PlayerId::Player2, &BLOODFEN_RAPTOR);
        }
        let mut state = builder.build();
        state.begin_game();
        let choice_id = state.pending_choice().expect("mulligan").id;
        let replaced = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("starting card");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply_choices(
                &mut state,
                Action::Choose {
                    choice_id,
                    option: 1, // replace the first starting card
                },
            )
            .unwrap();
        // Hand size unchanged; the replaced card went back to the deck
        assert_eq!(state.world().zones().len(Zone::Hand, PlayerId::Player1), 3);
        assert_eq!(state.world().zone(replaced), Some(Zone::Deck));
        // P2's mulligan surfaces next
        assert_eq!(
            state.pending_choice().map(|c| c.kind),
            Some(ChoiceKind::Mulligan)
        );
        // Resolve P2's mulligan (keep all) — the opening finishes
        let choice_id = state.pending_choice().expect("mulligan").id;
        engine
            .apply_choices(
                &mut state,
                Action::Choose {
                    choice_id,
                    option: 0,
                },
            )
            .unwrap();
        assert!(state.pending_choice().is_none(), "opening complete");
        assert_eq!(state.step(), Step::Main);
        // Official rule: as the opening finishes, the first player draws the
        // 4th card as their turn 1 starts
        assert_eq!(state.world().zones().len(Zone::Hand, PlayerId::Player1), 4);
    }

    #[test]
    fn coin_gives_mana_this_turn_only() {
        // G7: The Coin adds +1 mana this turn without a permanent crystal
        use crate::cards::def::THE_COIN;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &THE_COIN);
        builder.set_mana(PlayerId::Player1, 1, 1);
        let mut state = builder.build();
        let coin = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("coin in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: coin,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        assert_eq!(state.player(PlayerId::Player1).current_mana, 2);
        assert_eq!(
            state.player(PlayerId::Player1).mana_crystals,
            1,
            "the coin does not add a permanent crystal"
        );
    }

    // ============================================================
    // G8 — secret interception at step boundaries
    // ============================================================

    /// Adds a custom spell to the player's hand (type + effect set manually).
    fn add_custom_spell(
        state: &mut GameState,
        player: PlayerId,
        effect: crate::core::effect::CardEffect,
    ) -> Entity {
        use crate::core::component::Battlecry;
        let world = state.world_mut();
        let e = world.spawn();
        world.set_card_type(e, CardType::Spell);
        world.set_cost(e, crate::core::component::Cost(1));
        world.set_player(e, player);
        world.set_battlecry(e, Battlecry(effect));
        world.set_zone(e, Zone::Hand);
        world.zones_mut().insert(Zone::Hand, player, e);
        e
    }

    #[test]
    fn counterspell_negates_enemy_spell() {
        // G8: Counterspell intercepts the enemy spell BEFORE its effect resolves
        use crate::cards::def::COUNTERSPELL;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &COUNTERSPELL);
        builder.set_mana(PlayerId::Player1, 10, 10);
        builder.set_mana(PlayerId::Player2, 10, 10);
        let minion = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 3, 3);
        let mut state = builder.build();
        let counterspell = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("counterspell in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: counterspell,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        // P2 casts a damaging spell — it is negated
        state.set_active_player(PlayerId::Player2);
        let spell = add_custom_spell(
            &mut state,
            PlayerId::Player2,
            crate::core::effect::CardEffect::DealDamage {
                amount: 5,
                target: crate::core::effect::EffectTarget::AnyEnemyMinion,
            },
        );
        let log = engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: spell,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        assert!(
            log.iter()
                .any(|e| matches!(e, Event::SecretRevealed { .. })),
            "counterspell must be revealed"
        );
        assert_eq!(
            state.world().effective_health(minion),
            Some(Health(3)),
            "the countered spell's effect must not resolve"
        );
        assert_eq!(
            state.world().zone(spell),
            Some(Zone::Graveyard),
            "the countered spell still goes to the graveyard"
        );
    }

    #[test]
    fn spellbender_does_not_redirect_aoe_spells() {
        // G8: Spellbender only redirects single-target effects — AOE spells hit
        // normally (including the token), matching HS
        use crate::cards::def::SPELLBENDER;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &SPELLBENDER);
        builder.set_mana(PlayerId::Player1, 10, 10);
        builder.set_mana(PlayerId::Player2, 10, 10);
        let minion = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 3, 3);
        let mut state = builder.build();
        let spellbender = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("spellbender in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: spellbender,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        state.set_active_player(PlayerId::Player2);
        let spell = add_custom_spell(
            &mut state,
            PlayerId::Player2,
            crate::core::effect::CardEffect::DealDamage {
                amount: 2,
                target: crate::core::effect::EffectTarget::AllEnemyMinions,
            },
        );
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: spell,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        assert_eq!(
            state.world().effective_health(minion),
            Some(Health(1)),
            "AOE spells are not redirected by Spellbender"
        );
    }

    // ============================================================
    // G9 — target legality at resolution
    // ============================================================

    #[test]
    fn stealthed_minion_cannot_be_explicitly_targeted() {
        // G9: single-target effects cannot target stealthed characters — an
        // explicit stealthed target makes the effect fizzle (no random fallback)
        use crate::cards::def::FROSTBOLT;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &FROSTBOLT);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let stealthed = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 5, 3);
        let visible = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 5, 3);
        let mut state = builder.build();
        state
            .world_mut()
            .set_stealth(stealthed, crate::core::component::Stealth);
        let frostbolt = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("frostbolt in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: frostbolt,
                    target: Some(stealthed),
                    position: None,
                },
            )
            .unwrap();
        assert_eq!(
            state.world().effective_health(stealthed),
            Some(Health(5)),
            "the stealthed minion cannot be targeted — the spell fizzles"
        );
        assert_eq!(
            state.world().effective_health(visible),
            Some(Health(5)),
            "no random fallback to another target"
        );
    }

    // ============================================================
    // F1 — overload mana lock
    // ============================================================

    #[test]
    fn overload_locks_mana_on_next_turn() {
        // F1: Lightning Bolt (overload 1) locks 1 mana on the owner's NEXT
        // turn — applied at the ManaRefill step and cleared afterwards.
        use crate::cards::def::LIGHTNING_BOLT;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &LIGHTNING_BOLT);
        // Turn 1: 1 mana
        let mut state = builder.build();
        let bolt = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("lightning bolt in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: bolt,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        assert_eq!(
            state.player(PlayerId::Player1).overload_locked,
            1,
            "playing an overload card locks mana for the next turn"
        );
        // P2's turn, then P1's next turn (turn 3): crystals 2, mana 2 − 1 lock
        engine.apply(&mut state, Action::EndTurn).unwrap();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        assert_eq!(state.player(PlayerId::Player1).mana_crystals, 2);
        assert_eq!(
            state.player(PlayerId::Player1).current_mana,
            1,
            "the overload lock reduces the next turn's mana"
        );
        assert_eq!(
            state.player(PlayerId::Player1).overload_locked,
            0,
            "the lock is consumed at the refill"
        );
    }

    // ============================================================
    // F2 — stealth fidelity
    // ============================================================

    #[test]
    fn attacking_breaks_stealth() {
        // F2: a stealthed minion loses stealth when it attacks
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 3);
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 2, 5);
        state
            .world_mut()
            .set_stealth(attacker, crate::core::component::Stealth);
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(&mut state, Action::Attack { attacker, defender })
            .unwrap();
        assert!(
            state.world().stealth(attacker).is_none(),
            "attacking breaks the attacker's stealth"
        );
    }

    // ============================================================
    // F4 — per-effect fidelity audit
    // ============================================================

    #[test]
    fn single_target_destroy_hits_one_random_minion() {
        // F4: Assassinate-style destroy with no explicit target destroys ONE
        // random matching minion — never all of them.
        use crate::cards::def::ASSASSINATE;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &ASSASSINATE);
        builder.set_mana(PlayerId::Player1, 10, 10);
        builder.add_minion_to_board(PlayerId::Player2, &crate::cards::def::BLOODFEN_RAPTOR);
        builder.add_minion_to_board(PlayerId::Player2, &crate::cards::def::BLOODFEN_RAPTOR);
        builder.add_minion_to_board(PlayerId::Player2, &crate::cards::def::BLOODFEN_RAPTOR);
        let mut state = builder.build();
        let assassinate = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("assassinate in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: assassinate,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        assert_eq!(
            state
                .world()
                .zones()
                .iter(Zone::Play, PlayerId::Player2)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .count(),
            2,
            "a single-target destroy removes exactly one minion"
        );
    }

    #[test]
    fn execute_destroys_one_damaged_minion() {
        // F4: Execute (destroy a damaged enemy minion) hits exactly one damaged
        // minion; undamaged minions are untouched.
        use crate::cards::def::EXECUTE;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &EXECUTE);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let damaged = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 4, 3);
        builder.add_minion_to_board(PlayerId::Player2, &crate::cards::def::BLOODFEN_RAPTOR);
        let mut state = builder.build();
        state
            .world_mut()
            .set_damage(damaged, crate::core::component::Damage(1));
        let execute = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("execute in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: execute,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        assert_eq!(
            state.world().zone(damaged),
            Some(Zone::Graveyard),
            "the damaged minion is destroyed"
        );
        let remaining = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player2)
            .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
            .count();
        assert_eq!(remaining, 1, "only the damaged minion is destroyed");
    }

    #[test]
    fn aura_stacking_is_additive() {
        // F4: multiple auras stack additively (two Stormwind Champions: +2/+2)
        use crate::cards::def::STORMWIND_CHAMPION;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_board(PlayerId::Player1, &STORMWIND_CHAMPION);
        builder.add_minion_to_board(PlayerId::Player1, &STORMWIND_CHAMPION);
        builder.add_minion_to_board(PlayerId::Player1, &crate::cards::def::BLOODFEN_RAPTOR);
        let state = builder.build();
        let target = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .find(|&e| {
                state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| c.0 == "CLASSIC_001")
            })
            .expect("raptor on board");
        assert_eq!(
            state.world().effective_attack(target),
            Some(Attack(5)),
            "two champions grant +2/+2 (additive aura stacking)"
        );
        assert_eq!(
            state.world().effective_health(target),
            Some(Health(4)),
            "two champions grant +2/+2 (additive aura stacking)"
        );
    }
}
