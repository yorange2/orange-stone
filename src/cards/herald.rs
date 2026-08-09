//! Herald registry and resolution (2025–2026 expansions M4-W2 — the
//! Cataclysm Herald wave, `exp_cata_w2.rs`): the id-keyed registry for
//! the 13 Herald cards plus the `resolve_herald` hook the play paths call.
//!
//! # The pinned mechanic (verified from the official card texts, 2026-08-09)
//!
//! **"Herald {0}"** — resolving a Herald keyword increments the player's
//! herald counter and summons the class patron's Colossal Soldier. The
//! {0} on the card text is the counter value; **"Herald twice to
//! upgrade"** is tiered: the Soldier numbers are the family base times a
//! tier that follows the counter — ×1 at counter 1, ×2 at counters 2–3,
//! ×4 at counters 4+ (the official example: the Ragnaros Soldier's
//! deathrattle damage 2 → 4 → 8). The counter increments BEFORE the
//! summon (the first Herald of the game reads "Herald 1"), and each
//! Herald resolution summons a NEW Soldier — the upgrade applies to the
//! freshly summoned Soldier's numbers AND to every friendly on-board
//! Soldier's live numbers.
//!
//! The 15 Herald cards minus W4's two (CATA_190h Deathwing, CATA_497
//! Ultraxion) = 13 here. CATA_722 Envoy of the End is the NEUTRAL patron:
//! the full dump has no neutral Soldier token, so its Herald increments
//! the counter and summons nothing (fidelity-debt §24).
//!
//! # Resolution shape
//!
//! A Herald card's CardDef carries NO battlecry/deathrattle for the
//! Herald itself (there is no `CardEffect::Herald` variant — Design B):
//! the keyword is resolved by the play paths calling `resolve_herald`
//! at the battlecry / spell / weapon / deathrattle / location-activation
//! resolution points, keyed by the registry. Because the minion Heralds
//! are "Battlecry: Herald {0}" in the official text, the hook fires in
//! the battlecry conditions (an effect-summoned copy Heralds too); Deios
//! and the BattlecryTwice dark gift do NOT double it (the CardDefs carry
//! no battlecry — the keyword resolves once, §24).
//!
//! The Soldiers' {0} numbers are BAKED into components at summon and
//! RE-BAKED on every later Herald (the numbers update live; the aura
//! system cannot read the per-player counter at query time — the bake
//! approach, §24). A component stripped by Silence is not re-granted
//! (`silence_entity` removes the component; the existence checks in the
//! re-bake loop skip entities that lost it).

use crate::core::component::{
    Aura, AuraEffect, AuraTarget, CardId, Deathrattle, Rush, Trigger, TriggerEvent, TriggerTiming,
};
use crate::core::effect::{CardEffect, EffectTarget};
use crate::core::entity::Entity;
use crate::core::event::EventQueue;
use crate::core::player::PlayerId;
use crate::core::small_list::SmallList;
use crate::core::state::GameState;
use crate::core::zone::Zone;
use crate::engine::trigger;

/// The 13 Herald cards (the 15 official minus W4's CATA_190h Deathwing
/// and CATA_497 Ultraxion — fidelity-debt §24).
pub const HERALD_CARD_IDS: &[&str] = &[
    "CATA_156", // Experimental Animation (DK spell)
    "CATA_158", // Maniacal Follower (Rogue minion — deathrattle Herald)
    "CATA_160", // Scorching Ravager (Warrior minion)
    "CATA_492", // Shrine of Twilight (Warlock location)
    "CATA_525", // Armored Bloodletter (DH minion)
    "CATA_530", // Fel Infusion (DH spell)
    "CATA_561", // Ritual of Power (Shaman spell)
    "CATA_565", // Skywall Sentinel (Shaman minion)
    "CATA_580", // Cataclysmic War Axe (Warrior weapon)
    "CATA_722", // Envoy of the End (neutral minion)
    "CATA_725", // Shadowsworn Disciple (Warlock minion)
    "CATA_780", // Obsessive Technician (DK minion)
    "CATA_785", // Rite of Twilight (Rogue spell)
];

/// The class patron whose Colossal Soldier a Herald card summons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeraldPatron {
    /// Death Knight — Soldier of Onyxia (CATA_780t)
    DeathKnight,
    /// Demon Hunter — Soldier of Azshara (CATA_525t)
    DemonHunter,
    /// Rogue — Soldier of Sinestra (CATA_158t)
    Rogue,
    /// Shaman — Soldier of Al'Akir (CATA_565t)
    Shaman,
    /// Warlock — Soldier of Cho'gall (CATA_725t)
    Warlock,
    /// Warrior — Soldier of Ragnaros (CATA_580t)
    Warrior,
    /// Neutral — CATA_722 Envoy of the End (no Soldier token exists)
    Neutral,
}

/// Maps the 13 Herald cards to their class patron (CATA_722 → `Neutral`).
#[must_use]
pub fn herald_patron(card_id: &str) -> Option<HeraldPatron> {
    match card_id {
        "CATA_156" | "CATA_780" => Some(HeraldPatron::DeathKnight),
        "CATA_525" | "CATA_530" => Some(HeraldPatron::DemonHunter),
        "CATA_158" | "CATA_785" => Some(HeraldPatron::Rogue),
        "CATA_561" | "CATA_565" => Some(HeraldPatron::Shaman),
        "CATA_492" | "CATA_725" => Some(HeraldPatron::Warlock),
        "CATA_160" | "CATA_580" => Some(HeraldPatron::Warrior),
        "CATA_722" => Some(HeraldPatron::Neutral),
        _ => None,
    }
}

/// Maps a patron to its Soldier token id (`None` for `Neutral` — the dump
/// has no neutral Soldier token).
#[must_use]
pub fn herald_soldier(patron: HeraldPatron) -> Option<&'static str> {
    match patron {
        HeraldPatron::DeathKnight => Some("CATA_780t"),
        HeraldPatron::DemonHunter => Some("CATA_525t"),
        HeraldPatron::Rogue => Some("CATA_158t"),
        HeraldPatron::Shaman => Some("CATA_565t"),
        HeraldPatron::Warlock => Some("CATA_725t"),
        HeraldPatron::Warrior => Some("CATA_580t"),
        HeraldPatron::Neutral => None,
    }
}

/// The official "Herald twice to upgrade" tiers (2026-08-09, pinned from
/// the card texts + the official Ragnaros example: the Soldier's damage
/// 2 → 4 → 8): ×1 at herald counter 1, ×2 at counters 2–3, ×4 at
/// counters 4+ (counter 0 is unreachable — the counter increments before
/// the first summon — and maps to ×1 defensively).
#[must_use]
pub const fn herald_number(base: i32, count: u32) -> i32 {
    match count {
        0 | 1 => base,
        2..=3 => base * 2,
        _ => base * 4,
    }
}

/// The one-shot "when summoned" effect of the {0}-carrying Soldiers,
/// resolved at summon with the counter read at resolution time (the
/// official numbers are live, §24). CATA_158t Soldier of Sinestra's
/// "costs ({0}) less" reduction is DROPPED — the Sinestra's Wing §23
/// convention (the engine's other-class-spell effect has no discount
/// parameter).
#[must_use]
fn soldier_on_summon(soldier_id: &str, count: u32) -> Option<CardEffect> {
    match soldier_id {
        // Soldier of Azshara — "When summoned, give your hero +{0}
        // Attack this turn" (base 2, the Azshara's-Tentacle §23 value).
        "CATA_525t" => Some(CardEffect::GainHeroAttack {
            attack: herald_number(2, count),
            armor: 0,
        }),
        // Soldier of Onyxia — "get a random {0}-Cost minion. It costs
        // Health this turn" (base 1-Cost).
        "CATA_780t" => Some(CardEffect::AddRandomCostMinionCostsHealth {
            cost: herald_number(1, count),
        }),
        // Soldier of Sinestra — "get a random spell from another class"
        // (the {0}-less reduction is dropped).
        "CATA_158t" => Some(CardEffect::AddRandomOtherClassSpells {
            count: 1,
            min_cost: 0,
        }),
        _ => None,
    }
}

/// Resolves a Herald keyword for `player`, keyed by the `source` entity's
/// card id (the source may be a played minion/weapon, a spell, a location,
/// or a DEAD minion in the graveyard — CATA_158's deathrattle; the card id
/// stays readable). Steps, in order:
///
/// 1. The herald counter increments (per-player, never resets).
/// 2. The patron's Soldier is summoned (baked values read the NEW counter;
///    `Neutral` summons nothing; a full board fails the summon).
/// 3. CATA_160 Scorching Ravager's add-on gives the just-summoned Soldier
///    Rush ("Battlecry: Herald {0}. Give the Soldier Rush").
/// 4. The Soldier's own "when summoned" effect resolves (the {0} read at
///    resolution time).
/// 5. Every friendly on-board Soldier's {0}-carrying components (the
///    Al'Akir aura, the Ragnaros deathrattle, the Cho'gall end-of-turn
///    trigger) re-bake with the NEW counter — the numbers update live.
///
/// Returns the summoned Soldier entity (`None` for a neutral patron or a
/// full board) — callers that need the entity (the W4 Deathwing
/// interaction, tests) use it; the play-path call sites discard it.
pub fn resolve_herald(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    player: PlayerId,
) -> Option<Entity> {
    let CardId(card_id) = state.world().card_id(source)?;
    let patron = herald_patron(card_id)?;

    // 1. The counter increments BEFORE the summon (the card text shows the
    // post-increment value — the first Herald of the game reads "Herald 1").
    let count = {
        let inner = state.make_mut();
        inner.players[player.index()].herald_count += 1;
        inner.players[player.index()].herald_count
    };

    // 2. Summon the Soldier (a neutral patron summons nothing — CATA_722;
    // a full board fails the summon, the counter still ticked).
    let soldier_id = herald_soldier(patron)?;
    let soldier = trigger::resolve_summon(state, queue, source, player, soldier_id);
    if let Some(soldier) = soldier {
        // 3. CATA_160 Scorching Ravager's add-on — give the Soldier Rush.
        if card_id == "CATA_160" {
            state.world_mut().set_rush(soldier, Rush);
        }
        // 4. The Soldier's own "when summoned" effect ({0} at resolution).
        if let Some(effect) = soldier_on_summon(soldier_id, count) {
            trigger::resolve_effect(state, queue, soldier, player, effect, None, None);
        }
    }

    // 5. Re-bake the {0}-carrying components of every friendly on-board
    // Soldier with the new counter — the official live-updating numbers.
    // A component stripped by Silence is not re-granted (the existence
    // checks skip entities that lost it).
    let soldiers: SmallList<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, player)
        .filter(|&e| {
            state
                .world()
                .card_id(e)
                .is_some_and(|c| matches!(c.0, "CATA_565t" | "CATA_580t" | "CATA_725t"))
        })
        .collect();
    for e in soldiers {
        let Some(CardId(id)) = state.world().card_id(e) else {
            continue;
        };
        match id {
            // Soldier of Al'Akir — "Adjacent minions have +{0} Attack"
            // (base 1, the Charged-Hand-of-Al'Akir §23 value).
            "CATA_565t" if state.world().aura(e).is_some() => {
                state.world_mut().set_aura(
                    e,
                    Aura {
                        effect: AuraEffect::GainAttack(herald_number(1, count)),
                        target: AuraTarget::AdjacentMinions,
                    },
                );
            }
            // Soldier of Ragnaros — "Deathrattle: Deal {0} damage to a
            // random enemy" (base 2, the Hand-of-Ragnaros §23 value).
            "CATA_580t" if state.world().deathrattle(e).is_some() => {
                state.world_mut().set_deathrattle(
                    e,
                    Deathrattle(CardEffect::DealDamageRandomly {
                        amount: herald_number(2, count),
                        count: 1,
                        target: EffectTarget::AnyEnemy,
                    }),
                );
            }
            // Soldier of Cho'gall — "At the end of your turn, destroy the
            // minion to the right to gain +{0}/+{0}" (base 2, the
            // Cho's-Arm §23 value; the Cho'gall deck-destroy redirect is
            // baked into ColossalArmDestroyRight itself).
            "CATA_725t" if state.world().trigger(e).is_some() => {
                state.world_mut().set_trigger(
                    e,
                    Trigger {
                        event: TriggerEvent::TurnEnd,
                        timing: TriggerTiming::Whenever,
                        race: None,
                        max_attack: None,
                        effect: CardEffect::ColossalArmDestroyRight {
                            attack: herald_number(2, count),
                            health: herald_number(2, count),
                        },
                    },
                );
            }
            _ => {}
        }
    }
    soldier
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::def::card_by_id;

    /// Every card with "Herald" text has a `herald_patron` entry — the 13
    /// implementable Herald cards (the official 15 minus W4's Deathwing
    /// and Ultraxion, §24).
    #[test]
    fn herald_patron_table_is_complete() {
        for id in HERALD_CARD_IDS {
            assert!(
                herald_patron(id).is_some(),
                "missing herald_patron for {id}"
            );
        }
        assert_eq!(HERALD_CARD_IDS.len(), 13);
    }

    /// `herald_soldier` resolves to a registered CardDef for every patron
    /// except the neutral one (CATA_722 — no Soldier token in the dump).
    #[test]
    fn herald_soldier_resolves_to_registered_defs() {
        for patron in [
            HeraldPatron::DeathKnight,
            HeraldPatron::DemonHunter,
            HeraldPatron::Rogue,
            HeraldPatron::Shaman,
            HeraldPatron::Warlock,
            HeraldPatron::Warrior,
        ] {
            let id = herald_soldier(patron).expect("patron must summon a Soldier");
            card_by_id(id).unwrap_or_else(|| panic!("no CardDef for {id}"));
        }
        assert_eq!(herald_soldier(HeraldPatron::Neutral), None);
    }

    /// The pinned "Herald twice to upgrade" tiers: ×1 at counter 1, ×2 at
    /// counters 2–3, ×4 at counters 4+ (counter 0 maps to ×1 defensively).
    #[test]
    fn herald_number_follows_the_pinned_tiers() {
        assert_eq!(herald_number(2, 0), 2);
        assert_eq!(herald_number(2, 1), 2);
        assert_eq!(herald_number(2, 2), 4);
        assert_eq!(herald_number(2, 3), 4);
        assert_eq!(herald_number(2, 4), 8);
        assert_eq!(herald_number(2, 5), 8);
    }
}
