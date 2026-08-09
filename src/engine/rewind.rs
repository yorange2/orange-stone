//! Rewind resolution (2025–2026 expansions M3-W1 — the Across the
//! Timeways rewind primitive). The play path calls [`hook_after_play`]
//! after a card's own effect (battlecry / spell effect) resolved: a
//! Rewind card first replays the effects of the cards played BEFORE it
//! (the pre-push history snapshot — itself never replays), then its own
//! entry is recorded. See `crate::cards::rewind` for the keyword
//! semantics and the count table.
use crate::cards::rewind::{MAX_REWIND_HISTORY, RewindEntry, rewind_count};
use crate::core::effect::CardEffect;
use crate::core::entity::Entity;
use crate::core::event::EventQueue;
use crate::core::player::PlayerId;
use crate::core::state::GameState;
use crate::engine::trigger;

/// Resolves a sequence of recorded entries with `source` as the effect
/// source and `owner` as the owner — a standard `resolve_effect` call per
/// entry with no explicit target and no event subject: random targeting
/// inside an effect resolves via the normal machinery, and deaths caused
/// by a replay process through the standard queue. Entries without an
/// effect (`None`) are skipped (they only occupy history slots).
pub(crate) fn resolve_replay(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    entries: &[RewindEntry],
) {
    // M3-W3 — Morchie (END_036): "Your Rewinds keep BOTH potential
    // outcomes." The registered simplification (§22) resolves each replayed
    // random-outcome effect twice — a fresh roll per resolution, which is
    // the engine's stand-in for "both outcomes". The aura is a unit-marker
    // (`AuraEffect::RewindKeepsBothOutcomes` on a friendly Morchie — the
    // Deios DoubleTriggers consultation shape); a silenced Morchie drops
    // the aura and the doubling.
    let morchie = state
        .world()
        .zones()
        .iter(crate::core::zone::Zone::Play, owner)
        .any(|e| {
            state.world().card_id(e).is_some_and(|c| c.0 == "END_036")
                && state.world().aura(e).is_some_and(|a| {
                    a.effect == crate::core::component::AuraEffect::RewindKeepsBothOutcomes
                })
        });
    for entry in entries {
        if let Some(effect) = entry.effect {
            trigger::resolve_effect(state, queue, source, owner, effect, None, None);
            if morchie {
                trigger::resolve_effect(state, queue, source, owner, effect, None, None);
            }
        }
    }
}

/// The rewind resolution itself — the play-path entry for a Rewind card:
/// snapshots the owner's history (the entries recorded before the current
/// play; the card's own entry is pushed only after this call), clamps
/// `count` to the history length, and replays the last `count` entries in
/// chronological (record) order.
pub(crate) fn resolve(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    owner: PlayerId,
    count: u32,
) {
    let history = state.player(owner).last_played.clone();
    let replayed = history.len().saturating_sub(count as usize);
    resolve_replay(state, queue, source, owner, &history[replayed..]);
}

/// The play-path hook — called after the card's own effect (battlecry /
/// spell effect) resolved. (1) A Rewind card resolves its replay(s)
/// against the PRE-push history; (2) the card's own entry — `effect` — is
/// recorded (capped at [`MAX_REWIND_HISTORY`], oldest dropped first).
/// Order matters: the push comes last, so the card never replays itself.
pub(crate) fn hook_after_play(
    state: &mut GameState,
    queue: &mut EventQueue,
    card: Entity,
    player: PlayerId,
    effect: Option<CardEffect>,
) {
    let count = state.world().card_id(card).map_or(0, |c| rewind_count(c.0));
    if count > 0 {
        resolve(state, queue, card, player, count);
    }
    let card_id = state
        .world()
        .card_id(card)
        .map_or_else(String::new, |c| c.0.to_string());
    let history = &mut state.make_mut().players[player.index()].last_played;
    history.push(RewindEntry { card_id, effect });
    if history.len() > MAX_REWIND_HISTORY {
        history.drain(0..history.len() - MAX_REWIND_HISTORY);
    }
}

/// The record-only half of [`hook_after_play`]: pushes the card's history
/// entry WITHOUT replaying anything. Used by the spell play paths whose
/// own effect never resolved — countered and spellbent spells (the
/// negation consumed the play), secret plays (the secret is armed, its
/// reveal effect never resolves at play), and location plays (the battlecry
/// is the ACTIVATE effect, resolved by LocationActivated — never by the
/// play). The play still occupies a history slot.
pub(crate) fn record_play(
    state: &mut GameState,
    card: Entity,
    player: PlayerId,
    effect: Option<CardEffect>,
) {
    let card_id = state
        .world()
        .card_id(card)
        .map_or_else(String::new, |c| c.0.to_string());
    let history = &mut state.make_mut().players[player.index()].last_played;
    history.push(RewindEntry { card_id, effect });
    if history.len() > MAX_REWIND_HISTORY {
        history.drain(0..history.len() - MAX_REWIND_HISTORY);
    }
}
