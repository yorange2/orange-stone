//! Secret system — checking and triggering logic for Secret components.
//!
//! Secret cards go into `Zone::SetAside` when played (hidden from the opponent).
//! After each event is processed, all secrets in SetAside are iterated and
//! checked for a matching trigger. Matched secrets are revealed and their
//! effects executed.
//!
//! # Trigger order
//!
//! When multiple secrets of the same player trigger at once, they trigger in
//! play order (the order in SetAside). Between players, the active player's
//! secrets trigger first.

use crate::core::component::{CardType, Secret, SecretTrigger};
use crate::core::effect::CardEffect;
use crate::core::entity::Entity;
use crate::core::event::{Event, EventQueue};
use crate::core::player::PlayerId;
use crate::core::state::GameState;
use crate::core::zone::Zone;

/// Checks all secrets, triggering those matching the current event.
///
/// Called after `apply_event` processes each event.
/// Returns the number of revealed secrets.
pub fn check_secrets(state: &mut GameState, queue: &mut EventQueue, event: &Event) -> usize {
    let active = state.active_player();

    // Collect all secrets in SetAside (in play order, active player first)
    let secrets: Vec<(Entity, PlayerId, Secret)> = {
        let world = state.world();
        let mut active_secrets = Vec::new();
        let mut opponent_secrets = Vec::new();
        // SetAside is a shared zone; sort by player manually
        for entity in world.zones().iter(Zone::SetAside, active) {
            if let Some(secret) = world.secret(entity) {
                if let Some(owner) = world.player(entity) {
                    if owner == active {
                        active_secrets.push((entity, owner, secret));
                    } else {
                        opponent_secrets.push((entity, owner, secret));
                    }
                }
            }
        }
        // Active player triggers first
        active_secrets.extend(opponent_secrets);
        active_secrets
    };

    let mut triggered = 0;

    for (entity, player, secret) in &secrets {
        if matches_trigger(secret.trigger, event, state, *player) {
            // Reveal the secret: move from SetAside to Graveyard
            let _ = state.world_mut().move_to_zone(*entity, Zone::Graveyard);
            queue.push(Event::SecretRevealed {
                player: *player,
                secret: *entity,
            });
            // Resolve the secret effect (some effects need trigger event context, e.g. Snipe/Misdirection)
            resolve_secret_effect(state, queue, event, *entity, *player, secret.effect);
            triggered += 1;
        }
    }

    triggered
}

/// Checks whether a secret's trigger condition matches the current event.
fn matches_trigger(
    trigger: SecretTrigger,
    event: &Event,
    state: &GameState,
    owner: PlayerId,
) -> bool {
    match trigger {
        SecretTrigger::AfterFriendlyAttacked => {
            // A friendly character was attacked
            matches_after_friendly_attacked(event, state, owner)
        }
        SecretTrigger::AfterEnemyMinionPlayed => {
            matches!(event, Event::MinionSummoned { player, .. } if *player != owner)
        }
        SecretTrigger::AfterEnemyHeroAttacks => {
            // The enemy hero attacks
            matches_after_enemy_hero_attacks(event, state, owner)
        }
        SecretTrigger::OnFriendlyTurnStart => {
            matches!(event, Event::TurnStarted { player } if *player == owner)
        }
        SecretTrigger::AfterMinionDied => {
            // Any minion dies (Phase 5 may restrict to enemy minions)
            matches!(event, Event::MinionDied { .. })
        }
        SecretTrigger::WhenEnemySpellCast => {
            // The enemy casts a spell (a spell card was played)
            matches!(event, Event::CardPlayed { player, card, .. } if *player != owner && state.world().card_type(*card) == Some(CardType::Spell))
        }
        SecretTrigger::WhenEnemyMinionAttacksHero => {
            matches_enemy_minion_attacks_hero(event, state, owner)
        }
        SecretTrigger::WhenEnemyAttacksHero => {
            // Any enemy character (minion or hero) attacks own hero
            matches_when_enemy_attacks_hero(event, state, owner)
        }
        SecretTrigger::WhenEnemyAttacks => {
            // Any enemy character declares an attack
            matches!(event, Event::AttackDeclared { attacker, .. } if state.world().player(*attacker).is_some_and(|p| p != owner))
        }
        SecretTrigger::WhenFriendlyMinionDamaged => {
            // A friendly minion takes damage
            matches!(event, Event::DamageDealt { target, amount, .. } if *amount > 0 && state.world().player(*target).is_some_and(|p| p == owner) && state.world().card_type(*target) == Some(CardType::Minion))
        }
    }
}

/// Checks whether an enemy minion attacks own hero.
fn matches_enemy_minion_attacks_hero(event: &Event, state: &GameState, owner: PlayerId) -> bool {
    use crate::core::component::CardType;
    if let Event::AttackDeclared { attacker, defender } = event {
        let hero = state.player(owner).hero;
        *defender == hero
            && state.world().card_type(*attacker) == Some(CardType::Minion)
            && state.world().player(*attacker) == Some(owner.opponent())
    } else {
        false
    }
}

/// Checks the AfterFriendlyAttacked trigger condition.
fn matches_after_friendly_attacked(event: &Event, state: &GameState, owner: PlayerId) -> bool {
    let Event::AttackDeclared { defender, .. } = event else {
        return false;
    };
    // The defender is a friendly character
    state.world().player(*defender).is_some_and(|p| p == owner)
}

/// Checks the AfterEnemyHeroAttacks trigger condition.
fn matches_after_enemy_hero_attacks(event: &Event, state: &GameState, owner: PlayerId) -> bool {
    let Event::AttackDeclared { attacker, .. } = event else {
        return false;
    };
    // The attacker is the enemy hero
    state.world().card_type(*attacker) == Some(CardType::Hero)
        && state.world().player(*attacker).is_some_and(|p| p != owner)
}

/// Checks whether any enemy character attacks own hero (Misdirection trigger condition).
fn matches_when_enemy_attacks_hero(event: &Event, state: &GameState, owner: PlayerId) -> bool {
    let Event::AttackDeclared { attacker, defender } = event else {
        return false;
    };
    let hero = state.player(owner).hero;
    *defender == hero && state.world().player(*attacker).is_some_and(|p| p != owner)
}

/// Resolves a secret effect.
///
/// Some secret effects depend on the triggering event's context (Snipe needs
/// the just-played minion; Misdirection / Noble Sacrifice / Spellbender need
/// to redirect pending events in the queue); these are handled here. All
/// other effects are delegated to `trigger::resolve_effect`.
fn resolve_secret_effect(
    state: &mut GameState,
    queue: &mut EventQueue,
    event: &Event,
    entity: Entity,
    player: PlayerId,
    effect: CardEffect,
) {
    match effect {
        CardEffect::DamagePlayedMinion { amount } => {
            // Snipe: deal damage to the just-played minion
            if let Event::MinionSummoned { minion, .. } = event {
                queue.push(Event::DamageDealt {
                    source: entity,
                    target: *minion,
                    amount,
                });
            }
        }
        CardEffect::RedirectAttackToRandomCharacter => {
            resolve_misdirection(state, queue, event, player);
        }
        CardEffect::SummonAndRedirectAttack { card_id } => {
            resolve_noble_sacrifice(state, queue, event, player, card_id);
        }
        CardEffect::SummonSpellbender => {
            resolve_spellbender(state, queue, event, player);
        }
        _ => crate::engine::trigger::resolve_effect(state, queue, entity, player, effect, None),
    }
}

/// Misdirection: redirects an attack to another random character (including
/// the attacker itself, excluding own hero).
///
/// Only the defender of the `ResolveAttack` event is replaced; retaliation
/// damage is computed automatically by the resolution logic from the new
/// target's current state.
fn resolve_misdirection(
    state: &mut GameState,
    queue: &mut EventQueue,
    event: &Event,
    owner: PlayerId,
) {
    let Event::AttackDeclared { attacker, .. } = event else {
        return;
    };
    let hero = state.player(owner).hero;
    // Collect all characters of both players (heroes + minions, including stealthed), excluding own hero
    let mut candidates: Vec<Entity> = [owner, owner.opponent()]
        .iter()
        .flat_map(|&pid| {
            state
                .world()
                .zones()
                .iter(Zone::Play, pid)
                .filter(|&e| {
                    let ct = state.world().card_type(e);
                    ct == Some(CardType::Minion) || ct == Some(CardType::Hero)
                })
                .collect::<Vec<_>>()
        })
        .collect();
    candidates.retain(|&e| e != hero);
    if candidates.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(candidates.len());
    let new_target = candidates[idx];
    // Redirect the pending attack
    queue.redirect_attack(*attacker, hero, new_target);
}

/// Noble Sacrifice: summons a defender as the attack's new target.
///
/// Only the defender of the `ResolveAttack` event is replaced; the original
/// defender's retaliation is automatically superseded by the new defender's
/// (computed from the new target's current state by the resolution logic).
fn resolve_noble_sacrifice(
    state: &mut GameState,
    queue: &mut EventQueue,
    event: &Event,
    owner: PlayerId,
    card_id: &str,
) {
    let Event::AttackDeclared { attacker, defender } = event else {
        return;
    };
    let Some(defender_minion) =
        crate::engine::trigger::resolve_summon(state, queue, *defender, owner, card_id)
    else {
        // Board is full; cannot summon a defender
        return;
    };
    // Redirect the attack to the defender
    queue.redirect_attack(*attacker, *defender, defender_minion);
}

/// Spellbender: summons a 1/3 minion and redirects pending damage from enemy spells targeting minions to it.
fn resolve_spellbender(
    state: &mut GameState,
    queue: &mut EventQueue,
    event: &Event,
    owner: PlayerId,
) {
    let Event::CardPlayed { card, .. } = event else {
        return;
    };
    let Some(spellbender) =
        crate::engine::trigger::resolve_summon(state, queue, *card, owner, "MAGE_019t")
    else {
        return;
    };
    // Redirect pending damage whose source is the spell and whose target is a minion
    queue.redirect_damages(
        |s, t| s == *card && state.world().card_type(t) == Some(CardType::Minion),
        spellbender,
    );
}
