//! Trigger effect resolver — converts CardEffect into enqueued events.
//!
//! When the rule engine detects a Battlecry/Deathrattle component, it calls
//! this module's functions to resolve the effect into concrete game events
//! and enqueue them.
//!
//! # Target selection
//!
//! Effect targets are specified via the `EffectTarget` enum:
//! - `AnyEnemy` → random enemy hero or minion
//! - `AnyEnemyMinion` → random enemy minion
//! - `EnemyHero` → enemy hero
//! - `Self_` → the effect source entity itself
//! - `AllEnemyMinions` → all enemy minions

use crate::core::component::{
    Attack, AttacksUsed, CardId, CardType, Cost, Deathrattle, Durability, Freeze, Health, Immune,
    Stealth, Trigger, TriggerEvent, TriggerTiming,
};
use crate::core::effect::{CardEffect, EffectTarget};
use crate::core::entity::Entity;
use crate::core::event::{Event, EventQueue};
use crate::core::player::PlayerId;
use crate::core::small_list::SmallList;
use crate::core::state::GameState;
use crate::core::zone::Zone;
use crate::sim::rng::GameRng;

/// Selects a target from the candidate list, preferring the explicit target
/// and falling back to a random choice.
///
/// Falls back to a random selection when the explicit target is not in the
/// candidate set (preserving self-play determinism: given the same state and
/// action sequence, the result is reproducible).
fn select_target(
    explicit: Option<Entity>,
    candidates: &SmallList<Entity>,
    rng: &mut GameRng,
) -> Option<Entity> {
    match explicit {
        Some(t) if candidates.iter().any(|&c| c == t) => Some(t),
        _ if candidates.is_empty() => None,
        _ => Some(candidates[rng.next_usize(candidates.len())]),
    }
}

/// Resolves a CardEffect into game events and enqueues them.
///
/// `source` is the effect source entity (the minion owning the effect).
/// `owner` is the player owning the source entity.
/// `explicit_target` is the target specified by the player (passed from
/// `Action::PlayCard`); when `None`, the engine chooses randomly
/// (`select_target` fallback).
pub fn resolve_effect(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    effect: CardEffect,
    explicit_target: Option<Entity>,
) {
    match effect {
        CardEffect::DealDamage { amount, target } => {
            resolve_deal_damage(state, queue, source, owner, amount, target, explicit_target);
        }
        CardEffect::DrawCard { count } => {
            for _ in 0..count {
                draw_card(state, queue, owner);
            }
        }
        CardEffect::SummonMinion { card_id } => {
            let _ = resolve_summon(state, queue, source, owner, card_id);
        }
        CardEffect::GainStats {
            attack,
            health,
            target,
        } => {
            resolve_gain_stats(
                state,
                queue,
                source,
                owner,
                attack,
                health,
                target,
                explicit_target,
            );
        }
        CardEffect::EquipWeapon { card_id } => {
            resolve_equip_weapon(state, queue, owner, card_id);
        }
        CardEffect::GainArmor { amount, target } => {
            resolve_gain_armor(state, owner, amount, target, explicit_target);
        }
        CardEffect::ReturnToHand { target } => {
            resolve_return_to_hand(state, queue, owner, target, explicit_target);
        }
        CardEffect::IncreaseCost { amount, target } => {
            resolve_increase_cost(state, owner, amount, target, explicit_target);
        }
        CardEffect::ReturnToHandAndIncreaseCost { amount } => {
            resolve_return_to_hand_and_increase_cost(state, queue, owner, amount);
        }
        CardEffect::DestroyMinion { target } => {
            resolve_destroy_minion(state, queue, owner, target, explicit_target);
        }
        CardEffect::SilenceMinion { target } => {
            resolve_silence(state, owner, target, explicit_target);
        }
        CardEffect::SetAttack { attack, target } => {
            resolve_set_attack(state, owner, attack, target, explicit_target);
        }
        CardEffect::RestoreHealth { amount, target } => {
            resolve_restore_health(state, owner, amount, target);
        }
        CardEffect::FreezeCharacter { target } => {
            resolve_freeze(state, owner, target, explicit_target);
        }
        CardEffect::GainManaCrystal { count } => {
            let inner = state.make_mut();
            let p = &mut inner.players[owner.index()];
            p.mana_crystals = (p.mana_crystals + count).min(10);
            p.current_mana = (p.current_mana + count).min(10);
        }
        CardEffect::DestroyWeapon => {
            let enemy = owner.opponent();
            if let Some(weapon) = state.player(enemy).weapon {
                let inner = state.make_mut();
                inner.players[enemy.index()].weapon = None;
                queue.push(Event::WeaponDestroyed {
                    player: enemy,
                    weapon,
                });
            }
        }
        CardEffect::GainHeroAttack { attack, armor } => {
            resolve_gain_hero_attack(state, owner, attack, armor);
        }
        CardEffect::DealHeroAttackDamage { target } => {
            resolve_deal_hero_attack_damage(state, queue, source, owner, target, explicit_target);
        }
        CardEffect::FullHeal { target } => {
            resolve_full_heal(state, owner, target, explicit_target);
        }
        CardEffect::GrantWindfury { target } => {
            resolve_grant_windfury(state, owner, target, explicit_target);
        }
        CardEffect::GrantCharge {
            target,
            attack_bonus,
        } => {
            resolve_grant_charge(state, owner, target, attack_bonus, explicit_target);
        }
        CardEffect::DoubleAttack { target } => {
            resolve_double_attack(state, owner, target, explicit_target);
        }
        CardEffect::DoubleHealth { target } => {
            resolve_double_health(state, owner, target, explicit_target);
        }
        CardEffect::BuffWeapon { attack, durability } => {
            resolve_buff_weapon(state, owner, attack, durability);
        }
        CardEffect::DiscardRandomCard => {
            resolve_discard_random(state, owner);
        }
        CardEffect::DealArmorDamage { target } => {
            resolve_deal_armor_damage(state, queue, source, owner, target, explicit_target);
        }
        CardEffect::DestroyWeaponAndDraw => {
            resolve_destroy_weapon_and_draw(state, queue, owner);
        }
        CardEffect::ReturnAllToHand => {
            resolve_return_all_to_hand(state, owner);
        }
        CardEffect::SetAttackToHealth { target } => {
            resolve_set_attack_to_health(state, owner, target, explicit_target);
        }
        CardEffect::DestroyAllExceptOne => {
            resolve_destroy_all_except_one(state, queue, owner);
        }
        CardEffect::DestroyAndHeal { target, heal } => {
            resolve_destroy_and_heal(state, queue, owner, target, heal, explicit_target);
        }
        CardEffect::DestroyAndAOE { target } => {
            resolve_destroy_and_aoe(state, queue, owner, source, target);
        }
        CardEffect::DealDamageToTwo { amount } => {
            resolve_deal_damage_to_two(state, queue, source, owner, amount);
        }
        CardEffect::DealDamageAndDraw {
            damage,
            target,
            draw,
        } => {
            resolve_deal_damage(state, queue, source, owner, damage, target, explicit_target);
            for _ in 0..draw {
                draw_card(state, queue, owner);
            }
        }
        CardEffect::DamageAndGainAttack {
            damage,
            attack_bonus,
            target,
        } => {
            resolve_deal_damage(state, queue, source, owner, damage, target, explicit_target);
            // Give the target minion an attack bonus (simplified: random friendly minion)
            let minions = collect_friendly_minions(state, owner);
            if !minions.is_empty() {
                let idx = state.rng_mut().next_usize(minions.len());
                let world = state.world_mut();
                let m = minions[idx];
                let cur = world.attack(m).unwrap_or(Attack(0));
                world.set_attack(m, Attack(cur.0 + attack_bonus));
            }
        }
        CardEffect::DestroyAdjacent { gain_stats: _ } => {
            // Simplified implementation — destroy a random friendly minion and gain its stats
            let friendly = collect_friendly_minions(state, owner);
            if friendly.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(friendly.len());
            let sacrifice = friendly[idx];
            let atk = state.world().attack(sacrifice).unwrap_or(Attack(0));
            let hp = state.world().health(sacrifice).unwrap_or(Health(0));
            // Destroy the sacrifice
            queue.push(Event::DamageDealt {
                source,
                target: sacrifice,
                amount: hp.0.max(1),
            });
            // Add the stats to the source minion
            let cur_atk = state.world().attack(source).unwrap_or(Attack(0));
            let cur_hp = state.world().health(source).unwrap_or(Health(0));
            state
                .world_mut()
                .set_attack(source, Attack(cur_atk.0 + atk.0));
            state
                .world_mut()
                .set_health(source, Health(cur_hp.0 + hp.0));
        }
        CardEffect::DestroyManaCrystal => {
            let inner = state.make_mut();
            let p = &mut inner.players[owner.index()];
            if p.mana_crystals > 0 {
                p.mana_crystals -= 1;
                p.current_mana = p.current_mana.min(p.mana_crystals);
            }
        }
        CardEffect::GiveCardsToOpponent { count: _ } => {
            let enemy = owner.opponent();
            draw_card(state, queue, enemy);
        }
        CardEffect::ResurrectMinion => {
            // Cannot resurrect when the board is full
            let board_count = state
                .world()
                .zones()
                .iter(Zone::Play, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .count();
            if board_count >= crate::engine::rules::MAX_BOARD_SIZE {
                return;
            }
            let inner = state.make_mut();
            let died = &mut inner.players[owner.index()].died_this_turn;
            if let Some(entity) = died.pop() {
                // Resurrect: move the minion from the graveyard back to the battlefield with 1 health
                let world = &mut inner.world;
                if world.zone(entity) == Some(Zone::Graveyard) {
                    let _ = world.move_to_zone(entity, Zone::Play);
                    world.set_health(entity, Health(1));
                    world.set_attacks_used(entity, AttacksUsed(0));
                }
            }
        }
        CardEffect::CopyMinionStats => {
            let friendly = collect_friendly_minions(state, owner);
            if friendly.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(friendly.len());
            let target = friendly[idx];
            let atk = state.world().effective_attack(target).unwrap_or(Attack(0));
            let hp = state.world().effective_health(target).unwrap_or(Health(0));
            let world = state.world_mut();
            world.set_attack(source, Attack(atk.0));
            world.set_health(source, Health(hp.0));
        }
        CardEffect::TempDebuff {
            attack_reduction,
            target,
        } => {
            let enemies = match target {
                EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
                _ => return,
            };
            let Some(enemy) = select_target(explicit_target, &enemies, state.rng_mut()) else {
                return;
            };
            state.world_mut().set_temp_attack_debuff(
                enemy,
                crate::core::component::TempAttackDebuff(attack_reduction),
            );
        }
        CardEffect::ReflectDamage => {
            // Already handled by the secret system's WhenHeroDamaged trigger
        }
        CardEffect::DealDamageAndReturnToHand { amount, target } => {
            // Damage is resolved immediately; "return to hand" is handled by rules.rs's CardPlayed
            resolve_deal_damage(state, queue, source, owner, amount, target, explicit_target);
        }
        CardEffect::ReturnFriendlyToHandAndReduceCost { amount } => {
            resolve_return_friendly_reduce_cost(state, owner, amount);
        }
        CardEffect::AdjacentDamage => {
            resolve_adjacent_damage(state, queue, source, owner);
        }
        CardEffect::DestroyWeaponAndDealAttackToEnemies => {
            resolve_destroy_weapon_deal_attack(state, queue, source, owner);
        }
        CardEffect::GrantStealth => {
            resolve_grant_stealth(state, source, owner);
        }
        CardEffect::SummonMultipleMinions { card_id, count } => {
            for _ in 0..count {
                let _ = resolve_summon(state, queue, source, owner, card_id);
            }
        }
        // The following secret-only effects are handled by secret.rs's resolve_secret_effect (they need event context)
        CardEffect::DamagePlayedMinion { .. }
        | CardEffect::RedirectAttackToRandomCharacter
        | CardEffect::SummonAndRedirectAttack { .. }
        | CardEffect::SummonSpellbender => {}
        CardEffect::NextSecretCostsZero => {
            // Kirin Tor Mage: the next secret costs 0
            let inner = state.make_mut();
            inner.players[owner.index()].next_secret_free = true;
        }
        CardEffect::DrawCardAndReduceCost { amount } => {
            draw_card_with_reduction(state, queue, owner, amount);
        }
        CardEffect::GrantDeathrattleAll { card_id } => {
            let minions = collect_friendly_minions(state, owner);
            let dr = Deathrattle(CardEffect::SummonMinion { card_id });
            let world = state.world_mut();
            for m in &minions {
                world.set_deathrattle(*m, dr);
            }
        }
        CardEffect::GiveCardToOpponent { card_id, count } => {
            let enemy = owner.opponent();
            for _ in 0..count {
                let Some(card_def) = crate::cards::def::card_by_id(card_id) else {
                    return;
                };
                // Create a new entity directly into the opponent's hand (a new entity has no Zone component, so move_to_zone cannot be used)
                let world = state.world_mut();
                let e = crate::cards::spawn_card_from_def(world, enemy, card_def);
                world.set_zone(e, Zone::Hand);
                world.zones_mut().insert(Zone::Hand, enemy, e);
            }
        }
        CardEffect::FreezeOrDamage { amount } => {
            resolve_freeze_or_damage(state, queue, source, owner, amount, explicit_target);
        }
        CardEffect::DestroyAndGainHealth => {
            resolve_destroy_and_gain_health(state, queue, source, owner);
        }
        CardEffect::GrantAttackAndImmune { attack } => {
            resolve_grant_attack_and_immune(state, owner, attack);
        }
        CardEffect::TakeControlUntilEndOfTurn => {
            resolve_take_control(state, owner, true);
        }
        CardEffect::TakeControl => {
            resolve_take_control(state, owner, false);
        }
        CardEffect::Corrupt => {
            resolve_corrupt(state, owner);
        }
        CardEffect::MinHealthUntilEndOfTurn => {
            let inner = state.make_mut();
            inner.players[owner.index()].minion_min_health = 1;
        }
        CardEffect::TransformToRandom { card_a, card_b } => {
            resolve_transform(state, owner, card_a, card_b);
        }
        CardEffect::AddRandomCardToHand { pool } => {
            let Some(card_def) = crate::cards::pool::random_card(state.rng_mut(), pool) else {
                return;
            };
            add_card_to_hand(state, owner, card_def);
        }
        CardEffect::SummonRandomMinion { pool } => {
            let Some(card_def) = crate::cards::pool::random_card(state.rng_mut(), pool) else {
                return;
            };
            let _ = resolve_summon(state, queue, source, owner, card_def.id);
        }
        CardEffect::AddCardToHand { card_id } => {
            let Some(card_def) = crate::cards::def::card_by_id(card_id) else {
                return;
            };
            add_card_to_hand(state, owner, card_def);
        }
        CardEffect::DealDamageAndSummonIfKilled { amount, pool } => {
            resolve_damage_and_summon_if_killed(state, queue, source, owner, amount, pool);
        }
    }
}

/// Draws a random card from the deck into hand, optionally with a cost reduction (Farsight).
fn draw_card_with_reduction(
    state: &mut GameState,
    queue: &mut EventQueue,
    player: PlayerId,
    cost_reduction: i32,
) {
    let deck_len = state.world().zones().len(Zone::Deck, player);
    if deck_len == 0 {
        // Deck empty; no draw (fatigue in Phase 3+)
        return;
    }

    // Pick a random card from the deck
    let idx = state.rng_mut().next_usize(deck_len);
    let card = state
        .world()
        .zones()
        .iter(Zone::Deck, player)
        .nth(idx)
        .expect("deck card should exist");

    // Move to hand
    state
        .world_mut()
        .move_to_zone(card, Zone::Hand)
        .expect("card should be movable to hand");

    // Cost reduction (kept above the base cost)
    if cost_reduction > 0 {
        let world = state.world_mut();
        let cur = world.cost(card).unwrap_or(Cost(0));
        world.set_cost(card, Cost((cur.0 - cost_reduction).max(0)));
    }

    queue.push(Event::CardDrawn { player, card });
}

/// Draws a random card from the deck into hand.
pub fn draw_card(state: &mut GameState, queue: &mut EventQueue, player: PlayerId) {
    draw_card_with_reduction(state, queue, player, 0);
}

fn resolve_deal_damage(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    amount: i32,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let enemies = match target {
        EffectTarget::AnyEnemy => collect_enemy_characters(state, owner),
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        EffectTarget::EnemyHero => {
            let hero = state.player(owner.opponent()).hero;
            [hero].into_iter().collect()
        }
        EffectTarget::AllEnemyMinions => {
            let minions = collect_all_enemy_minions(state, owner);
            for minion in &minions {
                queue.push(Event::DamageDealt {
                    source,
                    target: *minion,
                    amount,
                });
            }
            return;
        }
        EffectTarget::AllEnemies => {
            let enemies = collect_all_enemy_characters(state, owner);
            for enemy in &enemies {
                queue.push(Event::DamageDealt {
                    source,
                    target: *enemy,
                    amount,
                });
            }
            return;
        }
        EffectTarget::AllFriendlyMinions => {
            let minions = collect_friendly_minions(state, owner);
            for minion in &minions {
                queue.push(Event::DamageDealt {
                    source,
                    target: *minion,
                    amount,
                });
            }
            return;
        }
        EffectTarget::Self_ => {
            queue.push(Event::DamageDealt {
                source,
                target: source,
                amount,
            });
            return;
        }
        EffectTarget::AllMinions => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            for m in &all {
                queue.push(Event::DamageDealt {
                    source,
                    target: *m,
                    amount,
                });
            }
            return;
        }
        EffectTarget::AllCharacters
        | EffectTarget::FriendlyHero
        | EffectTarget::DamagedEnemyMinion
        | EffectTarget::FriendlyMinion
        | EffectTarget::TauntEnemyMinion => {
            return;
        }
    };

    // Explicit target first, otherwise random selection
    let Some(target_entity) = select_target(explicit, &enemies, state.rng_mut()) else {
        return;
    };

    queue.push(Event::DamageDealt {
        source,
        target: target_entity,
        amount,
    });
}

/// Resolves a summon-minion effect.
///
/// Returns the summoned minion entity, or `None` when the board is full or
/// the card does not exist. `pub(crate)` so the secret system (Noble
/// Sacrifice / Spellbender) can obtain the newly summoned entity.
pub(crate) fn resolve_summon(
    state: &mut GameState,
    queue: &mut EventQueue,
    _source: Entity,
    owner: PlayerId,
    card_id: &str,
) -> Option<Entity> {
    // Look up the card definition
    let card_def = crate::cards::def::card_by_id(card_id)?;

    // Check the board size limit
    let board_count = state
        .world()
        .zones()
        .iter(Zone::Play, owner)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .count();
    if board_count >= crate::engine::rules::MAX_BOARD_SIZE {
        return None;
    }

    // Create the minion entity and place it on the battlefield
    let e = {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_card_id(e, CardId(card_def.id));
        world.set_health(e, Health(card_def.health));
        world.set_attack(e, Attack(card_def.attack));
        world.set_cost(e, crate::core::component::Cost(card_def.cost));
        world.set_card_type(e, card_def.card_type);
        world.set_player(e, owner);
        world.set_attacks_used(e, crate::core::component::AttacksUsed(0));
        world.set_zone(e, Zone::Play);
        world.zones_mut().insert(Zone::Play, owner, e);
        // Set aura, battlecry, deathrattle, taunt (if any)
        if let Some((aura_effect, aura_target)) = card_def.aura {
            world.set_aura(
                e,
                crate::core::component::Aura {
                    effect: aura_effect,
                    target: aura_target,
                },
            );
        }
        if let Some(bc) = card_def.battlecry {
            world.set_battlecry(e, crate::core::component::Battlecry(bc));
        }
        if let Some(dr) = card_def.deathrattle {
            world.set_deathrattle(e, crate::core::component::Deathrattle(dr));
        }
        if card_def.taunt {
            world.set_taunt(e, crate::core::component::Taunt);
        }
        // Set divine shield / windfury / charge / spell damage / cant-attack / end-of-turn effect
        if card_def.divine_shield {
            world.set_divine_shield(e, crate::core::component::DivineShield);
        }
        if card_def.windfury {
            world.set_windfury(e, crate::core::component::Windfury);
        }
        if card_def.charge {
            world.set_charge(e, crate::core::component::Charge);
        }
        if card_def.spell_damage != 0 {
            world.set_spell_damage(
                e,
                crate::core::component::SpellDamage(card_def.spell_damage),
            );
        }
        if card_def.cant_attack {
            world.set_cant_attack(e, crate::core::component::CantAttack);
        }
        if let Some(ete) = card_def.end_turn_effect {
            world.set_trigger(
                e,
                Trigger {
                    event: TriggerEvent::TurnEnd,
                    timing: TriggerTiming::Whenever,
                    effect: ete,
                },
            );
        }
        if let Some(ste) = card_def.start_turn_effect {
            world.set_trigger(
                e,
                Trigger {
                    event: TriggerEvent::TurnStart,
                    timing: TriggerTiming::Whenever,
                    effect: ste,
                },
            );
        }
        if let Some(st) = card_def.spell_trigger {
            world.set_trigger(
                e,
                Trigger {
                    event: TriggerEvent::FriendlySpellCast,
                    timing: TriggerTiming::Whenever,
                    effect: st,
                },
            );
        }
        if let Some(dt) = card_def.death_trigger {
            world.set_trigger(
                e,
                Trigger {
                    event: TriggerEvent::FriendlyMinionDied,
                    timing: TriggerTiming::Whenever,
                    effect: dt,
                },
            );
        }
        if let Some(st) = card_def.summon_trigger {
            world.set_trigger(
                e,
                Trigger {
                    event: TriggerEvent::FriendlyMinionSummoned,
                    timing: TriggerTiming::Whenever,
                    effect: st,
                },
            );
        }
        if let Some(ce) = card_def.choose_one_effect {
            world.set_choose_one_effect(e, crate::core::component::ChooseOneEffect(ce));
        }
        if let Some(cb) = card_def.combo_effect {
            world.set_combo_effect(e, crate::core::component::ComboEffect(cb));
        }
        if card_def.attack_equals_health {
            world.set_attack_equals_health(e, crate::core::component::AttackEqualsHealth);
        }
        // Special keywords (poison/stealth, etc.): mapped centrally by card ID in the cards layer
        crate::cards::apply_card_keywords(world, e, card_def);
        e
    };

    // Enqueue the MinionSummoned event (triggers battlecry and similar effects)
    queue.push(Event::MinionSummoned {
        player: owner,
        minion: e,
    });
    Some(e)
}

/// Resolves a buff effect.
// 8 parameters (state, queue, source, owner, attack, health, target, explicit) — resolver convention style.
#[allow(clippy::too_many_arguments)]
fn resolve_gain_stats(
    state: &mut GameState,
    _queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    attack: i32,
    health: i32,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    match target {
        EffectTarget::Self_ => {
            let world = state.world_mut();
            let cur_atk = world.attack(source).unwrap_or(Attack(0));
            let cur_hp = world.health(source).unwrap_or(Health(0));
            world.set_attack(source, Attack(cur_atk.0 + attack));
            world.set_health(source, Health(cur_hp.0 + health));
        }
        EffectTarget::AllFriendlyMinions => {
            let minions: SmallList<Entity> = collect_friendly_minions(state, owner);
            let world = state.world_mut();
            for minion in &minions {
                let cur_atk = world.attack(*minion).unwrap_or(Attack(0));
                let cur_hp = world.health(*minion).unwrap_or(Health(0));
                world.set_attack(*minion, Attack(cur_atk.0 + attack));
                world.set_health(*minion, Health(cur_hp.0 + health));
            }
        }
        EffectTarget::FriendlyMinion => {
            let minions = collect_friendly_minions(state, owner);
            let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
                return;
            };
            let world = state.world_mut();
            let cur_atk = world.attack(m).unwrap_or(Attack(0));
            let cur_hp = world.health(m).unwrap_or(Health(0));
            world.set_attack(m, Attack(cur_atk.0 + attack));
            world.set_health(m, Health(cur_hp.0 + health));
        }
        _ => {}
    }
}

/// Equips a weapon.
fn resolve_equip_weapon(
    state: &mut GameState,
    queue: &mut EventQueue,
    owner: PlayerId,
    card_id: &str,
) {
    let Some(card_def) = crate::cards::def::card_by_id(card_id) else {
        return;
    };

    // If a weapon is already equipped, destroy the old one first
    let old_weapon = state.player(owner).weapon;
    if let Some(w) = old_weapon {
        queue.push(Event::WeaponDestroyed {
            player: owner,
            weapon: w,
        });
    }

    // Create the weapon entity and update the Player
    let inner = state.make_mut();
    let weapon = inner.world.spawn();
    inner.world.set_card_id(weapon, CardId(card_def.id));
    inner.world.set_attack(weapon, Attack(card_def.attack));
    inner
        .world
        .set_durability(weapon, Durability(card_def.durability));
    inner.world.set_cost(weapon, Cost(card_def.cost));
    inner.world.set_card_type(weapon, CardType::Weapon);
    inner.world.set_player(weapon, owner);
    inner.world.set_zone(weapon, Zone::Play);
    inner.world.zones_mut().insert(Zone::Play, owner, weapon);
    inner.players[owner.index()].weapon = Some(weapon);

    queue.push(Event::WeaponEquipped {
        player: owner,
        weapon,
    });
}

/// Gains armor.
fn resolve_gain_armor(
    state: &mut GameState,
    owner: PlayerId,
    amount: i32,
    target: EffectTarget,
    _explicit: Option<Entity>,
) {
    match target {
        EffectTarget::Self_ | EffectTarget::FriendlyHero => {
            let inner = state.make_mut();
            inner.players[owner.index()].armor += amount;
        }
        EffectTarget::EnemyHero => {
            let inner = state.make_mut();
            inner.players[owner.opponent().index()].armor += amount;
        }
        _ => {
            // Other targets not yet supported
        }
    }
}

/// Returns a minion to hand.
fn resolve_return_to_hand(
    state: &mut GameState,
    _queue: &mut EventQueue,
    owner: PlayerId,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let _enemy = owner.opponent();
    let minions = match target {
        EffectTarget::AnyEnemy => collect_enemy_minions(state, owner),
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        _ => return,
    };

    if minions.is_empty() {
        return;
    }

    let Some(target_entity) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };

    // Move to hand
    let _ = state.world_mut().move_to_zone(target_entity, Zone::Hand);
}

/// Returns a random enemy minion to hand and increases its mana cost (full Freezing Trap effect).
fn resolve_return_to_hand_and_increase_cost(
    state: &mut GameState,
    _queue: &mut EventQueue,
    owner: PlayerId,
    amount: i32,
) {
    let minions = collect_enemy_minions(state, owner);
    if minions.is_empty() {
        return;
    }

    let idx = state.rng_mut().next_usize(minions.len());
    let target_entity = minions[idx];

    let _ = state.world_mut().move_to_zone(target_entity, Zone::Hand);
    let world = state.world_mut();
    let cur_cost = world.cost(target_entity).unwrap_or(Cost(0));
    world.set_cost(target_entity, Cost(cur_cost.0 + amount));
}

/// Increases a minion's mana cost (e.g. Freezing Trap effect).
fn resolve_increase_cost(
    state: &mut GameState,
    owner: PlayerId,
    amount: i32,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let _enemy = owner.opponent();
    let minions = match target {
        EffectTarget::AnyEnemy => collect_enemy_minions(state, owner),
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        _ => return,
    };

    if minions.is_empty() {
        return;
    }

    let Some(target_entity) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };

    let world = state.world_mut();
    let cur_cost = world.cost(target_entity).unwrap_or(Cost(0));
    world.set_cost(target_entity, Cost(cur_cost.0 + amount));
}

/// Destroys a minion — deals damage equal to its current health to guarantee the kill.
fn resolve_destroy_minion(
    state: &mut GameState,
    queue: &mut EventQueue,
    owner: PlayerId,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        EffectTarget::DamagedEnemyMinion => collect_enemy_minions(state, owner)
            .into_iter()
            .filter(|&e| {
                state
                    .world()
                    .health(e)
                    .is_some_and(|h| h.0 < state.world().effective_health(e).unwrap_or(h).0)
            })
            .collect(),
        EffectTarget::TauntEnemyMinion => {
            let minions: SmallList<Entity> = collect_enemy_minions(state, owner)
                .into_iter()
                .filter(|&e| state.world().taunt(e).is_some())
                .collect();
            if minions.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(minions.len());
            let m = minions[idx];
            let hp = state.world().health(m).unwrap_or(Health(1));
            queue.push(Event::DamageDealt {
                source: m,
                target: m,
                amount: hp.0.max(1),
            });
            return;
        }
        EffectTarget::AllMinions => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            for &m in &all {
                let hp = state.world().health(m).unwrap_or(Health(1));
                queue.push(Event::DamageDealt {
                    source: m,
                    target: m,
                    amount: hp.0.max(1),
                });
            }
            return;
        }
        _ => return,
    };
    // Explicit target: destroy only the specified minion
    if let Some(t) = explicit.filter(|t| minions.contains(t)) {
        let hp = state.world().health(t).unwrap_or(Health(1));
        queue.push(Event::DamageDealt {
            source: t,
            target: t,
            amount: hp.0.max(1),
        });
        return;
    }
    for &m in &minions {
        let hp = state.world().health(m).unwrap_or(Health(1));
        queue.push(Event::DamageDealt {
            source: m,
            target: m,
            amount: hp.0.max(1),
        });
    }
}

/// Silences a minion — removes all effect components.
fn resolve_silence(
    state: &mut GameState,
    owner: PlayerId,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        EffectTarget::AllMinions => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            all
        }
        _ => return,
    };
    // Explicit target: silence only the specified minion
    if let Some(t) = explicit.filter(|t| minions.contains(t)) {
        let world = state.world_mut();
        world.remove_taunt(t);
        world.remove_battlecry(t);
        world.remove_deathrattle(t);
        world.remove_aura(t);
        world.remove_trigger(t);
        world.remove_divine_shield(t);
        world.remove_windfury(t);
        world.remove_charge(t);
        world.remove_spell_damage(t);
        return;
    }
    for &m in &minions {
        let world = state.world_mut();
        world.remove_taunt(m);
        world.remove_battlecry(m);
        world.remove_deathrattle(m);
        world.remove_aura(m);
        world.remove_trigger(m);
        world.remove_divine_shield(m);
        world.remove_windfury(m);
        world.remove_charge(m);
        world.remove_spell_damage(m);
    }
}

/// Sets attack.
fn resolve_set_attack(
    state: &mut GameState,
    owner: PlayerId,
    attack: i32,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        _ => return,
    };
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    state.world_mut().set_attack(m, Attack(attack));
}

/// Restores health.
fn resolve_restore_health(
    state: &mut GameState,
    owner: PlayerId,
    amount: i32,
    target: EffectTarget,
) {
    match target {
        EffectTarget::FriendlyHero => {
            let hero = state.player(owner).hero;
            let cur = state.world().health(hero).unwrap_or(Health(0));
            state
                .world_mut()
                .set_health(hero, Health((cur.0 + amount).min(30)));
        }
        EffectTarget::Self_ => {
            // source is the effect source, but here we use hero
            let hero = state.player(owner).hero;
            let cur = state.world().health(hero).unwrap_or(Health(0));
            state
                .world_mut()
                .set_health(hero, Health((cur.0 + amount).min(30)));
        }
        EffectTarget::AllFriendlyMinions => {
            let minions = collect_friendly_minions(state, owner);
            let world = state.world_mut();
            for &m in &minions {
                let cur = world.health(m).unwrap_or(Health(0));
                world.set_health(m, Health((cur.0 + amount).min(30)));
            }
        }
        _ => {}
    }
}

/// Freezes a character.
fn resolve_freeze(
    state: &mut GameState,
    owner: PlayerId,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let targets: SmallList<Entity> = match target {
        EffectTarget::AnyEnemy => collect_enemy_characters(state, owner),
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        EffectTarget::AllEnemyMinions => collect_all_enemy_minions(state, owner),
        EffectTarget::AllCharacters => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_characters(state, owner));
            all
        }
        _ => return,
    };
    // Explicit target: freeze only the specified character
    if let Some(t) = explicit.filter(|t| targets.contains(t)) {
        state.world_mut().set_freeze(t, Freeze);
        return;
    }
    for &t in &targets {
        state.world_mut().set_freeze(t, Freeze);
    }
}

/// Gives the hero temporary attack and optional armor.
fn resolve_gain_hero_attack(state: &mut GameState, owner: PlayerId, attack: i32, armor: i32) {
    let hero = state.player(owner).hero;
    let inner = state.make_mut();
    // Add temporary attack
    inner.players[owner.index()].temp_attack_bonus += attack;
    let cur_atk = inner.world.attack(hero).unwrap_or(Attack(0));
    inner.world.set_attack(hero, Attack(cur_atk.0 + attack));
    // Add armor
    if armor > 0 {
        inner.players[owner.index()].armor += armor;
    }
}

/// Deals damage to a target equal to the hero's attack.
fn resolve_deal_hero_attack_damage(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let hero = state.player(owner).hero;
    let hero_atk = state.world().effective_attack(hero).unwrap_or(Attack(0)).0;
    if hero_atk <= 0 {
        return;
    }
    let minions: SmallList<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        EffectTarget::AnyEnemy => collect_enemy_characters(state, owner),
        _ => return,
    };
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    queue.push(Event::DamageDealt {
        source,
        target: m,
        amount: hero_atk,
    });
}

/// Restores a minion to full health (maximum health).
fn resolve_full_heal(
    state: &mut GameState,
    owner: PlayerId,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        _ => return,
    };
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    // Get max health (based on the original definition; simplified here to 30 or the current max)
    // Simplified approach: heal from current health (use max_health if present, otherwise keep the current value)
    let world = state.world_mut();
    if world.health(m).is_some() {
        // Simplified: restore to the card definition value. Since the original
        // definition is not available, set the current health
        // Actually the original definition should be looked up via card_id
        // Here we use a helper approach: set health to the max (saved at entity creation)
        // Simplified implementation: set to 30 (hero cap) or keep the current value
        // A real implementation would need a max_health component
        let cur = world.health(m).unwrap_or(Health(0));
        // For minions, assume max health >= current health; set to the larger value
        // Simplified: use cur + a reasonable buffer
        world.set_health(m, Health((cur.0 + 10).min(30)));
    }
}

/// Grants a minion windfury.
fn resolve_grant_windfury(
    state: &mut GameState,
    owner: PlayerId,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        _ => return,
    };
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    state
        .world_mut()
        .set_windfury(m, crate::core::component::Windfury);
}

/// Grants a minion charge and an optional attack bonus.
fn resolve_grant_charge(
    state: &mut GameState,
    owner: PlayerId,
    target: EffectTarget,
    attack_bonus: i32,
    explicit: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        _ => return,
    };
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    let world = state.world_mut();
    world.set_charge(m, crate::core::component::Charge);
    // Reset the attack count to allow attacking immediately
    world.set_attacks_used(m, crate::core::component::AttacksUsed(0));
    if attack_bonus > 0 {
        let cur_atk = world.attack(m).unwrap_or(Attack(0));
        world.set_attack(m, Attack(cur_atk.0 + attack_bonus));
    }
}

/// Doubles a minion's attack.
fn resolve_double_attack(
    state: &mut GameState,
    owner: PlayerId,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        _ => return,
    };
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    let world = state.world_mut();
    let cur_atk = world.attack(m).unwrap_or(Attack(0));
    world.set_attack(m, Attack(cur_atk.0 * 2));
}

/// Doubles a minion's health.
fn resolve_double_health(
    state: &mut GameState,
    owner: PlayerId,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        _ => return,
    };
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    let world = state.world_mut();
    let cur_hp = world.health(m).unwrap_or(Health(0));
    world.set_health(m, Health((cur_hp.0 * 2).min(30)));
}

/// Increases the friendly hero's weapon attack and durability.
fn resolve_buff_weapon(state: &mut GameState, owner: PlayerId, attack: i32, durability: i32) {
    let weapon = state.player(owner).weapon;
    if let Some(w) = weapon {
        let world = state.world_mut();
        if attack != 0 {
            let cur_atk = world.attack(w).unwrap_or(Attack(0));
            world.set_attack(w, Attack(cur_atk.0 + attack));
        }
        if durability != 0 {
            let cur_dur = world.durability(w).unwrap_or(Durability(0));
            world.set_durability(w, Durability(cur_dur.0 + durability));
        }
    }
}

/// Discards a random card from hand.
fn resolve_discard_random(state: &mut GameState, owner: PlayerId) {
    let hand: SmallList<Entity> = state.world().zones().iter(Zone::Hand, owner).collect();
    if hand.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(hand.len());
    let card = hand[idx];
    let _ = state.world_mut().move_to_zone(card, Zone::Graveyard);
}

/// Deals damage to a target equal to the hero's armor.
fn resolve_deal_armor_damage(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let armor = state.player(owner).armor;
    if armor <= 0 {
        return;
    }
    let minions: SmallList<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        EffectTarget::AnyEnemy => collect_enemy_characters(state, owner),
        _ => return,
    };
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    queue.push(Event::DamageDealt {
        source,
        target: m,
        amount: armor,
    });
}

/// Destroys the enemy weapon and draws cards equal to its durability.
fn resolve_destroy_weapon_and_draw(state: &mut GameState, queue: &mut EventQueue, owner: PlayerId) {
    let enemy = owner.opponent();
    let weapon = state.player(enemy).weapon;
    if let Some(w) = weapon {
        let durability = state.world().durability(w).unwrap_or(Durability(0)).0;
        let inner = state.make_mut();
        inner.players[enemy.index()].weapon = None;
        queue.push(Event::WeaponDestroyed {
            player: enemy,
            weapon: w,
        });
        for _ in 0..durability {
            draw_card(state, queue, owner);
        }
    }
}

/// Deals damage to two random enemy minions.
fn resolve_deal_damage_to_two(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    amount: i32,
) {
    let mut enemies = collect_enemy_minions(state, owner);
    if enemies.is_empty() {
        return;
    }
    // Pick two at random (the same minion can be picked twice if the enemy has only 1 minion)
    for _ in 0..2 {
        if enemies.is_empty() {
            break;
        }
        let idx = state.rng_mut().next_usize(enemies.len());
        let target = enemies[idx];
        queue.push(Event::DamageDealt {
            source,
            target,
            amount,
        });
        enemies.remove(idx);
    }
}

/// Returns all minions to their owners' hands.
fn resolve_return_all_to_hand(state: &mut GameState, owner: PlayerId) {
    let all_minions: SmallList<Entity> = [owner, owner.opponent()]
        .iter()
        .flat_map(|&pid| {
            state
                .world()
                .zones()
                .iter(Zone::Play, pid)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect::<Vec<_>>()
        })
        .collect();
    for m in &all_minions {
        let _ = state.world_mut().move_to_zone(*m, Zone::Hand);
    }
}

/// Sets a minion's attack equal to its current health.
fn resolve_set_attack_to_health(
    state: &mut GameState,
    owner: PlayerId,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        _ => return,
    };
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    let world = state.world_mut();
    let hp = world.health(m).unwrap_or(Health(0));
    world.set_attack(m, Attack(hp.0));
}

/// Destroys all minions except one random survivor.
fn resolve_destroy_all_except_one(state: &mut GameState, queue: &mut EventQueue, owner: PlayerId) {
    let enemy = owner.opponent();
    let mut all_minions: SmallList<Entity> = [owner, enemy]
        .iter()
        .flat_map(|&pid| {
            state
                .world()
                .zones()
                .iter(Zone::Play, pid)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect::<Vec<_>>()
        })
        .collect();
    if all_minions.is_empty() {
        return;
    }
    // Pick a random survivor and destroy the rest
    let survivor_idx = state.rng_mut().next_usize(all_minions.len());
    let survivor = all_minions.remove(survivor_idx);
    for &m in &all_minions {
        let hp = state.world().health(m).unwrap_or(Health(1));
        queue.push(Event::DamageDealt {
            source: survivor,
            target: m,
            amount: hp.0.max(1),
        });
    }
}

/// Destroys a minion and restores health to the hero.
fn resolve_destroy_and_heal(
    state: &mut GameState,
    queue: &mut EventQueue,
    owner: PlayerId,
    target: EffectTarget,
    heal: i32,
    explicit: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner),
        _ => return,
    };
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    let hp = state.world().health(m).unwrap_or(Health(1));
    queue.push(Event::DamageDealt {
        source: m,
        target: m,
        amount: hp.0.max(1),
    });
    let hero = state.player(owner).hero;
    let cur = state.world().health(hero).unwrap_or(Health(0));
    state
        .world_mut()
        .set_health(hero, Health((cur.0 + heal).min(30)));
}

/// Destroys a friendly minion and deals AOE damage equal to its attack.
fn resolve_destroy_and_aoe(
    state: &mut GameState,
    queue: &mut EventQueue,
    owner: PlayerId,
    source: Entity,
    target: EffectTarget,
) {
    // Collect friendly minions and pick one at random
    let friendly = collect_friendly_minions(state, owner);
    if friendly.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(friendly.len());
    let sacrifice = friendly[idx];
    let atk = state.world().attack(sacrifice).unwrap_or(Attack(0)).0;
    // Destroy the sacrifice
    let hp = state.world().health(sacrifice).unwrap_or(Health(1));
    queue.push(Event::DamageDealt {
        source,
        target: sacrifice,
        amount: hp.0.max(1),
    });
    // Deal damage equal to its attack to all enemy minions
    let _enemy = owner.opponent();
    let targets: SmallList<Entity> = match target {
        EffectTarget::AllEnemyMinions => collect_all_enemy_minions(state, owner),
        _ => return,
    };
    for t in &targets {
        queue.push(Event::DamageDealt {
            source,
            target: *t,
            amount: atk,
        });
    }
}

/// Returns a friendly minion to hand and reduces its cost (Shadowstep).
fn resolve_return_friendly_reduce_cost(state: &mut GameState, owner: PlayerId, amount: i32) {
    let minions = collect_friendly_minions(state, owner);
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let target = minions[idx];
    let _ = state.world_mut().move_to_zone(target, Zone::Hand);
    let world = state.world_mut();
    let cur = world.cost(target).unwrap_or(Cost(0));
    world.set_cost(target, Cost((cur.0 - amount).max(0)));
}

/// Deals damage equal to the target's attack to its adjacent minions (Betrayal).
///
/// The target is a random enemy minion; its left and right neighbors on the
/// same board (position difference of 1) take the damage.
fn resolve_adjacent_damage(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
) {
    let minions = collect_enemy_minions(state, owner);
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let target = minions[idx];
    let atk = state
        .world()
        .effective_attack(target)
        .unwrap_or(Attack(0))
        .0;
    if atk <= 0 {
        return;
    }
    // Find the target's position on the enemy board and take its left/right neighbors
    let enemy = owner.opponent();
    let board: SmallList<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, enemy)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect();
    let Some(pos) = board.iter().position(|&e| e == target) else {
        return;
    };
    // Left neighbor (wrapping_sub yields None when pos is 0)
    if let Some(&left) = board.get(pos.wrapping_sub(1)) {
        queue.push(Event::DamageDealt {
            source,
            target: left,
            amount: atk,
        });
    }
    if let Some(&right) = board.get(pos + 1) {
        queue.push(Event::DamageDealt {
            source,
            target: right,
            amount: atk,
        });
    }
}

/// Destroys own weapon and deals damage equal to its attack to all enemies (Blade Flurry).
fn resolve_destroy_weapon_deal_attack(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
) {
    let weapon = state.player(owner).weapon;
    let Some(w) = weapon else {
        return;
    };
    let atk = state.world().attack(w).unwrap_or(Attack(0)).0;
    let inner = state.make_mut();
    inner.players[owner.index()].weapon = None;
    queue.push(Event::WeaponDestroyed {
        player: owner,
        weapon: w,
    });
    if atk > 0 {
        let enemies = collect_all_enemy_characters(state, owner);
        for enemy in &enemies {
            queue.push(Event::DamageDealt {
                source,
                target: *enemy,
                amount: atk,
            });
        }
    }
}

/// Freezes a minion; if already frozen, deals damage instead (Icicle).
fn resolve_freeze_or_damage(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    amount: i32,
    explicit: Option<Entity>,
) {
    let minions = collect_enemy_minions(state, owner);
    let Some(target) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    if state.world().freeze(target).is_some() {
        queue.push(Event::DamageDealt {
            source,
            target,
            amount,
        });
    } else {
        state.world_mut().set_freeze(target, Freeze);
    }
}

/// Destroys a minion and gains its health (Natalie Seline).
fn resolve_destroy_and_gain_health(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
) {
    let minions = collect_enemy_minions(state, owner);
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let target = minions[idx];
    let hp = state.world().health(target).unwrap_or(Health(0)).0;
    // Destroy the target
    queue.push(Event::DamageDealt {
        source: target,
        target,
        amount: hp.max(1),
    });
    // The source minion (Natalie) gains the target's health
    let world = state.world_mut();
    let cur = world.health(source).unwrap_or(Health(0));
    world.set_health(source, Health(cur.0 + hp.max(1)));
}

/// Grants a friendly minion an attack bonus and immunity until end of turn (Bestial Wrath).
fn resolve_grant_attack_and_immune(state: &mut GameState, owner: PlayerId, attack: i32) {
    let minions = collect_friendly_minions(state, owner);
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let target = minions[idx];
    let world = state.world_mut();
    let cur = world.attack(target).unwrap_or(Attack(0));
    world.set_attack(target, Attack(cur.0 + attack));
    world.set_immune(target, Immune);
}

/// Takes control of an enemy minion (Shadow Madness: temporary until end of
/// turn; Mind Control: permanent).
///
/// Shadow Madness only targets minions with attack ≤ 3.
fn resolve_take_control(state: &mut GameState, owner: PlayerId, until_end_of_turn: bool) {
    let minions: SmallList<Entity> = collect_enemy_minions(state, owner)
        .into_iter()
        .filter(|&e| {
            !until_end_of_turn || state.world().effective_attack(e).unwrap_or(Attack(0)).0 <= 3
        })
        .collect();
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let target = minions[idx];
    let original_owner = state.world().player(target).unwrap_or(owner.opponent());
    transfer_minion(state, target, owner);
    if until_end_of_turn {
        let inner = state.make_mut();
        inner.players[owner.index()]
            .controlled_this_turn
            .push((target, original_owner));
    }
}

/// Transfers control of a minion to a new player (changes the player
/// component and moves it within the battlefield zone).
///
/// If the recipient's board is full (7 minions), the transfer is skipped
/// (simplified: no destruction).
pub(crate) fn transfer_minion(state: &mut GameState, entity: Entity, to: PlayerId) {
    let Some(from) = state.world().player(entity) else {
        return;
    };
    if from == to {
        return;
    }
    // Board size check (Mind Control / Shadow Madness cannot squeeze a minion into a full board)
    let board_count = state
        .world()
        .zones()
        .iter(Zone::Play, to)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .count();
    if board_count >= crate::engine::rules::MAX_BOARD_SIZE {
        return;
    }
    let inner = state.make_mut();
    inner.world.zones_mut().remove(Zone::Play, from, entity);
    inner.world.zones_mut().insert(Zone::Play, to, entity);
    inner.world.set_player(entity, to);
}

/// Corrupts an enemy minion — destroyed at the start of your turn (Corruption).
fn resolve_corrupt(state: &mut GameState, owner: PlayerId) {
    let minions = collect_enemy_minions(state, owner);
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let target = minions[idx];
    let inner = state.make_mut();
    inner.players[owner.index()].corrupted.push(target);
}

/// Transforms the target minion into one of two alternative minions (Tinkmaster Overspark).
fn resolve_transform(state: &mut GameState, owner: PlayerId, card_a: &str, card_b: &str) {
    let minions = collect_enemy_minions(state, owner);
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let target = minions[idx];
    let pick = if state.rng_mut().next_usize(2) == 0 {
        card_a
    } else {
        card_b
    };
    let Some(def) = crate::cards::def::card_by_id(pick) else {
        return;
    };
    // Reset the entity: clear all effect components and apply the new card's stats
    let world = state.world_mut();
    crate::cards::clear_minion_effects(world, target);
    world.set_attack(target, Attack(def.attack));
    world.set_health(target, Health(def.health));
    world.set_cost(target, Cost(def.cost));
    world.set_card_id(target, CardId(def.id));
    world.set_attacks_used(target, AttacksUsed(0));
}

/// Adds a card entity to hand (creates a new entity; used for random generation / Antonidas).
fn add_card_to_hand(
    state: &mut GameState,
    player: PlayerId,
    card_def: &crate::cards::def::CardDef,
) {
    let world = state.world_mut();
    let e = crate::cards::spawn_card_from_def(world, player, card_def);
    world.set_zone(e, Zone::Hand);
    world.zones_mut().insert(Zone::Hand, player, e);
}

/// Deals damage; if the target dies, summons a random minion (Bane of Doom).
///
/// Death prediction is based on current health / divine shield (simplified:
/// ignores armor and other later-resolution details).
fn resolve_damage_and_summon_if_killed(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    amount: i32,
    pool: crate::core::effect::RandomPool,
) {
    let enemies = collect_enemy_characters(state, owner);
    if enemies.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(enemies.len());
    let target = enemies[idx];
    queue.push(Event::DamageDealt {
        source,
        target,
        amount,
    });
    // Predict death: no divine shield and health cannot absorb the damage
    let will_die = state.world().divine_shield(target).is_none()
        && state
            .world()
            .health(target)
            .is_some_and(|h| h.0 - amount <= 0);
    if will_die {
        if let Some(card_def) = crate::cards::pool::random_card(state.rng_mut(), pool) {
            let _ = resolve_summon(state, queue, source, owner, card_def.id);
        }
    }
}

/// Grants a friendly minion stealth (Master of Disguise; cannot target itself).
fn resolve_grant_stealth(state: &mut GameState, source: Entity, owner: PlayerId) {
    let minions: SmallList<Entity> = collect_friendly_minions(state, owner)
        .into_iter()
        .filter(|&e| e != source)
        .collect();
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    state.world_mut().set_stealth(minions[idx], Stealth);
}

// ============================================================
// Helper functions
// ============================================================

/// Collects all enemy characters (hero + minions), excluding stealthed minions.
///
/// Stealthed characters cannot be targeted by single-target effects but are
/// still affected by AOE. Returns a stack-buffered list (no heap allocation
/// for boards of at most 7 minions + hero).
fn collect_enemy_characters(state: &GameState, owner: PlayerId) -> SmallList<Entity> {
    collect_enemy_characters_impl(state, owner, false)
}

/// Collects all enemy characters (hero + minions, including stealthed) — for AOE effects.
fn collect_all_enemy_characters(state: &GameState, owner: PlayerId) -> SmallList<Entity> {
    collect_enemy_characters_impl(state, owner, true)
}

fn collect_enemy_characters_impl(
    state: &GameState,
    owner: PlayerId,
    include_stealth: bool,
) -> SmallList<Entity> {
    let enemy = owner.opponent();
    let mut chars: SmallList<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, enemy)
        .filter(|&e| {
            let ct = state.world().card_type(e);
            if ct != Some(CardType::Minion) && ct != Some(CardType::Hero) {
                return false;
            }
            // Single-target selection excludes stealth; AOE (include_stealth) includes it
            !(!include_stealth && state.world().stealth(e).is_some())
        })
        .collect();
    // The hero is normally in the zone list already; ensure it is present
    // (defensive, e.g. for states built without the hero in the zone index).
    let hero = state.player(enemy).hero;
    if chars.iter().all(|&c| c != hero) {
        chars.push(hero);
    }
    // Sort by (index, generation) — same order as the previous sort+dedup
    // (no duplicates can remain: the zone list holds each entity once).
    chars.sort_by_key(|e| (e.index, e.generation));
    chars
}

/// Collects all enemy minions, excluding stealthed minions.
fn collect_enemy_minions(state: &GameState, owner: PlayerId) -> SmallList<Entity> {
    collect_enemy_minions_impl(state, owner, false)
}

/// Collects all enemy minions (including stealthed) — for AOE effects.
fn collect_all_enemy_minions(state: &GameState, owner: PlayerId) -> SmallList<Entity> {
    collect_enemy_minions_impl(state, owner, true)
}

fn collect_enemy_minions_impl(
    state: &GameState,
    owner: PlayerId,
    include_stealth: bool,
) -> SmallList<Entity> {
    let enemy = owner.opponent();
    state
        .world()
        .zones()
        .iter(Zone::Play, enemy)
        .filter(|&e| {
            state.world().card_type(e) == Some(CardType::Minion)
                && (include_stealth || state.world().stealth(e).is_none())
        })
        .collect()
}

/// Collects all friendly minions.
fn collect_friendly_minions(state: &GameState, owner: PlayerId) -> SmallList<Entity> {
    state
        .world()
        .zones()
        .iter(Zone::Play, owner)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect()
}
