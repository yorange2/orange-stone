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
pub mod core_w1;
pub mod core_w2;
pub mod core_w3a;
pub mod core_w3b;
pub mod def;
pub mod generated;
pub mod pool;
pub mod sets;

use crate::core::component::{
    Attack, AttacksUsed, Aura, CardId, Cost, Deathrattle, Durability, Enrage, Health, Lifesteal,
    Overload, Poison, Reborn, Rush, Stealth, Tradeable, Trigger, TriggerEvent, TriggerTiming,
};
use crate::core::effect::{CardEffect, EffectTarget};
use crate::core::entity::Entity;
use crate::core::player::PlayerId;
use crate::core::world::World;
use def::CardDef;

/// Looks up a card by its ID (first match in `ALL_CARDS`, which deduplicates IDs).
///
/// Used by the RL environment and Python bindings to build explicit decks
/// (roadmap M1-G2); `None` for unknown IDs.
#[must_use]
pub fn card_by_id(id: &str) -> Option<&'static CardDef> {
    sets::ALL_CARDS.iter().find(|c| c.id == id)
}

#[cfg(test)]
mod lookup_tests {
    use super::card_by_id;
    use crate::cards::def::BLOODFEN_RAPTOR;

    #[test]
    fn card_by_id_resolves_known_and_unknown() {
        let card = card_by_id("CLASSIC_001").expect("Bloodfen Raptor is in ALL_CARDS");
        assert_eq!(card.id, BLOODFEN_RAPTOR.id);
        assert_eq!(card.name, "Bloodfen Raptor");
        assert!(card_by_id("NOT_A_REAL_CARD_999").is_none());
        assert!(card_by_id("").is_none());
    }
}

/// Applies special keyword components (Poison, Stealth, Overload, etc.) to an entity.
///
/// These keywords do not add `CardDef` fields (to avoid large struct changes);
/// instead they are mapped here centrally by card ID. Called when summoning minions
/// (`trigger::resolve_summon`) and building cards (`GameBuilder::spawn_minion`).
///
/// Pool-open triggers registered through this hook (no `CardDef` field to live
/// in): the sets.rs closure test treats these IDs as pool-open as well. The
/// registration itself is per-ID in `apply_card_keywords` (Cho's AnySpellCast
/// copy, Shaku's Attacked copy) — this list is the test's second source of
/// truth only.
#[allow(dead_code)] // consumed by the sets.rs registry test (cfg(test))
pub(crate) const POOL_OPEN_KEYWORD_IDS: &[&str] = &[
    "LEGENDARY_024", // Lorewalker Cho — AnySpellCast copy trigger
    "CORE_EX1_100",  // Lorewalker Cho (Core Set W3a) — same trigger
    "CORE_CFM_781",  // Shaku, the Collector — Attacked copy trigger (registered below)
];

pub(crate) fn apply_card_keywords(world: &mut World, entity: Entity, card_def: &CardDef) {
    // Core Set W1 keywords — RUSH / LIFESTEAL / REBORN as components (Core
    // Set W1 primitives; the `CardDef` struct stays untouched per the
    // "avoid large struct changes" convention, matching Poison/Overload).
    if matches!(
        card_def.id,
        // Rush (5)
        "CORE_BT_156"   // Imprisoned Vilefiend
        | "CORE_DRG_079" // Evasive Wyrm
        | "CORE_RLK_657" // Underking
        | "CORE_TRL_900" // Halazzi, the Lynx
        | "CORE_WC_701" // Felrattler
    ) {
        world.set_rush(entity, Rush);
    }
    if matches!(
        card_def.id,
        // Lifesteal (8)
        "CORE_BAR_311"   // Devouring Plague (spell)
        | "CORE_BT_801"  // Eye Beam (spell)
        | "CORE_BT_921"  // Aldrachi Warblades (weapon)
        | "CORE_GIL_558" // Swamp Leech
        | "CORE_ICC_055" // Drain Soul (spell)
        | "CORE_ICC_214" // Obsidian Statue
        | "CORE_SW_442"  // Void Shard (spell)
        | "CORE_TTN_866" // Mythical Terror
    ) {
        world.set_lifesteal(entity, Lifesteal);
    }
    if matches!(card_def.id, "CORE_RLK_745" | "CORE_ULD_723") {
        // Reborn (2): Malignant Horror, Murmy
        world.set_reborn(entity, Reborn);
    }
    if card_def.id == "CORE_TTN_866" {
        // Mythical Terror — dual tribe Demon + Beast (the CardDef carries
        // the primary Demon; the Beast half lands here, Core Set W1)
        world.add_race(entity, crate::core::component::Race::Beast);
    }
    // Tradeable (Core Set W2) — the 6 Tradeable cards: shuffle-for-1-and-draw
    if matches!(
        card_def.id,
        "CORE_EX1_002"   // The Black Knight
        | "CORE_EX1_005" // Big Game Hunter
        | "CORE_REV_023" // Demolition Renovator
        | "CORE_SW_066"  // Royal Librarian
        | "CORE_SW_072"  // Rustrot Viper
        | "CORE_SW_429" // Best in Shell
    ) {
        world.set_tradeable(entity, Tradeable);
    }
    // Core Set W3a triggers — Acolyte of Pain (draw on damage), Murloc
    // Tidecaller (attack on friendly Murloc summon), Frothing Berserker
    // (attack on friendly minion damage), Archmage Antonidas (Fireball on
    // friendly spell cast — declared via spell_trigger, no hook needed).
    if card_def.id == "CORE_EX1_007" {
        // Acolyte of Pain — whenever this minion takes damage, draw a card
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::ThisMinionDamaged,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::DrawCard { count: 1 },
            },
        );
    }
    if card_def.id == "CORE_EX1_509" {
        // Murloc Tidecaller — whenever you summon a Murloc, gain +1 Attack
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                race: Some(crate::core::component::Race::Murloc),
                max_attack: None,
                effect: CardEffect::GainStats {
                    attack: 1,
                    health: 0,
                    target: EffectTarget::Self_,
                },
            },
        );
    }
    if card_def.id == "CORE_CFM_344" {
        // Finja, the Flying Star — when this minion attacks, summon a random
        // Murloc from the deck (the subject check lives in the effect)
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::Attacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::SummonRandomFishFromDeck,
            },
        );
    }
    if card_def.id == "CORE_CFM_781" {
        // Shaku, the Collector — when this minion attacks, copy a random
        // enemy deck card (pool-open; registered in POOL_OPEN_CARDS)
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::Attacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::CopyEnemyDeckCardOnSelfAttack,
            },
        );
    }
    if card_def.id == "CORE_EX1_604" {
        // Frothing Berserker — whenever a friendly minion takes damage, +1 Attack
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionDamaged,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GainStats {
                    attack: 1,
                    health: 0,
                    target: EffectTarget::Self_,
                },
            },
        );
    }
    // Shaman cards with Overload — the amount locks mana on the owner's next
    // turn (roadmap F1): Lightning Bolt 1, Lightning Storm 2, Feral Spirit 2,
    // Dust Devil 2, Forked Lightning 2, Lava Burst 2, Stormforged Axe 1,
    // Doomhammer 2, Earth Elemental 3. Keyed by actual card IDs (F-A8 fix:
    // the map was mis-wired by the 8 duplicate-ID pairs, giving Forked
    // Lightning 1 instead of 2 and phantom Overload to Windfury / Windspeaker /
    // Ancestral Spirit — those cards are now on unique CS2_*/CS1_* IDs).
    let overload_amount = match card_def.id {
        "SHAMAN_002" => Some(1), // Lightning Bolt
        "SHAMAN_006" => Some(2), // Lightning Storm
        "SHAMAN_009" => Some(2), // Feral Spirit
        "SHAMAN_011" => Some(2), // Doomhammer
        "SHAMAN_015" => Some(2), // Dust Devil
        "SHAMAN_016" => Some(2), // Forked Lightning
        "SHAMAN_017" => Some(2), // Lava Burst
        "SHAMAN_018" => Some(1), // Stormforged Axe
        "SHAMAN_019" => Some(3), // Earth Elemental
        _ => None,
    };
    if let Some(amount) = overload_amount {
        world.set_overload(entity, Overload(amount));
    }
    if card_def.id == "SHAMAN_021" {
        // Unbound Elemental — gain +1/+1 whenever you play a card with Overload
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyOverloadPlayed,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
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
                race: None,
                max_attack: None,
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
                race: None,
                max_attack: None,
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
                race: None,
                max_attack: None,
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
    if card_def.id == "WARRIOR_008" {
        // Warsong Commander — whenever you summon a minion with 3 or less
        // Attack, give it Charge. A trigger rather than an aura: the Charge is
        // granted once, to that minion, and outlives the commander.
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: Some(3),
                effect: CardEffect::GrantCharge {
                    target: EffectTarget::EventSubject,
                    attack_bonus: 0,
                },
            },
        );
    }
    // Enrage — a conditional state, not a buff. The bonus applies while the
    // minion is damaged, does not stack across damage instances, and is gone
    // the moment the minion is healed to full; `World::effective_attack` and
    // `World::max_attacks` resolve it on read.
    let enrage: Option<Enrage> = match card_def.id {
        // Tauren Warrior — Taunt. Enrage: +3 Attack
        "NEUTRAL_C11" => Some(Enrage {
            attack: 3,
            ..Enrage::default()
        }),
        // Amani Berserker — Enrage: +3 Attack (2/3 → 5/3 while damaged)
        "CLASSIC_018" => Some(Enrage {
            attack: 3,
            ..Enrage::default()
        }),
        // Raging Worgen — Enrage: Windfury and +1 Attack. Both halves are part
        // of the Enrage, so the Windfury goes away with the damage too.
        "NEUTRAL_008" => Some(Enrage {
            attack: 1,
            windfury: true,
            ..Enrage::default()
        }),
        // Grommash Hellscream — Charge. Enrage: +6 Attack (4/9 → 10/9)
        "WARRIOR_010" => Some(Enrage {
            attack: 6,
            ..Enrage::default()
        }),
        // Angry Chicken — Enrage: +5 Attack (1/1 → 6/1)
        "NEUTRAL_R02" => Some(Enrage {
            attack: 5,
            ..Enrage::default()
        }),
        // Spiteful Smith — Enrage: your weapon has +2 Attack
        "NEUTRAL_C15" => Some(Enrage {
            weapon_attack: 2,
            ..Enrage::default()
        }),
        _ => None,
    };
    if let Some(enrage) = enrage {
        world.set_enrage(entity, enrage);
    }
    // Gurubashi Berserker is deliberately NOT an Enrage minion: its real text
    // is "Whenever this minion takes damage, gain +3 Attack", a permanent buff
    // that stacks per damage instance and survives a full heal.
    if card_def.id == "NEUTRAL_B19" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::ThisMinionDamaged,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GainStats {
                    attack: 3,
                    health: 0,
                    target: EffectTarget::Self_,
                },
            },
        );
    }
    if card_def.id == "NEUTRAL_R16" {
        // Emperor Cobra — Poison
        world.set_poison(entity, Poison);
    }
    if matches!(card_def.id, "LEGENDARY_024" | "CORE_EX1_100") {
        // Lorewalker Cho — whenever a player casts a spell, put a copy into
        // the other player's hand. Pool-open (reads the cast spell — it can
        // move a card across a pool boundary); registered here by ID because
        // CardDef has no generic trigger field. The registry check below
        // keeps the closure invariant testable: this ID list is the second
        // source of truth the sets.rs closure test folds in.
        debug_assert!(
            crate::cards::sets::POOL_OPEN_CARDS.contains(&card_def.id),
            "pool-open trigger ({}) requires a POOL_OPEN_CARDS row",
            card_def.id
        );
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::AnySpellCast,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::CopyCastSpellToOtherPlayerHand,
            },
        );
    }
    // Race-conditioned triggers (fidelity-debt W1): the Trigger carries the
    // race requirement — the event's subject must match for the trigger to fire.
    let race_trigger: Option<(TriggerEvent, CardEffect)> = match card_def.id {
        // Murloc Tidecaller — whenever you summon a Murloc, gain +1 Attack
        "NEUTRAL_R05" => Some((
            TriggerEvent::FriendlyMinionSummoned,
            CardEffect::GainStats {
                attack: 1,
                health: 0,
                target: EffectTarget::Self_,
            },
        )),
        // Starving Buzzard — whenever you summon a Beast, draw a card
        "HUNTER_014" => Some((
            TriggerEvent::FriendlyMinionSummoned,
            CardEffect::DrawCard { count: 1 },
        )),
        // Scavenging Hyena — whenever a friendly Beast dies, gain +2/+1
        "HUNTER_013" => Some((
            TriggerEvent::FriendlyMinionDied,
            CardEffect::GainStats {
                attack: 2,
                health: 1,
                target: EffectTarget::Self_,
            },
        )),
        _ => None,
    };
    if let Some((event, effect)) = race_trigger {
        use crate::core::component::Race;
        let race = match card_def.id {
            "NEUTRAL_R05" => Race::Murloc,
            "HUNTER_014" | "HUNTER_013" => Race::Beast,
            _ => unreachable!(),
        };
        world.set_trigger(
            entity,
            Trigger {
                event,
                timing: TriggerTiming::Whenever,
                effect,
                race: Some(race),
                max_attack: None,
            },
        );
    }
    // W2 trigger classes (fidelity-debt): heal / card-played / secret-played /
    // any-minion-died — registered per card ID via the unified Trigger component.
    let w2_trigger: Option<(TriggerEvent, CardEffect)> = match card_def.id {
        // Lightwarden — whenever a character is healed, gain +2 Attack
        "NEUTRAL_R04" => Some((
            TriggerEvent::CharacterHealed,
            CardEffect::GainStats {
                attack: 2,
                health: 0,
                target: EffectTarget::Self_,
            },
        )),
        // Northshire Cleric — whenever a MINION is healed, draw a card.
        // Either player's minion counts; healing a hero does not.
        "PRIEST_004" => Some((
            TriggerEvent::MinionHealed,
            CardEffect::DrawCard { count: 1 },
        )),
        // Questing Adventurer — whenever you play a card, gain +1/+1
        "NEUTRAL_R17" => Some((
            TriggerEvent::CardPlayed,
            CardEffect::GainStats {
                attack: 1,
                health: 1,
                target: EffectTarget::Self_,
            },
        )),
        // Secretkeeper — whenever a Secret is played, gain +1/+1
        "NEUTRAL_R06" => Some((
            TriggerEvent::SecretPlayed,
            CardEffect::GainStats {
                attack: 1,
                health: 1,
                target: EffectTarget::Self_,
            },
        )),
        // Flesheating Ghoul — whenever a minion dies, gain +1 Attack
        "NEUTRAL_C12" => Some((
            TriggerEvent::MinionDied,
            CardEffect::GainStats {
                attack: 1,
                health: 0,
                target: EffectTarget::Self_,
            },
        )),
        _ => None,
    };
    if let Some((event, effect)) = w2_trigger {
        world.set_trigger(
            entity,
            Trigger {
                event,
                timing: TriggerTiming::Whenever,
                effect,
                race: None,
                max_attack: None,
            },
        );
    }
    // W9 weapon-attack triggers (fidelity-debt): the trigger rides the weapon
    // entity — trigger_applies pins `Attacked`/`AttackedMinion` to the
    // attacker OR the attacking hero's equipped weapon.
    let weapon_trigger: Option<(TriggerEvent, CardEffect)> = match card_def.id {
        // Truesilver Champion — whenever your hero attacks, restore 2 Health
        "PALADIN_006" => Some((
            TriggerEvent::Attacked,
            CardEffect::RestoreHealth {
                amount: 2,
                target: EffectTarget::FriendlyHero,
            },
        )),
        // Gorehowl — attacking a minion costs 1 Attack instead of 1 Durability
        "WARRIOR_009" => Some((
            TriggerEvent::AttackedMinion,
            CardEffect::BuffWeapon {
                attack: -1,
                durability: 0,
            },
        )),
        // Eaglehorn Bow — +1 Durability whenever a friendly Secret is revealed
        "HUNTER_007" => Some((
            TriggerEvent::FriendlySecretRevealed,
            CardEffect::BuffWeapon {
                attack: 0,
                durability: 1,
            },
        )),
        _ => None,
    };
    if let Some((event, effect)) = weapon_trigger {
        world.set_trigger(
            entity,
            Trigger {
                event,
                timing: TriggerTiming::Whenever,
                effect,
                race: None,
                max_attack: None,
            },
        );
    }
    // Mana Addict (W6) — whenever you cast a spell, gain +2 Attack THIS TURN
    // (the temporary buff expires at the end of the turn)
    if card_def.id == "NEUTRAL_R10" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlySpellCast,
                timing: TriggerTiming::Whenever,
                effect: CardEffect::GainStatsThisTurn {
                    attack: 2,
                    health: 0,
                    target: EffectTarget::Self_,
                },
                race: None,
                max_attack: None,
            },
        );
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
    world.remove_enrage(entity);
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
    // Set Stealth (roadmap M5: card-level stealth — the mechanic existed
    // (F2) but CardDef had no field, so stealth cards were defined vanilla)
    if card.stealth {
        world.set_stealth(e, crate::core::component::Stealth);
    }
    // Set Elusive (roadmap M5: cannot be targeted by spells/hero powers)
    if card.elusive {
        world.set_elusive(e, crate::core::component::Elusive);
    }
    // Set Taunt
    if card.taunt {
        world.set_taunt(e, crate::core::component::Taunt);
    }
    // Set Race / tribe (fidelity-debt W1)
    if let Some(race) = card.race {
        world.set_race(e, race);
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
                race: None,
                max_attack: None,
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
                race: None,
                max_attack: None,
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
                race: None,
                max_attack: None,
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
                race: None,
                max_attack: None,
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
                race: None,
                max_attack: None,
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
    use crate::cards::generated;
    use crate::cards::sets::ALL_CARDS;

    /// Cards whose official (post-rebalance) static attributes intentionally
    /// differ from the handwritten 2014-classic pool — the rebalance ledger
    /// (roadmap E4). Each entry lists the fields that differ.
    fn known_rebalanced(name: &str, field: &str) -> bool {
        let fields: &[&str] = match name {
            "Abusive Sergeant" => &["attack"],
            "Acolyte of Pain" => &["health"],
            "Al'Akir the Windlord" => &["health"],
            "Ancient of Lore" => &["attack", "health"],
            "Arcane Golem" => &["health"],
            "Argent Protector" => &["attack"],
            "Assassin's Blade" => &["attack", "cost", "durability"],
            "Assassinate" => &["cost"],
            "Azure Drake" => &["health"],
            "Baron Geddon" => &["health"],
            "Barrens Stablehand" => &["attack", "cost", "health"],
            "Blizzard" => &["cost"],
            "Brightwing" => &["cost"],
            "Cairne Bloodhoof" => &["attack", "taunt"],
            "Cenarius" => &["cost"],
            "Cone of Cold" => &["cost"],
            "Consecration" => &["cost"],
            "Cruel Taskmaster" => &["health"],
            "Defender of Argus" => &["attack", "taunt"],
            "Defias Ringleader" => &["attack"],
            "Druid of the Claw" => &["cost", "health"],
            "Earth Elemental" => &["health"],
            "Fan of Knives" => &["cost"],
            "Force of Nature" => &["cost"],
            "Gadgetzan Auctioneer" => &["cost"],
            "Gnoll" => &["cost"],
            "Guardian of Kings" => &["health", "taunt"],
            "Hammer of Wrath" => &["cost"],
            "Hellfire" => &["cost"],
            "Holy Nova" => &["cost"],
            "Holy Wrath" => &["cost"],
            "Ironbeak Owl" => &["cost"],
            "King Mukla" => &["health"],
            "Lay on Hands" => &["cost"],
            "Lightspawn" => &["cost", "health"],
            "Lord Jaraxxus" => &["attack", "card_type", "cost", "health"],
            "Malygos" => &["spell_damage"],
            "Mind Control" => &["cost"],
            "Mirror Image" => &["card_type", "cost", "health", "taunt"],
            "Misdirection" => &["cost"],
            "Misha" => &["attack"],
            "Natalie Seline" => &["attack", "cost", "health"],
            "Patient Assassin" => &["health"],
            "Prophet Velen" => &["spell_damage"],
            "Savannah Highmane" => &["attack"],
            "Shadow Madness" => &["cost"],
            "Shadow Word: Death" => &["cost"],
            "Shield Block" => &["cost"],
            "Siphon Soul" => &["cost"],
            "Slam" => &["cost"],
            "Soul of the Forest" => &["cost"],
            "Southsea Deckhand" => &["charge"],
            "Sprint" => &["cost"],
            "Stormwind Champion" => &["attack", "health"],
            "Swipe" => &["cost"],
            "Temple Enforcer" => &["attack", "cost"],
            "The Black Knight" => &["cost", "health"],
            "Timber Wolf" => &["health"],
            "Tirion Fordring" => &["attack", "health"],
            "Thoughtsteal" => &["cost"],
            "Treant" => &["charge"],
            "Unbound Elemental" => &["attack"],
            "Void Terror" => &["health"],
            "Xavius" => &["attack", "cost", "health"],
            "Baine Bloodhoof" => &["attack", "cost"],
            "Panther" => &["cost"],
            "Big Game Hunter" => &["cost"],
            "Spellbender" => &["attack", "card_type", "cost", "health"],
            "Emerald Drake" => &["attack", "health"],
            "Laughing Sister" => &["cost"],
            "Huffer" => &["charge", "health"],
            "Leokk" => &["attack", "health"],
            _ => &[],
        };
        fields.contains(&field)
    }

    /// The official database (roadmap E4) covers the classic-era constructed
    /// sets. The handwritten pool uses custom IDs (CLASSIC_001, NEUTRAL_...)
    /// while the official data uses Blizzard IDs (CS2_172, EX1_...), so the
    /// verification matches by NAME: every handwritten card with an official
    /// counterpart must agree on the statically representable fields, except
    /// for the documented rebalances (known_rebalanced).
    ///
    /// Core Set W0 (decision D2): a `CORE_<P>_<n>` card's counterpart is its
    /// ID without the `CORE_` prefix (`CORE_EX1_100` ↔ generated `EX1_100`),
    /// matched by ID first — name matching would collide with the Classic
    /// reprint of the same card. Classic cards keep matching by name.
    #[test]
    fn generated_cards_match_handwritten() {
        assert!(
            !generated::GENERATED_IDS.is_empty(),
            "generated registry must be non-empty"
        );
        let mut compared = 0;
        let mut rebalanced = 0;
        for card in ALL_CARDS {
            let generated = card
                .id
                .strip_prefix("CORE_")
                .and_then(generated::find_by_id)
                .or_else(|| generated::find_by_name(card.name));
            let Some(generated) = generated else {
                continue; // no official counterpart (custom tokens)
            };
            compared += 1;
            for (field, generated_value, handwritten_value) in [
                ("card_type", generated.card_type == card.card_type, true),
                ("cost", generated.cost == card.cost, true),
                ("attack", generated.attack == card.attack, true),
                ("health", generated.health == card.health, true),
                ("durability", generated.durability == card.durability, true),
                ("taunt", generated.taunt == card.taunt, true),
                (
                    "divine_shield",
                    generated.divine_shield == card.divine_shield,
                    true,
                ),
                ("windfury", generated.windfury == card.windfury, true),
                ("charge", generated.charge == card.charge, true),
                (
                    "spell_damage",
                    generated.spell_damage == card.spell_damage,
                    true,
                ),
            ] {
                let _ = handwritten_value;
                if !generated_value {
                    assert!(
                        known_rebalanced(card.name, field),
                        "{field} mismatch: {} (rebalance not documented)",
                        card.name
                    );
                    rebalanced += 1;
                }
            }
        }
        assert!(
            compared > 100,
            "name-based verification should cover a meaningful share of the handwritten pool (got {compared})"
        );
        // Sanity: the ledger documents a meaningful number of rebalances but
        // the vast majority of the pool matches exactly.
        assert!(
            compared - rebalanced > compared / 2,
            "too many rebalances documented"
        );
    }

    /// Core Set reprint fidelity (decision D2, 2026-08-08): every generated
    /// `CORE_<P>_<n>` card whose classic-era original `<P>_<n>` exists in the
    /// generated database must agree with it on the statically representable
    /// fields — the Core version is a reprint, not a rebalance (verified
    /// 2026-08-08: all 90 reprint pairs in `cards.json` match exactly).
    #[test]
    fn core_reprints_match_originals() {
        let mut pairs = 0;
        for id in generated::GENERATED_IDS {
            let Some(original_id) = id.strip_prefix("CORE_") else {
                continue; // not a Core Set card
            };
            let Some(core_card) = generated::find_by_id(id) else {
                panic!("generated registry lists {id} but lookup fails");
            };
            let Some(original) = generated::find_by_id(original_id) else {
                continue; // no classic-era original (e.g. CORE_AV_* from newer sets)
            };
            pairs += 1;
            assert_eq!(
                core_card.card_type, original.card_type,
                "card_type mismatch: {id} vs {original_id}"
            );
            assert_eq!(
                core_card.cost, original.cost,
                "cost mismatch: {id} vs {original_id}"
            );
            assert_eq!(
                core_card.attack, original.attack,
                "attack mismatch: {id} vs {original_id}"
            );
            assert_eq!(
                core_card.health, original.health,
                "health mismatch: {id} vs {original_id}"
            );
            assert_eq!(
                core_card.durability, original.durability,
                "durability mismatch: {id} vs {original_id}"
            );
            assert_eq!(
                core_card.taunt, original.taunt,
                "taunt mismatch: {id} vs {original_id}"
            );
            assert_eq!(
                core_card.divine_shield, original.divine_shield,
                "divine_shield mismatch: {id} vs {original_id}"
            );
            assert_eq!(
                core_card.windfury, original.windfury,
                "windfury mismatch: {id} vs {original_id}"
            );
            assert_eq!(
                core_card.charge, original.charge,
                "charge mismatch: {id} vs {original_id}"
            );
            assert_eq!(
                core_card.spell_damage, original.spell_damage,
                "spell_damage mismatch: {id} vs {original_id}"
            );
        }
        assert!(
            pairs > 50,
            "reprint-pair coverage should cover the Classic reprints (got {pairs})"
        );
    }

    /// The generated database must be internally consistent: every ID in the
    /// registry resolves to a const.
    #[test]
    fn generated_lookup_round_trips() {
        for id in generated::GENERATED_IDS {
            let card = generated::find_by_id(id).expect("generated const for id");
            assert_eq!(card.id, *id);
        }
    }
}
