//! Card definition module — basic card data.
//!
//! Contains the CardDef struct, the vanilla! macro, and all Classic card definitions.
//! All card constants are re-exported through the `def` module,
//! accessible to external code via `crate::cards::def::*`.

pub mod classic_druid;
pub mod classic_hunter;
pub mod classic_legendary;
pub mod classic_mage;
pub mod classic_neutral;
pub mod classic_paladin;
pub mod classic_priest;
pub mod classic_rogue;
pub mod classic_shaman;
pub mod classic_warlock;
pub mod classic_warrior;
pub mod def;
pub mod generated;
pub mod pool;
pub mod sets;

use crate::core::component::{
    Attack, AttacksUsed, Aura, CardId, Cost, Deathrattle, Durability, Health, Overload, Poison,
    Stealth, Trigger, TriggerEvent, TriggerTiming,
};
use crate::core::effect::{CardEffect, EffectTarget};
use crate::core::entity::Entity;
use crate::core::player::PlayerId;
use crate::core::world::World;
use def::CardDef;

/// Applies special keyword components (Poison, Stealth, Overload, etc.) to an entity.
///
/// These keywords do not add `CardDef` fields (to avoid large struct changes);
/// instead they are mapped here centrally by card ID. Called when summoning minions
/// (`trigger::resolve_summon`) and building cards (`GameBuilder::spawn_minion`).
pub(crate) fn apply_card_keywords(world: &mut World, entity: Entity, card_def: &CardDef) {
    // Shaman cards with Overload (mana lock not simulated; used only as a trigger marker)
    if matches!(
        card_def.id,
        "SHAMAN_002"
            | "SHAMAN_006"
            | "SHAMAN_009"
            | "SHAMAN_015"
            | "SHAMAN_016"
            | "SHAMAN_017"
            | "SHAMAN_019"
    ) {
        world.set_overload(entity, Overload);
    }
    if card_def.id == "SHAMAN_021" {
        // Unbound Elemental — gain +1/+1 whenever you play a card with Overload
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyOverloadPlayed,
                timing: TriggerTiming::Whenever,
                effect: CardEffect::GainStats {
                    attack: 1,
                    health: 1,
                    target: EffectTarget::Self_,
                },
            },
        );
    }
    if card_def.id == "NEUTRAL_004" {
        // Acolyte of Pain — whenever this minion takes damage, draw a card
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::ThisMinionDamaged,
                timing: TriggerTiming::Whenever,
                effect: CardEffect::DrawCard { count: 1 },
            },
        );
    }
    if card_def.id == "WARRIOR_007" {
        // Frothing Berserker — whenever a friendly minion takes damage, gain +1 attack
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionDamaged,
                timing: TriggerTiming::Whenever,
                effect: CardEffect::GainStats {
                    attack: 1,
                    health: 0,
                    target: EffectTarget::Self_,
                },
            },
        );
    }
    if card_def.id == "WARRIOR_013" {
        // Armorsmith — whenever a friendly minion takes damage, gain 1 armor
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionDamaged,
                timing: TriggerTiming::Whenever,
                effect: CardEffect::GainArmor {
                    amount: 1,
                    target: EffectTarget::FriendlyHero,
                },
            },
        );
    }
    if card_def.id == "ROGUE_022" {
        // Patient Assassin — Stealth + Poison
        world.set_poison(entity, Poison);
        world.set_stealth(entity, Stealth);
    }
}

/// Clears all effect components of a minion (resets the entity before transform).
///
/// Keeps base components (Health/Attack/zone, etc.); the caller re-applies the target card's attributes.
pub(crate) fn clear_minion_effects(world: &mut World, entity: Entity) {
    world.remove_battlecry(entity);
    world.remove_deathrattle(entity);
    world.remove_taunt(entity);
    world.remove_aura(entity);
    world.remove_secret(entity);
    world.remove_divine_shield(entity);
    world.remove_windfury(entity);
    world.remove_charge(entity);
    world.remove_spell_damage(entity);
    world.remove_cant_attack(entity);
    world.remove_trigger(entity);
    world.remove_choose_one_effect(entity);
    // Enchantments and damage (roadmap G4): transform resets to the new base
    world.remove_enchantments(entity);
    world.remove_damage(entity);
    world.remove_combo_effect(entity);
    world.remove_attack_equals_health(entity);
    world.remove_poison(entity);
    world.remove_stealth(entity);
    world.remove_immune(entity);
    world.remove_freeze(entity);
    world.remove_overload(entity);
}

/// Creates a card entity from a `CardDef` under the given player (zone not set).
///
/// Shared by `GameBuilder` and the effect system (e.g., King Mukla's Banana, random pools),
/// ensuring card entities in hand/deck carry the full set of components (Battlecry/Deathrattle/keywords, etc.).
pub(crate) fn spawn_card_from_def(world: &mut World, player: PlayerId, card: &CardDef) -> Entity {
    let e = world.spawn();
    world.set_card_id(e, CardId(card.id));
    world.set_health(e, Health(card.health));
    world.set_attack(e, Attack(card.attack));
    world.set_cost(e, Cost(card.cost));
    world.set_card_type(e, card.card_type);
    world.set_player(e, player);
    world.set_attacks_used(e, AttacksUsed(0));
    // Set weapon durability (if it is a weapon card)
    if card.card_type == crate::core::component::CardType::Weapon && card.durability > 0 {
        world.set_durability(e, Durability(card.durability));
    }
    // Set Divine Shield / Windfury / Charge / Spell Damage
    if card.divine_shield {
        world.set_divine_shield(e, crate::core::component::DivineShield);
    }
    if card.windfury {
        world.set_windfury(e, crate::core::component::Windfury);
    }
    if card.charge {
        world.set_charge(e, crate::core::component::Charge);
    }
    if card.spell_damage != 0 {
        world.set_spell_damage(e, crate::core::component::SpellDamage(card.spell_damage));
    }
    // Set aura (if any)
    if let Some((aura_effect, aura_target)) = card.aura {
        world.set_aura(
            e,
            Aura {
                effect: aura_effect,
                target: aura_target,
            },
        );
    }
    // Set Battlecry / Deathrattle (existing fields)
    if let Some(bc) = card.battlecry {
        world.set_battlecry(e, crate::core::component::Battlecry(bc));
    }
    if let Some(dr) = card.deathrattle {
        world.set_deathrattle(e, Deathrattle(dr));
    }
    // Set Taunt
    if card.taunt {
        world.set_taunt(e, crate::core::component::Taunt);
    }
    // Set cannot-attack
    if card.cant_attack {
        world.set_cant_attack(e, crate::core::component::CantAttack);
    }
    // Register triggers (roadmap G2): CardDef trigger fields map to the unified
    // Trigger component, fired in play order with current-player precedence.
    if let Some(ete) = card.end_turn_effect {
        world.set_trigger(
            e,
            Trigger {
                event: TriggerEvent::TurnEnd,
                timing: TriggerTiming::Whenever,
                effect: ete,
            },
        );
    }
    if let Some(ste) = card.start_turn_effect {
        world.set_trigger(
            e,
            Trigger {
                event: TriggerEvent::TurnStart,
                timing: TriggerTiming::Whenever,
                effect: ste,
            },
        );
    }
    // Spell card effects are stored in the battlecry component (resolved by the engine when played)
    if let Some(se) = card.spell_effect {
        world.set_battlecry(e, crate::core::component::Battlecry(se));
    }
    if let Some(st) = card.spell_trigger {
        world.set_trigger(
            e,
            Trigger {
                event: TriggerEvent::FriendlySpellCast,
                timing: TriggerTiming::Whenever,
                effect: st,
            },
        );
    }
    if let Some(dt) = card.death_trigger {
        world.set_trigger(
            e,
            Trigger {
                event: TriggerEvent::FriendlyMinionDied,
                timing: TriggerTiming::Whenever,
                effect: dt,
            },
        );
    }
    if let Some(st) = card.summon_trigger {
        world.set_trigger(
            e,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                effect: st,
            },
        );
    }
    // Choose One effect
    if let Some(ce) = card.choose_one_effect {
        world.set_choose_one_effect(e, crate::core::component::ChooseOneEffect(ce));
    }
    // Combo effect
    if let Some(cb) = card.combo_effect {
        world.set_combo_effect(e, crate::core::component::ComboEffect(cb));
    }
    // Attack equals Health
    if card.attack_equals_health {
        world.set_attack_equals_health(e, crate::core::component::AttackEqualsHealth);
    }
    // Special keywords (Poison/Stealth, etc.)
    apply_card_keywords(world, e, card);
    e
}

#[cfg(test)]
mod generated_tests {
    use super::def::{CardDef, card_by_id};
    use crate::cards::generated;

    /// Generated card constants must match the handwritten constants field by field (for statically representable parts).
    #[test]
    fn generated_cards_match_handwritten() {
        assert!(
            !generated::GENERATED_IDS.is_empty(),
            "generated registry must be non-empty"
        );
        for id in generated::GENERATED_IDS {
            let generated: CardDef = match find_generated(id) {
                Some(c) => c,
                None => panic!("generated const for {id} missing"),
            };
            let handwritten = card_by_id(id).unwrap_or_else(|| panic!("handwritten {id} missing"));
            // Static fields must match exactly; effect fields (Battlecry/Deathrattle, etc.) are always None in generated code,
            // so only cards whose static fields are fully representable are asserted equal
            assert_eq!(generated.id, handwritten.id);
            assert_eq!(generated.name, handwritten.name);
            assert_eq!(generated.card_type, handwritten.card_type);
            assert_eq!(generated.cost, handwritten.cost);
            assert_eq!(generated.attack, handwritten.attack);
            assert_eq!(generated.health, handwritten.health);
            assert_eq!(generated.durability, handwritten.durability);
            assert_eq!(generated.taunt, handwritten.taunt);
            assert_eq!(generated.divine_shield, handwritten.divine_shield);
            assert_eq!(generated.windfury, handwritten.windfury);
            assert_eq!(generated.charge, handwritten.charge);
            assert_eq!(generated.spell_damage, handwritten.spell_damage);
        }
    }

    /// Purely static cards (no effect fields) should be fully equal.
    #[test]
    fn vanilla_generated_cards_fully_equal() {
        use crate::cards::def::BLOODFEN_RAPTOR;
        assert_eq!(
            find_generated("CLASSIC_001").unwrap(),
            BLOODFEN_RAPTOR,
            "vanilla card must be exactly equal"
        );
    }

    fn find_generated(id: &str) -> Option<CardDef> {
        // Locate the generated constant via the registry and naming rules
        match id {
            "CLASSIC_001" => Some(generated::CLASSIC_001),
            "NEUTRAL_B02" => Some(generated::NEUTRAL_B02),
            "NEUTRAL_013" => Some(generated::NEUTRAL_013),
            "CLASSIC_014" => Some(generated::CLASSIC_014),
            _ => None,
        }
    }
}
