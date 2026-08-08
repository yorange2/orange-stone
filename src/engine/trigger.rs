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
    Attack, AttacksUsed, CardId, CardType, Charge, Cost, Damage, Deathrattle, DivineShield,
    Durability, Enchantment, EnchantmentExpiry, Freeze, Health, Immune, Poison, Reborn, Stealth,
    Taunt, Trigger, TriggerEvent, TriggerTiming, Windfury,
};
use crate::core::effect::{CardEffect, EffectTarget};
use crate::core::entity::Entity;

/// Prophet Velen's card ID — his doubling is looked up by ID on the owner's board.
const VELEN_ID: &str = "PRIEST_012";

/// Hand-size cap (official rule, F-A11): a hand holds at most 10 cards.
/// Drawn cards past the cap are burned (destroyed, but the draw still counts
/// for deck depletion); generated cards past the cap are never created.
/// `rl::obs::MAX_HAND` must stay equal — pinned by the obs test.
pub const MAX_HAND_SIZE: usize = 10;

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
    // Hearthstone's re-validation semantics (roadmap G9): an explicit target
    // that is no longer in the legal candidate set at resolution time (gained
    // stealth, left play, became untargetable) makes the effect **fizzle** —
    // it does NOT fall back to a random target. With no explicit target, the
    // effect picks randomly at resolution (self-play).
    match explicit {
        // Explicit target still legal → hit it
        Some(t) if candidates.iter().any(|&c| c == t) => Some(t),
        // Explicit target provided but no longer legal → the effect fizzles
        Some(_) => None,
        // No explicit target → random pick at resolution
        None if candidates.is_empty() => None,
        None => Some(candidates[rng.next_usize(candidates.len())]),
    }
}

/// Whether an effect resolving from `source` is spell-powered — that is,
/// whether Spell Damage and Prophet Velen apply to its numbers.
///
/// In Hearthstone that is true for spells and hero powers, and false for
/// attacks, battlecries, deathrattles, and minion triggers. The source's card
/// type carries the distinction: a Spell entity is a spell (including a Secret,
/// which is a spell card), and a Hero entity reaching this function is a hero
/// power — a hero's *attack* damage never routes through `resolve_effect`.
fn is_spell_powered(state: &GameState, source: Entity) -> bool {
    matches!(
        state.world().card_type(source),
        Some(CardType::Spell | CardType::Hero)
    )
}

/// Rewrites an effect's damage and healing amounts for Spell Damage and
/// Prophet Velen.
///
/// The order matches Hearthstone: the Spell Damage bonus is added first, then
/// Velen doubles the result. Mind Blast (5) is 6 alongside a Kobold Geomancer,
/// 10 alongside Velen, and 12 alongside both.
///
/// Only damage and healing amounts are touched. Card draw, summon counts,
/// armor, and stat buffs are unaffected, as are effects whose source is not a
/// spell or hero power.
fn apply_spell_power(
    state: &GameState,
    source: Entity,
    owner: PlayerId,
    effect: CardEffect,
) -> CardEffect {
    if !is_spell_powered(state, source) {
        return effect;
    }
    // ImmuneToSpellpower (Core Set W2): Devouring Plague, Explosive Runes,
    // Healing Rain are explicitly exempt from the Spell Damage pipeline
    // (the official data marks them ImmuneToSpellpower).
    if state
        .world()
        .card_id(source)
        .is_some_and(|c| matches!(c.0, "CORE_BAR_311" | "CORE_LOOT_101" | "CORE_LOOT_373"))
    {
        return effect;
    }
    let bonus = state.world().total_spell_damage(owner);
    let velen = state
        .world()
        .zones()
        .iter(Zone::Play, owner)
        .any(|e| state.world().card_id(e).is_some_and(|c| c.0 == VELEN_ID));
    if bonus == 0 && !velen {
        return effect;
    }
    // Spell Damage adds, Velen then doubles. A 0-damage effect stays 0 — Spell
    // Damage only boosts effects that already deal damage.
    let adjust = |amount: i32| -> i32 {
        if amount <= 0 {
            return amount;
        }
        let with_bonus = amount + bonus;
        if velen { with_bonus * 2 } else { with_bonus }
    };
    match effect {
        CardEffect::DealDamage { amount, target } => CardEffect::DealDamage {
            amount: adjust(amount),
            target,
        },
        CardEffect::DealDamageToTwo { amount } => CardEffect::DealDamageToTwo {
            amount: adjust(amount),
        },
        CardEffect::DealDamageAndDraw {
            damage,
            target,
            draw,
        } => CardEffect::DealDamageAndDraw {
            damage: adjust(damage),
            target,
            draw,
        },
        CardEffect::DamageAndGainAttack {
            damage,
            attack_bonus,
            target,
        } => CardEffect::DamageAndGainAttack {
            damage: adjust(damage),
            attack_bonus,
            target,
        },
        CardEffect::DealDamageAndReturnToHand { amount, target } => {
            CardEffect::DealDamageAndReturnToHand {
                amount: adjust(amount),
                target,
            }
        }
        CardEffect::DamagePlayedMinion { amount } => CardEffect::DamagePlayedMinion {
            amount: adjust(amount),
        },
        CardEffect::FreezeOrDamage { amount } => CardEffect::FreezeOrDamage {
            amount: adjust(amount),
        },
        // Core Set W6 — damage spells (Frostbolt / Blizzard / Runed Orb)
        // take the Spell Damage bonus like any other spell.
        CardEffect::DamageAndFreeze { damage, target } => CardEffect::DamageAndFreeze {
            damage: adjust(damage),
            target,
        },
        CardEffect::DamageAllEnemyMinionsAndFreeze { damage } => {
            CardEffect::DamageAllEnemyMinionsAndFreeze {
                damage: adjust(damage),
            }
        }
        CardEffect::DamageAndAddRandomSpell { damage, target } => {
            CardEffect::DamageAndAddRandomSpell {
                damage: adjust(damage),
                target,
            }
        }
        CardEffect::DealDamageAndSummonIfKilled { amount, pool } => {
            CardEffect::DealDamageAndSummonIfKilled {
                amount: adjust(amount),
                pool,
            }
        }
        // Each missile is boosted individually, as in HS (Arcane Missiles with
        // a Spell Damage minion fires the same number of stronger missiles).
        CardEffect::DealDamageRandomly {
            amount,
            count,
            target,
        } => CardEffect::DealDamageRandomly {
            amount: adjust(amount),
            count,
            target,
        },
        // Velen doubles healing too; Spell Damage does not, so the bonus is
        // excluded here.
        CardEffect::RestoreHealth { amount, target } => CardEffect::RestoreHealth {
            amount: if velen { amount * 2 } else { amount },
            target,
        },
        other => other,
    }
}

/// Resolves a CardEffect into game events and enqueues them.
///
/// `source` is the effect source entity (the minion owning the effect).
/// `owner` is the player owning the source entity.
/// `explicit_target` is the target specified by the player (passed from
/// `Action::PlayCard`); when `None`, the engine chooses randomly
/// (`select_target` fallback).
/// `event_subject` is the entity a triggering event happened to (passed by
/// `fire_triggers` for `EffectTarget::EventSubject`); `None` for non-trigger
/// resolutions.
pub fn resolve_effect(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    effect: CardEffect,
    explicit_target: Option<Entity>,
    event_subject: Option<Entity>,
) {
    // Spell Damage and Prophet Velen adjust the numbers before anything else
    // resolves — see `apply_spell_power`.
    let effect = apply_spell_power(state, source, owner, effect);
    // Mayor Noggenfogger (Core Set W3a): all targets are chosen randomly —
    // the explicit target is ignored wherever a Noggenfogger is on the
    // active player's board.
    let explicit_target = if state
        .world()
        .zones()
        .iter(Zone::Play, state.active_player())
        .any(|e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| c.0 == "CORE_CFM_670")
        }) {
        None
    } else {
        explicit_target
    };
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
                event_subject,
            );
        }
        CardEffect::EquipWeapon { card_id } => {
            resolve_equip_weapon(state, queue, owner, card_id);
        }
        CardEffect::GainArmor { amount, target } => {
            resolve_gain_armor(state, owner, amount, target, explicit_target);
        }
        CardEffect::ReturnToHand { target } => {
            resolve_return_to_hand(state, queue, owner, source, target, explicit_target);
        }
        CardEffect::IncreaseCost { amount, target } => {
            resolve_increase_cost(state, owner, source, amount, target, explicit_target);
        }
        CardEffect::ReturnToHandAndIncreaseCost { amount } => {
            resolve_return_to_hand_and_increase_cost(state, queue, owner, source, amount);
        }
        CardEffect::DestroyMinion { target } => {
            resolve_destroy_minion(state, queue, owner, source, target, explicit_target);
        }
        CardEffect::SilenceMinion { target } => {
            resolve_silence(state, owner, target, explicit_target);
        }
        CardEffect::SetAttack { attack, target } => {
            resolve_set_attack(state, owner, attack, target, explicit_target);
        }
        CardEffect::SetHealth { health, target } => {
            resolve_set_health(state, owner, health, target, explicit_target);
        }
        CardEffect::RestoreHealth { amount, target } => {
            resolve_restore_health(state, queue, owner, amount, target, explicit_target);
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
        CardEffect::GainManaThisTurn { count } => {
            // The Coin: +1 mana this turn only (no permanent crystal)
            let inner = state.make_mut();
            let p = &mut inner.players[owner.index()];
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
            resolve_full_heal(state, queue, owner, target, explicit_target);
        }
        CardEffect::GrantWindfury { target } => {
            resolve_grant_windfury(state, owner, target, explicit_target);
        }
        CardEffect::GainStatsAndGrantWindfury {
            attack,
            health,
            target,
        } => {
            // Raging Worgen's Enrage — +1 Attack AND Windfury while damaged
            resolve_gain_stats(
                state,
                queue,
                source,
                owner,
                attack,
                health,
                target,
                explicit_target,
                event_subject,
            );
            match target {
                EffectTarget::Self_ => {
                    state
                        .world_mut()
                        .set_windfury(source, crate::core::component::Windfury);
                }
                _ => resolve_grant_windfury(state, owner, target, explicit_target),
            }
        }
        CardEffect::GrantCharge {
            target,
            attack_bonus,
        } => {
            resolve_grant_charge(
                state,
                source,
                owner,
                target,
                attack_bonus,
                explicit_target,
                event_subject,
            );
        }
        CardEffect::DiscoverDeckTop3 => {
            resolve_discover_deck_top3(state, source, owner);
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
        CardEffect::DiscardHand => {
            resolve_discard_hand(state, owner);
        }
        CardEffect::NextSpellDiscount { amount } => {
            state.make_mut().players[owner.index()].next_spell_discount = amount;
        }
        CardEffect::GrantAdjacentStatsAndDivineShield { attack, health } => {
            resolve_grant_adjacent_stats_and_divine_shield(state, source, owner, attack, health);
        }
        CardEffect::DestroyAllOtherMinionsAndDiscardHand => {
            resolve_destroy_all_other_minions_and_discard_hand(state, queue, source, owner);
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
            // The +Attack lands on the SAME target that took the damage
            // (battlecry-target-debt roadmap W14 — Cruel Taskmaster, Inner
            // Rage: "give it +2 Attack"); a fizzled target buffs nobody.
            if let Some(t) =
                resolve_deal_damage(state, queue, source, owner, damage, target, explicit_target)
            {
                state.world_mut().add_enchantment(
                    t,
                    Enchantment {
                        attack: attack_bonus,
                        health: 0,
                        cost: 0,
                        expiry: EnchantmentExpiry::Permanent,
                    },
                );
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
            let atk = state
                .world()
                .effective_attack(sacrifice)
                .unwrap_or(Attack(0));
            let hp = state
                .world()
                .effective_health(sacrifice)
                .unwrap_or(Health(0));
            // Destroy the sacrifice
            queue.push(Event::DamageDealt {
                source,
                target: sacrifice,
                amount: hp.0.max(1),
            });
            // Add the stats to the source minion as an enchantment (roadmap G4)
            state.world_mut().add_enchantment(
                source,
                Enchantment {
                    attack: atk.0,
                    health: hp.0,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
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
                    // Resurrected at 1 health: damage = base − 1 (enchantments
                    // were cleared when the minion left play)
                    let base = world.health(entity).unwrap_or(Health(0)).0;
                    let dmg = (base - 1).max(0);
                    if dmg > 0 {
                        world.set_damage(entity, Damage(dmg));
                    } else {
                        world.remove_damage(entity);
                    }
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
            // Copy (Faceless Manipulator-style, roadmap G4): copy the target's
            // current stats as an enchantment on top of the source's own base —
            // silencing the copy reverts to the source's printed stats.
            let atk = state
                .world()
                .effective_attack(target)
                .unwrap_or(Attack(0))
                .0;
            let hp = state
                .world()
                .effective_health(target)
                .unwrap_or(Health(0))
                .0;
            let world = state.world_mut();
            let base_atk = world.attack(source).unwrap_or(Attack(0)).0;
            let base_hp = world.health(source).unwrap_or(Health(0)).0;
            world.remove_enchantments(source);
            world.remove_damage(source);
            world.add_enchantment(
                source,
                Enchantment {
                    attack: atk - base_atk,
                    health: hp - base_hp,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
        }
        CardEffect::TempDebuff {
            attack_reduction,
            target,
        } => {
            let enemies = match target {
                EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner, Some(source)),
                _ => return,
            };
            let Some(enemy) = select_target(explicit_target, &enemies, state.rng_mut()) else {
                return;
            };
            state.world_mut().add_enchantment(
                enemy,
                Enchantment {
                    attack: -attack_reduction,
                    health: 0,
                    cost: 0,
                    expiry: EnchantmentExpiry::UntilEndOfTurn,
                },
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
        | CardEffect::DamagePlayedMinionAndExcess { .. }
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
        CardEffect::GrantAttackAndImmune { attack, target } => {
            resolve_grant_attack_and_immune(state, owner, attack, target, explicit_target);
        }
        CardEffect::TakeControlUntilEndOfTurn => {
            resolve_take_control(state, owner, true);
        }
        CardEffect::TakeControl => {
            resolve_take_control(state, owner, false);
        }
        CardEffect::TakeControlAttackLE { max_attack } => {
            resolve_take_control_attack_le(state, owner, max_attack);
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
            // Roadmap G6: the discover surfaces as a pending choice; the default
            // policy (GameEngine::apply) picks randomly via the embedded RNG —
            // the same distribution as the previous direct pick, preserving
            // determinism.
            let cards = crate::cards::pool::pool_cards(pool);
            if cards.is_empty() {
                return;
            }
            let options = cards
                .iter()
                .map(|c| format!("{} ({})", c.name, c.id))
                .collect();
            let pool_ids = cards.iter().map(|c| c.id.to_string()).collect();
            state.set_pending_choice(
                crate::core::state::ChoiceKind::Discover,
                source,
                options,
                pool_ids,
            );
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
        CardEffect::DrawCardByRace { count, race } => {
            resolve_draw_by_race(state, queue, owner, count, race);
        }
        CardEffect::Demonfire {
            damage,
            attack_bonus,
            health_bonus,
        } => {
            resolve_demonfire(
                state,
                queue,
                source,
                owner,
                damage,
                attack_bonus,
                health_bonus,
                explicit_target,
            );
        }
        CardEffect::GainStatsAndTaunt {
            attack,
            health,
            target,
        } => {
            resolve_gain_stats_and_taunt(
                state,
                source,
                owner,
                attack,
                health,
                target,
                explicit_target,
                event_subject,
            );
        }
        CardEffect::DestroyAndGainStats {
            attack,
            health,
            target,
        } => {
            resolve_destroy_and_gain_stats(
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
        CardEffect::DestroyRandomEnemySecret => {
            resolve_destroy_enemy_secrets(state, 1);
        }
        CardEffect::DestroyAllEnemySecretsAndGainStats { attack, health } => {
            resolve_destroy_enemy_secrets(state, i32::MAX);
            state.world_mut().add_enchantment(
                source,
                Enchantment {
                    attack,
                    health,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
        }
        CardEffect::DestroyAllEnemySecretsAndDraw { count } => {
            resolve_destroy_enemy_secrets(state, i32::MAX);
            for _ in 0..count {
                draw_card(state, queue, owner);
            }
        }
        CardEffect::AttachAttackDraw { count } => {
            resolve_attach_attack_draw(state, owner, count, explicit_target);
        }
        CardEffect::RemoveWeaponDurability { amount } => {
            resolve_remove_weapon_durability(state, queue, owner, amount);
        }
        CardEffect::GainAttackEqualToWeapon => {
            let atk = state
                .player(owner)
                .weapon
                .and_then(|w| state.world().attack(w))
                .map_or(0, |a| a.0);
            state.world_mut().add_enchantment(
                source,
                Enchantment {
                    attack: atk,
                    health: 0,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
        }
        CardEffect::EnemySpellsCostZero => {
            let inner = state.make_mut();
            inner.players[owner.opponent().index()].spells_cost_zero = true;
        }
        CardEffect::GiveOpponentManaCrystal { count } => {
            let inner = state.make_mut();
            let p = &mut inner.players[owner.opponent().index()];
            p.mana_crystals = (p.mana_crystals + count).min(10);
        }
        CardEffect::SetPlayedMinionHealth { .. } => {
            // Secret-context effect — resolved by the secret system with the
            // played minion (Repentance)
        }
        CardEffect::SilenceAllEnemyMinionsAndDraw { count } => {
            resolve_silence(state, owner, EffectTarget::AllEnemyMinions, None);
            for _ in 0..count {
                draw_card(state, queue, owner);
            }
        }
        CardEffect::SwapAttackAndHealth { target } => {
            resolve_swap_attack_health(state, owner, target, explicit_target);
        }
        CardEffect::FreezeAdjacent => {
            resolve_freeze_adjacent(state, owner);
        }
        CardEffect::GrantAdjacentTaunt => {
            resolve_grant_adjacent_taunt(state, source, owner);
        }
        CardEffect::GrantAdjacentSpellDamage { amount } => {
            resolve_grant_adjacent_spell_damage(state, source, owner, amount);
        }
        CardEffect::FullHealAndTaunt { target } => {
            resolve_full_heal_and_taunt(state, owner, target, explicit_target);
        }
        CardEffect::ChanceDraw { percent } => {
            // Nat Pagle — a percent chance to draw at the end of the turn
            if state.rng_mut().next_usize(100) < percent as usize {
                draw_card(state, queue, owner);
            }
        }
        CardEffect::GainStatsThisTurn {
            attack,
            health,
            target,
        } => {
            resolve_gain_stats_this_turn(
                state,
                source,
                owner,
                attack,
                health,
                target,
                explicit_target,
            );
        }
        CardEffect::GrantDivineShieldAllFriendly => {
            let minions = collect_friendly_minions(state, owner);
            for m in &minions {
                state
                    .world_mut()
                    .set_divine_shield(*m, crate::core::component::DivineShield);
            }
        }
        CardEffect::GrantDivineShield { target } => {
            let minions: SmallList<Entity> = match target {
                EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
                EffectTarget::Self_ => SmallList::from_iter([source]),
                _ => return,
            };
            if let Some(m) = select_target(explicit_target, &minions, state.rng_mut()) {
                state
                    .world_mut()
                    .set_divine_shield(m, crate::core::component::DivineShield);
            }
        }
        CardEffect::YseraAwakens { damage } => {
            // The Dream card spares Ysera herself (the generator minion)
            let mut chars = collect_friendly_minions(state, owner);
            chars.push(state.player(owner).hero);
            chars.extend(collect_all_enemy_characters(state, owner));
            for c in &chars {
                let is_ysera = state
                    .world()
                    .card_id(*c)
                    .is_some_and(|cid| cid.0 == "NEUTRAL_T21");
                if !is_ysera {
                    queue.push(Event::DamageDealt {
                        source,
                        target: *c,
                        amount: damage,
                    });
                }
            }
        }
        CardEffect::GainStatsAndTauntAllFriendly { attack, health } => {
            let minions = collect_friendly_minions(state, owner);
            for m in &minions {
                state.world_mut().add_enchantment(
                    *m,
                    Enchantment {
                        attack,
                        health,
                        cost: 0,
                        expiry: EnchantmentExpiry::Permanent,
                    },
                );
                state
                    .world_mut()
                    .set_taunt(*m, crate::core::component::Taunt);
            }
        }
        CardEffect::DrawAndDamageByCost => {
            // Holy Wrath — draw a card, deal damage equal to its mana cost
            if let Some(drawn) = draw_card_no_queue(state, queue, owner) {
                let cost = state.world().effective_cost(drawn).unwrap_or(Cost(0)).0;
                queue.push(Event::CardDrawn {
                    player: owner,
                    card: drawn,
                });
                if cost > 0 {
                    resolve_deal_damage(
                        state,
                        queue,
                        source,
                        owner,
                        cost,
                        EffectTarget::AnyEnemy,
                        None,
                    );
                }
            }
        }
        CardEffect::SwapWithHandMinion => {
            resolve_swap_with_hand_minion(state, source, owner);
        }
        // ------------------------------------------------------------
        // Pool-open effects (pool-open-cards roadmap M1) — read the
        // opponent's actual zones; see docs/pool-openness.md.
        // ------------------------------------------------------------
        CardEffect::CopyRandomEnemyHandCard { count } => {
            resolve_copy_random_enemy_zone(state, owner, count, Zone::Hand);
        }
        CardEffect::CopyRandomEnemyDeckCards { count } => {
            resolve_copy_random_enemy_zone(state, owner, count, Zone::Deck);
        }
        CardEffect::SummonRandomEnemyDeckMinion { fallback_card_id } => {
            resolve_summon_random_enemy_deck_minion(state, queue, source, owner, fallback_card_id);
        }
        CardEffect::CopyCastSpellToOtherPlayerHand => {
            // Lorewalker Cho — the copy goes to the caster's opponent. The
            // subject is the cast spell; a missing subject (e.g. the spell
            // entity is gone) is a no-op.
            if let Some(spell) = event_subject {
                if let Some(caster) = state.world().player(spell) {
                    copy_card_to_hand(state, spell, caster.opponent());
                }
            }
        }
        CardEffect::FillHandWithMinion { card_id } => {
            // Halazzi, the Lynx — fill the hand with 1/1 Lynxes (the
            // hand-size cap in add_card_to_hand stops the fill at 10)
            let Some(card_def) = crate::cards::def::card_by_id(card_id) else {
                return;
            };
            while state.world().zones().len(Zone::Hand, owner) < MAX_HAND_SIZE {
                add_card_to_hand(state, owner, card_def);
            }
        }
        CardEffect::ForceEnemyMinionsAttackThis => {
            // Mythical Terror — end-of-turn: every enemy minion that CAN
            // attack (not frozen, not exhausted, attack > 0, not
            // cant-attack) is forced to attack this character. Taunt is
            // bypassed (a forced attack is not a choice); the attacks run
            // through the normal AttackDeclared/ResolveAttack pipeline so
            // attack triggers and retaliation apply.
            let enemy = owner.opponent();
            let forced: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Play, enemy)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .filter(|&e| {
                    state.world().freeze(e).is_none()
                        && state.world().cant_attack(e).is_none()
                        && state.world().effective_attack(e).is_some_and(|a| a.0 > 0)
                        && !state
                            .world()
                            .attacks_used(e)
                            .is_some_and(|a| a.is_exhausted_with(state.world().max_attacks(e)))
                })
                .collect();
            for minion in forced {
                let atk = crate::engine::rules::compute_attacker_damage(state, minion);
                queue.push(Event::AttackDeclared {
                    attacker: minion,
                    defender: source,
                });
                queue.push(Event::ResolveAttack {
                    attacker: minion,
                    defender: source,
                    attacker_damage: atk,
                    retaliation_immune: false,
                });
            }
        }
        CardEffect::SpendCorpsesSummonCopy { cost } => {
            // Malignant Horror — end-of-turn: spend `cost` corpses to
            // summon a copy of this minion (does nothing with fewer
            // corpses; the summoned copy is a fresh card instance with its
            // own Reborn).
            let have = state.player(owner).corpses;
            if have < cost {
                return;
            }
            {
                let inner = state.make_mut();
                inner.players[owner.index()].corpses -= cost;
            }
            let card_def = crate::cards::def::card_by_id(
                state
                    .world()
                    .card_id(source)
                    .expect("source has a card id")
                    .0,
            );
            if let Some(card_def) = card_def {
                let _ = resolve_summon(state, queue, source, owner, card_def.id);
            }
        }
        CardEffect::DrawCardOutcast { normal, outcast } => {
            // Outcast (Core Set W2): the outcast amount applies when the
            // card was played from the hand edge (Spectral Sight, Crimson
            // Sigil Runner)
            let count = if state.world().outcast_played(source).is_some() {
                outcast
            } else {
                normal
            };
            for _ in 0..count {
                draw_card(state, queue, owner);
            }
        }
        CardEffect::OutcastDamage {
            amount,
            outcast_amount,
            target,
        } => {
            // Eye Beam — 3 damage, or 6 when played from the hand edge
            let damage = if state.world().outcast_played(source).is_some() {
                outcast_amount
            } else {
                amount
            };
            resolve_deal_damage(state, queue, source, owner, damage, target, None);
        }
        CardEffect::RestoreRandomFriendly { amount } => {
            // Healing Rain — `amount` 1-point heals randomly spread across
            // all friendly characters, hero included (overhealing wasted)
            let chars: SmallList<Entity> = {
                let mut all = collect_friendly_characters(state, owner, None);
                all.extend([state.player(owner).hero]);
                all
            };
            if chars.is_empty() {
                return;
            }
            for _ in 0..amount {
                let idx = state.rng_mut().next_usize(chars.len());
                let target = chars[idx];
                // Inline heal: reduce accumulated damage by 1 (same shape as
                // resolve_restore_health's nested helper)
                let dmg = state.world().damage(target).unwrap_or(Damage(0)).0;
                if dmg > 0 {
                    let new_dmg = (dmg - 1).max(0);
                    if new_dmg > 0 {
                        state.world_mut().set_damage(target, Damage(new_dmg));
                    } else {
                        state.world_mut().remove_damage(target);
                    }
                    fire_healed_trigger(state, queue, target);
                }
            }
        }
        CardEffect::DestroyEnemyLocation => {
            // Demolition Renovator — destroy an enemy location. The engine
            // has no Location card type until W8 (CORE_REV_990 Sanguine
            // Depths), so there are no targets and the effect fizzles.
        }
        CardEffect::DamageAndDrawIfHandEmpty { damage, target } => {
            // Quick Shot — deal damage; draw only when the hand is empty
            resolve_deal_damage(state, queue, source, owner, damage, target, explicit_target);
            let hand_count = state.world().zones().len(Zone::Hand, owner);
            if hand_count == 0 {
                draw_card(state, queue, owner);
            }
        }
        CardEffect::AoeDamageAndHealFriendly { damage, heal } => {
            // Holy Nova — damage all enemy minions, heal all friendly
            // characters (the Classic pool's damage-only version is a
            // simplification; the Core version is faithful)
            let enemies = collect_all_enemy_minions(state, owner);
            for minion in &enemies {
                queue.push(Event::DamageDealt {
                    source,
                    target: *minion,
                    amount: damage,
                });
            }
            let mut healed: SmallList<Entity> = SmallList::new();
            let hero = state.player(owner).hero;
            if heal_char(state.world_mut(), hero, heal) {
                healed.push(hero);
            }
            for m in collect_friendly_minions(state, owner) {
                if heal_char(state.world_mut(), m, heal) {
                    healed.push(m);
                }
            }
            for entity in healed {
                fire_healed_trigger(state, queue, entity);
            }
        }
        CardEffect::DamageAndDrawIfSurvives { damage, target } => {
            // Slam — deal damage; the target SURVIVING draws a card. The
            // survival is predicted before the damage resolves (same
            // approach as Bane of Doom): no divine shield and the health
            // can absorb the damage.
            if let Some(t) =
                resolve_deal_damage(state, queue, source, owner, damage, target, explicit_target)
            {
                let survives = state.world().divine_shield(t).is_some()
                    || state
                        .world()
                        .effective_health(t)
                        .is_some_and(|h| h.0 - damage > 0);
                if survives {
                    draw_card(state, queue, owner);
                }
            }
        }
        CardEffect::GainArmorAndDraw { armor, draw } => {
            // Shield Block — armor + draw (the Classic pool's armor-only
            // version is a simplification)
            {
                let inner = state.make_mut();
                inner.players[owner.index()].armor += armor;
            }
            for _ in 0..draw {
                draw_card(state, queue, owner);
            }
        }
        CardEffect::DamageAndDrawIfKilled { damage, target } => {
            // Mortal Coil — deal damage; the target DYING draws a card
            // (predicted before the damage resolves, same as Bane of Doom)
            if let Some(t) =
                resolve_deal_damage(state, queue, source, owner, damage, target, explicit_target)
            {
                let dies = state.world().divine_shield(t).is_none()
                    && state
                        .world()
                        .effective_health(t)
                        .is_some_and(|h| h.0 - damage <= 0);
                if dies {
                    draw_card(state, queue, owner);
                }
            }
        }
        CardEffect::DamageAndGainArmor {
            damage,
            armor,
            target,
        } => {
            // Bash — deal damage and gain armor
            resolve_deal_damage(state, queue, source, owner, damage, target, explicit_target);
            {
                let inner = state.make_mut();
                inner.players[owner.index()].armor += armor;
            }
        }
        CardEffect::GainPoisonousToFriendlyUndead => {
            // Poison Breath — give a friendly Undead minion Poisonous
            let minions: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Play, owner)
                .filter(|&e| {
                    state.world().card_type(e) == Some(CardType::Minion)
                        && state
                            .world()
                            .has_race(e, crate::core::component::Race::Undead)
                })
                .collect();
            if let Some(target) = select_target(explicit_target, &minions, state.rng_mut()) {
                state.world_mut().set_poison(target, Poison);
            }
        }
        CardEffect::TransformToMinion { card_id } => {
            // Hex — transform a minion into the given card (a real
            // transform: effects cleared, deathrattles do not fire)
            let minions: SmallList<Entity> = {
                let mut all = collect_friendly_minions(state, owner);
                all.extend(collect_enemy_minions(state, owner, None));
                all
            };
            let Some(target) = select_target(explicit_target, &minions, state.rng_mut()) else {
                return;
            };
            let Some(def) = crate::cards::def::card_by_id(card_id) else {
                return;
            };
            let world = state.world_mut();
            crate::cards::clear_minion_effects(world, target);
            world.set_attack(target, Attack(def.attack));
            world.set_health(target, Health(def.health));
            world.set_cost(target, Cost(def.cost));
            world.set_card_id(target, CardId(def.id));
            world.set_attacks_used(target, AttacksUsed(0));
            // The transform target's static keywords apply (Frog: Taunt)
            if def.taunt {
                world.set_taunt(target, Taunt);
            }
            if def.stealth {
                world.set_stealth(target, Stealth);
            }
            if def.divine_shield {
                world.set_divine_shield(target, DivineShield);
            }
            if def.windfury {
                world.set_windfury(target, Windfury);
            }
            if def.charge {
                world.set_charge(target, Charge);
            }
        }
        CardEffect::GrantDeathrattleToTarget { card_id } => {
            // Spikeridged Steed — single-target deathrattle grant
            let minions: SmallList<Entity> = {
                let mut all = collect_friendly_minions(state, owner);
                all.extend(collect_enemy_minions(state, owner, None));
                all
            };
            let Some(target) = select_target(explicit_target, &minions, state.rng_mut()) else {
                return;
            };
            let dr = Deathrattle(CardEffect::SummonMinion { card_id });
            state.world_mut().set_deathrattle(target, dr);
        }
        CardEffect::AoeDamageAndDraw { damage, draw } => {
            // Fan of Knives — damage all enemy minions, draw
            let enemies = collect_all_enemy_minions(state, owner);
            for minion in &enemies {
                queue.push(Event::DamageDealt {
                    source,
                    target: *minion,
                    amount: damage,
                });
            }
            for _ in 0..draw {
                draw_card(state, queue, owner);
            }
        }
        CardEffect::SummonRandomFishFromDeck => {
            // Finja — summon a random Murloc from the owner's deck, but
            // only when THIS minion is the one attacking (the event subject)
            if event_subject != Some(source) {
                return;
            }
            // scanned for the first Murloc in deck order like the
            // race-draw effects
            let fish = state.world().zones().iter(Zone::Deck, owner).find(|&e| {
                state.world().card_id(e).is_some_and(|c| {
                    crate::cards::def::card_by_id(c.0)
                        .is_some_and(|d| d.race == Some(crate::core::component::Race::Murloc))
                })
            });
            if let Some(fish) = fish {
                if let Some(def) = state
                    .world()
                    .card_id(fish)
                    .and_then(|c| crate::cards::def::card_by_id(c.0))
                {
                    let _ = resolve_summon(state, queue, source, owner, def.id);
                }
            }
        }
        CardEffect::DamageAndSummon {
            damage,
            target,
            card_id,
        } => {
            // Wound Prey — damage and summon the token
            resolve_deal_damage(state, queue, source, owner, damage, target, explicit_target);
            let _ = resolve_summon(state, queue, source, owner, card_id);
        }
        CardEffect::RehgarBolt => {
            // Rehgar Earthfury — THIS or an adjacent friendly minion attacks
            if event_subject != Some(source) {
                let adjacent: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Play, owner)
                    .filter(|&e| {
                        e != source && state.world().card_type(e) == Some(CardType::Minion)
                    })
                    .collect();
                let Some(subject) = event_subject else {
                    return;
                };
                let is_adjacent = adjacent.iter().enumerate().any(|(i, &e)| {
                    e == subject
                        && (i > 0 && adjacent[i - 1] == source
                            || i + 1 < adjacent.len() && adjacent[i + 1] == source)
                });
                if !is_adjacent {
                    return;
                }
            }
            // Add a Lightning Bolt to hand (the classic SHAMAN_002 until
            // CORE_EX1_238 lands in W7)
            let Some(bolt) = crate::cards::def::card_by_id("SHAMAN_002") else {
                return;
            };
            add_card_to_hand(state, owner, bolt);
        }
        CardEffect::DamageTwoDrawIfKilled { damage } => {
            // Consumption — two random enemy minions take damage; each kill
            // draws (prediction-based)
            let minions = collect_enemy_minions(state, owner, Some(source));
            let targets: SmallList<Entity> = if minions.len() <= 2 {
                minions
            } else {
                let mut picks = SmallList::new();
                let idx1 = state.rng_mut().next_usize(minions.len());
                picks.push(minions[idx1]);
                let mut idx2 = state.rng_mut().next_usize(minions.len() - 1);
                if idx2 >= idx1 {
                    idx2 += 1;
                }
                picks.push(minions[idx2]);
                picks
            };
            for t in targets {
                queue.push(Event::DamageDealt {
                    source,
                    target: t,
                    amount: damage,
                });
                let dies = state.world().divine_shield(t).is_none()
                    && state
                        .world()
                        .effective_health(t)
                        .is_some_and(|h| h.0 - damage <= 0);
                if dies {
                    draw_card(state, queue, owner);
                }
            }
        }
        CardEffect::FreezeAndDiscoverSpell => {
            // Death's Advance — freeze a character (explicit/random) and
            // Discover a spell (pending choice; random default)
            let chars: SmallList<Entity> = {
                let mut all = collect_friendly_characters(state, owner, None);
                all.extend(collect_enemy_characters(state, owner, None));
                all
            };
            if let Some(target) = select_target(explicit_target, &chars, state.rng_mut()) {
                state.world_mut().set_freeze(target, Freeze);
            }
            let spells: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Spell)
                    .collect();
            if spells.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(spells.len());
            let spell = spells[idx];
            add_card_to_hand(state, owner, spell);
        }
        CardEffect::HenchThugBuff => {
            // Hench-Clan Thug — the OWNER'S HERO attacked
            let hero = state.player(owner).hero;
            if event_subject != Some(hero) {
                return;
            }
            let world = state.world_mut();
            let base = world.attack(source).unwrap_or(Attack(0));
            world.set_attack(source, Attack(base.0 + 1));
            let base_hp = world.health(source).unwrap_or(Health(1));
            world.set_health(source, Health(base_hp.0 + 1));
        }
        CardEffect::SummonRecruitsAndEquipWeapon => {
            // Muster for Battle — three 1/1 recruits + a 1/4 weapon
            for _ in 0..3 {
                let _ = resolve_summon(state, queue, source, owner, "CORE_GVG_061t");
            }
            if let Some(weapon) = crate::cards::def::card_by_id("PALADIN_015") {
                let e = crate::cards::spawn_card_from_def(state.world_mut(), owner, weapon);
                state.world_mut().set_zone(e, Zone::Play);
                state.world_mut().zones_mut().insert(Zone::Play, owner, e);
                let inner = state.make_mut();
                inner.players[owner.index()].weapon = Some(e);
            }
        }
        CardEffect::BuffAndSummonRandomCost2 => {
            // Silvermoon Portal — buff a minion and summon a random 2-cost
            let minions: SmallList<Entity> = {
                let mut all = collect_friendly_minions(state, owner);
                all.extend(collect_enemy_minions(state, owner, None));
                all
            };
            if let Some(target) = select_target(explicit_target, &minions, state.rng_mut()) {
                let world = state.world_mut();
                let base = world.attack(target).unwrap_or(Attack(0));
                world.set_attack(target, Attack(base.0 + 2));
                let base_hp = world.health(target).unwrap_or(Health(1));
                world.set_health(target, Health(base_hp.0 + 2));
            }
            let cost2: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| {
                        c.card_type == CardType::Minion && c.cost == 2 && !c.id.ends_with('t')
                    })
                    .collect();
            if let Some(&pick) = cost2.get(state.rng_mut().next_usize(cost2.len())) {
                let _ = resolve_summon(state, queue, source, owner, pick.id);
            }
        }
        CardEffect::DamageAndSummonCopyIfKilled { damage } => {
            // Initiation — 4 damage; a kill summons a fresh copy
            if let Some(t) = resolve_deal_damage(
                state,
                queue,
                source,
                owner,
                damage,
                EffectTarget::AnyMinion,
                explicit_target,
            ) {
                let dies = state.world().divine_shield(t).is_none()
                    && state
                        .world()
                        .effective_health(t)
                        .is_some_and(|h| h.0 - damage <= 0);
                if dies {
                    if let Some(def) = state
                        .world()
                        .card_id(t)
                        .and_then(|c| crate::cards::def::card_by_id(c.0))
                    {
                        let _ = resolve_summon(state, queue, source, owner, def.id);
                    }
                }
            }
        }
        CardEffect::KeymasterCopy => {
            // Keymaster Alabaster — the OPPONENT drew a card; add a 1-cost
            // copy to hand
            let Some(subject) = event_subject else {
                return;
            };
            if state.world().player(subject) == Some(owner) {
                return; // the owner's own draw
            }
            let Some(def) = state
                .world()
                .card_id(subject)
                .and_then(|c| crate::cards::def::card_by_id(c.0))
            else {
                return;
            };
            let e = crate::cards::spawn_card_from_def(state.world_mut(), owner, def);
            state.world_mut().set_cost(e, Cost(1));
            state.world_mut().set_zone(e, Zone::Hand);
            state.world_mut().zones_mut().insert(Zone::Hand, owner, e);
        }
        CardEffect::FordragonBuff => {
            // Highlord Fordragon — a friendly minion lost Divine Shield;
            // buff a random minion in hand +5/+5
            let hand: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            if hand.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(hand.len());
            let target = hand[idx];
            let world = state.world_mut();
            let base = world.attack(target).unwrap_or(Attack(0));
            world.set_attack(target, Attack(base.0 + 5));
            let base_hp = world.health(target).unwrap_or(Health(1));
            world.set_health(target, Health(base_hp.0 + 5));
        }
        CardEffect::DamageAndSummonVoidwalkers { damage, target } => {
            // Demonic Assault — damage and two 1/3 Voidwalkers with Taunt
            resolve_deal_damage(state, queue, source, owner, damage, target, explicit_target);
            for _ in 0..2 {
                let _ = resolve_summon(state, queue, source, owner, "CS2_065");
            }
        }
        CardEffect::DamageAndAddToHand { damage, card_id } => {
            // First Flame — damage a minion and add the token to hand
            resolve_deal_damage(
                state,
                queue,
                source,
                owner,
                damage,
                EffectTarget::AnyMinion,
                explicit_target,
            );
            if let Some(def) = crate::cards::def::card_by_id(card_id) {
                add_card_to_hand(state, owner, def);
            }
        }
        CardEffect::AddRandomOtherClassSpells { count, min_cost } => {
            // Jackpot! — random spells of 5+ cost from OTHER classes
            for _ in 0..count {
                let spells: SmallList<&'static crate::cards::def::CardDef> =
                    crate::cards::sets::ALL_CARDS
                        .iter()
                        .filter(|c| {
                            c.card_type == CardType::Spell
                                && c.cost >= min_cost
                                && crate::cards::pool::is_other_class_card_for(c, owner)
                        })
                        .collect();
                if spells.is_empty() {
                    return;
                }
                let idx = state.rng_mut().next_usize(spells.len());
                add_card_to_hand(state, owner, spells[idx]);
            }
        }
        CardEffect::SummonFelbatOnDraw => {
            // Eredar Deceptor — the OWNER drew a card; summon a 1/1 Demon
            // with Rush
            if event_subject.is_none()
                || state.world().player(event_subject.unwrap()) != Some(owner)
            {
                return;
            }
            let _ = resolve_summon(state, queue, source, owner, "CORE_TTN_843t");
        }
        CardEffect::SpendCorpsesSummonRandomMinion { max } => {
            // Corpse Farm — spend up to 8 corpses for a random minion of
            // that cost
            let have = state.player(owner).corpses;
            if have == 0 {
                return;
            }
            let spend = have.min(max) as i32;
            {
                let inner = state.make_mut();
                inner.players[owner.index()].corpses -= spend as u32;
            }
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Minion && c.cost == spend)
                    .collect();
            if candidates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            let _ = resolve_summon(state, queue, source, owner, candidates[idx].id);
        }
        CardEffect::GainStatsAndDraw {
            attack,
            health,
            target: _,
            draw,
        } => {
            // Power Word: Shield / Hand of A'dal — buff and draw
            let minions: SmallList<Entity> = {
                let mut all = collect_friendly_minions(state, owner);
                all.extend(collect_enemy_minions(state, owner, None));
                all
            };
            if let Some(t) = select_target(explicit_target, &minions, state.rng_mut()) {
                let world = state.world_mut();
                let base = world.attack(t).unwrap_or(Attack(0));
                world.set_attack(t, Attack(base.0 + attack));
                let base_hp = world.health(t).unwrap_or(Health(1));
                world.set_health(t, Health(base_hp.0 + health));
            }
            for _ in 0..draw {
                draw_card(state, queue, owner);
            }
        }
        CardEffect::DamageUndamaged { damage } => {
            // Backstab — 2 damage to an UNDAMAGED minion
            let minions: SmallList<Entity> = {
                let mut all = collect_friendly_minions(state, owner);
                all.extend(collect_enemy_minions(state, owner, None));
                all
            };
            let Some(t) = select_target(explicit_target, &minions, state.rng_mut()) else {
                return;
            };
            if state.world().damage(t).is_some_and(|d| d.0 > 0) {
                return; // damaged — Backstab fizzles
            }
            queue.push(Event::DamageDealt {
                source,
                target: t,
                amount: damage,
            });
        }
        CardEffect::DamageMinionAndSelfHero { damage } => {
            // Spirit Bomb — damage a minion and the caster's hero
            resolve_deal_damage(
                state,
                queue,
                source,
                owner,
                damage,
                EffectTarget::AnyMinion,
                explicit_target,
            );
            let hero = state.player(owner).hero;
            queue.push(Event::DamageDealt {
                source,
                target: hero,
                amount: damage,
            });
        }
        CardEffect::GainHeroAttackAndDraw { attack } => {
            // Chaos Strike — hero attack this turn + draw
            let hero = state.player(owner).hero;
            {
                let world = state.world_mut();
                let base = world.attack(hero).unwrap_or(Attack(0));
                world.set_attack(hero, Attack(base.0 + attack));
                world.add_enchantment(
                    hero,
                    Enchantment {
                        attack,
                        health: 0,
                        cost: 0,
                        expiry: EnchantmentExpiry::UntilEndOfTurn,
                    },
                );
            }
            draw_card(state, queue, owner);
        }
        CardEffect::GainArmorAndSummonDeckMinion { armor, max_cost } => {
            // Oaken Summons — armor + a deck minion of at most max_cost
            {
                let inner = state.make_mut();
                inner.players[owner.index()].armor += armor;
            }
            let deck: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Deck, owner)
                .filter(|&e| {
                    state
                        .world()
                        .card_id(e)
                        .and_then(|c| crate::cards::def::card_by_id(c.0))
                        .is_some_and(|d| d.card_type == CardType::Minion && d.cost <= max_cost)
                })
                .collect();
            if let Some(&m) = deck.iter().next() {
                if let Some(def) = state
                    .world()
                    .card_id(m)
                    .and_then(|c| crate::cards::def::card_by_id(c.0))
                {
                    let _ = resolve_summon(state, queue, source, owner, def.id);
                }
            }
        }
        CardEffect::GainArmorAndDrawOnHeroAttack { armor } => {
            // Hookfist-3000 — the OWNER'S HERO attacked
            let hero = state.player(owner).hero;
            if event_subject != Some(hero) {
                return;
            }
            {
                let inner = state.make_mut();
                inner.players[owner.index()].armor += armor;
            }
            draw_card(state, queue, owner);
        }
        CardEffect::SummonAllCompanions => {
            // Call of the Wild — all three Animal Companions
            for companion in ["HUNTER_023a", "HUNTER_023b", "HUNTER_023c"] {
                let _ = resolve_summon(state, queue, source, owner, companion);
            }
        }
        CardEffect::DamageFreezeAllAndSummon { damage, card_id } => {
            // Frostwyrm's Fury — damage, freeze all enemy minions, summon
            resolve_deal_damage(
                state,
                queue,
                source,
                owner,
                damage,
                EffectTarget::AnyEnemy,
                explicit_target,
            );
            let enemies = collect_all_enemy_minions(state, owner);
            for m in &enemies {
                state.world_mut().set_freeze(*m, Freeze);
            }
            let _ = resolve_summon(state, queue, source, owner, card_id);
        }
        CardEffect::DestroyHighestAttackEnemy => {
            // Asphyxiate — destroy the highest-Attack enemy minion
            let mut enemies: SmallList<Entity> = collect_enemy_minions(state, owner, None);
            if enemies.is_empty() {
                return;
            }
            enemies.sort_by_key(|&e| state.world().effective_attack(e).unwrap_or(Attack(0)).0);
            let target = enemies[enemies.len() - 1];
            let hp = state.world().effective_health(target).map_or(0, |h| h.0);
            queue.push(Event::DamageDealt {
                source,
                target,
                amount: hp.max(1),
            });
        }
        CardEffect::SummonZombiesWithCorpseReborn { corpses } => {
            // Tomb Guardians — two 2/2 Taunt Zombies; corpses give Reborn
            let mut summoned = Vec::new();
            for _ in 0..2 {
                if let Some(e) = resolve_summon(state, queue, source, owner, "CORE_RLK_118t") {
                    summoned.push(e);
                }
            }
            let have = state.player(owner).corpses;
            if have >= corpses && !summoned.is_empty() {
                {
                    let inner = state.make_mut();
                    inner.players[owner.index()].corpses -= corpses;
                }
                for e in summoned {
                    state.world_mut().set_reborn(e, Reborn);
                }
            }
        }
        CardEffect::TransformSelfToCastSpell => {
            // Shadow of Demise — transform into a copy of the just-cast spell
            let Some(subject) = event_subject else {
                return;
            };
            let Some(def) = state
                .world()
                .card_id(subject)
                .and_then(|c| crate::cards::def::card_by_id(c.0))
            else {
                return;
            };
            if def.card_type != CardType::Spell {
                return;
            }
            let world = state.world_mut();
            world.set_card_id(source, CardId(def.id));
            world.set_cost(source, Cost(def.cost));
        }
        CardEffect::BuffHandMinionsWithCorpses { corpses } => {
            // Blood Tap — hand minions +1/+1; corpses for another +1/+1
            let hand: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            let have = state.player(owner).corpses;
            let extra = if have >= corpses { 1 } else { 0 };
            if extra == 1 {
                let inner = state.make_mut();
                inner.players[owner.index()].corpses -= corpses;
            }
            let bonus = 1 + extra;
            for e in hand {
                let world = state.world_mut();
                let base = world.attack(e).unwrap_or(Attack(0));
                world.set_attack(e, Attack(base.0 + bonus));
                let base_hp = world.health(e).unwrap_or(Health(1));
                world.set_health(e, Health(base_hp.0 + bonus));
            }
        }
        CardEffect::RestoreHealthAndDraw { amount, target: _ } => {
            // Flash of Light — heal and draw
            let chars: SmallList<Entity> = {
                let mut all = collect_friendly_characters(state, owner, None);
                all.extend(collect_enemy_characters(state, owner, None));
                all
            };
            if let Some(t) = select_target(explicit_target, &chars, state.rng_mut()) {
                if heal_char(state.world_mut(), t, amount) {
                    fire_healed_trigger(state, queue, t);
                }
            }
            draw_card(state, queue, owner);
        }
        CardEffect::DrawIfUnspentMana => {
            // Crystal Merchant — end of turn with unspent mana: draw
            let p = state.player(owner);
            if p.current_mana > 0 {
                draw_card(state, queue, owner);
            }
        }
        CardEffect::GainArmorAndSummonRandomCost { armor, cost } => {
            // Ironforge Portal — armor and a random minion of exactly cost
            {
                let inner = state.make_mut();
                inner.players[owner.index()].armor += armor;
            }
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Minion && c.cost == cost)
                    .collect();
            if candidates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            let _ = resolve_summon(state, queue, source, owner, candidates[idx].id);
        }
        CardEffect::GainStatsIfHealedThisTurn { attack, health } => {
            // Priest of An'she — restored Health this turn?
            if !state.player(owner).healed_this_turn {
                return;
            }
            let world = state.world_mut();
            let base = world.attack(source).unwrap_or(Attack(0));
            world.set_attack(source, Attack(base.0 + attack));
            let base_hp = world.health(source).unwrap_or(Health(1));
            world.set_health(source, Health(base_hp.0 + health));
        }
        CardEffect::BattleToTheDeath => {
            // Warmaul Challenger — both deal their attack to each other
            let minions: SmallList<Entity> = collect_enemy_minions(state, owner, None);
            let Some(target) = select_target(explicit_target, &minions, state.rng_mut()) else {
                return;
            };
            let atk = state
                .world()
                .effective_attack(source)
                .unwrap_or(Attack(0))
                .0;
            let target_atk = state
                .world()
                .effective_attack(target)
                .unwrap_or(Attack(0))
                .0;
            queue.push(Event::DamageDealt {
                source,
                target,
                amount: atk,
            });
            queue.push(Event::DamageDealt {
                source: target,
                target: source,
                amount: target_atk,
            });
        }
        CardEffect::NextDemonDiscount { amount } => {
            // Raging Felscreamer — the next Demon costs less
            let inner = state.make_mut();
            inner.players[owner.index()].next_demon_discount =
                (inner.players[owner.index()].next_demon_discount + amount).max(0);
        }
        CardEffect::BuffHandMinions { attack, health } => {
            // Grimestreet Outfitter — all minions in hand
            let hand: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            for e in hand {
                let world = state.world_mut();
                let base = world.attack(e).unwrap_or(Attack(0));
                world.set_attack(e, Attack(base.0 + attack));
                let base_hp = world.health(e).unwrap_or(Health(1));
                world.set_health(e, Health(base_hp.0 + health));
            }
        }
        CardEffect::SummonRandomEnemyHandMinion => {
            // Dirty Rat — the OPPONENT summons a random minion from their
            // hand (pool-open: reads the opponent's hand)
            let enemy = owner.opponent();
            let hand: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, enemy)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            if hand.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(hand.len());
            let picked = hand[idx];
            if let Some(def) = state
                .world()
                .card_id(picked)
                .and_then(|c| crate::cards::def::card_by_id(c.0))
            {
                // move the actual card to the enemy's board (official
                // semantics: the card leaves the hand)
                let world = state.world_mut();
                world.zones_mut().remove(Zone::Hand, enemy, picked);
                world.set_zone(picked, Zone::Play);
                world.zones_mut().insert(Zone::Play, enemy, picked);
                world.set_attacks_used(picked, AttacksUsed(1));
                let _ = def;
            }
        }
        CardEffect::DrawForBoth => {
            // Prize Vendor — each player draws
            draw_card(state, queue, owner);
            draw_card(state, queue, owner.opponent());
        }
        CardEffect::NextComboDiscount { amount } => {
            // Foxy Fraud — the next Combo card costs less this turn
            let inner = state.make_mut();
            inner.players[owner.index()].next_combo_discount =
                (inner.players[owner.index()].next_combo_discount + amount).max(0);
        }
        CardEffect::AddRandomMageSpells { count } => {
            // Babbling Bookcase — random Mage spells
            for _ in 0..count {
                let spells: SmallList<&'static crate::cards::def::CardDef> =
                    crate::cards::sets::MAGE_CLASSIC
                        .iter()
                        .filter(|c| c.card_type == CardType::Spell)
                        .collect();
                if spells.is_empty() {
                    return;
                }
                let idx = state.rng_mut().next_usize(spells.len());
                add_card_to_hand(state, owner, spells[idx]);
            }
        }
        CardEffect::DamageEnemyHeroAndHealSelf { amount } => {
            // Lifedrinker — enemy hero damage + friendly hero heal
            let enemy_hero = state.player(owner.opponent()).hero;
            queue.push(Event::DamageDealt {
                source,
                target: enemy_hero,
                amount,
            });
            let hero = state.player(owner).hero;
            if heal_char(state.world_mut(), hero, amount) {
                fire_healed_trigger(state, queue, hero);
            }
        }
        CardEffect::LoseHealthPerOpponentHandCard => {
            // Witchwood Grizzly — lose 1 Health per card in the opponent's
            // hand
            let hand_count = state.world().zones().len(Zone::Hand, owner.opponent()) as i32;
            let base = state.world().health(source).unwrap_or(Health(1)).0;
            state
                .world_mut()
                .set_health(source, Health((base - hand_count).max(0)));
        }
        CardEffect::GrantRandomFriendlyDivineShieldTaunt => {
            // Coghammer — random friendly minion gets Shield and Taunt
            let minions: SmallList<Entity> = collect_friendly_minions(state, owner);
            let Some(target) = select_target(None, &minions, state.rng_mut()) else {
                return;
            };
            state.world_mut().set_divine_shield(target, DivineShield);
            state.world_mut().set_taunt(target, Taunt);
        }
        CardEffect::RemoveTopEnemyDeckCard => {
            // Gnomeferatu — remove the top card of the opponent's deck
            // (pool-open: reads the opponent's deck)
            let enemy = owner.opponent();
            let top = state.world().zones().iter(Zone::Deck, enemy).next();
            if let Some(top) = top {
                state.world_mut().move_to_zone(top, Zone::Graveyard).ok();
            }
        }
        CardEffect::DiscoverSpellAndHealCost => {
            // Ivory Knight — a spell (Discover simplified to random) and
            // heal equal to its cost
            let spells: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Spell)
                    .collect();
            if spells.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(spells.len());
            let spell = spells[idx];
            add_card_to_hand(state, owner, spell);
            let hero = state.player(owner).hero;
            if heal_char(state.world_mut(), hero, spell.cost) {
                fire_healed_trigger(state, queue, hero);
            }
        }
        CardEffect::DrawBeastDragonMurloc => {
            // The Curator — draw a Beast, a Dragon and a Murloc from the deck
            for race in [
                crate::core::component::Race::Beast,
                crate::core::component::Race::Dragon,
                crate::core::component::Race::Murloc,
            ] {
                let found = state.world().zones().iter(Zone::Deck, owner).find(|&e| {
                    state.world().card_id(e).is_some_and(|c| {
                        crate::cards::def::card_by_id(c.0).is_some_and(|d| d.race == Some(race))
                    })
                });
                if let Some(card) = found {
                    let world = state.world_mut();
                    world.set_zone(card, Zone::Hand);
                    world.zones_mut().insert(Zone::Hand, owner, card);
                }
            }
        }
        CardEffect::AddRandomOtherClassCard => {
            // Swashburglar — a random card from another class
            let cards: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| crate::cards::pool::is_other_class_card_for(c, owner))
                    .collect();
            if cards.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(cards.len());
            add_card_to_hand(state, owner, cards[idx]);
        }
        CardEffect::AddRandomShamanSpell => {
            // Witch's Apprentice — a random Shaman spell
            let spells: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::SHAMAN_CLASSIC
                    .iter()
                    .filter(|c| c.card_type == CardType::Spell)
                    .collect();
            if spells.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(spells.len());
            add_card_to_hand(state, owner, spells[idx]);
        }
        CardEffect::DamageSelfHero { damage } => {
            // Vulgar Homunculus — damage the friendly hero
            let hero = state.player(owner).hero;
            queue.push(Event::DamageDealt {
                source,
                target: hero,
                amount: damage,
            });
        }
        CardEffect::SummonTwoCopiesOfSelf => {
            // Nerubian Swarmguard — two copies of this minion
            if let Some(def) = state
                .world()
                .card_id(source)
                .and_then(|c| crate::cards::def::card_by_id(c.0))
            {
                if let Some(copy) = resolve_summon(state, queue, source, owner, def.id) {
                    // the summoned copies do not re-trigger the battlecry
                    state.world_mut().remove_battlecry(copy);
                }
                if let Some(copy) = resolve_summon(state, queue, source, owner, def.id) {
                    state.world_mut().remove_battlecry(copy);
                }
            }
        }
        CardEffect::SpendCorpsesDamageRandom { max, damage } => {
            // Marrow Manipulator — corpses for random enemy damage
            let have = state.player(owner).corpses;
            if have == 0 {
                return;
            }
            let spend = have.min(max);
            {
                let inner = state.make_mut();
                inner.players[owner.index()].corpses -= spend;
            }
            for _ in 0..spend {
                let enemies = collect_enemy_characters(state, owner, Some(source));
                if enemies.is_empty() {
                    return;
                }
                let idx = state.rng_mut().next_usize(enemies.len());
                queue.push(Event::DamageDealt {
                    source,
                    target: enemies[idx],
                    amount: damage,
                });
            }
        }
        CardEffect::SpendCorpsesSummonFootmen { max } => {
            // Boneguard Commander — corpses as 1/3 Risen Footmen
            let have = state.player(owner).corpses;
            if have == 0 {
                return;
            }
            let spend = have.min(max);
            {
                let inner = state.make_mut();
                inner.players[owner.index()].corpses -= spend;
            }
            for _ in 0..spend {
                let _ = resolve_summon(state, queue, source, owner, "CORE_RLK_061t");
            }
        }
        CardEffect::OngoingEndTurnDamage { damage } => {
            // Alexandros Mograine — mark the game-long effect
            let inner = state.make_mut();
            inner.players[owner.index()].ongoing_end_turn_damage += damage;
        }
        CardEffect::DamageAllOtherMinions { damage } => {
            // Primordial Drake — damage all minions except this one
            for player in [owner, owner.opponent()] {
                let minions: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Play, player)
                    .filter(|&e| {
                        state.world().card_type(e) == Some(CardType::Minion) && e != source
                    })
                    .collect();
                for m in minions {
                    queue.push(Event::DamageDealt {
                        source,
                        target: m,
                        amount: damage,
                    });
                }
            }
        }
        CardEffect::BuffTauntHandMinions { attack, health } => {
            // Detonation Juggernaut — Taunt minions in hand
            let hand: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter(|&e| {
                    state.world().card_type(e) == Some(CardType::Minion)
                        && state
                            .world()
                            .card_id(e)
                            .and_then(|c| crate::cards::def::card_by_id(c.0))
                            .is_some_and(|d| d.taunt)
                })
                .collect();
            for e in hand {
                let world = state.world_mut();
                let base = world.attack(e).unwrap_or(Attack(0));
                world.set_attack(e, Attack(base.0 + attack));
                let base_hp = world.health(e).unwrap_or(Health(1));
                world.set_health(e, Health(base_hp.0 + health));
            }
        }
        CardEffect::SummonRandomMinionCostEqHandSize => {
            // Astromancer — a random minion with Cost equal to the hand size
            let hand_size = state.world().zones().len(Zone::Hand, owner) as i32;
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Minion && c.cost == hand_size)
                    .collect();
            if candidates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            let _ = resolve_summon(state, queue, source, owner, candidates[idx].id);
        }
        CardEffect::ResurrectHighestCostFallen => {
            // Calia Menethil — the highest-Cost friendly minion that died
            let fallen: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Graveyard, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            let best = fallen
                .iter()
                .max_by_key(|&&e| state.world().cost(e).map_or(0, |c| c.0));
            if let Some(&best) = best {
                if let Some(def) = state
                    .world()
                    .card_id(best)
                    .and_then(|c| crate::cards::def::card_by_id(c.0))
                {
                    let _ = resolve_summon(state, queue, source, owner, def.id);
                }
            }
        }
        CardEffect::GrantDeathrattleSummonOwnCost => {
            // Ulfar — other friendly minions get a cost-based summon deathrattle
            let minions: SmallList<Entity> = collect_friendly_minions(state, owner)
                .into_iter()
                .filter(|&e| e != source)
                .collect();
            for m in minions {
                let dr = Deathrattle(CardEffect::SummonRandomMinionOfCost {
                    cost: state.world().cost(m).map_or(0, |c| c.0),
                });
                state.world_mut().set_deathrattle(m, dr);
            }
        }
        CardEffect::AddRandomPirateToHand => {
            // Sky Raider — a random Pirate
            let pirates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.race == Some(crate::core::component::Race::Pirate))
                    .collect();
            if pirates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(pirates.len());
            add_card_to_hand(state, owner, pirates[idx]);
        }
        CardEffect::NextEnemyHeroPowerCostMore { amount } => {
            // Blowtorch Saboteur — the opponent's next Hero Power costs more
            let inner = state.make_mut();
            inner.players[owner.opponent().index()].hero_power_cost_more =
                (inner.players[owner.opponent().index()].hero_power_cost_more + amount).max(0);
        }
        CardEffect::SummonRandomMinionOfCost { cost } => {
            // Maze Guide / Ulfar's deathrattle — a random minion of the cost
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Minion && c.cost == cost)
                    .collect();
            if candidates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            let _ = resolve_summon(state, queue, source, owner, candidates[idx].id);
        }
        CardEffect::SummonRandomDemonFromHandOrDeck => {
            // Archwitch Willow — a random Demon from hand and deck
            let mut demons: SmallList<Entity> = SmallList::new();
            for zone in [Zone::Hand, Zone::Deck] {
                for e in state.world().zones().iter(zone, owner) {
                    if state
                        .world()
                        .card_id(e)
                        .and_then(|c| crate::cards::def::card_by_id(c.0))
                        .is_some_and(|d| d.race == Some(crate::core::component::Race::Demon))
                    {
                        demons.push(e);
                    }
                }
            }
            if demons.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(demons.len());
            let picked = demons[idx];
            if let Some(def) = state
                .world()
                .card_id(picked)
                .and_then(|c| crate::cards::def::card_by_id(c.0))
            {
                let _ = resolve_summon(state, queue, source, owner, def.id);
            }
        }
        CardEffect::NextEnemySpellsCostMore { amount } => {
            // Cult Neophyte — the opponent's spells cost more next turn
            let inner = state.make_mut();
            inner.players[owner.opponent().index()].enemy_spell_cost_more =
                (inner.players[owner.opponent().index()].enemy_spell_cost_more + amount).max(0);
        }
        CardEffect::BuffWeaponDurabilityIfBeast { amount } => {
            // Headhunter's Hatchet — +durability while a Beast is controlled
            let has_beast = state.world().zones().iter(Zone::Play, owner).any(|e| {
                state.world().card_type(e) == Some(CardType::Minion)
                    && state
                        .world()
                        .has_race(e, crate::core::component::Race::Beast)
            });
            if !has_beast {
                return;
            }
            if let Some(weapon) = state.player(owner).weapon {
                let dur = state.world().durability(weapon).unwrap_or(Durability(0));
                state
                    .world_mut()
                    .set_durability(weapon, Durability(dur.0 + amount));
            }
        }
        CardEffect::ReturnLastTurnSpells => {
            // Krag'wa, the Frog — return last turn's spells to hand
            let spells: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Graveyard, owner)
                .filter(|&e| {
                    state
                        .world()
                        .card_id(e)
                        .and_then(|c| crate::cards::def::card_by_id(c.0))
                        .is_some_and(|d| d.card_type == CardType::Spell)
                })
                .collect();
            for s in spells {
                let world = state.world_mut();
                world.set_zone(s, Zone::Hand);
                world.zones_mut().insert(Zone::Hand, owner, s);
            }
        }
        CardEffect::DestroyMinionAndSelfDamage => {
            // Riftcleaver — destroy a minion; the hero takes its Health
            let minions: SmallList<Entity> = {
                let mut all = collect_friendly_minions(state, owner);
                all.extend(collect_enemy_minions(state, owner, None));
                all
            };
            let Some(target) = select_target(explicit_target, &minions, state.rng_mut()) else {
                return;
            };
            let hp = state.world().effective_health(target).map_or(0, |h| h.0);
            queue.push(Event::DamageDealt {
                source,
                target,
                amount: hp.max(1),
            });
            let hero = state.player(owner).hero;
            queue.push(Event::DamageDealt {
                source,
                target: hero,
                amount: hp.max(1),
            });
        }
        CardEffect::DamageSelfMinion { damage } => {
            // Injured Tol'vir — damage this minion
            queue.push(Event::DamageDealt {
                source,
                target: source,
                amount: damage,
            });
        }
        CardEffect::AddRandomOneCostCard => {
            // Dark Peddler — a random 1-Cost card (Discover simplified)
            let cards: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.cost == 1)
                    .collect();
            if cards.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(cards.len());
            add_card_to_hand(state, owner, cards[idx]);
        }
        CardEffect::BuffThreeDifferentRaces { attack, health } => {
            // Menagerie Mug — 3 friendly minions of different races
            let mut chosen: Vec<Entity> = Vec::new();
            let minions: SmallList<Entity> = collect_friendly_minions(state, owner);
            for m in &minions {
                let race = state.world().race(*m).and_then(|r| r.first()).copied();
                if race.is_some()
                    && chosen
                        .iter()
                        .all(|&c| state.world().race(c).and_then(|r| r.first()).copied() != race)
                {
                    chosen.push(*m);
                    if chosen.len() == 3 {
                        break;
                    }
                }
            }
            for m in chosen {
                let world = state.world_mut();
                let base = world.attack(m).unwrap_or(Attack(0));
                world.set_attack(m, Attack(base.0 + attack));
                let base_hp = world.health(m).unwrap_or(Health(1));
                world.set_health(m, Health(base_hp.0 + health));
            }
        }
        CardEffect::AddFiveRandomCards => {
            // Avatar of Hearthstone — the pack simplified to five random cards
            for _ in 0..5 {
                let candidates: SmallList<&'static crate::cards::def::CardDef> =
                    crate::cards::sets::ALL_CARDS
                        .iter()
                        .filter(|c| {
                            c.card_type == CardType::Minion || c.card_type == CardType::Spell
                        })
                        .collect();
                if candidates.is_empty() {
                    return;
                }
                let idx = state.rng_mut().next_usize(candidates.len());
                add_card_to_hand(state, owner, candidates[idx]);
            }
        }
        CardEffect::DiscardTwoRandomCards => {
            // Doomguard — discard two random cards
            for _ in 0..2 {
                let hand: SmallList<Entity> =
                    state.world().zones().iter(Zone::Hand, owner).collect();
                if hand.is_empty() {
                    return;
                }
                let idx = state.rng_mut().next_usize(hand.len());
                state
                    .world_mut()
                    .move_to_zone(hand[idx], Zone::Graveyard)
                    .ok();
            }
        }

        CardEffect::DamageAllMinionsIfHoldingDragon { damage } => {
            // Chillmaw — holding a Dragon deals damage to all minions
            let holds_dragon = state.world().zones().iter(Zone::Hand, owner).any(|e| {
                state
                    .world()
                    .card_id(e)
                    .and_then(|c| crate::cards::def::card_by_id(c.0))
                    .is_some_and(|d| d.race == Some(crate::core::component::Race::Dragon))
            });
            if !holds_dragon {
                return;
            }
            for player in [owner, owner.opponent()] {
                let minions: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Play, player)
                    .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                    .collect();
                for m in minions {
                    queue.push(Event::DamageDealt {
                        source,
                        target: m,
                        amount: damage,
                    });
                }
            }
        }
        CardEffect::DamageAllEnemiesByAttack => {
            // Augmented Porcupine — Attack damage split among all enemies
            let atk = state
                .world()
                .effective_attack(source)
                .unwrap_or(Attack(0))
                .0;
            if atk <= 0 {
                return;
            }
            let enemies = collect_enemy_characters(state, owner, Some(source));
            if enemies.is_empty() {
                return;
            }
            for _ in 0..atk {
                let idx = state.rng_mut().next_usize(enemies.len());
                queue.push(Event::DamageDealt {
                    source,
                    target: enemies[idx],
                    amount: 1,
                });
            }
        }
        CardEffect::ReturnRandomFriendlyAndReduceCost { amount } => {
            // Waggle Pick — a random friendly minion returns, costing less
            let minions: SmallList<Entity> = collect_friendly_minions(state, owner);
            let Some(target) = select_target(None, &minions, state.rng_mut()) else {
                return;
            };
            state.world_mut().move_to_zone(target, Zone::Hand).ok();
            if let Some(card) = state
                .world()
                .card_id(target)
                .and_then(|c| crate::cards::def::card_by_id(c.0))
            {
                let cur = state
                    .world()
                    .effective_cost(target)
                    .unwrap_or(Cost(card.cost));
                state
                    .world_mut()
                    .set_cost(target, Cost((cur.0 - amount).max(0)));
            }
        }
        CardEffect::GrantAttackToRandomFriendly => {
            // Fiendish Servant — its Attack to a random friendly minion
            let atk = state
                .world()
                .effective_attack(source)
                .unwrap_or(Attack(0))
                .0;
            let minions: SmallList<Entity> = collect_friendly_minions(state, owner);
            let Some(target) = select_target(None, &minions, state.rng_mut()) else {
                return;
            };
            let world = state.world_mut();
            let base = world.attack(target).unwrap_or(Attack(0));
            world.set_attack(target, Attack(base.0 + atk));
        }
        CardEffect::SummonRandomLegendaryMinion => {
            // Sneed's Old Shredder — a random legendary minion
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| {
                        c.card_type == CardType::Minion
                            && crate::cards::sets::LEGENDARY_CLASSIC
                                .iter()
                                .any(|l| l.id == c.id)
                    })
                    .collect();
            if candidates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            let _ = resolve_summon(state, queue, source, owner, candidates[idx].id);
        }
        CardEffect::ResurrectWeaponKilled => {
            // Frostmourne — approximated by the generic resurrect
            let fallen: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Graveyard, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            if let Some(&best) = fallen
                .iter()
                .max_by_key(|&&e| state.world().cost(e).map_or(0, |c| c.0))
            {
                if let Some(def) = state
                    .world()
                    .card_id(best)
                    .and_then(|c| crate::cards::def::card_by_id(c.0))
                {
                    let _ = resolve_summon(state, queue, source, owner, def.id);
                }
            }
        }
        CardEffect::DestroyRandomEnemyMinion => {
            // Pressure Plate — destroy a random enemy minion
            let minions: SmallList<Entity> = collect_enemy_minions(state, owner, None);
            let Some(target) = select_target(None, &minions, state.rng_mut()) else {
                return;
            };
            let hp = state.world().effective_health(target).map_or(0, |h| h.0);
            queue.push(Event::DamageDealt {
                source,
                target,
                amount: hp.max(1),
            });
        }
        CardEffect::SummonOasisWaterElemental => {
            // Oasis Ally — a 3/6 Water Elemental
            let _ = resolve_summon(state, queue, source, owner, "CORE_BAR_812t");
        }
        CardEffect::SummonRandomCostAndFreeze { cost } => {
            // Glaciate — a random minion of the cost, summoned and frozen
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Minion && c.cost == cost)
                    .collect();
            if candidates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            if let Some(e) = resolve_summon(state, queue, source, owner, candidates[idx].id) {
                state.world_mut().set_freeze(e, Freeze);
            }
        }
        CardEffect::DamageAndAddRandomSpell { damage, target } => {
            // Runed Orb — damage and a random spell to hand
            resolve_deal_damage(state, queue, source, owner, damage, target, explicit_target);
            let spells: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Spell)
                    .collect();
            if spells.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(spells.len());
            add_card_to_hand(state, owner, spells[idx]);
        }
        CardEffect::FreezeAndSummonElementals => {
            // Deep Freeze — freeze an enemy and summon two 3/6 Water Elementals
            let chars: SmallList<Entity> = collect_enemy_characters(state, owner, None);
            if let Some(target) = select_target(explicit_target, &chars, state.rng_mut()) {
                state.world_mut().set_freeze(target, Freeze);
            }
            for _ in 0..2 {
                let _ = resolve_summon(state, queue, source, owner, "CORE_BT_072t");
            }
        }
        CardEffect::AddRandomTauntBuffed => {
            // I Know a Guy — a random Taunt minion, buffed +1/+2
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Minion && c.taunt)
                    .collect();
            if candidates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            let def = candidates[idx];
            let e = crate::cards::spawn_card_from_def(state.world_mut(), owner, def);
            let world = state.world_mut();
            let base = world.attack(e).unwrap_or(Attack(0));
            world.set_attack(e, Attack(base.0 + 1));
            let base_hp = world.health(e).unwrap_or(Health(1));
            world.set_health(e, Health(base_hp.0 + 2));
            state.world_mut().set_zone(e, Zone::Hand);
            state.world_mut().zones_mut().insert(Zone::Hand, owner, e);
        }
        CardEffect::DamageAndFreeze { damage, target } => {
            // Frostbolt — damage and freeze
            if let Some(t) =
                resolve_deal_damage(state, queue, source, owner, damage, target, explicit_target)
            {
                if state.world().is_alive(t) {
                    state.world_mut().set_freeze(t, Freeze);
                }
            }
        }
        CardEffect::DamageAllEnemyMinionsAndFreeze { damage } => {
            // Blizzard — damage all enemy minions and freeze them
            let enemies = collect_all_enemy_minions(state, owner);
            for m in &enemies {
                queue.push(Event::DamageDealt {
                    source,
                    target: *m,
                    amount: damage,
                });
                state.world_mut().set_freeze(*m, Freeze);
            }
        }
        CardEffect::AddRandomBattlecryMinion => {
            // Blazing Invocation — a random Battlecry minion
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Minion && c.battlecry.is_some())
                    .collect();
            if candidates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            add_card_to_hand(state, owner, candidates[idx]);
        }
        CardEffect::AddRandomOutcastCardNextCheaper => {
            // Illidari Studies — a random Outcast card to hand (Discover
            // simplified to random generation), next Outcast costs (1) less
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Minion || c.card_type == CardType::Spell)
                    .filter(|c| crate::cards::def::has_outcast(c))
                    .collect();
            if candidates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            add_card_to_hand(state, owner, candidates[idx]);
            state.make_mut().players[owner.index()].next_outcast_discount = 1;
        }
        CardEffect::CopyEnemyDeckCardOnSelfAttack => {
            // Shaku — copy a random enemy deck card to hand, only when THIS
            // minion is the one attacking
            if event_subject != Some(source) {
                return;
            }
            let enemy = owner.opponent();
            let deck: SmallList<Entity> = state.world().zones().iter(Zone::Deck, enemy).collect();
            if deck.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(deck.len());
            copy_card_to_hand(state, deck[idx], owner);
        }
        CardEffect::AddRandomSpellToOpponentDeckTop => {
            // Merch Seller — a random spell (from the full card database)
            // lands on top of the opponent's deck
            let spells: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Spell)
                    .collect();
            if spells.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(spells.len());
            let spell = spells[idx];
            let enemy = owner.opponent();
            let e = crate::cards::spawn_card_from_def(state.world_mut(), enemy, spell);
            state.world_mut().set_zone(e, Zone::Deck);
            state
                .world_mut()
                .zones_mut()
                .insert_at(Zone::Deck, enemy, e, 0);
        }
        CardEffect::SummonStatueTrio => {
            // Immortalized in Stone — the 4/8, 2/4 and 1/2 statues with Taunt
            for statue in ["CORE_TSC_076a", "CORE_TSC_076b", "CORE_TSC_076c"] {
                let _ = resolve_summon(state, queue, source, owner, statue);
            }
        }
        CardEffect::GainStatsTauntAndDeathrattle {
            attack,
            health,
            card_id,
        } => {
            // Spikeridged Steed — buff + Taunt + deathrattle summon
            let minions: SmallList<Entity> = {
                let mut all = collect_friendly_minions(state, owner);
                all.extend(collect_enemy_minions(state, owner, None));
                all
            };
            let Some(target) = select_target(explicit_target, &minions, state.rng_mut()) else {
                return;
            };
            {
                let world = state.world_mut();
                let base = world.attack(target).unwrap_or(Attack(0));
                world.set_attack(target, Attack(base.0 + attack));
                let base_hp = world.health(target).unwrap_or(Health(1));
                world.set_health(target, Health(base_hp.0 + health));
                world.set_taunt(target, Taunt);
            }
            let dr = Deathrattle(CardEffect::SummonMinion { card_id });
            state.world_mut().set_deathrattle(target, dr);
        }
        CardEffect::DestroyAllMinionsAttackGE { attack } => {
            // Shadow Word: Ruin — destroy all minions (both sides) with at
            // least this Attack
            let doomed: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Play, owner)
                .filter(|&e| {
                    state.world().card_type(e) == Some(CardType::Minion)
                        && state
                            .world()
                            .effective_attack(e)
                            .is_some_and(|a| a.0 >= attack)
                })
                .chain(
                    state
                        .world()
                        .zones()
                        .iter(Zone::Play, owner.opponent())
                        .filter(|&e| {
                            state.world().card_type(e) == Some(CardType::Minion)
                                && state
                                    .world()
                                    .effective_attack(e)
                                    .is_some_and(|a| a.0 >= attack)
                        }),
                )
                .collect();
            for m in doomed {
                queue.push(Event::DamageDealt {
                    source,
                    target: m,
                    amount: state.world().effective_health(m).map_or(0, |h| h.0),
                });
            }
        }
        CardEffect::ResurrectDiedMinion => {
            // Secret-context effect — resolved by the secret system with the
            // death event (Redemption)
        }
        CardEffect::PreventFatalDamageAndImmune => {
            // Secret-context effect — resolved by the secret system (Ice Block)
        }
        CardEffect::RestoreDamagedFriendly { amount } => {
            // Lightwell — restore to a random damaged friendly character
            let damaged: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Play, owner)
                .filter(|&e| {
                    state.world().damage(e).is_some_and(|d| d.0 > 0)
                        && (state.world().card_type(e) == Some(CardType::Minion)
                            || state.world().card_type(e) == Some(CardType::Hero))
                })
                .collect();
            if let Some(target) = select_target(None, &damaged, state.rng_mut()) {
                let dmg = state.world().damage(target).map_or(0, |d| d.0);
                let new_dmg = (dmg - amount).max(0);
                if new_dmg > 0 {
                    state.world_mut().set_damage(target, Damage(new_dmg));
                } else {
                    state.world_mut().remove_damage(target);
                }
                fire_healed_trigger(state, queue, target);
            }
        }
        CardEffect::GainStatsPerHandCard {
            attack,
            health_per_card,
        } => {
            let hand = state.world().zones().len(Zone::Hand, owner) as i32;
            state.world_mut().add_enchantment(
                source,
                Enchantment {
                    attack,
                    health: health_per_card * hand,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
        }
        CardEffect::GainStatsPerFriendlyMinion {
            attack,
            health_per_minion,
        } => {
            // Frostwolf Warlord (W16): +1/+1 per OTHER friendly minion —
            // the source itself is excluded.
            let others = collect_friendly_minions(state, owner)
                .into_iter()
                .filter(|&e| e != source)
                .count() as i32;
            state.world_mut().add_enchantment(
                source,
                Enchantment {
                    attack: attack * others,
                    health: health_per_minion * others,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
        }
        CardEffect::DealDamageRandomly {
            amount,
            count,
            target,
        } => {
            resolve_deal_damage_randomly(state, queue, source, owner, amount, count, target);
        }
        CardEffect::MortalStrike {
            damage,
            boosted,
            threshold,
        } => {
            let hero = state.player(owner).hero;
            let low = state
                .world()
                .effective_health(hero)
                .is_some_and(|h| h.0 <= threshold);
            let amount = if low { boosted } else { damage };
            resolve_deal_damage(
                state,
                queue,
                source,
                owner,
                amount,
                EffectTarget::AnyEnemy,
                explicit_target,
            );
        }
        CardEffect::DrawPerDamagedFriendlyCharacter => {
            let damaged = state
                .world()
                .zones()
                .iter(Zone::Play, owner)
                .filter(|&e| {
                    state.world().damage(e).is_some_and(|d| d.0 > 0)
                        && (state.world().card_type(e) == Some(CardType::Minion)
                            || state.world().card_type(e) == Some(CardType::Hero))
                })
                .count() as u32;
            for _ in 0..damaged {
                draw_card(state, queue, owner);
            }
        }
        CardEffect::GainStatsIfOwnSecret { attack, health } => {
            let owns_secret = state
                .world()
                .zones()
                .iter(Zone::SetAside, owner)
                .any(|e| state.world().secret(e).is_some());
            if owns_secret {
                state.world_mut().add_enchantment(
                    source,
                    Enchantment {
                        attack,
                        health,
                        cost: 0,
                        expiry: EnchantmentExpiry::Permanent,
                    },
                );
            }
        }
        CardEffect::AbsorbDivineShields {
            attack_per_shield,
            health_per_shield,
        } => {
            let mut shields = 0;
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
                if state.world().divine_shield(*m).is_some() {
                    state.world_mut().remove_divine_shield(*m);
                    shields += 1;
                }
            }
            if shields > 0 {
                state.world_mut().add_enchantment(
                    source,
                    Enchantment {
                        attack: attack_per_shield * shields,
                        health: health_per_shield * shields,
                        cost: 0,
                        expiry: EnchantmentExpiry::Permanent,
                    },
                );
            }
        }
    }
}

/// Draws the top card of the ordered deck into hand (roadmap G7 — the deck is
/// shuffled at game start, so the top draw is the random pick), without
/// enqueueing a CardDrawn event. Returns the drawn card, or `None` when the
/// deck is empty — in which case the player suffers **fatigue** (official
/// rule, docs/fatigue-roadmap.md): the drawing hero takes damage equal to the
/// player's 1-based fatigue counter, the counter increments by 1, and no
/// CardDrawn is emitted ("whenever you draw a card" triggers must not fire).
/// The damage goes through the unified `DamageDealt` pipeline (armor absorbs,
/// lethal ends the game, lethal-prevention secrets fire).
pub(crate) fn draw_card_no_queue(
    state: &mut GameState,
    queue: &mut EventQueue,
    player: PlayerId,
) -> Option<Entity> {
    let Some(card) = state.world().zones().iter(Zone::Deck, player).next() else {
        // Empty deck: fatigue (official rule) — the counter IS the damage for
        // this attempt, then it increments
        let inner = state.make_mut();
        let p = &mut inner.players[player.index()];
        let hero = p.hero;
        let amount = p.fatigue as i32;
        p.fatigue += 1;
        queue.push(Event::DamageDealt {
            source: hero,
            target: hero,
            amount,
        });
        return None;
    };
    // CardDrawn triggers (Core Set W3b — Keymaster Alabaster, Eredar
    // Deceptor) fire for the drawing player with the drawn card as subject
    {
        let owner = state.world().player(card);
        if let Some(owner) = owner {
            crate::engine::rules::fire_triggers(
                state,
                queue,
                TriggerEvent::CardDrawn,
                owner,
                Some(card),
                None,
            );
        }
    }
    // Hand-size cap (F-A11): a card drawn past the 10-card limit is burned —
    // destroyed (sent to the graveyard), while the draw still counts for deck
    // depletion and the CardDrawn event still fires (the caller pushes it).
    let world = state.world_mut();
    let hand_full = world.zones().len(Zone::Hand, player) >= MAX_HAND_SIZE;
    let zone = if hand_full {
        Zone::Graveyard
    } else {
        Zone::Hand
    };
    world
        .move_to_zone(card, zone)
        .expect("card should be movable to hand");
    Some(card)
}

/// Draws the top card of the deck without a queue — used only at
/// opening/mulligan time, where the deck is provably non-empty (fatigue is
/// physically impossible, official rule 5). The non-empty invariant is
/// debug-asserted; the hot opening path keeps its no-queue shape.
pub(crate) fn draw_top_card_no_queue(state: &mut GameState, player: PlayerId) -> Option<Entity> {
    let card = state.world().zones().iter(Zone::Deck, player).next();
    debug_assert!(
        card.is_some(),
        "opening/mulligan draws must hit a non-empty deck"
    );
    let card = card?;
    state
        .world_mut()
        .move_to_zone(card, Zone::Hand)
        .expect("card should be movable to hand");
    Some(card)
}

fn draw_card_with_reduction(
    state: &mut GameState,
    queue: &mut EventQueue,
    player: PlayerId,
    cost_reduction: i32,
) {
    let Some(card) = draw_card_no_queue(state, queue, player) else {
        return;
    };

    // Cost reduction (kept above the base cost) — a cost enchantment (roadmap G4)
    if cost_reduction > 0 {
        let world = state.world_mut();
        let cur = world.effective_cost(card).unwrap_or(Cost(0));
        world.set_cost(card, Cost((cur.0 - cost_reduction).max(0)));
    }

    queue.push(Event::CardDrawn { player, card });
}

/// Draws a random card from the deck into hand.
pub fn draw_card(state: &mut GameState, queue: &mut EventQueue, player: PlayerId) {
    draw_card_with_reduction(state, queue, player, 0);
}

/// Draws cards of the given race from the deck (Sense Demons — draw two
/// Demons). Unlike a top-deck draw, the deck is scanned for matching cards —
/// the first `count` matches in deck order (the deck is shuffled at game
/// start, so the pick order is the deterministic shuffle order).
///
/// No fatigue applies here (official rule 5): a scan draws what exists and
/// nothing more — a "draw N minions of a race" that finds 0 draws 0 and deals
/// 0 damage.
fn resolve_draw_by_race(
    state: &mut GameState,
    queue: &mut EventQueue,
    player: PlayerId,
    count: u32,
    race: crate::core::component::Race,
) {
    let mut drawn = 0;
    // Collect matches first (the deck zone is borrowed); move them after.
    let matches: SmallList<Entity> = state
        .world()
        .zones()
        .iter(Zone::Deck, player)
        .filter(|&e| state.world().has_race(e, race))
        .take(count as usize)
        .collect();
    for card in matches {
        if state.world_mut().move_to_zone(card, Zone::Hand).is_ok() {
            queue.push(Event::CardDrawn { player, card });
            drawn += 1;
            if drawn >= count {
                break;
            }
        }
    }
}

/// Demonfire — deal damage to a minion; if the target is a friendly Demon,
/// buff it instead (WARLOCK_021: 2 damage, or +2/+2 to a friendly Demon).
// 8 parameters (state, queue, source, owner, damage, attack_bonus, health_bonus, explicit) — resolver convention style.
#[allow(clippy::too_many_arguments)]
fn resolve_demonfire(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    damage: i32,
    attack_bonus: i32,
    health_bonus: i32,
    explicit: Option<Entity>,
) {
    // Any minion (either side) is a legal target
    let mut minions = collect_friendly_minions(state, owner);
    minions.extend(collect_all_enemy_minions(state, owner));
    let Some(target) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    let friendly_demon = state.world().player(target) == Some(owner)
        && state
            .world()
            .has_race(target, crate::core::component::Race::Demon);
    if friendly_demon {
        state.world_mut().add_enchantment(
            target,
            Enchantment {
                attack: attack_bonus,
                health: health_bonus,
                cost: 0,
                expiry: EnchantmentExpiry::Permanent,
            },
        );
    } else {
        queue.push(Event::DamageDealt {
            source,
            target,
            amount: damage,
        });
    }
}

/// Deals damage to the target set and enqueues DamageDealt events. Returns
/// the single-picked target entity when the effect picks exactly one, so
/// composite effects (DamageAndGainAttack — Cruel Taskmaster, Inner Rage)
/// can buff the same target that took the damage; AOE and no-op resolutions
/// return None.
fn resolve_deal_damage(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    amount: i32,
    target: EffectTarget,
    explicit: Option<Entity>,
) -> Option<Entity> {
    let enemies = match target {
        EffectTarget::AnyEnemy => collect_enemy_characters(state, owner, Some(source)),
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner, Some(source)),
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
            return None;
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
            return None;
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
            return None;
        }
        EffectTarget::Self_ => {
            queue.push(Event::DamageDealt {
                source,
                target: source,
                amount,
            });
            return None;
        }
        // Battlecry-target kinds (battlecry-target-debt roadmap W13): a single
        // character (hero + minion) on either side, or either hero. Friendly
        // characters keep the friendly convention (stealth does not filter
        // friendly targets); the enemy side keeps the stealth filter.
        EffectTarget::AnyCharacter => {
            let mut all = collect_friendly_characters(state, owner, Some(source));
            all.extend(collect_enemy_characters(state, owner, Some(source)));
            all
        }
        EffectTarget::AnyHero => {
            let hero = state.player(owner).hero;
            let enemy_hero = state.player(owner.opponent()).hero;
            [hero, enemy_hero].into_iter().collect()
        }
        // Cruel Taskmaster (W14) — damage a chosen friendly minion.
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
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
            return None;
        }
        EffectTarget::AllCharacters
        | EffectTarget::AllFriendlyCharacters
        | EffectTarget::FriendlyHero
        | EffectTarget::DamagedEnemyMinion
        | EffectTarget::OtherFriendlyMinion
        | EffectTarget::TauntEnemyMinion
        | EffectTarget::EventSubject
        | EffectTarget::FriendlyRace(_)
        | EffectTarget::AllOtherFriendlyRace(_)
        | EffectTarget::AnyRace(_)
        | EffectTarget::EnemyMinionAttackLE(_)
        | EffectTarget::AnyMinionAttackGE(_)
        | EffectTarget::EnemyMinionAttackGE(_)
        | EffectTarget::DamagedFriendlyMinion
        | EffectTarget::DamagedMinion => {
            return None;
        }
        EffectTarget::AnyMinion => {
            // Any minion on either side (Quick Shot — Core Set W2): an
            // explicit target is required; without one the effect fizzles
            // (G9), matching the engine's other any-minion damage effects.
            let t = explicit?;
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_enemy_minions(state, owner, None));
            if !all.contains(&t) {
                return None;
            }
            [t].into_iter().collect()
        }
    };

    // Explicit target first, otherwise random selection
    let target_entity = select_target(explicit, &enemies, state.rng_mut())?;

    queue.push(Event::DamageDealt {
        source,
        target: target_entity,
        amount,
    });
    Some(target_entity)
}

/// Tracking (W10): surfaces a Discover choice over the top 3 cards of the
/// owner's deck. The choice carries `discard_rest` — ChoiceResolved moves the
/// picked card's EXISTING entity to hand and discards the other two.
fn resolve_discover_deck_top3(state: &mut GameState, source: Entity, owner: PlayerId) {
    let deck: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Deck, owner)
        .take(3)
        .collect();
    if deck.is_empty() {
        return;
    }
    let options: Vec<String> = deck
        .iter()
        .map(|&e| {
            state
                .world()
                .card_id(e)
                .and_then(|c| crate::cards::def::card_by_id(c.0))
                .map_or_else(|| String::from("?"), |c| format!("{} ({})", c.name, c.id))
        })
        .collect();
    let pool_ids = deck
        .iter()
        .map(|&e| {
            state
                .world()
                .card_id(e)
                .map_or_else(String::new, |c| c.0.to_string())
        })
        .collect();
    state.set_pending_choice_discard_rest(
        crate::core::state::ChoiceKind::Discover,
        source,
        options,
        pool_ids,
    );
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
    resolve_summon_doubled(state, queue, _source, owner, card_id, false)
}

/// The summon worker. `doubled` marks a summon produced by Khadgar's
/// doubling — the doubling does not recurse (a doubled summon is not
/// doubled again, real Hearthstone semantics).
fn resolve_summon_doubled(
    state: &mut GameState,
    queue: &mut EventQueue,
    _source: Entity,
    owner: PlayerId,
    card_id: &str,
    doubled: bool,
) -> Option<Entity> {
    // Look up the card definition
    let card_def = crate::cards::def::card_by_id(card_id)?;

    // Khadgar (Core Set W3a): a friendly Khadgar on the board doubles the
    // summon — once. Doubled summons (doubled=true) are not doubled again.
    let khadgar = !doubled
        && state.world().zones().iter(Zone::Play, owner).any(|e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| c.0 == "CORE_DAL_575")
        });

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
        // Race / tribe (fidelity-debt W1)
        if let Some(race) = card_def.race {
            world.set_race(e, race);
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
                    race: None,
                    max_attack: None,
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
                    race: None,
                    max_attack: None,
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
                    race: None,
                    max_attack: None,
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
                    race: None,
                    max_attack: None,
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
                    race: None,
                    max_attack: None,
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

    // Enqueue the MinionSummoned event (triggers battlecry and similar effects).
    // Effect-summoned minions carry no explicit battlecry target (M1).
    queue.push(Event::MinionSummoned {
        player: owner,
        minion: e,
        target: None,
    });
    if khadgar {
        // Khadgar doubles: summon a second copy (fresh entity; the board
        // limit applies to it too)
        let _ = resolve_summon_doubled(state, queue, _source, owner, card_id, true);
    }
    Some(e)
}

/// Removes durability from the opponent's weapon; a weapon at 0 durability
/// is destroyed (Bloodsail Corsair).
fn resolve_remove_weapon_durability(
    state: &mut GameState,
    queue: &mut EventQueue,
    owner: PlayerId,
    amount: i32,
) {
    let enemy = owner.opponent();
    let Some(weapon) = state.player(enemy).weapon else {
        return;
    };
    let dur = state.world().durability(weapon).unwrap_or(Durability(0)).0;
    let new_dur = dur - amount;
    if new_dur <= 0 {
        let inner = state.make_mut();
        inner.players[enemy.index()].weapon = None;
        queue.push(Event::WeaponDestroyed {
            player: enemy,
            weapon,
        });
    } else {
        state
            .world_mut()
            .set_durability(weapon, Durability(new_dur));
    }
}

/// Swaps a minion's Attack and Health (Crazed Alchemist) — expressed as
/// enchantment deltas so the swap survives and stacks on the base stats.
fn resolve_swap_attack_health(
    state: &mut GameState,
    owner: PlayerId,
    _target: EffectTarget,
    explicit: Option<Entity>,
) {
    let mut minions = collect_friendly_minions(state, owner);
    minions.extend(collect_all_enemy_minions(state, owner));
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    let Some(atk) = state.world().effective_attack(m) else {
        return;
    };
    let Some(hp) = state.world().effective_health(m) else {
        return;
    };
    let base_atk = state.world().attack(m).unwrap_or(Attack(0)).0;
    let base_hp = state.world().health(m).unwrap_or(Health(0)).0;
    let world = state.world_mut();
    world.remove_enchantments(m);
    world.remove_damage(m);
    world.add_enchantment(
        m,
        Enchantment {
            attack: hp.0 - base_atk,
            health: atk.0 - base_hp,
            cost: 0,
            expiry: EnchantmentExpiry::Permanent,
        },
    );
}

/// Freezes a random enemy minion and its neighbors (Cone of Cold).
fn resolve_freeze_adjacent(state: &mut GameState, owner: PlayerId) {
    let minions = collect_enemy_minions(state, owner, None);
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let target = minions[idx];
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
    state.world_mut().set_freeze(target, Freeze);
    if let Some(&left) = board.get(pos.wrapping_sub(1)) {
        state.world_mut().set_freeze(left, Freeze);
    }
    if let Some(&right) = board.get(pos + 1) {
        state.world_mut().set_freeze(right, Freeze);
    }
}

/// Gives the source's adjacent minions Taunt (Sunfury Protector).
fn resolve_grant_adjacent_taunt(state: &mut GameState, source: Entity, owner: PlayerId) {
    let board: SmallList<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, owner)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect();
    let Some(pos) = board.iter().position(|&e| e == source) else {
        return;
    };
    for neighbor in [pos.wrapping_sub(1), pos + 1] {
        if let Some(&n) = board.get(neighbor) {
            state
                .world_mut()
                .set_taunt(n, crate::core::component::Taunt);
        }
    }
}

/// Gives the source's adjacent minions stats and Divine Shield (Defender of
/// Argus — +1/+1 and Divine Shield to adjacent minions).
fn resolve_grant_adjacent_stats_and_divine_shield(
    state: &mut GameState,
    source: Entity,
    owner: PlayerId,
    attack: i32,
    health: i32,
) {
    let board: SmallList<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, owner)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect();
    let Some(pos) = board.iter().position(|&e| e == source) else {
        return;
    };
    let buff = Enchantment {
        attack,
        health,
        cost: 0,
        expiry: EnchantmentExpiry::Permanent,
    };
    for neighbor in [pos.wrapping_sub(1), pos + 1] {
        if let Some(&n) = board.get(neighbor) {
            let world = state.world_mut();
            world.add_enchantment(n, buff);
            world.set_divine_shield(n, crate::core::component::DivineShield);
        }
    }
}

/// Deathwing's battlecry: discard your whole hand, then destroy all other
/// minions (the source survives).
fn resolve_destroy_all_other_minions_and_discard_hand(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
) {
    resolve_discard_hand(state, owner);
    let mut all = collect_friendly_minions(state, owner);
    all.extend(collect_all_enemy_minions(state, owner));
    for &m in &all {
        if m == source {
            continue;
        }
        let hp = state.world().effective_health(m).unwrap_or(Health(1));
        queue.push(Event::DamageDealt {
            source: m,
            target: m,
            amount: hp.0.max(1),
        });
    }
}

/// Gives the source's adjacent minions Spell Damage (Ancient Mage).
fn resolve_grant_adjacent_spell_damage(
    state: &mut GameState,
    source: Entity,
    owner: PlayerId,
    amount: i32,
) {
    let board: SmallList<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, owner)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect();
    let Some(pos) = board.iter().position(|&e| e == source) else {
        return;
    };
    for neighbor in [pos.wrapping_sub(1), pos + 1] {
        if let Some(&n) = board.get(neighbor) {
            let cur = state.world().spell_damage(n).map_or(0, |s| s.0);
            state
                .world_mut()
                .set_spell_damage(n, crate::core::component::SpellDamage(cur + amount));
        }
    }
}

/// Restores a minion to full Health and gives it Taunt (Ancestral Healing).
fn resolve_full_heal_and_taunt(
    state: &mut GameState,
    owner: PlayerId,
    _target: EffectTarget,
    explicit: Option<Entity>,
) {
    let mut minions = collect_friendly_minions(state, owner);
    minions.extend(collect_all_enemy_minions(state, owner));
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    state.world_mut().remove_damage(m);
    state
        .world_mut()
        .set_taunt(m, crate::core::component::Taunt);
}

/// Alarm-o-Bot — swap this minion with a random minion in your hand (the
/// swapped-in minion lands at the bot's position with summoning sickness).
fn resolve_swap_with_hand_minion(state: &mut GameState, source: Entity, owner: PlayerId) {
    let hand_minions: SmallList<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, owner)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect();
    if hand_minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(hand_minions.len());
    let incoming = hand_minions[idx];
    let board: SmallList<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, owner)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect();
    let Some(pos) = board.iter().position(|&e| e == source) else {
        return;
    };
    // The bot goes back to hand; the hand minion takes its position
    let _ = state.world_mut().move_to_zone(source, Zone::Hand);
    let _ = state.world_mut().move_to_zone(incoming, Zone::Play);
    let world = state.world_mut();
    world.zones_mut().remove(Zone::Play, owner, incoming);
    world
        .zones_mut()
        .insert_at(Zone::Play, owner, incoming, pos);
    // Summoning sickness unless it has Charge
    if world.effective_charge(incoming) {
        world.set_attacks_used(incoming, crate::core::component::AttacksUsed(0));
    } else {
        world.set_attacks_used(incoming, crate::core::component::AttacksUsed(1));
    }
}

/// Destroys up to `limit` random enemy Secrets (SI:7 Infiltrator — one;
/// Eater of Secrets / Flare — all). Secrets live in the SetAside zone.
fn resolve_destroy_enemy_secrets(state: &mut GameState, limit: i32) {
    let enemy = state.active_player().opponent();
    let secrets: SmallList<Entity> = state
        .world()
        .zones()
        .iter(Zone::SetAside, enemy)
        .filter(|&e| state.world().secret(e).is_some())
        .collect();
    if secrets.is_empty() {
        return;
    }
    // Shuffle-pick: remove random secrets until the limit is reached
    let mut remaining = secrets;
    let mut destroyed = 0;
    while !remaining.is_empty() && destroyed < limit {
        let idx = state.rng_mut().next_usize(remaining.len());
        let secret = remaining.remove(idx);
        let _ = state.world_mut().move_to_zone(secret, Zone::Graveyard);
        destroyed += 1;
    }
}

/// Blessing of Wisdom — attach "Whenever this minion attacks, draw a card"
/// to a random minion (any side, in self-play).
fn resolve_attach_attack_draw(
    state: &mut GameState,
    owner: PlayerId,
    count: u32,
    explicit: Option<Entity>,
) {
    let mut minions = collect_friendly_minions(state, owner);
    minions.extend(collect_all_enemy_minions(state, owner));
    let Some(target) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    state.world_mut().set_trigger(
        target,
        Trigger {
            event: TriggerEvent::Attacked,
            timing: TriggerTiming::Whenever,
            effect: CardEffect::DrawCard { count },
            race: None,
            max_attack: None,
        },
    );
}

/// Buffs a target AND grants it Taunt — one target selection shared by both
/// parts (Houndmaster: +2/+2 and Taunt to a friendly Beast). Reuses the
/// single-target selection from `resolve_gain_stats` for the same target
/// variants (FriendlyRace etc.).
// 8 parameters (state, source, owner, attack, health, target, explicit, subject) — resolver convention style.
#[allow(clippy::too_many_arguments)]
fn resolve_gain_stats_and_taunt(
    state: &mut GameState,
    source: Entity,
    owner: PlayerId,
    attack: i32,
    health: i32,
    target: EffectTarget,
    explicit: Option<Entity>,
    _subject: Option<Entity>,
) {
    let buff = Enchantment {
        attack,
        health,
        cost: 0,
        expiry: EnchantmentExpiry::Permanent,
    };
    let minions: SmallList<Entity> = match target {
        EffectTarget::FriendlyRace(race) => collect_friendly_minions(state, owner)
            .into_iter()
            .filter(|&e| state.world().has_race(e, race))
            .collect(),
        // Choose One taunt branches (Druid of the Claw, Ancient of War) — the
        // minion itself
        EffectTarget::Self_ => {
            let mut list = SmallList::new();
            list.push(source);
            list
        }
        _ => return,
    };
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    let world = state.world_mut();
    world.add_enchantment(m, buff);
    world.set_taunt(m, crate::core::component::Taunt);
}

/// Destroys a minion of the target scope, then grants the source fixed stats
/// (Hungry Crab — destroy a Murloc and gain +2/+2).
// 8 parameters (state, queue, source, owner, attack, health, target, explicit) — resolver convention style.
#[allow(clippy::too_many_arguments)]
fn resolve_destroy_and_gain_stats(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    attack: i32,
    health: i32,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        EffectTarget::AnyRace(race) => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            all.into_iter()
                .filter(|&e| state.world().has_race(e, race))
                .collect()
        }
        _ => return,
    };
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    let hp = state.world().effective_health(m).unwrap_or(Health(1));
    queue.push(Event::DamageDealt {
        source: m,
        target: m,
        amount: hp.0.max(1),
    });
    state.world_mut().add_enchantment(
        source,
        Enchantment {
            attack,
            health,
            cost: 0,
            expiry: EnchantmentExpiry::Permanent,
        },
    );
}

/// This-turn buff (Mana Addict): same target selection as `resolve_gain_stats`
/// but the enchantment expires at the end of the turn.
fn resolve_gain_stats_this_turn(
    state: &mut GameState,
    source: Entity,
    owner: PlayerId,
    attack: i32,
    health: i32,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let buff = Enchantment {
        attack,
        health,
        cost: 0,
        expiry: EnchantmentExpiry::UntilEndOfTurn,
    };
    let minions: SmallList<Entity> = match target {
        EffectTarget::Self_ => {
            state.world_mut().add_enchantment(source, buff);
            return;
        }
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        _ => return,
    };
    if let Some(m) = select_target(explicit, &minions, state.rng_mut()) {
        state.world_mut().add_enchantment(m, buff);
    }
}

/// Resolves a buff effect.
// 9 parameters (state, queue, source, owner, attack, health, target, explicit, subject) — resolver convention style.
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
    subject: Option<Entity>,
) {
    // Buffs attach enchantments (roadmap G4) instead of writing the base stats
    let buff = Enchantment {
        attack,
        health,
        cost: 0,
        expiry: EnchantmentExpiry::Permanent,
    };
    match target {
        EffectTarget::Self_ => {
            state.world_mut().add_enchantment(source, buff);
        }
        EffectTarget::AllFriendlyMinions => {
            let minions: SmallList<Entity> = collect_friendly_minions(state, owner);
            let world = state.world_mut();
            for minion in &minions {
                world.add_enchantment(*minion, buff);
            }
        }
        EffectTarget::FriendlyMinion => {
            let minions = collect_friendly_minions(state, owner);
            let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
                return;
            };
            state.world_mut().add_enchantment(m, buff);
        }
        // Friendly minions other than the source (Young Priestess — "another")
        EffectTarget::OtherFriendlyMinion => {
            let minions: SmallList<Entity> = collect_friendly_minions(state, owner)
                .into_iter()
                .filter(|&e| e != source)
                .collect();
            let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
                return;
            };
            state.world_mut().add_enchantment(m, buff);
        }
        // A random friendly minion of the given race (Houndmaster — a Beast)
        EffectTarget::FriendlyRace(race) => {
            let minions: SmallList<Entity> = collect_friendly_minions(state, owner)
                .into_iter()
                .filter(|&e| state.world().has_race(e, race))
                .collect();
            let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
                return;
            };
            state.world_mut().add_enchantment(m, buff);
        }
        // A random damaged friendly minion (Rampage — friendly scope)
        EffectTarget::DamagedFriendlyMinion => {
            let minions: SmallList<Entity> = collect_friendly_minions(state, owner)
                .into_iter()
                .filter(|&e| state.world().damage(e).is_some_and(|d| d.0 > 0))
                .collect();
            let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
                return;
            };
            state.world_mut().add_enchantment(m, buff);
        }
        // A random damaged minion on either side (Rampage)
        EffectTarget::DamagedMinion => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            let minions: SmallList<Entity> = all
                .into_iter()
                .filter(|&e| state.world().damage(e).is_some_and(|d| d.0 > 0))
                .collect();
            let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
                return;
            };
            state.world_mut().add_enchantment(m, buff);
        }
        // All friendly minions of the race, excluding the source
        // (Coldlight Seer — all other Murlocs)
        EffectTarget::AllOtherFriendlyRace(race) => {
            let minions: SmallList<Entity> = collect_friendly_minions(state, owner)
                .into_iter()
                .filter(|&e| e != source && state.world().has_race(e, race))
                .collect();
            let world = state.world_mut();
            for minion in &minions {
                world.add_enchantment(*minion, buff);
            }
        }
        // The entity the triggering event happened to (Sword of Justice — the
        // just-summoned minion). Buffed directly; a subject that left play is
        // a no-op (the enchantment on a dead entity has no effect).
        EffectTarget::EventSubject => {
            if let Some(s) = subject {
                state.world_mut().add_enchantment(s, buff);
            }
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

    // Create the weapon entity with the full component set (triggers included —
    // e.g. Sword of Justice's summon trigger) and update the Player
    let inner = state.make_mut();
    let weapon = crate::cards::spawn_card_from_def(&mut inner.world, owner, card_def);
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
    source: Entity,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let _enemy = owner.opponent();
    let minions = match target {
        EffectTarget::AnyEnemy => collect_enemy_minions(state, owner, Some(source)),
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner, Some(source)),
        // Brewmasters (W15): return a FRIENDLY minion (the source included,
        // when chosen) to its owner's hand.
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
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
    source: Entity,
    amount: i32,
) {
    let minions = collect_enemy_minions(state, owner, Some(source));
    if minions.is_empty() {
        return;
    }

    let idx = state.rng_mut().next_usize(minions.len());
    let target_entity = minions[idx];

    let _ = state.world_mut().move_to_zone(target_entity, Zone::Hand);
    // Cost increase as a cost enchantment (roadmap G4)
    state.world_mut().add_enchantment(
        target_entity,
        Enchantment {
            attack: 0,
            health: 0,
            cost: amount,
            expiry: EnchantmentExpiry::Permanent,
        },
    );
}

/// Increases a minion's mana cost (e.g. Freezing Trap effect).
fn resolve_increase_cost(
    state: &mut GameState,
    owner: PlayerId,
    source: Entity,
    amount: i32,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let _enemy = owner.opponent();
    let minions = match target {
        EffectTarget::AnyEnemy => collect_enemy_minions(state, owner, Some(source)),
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner, Some(source)),
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
    source: Entity,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner, Some(source)),
        EffectTarget::AnyRace(race) => {
            // Any minion of the race on either side (Hungry Crab — destroy a Murloc)
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            all.into_iter()
                .filter(|&e| state.world().has_race(e, race))
                .collect()
        }
        EffectTarget::EnemyMinionAttackLE(max_atk) => collect_enemy_minions(state, owner, None)
            .into_iter()
            .filter(|&e| {
                state
                    .world()
                    .effective_attack(e)
                    .is_some_and(|a| a.0 <= max_atk)
            })
            .collect(),
        // Big Game Hunter (W14): an ENEMY minion with attack ≥ N — the
        // friendly side is not a legal destroy target.
        EffectTarget::EnemyMinionAttackGE(min_atk) => collect_enemy_minions(state, owner, None)
            .into_iter()
            .filter(|&e| {
                state
                    .world()
                    .effective_attack(e)
                    .is_some_and(|a| a.0 >= min_atk)
            })
            .collect(),
        EffectTarget::AnyMinionAttackGE(min_atk) => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            all.into_iter()
                .filter(|&e| {
                    state
                        .world()
                        .effective_attack(e)
                        .is_some_and(|a| a.0 >= min_atk)
                })
                .collect()
        }
        EffectTarget::DamagedEnemyMinion => collect_enemy_minions(state, owner, Some(source))
            .into_iter()
            .filter(|&e| {
                // "Damaged" = accumulated damage > 0 (roadmap G4 — the damage
                // component, not a base-vs-effective comparison)
                state.world().damage(e).is_some_and(|d| d.0 > 0)
            })
            .collect(),
        EffectTarget::TauntEnemyMinion => collect_enemy_minions(state, owner, Some(source))
            .into_iter()
            .filter(|&e| state.world().taunt(e).is_some())
            .collect(),
        EffectTarget::AllMinions => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            for &m in &all {
                let hp = state.world().effective_health(m).unwrap_or(Health(1));
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
    // Single-target destroy (roadmap F4): an explicit target destroys only
    // that minion; an explicit target that left the legal set fizzles (G9);
    // with no explicit target the effect destroys ONE random matching minion —
    // never all of them.
    let chosen = match explicit {
        Some(t) if minions.contains(&t) => Some(t),
        Some(_) => None, // fizzle — G9 re-validation
        None if minions.is_empty() => None,
        None => {
            let idx = state.rng_mut().next_usize(minions.len());
            Some(minions[idx])
        }
    };
    if let Some(m) = chosen {
        let hp = state.world().effective_health(m).unwrap_or(Health(1));
        queue.push(Event::DamageDealt {
            source: m,
            target: m,
            amount: hp.0.max(1),
        });
    }
}

/// Silences a minion — removes all effect components.
fn silence_entity(world: &mut crate::core::world::World, e: Entity) {
    world.remove_taunt(e);
    world.remove_battlecry(e);
    world.remove_deathrattle(e);
    world.remove_aura(e);
    world.remove_trigger(e);
    world.remove_divine_shield(e);
    world.remove_windfury(e);
    world.remove_charge(e);
    world.remove_spell_damage(e);
    // Enrage is an ability, so Silence takes it away too
    world.remove_enrage(e);
    // Silence strips enchantments, keeping base stats and damage (roadmap G4)
    world.remove_enchantments(e);
}

fn resolve_silence(
    state: &mut GameState,
    owner: PlayerId,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        // Single-pick silence (Ironbeak Owl, Spellbreaker — W14: either side):
        // the explicit target wins, otherwise a random pick at resolution; a
        // target that left the legal set fizzles (G9).
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner, None),
        EffectTarget::AnyMinion => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            all
        }
        EffectTarget::AllEnemyMinions => collect_all_enemy_minions(state, owner),
        EffectTarget::AllMinions => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            all
        }
        _ => return,
    };
    // Single-pick scopes pick one target (explicit → random → fizzle); the
    // AOE scopes silence every minion in the set.
    if matches!(
        target,
        EffectTarget::AnyEnemyMinion | EffectTarget::AnyMinion
    ) {
        let Some(t) = select_target(explicit, &minions, state.rng_mut()) else {
            return;
        };
        silence_entity(state.world_mut(), t);
        return;
    }
    for &m in &minions {
        silence_entity(state.world_mut(), m);
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
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner, None),
        _ => return,
    };
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    state.world_mut().set_attack(m, Attack(attack));
}

/// Sets a character's health to a fixed value (Alexstrasza — a hero's Health
/// to 15; W14).
fn resolve_set_health(
    state: &mut GameState,
    owner: PlayerId,
    health: i32,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let heroes: SmallList<Entity> = match target {
        EffectTarget::AnyHero => {
            let hero = state.player(owner).hero;
            let enemy_hero = state.player(owner.opponent()).hero;
            [hero, enemy_hero].into_iter().collect()
        }
        _ => return,
    };
    let Some(h) = select_target(explicit, &heroes, state.rng_mut()) else {
        return;
    };
    // Set the hero's health: the damage component moves to base − target
    // (roadmap G4 — damage, not base health). A 30-HP hero set to 15 takes
    // 15 accumulated damage; a target already below 15 is untouched only if
    // its base is below 15 (defensive).
    let base = state.world().health(h).unwrap_or(Health(30)).0;
    let dmg = (base - health).max(0);
    if dmg > 0 {
        state.world_mut().set_damage(h, Damage(dmg));
    } else {
        state.world_mut().remove_damage(h);
    }
}

/// Restores health — fires `CharacterHealed` triggers (Lightwarden) for each
/// character that actually healed (a heal that lands on an undamaged
/// character is not a heal event).
fn resolve_restore_health(
    state: &mut GameState,
    queue: &mut EventQueue,
    owner: PlayerId,
    amount: i32,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    // Prophet Velen's doubling is applied by `apply_spell_power` before the
    // effect reaches here, so that it covers only spell and hero-power healing
    // — a minion battlecry heal (Voodoo Doctor) is not doubled in HS.
    //
    // Healing reduces accumulated damage (roadmap G4); returns whether any
    // damage was actually removed
    fn heal(world: &mut crate::core::world::World, entity: Entity, amount: i32) -> bool {
        let dmg = world.damage(entity).unwrap_or(Damage(0)).0;
        if dmg <= 0 {
            return false;
        }
        let new_dmg = (dmg - amount).max(0);
        if new_dmg > 0 {
            world.set_damage(entity, Damage(new_dmg));
        } else {
            world.remove_damage(entity);
        }
        true
    }
    let mut healed: SmallList<Entity> = SmallList::new();
    // Single-pick scope (Earthen Ring Farseer, Voodoo Doctor — W15: any
    // character): explicit target wins, random at resolution, G9 fizzle.
    if target == EffectTarget::AnyCharacter {
        let mut chars = collect_friendly_characters(state, owner, None);
        chars.extend(collect_enemy_characters(state, owner, None));
        if let Some(c) = select_target(explicit, &chars, state.rng_mut()) {
            if heal(state.world_mut(), c, amount) {
                healed.push(c);
            }
        }
    } else {
        match target {
            EffectTarget::FriendlyHero | EffectTarget::Self_ => {
                let hero = state.player(owner).hero;
                if heal(state.world_mut(), hero, amount) {
                    healed.push(hero);
                }
            }
            EffectTarget::AllFriendlyMinions => {
                let minions = collect_friendly_minions(state, owner);
                let world = state.world_mut();
                for &m in &minions {
                    if heal(world, m, amount) {
                        healed.push(m);
                    }
                }
            }
            // Darkscale Healer (W14): all friendly characters, hero included.
            EffectTarget::AllFriendlyCharacters => {
                let hero = state.player(owner).hero;
                let minions = collect_friendly_minions(state, owner);
                let world = state.world_mut();
                if heal(world, hero, amount) {
                    healed.push(hero);
                }
                for &m in &minions {
                    if heal(world, m, amount) {
                        healed.push(m);
                    }
                }
            }
            _ => {}
        }
    }
    for entity in healed {
        fire_healed_trigger(state, queue, entity);
    }
}

/// Heals a character by reducing accumulated damage (the shared heal shape
/// of `resolve_restore_health` and the W3a combination effects); returns
/// whether any damage was actually removed.
fn heal_char(world: &mut crate::core::world::World, entity: Entity, amount: i32) -> bool {
    let dmg = world.damage(entity).unwrap_or(Damage(0)).0;
    if dmg <= 0 {
        return false;
    }
    let new_dmg = (dmg - amount).max(0);
    if new_dmg > 0 {
        world.set_damage(entity, Damage(new_dmg));
    } else {
        world.remove_damage(entity);
    }
    true
}

/// Fires heal triggers for a healed entity: `CharacterHealed` (Lightwarden —
/// any character healed, hero or minion, either side) and `MinionHealed`
/// (Northshire Cleric — any minion, either side, heroes excluded).
fn fire_healed_trigger(state: &mut GameState, queue: &mut EventQueue, entity: Entity) {
    let owner = state
        .world()
        .player(entity)
        .unwrap_or(state.active_player());
    // Core Set W3a — a healed hero marks the player for Death Metal Knight's
    // pay-health-instead-of-mana cost
    if state.world().card_type(entity) == Some(CardType::Hero) {
        let inner = state.make_mut();
        inner.players[owner.index()].healed_this_turn = true;
    }
    crate::engine::rules::fire_triggers(
        state,
        queue,
        crate::core::component::TriggerEvent::CharacterHealed,
        owner,
        Some(entity),
        None,
    );
    if state.world().card_type(entity) == Some(CardType::Minion) {
        crate::engine::rules::fire_triggers(
            state,
            queue,
            crate::core::component::TriggerEvent::MinionHealed,
            owner,
            Some(entity),
            None,
        );
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
        EffectTarget::AnyEnemy => collect_enemy_characters(state, owner, None),
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner, None),
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
    // Temporary attack as an until-end-of-turn enchantment (roadmap G4)
    if attack != 0 {
        state.world_mut().add_enchantment(
            hero,
            Enchantment {
                attack,
                health: 0,
                cost: 0,
                expiry: EnchantmentExpiry::UntilEndOfTurn,
            },
        );
    }
    // Add armor
    if armor > 0 {
        let inner = state.make_mut();
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
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner, Some(source)),
        EffectTarget::AnyEnemy => collect_enemy_characters(state, owner, Some(source)),
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
    queue: &mut EventQueue,
    owner: PlayerId,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner, None),
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        _ => return,
    };
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    // Get max health (based on the original definition; simplified here to 30 or the current max)
    // Simplified approach: heal from current health (use max_health if present, otherwise keep the current value)
    // Full heal clears all accumulated damage (roadmap G4) — a real heal
    // event for the trigger system (Lightwarden)
    if state.world().damage(m).is_some_and(|d| d.0 > 0) {
        state.world_mut().remove_damage(m);
        fire_healed_trigger(state, queue, m);
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
    source: Entity,
    owner: PlayerId,
    target: EffectTarget,
    attack_bonus: i32,
    explicit: Option<Entity>,
    subject: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        // Druid of the Claw's Charge branch — the minion itself
        EffectTarget::Self_ => {
            let mut list = SmallList::new();
            list.push(source);
            list
        }
        // Warsong Commander — the minion that was just summoned. The Charge is
        // granted permanently to that minion, so it survives the commander
        // leaving play.
        EffectTarget::EventSubject => {
            let mut list = SmallList::new();
            if let Some(s) = subject {
                list.push(s);
            }
            list
        }
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
    // Doubling the current attack becomes an enchantment equal to the current
    // attack (roadmap G4)
    let cur_atk = state.world().effective_attack(m).unwrap_or(Attack(0)).0;
    state.world_mut().add_enchantment(
        m,
        Enchantment {
            attack: cur_atk,
            health: 0,
            cost: 0,
            expiry: EnchantmentExpiry::Permanent,
        },
    );
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
    // Doubling the current health becomes an enchantment equal to the current
    // health (roadmap G4)
    let cur_hp = state.world().effective_health(m).unwrap_or(Health(0)).0;
    state.world_mut().add_enchantment(
        m,
        Enchantment {
            attack: 0,
            health: cur_hp,
            cost: 0,
            expiry: EnchantmentExpiry::Permanent,
        },
    );
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

/// Discards the whole hand (Deathwing — every card in hand goes to the
/// graveyard).
fn resolve_discard_hand(state: &mut GameState, owner: PlayerId) {
    let hand: Vec<Entity> = state.world().zones().iter(Zone::Hand, owner).collect();
    for card in hand {
        let _ = state.world_mut().move_to_zone(card, Zone::Graveyard);
    }
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
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner, Some(source)),
        EffectTarget::AnyEnemy => collect_enemy_characters(state, owner, Some(source)),
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
/// Deals `count` random pings of `amount` damage each across the target
/// scope (Mad Bomber, W16 — three random 1-damage pings across all OTHER
/// characters). The same character can be hit repeatedly (an official HS
/// property: 3 pings on a lone minion all land on it); stealthed characters
/// are included (random pings, not targeting). The source is excluded.
fn resolve_deal_damage_randomly(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    amount: i32,
    count: i32,
    target: EffectTarget,
) {
    let chars: SmallList<Entity> = match target {
        EffectTarget::AllCharacters => {
            let mut all = collect_friendly_characters(state, owner, None);
            all.extend(collect_all_enemy_characters(state, owner));
            all.into_iter().filter(|&e| e != source).collect()
        }
        // Devouring Plague (Core Set W1) — random enemy minions only
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner, Some(source)),
        // Dragonbane (Core Set W3a) — one random enemy character (hero included)
        EffectTarget::AnyEnemy => collect_enemy_characters(state, owner, Some(source)),
        _ => return,
    };
    if chars.is_empty() {
        return;
    }
    for _ in 0..count {
        let idx = state.rng_mut().next_usize(chars.len());
        queue.push(Event::DamageDealt {
            source,
            target: chars[idx],
            amount,
        });
    }
}

fn resolve_deal_damage_to_two(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    amount: i32,
) {
    let mut enemies = collect_enemy_minions(state, owner, Some(source));
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
    // "Set attack equal to health" — one-shot set; expressed as a base write
    // for now (set-to-value modifiers land with G5's modifier stack).
    let hp = state.world().effective_health(m).unwrap_or(Health(0)).0;
    state.world_mut().set_attack(m, Attack(hp));
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
        let hp = state.world().effective_health(m).unwrap_or(Health(1));
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
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner, None),
        _ => return,
    };
    let Some(m) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    let hp = state.world().effective_health(m).unwrap_or(Health(1));
    queue.push(Event::DamageDealt {
        source: m,
        target: m,
        amount: hp.0.max(1),
    });
    // Heal the hero (reduce accumulated damage)
    let hero = state.player(owner).hero;
    let dmg = state.world().damage(hero).unwrap_or(Damage(0)).0;
    let new_dmg = (dmg - heal).max(0);
    if new_dmg > 0 {
        state.world_mut().set_damage(hero, Damage(new_dmg));
    } else {
        state.world_mut().remove_damage(hero);
    }
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
    let atk = state
        .world()
        .effective_attack(sacrifice)
        .unwrap_or(Attack(0))
        .0;
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
    // Cost reduction as a cost enchantment — survives the bounce (Shadowstep, roadmap G4)
    state.world_mut().add_enchantment(
        target,
        Enchantment {
            attack: 0,
            health: 0,
            cost: -amount,
            expiry: EnchantmentExpiry::Permanent,
        },
    );
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
    let minions = collect_enemy_minions(state, owner, Some(source));
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
    let minions = collect_enemy_minions(state, owner, Some(source));
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
    let minions = collect_enemy_minions(state, owner, Some(source));
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    let target = minions[idx];
    let hp = state
        .world()
        .effective_health(target)
        .unwrap_or(Health(0))
        .0;
    // Destroy the target
    queue.push(Event::DamageDealt {
        source: target,
        target,
        amount: hp.max(1),
    });
    // The source minion (Natalie) gains the target's health as an enchantment
    state.world_mut().add_enchantment(
        source,
        Enchantment {
            attack: 0,
            health: hp.max(1),
            cost: 0,
            expiry: EnchantmentExpiry::Permanent,
        },
    );
}

/// Grants a friendly minion an attack bonus and immunity until end of turn
/// (Bestial Wrath — restricted to a friendly Beast).
fn resolve_grant_attack_and_immune(
    state: &mut GameState,
    owner: PlayerId,
    attack: i32,
    target: EffectTarget,
    explicit: Option<Entity>,
) {
    let minions: SmallList<Entity> = match target {
        EffectTarget::FriendlyRace(race) => collect_friendly_minions(state, owner)
            .into_iter()
            .filter(|&e| state.world().has_race(e, race))
            .collect(),
        _ => collect_friendly_minions(state, owner),
    };
    if minions.is_empty() {
        return;
    }
    let Some(target) = select_target(explicit, &minions, state.rng_mut()) else {
        return;
    };
    let world = state.world_mut();
    world.add_enchantment(
        target,
        Enchantment {
            attack,
            health: 0,
            cost: 0,
            expiry: EnchantmentExpiry::UntilEndOfTurn,
        },
    );
    world.set_immune(target, Immune);
}

/// Takes control of an enemy minion (Shadow Madness: temporary until end of
/// turn; Mind Control: permanent).
///
/// Shadow Madness only targets minions with attack ≤ 3.
fn resolve_take_control(state: &mut GameState, owner: PlayerId, until_end_of_turn: bool) {
    let minions: SmallList<Entity> = collect_enemy_minions(state, owner, None)
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

/// Takes permanent control of a random enemy minion with at most `max_attack`
/// attack (Cabal Shadow Priest — Battlecry: take control of an enemy minion
/// with 2 or less Attack).
fn resolve_take_control_attack_le(state: &mut GameState, owner: PlayerId, max_attack: i32) {
    let minions: SmallList<Entity> = collect_enemy_minions(state, owner, None)
        .into_iter()
        .filter(|&e| state.world().effective_attack(e).unwrap_or(Attack(0)).0 <= max_attack)
        .collect();
    if minions.is_empty() {
        return;
    }
    let idx = state.rng_mut().next_usize(minions.len());
    transfer_minion(state, minions[idx], owner);
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
    let minions = collect_enemy_minions(state, owner, None);
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
    let minions = collect_enemy_minions(state, owner, None);
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
pub(crate) fn add_card_to_hand(
    state: &mut GameState,
    player: PlayerId,
    card_def: &crate::cards::def::CardDef,
) {
    // Hand-size cap (F-A11): a GENERATED card past the 10-card limit is never
    // created (official rule — a full-hand add destroys the card).
    if state.world().zones().len(Zone::Hand, player) >= MAX_HAND_SIZE {
        return;
    }
    let world = state.world_mut();
    let e = crate::cards::spawn_card_from_def(world, player, card_def);
    world.set_zone(e, Zone::Hand);
    world.zones_mut().insert(Zone::Hand, player, e);
}

/// Copies a card entity's base definition into `to_player`'s hand
/// (pool-open M1). Copies the base card definition, not in-zone
/// enchantments — matches Classic-era behaviour and keeps copies
/// indistinguishable from freshly generated cards.
pub(crate) fn copy_card_to_hand(state: &mut GameState, src: Entity, to_player: PlayerId) {
    let Some(card_id) = state.world().card_id(src) else {
        return;
    };
    let Some(card_def) = crate::cards::def::card_by_id(card_id.0) else {
        return;
    };
    add_card_to_hand(state, to_player, card_def);
}

/// Copies `count` random cards from one of the enemy's zones into this
/// player's hand (Mind Vision — enemy hand; Thoughtsteal — enemy deck).
/// Sampling is without replacement over zone entities: two copies of the
/// same card are two distinct entities and may both be picked, the same
/// entity may not. An empty zone copies nothing; nothing is drawn, so no
/// fatigue applies.
fn resolve_copy_random_enemy_zone(state: &mut GameState, owner: PlayerId, count: u32, zone: Zone) {
    let enemy = owner.opponent();
    let mut candidates: Vec<Entity> = state.world().zones().iter(zone, enemy).collect();
    for _ in 0..count {
        if candidates.is_empty() {
            return;
        }
        let idx = state.rng_mut().next_usize(candidates.len());
        let card = candidates.remove(idx);
        copy_card_to_hand(state, card, owner);
    }
}

/// Summons a copy of a random minion from the enemy deck (Mindgames). The
/// enemy deck is not modified; a deck with no minions summons the
/// `fallback_card_id` token instead; a full board summons nothing
/// (`resolve_summon`'s board cap).
fn resolve_summon_random_enemy_deck_minion(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    fallback_card_id: &'static str,
) {
    let enemy = owner.opponent();
    let minions: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Deck, enemy)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect();
    let chosen: &'static str = if minions.is_empty() {
        fallback_card_id
    } else {
        let idx = state.rng_mut().next_usize(minions.len());
        state
            .world()
            .card_id(minions[idx])
            .expect("deck entity always carries a card id")
            .0
    };
    let _ = resolve_summon(state, queue, source, owner, chosen);
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
    let enemies = collect_enemy_characters(state, owner, Some(source));
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
            .effective_health(target)
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
fn collect_enemy_characters(
    state: &GameState,
    owner: PlayerId,
    source: Option<Entity>,
) -> SmallList<Entity> {
    collect_enemy_characters_impl(state, owner, false, source)
}

/// Collects all enemy characters (hero + minions, including stealthed) — for AOE effects.
fn collect_all_enemy_characters(state: &GameState, owner: PlayerId) -> SmallList<Entity> {
    collect_enemy_characters_impl(state, owner, true, None)
}

fn collect_enemy_characters_impl(
    state: &GameState,
    owner: PlayerId,
    include_stealth: bool,
    source: Option<Entity>,
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
            // Single-target selection excludes stealth; AOE (include_stealth) includes
            // it. Elusive (M5) is additionally excluded when the source is a spell
            // (spells can't target elusive minions; battlecries and attacks can).
            let spell = source.and_then(|s| state.world().card_type(s)) == Some(CardType::Spell);
            !(!include_stealth
                && (state.world().stealth(e).is_some()
                    || (spell && state.world().elusive(e).is_some())))
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
fn collect_enemy_minions(
    state: &GameState,
    owner: PlayerId,
    source: Option<Entity>,
) -> SmallList<Entity> {
    collect_enemy_minions_impl(state, owner, false, source)
}

/// Collects all enemy minions (including stealthed) — for AOE effects.
fn collect_all_enemy_minions(state: &GameState, owner: PlayerId) -> SmallList<Entity> {
    collect_enemy_minions_impl(state, owner, true, None)
}

fn collect_enemy_minions_impl(
    state: &GameState,
    owner: PlayerId,
    include_stealth: bool,
    source: Option<Entity>,
) -> SmallList<Entity> {
    let enemy = owner.opponent();
    state
        .world()
        .zones()
        .iter(Zone::Play, enemy)
        .filter(|&e| {
            state.world().card_type(e) == Some(CardType::Minion)
                && (include_stealth || state.world().stealth(e).is_none())
                && (include_stealth
                    || source.and_then(|s| state.world().card_type(s)) != Some(CardType::Spell)
                    || state.world().elusive(e).is_none())
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

/// Collects all friendly characters (hero + minions) for single-target picks.
/// Stealth is NOT a filter on the friendly side — stealth only blocks enemy
/// targeting (mirrors `collect_friendly_minions`; `collect_enemy_characters`
/// keeps the enemy-side stealth filter).
fn collect_friendly_characters(
    state: &GameState,
    owner: PlayerId,
    source: Option<Entity>,
) -> SmallList<Entity> {
    let mut chars: SmallList<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, owner)
        .filter(|&e| {
            let ct = state.world().card_type(e);
            if ct != Some(CardType::Minion) && ct != Some(CardType::Hero) {
                return false;
            }
            // Elusive (M5): spells can't target elusive minions; battlecries can.
            let spell = source.and_then(|s| state.world().card_type(s)) == Some(CardType::Spell);
            !(spell && state.world().elusive(e).is_some())
        })
        .collect();
    let hero = state.player(owner).hero;
    if chars.iter().all(|&c| c != hero) {
        chars.push(hero);
    }
    chars.sort_by_key(|e| (e.index, e.generation));
    chars
}
