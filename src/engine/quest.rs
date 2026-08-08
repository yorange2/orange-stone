//! Quest progress dispatch (2025–2026 expansions M2-W1 — the Un'Goro quest
//! mechanic): `progress()` accumulates progress on quest cards sitting in
//! the player's `Zone::Quest` slot and `complete()` resolves their rewards.
//!
//! Call sites live in `engine/rules.rs` (play path, damage resolution,
//! turn-end, spell-cast) and `engine/trigger.rs` (corpse spend, discover,
//! shuffle) — each fires at the natural event point with the amount/marker
//! the condition needs.

use crate::cards::quest::{QuestCondition, QuestDef, quest_def};
use crate::core::entity::Entity;
use crate::core::event::EventQueue;
use crate::core::player::PlayerId;
use crate::core::state::GameState;
use crate::core::zone::Zone;

/// Accumulate quest progress for `condition` for `player`.
///
/// Iterates the player's `Zone::Quest` entities; for each quest whose
/// definition matches `condition`:
/// - a `Some(marker)` is only counted once (skipped when `markers` already
///   contains it, else appended) — the per-condition dedup for set-based
///   conditions (unique races, distinct turns, attack values);
/// - `progress` grows by `amount`; at `>= target` the quest completes.
///
/// Only the first matching quest in the slot receives progress per event
/// (one quest per player anyway). Completion moves the quest out of the
/// zone (or resets it for repeatable quests).
pub(crate) fn progress(
    state: &mut GameState,
    queue: &mut EventQueue,
    player: PlayerId,
    condition: QuestCondition,
    amount: u32,
    marker: Option<u32>,
) {
    // Collect the slot entities first: completing a quest mutates the zone
    // table (graveyard move), so iterating while mutating would be unsound.
    let entities: Vec<Entity> = state.world().zones().entities(Zone::Quest, player);
    for quest_entity in entities {
        let Some(qdef) = state
            .world()
            .card_id(quest_entity)
            .and_then(|card| quest_def(card.0))
        else {
            continue;
        };
        if qdef.condition != condition {
            continue;
        }
        let mut quest = state
            .world()
            .quest(quest_entity)
            .expect("quest entity in the quest zone carries the Quest component");
        if let Some(m) = marker {
            if quest.markers.contains(&m) {
                return;
            }
            quest.markers.push(m);
        }
        quest.progress += amount;
        if quest.progress >= quest.target {
            complete(state, queue, quest_entity, player, qdef);
        } else {
            state.world_mut().set_quest(quest_entity, quest);
        }
        return;
    }
}

/// Complete a quest: resolve its reward, then remove it from the quest slot
/// (or reset it for repeatable quests).
///
/// The reward resolves with the quest entity as the effect source and no
/// explicit target — the same battlecry-resolution call shape
/// (`trigger::resolve_effect`). Reward tokens are not registered until W2,
/// so `SummonMinion`/`EquipWeapon` rewards no-op gracefully.
fn complete(
    state: &mut GameState,
    queue: &mut EventQueue,
    quest_entity: Entity,
    owner: PlayerId,
    qdef: &QuestDef,
) {
    crate::engine::trigger::resolve_effect(
        state,
        queue,
        quest_entity,
        owner,
        qdef.reward,
        None,
        None,
    );
    if qdef.repeatable {
        // Repeatable Quest: progress resets, markers clear, the quest stays
        // in the slot to be completed again.
        let reset = crate::core::component::Quest {
            progress: 0,
            target: qdef.target,
            repeatable: true,
            markers: Vec::new(),
        };
        state.world_mut().set_quest(quest_entity, reset);
    } else {
        // Standard quest: completed — leave the slot for the graveyard.
        let _ = state
            .world_mut()
            .move_to_zone(quest_entity, Zone::Graveyard);
    }
}
