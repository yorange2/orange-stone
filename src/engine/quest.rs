//! Quest progress dispatch (2025–2026 expansions M2-W1/W2 — the Un'Goro
//! quest mechanic): `progress()` accumulates progress on quest cards
//! sitting in the player's `Zone::Quest` slot and `complete_bar()` resolves
//! their rewards.
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
/// - a `Some(marker)` is only counted once (skipped when the matching bar's
///   `markers` already contains it, else appended) — the per-condition
///   dedup for set-based conditions (unique races, distinct turns, attack
///   values);
/// - the bar's `progress` grows by `amount`; at `>= target` the bar
///   completes (`complete_bar`).
///
/// Dual-bar quests (TLC_817, M2-W2) match each condition against its own
/// bar: the first bar via `QuestDef::condition`, the second via
/// `QuestDef::second`. Each bar's reward resolves independently at its own
/// target; the card leaves the quest zone only when BOTH bars are done (a
/// completed bar ignores further matching events while the other bar
/// finishes).
///
/// Only the first matching quest in the slot receives progress per event
/// (one quest per player anyway).
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
        // Which bar does the event feed? A condition matching neither bar of
        // the quest is ignored.
        let is_second = qdef.second.is_some_and(|s| s.condition == condition);
        if qdef.condition != condition && !is_second {
            return;
        }
        // M3-W3 — Battle at the End Time (END_017): "Fill your hand, then
        // empty it" is a SEQUENCE, not an accumulating counter: the hand
        // must reach 10 once (marker 0), then be emptied by plays (marker
        // 1) — only the emptying completes the quest, and only when the
        // filled marker is already set. Both markers are deduped, so a
        // second fill/empty cycle cannot double-count (§22).
        if qdef.condition == QuestCondition::FillThenEmptyHand && !is_second {
            let mut quest = state
                .world()
                .quest(quest_entity)
                .expect("quest entity in the quest zone carries the Quest component");
            if quest.progress >= quest.target {
                return;
            }
            let Some(m) = marker else {
                return;
            };
            if m == 0 {
                if !quest.markers.contains(&0) {
                    quest.markers.push(0);
                }
            } else if quest.markers.contains(&0) && !quest.markers.contains(&1) {
                quest.markers.push(1);
                quest.progress += amount;
            }
            if quest.progress >= quest.target {
                complete_bar(state, queue, quest_entity, player, qdef, quest, false);
            } else {
                state.world_mut().set_quest(quest_entity, quest);
            }
            return;
        }
        let mut quest = state
            .world()
            .quest(quest_entity)
            .expect("quest entity in the quest zone carries the Quest component");
        if is_second {
            // Second bar (TLC_817's Shadow half): mirrors the first bar's
            // progress logic on the `Quest::second` state.
            let second = quest
                .second
                .as_mut()
                .expect("a second-bar condition implies a second-bar def");
            // A completed bar ignores further matching events (the card
            // stays in the slot while the other bar finishes).
            if second.progress >= second.target {
                return;
            }
            if let Some(m) = marker {
                if second.markers.contains(&m) {
                    return;
                }
                second.markers.push(m);
            }
            second.progress += amount;
            if second.progress >= second.target {
                complete_bar(state, queue, quest_entity, player, qdef, quest, true);
            } else {
                state.world_mut().set_quest(quest_entity, quest);
            }
        } else {
            // First bar — the W1 semantics unchanged.
            if quest.progress >= quest.target {
                return;
            }
            if let Some(m) = marker {
                if quest.markers.contains(&m) {
                    return;
                }
                quest.markers.push(m);
            }
            quest.progress += amount;
            if quest.progress >= quest.target {
                complete_bar(state, queue, quest_entity, player, qdef, quest, false);
            } else {
                state.world_mut().set_quest(quest_entity, quest);
            }
        }
        return;
    }
}

/// Complete one bar of a quest: resolve its reward, then remove the quest
/// from the slot (or reset / keep it per the bar's semantics).
///
/// - First bar of a repeatable quest: resets to 0 (markers clear) and
///   stays in the slot to be completed again.
/// - Single-bar quests (and the second bar of dual-bar quests, when the
///   first bar is done too): both bars complete — the card leaves the slot
///   for the graveyard.
/// - One bar of a dual-bar quest while the other is still running: the
///   reward resolves and the card stays in the slot with the completed
///   bar's progress capped at its target (matching events then skip it).
///
/// The reward resolves with the quest entity as the effect source and no
/// explicit target — the same battlecry-resolution call shape
/// (`trigger::resolve_effect`).
fn complete_bar(
    state: &mut GameState,
    queue: &mut EventQueue,
    quest_entity: Entity,
    owner: PlayerId,
    qdef: &QuestDef,
    quest: crate::core::component::Quest,
    is_second: bool,
) {
    let reward = if is_second {
        qdef.second
            .expect("a second bar completes only with a second-bar def")
            .reward
    } else {
        qdef.reward
    };
    crate::engine::trigger::resolve_effect(state, queue, quest_entity, owner, reward, None, None);
    if !is_second && qdef.repeatable {
        // Repeatable Quest: progress resets, markers clear, the quest stays
        // in the slot to be completed again.
        let mut reset = quest;
        reset.progress = 0;
        reset.markers.clear();
        state.world_mut().set_quest(quest_entity, reset);
        return;
    }
    // Dual-bar quests leave the slot only when BOTH bars are done.
    let both_done = match (is_second, qdef.second) {
        (false, Some(_)) => quest
            .second
            .as_ref()
            .is_some_and(|s| s.progress >= s.target),
        (true, Some(_)) => quest.progress >= quest.target,
        _ => true,
    };
    if both_done {
        // Standard (or fully completed) quest: leave the slot for the
        // graveyard.
        let _ = state
            .world_mut()
            .move_to_zone(quest_entity, Zone::Graveyard);
    } else {
        // The other bar is still running: cap the completed bar's progress
        // at its target and keep the card in the slot.
        let mut kept = quest;
        if is_second {
            let second = kept
                .second
                .as_mut()
                .expect("a second bar completes only with a second-bar def");
            second.progress = second.target;
        } else {
            kept.progress = kept.target;
        }
        state.world_mut().set_quest(quest_entity, kept);
    }
}
