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
    ALL_DARK_GIFTS, Attack, AttacksUsed, CardId, CardType, Charge, Cost, CostHealth, Damage,
    DarkGiftKind, Deathrattle, DivineShield, Durability, Elusive, Enchantment, EnchantmentExpiry,
    Freeze, Health, HeroPowerDef, ImbueClass, Immune, Lifesteal, Poison, Race, Reborn, Rush,
    Stealth, Taunt, Temporary, Trigger, TriggerEvent, TriggerTiming, Windfury,
};
use crate::core::effect::{CardEffect, EffectTarget, KeywordKind, RandomPool};
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
            // Quest progress (M2-W1): TLC_460 — "Discover 7 cards" — one per
            // discover surfaced.
            crate::engine::quest::progress(
                state,
                queue,
                owner,
                crate::cards::quest::QuestCondition::DiscoverCards,
                1,
                None,
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
            // Quest progress (M2-W1): TLC_433 — "Spend 15 Corpses" — the
            // amount is the corpses actually spent.
            crate::engine::quest::progress(
                state,
                queue,
                owner,
                crate::cards::quest::QuestCondition::SpendCorpses,
                cost,
                None,
            );
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
            // Quest progress (M2-W1): TLC_433 — "Spend 15 Corpses" — the
            // amount is the corpses actually spent.
            crate::engine::quest::progress(
                state,
                queue,
                owner,
                crate::cards::quest::QuestCondition::SpendCorpses,
                spend as u32,
                None,
            );
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
                // Quest progress (M2-W1): TLC_433 — "Spend 15 Corpses" — the
                // amount is the corpses actually spent.
                crate::engine::quest::progress(
                    state,
                    queue,
                    owner,
                    crate::cards::quest::QuestCondition::SpendCorpses,
                    corpses,
                    None,
                );
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
                // Quest progress (M2-W1): TLC_433 — "Spend 15 Corpses" —
                // the amount is the corpses actually spent.
                crate::engine::quest::progress(
                    state,
                    queue,
                    owner,
                    crate::cards::quest::QuestCondition::SpendCorpses,
                    corpses,
                    None,
                );
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
            // Quest progress (M2-W1): TLC_433 — "Spend 15 Corpses" — the
            // amount is the corpses actually spent.
            crate::engine::quest::progress(
                state,
                queue,
                owner,
                crate::cards::quest::QuestCondition::SpendCorpses,
                spend,
                None,
            );
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
            // Quest progress (M2-W1): TLC_433 — "Spend 15 Corpses" — the
            // amount is the corpses actually spent.
            crate::engine::quest::progress(
                state,
                queue,
                owner,
                crate::cards::quest::QuestCondition::SpendCorpses,
                spend,
                None,
            );
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
        // ----------------------------------------------------------------
        // 2025–2026 expansions M1-W1 (exp_edr_w1) — the Emerald Dream imbue
        // mechanic (see the design brief; simplifications registered in
        // fidelity-debt §14).
        // ----------------------------------------------------------------
        CardEffect::ImbueHeroPower => {
            resolve_imbue(state, owner);
        }
        CardEffect::ImbuedHeroPower { class } => {
            resolve_imbued_hero_power(state, queue, source, owner, class);
        }
        CardEffect::UseHeroPower => {
            resolve_use_hero_power(state, queue, owner);
        }
        CardEffect::DrawBeastAndImbue => {
            // Exotic Houndmaster — draw a Beast, then imbue
            resolve_draw_by_race(state, queue, owner, 1, crate::core::component::Race::Beast);
            resolve_imbue(state, owner);
        }
        CardEffect::RestoreAndDrawAndImbue { amount } => {
            // Aspect's Embrace — restore to the friendly hero, draw, imbue
            resolve_restore_health(
                state,
                queue,
                owner,
                amount,
                EffectTarget::FriendlyHero,
                None,
            );
            draw_card(state, queue, owner);
            resolve_imbue(state, owner);
        }
        CardEffect::SummonRandomTwoCostTauntAndImbue => {
            // Aegis of Light — summon a random 2-Cost minion, give it Taunt,
            // then imbue (token-excluded pool, Silvermoon Portal convention)
            let cost2: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| {
                        c.card_type == CardType::Minion && c.cost == 2 && !c.id.ends_with('t')
                    })
                    .collect();
            if let Some(&pick) = cost2.get(state.rng_mut().next_usize(cost2.len())) {
                if let Some(e) = resolve_summon(state, queue, source, owner, pick.id) {
                    state.world_mut().set_taunt(e, Taunt);
                }
            }
            resolve_imbue(state, owner);
        }
        CardEffect::ImbueAndReduceHandCost => {
            // Living Garden — imbue, then a random minion in hand costs (1)
            // less (the engine has no hand-targeting action space; the
            // choice is simplified to a random pick)
            resolve_imbue(state, owner);
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
            let cur = world.effective_cost(target).unwrap_or(Cost(0));
            world.set_cost(target, Cost((cur.0 - 1).max(0)));
        }
        CardEffect::ImbueAndTriggerHeroPower => {
            // Wisprider — imbue FIRST, then trigger the (just-replaced) hero
            // power once, free of charge
            resolve_imbue(state, owner);
            resolve_use_hero_power(state, queue, owner);
        }
        CardEffect::ImbueAndGetWisp => {
            // Spirit Gatherer — get a Wisp, then imbue
            if let Some(def) = crate::cards::def::card_by_id("EDR_851t") {
                add_card_to_hand(state, owner, def);
            }
            resolve_imbue(state, owner);
        }
        CardEffect::ImbueAndDebuffEnemies { attack_reduction } => {
            // Kaldorei Priestess — all enemy minions -2 Attack until your
            // next turn (the TempDebuff precedent: the debuff lasts until
            // the turn-end wrap-up, registered in fidelity-debt §14), then
            // imbue
            for enemy in collect_enemy_minions(state, owner, None) {
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
            resolve_imbue(state, owner);
        }
        CardEffect::DealDamageIfImbuedTwice { damage } => {
            // Resplendent Dreamweaver — 4 damage to a minion when the owner
            // has imbued at least twice (explicit target when given)
            if state.player(owner).imbue_count >= 2 {
                resolve_deal_damage(
                    state,
                    queue,
                    source,
                    owner,
                    damage,
                    EffectTarget::AnyMinion,
                    explicit_target,
                );
            }
        }
        CardEffect::DiscoverWildGodIfImbued4 => {
            // Malorne the Waywatcher — a random Wild God to hand; set its
            // Cost to (1) when the owner has imbued at least 4 times
            // (Discover simplified, fidelity-debt §14)
            if let Some(def) = crate::cards::pool::random_from_pool(
                crate::cards::pool::WILD_GOD_POOL,
                state.rng_mut(),
            ) {
                let hand_before = state.world().zones().len(Zone::Hand, owner);
                add_card_to_hand(state, owner, def);
                if state.world().zones().len(Zone::Hand, owner) > hand_before
                    && state.player(owner).imbue_count >= 4
                {
                    let added = state
                        .world()
                        .zones()
                        .iter(Zone::Hand, owner)
                        .nth(hand_before)
                        .expect("the added Wild God is in hand");
                    state.world_mut().set_cost(added, Cost(1));
                }
            }
        }
        CardEffect::ImbueEveryThirdSpell => {
            // Hamuul Runetotem — every 3 friendly spells cast (while he is
            // in play) imbues again; the counter is per-player so it
            // survives Hamuul leaving play (simplified, fidelity-debt §14)
            let count = {
                let inner = state.make_mut();
                let p = &mut inner.players[owner.index()];
                p.hamuul_spells_cast += 1;
                p.hamuul_spells_cast
            };
            if count % 3 == 0 {
                resolve_imbue(state, owner);
            }
        }
        CardEffect::SummonRandomDragonOfCost { cost } => {
            // The Emerald Portal token — a random 1-Cost Dragon. The current
            // Classic/Core window has no 1-Cost dragons, so the pool spans
            // the handwritten pool and the expansion baselines (token
            // excluded); the summoned dragon carries its generated vanilla
            // stats (fidelity-debt §14).
            let dragons: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .chain(crate::cards::sets::EXPANSION_CARDS.iter())
                    .filter(|c| {
                        c.card_type == CardType::Minion
                            && c.race == Some(crate::core::component::Race::Dragon)
                            && c.cost == cost
                            && !c.id.ends_with('t')
                    })
                    .collect();
            if dragons.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(dragons.len());
            let _ = resolve_summon(state, queue, source, owner, dragons[idx].id);
        }
        // ----------------------------------------------------------------
        // 2025–2026 expansions M1-W2 (exp_edr_w2) — the Emerald Dream
        // dark-gift mechanic (see the design brief; simplifications
        // registered in fidelity-debt §14). The W2 Discover cards add a
        // random qualifying minion to the hand and apply a random dark
        // gift; the gift marker rides the World `dark_gifts` component and
        // the static effects (enchantments + keyword components) are
        // applied by `apply_dark_gift`.
        // ----------------------------------------------------------------
        CardEffect::ApplyDarkGift { gift } => {
            // Direct grant — applies to the event subject (falling back to
            // the explicit target)
            if let Some(target) = event_subject.or(explicit_target) {
                apply_dark_gift(state, target, gift, owner);
            }
        }
        CardEffect::DiscoverWithDarkGift { pool } => {
            // Treacherous Tormentor (Legendary), Avant-Gardening (Deathrattle
            // minions), Jumpscare! (Demons costing 5+): a random qualifying
            // minion to hand, then a random dark gift (Discover simplified,
            // fidelity-debt §14)
            if let Some(def) = crate::cards::pool::random_card(state.rng_mut(), pool) {
                if let Some(added) = add_random_minion_to_hand(state, owner, def) {
                    let gift = random_dark_gift(state.rng_mut());
                    apply_dark_gift(state, added, gift, owner);
                }
            }
        }
        CardEffect::DiscoverDragonWithDarkGift => {
            // Darkrider — only while the owner is holding a Dragon
            let holding_dragon = state.world().zones().iter(Zone::Hand, owner).any(|e| {
                state
                    .world()
                    .has_race(e, crate::core::component::Race::Dragon)
            });
            if holding_dragon {
                if let Some(def) = crate::cards::pool::random_card(
                    state.rng_mut(),
                    crate::core::effect::RandomPool::Dragon,
                ) {
                    if let Some(added) = add_random_minion_to_hand(state, owner, def) {
                        let gift = random_dark_gift(state.rng_mut());
                        apply_dark_gift(state, added, gift, owner);
                    }
                }
            }
        }
        CardEffect::DiscoverUndeadWithCorpseGift { corpses } => {
            // Rite of Atrocity — a random Undead to hand; the dark gift is
            // given only when the owner can spend the corpses
            if let Some(def) = crate::cards::pool::random_card(
                state.rng_mut(),
                crate::core::effect::RandomPool::UndeadMinion,
            ) {
                if let Some(added) = add_random_minion_to_hand(state, owner, def) {
                    let have = state.player(owner).corpses;
                    if have >= corpses {
                        let inner = state.make_mut();
                        inner.players[owner.index()].corpses -= corpses;
                        // Quest progress (M2-W1): TLC_433 — "Spend 15
                        // Corpses" — the amount is the corpses actually spent.
                        crate::engine::quest::progress(
                            state,
                            queue,
                            owner,
                            crate::cards::quest::QuestCondition::SpendCorpses,
                            corpses,
                            None,
                        );
                        let gift = random_dark_gift(state.rng_mut());
                        apply_dark_gift(state, added, gift, owner);
                    }
                }
            }
        }
        CardEffect::DiscoverEnemyDeckMinionCopy { with_gift } => {
            // Nightmare Fuel — copy a random minion from the enemy deck into
            // the hand (**pool-open**, registered in POOL_OPEN_CARDS); the
            // Combo branch gives the copy a dark gift
            let enemy = owner.opponent();
            let candidates: Vec<Entity> = state
                .world()
                .zones()
                .iter(Zone::Deck, enemy)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            if candidates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            let hand_before = state.world().zones().len(Zone::Hand, owner);
            copy_card_to_hand(state, candidates[idx], owner);
            if with_gift {
                // the copy is the hand card at the pre-copy index (a full
                // hand creates nothing — F-A11 — and skips the gift)
                let added = state
                    .world()
                    .zones()
                    .iter(Zone::Hand, owner)
                    .nth(hand_before);
                if let Some(added) = added {
                    let gift = random_dark_gift(state.rng_mut());
                    apply_dark_gift(state, added, gift, owner);
                }
            }
        }
        CardEffect::DiscoverDeckMinionWithDarkGift => {
            // Nightmare Lord Xavius — a random minion from the player's own
            // deck moves to the hand (the own deck is in-pool, not
            // pool-open) and receives a dark gift
            let candidates: Vec<Entity> = state
                .world()
                .zones()
                .iter(Zone::Deck, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            if candidates.is_empty() {
                return;
            }
            if state.world().zones().len(Zone::Hand, owner) >= MAX_HAND_SIZE {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            let chosen = candidates[idx];
            let _ = state.world_mut().move_to_zone(chosen, Zone::Hand);
            let gift = random_dark_gift(state.rng_mut());
            apply_dark_gift(state, chosen, gift, owner);
        }
        CardEffect::ReduceHandMinionGiftCost => {
            // Overgrown Horror — hand minions carrying a dark gift cost (2)
            // less
            let targets: Vec<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter(|&e| {
                    state.world().card_type(e) == Some(CardType::Minion)
                        && state
                            .world()
                            .dark_gifts(e)
                            .is_some_and(|gifts| !gifts.is_empty())
                })
                .collect();
            for target in targets {
                let world = state.world_mut();
                let cur = world.effective_cost(target).unwrap_or(Cost(0));
                world.set_cost(target, Cost((cur.0 - 2).max(0)));
            }
        }
        CardEffect::GainStatsAndGrantDivineShield {
            attack,
            health,
            target,
        } => {
            // Lightmender choose branch 1 (M1-W3) — +stats and Divine Shield
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
            // The keyword lands on the same entity: Self_ resolves to the
            // source (the only target shape the W3 cards use); the generic
            // path re-selects a random eligible minion.
            if target == EffectTarget::Self_ {
                state.world_mut().set_divine_shield(source, DivineShield);
            } else {
                let mut all = collect_friendly_minions(state, owner);
                all.extend(collect_all_enemy_minions(state, owner));
                if let Some(t) = select_target(explicit_target, &all, state.rng_mut()) {
                    state.world_mut().set_divine_shield(t, DivineShield);
                }
            }
        }
        CardEffect::GainStatsAndGrantLifesteal {
            attack,
            health,
            target,
        } => {
            // Lightmender choose branch 2 (M1-W3) — +stats and Lifesteal
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
            if target == EffectTarget::Self_ {
                state.world_mut().set_lifesteal(source, Lifesteal);
            } else {
                let mut all = collect_friendly_minions(state, owner);
                all.extend(collect_all_enemy_minions(state, owner));
                if let Some(t) = select_target(explicit_target, &all, state.rng_mut()) {
                    state.world_mut().set_lifesteal(t, Lifesteal);
                }
            }
        }
        CardEffect::GrantPoisonousThisTurn => {
            // Barbed Thorn choose branch 1 (M1-W3) — the hero is Poisonous
            // until the end of this turn (weapon attacks check the attacker's
            // Poison component); the per-player flag expires both in the
            // turn-end wrap-up.
            let inner = state.make_mut();
            inner
                .world
                .set_poison(inner.players[owner.index()].hero, Poison);
            inner.players[owner.index()].hero_poisonous_this_turn = true;
        }
        CardEffect::GrantWeaponDeathrattleAllEnemies { damage } => {
            // Barbed Thorn choose branch 2 (M1-W3) — the equipped weapon
            // gains the deathrattle; it fires on break or replace
            // (WeaponDestroyed path in rules.rs).
            if let Some(weapon) = state.player(owner).weapon {
                state.world_mut().set_deathrattle(
                    weapon,
                    Deathrattle(CardEffect::DealDamage {
                        amount: damage,
                        target: EffectTarget::AllEnemies,
                    }),
                );
            }
        }
        CardEffect::DrawCardByType { count, card_type } => {
            // Reforestation choose branches (M1-W3) — draw a spell / draw a
            // minion: the deck is scanned for the first matches in deck order
            // (the resolve_draw_by_race pattern); no fatigue applies (a scan
            // draws what exists and nothing more).
            let matches: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Deck, owner)
                .filter(|&e| state.world().card_type(e) == Some(card_type))
                .take(count as usize)
                .collect();
            for card in matches {
                if state.world_mut().move_to_zone(card, Zone::Hand).is_ok() {
                    queue.push(Event::CardDrawn {
                        player: owner,
                        card,
                    });
                }
            }
        }
        CardEffect::SpendCorpsesDamageMinion { cost, damage } => {
            // Morbid Swarm choose branch 2 (M1-W3) — spend the corpses, then
            // deal damage to a minion; without the corpses the branch is a
            // no-op (the spend-corpses precedent).
            let have = state.player(owner).corpses;
            if have < cost {
                return;
            }
            {
                let inner = state.make_mut();
                inner.players[owner.index()].corpses -= cost;
            }
            // Quest progress (M2-W1): TLC_433 — "Spend 15 Corpses" — the
            // amount is the corpses actually spent.
            crate::engine::quest::progress(
                state,
                queue,
                owner,
                crate::cards::quest::QuestCondition::SpendCorpses,
                cost,
                None,
            );
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            if let Some(t) = select_target(explicit_target, &all, state.rng_mut()) {
                queue.push(Event::DamageDealt {
                    source,
                    target: t,
                    amount: damage,
                });
            }
        }
        CardEffect::DamageAllMinions { damage } => {
            // Ominous Nightmares choose branch 1 / Wyvern's Slumber choose
            // branch 2 (M1-W3) — damage every minion on either side.
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            for minion in &all {
                queue.push(Event::DamageDealt {
                    source,
                    target: *minion,
                    amount: damage,
                });
            }
        }
        CardEffect::AddRandomDruidSpell => {
            // Spark of Life choose branch 2 (M1-W3) — a random Druid spell
            // (the real Discover simplified to random, §14.2).
            let spells: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::DRUID_CLASSIC
                    .iter()
                    .filter(|c| c.card_type == CardType::Spell)
                    .collect();
            if spells.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(spells.len());
            add_card_to_hand(state, owner, spells[idx]);
        }
        CardEffect::AddRandomOtherClassChooseOneCard => {
            // Symbiosis (M1-W3) — a random Choose One card of another class
            // from the fixed OTHER_CLASS_CHOOSE_ONE_POOL table (the real
            // Discover simplified to random, §14.2).
            if let Some(def) = crate::cards::pool::random_from_pool(
                crate::cards::pool::OTHER_CLASS_CHOOSE_ONE_POOL,
                state.rng_mut(),
            ) {
                add_card_to_hand(state, owner, def);
            }
        }
        CardEffect::AttackTwoRandomEnemyMinionsIfCostLE { cost } => {
            // Verdant Dreamsaber (M1-W4a) — when the played card's Cost is at
            // most the threshold, the minion "attacks" two random enemy
            // minions (the attack modeled as direct damage, like the
            // excess-damage family). The cost read is the composed play cost
            // of the entity.
            if state
                .world()
                .effective_cost(source)
                .is_some_and(|c| c.0 > cost as i32)
            {
                return;
            }
            let atk = state.world().effective_attack(source).map_or(0, |a| a.0);
            let mut enemies = collect_enemy_minions(state, owner, Some(source));
            for _ in 0..2 {
                if enemies.is_empty() {
                    return;
                }
                let idx = state.rng_mut().next_usize(enemies.len());
                let target = enemies.remove(idx);
                queue.push(Event::DamageDealt {
                    source,
                    target,
                    amount: atk,
                });
            }
        }
        CardEffect::GainArmorSummonCostTaunt { armor, cost } => {
            // Ward of Earth (M1-W4a) — armor to the hero, then a random
            // minion of the Cost with Taunt
            {
                let inner = state.make_mut();
                inner.players[owner.index()].armor += armor;
            }
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Minion && c.cost == cost as i32)
                    .collect();
            if candidates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            if let Some(minion) = resolve_summon(state, queue, source, owner, candidates[idx].id) {
                state.world_mut().set_taunt(minion, Taunt);
            }
        }
        CardEffect::AddRandomCostMinionWithDarkGift { cost } => {
            // Creature of Madness (M1-W4a) — a random minion of the Cost with
            // a Dark Gift (the real Discover simplified to random, §14.3).
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Minion && c.cost == cost as i32)
                    .collect();
            if candidates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            let gift = random_dark_gift(state.rng_mut());
            if let Some(added) = add_random_minion_to_hand(state, owner, candidates[idx]) {
                apply_dark_gift(state, added, gift, owner);
            }
        }
        CardEffect::BuffTopDeckMinions {
            attack,
            health,
            count,
        } => {
            // Beanstalk Brute (M1-W4a) — the deck is ordered, so the "top n
            // minions" are the first n minions in deck order; enchantments
            // persist deck → hand → play (G4), so the buff lands when the
            // cards are drawn.
            let buff = Enchantment {
                attack,
                health,
                cost: 0,
                expiry: EnchantmentExpiry::Permanent,
            };
            let world = state.world();
            let top: SmallList<Entity> = world
                .zones()
                .iter(Zone::Deck, owner)
                .filter(|&e| world.card_type(e) == Some(CardType::Minion))
                .take(count as usize)
                .collect();
            for minion in &top {
                state.world_mut().add_enchantment(*minion, buff);
            }
        }
        CardEffect::ShuffleAllMinionsIntoDecks => {
            // Typhoon (M1-W4a) — every minion (both sides) goes into a random
            // player's deck at a random position (the Tradeable shuffle
            // pattern); leaving the battlefield strips enchantments and
            // damage (G4).
            let mut all: SmallList<Entity> = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            // Quest progress (M2-W1): TLC_513 — "Shuffle cards into your
            // deck" — one per minion actually shuffled into the owner's deck
            // (W2 pins the official per-effect counting).
            for minion in all {
                let old_owner = state.world().player(minion);
                let dest = if state.rng_mut().next_usize(2) == 0 {
                    owner
                } else {
                    owner.opponent()
                };
                let deck_count = state.world().zones().len(Zone::Deck, dest);
                let position = if deck_count > 0 {
                    state.rng_mut().next_usize(deck_count + 1)
                } else {
                    0
                };
                if let Some(old_owner) = old_owner {
                    let world = state.world_mut();
                    world.remove_enchantments(minion);
                    world.remove_damage(minion);
                    world.zones_mut().remove(Zone::Play, old_owner, minion);
                    world.set_player(minion, dest);
                    world.set_zone(minion, Zone::Deck);
                    world
                        .zones_mut()
                        .insert_at(Zone::Deck, dest, minion, position);
                    if dest == owner {
                        crate::engine::quest::progress(
                            state,
                            queue,
                            owner,
                            crate::cards::quest::QuestCondition::ShuffleCards,
                            1,
                            None,
                        );
                    }
                }
            }
        }
        CardEffect::DrawDeckSpellAndAddRandomSpell => {
            // Dragonscale Armaments (M1-W4a) — draw a spell that started in
            // the deck; the "one that didn't start there" half is a random
            // spell added to hand (no origin tracking, §14.3).
            let matches: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Deck, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Spell))
                .take(1)
                .collect();
            for card in matches {
                if state.world_mut().move_to_zone(card, Zone::Hand).is_ok() {
                    queue.push(Event::CardDrawn {
                        player: owner,
                        card,
                    });
                }
            }
            let spells: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Spell)
                    .collect();
            if !spells.is_empty() {
                let idx = state.rng_mut().next_usize(spells.len());
                add_card_to_hand(state, owner, spells[idx]);
            }
        }
        CardEffect::SetStatsByFriendlyTarget {
            enemy_attack,
            enemy_health,
            friendly_attack,
            friendly_health,
        } => {
            // Mark of Ursol (M1-W4a) — the stat block depends on which side
            // owns the target; setting health also clears accumulated damage
            // (set semantics).
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            let Some(t) = select_target(explicit_target, &all, state.rng_mut()) else {
                return;
            };
            let friendly = state.world().player(t) == Some(owner);
            let (atk, hp) = if friendly {
                (friendly_attack, friendly_health)
            } else {
                (enemy_attack, enemy_health)
            };
            let world = state.world_mut();
            world.set_attack(t, Attack(atk));
            world.set_health(t, Health(hp));
            world.remove_damage(t);
        }
        CardEffect::GainAttackEqualSpellCost => {
            // Animated Moonwell (M1-W4a) — the just-cast friendly spell rides
            // as the trigger subject; gain Attack equal to its Cost.
            if let Some(spell) = event_subject {
                if let Some(c) = state.world().cost(spell) {
                    state.world_mut().add_enchantment(
                        source,
                        Enchantment {
                            attack: c.0,
                            health: 0,
                            cost: 0,
                            expiry: EnchantmentExpiry::Permanent,
                        },
                    );
                }
            }
        }
        CardEffect::DamageLowestHealthEnemyTwice { amount } => {
            // Renewing Flames (M1-W4a) — the lowest-Health enemy is re-picked
            // per hit (Lifesteal rides the source spell via apply_card_keywords).
            for _ in 0..2 {
                let enemies = collect_enemy_characters(state, owner, Some(source));
                let Some(&target) = enemies
                    .iter()
                    .min_by_key(|&&e| state.world().effective_health(e).map_or(i32::MAX, |h| h.0))
                else {
                    return;
                };
                queue.push(Event::DamageDealt {
                    source,
                    target,
                    amount,
                });
            }
        }
        CardEffect::DrawAndGainStats { attack, health } => {
            // Dreamwarden (M1-W4a) — draw the top card and buff self (the
            // real "didn't start there" condition is dropped, §14.3).
            draw_card(state, queue, owner);
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
        CardEffect::ShuffleCardIntoDeck { card_id, count } => {
            // Illusory Greenwing (M1-W4a) — copies go into the deck at random
            // positions (the Tradeable shuffle pattern).
            let Some(def) = crate::cards::def::card_by_id(card_id) else {
                return;
            };
            for _ in 0..count {
                let deck_count = state.world().zones().len(Zone::Deck, owner);
                let position = if deck_count > 0 {
                    state.rng_mut().next_usize(deck_count + 1)
                } else {
                    0
                };
                let world = state.world_mut();
                let e = crate::cards::spawn_card_from_def(world, owner, def);
                world.set_zone(e, Zone::Deck);
                world.zones_mut().insert_at(Zone::Deck, owner, e, position);
                // Quest progress (M2-W1): TLC_513 — "Shuffle cards into your
                // deck" — one per shuffled copy.
                crate::engine::quest::progress(
                    state,
                    queue,
                    owner,
                    crate::cards::quest::QuestCondition::ShuffleCards,
                    1,
                    None,
                );
                // M2-W4a: the per-shuffle counter (Underbrush Tracker's
                // discount, Knockback's damage, Merchant of Legend's count)
                // counts each shuffled copy — one bump per card.
                state.make_mut().players[owner.index()].shuffled_count += 1;
            }
        }
        CardEffect::AmphibianSpiritBuff { attack, health } => {
            // Amphibian's Spirit (M1-W4a) — the buff and the recursive
            // deathrattle chain; the battlecry targets the explicit minion,
            // the deathrattle picks a random friendly minion. The engine
            // stores one Deathrattle per entity, so an existing one is
            // replaced (registered simplification, §14.3).
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            let Some(t) = select_target(explicit_target, &all, state.rng_mut()) else {
                return;
            };
            state.world_mut().add_enchantment(
                t,
                Enchantment {
                    attack,
                    health,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
            state.world_mut().set_deathrattle(
                t,
                Deathrattle(CardEffect::AmphibianSpiritBuff { attack, health }),
            );
        }
        CardEffect::DamageAndSummonWolfIfKilled { damage } => {
            // Spirit Bond (M1-W4a) — damage a minion; a kill summons the
            // 3/2 Wolf with Rush (the "if it dies" prediction, like
            // DamageAndSummonCopyIfKilled).
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            let Some(t) = select_target(explicit_target, &all, state.rng_mut()) else {
                return;
            };
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
                let _ = resolve_summon(state, queue, source, owner, "EDR_262t");
            }
        }
        CardEffect::AddRandomSpellCostsLess { reduction } => {
            // Horn of Plenty (M1-W4a) — a random spell (Nature school not
            // modeled, §14.3) costing less; the reduction rides a Permanent
            // cost enchantment (the draw_card_with_reduction pattern).
            let spells: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Spell)
                    .collect();
            if spells.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(spells.len());
            if let Some(added) = add_random_minion_to_hand(state, owner, spells[idx]) {
                let world = state.world_mut();
                let cur = world.effective_cost(added).unwrap_or(Cost(0));
                world.set_cost(added, Cost((cur.0 - reduction as i32).max(0)));
            }
        }
        CardEffect::SummonTreantCopyingSpell => {
            // Grove Shaper (M1-W4a) — the treant's deathrattle cannot
            // remember the triggering spell, so it adds a random spell
            // instead (the simplification lives on the token, §14.3).
            let _ = resolve_summon(state, queue, source, owner, "EDR_271t");
        }
        CardEffect::SummonEggHatchingDragon => {
            // Clutch of Corruption (M1-W4a) — the 0/2 Egg whose deathrattle
            // hatches a copy of a random friendly Dragon (see §14.3).
            let _ = resolve_summon(state, queue, source, owner, "EDR_454t");
        }
        CardEffect::ResurrectRandomFallenDragon => {
            // Succumb to Madness (M1-W4a) — the graveyard IS the death
            // record: a random friendly Dragon that died this game is
            // resummoned (Discover simplified to random, §14.3).
            let fallen: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Graveyard, owner)
                .filter(|&e| {
                    state.world().card_type(e) == Some(CardType::Minion)
                        && state
                            .world()
                            .has_race(e, crate::core::component::Race::Dragon)
                })
                .collect();
            if fallen.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(fallen.len());
            if let Some(def) = state
                .world()
                .card_id(fallen[idx])
                .and_then(|c| crate::cards::def::card_by_id(c.0))
            {
                let _ = resolve_summon(state, queue, source, owner, def.id);
            }
        }
        CardEffect::EquipSwordIfHoldingDragon => {
            // Brood Keeper (M1-W4a) — holding a Dragon equips the 2/2 Sword
            let holding = state.world().zones().iter(Zone::Hand, owner).any(|e| {
                state
                    .world()
                    .card_id(e)
                    .and_then(|c| crate::cards::def::card_by_id(c.0))
                    .is_some_and(|d| d.race == Some(crate::core::component::Race::Dragon))
            });
            if holding {
                resolve_equip_weapon(state, queue, owner, "EDR_457t");
            }
        }
        CardEffect::DamageAllOtherFriendlyMinions { damage } => {
            // Afflicted Devastator (M1-W4a) battlecry — all friendly minions
            // except the source take the damage
            for minion in collect_friendly_minions(state, owner) {
                if minion != source {
                    queue.push(Event::DamageDealt {
                        source,
                        target: minion,
                        amount: damage,
                    });
                }
            }
        }
        CardEffect::DamageMinionWithMoonLifesteal { amount } => {
            // Wish of the New Moon (M1-W4a) — after 3 spells cast the spell
            // gains Lifesteal (the spell entity carries it, so the damage
            // heals through the lifesteal pipeline); the per-card New Moon
            // counter is approximated by the player's spell total (§14.3).
            // The current cast counts: the official counter increments at
            // play time, before the effect — `spells_cast_total` is only
            // bumped by the after-cast SpellCast event.
            if state.player(owner).spells_cast_total + 1 >= 3 {
                state.world_mut().set_lifesteal(source, Lifesteal);
            }
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            if let Some(t) = select_target(explicit_target, &all, state.rng_mut()) {
                queue.push(Event::DamageDealt {
                    source,
                    target: t,
                    amount,
                });
            }
        }
        CardEffect::SummonTwoRandomCostMinions {
            base_cost,
            upgraded_cost,
        } => {
            // Ritual of the New Moon (M1-W4a) — the summon cost upgrades
            // after 3 spells cast (same approximation as Wish, §14.3 —
            // the current cast counts)
            let cost = if state.player(owner).spells_cast_total + 1 >= 3 {
                upgraded_cost as i32
            } else {
                base_cost as i32
            };
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Minion && c.cost == cost)
                    .collect();
            if candidates.is_empty() {
                return;
            }
            for _ in 0..2 {
                let idx = state.rng_mut().next_usize(candidates.len());
                let _ = resolve_summon(state, queue, source, owner, candidates[idx].id);
            }
        }
        CardEffect::DamageIfHoldingSpell5Plus { amount } => {
            // Weaver of the Cycle (M1-W4a) — holding a spell costing (5) or
            // more deals the damage to the enemy hero
            let holding = state.world().zones().iter(Zone::Hand, owner).any(|e| {
                state.world().card_type(e) == Some(CardType::Spell)
                    && state.world().effective_cost(e).is_some_and(|c| c.0 >= 5)
            });
            if holding {
                let hero = state.player(owner.opponent()).hero;
                queue.push(Event::DamageDealt {
                    source,
                    target: hero,
                    amount,
                });
            }
        }
        CardEffect::SummonCopyIfAttackGE { attack } => {
            // Mythical Runebear (M1-W4a) — summon a copy when the Attack
            // threshold is met (reads the source's effective Attack)
            if state
                .world()
                .effective_attack(source)
                .is_some_and(|a| a.0 >= attack)
            {
                if let Some(def) = state
                    .world()
                    .card_id(source)
                    .and_then(|c| crate::cards::def::card_by_id(c.0))
                {
                    let _ = resolve_summon(state, queue, source, owner, def.id);
                }
            }
        }
        CardEffect::RestoreHealthAndPendingSelfDamage {
            heal,
            damage,
            turns,
        } => {
            // Rotten Apple (M1-W4a) — heal the hero, then queue self-damage
            // that ticks at the END of the next turns (the TurnEnded timer
            // in rules.rs; timing simplification, §14.3).
            resolve_restore_health(state, queue, owner, heal, EffectTarget::FriendlyHero, None);
            {
                let p = &mut state.make_mut().players[owner.index()];
                p.self_damage_pending = damage;
                p.self_damage_turns = turns;
            }
        }
        CardEffect::DestroyCrystalGainCrystalsLater { gain, turns } => {
            // Fractured Power (M1-W4a) — destroy one crystal now, gain the
            // crystals at the END of the next turns (the TurnEnded timer in
            // rules.rs; timing simplification, §14.3).
            {
                let p = &mut state.make_mut().players[owner.index()];
                p.mana_crystals = (p.mana_crystals - 1).max(0);
                p.current_mana = p.current_mana.min(p.mana_crystals);
            }
            {
                let p = &mut state.make_mut().players[owner.index()];
                p.crystal_gain_pending = gain;
                p.crystal_gain_turns = turns;
            }
        }
        CardEffect::DrawMinionCostGE { cost } => {
            // Rotheart Dryad (M1-W4a) — the deck is scanned for the first
            // minion costing at least the threshold (no fatigue — a scan
            // draws what exists, the resolve_draw_by_race pattern).
            let matches: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Deck, owner)
                .filter(|&e| {
                    state.world().card_type(e) == Some(CardType::Minion)
                        && state
                            .world()
                            .effective_cost(e)
                            .is_some_and(|c| c.0 >= cost as i32)
                })
                .take(1)
                .collect();
            for card in matches {
                if state.world_mut().move_to_zone(card, Zone::Hand).is_ok() {
                    queue.push(Event::CardDrawn {
                        player: owner,
                        card,
                    });
                }
            }
        }
        CardEffect::GainDeathrattleOfDiedThisTurn => {
            // Archdruid of Thorns (M1-W4a) — gain the deathrattle of the
            // most recently died friendly minion this turn (one Deathrattle
            // slot per entity, §14.3).
            if let Some(&last) = state.player(owner).died_this_turn.last() {
                if let Some(def) = state
                    .world()
                    .card_id(last)
                    .and_then(|c| crate::cards::def::card_by_id(c.0))
                {
                    if let Some(dr) = def.deathrattle {
                        state.world_mut().set_deathrattle(source, Deathrattle(dr));
                    }
                }
            }
        }
        CardEffect::AddRandomDeckMinionToHand => {
            // Hungering Ancient (M1-W4a) deathrattle — a random minion from
            // the deck joins the hand (the eaten identity cannot be stored
            // per instance, §14.3).
            let minions: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Deck, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            if minions.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(minions.len());
            let card = minions[idx];
            if state.world_mut().move_to_zone(card, Zone::Hand).is_ok() {
                queue.push(Event::CardDrawn {
                    player: owner,
                    card,
                });
            }
        }
        CardEffect::EatDeckMinionGainStats => {
            // Hungering Ancient (M1-W4a) end of turn — eat the first minion
            // in deck order and gain its stats (read before the move).
            let minion = state
                .world()
                .zones()
                .iter(Zone::Deck, owner)
                .find(|&e| state.world().card_type(e) == Some(CardType::Minion));
            let Some(minion) = minion else {
                return;
            };
            let atk = state.world().effective_attack(minion).map_or(0, |a| a.0);
            let hp = state.world().effective_health(minion).map_or(0, |h| h.0);
            let _ = state.world_mut().move_to_zone(minion, Zone::Graveyard);
            state.world_mut().add_enchantment(
                source,
                Enchantment {
                    attack: atk,
                    health: hp,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
        }
        CardEffect::DebuffRandomHandMinionBoth { attack_reduction } => {
            // Twisted Treant (M1-W4a) — one random minion in EACH player's
            // hand loses -2/-2; the debuff enchantment persists from hand
            // into play (G4 keeps deck/hand enchantments).
            for p in [owner, owner.opponent()] {
                let minions: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Hand, p)
                    .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                    .collect();
                if minions.is_empty() {
                    continue;
                }
                let idx = state.rng_mut().next_usize(minions.len());
                state.world_mut().add_enchantment(
                    minions[idx],
                    Enchantment {
                        attack: -attack_reduction,
                        health: -attack_reduction,
                        cost: 0,
                        expiry: EnchantmentExpiry::Permanent,
                    },
                );
            }
        }
        CardEffect::SpendAllManaCastRandomSpell => {
            // Forbidden Shrine (M1-W4a) — spend all mana, then cast a random
            // spell of exactly that cost ("cast" = direct effect resolution —
            // no spell entity or SpellCast event, §14.3).
            let spent = state.player(owner).current_mana;
            state.make_mut().players[owner.index()].current_mana = 0;
            if spent <= 0 {
                return;
            }
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| c.card_type == CardType::Spell && c.cost == spent)
                    .collect();
            if candidates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            if let Some(effect) = candidates[idx].battlecry {
                resolve_effect(state, queue, source, owner, effect, None, None);
            }
        }
        CardEffect::CopyLowestCostEnemyHandCard => {
            // Tricky Satyr (M1-W4a) — a copy of the lowest-Cost card in the
            // opponent's hand (pool-open — registered in POOL_OPEN_CARDS)
            let enemy_hand: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner.opponent())
                .collect();
            if enemy_hand.is_empty() {
                return;
            }
            let lowest = enemy_hand
                .iter()
                .min_by_key(|&&e| state.world().effective_cost(e).map_or(0, |c| c.0));
            if let Some(&lowest) = lowest {
                copy_card_to_hand(state, lowest, owner);
            }
        }
        CardEffect::OpponentDrawsTwoAndCopies => {
            // Mimicry (M1-W4a) — the opponent draws two; the player gets
            // copies of the drawn cards (pool-open — POOL_OPEN_CARDS).
            let enemy = owner.opponent();
            for _ in 0..2 {
                if let Some(drawn) = draw_card_no_queue(state, queue, enemy) {
                    copy_card_to_hand(state, drawn, owner);
                }
            }
        }
        CardEffect::ReturnFriendlyMinionSummonSpider => {
            // Web of Deception (M1-W4a) — bounce a friendly minion to hand
            // (enchantments wiped by the move, G4) to summon the 4/4 Spider
            let minions = collect_friendly_minions(state, owner);
            let Some(t) = select_target(explicit_target, &minions, state.rng_mut()) else {
                return;
            };
            let _ = state.world_mut().move_to_zone(t, Zone::Hand);
            let _ = resolve_summon(state, queue, source, owner, "EDR_523t");
        }
        CardEffect::ShuffleMatchingEnemyHandCardIntoDeck => {
            // Shadowcloaked Assailant (M1-W4a) — when holding a card the
            // opponent also holds, shuffle one random matching enemy card
            // into their deck (pool-open — POOL_OPEN_CARDS; several matches
            // pick one randomly, §14.3).
            let enemy = owner.opponent();
            let own_ids: Vec<&'static str> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter_map(|e| state.world().card_id(e).map(|c| c.0))
                .collect();
            let matches: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, enemy)
                .filter(|&e| {
                    state
                        .world()
                        .card_id(e)
                        .is_some_and(|c| own_ids.contains(&c.0))
                })
                .collect();
            if matches.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(matches.len());
            let card = matches[idx];
            let deck_count = state.world().zones().len(Zone::Deck, enemy);
            let position = if deck_count > 0 {
                state.rng_mut().next_usize(deck_count + 1)
            } else {
                0
            };
            {
                let world = state.world_mut();
                world.zones_mut().remove(Zone::Hand, enemy, card);
                world.set_zone(card, Zone::Deck);
                world
                    .zones_mut()
                    .insert_at(Zone::Deck, enemy, card, position);
            }
        }
        CardEffect::DestroyFriendlyMinionGainArmor { armor } => {
            // Siphoning Growth (M1-W4a) — destroy a friendly minion to gain
            // the Armor
            let minions = collect_friendly_minions(state, owner);
            let Some(t) = select_target(explicit_target, &minions, state.rng_mut()) else {
                return;
            };
            resolve_destroy_minion(
                state,
                queue,
                owner,
                source,
                EffectTarget::FriendlyMinion,
                Some(t),
            );
            {
                let inner = state.make_mut();
                inner.players[owner.index()].armor += armor;
            }
        }
        CardEffect::DrawSpellCostGE { cost } => {
            // Fae Trickster (M1-W4a) — scan for the first spell costing at
            // least the threshold (no fatigue, the scan pattern)
            let matches: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Deck, owner)
                .filter(|&e| {
                    state.world().card_type(e) == Some(CardType::Spell)
                        && state
                            .world()
                            .effective_cost(e)
                            .is_some_and(|c| c.0 >= cost as i32)
                })
                .take(1)
                .collect();
            for card in matches {
                if state.world_mut().move_to_zone(card, Zone::Hand).is_ok() {
                    queue.push(Event::CardDrawn {
                        player: owner,
                        card,
                    });
                }
            }
        }
        CardEffect::DrawDragonsReduced { count, reduction } => {
            // Tormented Dreadwing (M1-W4a) — scan for Dragons, reducing each
            // drawn one (the draw_card_with_reduction pattern)
            let matches: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Deck, owner)
                .filter(|&e| {
                    state
                        .world()
                        .has_race(e, crate::core::component::Race::Dragon)
                })
                .take(count as usize)
                .collect();
            for card in matches {
                if state.world_mut().move_to_zone(card, Zone::Hand).is_ok() {
                    if reduction > 0 {
                        let world = state.world_mut();
                        let cur = world.effective_cost(card).unwrap_or(Cost(0));
                        world.set_cost(card, Cost((cur.0 - reduction as i32).max(0)));
                    }
                    queue.push(Event::CardDrawn {
                        player: owner,
                        card,
                    });
                }
            }
        }
        CardEffect::SummonCopyOfSelf => {
            // Bloodthistle Illusionist (M1-W4a) — a plain copy of the source
            // (the shared-secret-death clause is unmodeled, §14.3). The
            // copy's own battlecry is stripped: a summoned copy must not
            // re-fire the summoning battlecry (real HS — summons never fire
            // battlecries), or the Illusionist would recurse to a full board
            // through the MinionSummoned dispatch.
            if let Some(def) = state
                .world()
                .card_id(source)
                .and_then(|c| crate::cards::def::card_by_id(c.0))
            {
                if let Some(copy) = resolve_summon(state, queue, source, owner, def.id) {
                    state.world_mut().remove_battlecry(copy);
                }
            }
        }
        CardEffect::DestroyFriendlyWispDraw { count } => {
            // Divination (M1-W4a) — destroy a friendly Wisp (the EDR_851t
            // token — the engine's only Wisp) to draw
            let wisps: SmallList<Entity> = collect_friendly_minions(state, owner)
                .into_iter()
                .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "EDR_851t"))
                .collect();
            if wisps.is_empty() {
                return;
            }
            let t = select_target(explicit_target, &wisps, state.rng_mut()).unwrap_or(wisps[0]);
            resolve_destroy_minion(
                state,
                queue,
                owner,
                source,
                EffectTarget::FriendlyMinion,
                Some(t),
            );
            for _ in 0..count {
                draw_card(state, queue, owner);
            }
        }
        CardEffect::DrawAndSummonLeeches { draw } => {
            // Sanguine Infestation (M1-W4a) — draw, then summon the Leeches
            for _ in 0..draw {
                draw_card(state, queue, owner);
            }
            for _ in 0..draw {
                let _ = resolve_summon(state, queue, source, owner, "EDR_810t");
            }
        }
        CardEffect::DrawAndSummonDreadseed { draw } => {
            // Grim Harvest (M1-W4a) — draw, then summon the Dreadseed token
            // (Dormant simplified to the can't-attack token, §14.3)
            for _ in 0..draw {
                draw_card(state, queue, owner);
            }
            let _ = resolve_summon(state, queue, source, owner, "EDR_820t");
        }
        CardEffect::NextHeroPowerCostsZero => {
            // Dreambound Disciple (M1-W4a) — the flag is consumed at the
            // next hero-power activation (rules.rs)
            state.make_mut().players[owner.index()].next_hero_power_free = true;
        }
        CardEffect::SetMurlocSummonBuff => {
            // Dive the Golakka Depths (M2-W2) — the repeatable quest
            // reward: permanently, Murlocs the owner summons gain +1/+1
            // (the friendly-summon hook in rules.rs reads the flag)
            state.make_mut().players[owner.index()].murloc_summon_buff = true;
        }
        CardEffect::SetDealExact2Bonus => {
            // Gorishi Colossus (M2-W2) — the battlecry: permanently,
            // whenever the owner deals exactly 2 damage to an enemy, deal
            // 2 more (the damage hook in rules.rs reads the flag)
            state.make_mut().players[owner.index()].deal_exact_2_bonus = true;
        }
        CardEffect::RestoreHealthAndGetDruidSpells { amount, count } => {
            // Photosynthesis (M1-W4a) — heal the hero and gather Druid spells
            resolve_restore_health(
                state,
                queue,
                owner,
                amount,
                EffectTarget::FriendlyHero,
                None,
            );
            let spells: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::DRUID_CLASSIC
                    .iter()
                    .filter(|c| c.card_type == CardType::Spell)
                    .collect();
            if spells.is_empty() {
                return;
            }
            for _ in 0..count {
                let idx = state.rng_mut().next_usize(spells.len());
                add_card_to_hand(state, owner, spells[idx]);
            }
        }
        CardEffect::GainManaCrystalBoth { count } => {
            // Tranquil Treant (M1-W4a) — both players gain an empty crystal
            for p in [owner, owner.opponent()] {
                let inner = state.make_mut();
                inner.players[p.index()].mana_crystals =
                    (inner.players[p.index()].mana_crystals + count).min(10);
            }
        }
        CardEffect::TransformNeutralDeckToDruid => {
            // Envoy of the Glade (M1-W4a) — Neutral (not a class card) deck
            // cards become random Druid ones (the Druid class list is the
            // replacement pool; per-card positions are kept).
            let druids: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::DRUID_CLASSIC
                    .iter()
                    .filter(|c| c.card_type == CardType::Minion || c.card_type == CardType::Spell)
                    .collect();
            if druids.is_empty() {
                return;
            }
            let deck: SmallList<Entity> = state.world().zones().iter(Zone::Deck, owner).collect();
            for card in deck {
                let is_neutral = state
                    .world()
                    .card_id(card)
                    .and_then(|c| crate::cards::def::card_by_id(c.0))
                    .is_some_and(is_neutral_def);
                if is_neutral {
                    let idx = state.rng_mut().next_usize(druids.len());
                    let def = druids[idx];
                    let world = state.world_mut();
                    world.set_card_id(card, CardId(def.id));
                    world.set_attack(card, Attack(def.attack));
                    world.set_health(card, Health(def.health));
                    world.set_cost(card, Cost(def.cost));
                }
            }
        }
        CardEffect::AddMoonfireAndStarfireWithSpellDamage => {
            // Stellar Balance (M1-W4a) — Moonfire + Starfire with Spell
            // Damage +1 on each (the spell entity's own Spell Damage applies
            // when it is later cast)
            // The db's classic Moonfire (DRUID_011) / Starfire (DRUID_006)
            // predate the CORE_EX1_* rename — they are the same cards.
            for id in ["DRUID_011", "DRUID_006"] {
                if let Some(def) = crate::cards::def::card_by_id(id) {
                    if let Some(added) = add_random_minion_to_hand(state, owner, def) {
                        state
                            .world_mut()
                            .set_spell_damage(added, crate::core::component::SpellDamage(1));
                    }
                }
            }
        }
        CardEffect::BuffAnotherRandomFriendlyDragon { attack, health } => {
            // Petal Peddler (M1-W4a) — another random friendly Dragon
            let dragons: SmallList<Entity> = collect_friendly_minions(state, owner)
                .into_iter()
                .filter(|&e| {
                    e != source
                        && state
                            .world()
                            .has_race(e, crate::core::component::Race::Dragon)
                })
                .collect();
            if let Some(t) = select_target(None, &dragons, state.rng_mut()) {
                state.world_mut().add_enchantment(
                    t,
                    Enchantment {
                        attack,
                        health,
                        cost: 0,
                        expiry: EnchantmentExpiry::Permanent,
                    },
                );
            }
        }
        CardEffect::ReduceRightmostHandCardCost { reduction } => {
            // Nightmare Dragonkin (M1-W4a) — the right-most hand card (the
            // last in hand zone order) costs less
            if let Some(card) = state.world().zones().iter(Zone::Hand, owner).last() {
                let world = state.world_mut();
                let cur = world.effective_cost(card).unwrap_or(Cost(0));
                world.set_cost(card, Cost((cur.0 - reduction as i32).max(0)));
            }
        }
        CardEffect::ResurrectDeathrattleMinionCostLE { cost } => {
            // Ravenous Felhunter (M1-W4a) — a random friendly Deathrattle
            // minion that costs at most the threshold comes back as a copy
            let fallen: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Graveyard, owner)
                .filter(|&e| {
                    state.world().card_type(e) == Some(CardType::Minion)
                        && state.world().cost(e).is_some_and(|c| c.0 <= cost as i32)
                        && has_deathrattle_def(state, e)
                })
                .collect();
            if let Some(fallen) = select_target(None, &fallen, state.rng_mut()) {
                if let Some(def) = state
                    .world()
                    .card_id(fallen)
                    .and_then(|c| crate::cards::def::card_by_id(c.0))
                {
                    let _ = resolve_summon(state, queue, source, owner, def.id);
                }
            }
        }
        CardEffect::ResurrectDeathrattleMinionCostGE { cost } => {
            // Ferocious Felbat (M1-W4a) — like the Felhunter but costing at
            // least the threshold and excluding the dying minion itself
            // ("a different friendly Deathrattle minion")
            let fallen: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Graveyard, owner)
                .filter(|&e| {
                    e != source
                        && state.world().card_type(e) == Some(CardType::Minion)
                        && state.world().cost(e).is_some_and(|c| c.0 >= cost as i32)
                        && has_deathrattle_def(state, e)
                })
                .collect();
            if let Some(fallen) = select_target(None, &fallen, state.rng_mut()) {
                if let Some(def) = state
                    .world()
                    .card_id(fallen)
                    .and_then(|c| crate::cards::def::card_by_id(c.0))
                {
                    let _ = resolve_summon(state, queue, source, owner, def.id);
                }
            }
        }
        CardEffect::GainArmorPerWisp { base } => {
            // Merry Moonkin (M1-W4a) — armor plus one per friendly Wisp (the
            // EDR_851t token)
            let wisps = collect_friendly_minions(state, owner)
                .into_iter()
                .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "EDR_851t"))
                .count();
            let inner = state.make_mut();
            inner.players[owner.index()].armor += base + wisps as i32;
        }
        CardEffect::DamageMinionScaledByFallen { base } => {
            // Starsurge (M1-W4a) — plus one per friendly minion that died
            // this game (the graveyard IS the death record)
            let fallen = state
                .world()
                .zones()
                .iter(Zone::Graveyard, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .count() as i32;
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            if let Some(t) = select_target(explicit_target, &all, state.rng_mut()) {
                queue.push(Event::DamageDealt {
                    source,
                    target: t,
                    amount: base + fallen,
                });
            }
        }
        CardEffect::GrantHeroDivineShield => {
            // Curious Cumulus (M1-W4a) — the hero gains Divine Shield
            let hero = state.player(owner).hero;
            state.world_mut().set_divine_shield(hero, DivineShield);
        }
        CardEffect::RestoreBothHeroes { amount } => {
            // Critter Caretaker (M1-W4a) — both heroes heal
            for p in [owner, owner.opponent()] {
                let hero = state.player(p).hero;
                heal_char(state.world_mut(), hero, amount);
            }
        }
        CardEffect::AddSelfToDeckBottomCost { cost } => {
            // Meadowstrider (M1-W4a) — a copy goes to the deck's bottom (the
            // end of the draw order) costing (1)
            if let Some(def) = state
                .world()
                .card_id(source)
                .and_then(|c| crate::cards::def::card_by_id(c.0))
            {
                let world = state.world_mut();
                let e = crate::cards::spawn_card_from_def(world, owner, def);
                world.set_cost(e, Cost(cost as i32));
                world.set_zone(e, Zone::Deck);
                world.zones_mut().insert(Zone::Deck, owner, e);
            }
        }
        CardEffect::SummonCopyOfRandomFriendlyDragon => {
            // The Clutch of Corruption Egg's hatch (M1-W4a) — a copy of a
            // random friendly Dragon (the chosen identity is not remembered,
            // §14.3)
            let dragons: SmallList<Entity> = collect_friendly_minions(state, owner)
                .into_iter()
                .filter(|&e| {
                    state
                        .world()
                        .has_race(e, crate::core::component::Race::Dragon)
                })
                .collect();
            if let Some(dragon) = select_target(None, &dragons, state.rng_mut()) {
                if let Some(def) = state
                    .world()
                    .card_id(dragon)
                    .and_then(|c| crate::cards::def::card_by_id(c.0))
                {
                    let _ = resolve_summon(state, queue, source, owner, def.id);
                }
            }
        }
        CardEffect::GainHealthIfHeroPowerUsed { amount } => {
            // Barkshield Sentinel (M1-W4a) — the hero-power check runs at
            // the end of the turn instead of at the hero-power use (no
            // hero-power trigger event, §14.3)
            let hero = state.player(owner).hero;
            if state.world().hero_power_used(hero).is_some_and(|u| u.0) {
                state.world_mut().add_enchantment(
                    source,
                    Enchantment {
                        attack: 0,
                        health: amount,
                        cost: 0,
                        expiry: EnchantmentExpiry::Permanent,
                    },
                );
            }
        }
        CardEffect::AttackRandomEnemyMinionExcess => {
            // Briarspawn Drake (M1-W4a) — attack a random enemy minion; a
            // kill carries the excess damage to the enemy hero (direct
            // damage model, like the excess-damage family)
            let enemies = collect_enemy_minions(state, owner, Some(source));
            if enemies.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(enemies.len());
            let target = enemies[idx];
            let atk = state.world().effective_attack(source).map_or(0, |a| a.0);
            queue.push(Event::DamageDealt {
                source,
                target,
                amount: atk,
            });
            let dies = state.world().divine_shield(target).is_none()
                && state
                    .world()
                    .effective_health(target)
                    .is_some_and(|h| h.0 - atk <= 0);
            if dies {
                let hp = state.world().effective_health(target).map_or(0, |h| h.0);
                let excess = (atk - hp).max(0);
                if excess > 0 {
                    let hero = state.player(owner.opponent()).hero;
                    queue.push(Event::DamageDealt {
                        source,
                        target: hero,
                        amount: excess,
                    });
                }
            }
        }
        CardEffect::SplashHeroAttackToRandomEnemy => {
            // Defiled Spear (M1-W4a) — the hero's Attack splashes onto a
            // random enemy other than the attacked minion
            let hero = state.player(owner).hero;
            // The hero's Attack includes the equipped weapon's (like the
            // swing itself) — effective_attack on the hero reads the base
            // Attack component, which is 0 for a hero with a weapon.
            let atk = crate::engine::rules::compute_attacker_damage(state, hero);
            let candidates: SmallList<Entity> = collect_enemy_characters(state, owner, None)
                .into_iter()
                .filter(|&e| Some(e) != event_subject)
                .collect();
            if let Some(t) = select_target(None, &candidates, state.rng_mut()) {
                queue.push(Event::DamageDealt {
                    source: hero,
                    target: t,
                    amount: atk,
                });
            }
        }
        CardEffect::GainDeadMinionAttack => {
            // Scavenging Flytrap (M1-W4a) — gain the just-died minion's
            // Attack (the graveyard move wiped enchantments, so this is the
            // base Attack — §14.3)
            if let Some(subject) = event_subject {
                let atk = state.world().effective_attack(subject).map_or(0, |a| a.0);
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
        }
        CardEffect::DrawIfMinionPlayedBefore => {
            // Twisted Webweaver (M1-W4a) — the just-played minion was played
            // earlier this game (the log is pushed before the CardPlayed
            // triggers fire, so a count of 2+ means a repeat)
            if let Some(subject) = event_subject {
                let is_minion = state.world().card_type(subject) == Some(CardType::Minion);
                let repeats = state
                    .world()
                    .card_id(subject)
                    .map(|c| {
                        state
                            .player(owner)
                            .played_minion_ids
                            .iter()
                            .filter(|id| id.as_str() == c.0)
                            .count()
                    })
                    .unwrap_or(0);
                if is_minion && repeats >= 2 {
                    draw_card(state, queue, owner);
                }
            }
        }
        CardEffect::GrantRandomBonusEffect => {
            // Dreambound Raptor (M1-W4a) — a random keyword from the
            // approximated Bonus Effect pool (Taunt / Divine Shield /
            // Poisonous / Windfury / Elusive / Stealth — §14.3)
            if let Some(subject) = event_subject {
                if state.world().card_type(subject) != Some(CardType::Minion) {
                    return;
                }
                let pick = state.rng_mut().next_usize(6);
                let world = state.world_mut();
                match pick {
                    0 => world.set_taunt(subject, Taunt),
                    1 => world.set_divine_shield(subject, DivineShield),
                    2 => world.set_poison(subject, Poison),
                    3 => world.set_windfury(subject, Windfury),
                    4 => world.set_elusive(subject, Elusive),
                    _ => world.set_stealth(subject, Stealth),
                }
            }
        }
        CardEffect::YseraEmeraldAspect => {
            // Ysera, Emerald Aspect (M1-W4b) — the composite Start-of-Game +
            // battlecry effect, resolved at play time (registered
            // simplification, §14.4: the engine has no StartOfGame event, so
            // both players' +5 maximum Mana applies when Ysera is played and
            // fills the new crystals; the battlecry's +3 Mana Crystals then
            // stack on top). Both players get the +5 (a Start-of-Game effect
            // is global), the +3 belongs to the owner only.
            for p in [owner, owner.opponent()] {
                let inner = state.make_mut();
                let pl = &mut inner.players[p.index()];
                pl.mana_crystals = (pl.mana_crystals + 5).min(10);
                pl.current_mana = (pl.current_mana + 5).min(10);
            }
            let inner = state.make_mut();
            let pl = &mut inner.players[owner.index()];
            pl.mana_crystals = (pl.mana_crystals + 3).min(10);
            pl.current_mana = (pl.current_mana + 3).min(10);
        }
        CardEffect::ResurrectAllDifferentFriendlyCostGE { cost } => {
            // Merithra (M1-W4b) — resurrect ALL different friendly minions
            // that cost at least `cost` and died this game. "Different" is
            // deduped by card ID (first graveyard instance wins — graveyard
            // order is death order); the graveyard IS the death record.
            let mut seen: Vec<String> = Vec::new();
            let mut fallen: SmallList<Entity> = SmallList::new();
            for e in state.world().zones().iter(Zone::Graveyard, owner) {
                if state.world().card_type(e) != Some(CardType::Minion) {
                    continue;
                }
                if !state.world().cost(e).is_some_and(|c| c.0 >= cost as i32) {
                    continue;
                }
                let Some(cid) = state.world().card_id(e) else {
                    continue;
                };
                if seen.iter().any(|s| s == cid.0) {
                    continue;
                }
                seen.push(cid.0.to_string());
                fallen.push(e);
            }
            for e in fallen {
                if let Some(def) = state
                    .world()
                    .card_id(e)
                    .and_then(|c| crate::cards::def::card_by_id(c.0))
                {
                    let _ = resolve_summon(state, queue, source, owner, def.id);
                }
            }
        }
        CardEffect::CastHighestCostSpellFromHand => {
            // Ursol (M1-W4b) — cast the highest-Cost spell in hand (ties go
            // to the leftmost). Registered simplification (§14.4): the real
            // card plays the spell as a 3-turn aura, approximated as an
            // immediate normal cast — no SpellCast event fires (the cast is
            // not "your" spell play, so after-cast triggers stay quiet).
            let spells: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Spell))
                .collect();
            if spells.is_empty() {
                return;
            }
            let Some(best) = spells.iter().copied().max_by_key(|&e| {
                let cost = state.world().effective_cost(e).unwrap_or(Cost(0)).0;
                // The leftmost-first tiebreaker: negate the hand index
                let idx = state
                    .world()
                    .zones()
                    .iter(Zone::Hand, owner)
                    .position(|x| x == e)
                    .unwrap_or(0);
                (cost, std::cmp::Reverse(idx))
            }) else {
                return;
            };
            if let Some(effect) = state.world().battlecry(best).map(|b| b.0) {
                resolve_effect(state, queue, best, owner, effect, None, None);
            }
            let _ = state.world_mut().move_to_zone(best, Zone::Graveyard);
        }
        CardEffect::IncrementOmenAttack => {
            // Omen (M1-W4b) — the per-player attack counter feeding the
            // deathrattle's "Improves" (registered interpretation, §14.4:
            // the official per-minion enchantment is approximated as a
            // per-player counter). Only Omen's own attacks count.
            if event_subject != Some(source) {
                return;
            }
            let inner = state.make_mut();
            inner.players[owner.index()].omen_attacks += 1;
        }
        CardEffect::OmenDeathrattle => {
            // Omen (M1-W4b) — Deal 1 damage to all enemies, plus 1 for each
            // attack Omen has made this game (the counter is incremented by
            // IncrementOmenAttack on the Attacked trigger).
            let mut enemies: SmallList<Entity> = collect_all_enemy_minions(state, owner);
            enemies.push(state.player(owner.opponent()).hero);
            let atk = state.player(owner).omen_attacks as i32;
            for e in &enemies {
                queue.push(Event::DamageDealt {
                    source,
                    target: *e,
                    amount: 1 + atk,
                });
            }
        }
        CardEffect::SplitDamageAmongAllEnemiesIfFallen { amount, threshold } => {
            // Aessina (M1-W4b) — if at least `threshold` friendly minions
            // have died this game (the graveyard IS the death record, the
            // DamageMinionScaledByFallen precedent), deal `amount` damage
            // randomly split among all enemies — `amount` independent
            // 1-damage pings (each ping rolls a random enemy).
            let fallen = state
                .world()
                .zones()
                .iter(Zone::Graveyard, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .count() as u8;
            if fallen < threshold {
                return;
            }
            let mut enemies: SmallList<Entity> = collect_all_enemy_minions(state, owner);
            enemies.push(state.player(owner.opponent()).hero);
            if enemies.is_empty() {
                return;
            }
            for _ in 0..amount {
                let idx = state.rng_mut().next_usize(enemies.len());
                queue.push(Event::DamageDealt {
                    source,
                    target: enemies[idx],
                    amount: 1,
                });
            }
        }
        CardEffect::NextSpellsCastTwice { count } => {
            // Tyrande (M1-W4b) — the next `count` spells the owner casts are
            // cast twice. The re-cast happens at the spell play path
            // (rules.rs — normal spells and choose-one branches); the flag
            // is consumed there. Registered timing simplification (§14.4):
            // the official wording is "the next 3 spells you cast cast
            // twice" — the countdown here ticks per resolution, not per
            // "cast" event, but each double-cast still counts once.
            let inner = state.make_mut();
            inner.players[owner.index()].spells_cast_twice_pending = count as u32;
        }
        CardEffect::SummonRandomDragonPerSelfDeath => {
            // Ysondre (M1-W4b) — summon a random Dragon for each time this
            // has died this game. The graveyard IS the death record: every
            // death leaves an EDR_465 entity behind (the currently-dying
            // instance is already in the graveyard when the deathrattle
            // resolves, so the count includes this death — official).
            let deaths = state
                .world()
                .zones()
                .iter(Zone::Graveyard, owner)
                .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "EDR_465"))
                .count();
            for _ in 0..deaths {
                if let Some(def) =
                    crate::cards::pool::random_card(state.rng_mut(), RandomPool::Dragon)
                {
                    let _ = resolve_summon(state, queue, source, owner, def.id);
                }
            }
        }
        CardEffect::GainArmorAndSelfAttack { armor, attack } => {
            // Tortolla (M1-W4b) — after this takes damage: the OWNER's hero
            // gains armor and the minion gains Attack (the ThisMinionDamaged
            // trigger carries the subject pin, so no extra check needed).
            let inner = state.make_mut();
            inner.players[owner.index()].armor += armor;
            let world = state.world_mut();
            let base = world.attack(source).unwrap_or(Attack(0));
            world.set_attack(source, Attack(base.0 + attack));
        }
        CardEffect::NextCardCostsZero => {
            // Agamaggan (M1-W4b) — the next card the owner plays costs (0)
            // (registered simplification §14.4: the official effect sets the
            // next card's cost to the opponent's Health; approximated as 0).
            let inner = state.make_mut();
            inner.players[owner.index()].next_card_costs_zero = true;
        }
        CardEffect::TransformHandMinionsToRandomDemons => {
            // Alara'shi (M1-W4b) — every minion in hand transforms into a
            // random Demon, KEEPING its Attack, Health and Cost. Base stats
            // are read before the effect clear (clear_minion_effects keeps
            // base components but drops keywords/enchantments — the real
            // transform replaces the card identity and keywords).
            let hand: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            for m in &hand {
                let Some(def) = crate::cards::pool::random_card(state.rng_mut(), RandomPool::Demon)
                else {
                    continue;
                };
                let world = state.world_mut();
                let atk = world.attack(*m).unwrap_or(Attack(0)).0;
                let hp = world.health(*m).unwrap_or(Health(1)).0;
                let cst = world.cost(*m).unwrap_or(Cost(0)).0;
                crate::cards::clear_minion_effects(world, *m);
                world.set_card_id(*m, CardId(def.id));
                world.set_race(*m, crate::core::component::Race::Demon);
                world.set_attack(*m, Attack(atk));
                world.set_health(*m, Health(hp));
                world.set_cost(*m, Cost(cst));
                world.set_attacks_used(*m, AttacksUsed(0));
                if def.taunt {
                    world.set_taunt(*m, Taunt);
                }
                if def.stealth {
                    world.set_stealth(*m, Stealth);
                }
                if def.divine_shield {
                    world.set_divine_shield(*m, DivineShield);
                }
                if def.windfury {
                    world.set_windfury(*m, Windfury);
                }
                if def.charge {
                    world.set_charge(*m, Charge);
                }
            }
        }
        CardEffect::DiscoverSpellKeepOrTop => {
            // Q'onzu (M1-W4b) — Discover a spell, then choose to keep it or
            // place it on top of the opponent's deck. The Discover is
            // simplified to a random spell from the full database (the
            // standing Discover→random debt, §14.4); the keep/top decision
            // surfaces as the QonzuKeepOrTop choice (the spell entity is the
            // pending choice's card — ChoiceResolved moves it or leaves it).
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
            let before = state.world().zones().len(Zone::Hand, owner);
            add_card_to_hand(state, owner, spell);
            if state.world().zones().len(Zone::Hand, owner) == before {
                return; // full hand — nothing was added, no choice to make
            }
            // The added spell is the last entity in the owner's hand.
            let added = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .last()
                .expect("hand grew, so it is non-empty");
            state.set_pending_choice(
                crate::core::state::ChoiceKind::QonzuKeepOrTop,
                added,
                vec![
                    String::from("Keep the spell"),
                    String::from("Put it on top of the opponent's deck"),
                ],
                Vec::new(),
            );
        }
        CardEffect::DiscardRandomEnemyHandCard => {
            // Renferal, the Malignant (M1-W4b) — the enemy discards a random
            // hand card (registered simplification §14.4: the real trap
            // returns the card after a turn and escalates per play; the
            // discard is the light approximation).
            let enemy = owner.opponent();
            let hand: SmallList<Entity> = state.world().zones().iter(Zone::Hand, enemy).collect();
            if hand.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(hand.len());
            let _ = state.world_mut().move_to_zone(hand[idx], Zone::Graveyard);
        }
        CardEffect::FillHandWithEnemyDeckCopies { reduction } => {
            // Ashamane (M1-W4b) — fill the owner's hand with copies of cards
            // in the opponent's deck; the copies cost (3) less. The opponent's
            // deck IS the pool (pool-open card — registered in POOL_OPEN_CARDS,
            // the Nightmare Fuel precedent); sampling is with replacement
            // (official: random copies, duplicates possible). The hand-size
            // cap (F-A11) bounds the fill.
            let enemy = owner.opponent();
            let candidates: Vec<Entity> = state.world().zones().iter(Zone::Deck, enemy).collect();
            if candidates.is_empty() {
                return;
            }
            while state.world().zones().len(Zone::Hand, owner) < MAX_HAND_SIZE {
                let idx = state.rng_mut().next_usize(candidates.len());
                let src = candidates[idx];
                let Some(card_id) = state.world().card_id(src) else {
                    continue;
                };
                let Some(def) = crate::cards::def::card_by_id(card_id.0) else {
                    continue;
                };
                let e = crate::cards::spawn_card_from_def(state.world_mut(), owner, def);
                let base = state.world().cost(e).unwrap_or(Cost(0)).0;
                state
                    .world_mut()
                    .set_cost(e, Cost((base - reduction as i32).max(0)));
                state.world_mut().set_zone(e, Zone::Hand);
                state.world_mut().zones_mut().insert(Zone::Hand, owner, e);
            }
        }
        CardEffect::SummonBeetles { count } => {
            // Nythendra (M1-W4b) — deathrattle: summon `count` 1/1 Beetles
            // (registered simplification §14.4: the real card splits into
            // Beetles that reassemble at the start of your turn; only the
            // Beetle summon is modeled).
            for _ in 0..count {
                let _ = resolve_summon(state, queue, source, owner, "EDR_818t");
            }
        }
        CardEffect::UrsocBattlecry => {
            // Ursoc (M1-W4b) — attack all other minions (both sides, play
            // order — friendly board first, then the enemy board). Each
            // attack is a synchronous damage application (the Death step runs
            // at queue boundaries, so damage is applied immediately and the
            // dead targets are recorded in pending_deaths; the deaths caused
            // by THIS battlecry are the tail of that list — the entry
            // snapshot excludes any earlier deaths, though the battlecry runs
            // in a clean context in practice).
            let atk = state.world().attack(source).unwrap_or(Attack(0)).0;
            let mut targets: SmallList<Entity> = collect_friendly_minions(state, owner)
                .into_iter()
                .filter(|&e| e != source)
                .collect();
            targets.extend(collect_all_enemy_minions(state, owner));
            let before = state.pending_deaths().len();
            for e in &targets {
                let _ = crate::engine::rules::apply_event(
                    state,
                    Event::DamageDealt {
                        source,
                        target: *e,
                        amount: atk,
                    },
                    queue,
                );
            }
            let killed: SmallList<String, 16> = state
                .pending_deaths()
                .iter()
                .skip(before)
                .filter_map(|&e| state.world().card_id(e).map(|c| c.0.to_string()))
                .collect();
            if !killed.is_empty() {
                let inner = state.make_mut();
                inner.players[owner.index()].ursoc_killed_ids = killed.into_iter().collect();
            }
        }
        CardEffect::UrsocDeathrattle => {
            // Ursoc (M1-W4b) — resurrect every minion the battlecry killed
            // (the recorded card IDs are consumed here). Cascade deaths from
            // the resurrections are NOT recorded — faithful to the official
            // "minions this killed" wording.
            let ids = std::mem::take(&mut state.make_mut().players[owner.index()].ursoc_killed_ids);
            for id in ids {
                let _ = resolve_summon(state, queue, source, owner, &id);
            }
        }
        CardEffect::GainStatsAllOtherFriendlyMinions { attack, health } => {
            // Forest Lord Cenarius (M1-W4b) — choose-one option A: all OTHER
            // friendly minions gain +1/+3 (the source is excluded; the
            // enchantment is permanent like GainStatsAndTauntAllFriendly).
            for m in collect_friendly_minions(state, owner) {
                if m != source {
                    state.world_mut().add_enchantment(
                        m,
                        Enchantment {
                            attack,
                            health,
                            cost: 0,
                            expiry: EnchantmentExpiry::Permanent,
                        },
                    );
                }
            }
        }
        CardEffect::SummonRandomAnimalCompanion => {
            // Broll Bearmantle (M1-W4b) — after you cast a spell, summon a
            // random Animal Companion (the HUNTER_023 companion pool).
            if let Some(def) = crate::cards::pool::random_from_pool(
                crate::cards::pool::COMPANION_POOL,
                state.rng_mut(),
            ) {
                let _ = resolve_summon(state, queue, source, owner, def.id);
            }
        }
        CardEffect::AddAllDreamCards => {
            // Shaladrassil (M1-W4b) — add all five Dream cards to hand (the
            // DREAM_POOL; the hand-size cap F-A11 drops any overflow).
            // Registered simplification (§14.4): the corruption clause (the
            // cards corrupt if you play a higher-Cost card while holding
            // Shaladrassil) is not modeled.
            for id in crate::cards::pool::DREAM_POOL {
                if let Some(def) = crate::cards::def::card_by_id(id) {
                    add_card_to_hand(state, owner, def);
                }
            }
        }
        CardEffect::CardsCostOneThisGame => {
            // Aviana, Elune's Chosen (M1-W4b) — all the owner's cards cost
            // (1) for the rest of the game (registered simplification §14.4:
            // the real 3-turn lunar cycle with the full-moon effect is
            // approximated as an immediate game-long effect).
            let inner = state.make_mut();
            inner.players[owner.index()].cards_cost_1 = true;
        }
        // ----------------------------------------------------------------
        // 2025–2026 expansions M1-W5 (exp_edr_w5) — the Embers of the
        // World Tree miniset (see the design brief; simplifications
        // registered in fidelity-debt §14.5).
        // ----------------------------------------------------------------
        CardEffect::GainStatsIfHeroPowerUsed { attack, health } => {
            // Spirit of the Kaldorei (M1-W5) — +N/+M while the owner used
            // their hero power this turn (the Barkshield Sentinel W4a
            // pattern: the check runs at play time).
            let hero = state.player(owner).hero;
            if state.world().hero_power_used(hero).is_some_and(|u| u.0) {
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
        CardEffect::GiveMinionStatsRushIfHeroPowerUsed { attack, health } => {
            // Charred Chameleon (M1-W5) — the friendly minion target gains
            // +N/+M and Rush only while the owner used their hero power
            // this turn.
            let hero = state.player(owner).hero;
            if !state.world().hero_power_used(hero).is_some_and(|u| u.0) {
                return;
            }
            let minions: SmallList<Entity> = collect_friendly_minions(state, owner);
            let Some(target) = select_target(explicit_target, &minions, state.rng_mut()) else {
                return;
            };
            state.world_mut().add_enchantment(
                target,
                Enchantment {
                    attack,
                    health,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
            state.world_mut().set_rush(target, Rush);
        }
        CardEffect::DrawIfImbuedTwice { count } => {
            // Petal Picker (M1-W5) — draw while the owner has Imbued at
            // least twice (the W1 Dreamweaver threshold pattern).
            if state.player(owner).imbue_count >= 2 {
                for _ in 0..count {
                    draw_card(state, queue, owner);
                }
            }
        }
        CardEffect::DealDamageToAllEnemyMinions { damage } => {
            // Avatar of Destruction (M1-W5) deathrattle — damage to every
            // enemy minion (the hero is excluded).
            resolve_deal_damage(
                state,
                queue,
                source,
                owner,
                damage,
                EffectTarget::AllEnemyMinions,
                None,
            );
        }
        CardEffect::DiscoverWithDarkGiftCostReduction { reduction } => {
            // Cremate (M1-W5) — a random minion to hand, given a random
            // dark gift and costing the reduction less (Discover
            // simplified to random, §14.5).
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| {
                        c.card_type == CardType::Minion && crate::cards::pool::in_active_window(c)
                    })
                    .collect();
            if candidates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            if let Some(added) = add_random_minion_to_hand(state, owner, candidates[idx]) {
                let gift = random_dark_gift(state.rng_mut());
                apply_dark_gift(state, added, gift, owner);
                let world = state.world_mut();
                let cur = world.effective_cost(added).unwrap_or(Cost(0));
                world.set_cost(added, Cost((cur.0 - i32::from(reduction)).max(0)));
            }
        }
        CardEffect::SummonBroodlingsIfHoldingGift => {
            // Frostburn Matriarch (M1-W5) — while the owner holds a hand
            // minion with a dark gift, summon two 4/4 Taunt Dragons.
            if holding_minion_with_dark_gift(state, owner) {
                for _ in 0..2 {
                    let _ = resolve_summon(state, queue, source, owner, "FIR_901t");
                }
            }
        }
        CardEffect::FelfireBlazeTrigger { damage } => {
            // Felfire Blaze (M1-W5) spell trigger — destroy this and deal
            // damage to all enemies (the Fel-spell filter is unmodeled —
            // any friendly spell triggers it, §14.5). The self-destroy
            // follows the destroy convention: damage equal to its health
            // routed through the death pipeline.
            let hp = state.world().effective_health(source).unwrap_or(Health(1));
            queue.push(Event::DamageDealt {
                source,
                target: source,
                amount: hp.0.max(1),
            });
            for enemy in collect_all_enemy_characters(state, owner) {
                queue.push(Event::DamageDealt {
                    source,
                    target: enemy,
                    amount: damage,
                });
            }
        }
        CardEffect::BuffFriendlyMinionsDiscardBonus {
            attack,
            health,
            bonus_attack,
            bonus_health,
        } => {
            // Overheat (M1-W5) — all friendly minions +N/+M; discarding a
            // random hand spell grants the bonus instead (the Nature-spell
            // filter is unmodeled, §14.5).
            let spells: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Spell))
                .collect();
            let discarded = if spells.is_empty() {
                false
            } else {
                let idx = state.rng_mut().next_usize(spells.len());
                let _ = state.world_mut().move_to_zone(spells[idx], Zone::Graveyard);
                true
            };
            let (a, h) = if discarded {
                (attack + bonus_attack, health + bonus_health)
            } else {
                (attack, health)
            };
            for m in collect_friendly_minions(state, owner) {
                state.world_mut().add_enchantment(
                    m,
                    Enchantment {
                        attack: a,
                        health: h,
                        cost: 0,
                        expiry: EnchantmentExpiry::Permanent,
                    },
                );
            }
        }
        CardEffect::AmirdrassilActivate => {
            // Amirdrassil (M1-W5) location activation — summon a random
            // 1-Cost minion, gain 1 Armor, draw a card and refresh 1 Mana
            // (the "Improves each use!" escalation is unmodeled, §14.5).
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| {
                        c.card_type == CardType::Minion
                            && c.cost == 1
                            && crate::cards::pool::in_active_window(c)
                    })
                    .collect();
            if !candidates.is_empty() {
                let idx = state.rng_mut().next_usize(candidates.len());
                let _ = resolve_summon(state, queue, source, owner, candidates[idx].id);
            }
            {
                let inner = state.make_mut();
                let p = &mut inner.players[owner.index()];
                p.armor += 1;
                // Refresh 1 Mana — a top-up capped at the current crystal count
                p.current_mana = (p.current_mana + 1).min(p.mana_crystals);
            }
            draw_card(state, queue, owner);
        }
        CardEffect::InfernoHeraldTrigger { reduction } => {
            // Inferno Herald (M1-W5) spell trigger — a random Elemental to
            // hand, costing the reduction less (the Fire-spell filter is
            // unmodeled, §14.5).
            if let Some(def) = crate::cards::pool::random_card(
                state.rng_mut(),
                crate::core::effect::RandomPool::Elemental,
            ) {
                if let Some(added) = add_random_minion_to_hand(state, owner, def) {
                    let world = state.world_mut();
                    let cur = world.effective_cost(added).unwrap_or(Cost(0));
                    world.set_cost(added, Cost((cur.0 - i32::from(reduction)).max(0)));
                }
            }
        }
        CardEffect::BuffMinionReturnIfSpellsCast {
            attack,
            health,
            threshold,
        } => {
            // Light of the New Moon (M1-W5) — give a minion +N/+M; the
            // return-to-hand counter is approximated by the player's spell
            // total (the W4a Wish of the New Moon pattern — the current
            // cast counts, §14.5). The returned card is a fresh copy; the
            // Light of the Full Moon upgrade step is unmodeled.
            let mut minions = collect_friendly_minions(state, owner);
            minions.extend(collect_all_enemy_minions(state, owner));
            let Some(target) = select_target(explicit_target, &minions, state.rng_mut()) else {
                return;
            };
            state.world_mut().add_enchantment(
                target,
                Enchantment {
                    attack,
                    health,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
            if state.player(owner).spells_cast_total + 1 >= threshold {
                if let Some(def) = crate::cards::def::card_by_id("FIR_918") {
                    add_card_to_hand(state, owner, def);
                }
            }
        }
        CardEffect::GainWeaponAttackIfHoldingGift { amount } => {
            // Cindersword (M1-W5) — the weapon gains Attack while the owner
            // holds a hand minion with a dark gift.
            if holding_minion_with_dark_gift(state, owner) {
                resolve_buff_weapon(state, owner, amount, 0);
            }
        }
        CardEffect::DamageRandomEnemyMinionHoldingCostGE {
            base,
            upgraded,
            threshold,
        } => {
            // Flames of the Firelord (M1-W5) — damage to a random enemy
            // minion, upgraded while the owner holds a card costing the
            // threshold or more.
            let holding = state.world().zones().iter(Zone::Hand, owner).any(|e| {
                state
                    .world()
                    .effective_cost(e)
                    .is_some_and(|c| c.0 >= threshold)
            });
            let damage = if holding { upgraded } else { base };
            resolve_deal_damage(
                state,
                queue,
                source,
                owner,
                damage,
                EffectTarget::AnyEnemyMinion,
                None,
            );
        }
        CardEffect::DiscoverComboBattlecryStealthWithDarkGift => {
            // Smoke Bomb (M1-W5) — a random Combo, Battlecry or Stealth
            // minion to hand, given a random dark gift (Discover
            // simplified to random, §14.5).
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| {
                        c.card_type == CardType::Minion
                            && (c.combo_effect.is_some() || c.battlecry.is_some() || c.stealth)
                            && crate::cards::pool::in_active_window(c)
                    })
                    .collect();
            if candidates.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(candidates.len());
            if let Some(added) = add_random_minion_to_hand(state, owner, candidates[idx]) {
                let gift = random_dark_gift(state.rng_mut());
                apply_dark_gift(state, added, gift, owner);
            }
        }
        CardEffect::DiscoverDemonWithDarkGiftCopy => {
            // Shadowflame Stalker (M1-W5) — a random Demon to hand, given a
            // random dark gift, then "get a copy of it": a fresh copy
            // carrying the SAME gift (Discover simplified to random, §14.5).
            if let Some(def) = crate::cards::pool::random_card(
                state.rng_mut(),
                crate::core::effect::RandomPool::Demon,
            ) {
                if let Some(added) = add_random_minion_to_hand(state, owner, def) {
                    let gift = random_dark_gift(state.rng_mut());
                    apply_dark_gift(state, added, gift, owner);
                    let hand_before = state.world().zones().len(Zone::Hand, owner);
                    add_card_to_hand(state, owner, def);
                    let hand: SmallList<Entity> =
                        state.world().zones().iter(Zone::Hand, owner).collect();
                    if let Some(&copy) = hand.iter().nth(hand_before) {
                        apply_dark_gift(state, copy, gift, owner);
                    }
                }
            }
        }
        CardEffect::DiscoverCostCardGainTempMana { cost, mana } => {
            // Emberscarred Whelp (M1-W5) — a random card of the given Cost
            // to hand (Discover simplified to random, §14.5) and Mana
            // Crystals next turn only (a per-player flag spent at the
            // ManaRefill step — the crystal_gain_pending precedent).
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| {
                        c.cost == i32::from(cost) && crate::cards::pool::in_active_window(c)
                    })
                    .collect();
            if !candidates.is_empty() {
                let idx = state.rng_mut().next_usize(candidates.len());
                add_card_to_hand(state, owner, candidates[idx]);
            }
            let inner = state.make_mut();
            inner.players[owner.index()].temp_mana_crystal_pending += i32::from(mana);
        }
        CardEffect::DamageAndDiscoverWarriorWithGift { damage } => {
            // Shadowflame Suffusion (M1-W5) — damage to the target, then a
            // random Warrior minion to hand, given a random dark gift
            // (Discover simplified to random, §14.5).
            resolve_deal_damage(
                state,
                queue,
                source,
                owner,
                damage,
                EffectTarget::AnyEnemy,
                explicit_target,
            );
            if let Some(def) = crate::cards::pool::random_card(
                state.rng_mut(),
                crate::core::effect::RandomPool::WarriorMinion,
            ) {
                if let Some(added) = add_random_minion_to_hand(state, owner, def) {
                    let gift = random_dark_gift(state.rng_mut());
                    apply_dark_gift(state, added, gift, owner);
                }
            }
        }
        CardEffect::ReduceHandCostIfAllDistinct { reduction } => {
            // Zaqali Flamemancer (M1-W5) — when every hand card has a
            // distinct Cost, all hand cards cost the reduction less.
            let hand: Vec<Entity> = state.world().zones().iter(Zone::Hand, owner).collect();
            if hand.is_empty() {
                return;
            }
            let mut costs: Vec<i32> = hand
                .iter()
                .filter_map(|&e| state.world().effective_cost(e))
                .map(|c| c.0)
                .collect();
            costs.sort_unstable();
            if costs.windows(2).any(|w| w[0] == w[1]) {
                return;
            }
            for e in hand {
                let world = state.world_mut();
                let cur = world.effective_cost(e).unwrap_or(Cost(0));
                world.set_cost(e, Cost((cur.0 - i32::from(reduction)).max(0)));
            }
        }
        CardEffect::DrawMinionSummonDivineShieldCopy => {
            // Searing Reflection (M1-W5) — draw the first minion in the
            // deck and summon an 8/8 copy of it with Divine Shield (the
            // copy keeps the drawn card's identity with the FIR_941e1
            // token's 8/8 stats; a full hand burns the draw per F-A11).
            let minions: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Deck, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            let Some(&drawn) = minions.iter().next() else {
                return;
            };
            let hand_before = state.world().zones().len(Zone::Hand, owner);
            let _ = state.world_mut().move_to_zone(drawn, Zone::Hand);
            if state.world().zones().len(Zone::Hand, owner) > hand_before {
                queue.push(Event::CardDrawn {
                    player: owner,
                    card: drawn,
                });
            }
            let Some(def) = state
                .world()
                .card_id(drawn)
                .and_then(|c| crate::cards::def::card_by_id(c.0))
            else {
                return;
            };
            if let Some(copy) = resolve_summon(state, queue, source, owner, def.id) {
                let world = state.world_mut();
                world.set_attack(copy, Attack(8));
                world.set_health(copy, Health(8));
                world.set_divine_shield(copy, DivineShield);
            }
        }
        CardEffect::VolcorossBattlecry => {
            // Volcoross (M1-W5) — spend 10/20/30 Corpses to gain that many
            // stats; the real choose-one is simplified to the largest
            // affordable option (§14.5).
            let have = state.player(owner).corpses;
            let spend = if have >= 30 {
                30
            } else if have >= 20 {
                20
            } else if have >= 10 {
                10
            } else {
                return;
            };
            {
                let inner = state.make_mut();
                inner.players[owner.index()].corpses -= spend;
            }
            // Quest progress (M2-W1): TLC_433 — "Spend 15 Corpses" — the
            // amount is the corpses actually spent.
            crate::engine::quest::progress(
                state,
                queue,
                owner,
                crate::cards::quest::QuestCondition::SpendCorpses,
                spend,
                None,
            );
            state.world_mut().add_enchantment(
                source,
                Enchantment {
                    attack: spend as i32,
                    health: spend as i32,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
        }
        CardEffect::DiscoverSpellReduceHandSpells { reduction } => {
            // Scorchreaver (M1-W5) — reduce every hand spell's Cost by the
            // reduction, then add a random spell to hand, also reduced
            // (the Fel-spell filter is unmodeled, §14.5).
            let spells: Vec<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Spell))
                .collect();
            for e in &spells {
                let world = state.world_mut();
                let cur = world.effective_cost(*e).unwrap_or(Cost(0));
                world.set_cost(*e, Cost((cur.0 - i32::from(reduction)).max(0)));
            }
            if let Some(def) = crate::cards::pool::random_card(
                state.rng_mut(),
                crate::core::effect::RandomPool::Spell,
            ) {
                let hand_before = state.world().zones().len(Zone::Hand, owner);
                add_card_to_hand(state, owner, def);
                let hand: SmallList<Entity> =
                    state.world().zones().iter(Zone::Hand, owner).collect();
                if let Some(&added) = hand.iter().nth(hand_before) {
                    let world = state.world_mut();
                    let cur = world.effective_cost(added).unwrap_or(Cost(0));
                    world.set_cost(added, Cost((cur.0 - i32::from(reduction)).max(0)));
                }
            }
        }
        CardEffect::MagmaHoundSplash => {
            // Magma Hound (M1-W5) — after this attacks a minion and
            // survives, its Attack is dealt split among all enemies (the
            // Augmented Porcupine ping pattern). The trigger fires at
            // attack declaration (the W2 porcupine convention, §14.5) — the
            // pings are queued before the trade damage resolves, so a Hound
            // killed by the retaliation still splashes the pings queued at
            // declaration. The survive check reads effective health at
            // trigger time (the hound has not taken the trade damage yet).
            if !state
                .world()
                .effective_health(source)
                .is_some_and(|h| !h.is_dead())
            {
                return;
            }
            let atk = state
                .world()
                .effective_attack(source)
                .unwrap_or(Attack(0))
                .0;
            if atk <= 0 {
                return;
            }
            let enemies = collect_all_enemy_characters(state, owner);
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
        CardEffect::DamageMinionOwnerDraws { damage } => {
            // Conflagrate (M1-W5) — damage to a minion (either side); its
            // owner draws a card.
            let mut minions = collect_friendly_minions(state, owner);
            minions.extend(collect_all_enemy_minions(state, owner));
            let Some(target) = select_target(explicit_target, &minions, state.rng_mut()) else {
                return;
            };
            queue.push(Event::DamageDealt {
                source,
                target,
                amount: damage,
            });
            let target_owner = state.world().player(target).unwrap_or(owner);
            draw_card(state, queue, target_owner);
        }
        CardEffect::DeathrattleDamageAllEnemiesTurnScaled { base, boosted } => {
            // Tindral Sageswift (M1-W5) — damage to all enemy characters;
            // the boosted amount while it is the opponent's turn.
            let amount = if state.active_player() != owner {
                boosted
            } else {
                base
            };
            resolve_deal_damage(
                state,
                queue,
                source,
                owner,
                amount,
                EffectTarget::AllEnemies,
                None,
            );
        }
        CardEffect::DealDamageSplitAmongAllEnemies { amount } => {
            // Sigil of Cinder (M1-W5) — the damage is dealt as random
            // 1-damage pings across all enemy characters (the "start of
            // your next turn" timing is simplified to immediate, §14.5).
            let enemies = collect_all_enemy_characters(state, owner);
            if enemies.is_empty() {
                return;
            }
            for _ in 0..amount {
                let idx = state.rng_mut().next_usize(enemies.len());
                queue.push(Event::DamageDealt {
                    source,
                    target: enemies[idx],
                    amount: 1,
                });
            }
        }
        CardEffect::CopyLowestCostBeastInHand => {
            // Tending Dragonkin (M1-W5) — add a copy of the lowest-Cost
            // Beast in the hand (ties break toward hand order; a full hand
            // creates nothing, F-A11).
            let beasts: Vec<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter(|&e| {
                    state
                        .world()
                        .has_race(e, crate::core::component::Race::Beast)
                })
                .collect();
            let Some(&lowest) = beasts
                .iter()
                .min_by_key(|&&e| state.world().effective_cost(e).map_or(0, |c| c.0))
            else {
                return;
            };
            copy_card_to_hand(state, lowest, owner);
        }
        CardEffect::GainDivineShieldLifestealIfHoldingSpellGE { cost } => {
            // Ashleaf Pixie (M1-W5) — Divine Shield and Lifesteal while the
            // owner holds a spell costing the threshold or more.
            let holding = state.world().zones().iter(Zone::Hand, owner).any(|e| {
                state.world().card_type(e) == Some(CardType::Spell)
                    && state.world().effective_cost(e).is_some_and(|c| c.0 >= cost)
            });
            if holding {
                state.world_mut().set_divine_shield(source, DivineShield);
                state.world_mut().set_lifesteal(source, Lifesteal);
            }
        }
        CardEffect::GainHeroAttackArmorIfHoldingGift { attack, armor } => {
            // Dragon Turtle (M1-W5) — the hero gains Attack this turn and
            // Armor while the owner holds a hand minion with a dark gift.
            if holding_minion_with_dark_gift(state, owner) {
                resolve_gain_hero_attack(state, owner, attack, armor);
            }
        }
        CardEffect::DamageAndDiscardSpellMore { base, bonus } => {
            // Scorching Winds (M1-W5) — damage to the target; a random hand
            // spell is discarded for the bonus damage on the same target
            // (the Fire-spell filter is unmodeled, §14.5).
            let spells: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Spell))
                .collect();
            let total = if spells.is_empty() {
                base
            } else {
                let idx = state.rng_mut().next_usize(spells.len());
                let _ = state.world_mut().move_to_zone(spells[idx], Zone::Graveyard);
                base + bonus
            };
            resolve_deal_damage(
                state,
                queue,
                source,
                owner,
                total,
                EffectTarget::AnyEnemy,
                explicit_target,
            );
        }
        CardEffect::BuffAllHandMinions { attack, health } => {
            // Keeper of Flame (M1-W5) — all hand minions gain +N/+M (the
            // "destroyed in 3 turns" clause is unmodeled, §14.5).
            let hand: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            for e in hand {
                state.world_mut().add_enchantment(
                    e,
                    Enchantment {
                        attack,
                        health,
                        cost: 0,
                        expiry: EnchantmentExpiry::Permanent,
                    },
                );
            }
        }
        // ============================================================
        // 2025–2026 expansions M2-W3 — the Un'Goro Kindred wave (the
        // activation checks below run inside the arms: the played-type
        // push happened at the CardPlayed path, so `kindred_active(..., 2)`
        // counts the card itself plus an earlier same-type card).
        // ============================================================
        CardEffect::GainRush { target: _ } => {
            // Stormbrewer's Kindred (M2-W3) — give the source Rush
            let candidates: SmallList<Entity> = [source].into_iter().collect();
            if let Some(t) = select_target(explicit_target, &candidates, state.rng_mut()) {
                state.world_mut().set_rush(t, Rush);
            }
        }
        CardEffect::GainImmuneThisTurn { target: _ } => {
            // Whirling Stormdrake's Kindred (M2-W3) — Immune until the end
            // of the turn (the temporary immunity clears in the turn-end
            // wrap-up)
            let candidates: SmallList<Entity> = [source].into_iter().collect();
            if let Some(t) = select_target(explicit_target, &candidates, state.rng_mut()) {
                state.world_mut().set_immune(t, Immune);
            }
        }
        CardEffect::NextMurlocCostsLess { amount } => {
            // Hot Spring Glider's battlecry (M2-W3) — "your next Murloc
            // costs (1) less" (one-time, consumed by the next Murloc play)
            state.make_mut().players[owner.index()].next_murloc_discount = amount;
        }
        CardEffect::GiveNextMurlocDivineShield => {
            // Hot Spring Glider's Kindred (M2-W3) — "your next Murloc
            // gains Divine Shield" (consumed by the next Murloc play)
            state.make_mut().players[owner.index()].next_murloc_divine_shield = true;
        }
        CardEffect::SetNextKindredTwice => {
            // Primalfin Challenger's battlecry (M2-W3) — "your next
            // Kindred triggers twice" (consumed by the next OnPlay Kindred
            // resolution, which resolves its effect twice)
            state.make_mut().players[owner.index()].next_kindred_twice = true;
        }
        CardEffect::DrawKindredAndActivator => {
            // Torga's battlecry (M2-W3): draw the first Kindred-registry
            // card from the deck, then the first remaining card of the
            // same kindred type ("another card that activates it"). The
            // scans are top-down over the actual deck; an empty match
            // draws nothing (the deck-scan draw pattern — no fatigue for
            // scans).
            let deck: SmallList<Entity> = state.world().zones().iter(Zone::Deck, owner).collect();
            let Some(kindred_idx) = deck.iter().position(|&e| {
                state
                    .world()
                    .card_id(e)
                    .is_some_and(|cid| crate::cards::kindred::kindred_type(cid.0).is_some())
            }) else {
                return;
            };
            let kindred_card = deck[kindred_idx];
            let kindred_type = state
                .world()
                .card_id(kindred_card)
                .and_then(|cid| crate::cards::kindred::kindred_type(cid.0));
            let _ = state.world_mut().move_to_zone(kindred_card, Zone::Hand);
            queue.push(Event::CardDrawn {
                player: owner,
                card: kindred_card,
            });
            let Some(kindred_type) = kindred_type else {
                return;
            };
            for e in deck.iter().skip(kindred_idx + 1) {
                let Some(def) = state
                    .world()
                    .card_id(*e)
                    .and_then(|cid| crate::cards::def::card_by_id(cid.0))
                else {
                    continue;
                };
                if crate::cards::kindred::played_type_of(def) == Some(kindred_type) {
                    let _ = state.world_mut().move_to_zone(*e, Zone::Hand);
                    queue.push(Event::CardDrawn {
                        player: owner,
                        card: *e,
                    });
                    return;
                }
            }
        }
        CardEffect::DrawSpellGiveSpellDamage { amount } => {
            // Volcanic Thrasher's battlecry (M2-W3): draw a Fire spell;
            // the Kindred half gives it Spell Damage +2 (the spell-school
            // filter uses the W1 registry — the Fire school; the drawn
            // spell's SpellDamage boosts its own damage when cast, the
            // real "Give it Spell Damage" behavior).
            let fire_spell: Option<Entity> =
                state.world().zones().iter(Zone::Deck, owner).find(|&e| {
                    state.world().card_id(e).is_some_and(|cid| {
                        crate::cards::quest::spell_school(cid.0)
                            == Some(crate::cards::quest::SpellSchool::Fire)
                    })
                });
            let Some(drawn) = fire_spell else {
                return;
            };
            let _ = state.world_mut().move_to_zone(drawn, Zone::Hand);
            queue.push(Event::CardDrawn {
                player: owner,
                card: drawn,
            });
            if crate::cards::kindred::kindred_active(
                state,
                owner,
                crate::cards::kindred::KindredType::Minion(crate::core::component::Race::Elemental),
                2,
            ) {
                let cur = state.world().spell_damage(drawn).map_or(0, |s| s.0);
                state
                    .world_mut()
                    .set_spell_damage(drawn, crate::core::component::SpellDamage(cur + amount));
            }
        }
        CardEffect::DrawMinionsOfEachCost { up_to } => {
            // Hybridization's battlecry (M2-W3): draw a minion of each
            // cost from 1 to `up_to` (the exact-cost scan; a missing cost
            // draws nothing for that slot). The Kindred half makes each
            // drawn card cost (1) less.
            let mut drawn: SmallList<Entity> = SmallList::new();
            for target_cost in 1..=up_to {
                let Some(e) = state.world().zones().iter(Zone::Deck, owner).find(|&e| {
                    state.world().card_type(e) == Some(CardType::Minion)
                        && state.world().effective_cost(e).map_or(0, |c| c.0) == target_cost
                }) else {
                    continue;
                };
                let _ = state.world_mut().move_to_zone(e, Zone::Hand);
                queue.push(Event::CardDrawn {
                    player: owner,
                    card: e,
                });
                drawn.push(e);
            }
            if crate::cards::kindred::kindred_active(
                state,
                owner,
                crate::cards::kindred::KindredType::Spell,
                2,
            ) {
                for e in drawn {
                    let cur = state.world().effective_cost(e).map_or(0, |c| c.0);
                    state.world_mut().set_cost(e, Cost((cur - 1).max(0)));
                }
            }
        }
        CardEffect::DrawDeathrattleMinionCostLE { max_cost } => {
            // Dread Raptor's battlecry (M2-W3): draw a Deathrattle minion
            // costing at most `max_cost`; the Kindred half makes it cost
            // (0).
            let Some(e) = state.world().zones().iter(Zone::Deck, owner).find(|&e| {
                state.world().card_type(e) == Some(CardType::Minion)
                    && state.world().deathrattle(e).is_some()
                    && state.world().effective_cost(e).map_or(0, |c| c.0) <= max_cost
            }) else {
                return;
            };
            let _ = state.world_mut().move_to_zone(e, Zone::Hand);
            queue.push(Event::CardDrawn {
                player: owner,
                card: e,
            });
            if crate::cards::kindred::kindred_active(
                state,
                owner,
                crate::cards::kindred::KindredType::Minion(crate::core::component::Race::Undead),
                2,
            ) {
                state.world_mut().set_cost(e, Cost(0));
            }
        }
        CardEffect::DestroyLowestAttackEnemy => {
            // Scalehide Kodo's battlecry (M2-W3) — destroy the
            // lowest-Attack enemy minion (the DestroyHighestAttackEnemy
            // pattern; ties break toward the first — the stable sort).
            let mut enemies: SmallList<Entity> = collect_enemy_minions(state, owner, None);
            if enemies.is_empty() {
                return;
            }
            enemies.sort_by_key(|&e| state.world().effective_attack(e).unwrap_or(Attack(0)).0);
            let target = enemies[0];
            let hp = state.world().effective_health(target).map_or(0, |h| h.0);
            queue.push(Event::DamageDealt {
                source,
                target,
                amount: hp.max(1),
            });
        }
        CardEffect::TriggerFriendlyCinderDeathrattles => {
            // Slagclaw's Kindred add-on (M2-W3): trigger the Deathrattles
            // of all friendly Sizzling Cinders (the two his battlecry
            // summoned — the deathrattle pattern from rules.rs, the
            // deathrattle component resolves per Cinder).
            let cinders: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Play, owner)
                .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "TLC_249"))
                .collect();
            for cinder in cinders {
                if let (Some(dr), Some(cinder_owner)) = (
                    state.world().deathrattle(cinder),
                    state.world().player(cinder),
                ) {
                    resolve_effect(state, queue, cinder, cinder_owner, dr.0, None, None);
                }
            }
        }
        CardEffect::DestroyMinionAndGainItsStats { target } => {
            // Ravenous Devilsaur's Kindred (M2-W3): destroy a minion and
            // give the source its stats (the stats are read BEFORE the
            // destroy — the target's deathrattles still fire through the
            // destroy machinery).
            let enemies: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Play, owner)
                .chain(state.world().zones().iter(Zone::Play, owner.opponent()))
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            let Some(selected) = select_target(explicit_target, &enemies, state.rng_mut()) else {
                return;
            };
            let atk = state
                .world()
                .effective_attack(selected)
                .unwrap_or(Attack(0))
                .0;
            let hp = state
                .world()
                .effective_health(selected)
                .unwrap_or(Health(0))
                .0;
            resolve_destroy_minion(state, queue, owner, source, target, Some(selected));
            state.world_mut().add_enchantment(
                source,
                Enchantment {
                    attack: atk,
                    health: hp,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
        }
        CardEffect::DealSelfAttackDamage { target } => {
            // Ravasaur Matriarch's Kindred (M2-W3): deal damage equal to
            // the source's Attack to the target
            let atk = state
                .world()
                .effective_attack(source)
                .unwrap_or(Attack(0))
                .0;
            resolve_deal_damage(state, queue, source, owner, atk, target, explicit_target);
        }
        CardEffect::SummonRandomMinionCostTaunt { cost } => {
            // Gravedawn Voidbulb (M2-W3): summon a random minion of the
            // given cost and give it Taunt. The random pool follows the D2
            // simplification — ALL_CARDS minions of that cost, token
            // excluded (the BuffAndSummonRandomCost2 convention, §16).
            let pool: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| {
                        c.card_type == CardType::Minion && c.cost == cost && !c.id.ends_with('t')
                    })
                    .collect();
            if let Some(&pick) = pool.get(state.rng_mut().next_usize(pool.len())) {
                if let Some(e) = resolve_summon(state, queue, source, owner, pick.id) {
                    state.world_mut().set_taunt(e, Taunt);
                }
            }
        }
        // ===== 2025–2026 expansions M2-W4a (the Un'Goro main set) =====
        CardEffect::DiscoverPool { pool } => {
            // The D2 discover simplification surfaces a real three-option
            // choice built from `discover_pool_cards` (fidelity-debt §17 —
            // each pool is a filtered sampling procedure). The source card
            // keys the Map chain (the other options are stored so playing
            // the picked card this turn adds one of them) and the
            // Bloodpetal Biome Temporary marker.
            let cards = crate::cards::pool::discover_pool_cards(pool, state, owner);
            if cards.is_empty() {
                return;
            }
            let options: Vec<String> = cards
                .iter()
                .map(|c| format!("{} ({})", c.name, c.id))
                .collect();
            let pool_ids: Vec<String> = cards.iter().map(|c| c.id.to_string()).collect();
            let source_id = state
                .world()
                .card_id(source)
                .map(|c| c.0)
                .unwrap_or_default();
            // The Map cards (fidelity-debt §17): all six share one shape —
            // the unpicked options are stored and one of them is added when
            // the discovered card is played this turn (CardPlayed hook).
            let map_others = if matches!(
                source_id,
                "TLC_435" | "TLC_442" | "TLC_464" | "TLC_515" | "TLC_824" | "TLC_900"
            ) {
                pool_ids.clone()
            } else {
                Vec::new()
            };
            // Bloodpetal Biome (a Location): the discovered card is
            // Temporary (discarded at the end of the turn).
            let temporary = source_id == "TLC_449";
            state.set_pending_choice_w4a(
                crate::core::state::ChoiceKind::Discover,
                source,
                options,
                pool_ids,
                false,
                1,
                temporary,
                map_others,
            );
            // Quest progress (M2-W1): TLC_460 — "Discover 7 cards".
            crate::engine::quest::progress(
                state,
                queue,
                owner,
                crate::cards::quest::QuestCondition::DiscoverCards,
                1,
                None,
            );
        }
        CardEffect::SummonRandomFelBeast => {
            // Deathrot Maw's deathrattle — summon a random Fel Beast
            // (the D2 random simplification; the pool is in pool.rs).
            let Some(card) = crate::cards::pool::random_card(state.rng_mut(), RandomPool::FelBeast)
            else {
                return;
            };
            let _ = resolve_summon(state, queue, source, owner, card.id);
        }
        CardEffect::AddRandomBeastCostLess { amount } => {
            // Storm the Gates' reward — a random Beast crafted into the
            // hand with a cost reduction (the Zombeast "costs (3) less").
            let Some(card) = crate::cards::pool::random_card(state.rng_mut(), RandomPool::Beast)
            else {
                return;
            };
            if let Some(e) = add_card_to_hand(state, owner, card) {
                reduce_hand_card_cost(state, e, amount);
            }
        }
        CardEffect::TriggerFriendlyDeadDeathrattles { count } => {
            // Endbringer Umbra (M2-W4b): trigger the Deathrattles of up to
            // `count` friendly minions that died this game — the friendly
            // graveyard IS the died-this-game log (entities keep their
            // components in the graveyard, the MinionDied handler pattern).
            // The graveyard is scanned in death order; each deathrattle
            // resolves from the graveyard like a normal death (the W3
            // Sizzling Cinder scan generalized).
            let dead: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Graveyard, owner)
                .filter(|&e| {
                    state.world().card_type(e) == Some(CardType::Minion)
                        && state.world().deathrattle(e).is_some()
                })
                .take(count.max(0) as usize)
                .collect();
            for e in dead {
                if let (Some(dr), Some(e_owner)) =
                    (state.world().deathrattle(e), state.world().player(e))
                {
                    resolve_effect(state, queue, e, e_owner, dr.0, None, None);
                }
            }
        }
        CardEffect::EshoDeckCheckBuffEverywhere { attack, health } => {
            // City Chief Esho (M2-W4b): "If every minion in your deck
            // shares a minion type, give your other minions +2/+2 wherever
            // they are." The deck check reads the CURRENT deck: there must
            // exist a single race R such that every minion in the deck has
            // R (dual-tribe minions share through either tribe; a minion
            // with no type shares nothing and fails the check). Empty
            // deck: no minion falsifies the statement — the check passes
            // vacuously. The buff then hits the owner's OTHER minions —
            // hand and deck minions get base stat buffs (the Grimestreet
            // convention), board minions get a permanent enchantment; the
            // source is excluded ("other").
            let deck_minions: Vec<Entity> = state
                .world()
                .zones()
                .iter(Zone::Deck, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            let all_share = if deck_minions.is_empty() {
                true
            } else {
                let candidates = [
                    Race::Beast,
                    Race::Murloc,
                    Race::Demon,
                    Race::Dragon,
                    Race::Elemental,
                    Race::Mechanical,
                    Race::Pirate,
                    Race::Totem,
                    Race::Undead,
                    Race::Quilboar,
                    Race::Draenei,
                    Race::Naga,
                ];
                let first = deck_minions[0];
                candidates.iter().any(|&race| {
                    state.world().has_race(first, race)
                        && deck_minions
                            .iter()
                            .all(|&e| state.world().has_race(e, race))
                })
            };
            if !all_share {
                return;
            }
            for zone in [Zone::Hand, Zone::Deck] {
                let cards: Vec<Entity> = state.world().zones().iter(zone, owner).collect();
                for e in cards {
                    if e == source || state.world().card_type(e) != Some(CardType::Minion) {
                        continue;
                    }
                    let world = state.world_mut();
                    let base = world.attack(e).unwrap_or(Attack(0));
                    world.set_attack(e, Attack(base.0 + attack));
                    let base_hp = world.health(e).unwrap_or(Health(1));
                    world.set_health(e, Health(base_hp.0 + health));
                }
            }
            let board: Vec<Entity> = state.world().zones().iter(Zone::Play, owner).collect();
            for e in board {
                if e != source && state.world().card_type(e) == Some(CardType::Minion) {
                    state.world_mut().add_enchantment(
                        e,
                        Enchantment {
                            attack,
                            health,
                            cost: 0,
                            expiry: EnchantmentExpiry::Permanent,
                        },
                    );
                }
            }
        }
        CardEffect::SetStatsAllEnemyMinions { attack, health } => {
            // Krog, Crater King (M2-W4b): end of the owner's turn, set the
            // Attack and Health of all enemy minions to fixed values. The
            // set writes the base stats, clears accumulated damage and
            // strips permanent enchantments — the "make it a 1/1" set
            // semantics (dynamic auras still apply on top, as in HS).
            let enemies: SmallList<Entity> = collect_all_enemy_minions(state, owner);
            let world = state.world_mut();
            for e in enemies {
                world.set_attack(e, Attack(attack));
                world.set_health(e, Health(health));
                world.remove_damage(e);
                world.remove_enchantments(e);
            }
        }
        CardEffect::SummonDamagedCopiesRush => {
            // Nablya, the Watcher (M2-W4b): summon a fresh copy of each
            // damaged friendly minion; the copies gain Rush. The damaged
            // list is snapshotted first — a copy is a fresh base-stat
            // entity (the engine copy convention) and is not itself
            // damaged, so the snapshot prevents infinite copying.
            let damaged: SmallList<Entity> = collect_friendly_minions(state, owner)
                .into_iter()
                .filter(|&e| state.world().damage(e).is_some_and(|d| d.0 > 0))
                .collect();
            for e in damaged {
                let Some(cid) = state.world().card_id(e).map(|c| c.0) else {
                    continue;
                };
                if let Some(copy) = resolve_summon(state, queue, source, owner, cid) {
                    // The copies are SUMMONED, not played — battlecries
                    // (and combos) do not fire. The engine's MinionSummoned
                    // handler fires whatever battlecry component a summoned
                    // minion carries, so strip the components here: an
                    // Injured Blademaster copy must arrive as a fresh 4/7,
                    // not damage itself.
                    state.world_mut().remove_battlecry(copy);
                    state.world_mut().remove_combo_effect(copy);
                    state.world_mut().set_rush(copy, Rush);
                }
            }
        }
        CardEffect::SummonTwoDeathrattleMinionsAndFight => {
            // High Cultist Herenn (M2-W4b, registered simplification §18):
            // summon two random Deathrattle minions from the deck — as
            // copies, the deck itself is untouched (resolve_summon reads
            // the card definition, the established copy convention) — and
            // "they fight!": each deals damage equal to its Attack to the
            // other once, through the normal damage pipeline (divine
            // shields, deaths, etc. resolve normally). One or zero
            // deathrattle minions in the deck summon what exists.
            let mut ids: Vec<&'static str> = state
                .world()
                .zones()
                .iter(Zone::Deck, owner)
                .filter_map(|e| state.world().card_id(e).map(|c| c.0))
                .filter(|id| {
                    crate::cards::def::card_by_id(id).is_some_and(|def| def.deathrattle.is_some())
                })
                .collect();
            // Pick two distinct random ids.
            if ids.len() > 1 {
                let i = state.rng_mut().next_usize(ids.len());
                let a = ids.remove(i);
                let j = state.rng_mut().next_usize(ids.len());
                let b = ids.remove(j);
                if let Some(fa) = resolve_summon(state, queue, source, owner, a) {
                    if let Some(fb) = resolve_summon(state, queue, source, owner, b) {
                        let atk_a = state.world().effective_attack(fa).unwrap_or(Attack(0)).0;
                        let atk_b = state.world().effective_attack(fb).unwrap_or(Attack(0)).0;
                        queue.push(Event::DamageDealt {
                            source: fa,
                            target: fb,
                            amount: atk_a,
                        });
                        queue.push(Event::DamageDealt {
                            source: fb,
                            target: fa,
                            amount: atk_b,
                        });
                    }
                }
            } else if let Some(id) = ids.first() {
                let _ = resolve_summon(state, queue, source, owner, id);
            }
        }
        CardEffect::LohMinionsCost5 => {
            // Loh, the Living Legend (M2-W4b): "your minions cost (5) this
            // game" — the flag is read by the play-cost pipeline.
            state.make_mut().players[owner.index()].minions_cost_5 = true;
        }
        CardEffect::EliseCraftLocation => {
            // Elise the Navigator (M2-W4b, registered simplification §18):
            // the real battlecry crafts a custom Location when the deck
            // started with 10 cards of different Costs. The check runs
            // against the `Player::starting_deck` snapshot (taken by
            // GameBuilder at game start) and only sets the crafted-location
            // marker — no custom-location machinery exists yet.
            let costs: std::collections::HashSet<i32> = state
                .player(owner)
                .starting_deck
                .iter()
                .filter_map(|id| crate::cards::def::card_by_id(id).map(|def| def.cost))
                .collect();
            if costs.len() >= 10 {
                state.make_mut().players[owner.index()].elise_location_crafted = true;
            }
        }
        CardEffect::NiriOfTheCrater => {
            // Niri of the Crater (M2-W4b): the event subject is the card
            // that was just played (the CardPlayed event fires after full
            // resolution, for minion plays and spell casts alike — the
            // single trigger slot the per-ID registration can hold). The
            // effect branches on the subject's card type; "1-Cost" reads
            // the effective cost at trigger time.
            let Some(subject) = event_subject else {
                return;
            };
            let cost = state.world().effective_cost(subject).unwrap_or(Cost(0)).0;
            if cost != 1 {
                return;
            }
            match state.world().card_type(subject) {
                // A 1-Cost minion doubles its stats — an enchantment equal
                // to the current effective stats (the DoubleAttack/
                // DoubleHealth convention).
                Some(CardType::Minion) => {
                    let cur_atk = state
                        .world()
                        .effective_attack(subject)
                        .unwrap_or(Attack(0))
                        .0;
                    let cur_hp = state
                        .world()
                        .effective_health(subject)
                        .unwrap_or(Health(0))
                        .0;
                    state.world_mut().add_enchantment(
                        subject,
                        Enchantment {
                            attack: cur_atk,
                            health: cur_hp,
                            cost: 0,
                            expiry: EnchantmentExpiry::Permanent,
                        },
                    );
                }
                // A 1-Cost spell casts twice: the effect re-resolves once
                // with no explicit target and no second SpellCast event
                // (the Tyrande timing simplification §14.4). The subject
                // rides in the graveyard where its battlecry component
                // persists.
                Some(CardType::Spell) => {
                    if let Some(effect) = state.world().battlecry(subject).map(|b| b.0) {
                        resolve_effect(state, queue, subject, owner, effect, None, None);
                    }
                }
                _ => {}
            }
        }
        CardEffect::SetEventSubjectHealthToSource => {
            // Archaios (M2-W4b): "after a friendly minion attacks, set its
            // Health to this minion's Health" — the attacker (event
            // subject) has its Health set to the source's effective Health
            // (the set clears damage and permanent enchantments so the
            // effective Health equals the value).
            let Some(subject) = event_subject else {
                return;
            };
            // "another friendly minion": the source's own attack does not
            // re-set its own Health (a no-op set would be harmless, but the
            // enchantment strip could remove its own buffs).
            if subject == source {
                return;
            }
            let target_hp = state
                .world()
                .effective_health(source)
                .unwrap_or(Health(1))
                .0;
            let world = state.world_mut();
            world.set_health(subject, Health(target_hp));
            world.remove_damage(subject);
            world.remove_enchantments(subject);
        }
        CardEffect::AddRandomCardToHandCount { pool, count } => {
            // A random pool card added straight to the hand (the D2
            // "get a random X" shape — no choice is surfaced).
            for _ in 0..count {
                let Some(card) = crate::cards::pool::random_card(state.rng_mut(), pool) else {
                    return;
                };
                add_card_to_hand(state, owner, card);
            }
        }
        CardEffect::AddCardToHandCount { card_id, count } => {
            // Fixed-card adds (Infestation's Gorishi Stingers).
            let Some(def) = crate::cards::def::card_by_id(card_id) else {
                return;
            };
            for _ in 0..count {
                add_card_to_hand(state, owner, def);
            }
        }
        CardEffect::AddTemporaryRandomMinionsCost { cost, count } => {
            // Tunnel Terror's deathrattle — random minions of the given
            // cost marked Temporary (the W2 primitive; discarded at the end
            // of the turn).
            for _ in 0..count {
                let Some(pick) = random_minion_of_cost(state, cost) else {
                    return;
                };
                if let Some(e) = add_card_to_hand(state, owner, pick) {
                    state.world_mut().set_temporary(e, Temporary);
                }
            }
        }
        CardEffect::AddRandomFelSpellsCostHealth { count } => {
            // Whispering Stone — random Fel spells that cost Health instead
            // of Mana (the CostHealth marker is read by the CardPlayed
            // pay-health branch, fidelity-debt §17).
            for _ in 0..count {
                let Some(card) =
                    crate::cards::pool::random_card(state.rng_mut(), RandomPool::FelSpell)
                else {
                    return;
                };
                if let Some(e) = add_card_to_hand(state, owner, card) {
                    state.world_mut().set_cost_health(e, CostHealth);
                }
            }
        }
        CardEffect::AddRandomHolyAndShadowSpell => {
            // Twilight Mender — one random Holy and one random Shadow spell.
            if let Some(card) =
                crate::cards::pool::random_card(state.rng_mut(), RandomPool::HolySpell)
            {
                add_card_to_hand(state, owner, card);
            }
            if let Some(card) =
                crate::cards::pool::random_card(state.rng_mut(), RandomPool::ShadowSpell)
            {
                add_card_to_hand(state, owner, card);
            }
        }
        CardEffect::AddRandomHolySpellCost1 => {
            // Glade Ecologist — a random 1-Cost Holy spell (the pool is
            // pre-filtered in pool.rs; the cost is set to exactly 1 as a
            // belt-and-braces for cards with modified costs).
            if let Some(card) =
                crate::cards::pool::random_card(state.rng_mut(), RandomPool::HolySpellCost1)
            {
                if let Some(e) = add_card_to_hand(state, owner, card) {
                    state.world_mut().set_cost(e, Cost(1));
                }
            }
        }
        CardEffect::CopyRandomHandElementalOrDragon => {
            // Cloud Serpent — a copy of another Elemental or Dragon in the
            // hand; no eligible card copies nothing.
            let hand: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter(|&e| {
                    state.world().card_type(e) == Some(CardType::Minion)
                        && (state.world().has_race(e, Race::Elemental)
                            || state.world().has_race(e, Race::Dragon))
                })
                .collect();
            if hand.is_empty() {
                return;
            }
            let picked = hand[state.rng_mut().next_usize(hand.len())];
            copy_card_to_hand(state, picked, owner);
        }
        CardEffect::ReduceRandomEnemyHandMinionCost { amount } => {
            // Curious Explorer — reduce the Cost of a random minion in the
            // opponent's hand (a cost-reduction enchantment).
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
            let picked = hand[state.rng_mut().next_usize(hand.len())];
            reduce_hand_card_cost(state, picked, amount);
        }
        CardEffect::ReduceRandomBeastHandCost { amount } => {
            // Dinositter's end-of-turn effect — a random Beast in hand.
            let hand: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter(|&e| {
                    state.world().card_type(e) == Some(CardType::Minion)
                        && state.world().has_race(e, Race::Beast)
                })
                .collect();
            if hand.is_empty() {
                return;
            }
            let picked = hand[state.rng_mut().next_usize(hand.len())];
            reduce_hand_card_cost(state, picked, amount);
        }
        CardEffect::ReduceNonStartingHandCost { amount } => {
            // Story of the Waygate — every hand card that did not start in
            // the deck costs less (the starting deck is snapshotted by
            // GameBuilder, fidelity-debt §17).
            let hand: Vec<Entity> = state.world().zones().iter(Zone::Hand, owner).collect();
            for e in hand {
                let Some(cid) = state.world().card_id(e) else {
                    continue;
                };
                if state.player(owner).starting_deck.iter().any(|s| s == cid.0) {
                    continue;
                }
                reduce_hand_card_cost(state, e, amount);
            }
        }
        CardEffect::SummonTreantsAttackMinion => {
            // TREEEES!!! — summon four 2/2 Treants that each attack the
            // chosen minion (the attacks run through the normal attack
            // pipeline, per the ForceEnemyMinionsAttackThis convention).
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            let Some(target) = select_target(explicit_target, &all, state.rng_mut()) else {
                return;
            };
            for _ in 0..4 {
                let Some(treant) = resolve_summon(state, queue, source, owner, "TLC_230t") else {
                    continue;
                };
                let atk = crate::engine::rules::compute_attacker_damage(state, treant);
                queue.push(Event::AttackDeclared {
                    attacker: treant,
                    defender: target,
                });
                queue.push(Event::ResolveAttack {
                    attacker: treant,
                    defender: target,
                    attacker_damage: atk,
                    retaliation_immune: false,
                });
            }
        }
        CardEffect::DealDamageSummonCinders { amount } => {
            // Sizzling Swarm — damage a random enemy and summon that many
            // 2/1 Sizzling Cinders.
            resolve_deal_damage(
                state,
                queue,
                source,
                owner,
                amount,
                EffectTarget::AnyEnemy,
                None,
            );
            for _ in 0..amount.max(0) {
                let _ = resolve_summon(state, queue, source, owner, "TLC_249");
            }
        }
        CardEffect::DealDamageLowestHealthEnemyRepeated { amount, times } => {
            // Lava Flow — repeatedly hit the enemy with the lowest Health;
            // ties resolve randomly. Repeated hits read the same state
            // snapshot (the damage is queued), so the lowest-health enemy
            // receives all hits.
            for _ in 0..times {
                let enemies = collect_enemy_characters(state, owner, Some(source));
                if enemies.is_empty() {
                    return;
                }
                let min_hp = enemies
                    .iter()
                    .map(|&e| state.world().effective_health(e).unwrap_or(Health(0)).0)
                    .min()
                    .unwrap_or(0);
                let lowest: SmallList<Entity> = enemies
                    .into_iter()
                    .filter(|&e| state.world().effective_health(e).unwrap_or(Health(0)).0 == min_hp)
                    .collect();
                let target = lowest[state.rng_mut().next_usize(lowest.len())];
                queue.push(Event::DamageDealt {
                    source,
                    target,
                    amount,
                });
            }
        }
        CardEffect::DealDamageRandomEnemies { amount, count } => {
            // Bonechill Stegodon's deathrattle — `count` random DISTINCT
            // enemies (sampling without replacement).
            let mut enemies: Vec<Entity> = collect_enemy_characters(state, owner, Some(source))
                .into_iter()
                .collect();
            for _ in 0..count {
                if enemies.is_empty() {
                    break;
                }
                let idx = state.rng_mut().next_usize(enemies.len());
                let target = enemies.remove(idx);
                queue.push(Event::DamageDealt {
                    source,
                    target,
                    amount,
                });
            }
        }
        CardEffect::DrawMinionsDifferentTypesBuff {
            count,
            attack,
            health,
        } => {
            // Flight of the Firehawk — draw until `count` minions of
            // DIFFERENT minion types have been drawn (same-type minions
            // keep the draw going); each counted minion is buffed.
            let mut seen: Vec<Race> = Vec::new();
            while seen.len() < count as usize {
                let Some(e) = draw_card_no_queue(state, queue, owner) else {
                    break;
                };
                if state.world().card_type(e) != Some(CardType::Minion) {
                    continue;
                }
                let races: Vec<Race> = state
                    .world()
                    .race(e)
                    .map(|r| r.to_vec())
                    .unwrap_or_default();
                let Some(new_race) = races.iter().find(|r| !seen.contains(r)).copied() else {
                    continue;
                };
                seen.push(new_race);
                let world = state.world_mut();
                let base = world.attack(e).unwrap_or(Attack(0));
                world.set_attack(e, Attack(base.0 + attack));
                let base_hp = world.health(e).unwrap_or(Health(1));
                world.set_health(e, Health(base_hp.0 + health));
            }
        }
        CardEffect::DrawMinionBuffArmorIfAttackGE {
            min_attack,
            buff_health,
            armor,
        } => {
            // Story of Barnabus — draw a minion; a minion with at least
            // `min_attack` Attack is buffed and the owner gains armor.
            loop {
                let Some(e) = draw_card_no_queue(state, queue, owner) else {
                    return;
                };
                if state.world().card_type(e) != Some(CardType::Minion) {
                    continue;
                }
                if state
                    .world()
                    .effective_attack(e)
                    .is_some_and(|a| a.0 >= min_attack)
                {
                    let base = state.world().health(e).unwrap_or(Health(1));
                    state
                        .world_mut()
                        .set_health(e, Health(base.0 + buff_health));
                    state.make_mut().players[owner.index()].armor += armor;
                }
                return;
            }
        }
        CardEffect::SetFlockPending => {
            // Ravenous Flock — three 2/1 Hatchlings at the start of the
            // owner's NEXT turn (the turn-start hook resolves them).
            state.make_mut().players[owner.index()].flock_pending = true;
        }
        CardEffect::GiveBuffOtherMinionsAttackLE {
            attack,
            health,
            max_attack,
        } => {
            // Hatchery Helper — the owner's OTHER minions with at most
            // `max_attack` Attack get stats and Taunt.
            let others: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Play, owner)
                .filter(|&e| {
                    e != source
                        && state.world().card_type(e) == Some(CardType::Minion)
                        && state
                            .world()
                            .effective_attack(e)
                            .is_some_and(|a| a.0 <= max_attack)
                })
                .collect();
            for e in others {
                let world = state.world_mut();
                let base = world.attack(e).unwrap_or(Attack(0));
                world.set_attack(e, Attack(base.0 + attack));
                let base_hp = world.health(e).unwrap_or(Health(1));
                world.set_health(e, Health(base_hp.0 + health));
                world.set_taunt(e, Taunt);
            }
        }
        CardEffect::DestroyMinionSummonRandomSameCost { target } => {
            // Life Cycle — destroy a minion and summon a random minion of
            // the same Cost to replace it (the cost is read before the
            // destroy).
            let candidates = collect_target_candidates(state, owner, target, source);
            let Some(picked) = select_target(explicit_target, &candidates, state.rng_mut()) else {
                return;
            };
            let cost = state.world().effective_cost(picked).unwrap_or(Cost(0)).0;
            resolve_destroy_minion(state, queue, owner, source, target, Some(picked));
            if let Some(pick) = random_minion_of_cost(state, cost) {
                let _ = resolve_summon(state, queue, source, owner, pick.id);
            }
        }
        CardEffect::SummonMinionsGrantRandomBonus { card_id, count } => {
            // Tyrannogill's Dinolocs — each summoned copy gets a random
            // Bonus Effect from the approximated pool (§14.3).
            for _ in 0..count {
                if let Some(e) = resolve_summon(state, queue, source, owner, card_id) {
                    grant_random_bonus(state, e);
                }
            }
        }
        CardEffect::SummonMinionPair { a, b } => {
            // Blob of Tar — one copy of each of two tokens.
            let _ = resolve_summon(state, queue, source, owner, a);
            let _ = resolve_summon(state, queue, source, owner, b);
        }
        CardEffect::SummonRandomMinionCostOrEscalated {
            cost,
            escalated_cost,
        } => {
            // Unearthed Artifacts — a random minion of the base cost, or
            // of the escalated cost when the player Discovered this turn.
            let final_cost = if state.player(owner).discovered_this_turn {
                escalated_cost
            } else {
                cost
            };
            if let Some(pick) = random_minion_of_cost(state, final_cost) {
                let _ = resolve_summon(state, queue, source, owner, pick.id);
            }
        }
        CardEffect::DealDamageGainArmorIfKilled {
            amount,
            armor,
            target,
        } => {
            // Latorvian Armorer — damage an enemy minion; the armor is
            // gained when the damage would kill it (the predicted-death
            // convention of DamageAndDrawIfKilled).
            let candidates = collect_target_candidates(state, owner, target, source);
            let Some(picked) = select_target(explicit_target, &candidates, state.rng_mut()) else {
                return;
            };
            let dies = state.world().divine_shield(picked).is_none()
                && state
                    .world()
                    .effective_health(picked)
                    .is_some_and(|h| h.0 - amount <= 0);
            queue.push(Event::DamageDealt {
                source,
                target: picked,
                amount,
            });
            if dies {
                state.make_mut().players[owner.index()].armor += armor;
            }
        }
        CardEffect::DealDamageAllEnemyMinionsSetMinionsCostMore { damage } => {
            // Wave of Tar — damage all enemy minions; the enemy's minions
            // cost (2) more next turn. The flag lands on the CASTER's
            // player record (play_cost reads
            // `player.opponent().minions_cost_more`) and clears at the
            // caster's next turn start.
            for m in collect_all_enemy_minions(state, owner) {
                queue.push(Event::DamageDealt {
                    source,
                    target: m,
                    amount: damage,
                });
            }
            state.make_mut().players[owner.index()].minions_cost_more = true;
        }
        CardEffect::GiveBuffSameType {
            attack,
            health,
            target,
        } => {
            // Ready the Fleet — buff the target, then the other friendly
            // minions that share its minion type.
            let candidates = collect_target_candidates(state, owner, target, source);
            let Some(picked) = select_target(explicit_target, &candidates, state.rng_mut()) else {
                return;
            };
            state.world_mut().add_enchantment(
                picked,
                Enchantment {
                    attack,
                    health,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
            let race = state.world().race(picked).and_then(|r| r.first()).copied();
            if let Some(race) = race {
                for e in collect_friendly_minions(state, owner) {
                    if e != picked && state.world().has_race(e, race) {
                        state.world_mut().add_enchantment(
                            e,
                            Enchantment {
                                attack,
                                health,
                                cost: 0,
                                expiry: EnchantmentExpiry::Permanent,
                            },
                        );
                    }
                }
            }
        }
        CardEffect::GrantRandomBonusEffects { count, target } => {
            // Story of Galvadon — `count` random Bonus Effects on the
            // target.
            let candidates = collect_target_candidates(state, owner, target, source);
            let Some(picked) = select_target(explicit_target, &candidates, state.rng_mut()) else {
                return;
            };
            for _ in 0..count {
                grant_random_bonus(state, picked);
            }
        }
        CardEffect::GrantRandomBonusEffectAndDeathrattle => {
            // Stranglevine — a random friendly minion gets a random Bonus
            // Effect and THIS deathrattle (so the chain recurs on its
            // death).
            let friendly = collect_friendly_minions(state, owner);
            if friendly.is_empty() {
                return;
            }
            let picked = friendly[state.rng_mut().next_usize(friendly.len())];
            grant_random_bonus(state, picked);
            state.world_mut().set_deathrattle(
                picked,
                Deathrattle(CardEffect::GrantRandomBonusEffectAndDeathrattle),
            );
        }
        CardEffect::SetLakkariTicks { ticks } => {
            // Story of Lakkari — the end-of-turn loop (discard a card, fill
            // the board with 3/2 Imps) runs for `ticks` turns.
            state.make_mut().players[owner.index()].lakkari_ticks = ticks;
        }
        CardEffect::GiveBuffAndSummonDeathrattle {
            attack,
            health,
            summon_cost,
            target,
        } => {
            // Threshrider's Blessing — buff the target and give it
            // "Deathrattle: Summon a random minion of `summon_cost`".
            let candidates = collect_target_candidates(state, owner, target, source);
            let Some(picked) = select_target(explicit_target, &candidates, state.rng_mut()) else {
                return;
            };
            state.world_mut().add_enchantment(
                picked,
                Enchantment {
                    attack,
                    health,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
            state.world_mut().set_deathrattle(
                picked,
                Deathrattle(CardEffect::SummonRandomMinionOfCost { cost: summon_cost }),
            );
        }
        CardEffect::DealDamageImprovedByShuffles { amount, target } => {
            // Knockback — damage improved by the number of times the owner
            // shuffled cards into their deck this game.
            let bonus = state.player(owner).shuffled_count as i32;
            let candidates = collect_target_candidates(state, owner, target, source);
            let Some(picked) = select_target(explicit_target, &candidates, state.rng_mut()) else {
                return;
            };
            queue.push(Event::DamageDealt {
                source,
                target: picked,
                amount: amount + bonus,
            });
        }
        CardEffect::DrawCardLinkDeathrattle => {
            // Platysaur's battlecry — draw a card and link it to this
            // minion so its deathrattle can discard the same card.
            if let Some(drawn) = draw_card_no_queue(state, queue, owner) {
                state.make_mut().players[owner.index()]
                    .platysaur_drawn
                    .push((source, drawn));
            }
        }
        CardEffect::DiscardLinkedDrawnCard => {
            // Platysaur's deathrattle — discard the card linked by the
            // battlecry (no-op when the card already left the hand).
            let linked = state
                .player(owner)
                .platysaur_drawn
                .iter()
                .position(|(minion, _)| *minion == source);
            if let Some(i) = linked {
                let (_, drawn) = state.make_mut().players[owner.index()]
                    .platysaur_drawn
                    .remove(i);
                if state.world().zone(drawn) == Some(Zone::Hand) {
                    let _ = state.world_mut().move_to_zone(drawn, Zone::Graveyard);
                }
            }
        }
        CardEffect::GainArmorDealDamageEqual { armor, target } => {
            // Fortify — gain armor, then deal damage equal to the owner's
            // total Armor to an enemy minion.
            {
                let inner = state.make_mut();
                let p = &mut inner.players[owner.index()];
                p.armor += armor;
            }
            let total = state.player(owner).armor;
            let candidates = collect_target_candidates(state, owner, target, source);
            let Some(picked) = select_target(explicit_target, &candidates, state.rng_mut()) else {
                return;
            };
            queue.push(Event::DamageDealt {
                source,
                target: picked,
                amount: total,
            });
        }
        CardEffect::DestroyDeckTop { count } => {
            // Willful Watcher's deathrattle — destroy the top `count` cards
            // of the owner's deck (they go to the graveyard).
            let top: Vec<Entity> = state
                .world()
                .zones()
                .iter(Zone::Deck, owner)
                .take(count as usize)
                .collect();
            for e in top {
                let _ = state.world_mut().move_to_zone(e, Zone::Graveyard);
            }
        }
        CardEffect::ResurrectOneOfEachCostGiveReborn { max_cost } => {
            // Resuscitate — resurrect a 1-, 2-, and 3-Cost minion and give
            // them Reborn (one fallen minion per cost from the graveyard;
            // the graveyard order is death order, the first of each cost
            // wins, mirroring ResurrectAllDifferentFriendlyCostGE).
            for cost in 1..=max_cost {
                let fallen = state
                    .world()
                    .zones()
                    .iter(Zone::Graveyard, owner)
                    .find(|&e| {
                        state.world().card_type(e) == Some(CardType::Minion)
                            && state.world().cost(e).is_some_and(|c| c.0 == cost)
                    });
                if let Some(e) = fallen {
                    if let Some(def) = state
                        .world()
                        .card_id(e)
                        .and_then(|c| crate::cards::def::card_by_id(c.0))
                    {
                        if let Some(s) = resolve_summon(state, queue, source, owner, def.id) {
                            state.world_mut().set_reborn(s, Reborn);
                        }
                    }
                }
            }
        }
        CardEffect::DealDamageSetNextBeastDiscount {
            amount,
            discount,
            target,
        } => {
            // Cower in Fear — damage a minion; the next Beast the owner
            // plays this turn costs less (one-time, the cost pipeline reads
            // it).
            let candidates = collect_target_candidates(state, owner, target, source);
            let Some(picked) = select_target(explicit_target, &candidates, state.rng_mut()) else {
                return;
            };
            queue.push(Event::DamageDealt {
                source,
                target: picked,
                amount,
            });
            state.make_mut().players[owner.index()].next_beast_discount = discount;
        }
        CardEffect::BuffAllBeastsEverywhere { attack, health } => {
            // Supreme Dinomancy — all Beasts in the hand, deck, and on the
            // battlefield get stats. Hand/deck buffs set the base (the
            // Grimestreet convention); board minions get a permanent
            // enchantment.
            for zone in [Zone::Hand, Zone::Deck] {
                let cards: Vec<Entity> = state.world().zones().iter(zone, owner).collect();
                for e in cards {
                    if !state.world().has_race(e, Race::Beast) {
                        continue;
                    }
                    let world = state.world_mut();
                    let base = world.attack(e).unwrap_or(Attack(0));
                    world.set_attack(e, Attack(base.0 + attack));
                    let base_hp = world.health(e).unwrap_or(Health(1));
                    world.set_health(e, Health(base_hp.0 + health));
                }
            }
            let board: Vec<Entity> = state.world().zones().iter(Zone::Play, owner).collect();
            for e in board {
                if state.world().has_race(e, Race::Beast) {
                    state.world_mut().add_enchantment(
                        e,
                        Enchantment {
                            attack,
                            health,
                            cost: 0,
                            expiry: EnchantmentExpiry::Permanent,
                        },
                    );
                }
            }
        }
        CardEffect::DealDamageSameType { amount, target } => {
            // Fumigate — damage the target minion and all other minions
            // (either side) of the same minion type.
            let candidates = collect_target_candidates(state, owner, target, source);
            let Some(picked) = select_target(explicit_target, &candidates, state.rng_mut()) else {
                return;
            };
            let races: Vec<Race> = state
                .world()
                .race(picked)
                .map(|r| r.to_vec())
                .unwrap_or_default();
            queue.push(Event::DamageDealt {
                source,
                target: picked,
                amount,
            });
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            for e in all {
                if e == picked {
                    continue;
                }
                if races.iter().any(|r| state.world().has_race(e, *r)) {
                    queue.push(Event::DamageDealt {
                        source,
                        target: e,
                        amount,
                    });
                }
            }
        }
        CardEffect::SetNextTemporaryDiscount { amount } => {
            // Spelunker — the next Temporary card the owner plays costs
            // less (one-time, consumed by the next Temporary play).
            state.make_mut().players[owner.index()].next_temporary_discount = amount;
        }
        CardEffect::SetEnemyHeroCantBeHealed => {
            // Crater Gator — until the start of the owner's next turn, the
            // enemy hero can't be healed (resolve_restore_health checks the
            // flag; it clears at the owner's turn start).
            state.make_mut().players[owner.opponent().index()].enemy_hero_cant_be_healed = true;
        }
        CardEffect::DestroyFriendlyMinionAddBones { target } => {
            // Dissolving Ooze — destroy a friendly minion and add two Bone
            // spells to the hand whose Attack copies the destroyed
            // minion's Attack (the Health-half is approximated, see
            // fidelity-debt §17).
            let candidates = collect_target_candidates(state, owner, target, source);
            let Some(picked) = select_target(explicit_target, &candidates, state.rng_mut()) else {
                return;
            };
            let atk = state
                .world()
                .effective_attack(picked)
                .unwrap_or(Attack(0))
                .0;
            resolve_destroy_minion(state, queue, owner, source, target, Some(picked));
            let Some(bone) = crate::cards::def::card_by_id("TLC_252t") else {
                return;
            };
            for _ in 0..2 {
                if let Some(e) = add_card_to_hand(state, owner, bone) {
                    state.world_mut().set_attack(e, Attack(atk));
                }
            }
        }
        CardEffect::GainManaCrystalsMatchOpponent => {
            // Crystal Tender — gain empty Mana Crystals until both players
            // have the same amount.
            let diff = state
                .player(owner.opponent())
                .mana_crystals
                .saturating_sub(state.player(owner).mana_crystals);
            if diff > 0 {
                let inner = state.make_mut();
                let p = &mut inner.players[owner.index()];
                p.mana_crystals = (p.mana_crystals + diff).min(10);
                p.current_mana = (p.current_mana + diff).min(10);
            }
        }
        CardEffect::GiveBuffDifferentTypeMinions { attack, health } => {
            // Tortollan Storyteller (end of turn) — buff each friendly
            // minion whose minion type appears exactly once among friendly
            // minions (the "different type" reading).
            let friendly: Vec<Entity> = state
                .world()
                .zones()
                .iter(Zone::Play, owner)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            let mut counts: std::collections::HashMap<Race, usize> =
                std::collections::HashMap::new();
            for e in &friendly {
                if let Some(races) = state.world().race(*e) {
                    for r in races {
                        *counts.entry(*r).or_insert(0) += 1;
                    }
                }
            }
            for e in friendly {
                let unique = state.world().race(e).is_some_and(|races| {
                    !races.is_empty() && races.iter().all(|r| counts.get(r) == Some(&1))
                });
                if unique {
                    state.world_mut().add_enchantment(
                        e,
                        Enchantment {
                            attack,
                            health,
                            cost: 0,
                            expiry: EnchantmentExpiry::Permanent,
                        },
                    );
                }
            }
        }
        CardEffect::DealDamageIfQuestPlayed { amount, target } => {
            // Questing Assistant — fires only when the owner played a Quest
            // this game (the flag is set by the W1 play-path diversion).
            if !state.player(owner).quest_played {
                return;
            }
            let candidates = collect_target_candidates(state, owner, target, source);
            let Some(picked) = select_target(explicit_target, &candidates, state.rng_mut()) else {
                return;
            };
            queue.push(Event::DamageDealt {
                source,
                target: picked,
                amount,
            });
        }
        CardEffect::SwapHeroPowerToDeal8Random => {
            // Story of Sulfuras — swap the hero power to "Deal 8 damage to
            // a random enemy"; the HeroPowerActivated hook swaps the
            // original back after 2 uses.
            let hero = state.player(owner).hero;
            let Some(hp_def) = state.world().hero_power(hero) else {
                return;
            };
            {
                let p = &mut state.make_mut().players[owner.index()];
                p.sulfuras_original = Some(hp_def);
                p.sulfuras_uses = 1;
            }
            state.world_mut().set_hero_power(
                hero,
                HeroPowerDef {
                    cost: 2,
                    effect: CardEffect::DealDamage {
                        amount: 8,
                        target: EffectTarget::AnyEnemy,
                    },
                },
            );
        }
        CardEffect::RecastRandomHolySpellThisTurn => {
            // Creature of the Sacred Cave (end of turn) — recast a random
            // Holy spell the owner cast this turn, "targeting this if
            // possible": the spell's effect resolves with the creature as
            // the explicit target (an illegal target fizzles per G9).
            let ids = state.player(owner).holy_cast_ids.clone();
            if ids.is_empty() {
                return;
            }
            let picked_id = ids[state.rng_mut().next_usize(ids.len())].clone();
            if let Some(def) = crate::cards::def::card_by_id(&picked_id) {
                if let Some(effect) = def.spell_effect {
                    resolve_effect(state, queue, source, owner, effect, Some(source), None);
                }
            }
        }
        CardEffect::CastRandomSpellFromDeckCostLE { max_cost } => {
            // Violet Treasuregill — cast a random spell from the owner's
            // deck costing at most `max_cost` ("targets this if possible",
            // the same explicit-target shape as the recast above); the
            // spell leaves the deck.
            let spells: Vec<Entity> = state
                .world()
                .zones()
                .iter(Zone::Deck, owner)
                .filter(|&e| {
                    state.world().card_type(e) == Some(CardType::Spell)
                        && state
                            .world()
                            .effective_cost(e)
                            .is_some_and(|c| c.0 <= max_cost)
                })
                .collect();
            if spells.is_empty() {
                return;
            }
            let picked = spells[state.rng_mut().next_usize(spells.len())];
            if let Some(def) = state
                .world()
                .card_id(picked)
                .and_then(|c| crate::cards::def::card_by_id(c.0))
            {
                if let Some(effect) = def.spell_effect {
                    resolve_effect(state, queue, source, owner, effect, Some(source), None);
                }
            }
            let _ = state.world_mut().move_to_zone(picked, Zone::Graveyard);
        }
        CardEffect::SpendArmorDealDamageAllMinions { max_spend } => {
            // Shellnado — spend up to `max_spend` Armor and deal that much
            // damage to all minions.
            let spend = state.player(owner).armor.min(max_spend);
            if spend <= 0 {
                return;
            }
            {
                let inner = state.make_mut();
                inner.players[owner.index()].armor -= spend;
            }
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            for m in all {
                queue.push(Event::DamageDealt {
                    source,
                    target: m,
                    amount: spend,
                });
            }
        }
        CardEffect::DestroyTopCardDiscoverSameRarity => {
            // Relic Miner — destroy the top card of the deck and Discover
            // a card of the same Rarity (the pool is resolved from the
            // destroyed card's rarity; a card outside the rarity table
            // fizzles the discover).
            let Some(top) = state.world().zones().iter(Zone::Deck, owner).next() else {
                return;
            };
            let Some(cid) = state.world().card_id(top) else {
                return;
            };
            let _ = state.world_mut().move_to_zone(top, Zone::Graveyard);
            let Some(rarity) = crate::cards::pool::rarity_of(cid.0) else {
                return;
            };
            let cards = crate::cards::pool::cards_of_rarity(rarity);
            if cards.is_empty() {
                return;
            }
            let options: Vec<String> = cards
                .iter()
                .map(|c| format!("{} ({})", c.name, c.id))
                .collect();
            let pool_ids: Vec<String> = cards.iter().map(|c| c.id.to_string()).collect();
            state.set_pending_choice_w4a(
                crate::core::state::ChoiceKind::Discover,
                source,
                options,
                pool_ids,
                false,
                1,
                false,
                Vec::new(),
            );
            crate::engine::quest::progress(
                state,
                queue,
                owner,
                crate::cards::quest::QuestCondition::DiscoverCards,
                1,
                None,
            );
        }
        CardEffect::GrantKeyword { keyword, target } => {
            // The choose-one keyword branches of the Ancient Stegodon /
            // Ancient Raptor / Ancient Pterrordax trio.
            let candidates = collect_target_candidates(state, owner, target, source);
            let Some(picked) = select_target(explicit_target, &candidates, state.rng_mut()) else {
                return;
            };
            let world = state.world_mut();
            match keyword {
                KeywordKind::Taunt => world.set_taunt(picked, Taunt),
                KeywordKind::Poisonous => world.set_poison(picked, Poison),
                KeywordKind::Elusive => world.set_elusive(picked, Elusive),
                KeywordKind::Reborn => world.set_reborn(picked, Reborn),
                // Stealth until next turn is approximated as permanent
                // Stealth (fidelity-debt §17).
                KeywordKind::Stealth => world.set_stealth(picked, Stealth),
                KeywordKind::Windfury => world.set_windfury(picked, Windfury),
            }
        }
        CardEffect::GrantDeathrattleSummon {
            card_id,
            count,
            target,
        } => {
            // Ancient Raptor's third branch — grant the target
            // "Deathrattle: Summon `count` copies of a card".
            let candidates = collect_target_candidates(state, owner, target, source);
            let Some(picked) = select_target(explicit_target, &candidates, state.rng_mut()) else {
                return;
            };
            state.world_mut().set_deathrattle(
                picked,
                Deathrattle(CardEffect::SummonMultipleMinions {
                    card_id,
                    count: count as u32,
                }),
            );
        }
        CardEffect::AddRandomWeaponAnotherClassComboAttack { combo_attack } => {
            // Neferset Weaponsmith — a random weapon of another class; it
            // enters with extra Attack while the Combo condition holds
            // (another card was played this turn — the counter at
            // battlecry time excludes this card itself).
            let Some(weapon) =
                crate::cards::pool::random_card(state.rng_mut(), RandomPool::WeaponAnotherClass)
            else {
                return;
            };
            if let Some(e) = add_card_to_hand(state, owner, weapon) {
                if state.player(owner).cards_played_this_turn > 0 {
                    state.world_mut().add_enchantment(
                        e,
                        Enchantment {
                            attack: combo_attack,
                            health: 0,
                            cost: 0,
                            expiry: EnchantmentExpiry::Permanent,
                        },
                    );
                }
            }
        }
        CardEffect::Drain { amount } => {
            // Juvenile Pterrordax — deal `amount` damage to all other
            // minions (either side) and restore that much Health to this
            // minion per minion damaged.
            let others: Vec<Entity> = state
                .world()
                .zones()
                .iter(Zone::Play, owner)
                .chain(state.world().zones().iter(Zone::Play, owner.opponent()))
                .filter(|&e| e != source && state.world().card_type(e) == Some(CardType::Minion))
                .collect();
            for t in &others {
                queue.push(Event::DamageDealt {
                    source,
                    target: *t,
                    amount,
                });
            }
            let restored = amount * others.len() as i32;
            if restored > 0 && heal_char(state.world_mut(), source, restored) {
                fire_healed_trigger(state, queue, source);
            }
        }
        CardEffect::GainStatsEqualFireSpellCost => {
            // Mechanized Magma — the spell_trigger fires on every Friendly
            // SpellCast with the spell as the event subject; only Fire
            // spells grant stats, equal to the spell's effective cost at
            // cast time.
            let Some(subject) = event_subject else {
                return;
            };
            let Some(cid) = state.world().card_id(subject) else {
                return;
            };
            if crate::cards::quest::spell_school(cid.0)
                != Some(crate::cards::quest::SpellSchool::Fire)
            {
                return;
            }
            let cost = state.world().effective_cost(subject).unwrap_or_default().0;
            if cost <= 0 {
                return;
            }
            state.world_mut().add_enchantment(
                source,
                crate::core::component::Enchantment {
                    attack: cost,
                    health: cost,
                    cost: 0,
                    expiry: crate::core::component::EnchantmentExpiry::Permanent,
                },
            );
        }
        CardEffect::DealDamageAndSummon { amount, card_id } => {
            // Gorishi Stinger — deal `amount` damage to the explicit target
            // (or a random enemy) and summon the given minion.
            if let Some(target) = explicit_target {
                resolve_deal_damage(
                    state,
                    queue,
                    source,
                    owner,
                    amount,
                    EffectTarget::AnyEnemy,
                    Some(target),
                );
            } else {
                resolve_deal_damage(
                    state,
                    queue,
                    source,
                    owner,
                    amount,
                    EffectTarget::AnyEnemy,
                    None,
                );
            }
            let _ = resolve_summon(state, queue, source, owner, card_id);
        }
        CardEffect::DiscoverDeckCard => {
            // Cursed Catacombs / Cultist Map — discover a card from the
            // owner's deck: three random DISTINCT deck card ids (the source
            // card excluded), surfaced as the DiscoverDeck choice. The
            // picked deck ENTITY moves to hand at ChoiceResolved; Cursed
            // Catacombs marks it Temporary, Cultist Map runs the Map chain.
            let source_id = state
                .world()
                .card_id(source)
                .map(|c| c.0)
                .unwrap_or_default();
            let mut picked: Vec<&'static str> = Vec::new();
            for e in state.world().zones().iter(Zone::Deck, owner) {
                let Some(cid) = state.world().card_id(e) else {
                    continue;
                };
                if cid.0 == source_id {
                    continue;
                }
                if !picked.contains(&cid.0) {
                    picked.push(cid.0);
                }
                if picked.len() == 3 {
                    break;
                }
            }
            if picked.is_empty() {
                return;
            }
            let ids: Vec<String> = picked.iter().map(|s| s.to_string()).collect();
            let options: Vec<String> = picked
                .iter()
                .map(|id| {
                    crate::cards::def::card_by_id(id)
                        .map(|c| format!("{} ({})", c.name, c.id))
                        .unwrap_or_else(|| id.to_string())
                })
                .collect();
            let temporary = source_id == "TLC_451";
            let map_others = if source_id == "TLC_515" {
                ids.clone()
            } else {
                Vec::new()
            };
            state.set_pending_choice_w4a(
                crate::core::state::ChoiceKind::DiscoverDeck,
                source,
                options,
                ids,
                false,
                1,
                temporary,
                map_others,
            );
            crate::engine::quest::progress(
                state,
                queue,
                owner,
                crate::cards::quest::QuestCondition::DiscoverCards,
                1,
                None,
            );
        }
        CardEffect::DiscoverEnemyDeckTop => {
            // Eyes in the Sky — look at the top 3 cards of the enemy deck,
            // pick one to put on top (the deck is otherwise untouched).
            let enemy = owner.opponent();
            let top3: Vec<&'static str> = state
                .world()
                .zones()
                .iter(Zone::Deck, enemy)
                .take(3)
                .filter_map(|e| state.world().card_id(e).map(|c| c.0))
                .collect();
            if top3.is_empty() {
                return;
            }
            let options: Vec<String> = top3
                .iter()
                .map(|id| {
                    crate::cards::def::card_by_id(id)
                        .map(|c| format!("{} ({})", c.name, c.id))
                        .unwrap_or_else(|| id.to_string())
                })
                .collect();
            let ids: Vec<String> = top3.iter().map(|s| s.to_string()).collect();
            state.set_pending_choice_w4a(
                crate::core::state::ChoiceKind::DiscoverEnemyDeckPutOnTop,
                source,
                options,
                ids,
                false,
                1,
                false,
                Vec::new(),
            );
            crate::engine::quest::progress(
                state,
                queue,
                owner,
                crate::cards::quest::QuestCondition::DiscoverCards,
                1,
                None,
            );
        }
    }
}

/// A random minion of the given cost from ALL_CARDS (the Gravedawn
/// Voidbulb D2 convention — token cards excluded).
fn random_minion_of_cost(
    state: &mut GameState,
    cost: i32,
) -> Option<&'static crate::cards::def::CardDef> {
    let pool: SmallList<&'static crate::cards::def::CardDef> = crate::cards::sets::ALL_CARDS
        .iter()
        .filter(|c| c.card_type == CardType::Minion && c.cost == cost && !c.id.ends_with('t'))
        .collect();
    pool.get(state.rng_mut().next_usize(pool.len())).copied()
}

/// The candidate minions for a target selection (M2-W4a helper — the
/// target kinds used by the new effect arms).
fn collect_target_candidates(
    state: &GameState,
    owner: PlayerId,
    target: EffectTarget,
    source: Entity,
) -> SmallList<Entity> {
    match target {
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner, Some(source)),
        EffectTarget::AnyMinion => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            all
        }
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        EffectTarget::AnyEnemy => collect_enemy_characters(state, owner, Some(source)),
        EffectTarget::Self_ => [source].into_iter().collect(),
        _ => SmallList::new(),
    }
}

/// Grants a random Bonus Effect from the approximated pool — Taunt /
/// Divine Shield / Poisonous / Windfury / Elusive / Stealth (the
/// Dreambound Raptor pool, §14.3).
fn grant_random_bonus(state: &mut GameState, e: Entity) {
    let pick = state.rng_mut().next_usize(6);
    let world = state.world_mut();
    match pick {
        0 => world.set_taunt(e, Taunt),
        1 => world.set_divine_shield(e, DivineShield),
        2 => world.set_poison(e, Poison),
        3 => world.set_windfury(e, Windfury),
        4 => world.set_elusive(e, Elusive),
        _ => world.set_stealth(e, Stealth),
    }
}

/// Reduces a hand card's Cost by `amount` (a permanent cost-reduction
/// enchantment on the card entity).
fn reduce_hand_card_cost(state: &mut GameState, e: Entity, amount: i32) {
    state.world_mut().add_enchantment(
        e,
        Enchantment {
            attack: 0,
            health: 0,
            cost: -amount,
            expiry: EnchantmentExpiry::Permanent,
        },
    );
}

/// Whether the owner's hand holds a minion with a dark gift (Frostburn
/// Matriarch, Cindersword, Dragon Turtle — M1-W5).
fn holding_minion_with_dark_gift(state: &GameState, owner: PlayerId) -> bool {
    state.world().zones().iter(Zone::Hand, owner).any(|e| {
        state.world().card_type(e) == Some(CardType::Minion)
            && state
                .world()
                .dark_gifts(e)
                .is_some_and(|gifts| !gifts.is_empty())
    })
}

/// True when the card definition belongs to no class (Neutral — Envoy of
/// the Glade's transformation predicate, M1-W4a).
fn is_neutral_def(def: &crate::cards::def::CardDef) -> bool {
    use crate::cards::sets::{
        DRUID_CLASSIC, HUNTER_CLASSIC, LEGENDARY_CLASSIC, MAGE_CLASSIC, PALADIN_CLASSIC,
        PRIEST_CLASSIC, ROGUE_CLASSIC, SHAMAN_CLASSIC, WARLOCK_CLASSIC, WARRIOR_CLASSIC,
    };
    ![
        DRUID_CLASSIC,
        HUNTER_CLASSIC,
        MAGE_CLASSIC,
        PALADIN_CLASSIC,
        PRIEST_CLASSIC,
        ROGUE_CLASSIC,
        SHAMAN_CLASSIC,
        WARLOCK_CLASSIC,
        WARRIOR_CLASSIC,
        LEGENDARY_CLASSIC,
    ]
    .iter()
    .any(|list| list.iter().any(|c| c.id == def.id))
}

/// True when the entity's card definition carries a Deathrattle (the
/// Felhunter/Felbat resurrection predicate, M1-W4a).
fn has_deathrattle_def(state: &GameState, entity: Entity) -> bool {
    state
        .world()
        .card_id(entity)
        .and_then(|c| crate::cards::def::card_by_id(c.0))
        .is_some_and(|d| d.deathrattle.is_some() || d.death_trigger.is_some())
}

/// Picks a random dark gift from the fixed ten-gift pool
/// (`component::ALL_DARK_GIFTS`).
fn random_dark_gift(rng: &mut GameRng) -> DarkGiftKind {
    ALL_DARK_GIFTS[rng.next_usize(ALL_DARK_GIFTS.len())]
}

/// Adds a fresh minion card to the hand (F-A11: a full hand creates
/// nothing). Returns the added entity, or `None` when the hand is full.
fn add_random_minion_to_hand(
    state: &mut GameState,
    owner: PlayerId,
    def: &'static crate::cards::def::CardDef,
) -> Option<Entity> {
    let hand_before = state.world().zones().len(Zone::Hand, owner);
    add_card_to_hand(state, owner, def);
    if state.world().zones().len(Zone::Hand, owner) <= hand_before {
        return None;
    }
    state
        .world()
        .zones()
        .iter(Zone::Hand, owner)
        .nth(hand_before)
}

/// Applies a dark gift to a minion (2025–2026 expansions M1-W2 — the
/// Emerald Dream dark-gift mechanic): records the marker on the entity
/// (`World` `dark_gifts` — persists across zones), logs the kind in the
/// source player's `dark_gifts_given` list (Wallow, the Wretched reads the
/// log), applies the static effects, and syncs the gift onto every friendly
/// Wallow in the hand or deck. Gifts are deduplicated per entity.
pub(crate) fn apply_dark_gift(
    state: &mut GameState,
    target: Entity,
    gift: DarkGiftKind,
    source: PlayerId,
) {
    // The marker is the card-level source of truth (deduplicated)
    if state.world().has_dark_gift(target, gift) {
        return;
    }
    state.world_mut().add_dark_gift(target, gift);
    // Log the kind only (registered simplification: no targets —
    // fidelity-debt §14)
    state.make_mut().players[source.index()]
        .dark_gifts_given
        .push(gift);
    // Static effects (enchantments + keyword components)
    grant_dark_gift_static(state, target, gift);
    // Wallow sync — every friendly Wallow, the Wretched in the hand or deck
    // gains a copy of the gift (registered simplification: not retroactive —
    // fidelity-debt §14; copies are not re-logged)
    let wallows: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Hand, source)
        .chain(state.world().zones().iter(Zone::Deck, source))
        .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "EDR_487"))
        .collect();
    for wallow in wallows {
        if wallow == target || state.world().has_dark_gift(wallow, gift) {
            continue;
        }
        state.world_mut().add_dark_gift(wallow, gift);
        grant_dark_gift_static(state, wallow, gift);
    }
}

/// Applies a dark gift's static effects (enchantments + keyword components)
/// to the entity. The behavioral gifts (SummonCopyOnPlay, BattlecryTwice,
/// RebornFull) are resolved by the `engine/rules.rs` hooks reading the
/// marker, not here.
fn grant_dark_gift_static(state: &mut GameState, target: Entity, gift: DarkGiftKind) {
    match gift {
        DarkGiftKind::AttackLifesteal => {
            let world = state.world_mut();
            world.add_enchantment(
                target,
                Enchantment {
                    attack: 3,
                    health: 0,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
            world.set_lifesteal(target, Lifesteal);
        }
        DarkGiftKind::StatsElusive => {
            let world = state.world_mut();
            world.add_enchantment(
                target,
                Enchantment {
                    attack: 2,
                    health: 2,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
            world.set_elusive(target, Elusive);
        }
        DarkGiftKind::CostDiscount => {
            // Cost (2) less, -2 attack (the official "only if attack stays
            // at least 1" filter is not applied — registered simplification)
            let world = state.world_mut();
            world.add_enchantment(
                target,
                Enchantment {
                    attack: -2,
                    health: 0,
                    cost: -2,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
        }
        DarkGiftKind::Charge => {
            state.world_mut().set_charge(target, Charge);
        }
        DarkGiftKind::SummonCopyOnPlay => {
            // Behavioral — the MinionSummoned hook in `engine/rules.rs`
            // reads the marker.
        }
        DarkGiftKind::BattlecryTwice => {
            // Behavioral — the MinionSummoned hook in `engine/rules.rs`
            // reads the marker.
        }
        DarkGiftKind::HealthTaunt => {
            let world = state.world_mut();
            world.add_enchantment(
                target,
                Enchantment {
                    attack: 0,
                    health: 4,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
            world.set_taunt(target, Taunt);
        }
        DarkGiftKind::RebornFull => {
            state.world_mut().set_reborn(target, Reborn);
        }
        DarkGiftKind::DeckTopBuff => {
            // +4/+5 and on top of the deck. The move runs first so the buff
            // survives a play-zone target (the Play → elsewhere wipe clears
            // enchantments added before the move; buffing after keeps them).
            if let Some(player) = state.world().player(target) {
                let _ = state.world_mut().move_to_zone(target, Zone::Deck);
                state
                    .world_mut()
                    .zones_mut()
                    .remove(Zone::Deck, player, target);
                state
                    .world_mut()
                    .zones_mut()
                    .insert_at(Zone::Deck, player, target, 0);
            }
            let world = state.world_mut();
            world.add_enchantment(
                target,
                Enchantment {
                    attack: 4,
                    health: 5,
                    cost: 0,
                    expiry: EnchantmentExpiry::Permanent,
                },
            );
        }
        DarkGiftKind::ShieldWindfury => {
            let world = state.world_mut();
            world.set_divine_shield(target, DivineShield);
            world.set_windfury(target, Windfury);
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
        EffectTarget::AnyMinionAttackLE(limit) => {
            // A minion on either side with attack ≤ N (Twilight Influence,
            // M1-W3 — "Destroy a minion with 3 or less Attack"): an explicit
            // target is required; without one the effect fizzles (the
            // any-minion convention). Effective attack — enchantments and
            // auras count.
            let t = explicit?;
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_enemy_minions(state, owner, None));
            if !all.contains(&t)
                || state
                    .world()
                    .effective_attack(t)
                    .is_none_or(|a| a.0 > limit)
            {
                return None;
            }
            [t].into_iter().collect()
        }
        // M2-W4a target kinds (the Un'Goro main set):
        EffectTarget::DamagedOtherFriendlyMinion => {
            // A friendly minion other than the source that is currently
            // damaged (accumulated damage > 0).
            collect_friendly_minions(state, owner)
                .into_iter()
                .filter(|&e| e != source && state.world().damage(e).is_some_and(|d| d.0 > 0))
                .collect()
        }
        EffectTarget::EnemyMinionWithRace => {
            // An enemy minion that HAS a minion type (Bugsquasher, M2-W4a
            // — "an enemy minion with a minion type").
            collect_enemy_minions(state, owner, None)
                .into_iter()
                .filter(|&e| state.world().race(e).is_some_and(|r| !r.is_empty()))
                .collect()
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
        // Stealth / Elusive (M5 CardDef fields — mirrored here so
        // effect-summoned tokens like the Web of Deception Spider keep the
        // keywords; the play path applies them via spawn_card_from_def)
        if card_def.stealth {
            world.set_stealth(e, crate::core::component::Stealth);
        }
        if card_def.elusive {
            world.set_elusive(e, crate::core::component::Elusive);
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
        // A random friendly DAMAGED minion other than the source
        // (Stonecarver — "another friendly damaged minion")
        EffectTarget::DamagedOtherFriendlyMinion => {
            let minions: SmallList<Entity> = collect_friendly_minions(state, owner)
                .into_iter()
                .filter(|&e| e != source && state.world().damage(e).is_some_and(|d| d.0 > 0))
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
        // A random minion on either side (Ancestral Healing, the M2-W4b
        // Ido token's "+2/+2 and Divine Shield")
        EffectTarget::AnyMinion => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            let Some(m) = select_target(explicit, &all, state.rng_mut()) else {
                return;
            };
            state.world_mut().add_enchantment(m, buff);
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

/// The Emerald Dream imbue mechanic (2025–2026 expansions M1-W1): the
/// player's imbue count +1; the FIRST imbue replaces the hero power with the
/// class's imbued form (cost 2) when the hero is one of the six imbuing
/// classes (Druid/Hunter/Mage/Paladin/Priest/Shaman — detected via the
/// hero's card ID). Later imbues never replace again — they only scale the
/// imbued power's numbers, because the level is read from the count at
/// resolution time. A hero of another class (or without a card ID) counts
/// imbues without any replacement.
fn resolve_imbue(state: &mut GameState, owner: PlayerId) {
    let count = {
        let inner = state.make_mut();
        let p = &mut inner.players[owner.index()];
        p.imbue_count += 1;
        p.imbue_count
    };
    if count == 1 {
        let hero = state.player(owner).hero;
        let class = state
            .world()
            .card_id(hero)
            .and_then(|c| ImbueClass::from_hero_card_id(c.0));
        if let Some(class) = class {
            state.world_mut().set_hero_power(
                hero,
                HeroPowerDef {
                    cost: 2,
                    effect: CardEffect::ImbuedHeroPower { class },
                },
            );
        }
    }
}

/// Resolves the owner's current hero power effect once, free of charge: no
/// mana is spent and the hero power is not marked used (Wisprider's
/// "trigger it"). The hero is the effect source, so the Spell Damage /
/// Velen pipeline applies exactly as for a normal hero power activation.
fn resolve_use_hero_power(state: &mut GameState, queue: &mut EventQueue, owner: PlayerId) {
    let hero = state.player(owner).hero;
    let Some(hp) = state.world().hero_power(hero) else {
        return;
    };
    resolve_effect(state, queue, hero, owner, hp.effect, None, None);
}

/// Resolves an imbued hero power at level L = the owner's imbue count (the
/// six classes' EDR_*p hero powers; numbers per the W1 design brief). The
/// Mage damage re-enters `resolve_effect` so the Spell Damage / Velen
/// pipeline boosts each missile exactly as for a base hero power.
fn resolve_imbued_hero_power(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    class: ImbueClass,
) {
    let level = state.player(owner).imbue_count.max(1);
    match class {
        ImbueClass::Druid => {
            // Blessing of the Golem (EDR_847p) — summon an L/L plant golem
            // (the token is a 1/1 base; the level overrides its stats)
            if let Some(golem) = resolve_summon(state, queue, source, owner, "EDR_847pt2") {
                state.world_mut().set_attack(golem, Attack(level));
                state.world_mut().set_health(golem, Health(level));
            }
        }
        ImbueClass::Hunter => {
            // Blessing of the Wolf (EDR_850p) — a random Beast in hand gains
            // +L Attack and costs (L) less (hand buffs write the base
            // components, FordragonBuff convention)
            let beasts: SmallList<Entity> = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .filter(|&e| {
                    state
                        .world()
                        .card_id(e)
                        .and_then(|c| crate::cards::def::card_by_id(c.0))
                        .is_some_and(|d| d.race == Some(crate::core::component::Race::Beast))
                })
                .collect();
            if beasts.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(beasts.len());
            let beast = beasts[idx];
            let world = state.world_mut();
            let base = world.attack(beast).unwrap_or(Attack(0));
            world.set_attack(beast, Attack(base.0 + level));
            let cur = world.effective_cost(beast).unwrap_or(Cost(0));
            world.set_cost(beast, Cost((cur.0 - level).max(0)));
        }
        ImbueClass::Mage => {
            // Blessing of the Wisp (EDR_851p) — summon L Wisps and deal L
            // damage randomly split among all enemies (L single pings)
            for _ in 0..level {
                let _ = resolve_summon(state, queue, source, owner, "EDR_851t");
            }
            resolve_effect(
                state,
                queue,
                source,
                owner,
                CardEffect::DealDamageRandomly {
                    amount: 1,
                    count: level,
                    target: EffectTarget::AnyEnemy,
                },
                None,
                None,
            );
        }
        ImbueClass::Paladin => {
            // Blessing of the Dragon (EDR_445p) — shuffle two Emerald
            // Portals into the deck at random positions
            for _ in 0..2 {
                let Some(def) = crate::cards::def::card_by_id("EDR_445pt3") else {
                    return;
                };
                let e = crate::cards::spawn_card_from_def(state.world_mut(), owner, def);
                state.world_mut().set_zone(e, Zone::Deck);
                let deck_len = state.world().zones().len(Zone::Deck, owner);
                let pos = state.rng_mut().next_usize(deck_len + 1);
                state
                    .world_mut()
                    .zones_mut()
                    .insert_at(Zone::Deck, owner, e, pos);
            }
        }
        ImbueClass::Priest => {
            // Blessing of the Moon (EDR_449p, simplified — the real card
            // lets the player choose; fidelity-debt §14) — a random Priest
            // minion or spell to hand, costing (L) less
            let Some(def) =
                crate::cards::pool::random_card(state.rng_mut(), RandomPool::PriestCard)
            else {
                return;
            };
            let hand_before = state.world().zones().len(Zone::Hand, owner);
            add_card_to_hand(state, owner, def);
            if state.world().zones().len(Zone::Hand, owner) <= hand_before {
                return; // hand full (F-A11) — nothing was added
            }
            let added = state
                .world()
                .zones()
                .iter(Zone::Hand, owner)
                .nth(hand_before)
                .expect("the generated Priest card is in hand");
            let cur = state.world().effective_cost(added).unwrap_or(Cost(0));
            state
                .world_mut()
                .set_cost(added, Cost((cur.0 - level).max(0)));
        }
        ImbueClass::Shaman => {
            // Blessing of the Wind (EDR_448p) — transform a random friendly
            // minion into a random minion costing (L) more (token-excluded
            // pool, Silvermoon Portal convention; the reset follows the Hex
            // transform pattern)
            let minions = collect_friendly_minions(state, owner);
            if minions.is_empty() {
                return;
            }
            let idx = state.rng_mut().next_usize(minions.len());
            let target = minions[idx];
            let target_cost = state.world().cost(target).unwrap_or(Cost(0)).0;
            let candidates: SmallList<&'static crate::cards::def::CardDef> =
                crate::cards::sets::ALL_CARDS
                    .iter()
                    .filter(|c| {
                        c.card_type == CardType::Minion
                            && c.cost == target_cost + level
                            && !c.id.ends_with('t')
                    })
                    .collect();
            if candidates.is_empty() {
                return;
            }
            let pick = candidates[state.rng_mut().next_usize(candidates.len())];
            let world = state.world_mut();
            crate::cards::clear_minion_effects(world, target);
            world.set_attack(target, Attack(pick.attack));
            world.set_health(target, Health(pick.health));
            world.set_cost(target, Cost(pick.cost));
            world.set_card_id(target, CardId(pick.id));
            world.set_attacks_used(target, AttacksUsed(0));
        }
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
        // FriendlyMinion (M1-W4a — Siphoning Growth, Divination): destroy a
        // friendly minion (the explicit play target, or a random one)
        EffectTarget::FriendlyMinion => collect_friendly_minions(state, owner),
        EffectTarget::AnyEnemyMinion => collect_enemy_minions(state, owner, Some(source)),
        // A minion on either side (Ravenous Devilsaur — "destroy a minion";
        // the friendly side is legal too, mirroring resolve_silence's scope)
        EffectTarget::AnyMinion => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            all
        }
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
        // A minion on either side with attack ≤ N (Twilight Influence, M1-W3 —
        // "Destroy a minion with 3 or less Attack" targets either side)
        EffectTarget::AnyMinionAttackLE(max_atk) => {
            let mut all = collect_friendly_minions(state, owner);
            all.extend(collect_all_enemy_minions(state, owner));
            all.into_iter()
                .filter(|&e| {
                    state
                        .world()
                        .effective_attack(e)
                        .is_some_and(|a| a.0 <= max_atk)
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
    // Crater Gator (M2-W4a): a hero whose player has
    // `enemy_hero_cant_be_healed` set cannot be healed at all (the flag is
    // set by the Gator's battlecry on the opponent of its owner and clears
    // at the owner's next turn start).
    let hero_heal_blocked = |state: &GameState, entity: Entity| -> bool {
        if state.world().card_type(entity) != Some(CardType::Hero) {
            return false;
        }
        let Some(h_owner) = state.world().player(entity) else {
            return false;
        };
        state.player(h_owner).enemy_hero_cant_be_healed
    };
    // Single-pick scope (Earthen Ring Farseer, Voodoo Doctor — W15: any
    // character): explicit target wins, random at resolution, G9 fizzle.
    if target == EffectTarget::AnyCharacter {
        let mut chars = collect_friendly_characters(state, owner, None);
        chars.extend(collect_enemy_characters(state, owner, None));
        if let Some(c) = select_target(explicit, &chars, state.rng_mut()) {
            if !hero_heal_blocked(state, c) && heal(state.world_mut(), c, amount) {
                healed.push(c);
            }
        }
    } else {
        match target {
            EffectTarget::FriendlyHero | EffectTarget::Self_ => {
                let hero = state.player(owner).hero;
                if !hero_heal_blocked(state, hero) && heal(state.world_mut(), hero, amount) {
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
                let hero_blocked = hero_heal_blocked(state, hero);
                let minions = collect_friendly_minions(state, owner);
                let world = state.world_mut();
                if !hero_blocked && heal(world, hero, amount) {
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
    if healed.is_empty() {
        return;
    }
    // Wilted Shadow (M2-W4a): "Whenever you heal an enemy, this attacks
    // it" — a friendly Wilted Shadow attacks each enemy character this
    // effect actually healed (the attack runs through the normal attack
    // pipeline, per the ForceEnemyMinionsAttackThis convention).
    let shadows: SmallList<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, owner)
        .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "TLC_821"))
        .collect();
    for entity in healed {
        fire_healed_trigger(state, queue, entity);
        if state.world().player(entity) != Some(owner.opponent()) {
            continue;
        }
        for shadow in &shadows {
            let atk = crate::engine::rules::compute_attacker_damage(state, *shadow);
            queue.push(Event::AttackDeclared {
                attacker: *shadow,
                defender: entity,
            });
            queue.push(Event::ResolveAttack {
                attacker: *shadow,
                defender: entity,
                attacker_damage: atk,
                retaliation_immune: false,
            });
        }
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

/// Discards a random card from hand (Story of Lakkari's end-of-turn discard
/// — M2-W4a — fires it from the rules handler).
pub(crate) fn resolve_discard_random(state: &mut GameState, owner: PlayerId) {
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

/// Adds a card entity to hand (creates a new entity; used for random
/// generation / Antonidas). Returns the new entity so callers can mark it
/// (Temporary, cost enchantments — 2025–2026 expansions M2-W4a).
pub(crate) fn add_card_to_hand(
    state: &mut GameState,
    player: PlayerId,
    card_def: &crate::cards::def::CardDef,
) -> Option<Entity> {
    // Hand-size cap (F-A11): a GENERATED card past the 10-card limit is never
    // created (official rule — a full-hand add destroys the card).
    if state.world().zones().len(Zone::Hand, player) >= MAX_HAND_SIZE {
        return None;
    }
    let world = state.world_mut();
    let e = crate::cards::spawn_card_from_def(world, player, card_def);
    world.set_zone(e, Zone::Hand);
    world.zones_mut().insert(Zone::Hand, player, e);
    Some(e)
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
