//! Colossal definitions (2025–2026 expansions M4-W1 — the Cataclysm
//! Colossal primitive): the static half of the 11 Colossal cards, keyed
//! by card id.
//!
//! "Colossal +N" — when this minion is played, N attached body parts are
//! summoned alongside it. The parts are separate entities (tokens) that:
//! - occupy the board positions immediately to the right of the main, in
//!   order (the 7-minion cap applies: a full board summons nothing),
//! - die with the main minion (with their deathrattles),
//! - do NOT kill the main when a part dies (part death is independent),
//! - carry their own effects; the part→main link (the `ColossalPart`
//!   component) drives the cascade and the "refer to the main" effects
//!   (Wickerfang's stat copy, Chromatus's keyword removal).
//!
//! The registry is the id-keyed analogue of `CardDef` fields (the CardDef
//! gets NO new field): the play path (the MinionSummoned handler) looks up
//! `colossal_parts` and summons the appendages; the damage pipeline's
//! death check cascades a dying main's death to its parts. The `fill`
//! flag pins Magmaw's "Colossal +99 — summon any leftover appendages when
//! there is room": both shapes collapse to `min(parts.len(), free slots)`
//! at summon time — the maximum free space is 6 (the main occupies one
//! slot), which equals Magmaw's appendage count, so the +99 clamp is
//! exactly "fill all remaining slots".
//!
//! The appendage data (ids, stats, effects) was verified against
//! `/tmp/hs_full.json` 2026-08-09 (the fidelity rows are fidelity-debt.md
//! §23, en + zh).

use crate::core::component::{ColossalMain, ColossalPart};
use crate::core::effect::CardEffect;
use crate::core::entity::Entity;
use crate::core::event::EventQueue;
use crate::core::player::PlayerId;
use crate::core::state::GameState;
use crate::core::zone::Zone;

/// A Colossal registry entry — the appendage token ids of one card.
pub(crate) struct ColossalDef {
    /// Whether the +N fills ALL remaining board slots (Magmaw's +99 —
    /// both shapes clamp to the free space at summon, see the module
    /// docs; the flag documents the intent).
    pub fill: bool,
    /// The appendage token card ids, in left-to-right board order.
    pub parts: &'static [&'static str],
}

/// The Colossal registry — card id → appendage token list (M4-W1).
pub(crate) fn colossal_parts(card_id: &str) -> Option<&'static ColossalDef> {
    match card_id {
        // Wickerfang (DRUID 6/0/5, +4) — 4× Leg 1/0/2.
        "CATA_139" => Some(&ColossalDef {
            fill: false,
            parts: &["CATA_139t", "CATA_139t2", "CATA_139t3", "CATA_139t4"],
        }),
        // Ragnaros, the Great Fire (WARRIOR 8/8/8, +2) — 2× Hand 1/2/1.
        "CATA_150" => Some(&ColossalDef {
            fill: false,
            parts: &["CATA_150t", "CATA_150t1"],
        }),
        // Azshara, Ocean Lord (DEMONHUNTER 8/8/8, +2) — 2× Tentacle 1/2/1.
        "CATA_151" => Some(&ColossalDef {
            fill: false,
            parts: &["CATA_151t", "CATA_151t1"],
        }),
        // Al'Akir, Lord of Storms (SHAMAN 8/2/8, +2) — 2× Charged Hand 1/1/2.
        "CATA_153" => Some(&ColossalDef {
            fill: false,
            parts: &["CATA_153t", "CATA_153t1"],
        }),
        // Sinestra (ROGUE 6/5/5, +2) — 2× Wing 1/1/1.
        "CATA_154" => Some(&ColossalDef {
            fill: false,
            parts: &["CATA_154t", "CATA_154t1"],
        }),
        // Arisen Onyxia (DEATHKNIGHT 9/3/6, +2) — 2× Wing 1/1/1.
        "CATA_155" => Some(&ColossalDef {
            fill: false,
            parts: &["CATA_155t", "CATA_155t1"],
        }),
        // The Black Blood (PRIEST 7/5/9, +3) — 3× Body 2/2/2.
        "CATA_300" => Some(&ColossalDef {
            fill: false,
            parts: &["CATA_300t1", "CATA_300t2", "CATA_300t3"],
        }),
        // Chromatus (PALADIN 8/8/8, +4) — 4× Head 2/2/3 (Green/Red/
        // Blue/Bronze).
        "CATA_432" => Some(&ColossalDef {
            fill: false,
            parts: &["CATA_432t1", "CATA_432t2", "CATA_432t3", "CATA_432t4"],
        }),
        // Vulcanos (MAGE 7/4/8, +2) — 2× Plume 2/1/4.
        "CATA_488" => Some(&ColossalDef {
            fill: false,
            parts: &["CATA_488t", "CATA_488t2"],
        }),
        // Magmaw (HUNTER 7/2/12, +99) — 6× Body 1/2/1; the +99 clamps to
        // the free space (the `fill` flag documents the intent).
        "CATA_550" => Some(&ColossalDef {
            fill: true,
            parts: &[
                "CATA_550t",
                "CATA_550t2",
                "CATA_550t3",
                "CATA_550t4",
                "CATA_550t5",
                "CATA_550t6",
            ],
        }),
        // Cho'gall, Mastermind (WARLOCK 9/6/6, +2) — 2× Arm 1/1/1.
        "CATA_726" => Some(&ColossalDef {
            fill: false,
            parts: &["CATA_726t", "CATA_726t1"],
        }),
        _ => None,
    }
}

/// Per-token "when summoned" effects (M4-W1) — resolved by the summon
/// path right after the appendage enters the board. These ride a
/// per-token registry instead of a `summon_trigger` because the
/// FriendlyMinionSummoned trigger firing excludes the summoned minion
/// itself (a self-scoped on-summon effect cannot ride its own summon
/// event). The Herald {0} placeholders are fixed to sensible values
/// (W2's problem), registered in fidelity-debt §23.
pub(crate) fn appendage_on_summon(card_id: &str) -> Option<CardEffect> {
    match card_id {
        // Azshara's Tentacle — "give your hero +2 Attack this turn"
        // ({0} fixed to 2; `GainHeroAttack` is already until-end-of-turn).
        "CATA_151t" | "CATA_151t1" => Some(CardEffect::GainHeroAttack {
            attack: 2,
            armor: 0,
        }),
        // Sinestra's Wing — "get a random spell from another class. It
        // costs ({0}) less" — the reduction is dropped ({0} fixed to 0,
        // §23).
        "CATA_154t" | "CATA_154t1" => Some(CardEffect::AddRandomOtherClassSpells {
            count: 1,
            min_cost: 0,
        }),
        // Onyxia's Wing — "get a random {0}-Cost minion. It costs Health
        // this turn" — {0} fixed to 0; the CostHealth marker rides the
        // added card (the "this turn" scope is not cleared, §23).
        "CATA_155t" | "CATA_155t1" => Some(CardEffect::AddRandomCostMinionCostsHealth { cost: 0 }),
        _ => None,
    }
}

/// Summon a Colossal minion's appendages (M4-W1) — the summon path hook,
/// called after the main's battlecry resolves (the play-path MinionSummoned
/// handler, played-from-hand only).
///
/// Placement: the parts enter the board positions immediately right of the
/// main, left to right (`insert_at` on the running index — the main's own
/// index stays fixed, so part i lands at `main_idx + 1 + i`). The count is
/// `min(parts.len(), free slots)` — a full board summons nothing; Magmaw's
/// +99 clamps the same way (see the module docs). Each part carries the
/// `ColossalPart` link to the main, the main's `ColossalMain` lists them in
/// board order, a MinionSummoned event is enqueued per part (summoning
/// sickness, friendly-summon triggers, quest progress — all funnel there),
/// and the part's own on-summon effect resolves immediately.
pub(crate) fn summon_colossal_parts(
    state: &mut GameState,
    queue: &mut EventQueue,
    main: Entity,
    owner: PlayerId,
) {
    let Some(card_id) = state.world().card_id(main) else {
        return;
    };
    let Some(def) = colossal_parts(card_id.0) else {
        return;
    };
    let board_count = state
        .world()
        .zones()
        .iter(Zone::Play, owner)
        .filter(|&e| state.world().card_type(e) == Some(crate::core::component::CardType::Minion))
        .count();
    let free = crate::engine::rules::MAX_BOARD_SIZE - board_count;
    if free == 0 {
        // Full board — nothing summons (the main occupies the last slot).
        return;
    }
    if def.fill {
        // Magmaw's +99: the parts outnumber any possible free space, so
        // the clamp below ALWAYS fills every remaining slot (the flag
        // documents the intent — see the module docs).
        debug_assert!(def.parts.len() >= free);
    }
    let count = def.parts.len().min(free);
    // The main's index within the FULL play-zone vec (the hero sits at
    // position 0 — a minion-filtered index would insert the parts one slot
    // too far left, into the hero's gap, pushing the main right).
    let play_vec: Vec<Entity> = state.world().zones().iter(Zone::Play, owner).collect();
    let Some(main_idx) = play_vec.iter().position(|&e| e == main) else {
        return;
    };
    let mut parts = Vec::with_capacity(count);
    for (i, token_id) in def.parts.iter().take(count).enumerate() {
        let Some(card_def) = crate::cards::def::card_by_id(token_id) else {
            continue;
        };
        let part = {
            let world = state.world_mut();
            let e = crate::cards::spawn_card_from_def(world, owner, card_def);
            world.set_zone(e, Zone::Play);
            world.set_colossal_part(e, ColossalPart { main });
            e
        };
        state
            .world_mut()
            .zones_mut()
            .insert_at(Zone::Play, owner, part, main_idx + 1 + i);
        parts.push(part);
        // The part's own MinionSummoned event (sickness, triggers, quests)
        // and its "when summoned" effect.
        queue.push(crate::core::event::Event::MinionSummoned {
            player: owner,
            minion: part,
            target: None,
        });
        if let Some(effect) = appendage_on_summon(token_id) {
            crate::engine::trigger::resolve_effect(state, queue, part, owner, effect, None, None);
        }
    }
    if !parts.is_empty() {
        state
            .world_mut()
            .set_colossal_main(main, ColossalMain { parts });
    }
}

/// Cascade a dying Colossal main's death to its parts (M4-W1) — the death
/// check hook, called when the main is marked pending death (health ≤ 0).
///
/// Every attached part that is still alive, still on the board, and not
/// already pending death joins the pending-death batch, so the parts die
/// in the same death phase with their deathrattles, in play order (the
/// main is left of its parts, so its own death processes first).
///
/// The parts' deaths are COMMITTED at cascade time: each doomed part's
/// base health drops to 0, so the MinionDied handler's health re-check
/// (a healing-rescue check) passes — in Hearthstone a Colossal main's
/// death kills its parts unconditionally, mid-batch healing cannot save
/// them. No damage events are pushed, so no on-damage triggers fire.
///
/// The guards make every re-entry safe: an AOE that killed main + parts
/// marks the parts first (they are already pending when the main's entry
/// cascades — the zeroing is a no-op on a doomed part), and a part
/// bounced off the board is out of `Zone::Play` and keeps its link (it is
/// no longer a board part).
pub(crate) fn cascade_part_deaths(state: &mut GameState, main: Entity) {
    let Some(parts) = state.world().colossal_main(main).map(|cm| cm.parts.clone()) else {
        return;
    };
    if parts.is_empty() {
        return;
    }
    let world = state.world();
    let pending = state.pending_deaths();
    let doomed: Vec<Entity> = parts
        .into_iter()
        .filter(|&part| {
            world.is_alive(part) && world.zone(part) == Some(Zone::Play) && !pending.contains(&part)
        })
        .collect();
    if !doomed.is_empty() {
        let inner = state.make_mut();
        for &part in &doomed {
            inner
                .world
                .set_health(part, crate::core::component::Health(0));
        }
        inner.pending_deaths.extend(doomed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every registry entry's tokens resolve to real CardDefs (the
    /// exp_cata_w1.rs consts and the sets.rs registration stay in sync).
    #[test]
    fn colossal_registry_tokens_resolve() {
        for (card_id, def) in [
            "CATA_139", "CATA_150", "CATA_151", "CATA_153", "CATA_154", "CATA_155", "CATA_300",
            "CATA_432", "CATA_488", "CATA_550", "CATA_726",
        ]
        .into_iter()
        .filter_map(|id| colossal_parts(id).map(|d| (id, d)))
        {
            assert!(!def.parts.is_empty(), "{card_id} has parts");
            for token in def.parts {
                assert!(
                    crate::cards::def::card_by_id(token).is_some(),
                    "{card_id} token {token} resolves"
                );
            }
        }
    }

    /// The registry covers exactly the 11 W1 Colossal cards.
    #[test]
    fn colossal_registry_is_complete() {
        let expected: &[&str] = &[
            "CATA_139", "CATA_150", "CATA_151", "CATA_153", "CATA_154", "CATA_155", "CATA_300",
            "CATA_432", "CATA_488", "CATA_550", "CATA_726",
        ];
        for id in expected {
            assert!(colossal_parts(id).is_some(), "{id} registered");
        }
    }
}
