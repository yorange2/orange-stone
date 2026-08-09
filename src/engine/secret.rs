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

use crate::core::component::{CardType, Damage, Immune, Secret, SecretTrigger};
use crate::core::effect::CardEffect;
use crate::core::entity::Entity;
use crate::core::event::{Event, EventQueue};
use crate::core::player::PlayerId;
use crate::core::small_list::SmallList;
use crate::core::state::GameState;
use crate::core::zone::Zone;

/// Reveals a secret: moves it to the graveyard, emits `Event::SecretRevealed`,
/// and fires `FriendlySecretRevealed` triggers (Eaglehorn Bow — +1 Durability
/// whenever a friendly Secret is revealed).
fn reveal_secret(state: &mut GameState, queue: &mut EventQueue, owner: PlayerId, entity: Entity) {
    let _ = state.world_mut().move_to_zone(entity, Zone::Graveyard);
    queue.push(Event::SecretRevealed {
        player: owner,
        secret: entity,
    });
    crate::engine::rules::fire_triggers(
        state,
        queue,
        crate::core::component::TriggerEvent::FriendlySecretRevealed,
        owner,
        Some(entity),
        None,
    );
}

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
            // Reveal the secret (moves it, emits SecretRevealed, fires
            // FriendlySecretRevealed triggers — Eaglehorn Bow)
            reveal_secret(state, queue, *player, *entity);
            // Resolve the secret effect (some effects need trigger event context,
            // e.g. Snipe/Misdirection; negation-only secrets have no effect)
            if let Some(effect) = secret.effect {
                resolve_secret_effect(state, queue, event, *entity, *player, effect);
            }
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
        SecretTrigger::WhenFriendlyHeroDamaged => {
            // The friendly hero takes damage (Eye for an Eye)
            let hero = state.player(owner).hero;
            matches!(event, Event::DamageDealt { target, amount, .. } if *amount > 0 && *target == hero)
        }
        SecretTrigger::AfterEnemyPlaysThreeCards => {
            // Rat Trap (Core Set W5) — the opponent played three cards this turn
            matches!(event, Event::CardPlayed { player, .. }
                if *player == owner.opponent()
                    && state.player(*player).cards_played_this_turn >= 3)
        }
        SecretTrigger::WhenFriendlyHeroFatallyDamaged => {
            // The friendly hero takes FATAL damage (Ice Block) — the damage
            // must have brought the hero to 0 or below for the secret to fire
            let hero = state.player(owner).hero;
            matches!(event, Event::DamageDealt { target, amount, .. } if *amount > 0 && *target == hero && state.world().effective_health(hero).is_some_and(|h| h.is_dead()))
        }
        SecretTrigger::WhenFriendlyMinionDiedTurnAfterPlayed => {
            // Timecode the End (M3-W2a — TIME_620): a friendly minion died
            // while carrying the played-this-turn marker (the "turn after it
            // was played" wording is approximated as "any later turn" — the
            // marker survives until the owner's next turn start, §20).
            matches!(event, Event::MinionDied { minion }
                if state.world().player(*minion).is_some_and(|p| p == owner)
                    && state.world().played_this_turn(*minion).is_some())
        }
        SecretTrigger::WhenEnemyTurnEnds => {
            // Flames of Infinity (M3-W3 — END_024): "When your enemy's
            // turn ends, deal INFINITE damage to their highest Health
            // minion" — the enemy's `Event::TurnEnded` reveals the secret
            // (§22).
            matches!(event, Event::TurnEnded { player } if *player == owner.opponent())
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
        CardEffect::SummonMinion { card_id } => {
            // Rat Trap — summon the token when the secret fires
            let _ = crate::engine::trigger::resolve_summon(state, queue, entity, player, card_id);
        }
        CardEffect::SummonOasisWaterElemental => {
            // Oasis Ally — summon a 3/6 Water Elemental when a friendly
            // minion is attacked
            let _ = crate::engine::trigger::resolve_summon(
                state,
                queue,
                entity,
                player,
                "CORE_BAR_812t",
            );
        }
        CardEffect::DestroyRandomEnemyMinion => {
            // Pressure Plate — destroy a random enemy minion
            let minions: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Play, player.opponent())
                .filter(|&e| {
                    state.world().card_type(e) == Some(crate::core::component::CardType::Minion)
                })
                .collect();
            if let Some(&target) = minions.iter().next() {
                let hp = state.world().effective_health(target).map_or(0, |h| h.0);
                queue.push(Event::DamageDealt {
                    source: entity,
                    target,
                    amount: hp.max(1),
                });
            }
        }
        CardEffect::DamagePlayedMinionAndExcess { amount } => {
            // Explosive Runes (Core Set W2): deal `amount` to the just-played
            // minion; any excess carries over to the enemy hero
            if let Event::MinionSummoned { minion, .. } = event {
                let target_hp = state
                    .world()
                    .effective_health(*minion)
                    .map(|h| h.0)
                    .unwrap_or(0);
                let to_minion = amount.min(target_hp);
                queue.push(Event::DamageDealt {
                    source: entity,
                    target: *minion,
                    amount: to_minion,
                });
                let excess = amount - to_minion;
                if excess > 0 {
                    let enemy_hero = state.player(state.active_player()).hero;
                    queue.push(Event::DamageDealt {
                        source: entity,
                        target: enemy_hero,
                        amount: excess,
                    });
                }
            }
        }
        CardEffect::ReflectDamage => {
            // Eye for an Eye — the enemy hero takes the same damage the
            // friendly hero just took
            if let Event::DamageDealt { target, amount, .. } = event {
                let hero = state.player(player).hero;
                if *target == hero {
                    let enemy_hero = state.player(player.opponent()).hero;
                    queue.push(Event::DamageDealt {
                        source: entity,
                        target: enemy_hero,
                        amount: *amount,
                    });
                }
            }
        }
        CardEffect::PreventFatalDamageAndImmune => {
            // Ice Block — the hero survives the fatal hit at 1 health and
            // becomes Immune until the end of the turn
            if let Event::DamageDealt { .. } = event {
                let hero = state.player(player).hero;
                let world = state.world();
                let cur = world.effective_health(hero);
                let damage_old = world.damage(hero).unwrap_or(Damage(0)).0;
                if let Some(cur) = cur {
                    // Undo the fatal damage: health = undamaged total − 1
                    state
                        .world_mut()
                        .set_damage(hero, Damage(cur.0 + damage_old - 1));
                }
                state.world_mut().set_immune(hero, Immune);
                // Drop the pending GameOver — the fatal hit was prevented
                queue.retain(|e| !matches!(e, Event::GameOver { .. }));
            }
        }
        CardEffect::ResurrectDiedMinion => {
            // Redemption — resummon the minion that just died with 1 Health
            if let Event::MinionDied { minion } = event {
                if state.world().zone(*minion) == Some(Zone::Graveyard) {
                    let board_count = state
                        .world()
                        .zones()
                        .iter(Zone::Play, player)
                        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                        .count();
                    if board_count < crate::engine::rules::MAX_BOARD_SIZE {
                        let _ = state.world_mut().move_to_zone(*minion, Zone::Play);
                        let world = state.world_mut();
                        let base = world
                            .health(*minion)
                            .unwrap_or(crate::core::component::Health(0))
                            .0;
                        let dmg = (base - 1).max(0);
                        if dmg > 0 {
                            world.set_damage(*minion, crate::core::component::Damage(dmg));
                        } else {
                            world.remove_damage(*minion);
                        }
                        world.set_attacks_used(*minion, crate::core::component::AttacksUsed(1));
                    }
                }
            }
        }
        CardEffect::ResurrectDiedMinionFull => {
            // Timecode the End (M3-W2a — TIME_620): "resurrect it" — the
            // died minion is resummoned at FULL Health (unlike Redemption's
            // 1-Health shape), can attack, and its played-this-turn marker
            // is removed (the resurrected minion no longer counts as
            // played this turn for this secret — a second resurrection
            // would otherwise fire off the same death).
            if let Event::MinionDied { minion } = event {
                if state.world().zone(*minion) == Some(Zone::Graveyard) {
                    let board_count = state
                        .world()
                        .zones()
                        .iter(Zone::Play, player)
                        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                        .count();
                    if board_count < crate::engine::rules::MAX_BOARD_SIZE {
                        state.world_mut().remove_damage(*minion);
                        let _ = state.world_mut().move_to_zone(*minion, Zone::Play);
                        state
                            .world_mut()
                            .set_attacks_used(*minion, crate::core::component::AttacksUsed(0));
                        state.world_mut().remove_played_this_turn(*minion);
                    }
                }
            }
        }
        CardEffect::SetPlayedMinionHealth { health } => {
            // Repentance: set the just-played enemy minion's health to 1 —
            // damage is applied so the effective health equals the value
            if let Event::MinionSummoned { minion, .. } = event {
                let cur = state
                    .world()
                    .effective_health(*minion)
                    .unwrap_or(crate::core::component::Health(1));
                let dmg = cur.0 - health;
                if dmg > 0 {
                    let world = state.world_mut();
                    let existing = world.damage(*minion).map_or(0, |d| d.0);
                    world.set_damage(*minion, crate::core::component::Damage(existing + dmg));
                }
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
        _ => {
            crate::engine::trigger::resolve_effect(state, queue, entity, player, effect, None, None)
        }
    }
}

/// The outcome of counter-secret interception (roadmap G8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interception {
    /// No counter-secret matched — the spell resolves normally
    None,
    /// A counter-secret negated the spell (Counterspell) — no effect resolves
    Countered,
    /// Spellbender summoned its token; the spell's single-target effect should
    /// target it
    Spellbent(crate::core::entity::Entity),
}

/// Counter-secret interception (roadmap G8): `WhenEnemySpellCast` secrets fire
/// BEFORE the spell's effect resolves, at the play boundary — Counterspell
/// negates the spell, Spellbender summons its token and redirects the spell's
/// single-target effect to it. This retires the post-hoc `redirect_damages`
/// queue mutation (E2).
pub fn intercept_counter_secrets(
    state: &mut GameState,
    queue: &mut EventQueue,
    spell: crate::core::entity::Entity,
    caster: PlayerId,
) -> Interception {
    let owner = caster.opponent();
    let event = Event::CardPlayed {
        player: caster,
        card: spell,
        target: None,
        position: None,
    };
    let secrets: SmallList<Entity> = state
        .world()
        .zones()
        .iter(Zone::SetAside, owner)
        .filter(|&e| {
            state.world().player(e) == Some(owner)
                && state
                    .world()
                    .secret(e)
                    .is_some_and(|sec| sec.trigger == SecretTrigger::WhenEnemySpellCast)
        })
        .collect();
    // Only the first matching secret fires (HS: one counter-secret per event)
    if let Some(entity) = secrets.into_iter().next() {
        // Reveal the secret and resolve its interception effect
        reveal_secret(state, queue, owner, entity);
        let effect = state.world().battlecry(entity).map(|b| b.0);
        if matches!(effect, Some(CardEffect::SummonSpellbender)) {
            let token = resolve_spellbender(state, queue, &event, owner);
            return match token {
                Some(token) => Interception::Spellbent(token),
                None => Interception::Countered,
            };
        }
        if let Some(effect) = effect {
            resolve_secret_effect(state, queue, &event, entity, owner, effect);
        }
        return Interception::Countered;
    }
    Interception::None
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
    // Collect all characters of both players (heroes + minions, including
    // stealthed), excluding own hero — stack-buffered list, no allocation.
    let candidates: SmallList<Entity> = [owner, owner.opponent()]
        .iter()
        .flat_map(|&pid| {
            state.world().zones().iter(Zone::Play, pid).filter(|&e| {
                let ct = state.world().card_type(e);
                (ct == Some(CardType::Minion) || ct == Some(CardType::Hero)) && e != hero
            })
        })
        .collect();
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
) -> Option<Entity> {
    let Event::CardPlayed { card, .. } = event else {
        return None;
    };
    // The 1/3 token is summoned; the spell redirect is handled by the caller
    // (intercept_counter_secrets) — roadmap G8, the queue mutation is retired.
    crate::engine::trigger::resolve_summon(state, queue, *card, owner, "MAGE_019t")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::action::Action;
    use crate::core::player::PlayerId;
    use crate::core::zone::Zone;
    use crate::engine::game::GameEngine;
    use crate::sim::game::GameBuilder;

    /// Playing Ice Block wires the fatal-damage trigger and the
    /// prevent-lethal-and-immune effect onto the hero's secret.
    #[test]
    fn playing_ice_block_registers_fatal_damage_secret() {
        let mut builder = GameBuilder::new();
        builder.active_player(PlayerId::Player2);
        builder.set_mana(PlayerId::Player2, 10, 10);
        let card = {
            let world = builder.state_mut().world_mut();
            let e = crate::cards::spawn_card_from_def(
                world,
                PlayerId::Player2,
                &crate::cards::classic_mage::ICE_BLOCK,
            );
            world.set_zone(e, Zone::Hand);
            world.zones_mut().insert(Zone::Hand, PlayerId::Player2, e);
            e
        };
        let mut state = builder.build();
        let engine = GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card,
                    target: None,
                    position: None,
                },
            )
            .unwrap();

        let secret = state
            .world()
            .zones()
            .iter(Zone::SetAside, PlayerId::Player2)
            .find(|&e| state.world().secret(e).is_some())
            .expect("the played secret should sit in SetAside");
        assert_eq!(
            state.world().secret(secret),
            Some(Secret {
                trigger: SecretTrigger::WhenFriendlyHeroFatallyDamaged,
                effect: Some(CardEffect::PreventFatalDamageAndImmune),
            })
        );
    }
}
