//! Rewind registry (2025–2026 expansions M3-W1 — the Across the Timeways
//! rewind primitive; no TIME cards land this wave, they are W2).
//!
//! **"Rewind: X" — when this card is played, the effect of the PREVIOUS
//! card you played happens again.** The card's own effect (the text after
//! "Rewind") resolves first, then the replay(s); each "Rewind" instance on
//! the card text replays one more previous card. The play path
//! (`engine/rules.rs`) resolves the replay via
//! [`crate::engine::rewind::hook_after_play`] right before recording the
//! card's own history entry, so a card never replays itself.
//!
//! # Counts
//!
//! The per-card counts come from the card texts in `cards/data/TIME_TRAVEL.json`
//! (verified 2026-08-09 — the dump's current texts are authoritative; the
//! roadmap's 34.0 nerf note confirms the keyword stacks, and Mister
//! Clocksworth TIME_038 carries three instances in the dump):
//!
//! - ×1 — TIME_000 Semi-Stable Portal, TIME_001 Chrono Daggers, TIME_002
//!   Aeon Wizard, TIME_003 Portal Vanguard, TIME_004 Conflux Crasher,
//!   TIME_008 Bygone Doomspeaker, TIME_014 Instant Multiverse, TIME_018
//!   Mend the Timeline, TIME_033 Druid of Regrowth, TIME_034 Stadium
//!   Announcer, TIME_433 Cease to Exist, TIME_441 Aeon Rend, TIME_602
//!   Wormhole, TIME_610 Shadows of Yesterday.
//! - ×3 — TIME_038 Mister Clocksworth.
//! - 0 (listed for completeness — they reference Rewind without carrying
//!   an instance of their own) — TIME_035 Time Machine (its deathrattle
//!   GETS a random Rewind card; the id pool is [`REWIND_CARD_IDS`], the
//!   random-get is W2), END_036 Morchie (her aura modifies rewind
//!   resolution — W2).
//!
//! # History semantics
//!
//! Every non-quest, non-hero card play records one [`RewindEntry`] into
//! [`crate::core::player::Player::last_played`], capped at
//! [`MAX_REWIND_HISTORY`], in play order. Minions record their battlecry
//! (combo-aware), spells their spell effect (combo-aware chosen effect),
//! weapons their battlecry; a play without an effect still occupies a slot
//! (`effect: None`). Locations record `None` — their battlecry is the
//! ACTIVATE effect resolved by LocationActivated, never by the play.
//! Countered, spellbent and secret plays record their would-be effect
//! (the play happened; the negation is a later step of the same play
//! burst). Quest cards (no effects), hero cards (internal transformation),
//! and Choose One cards (the effect resolves later through the choice
//! system — unknowable at record time; no Rewind card interacts with
//! them) do NOT record.
//!
//! The replayed effects resolve with the REWIND card as source (a standard
//! `trigger::resolve_effect` call — a replayed spell effect is therefore
//! spell-powered through the rewind card); random targeting and deaths
//! caused by replays process through the normal machinery.
use serde::{Deserialize, Serialize};

use crate::core::effect::CardEffect;

/// The maximum number of entries a player's rewind history keeps
/// (2025–2026 expansions M3-W1). The largest rewind count in the set is 3
/// (Mister Clocksworth), so 10 is a comfortable bound; the cap keeps the
/// per-play push and the replay snapshot cheap. The oldest entries are
/// dropped first.
pub const MAX_REWIND_HISTORY: usize = 10;

/// One recorded card play in a player's rewind history (2025–2026
/// expansions M3-W1): the played card's id plus the effect the play
/// resolved. `String` (not `&'static str`) keeps the history
/// (de)serializable with the player state (bincode).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewindEntry {
    /// The played card's id.
    pub card_id: String,
    /// The effect the play resolved (the combo-aware battlecry / spell
    /// effect); `None` when the card carried no effect — the entry still
    /// occupies a history slot, and [`crate::engine::rewind::resolve_replay`]
    /// skips it.
    pub effect: Option<CardEffect>,
}

/// How many PREVIOUS plays the card replays ("Rewind ×N"); 0 for cards
/// without the keyword. The 17-entry table above (verified from the
/// `TIME_TRAVEL.json` card texts, 2026-08-09); the default is 0.
#[must_use]
pub fn rewind_count(card_id: &str) -> u32 {
    match card_id {
        // ×1 — the 14 single-instance Rewind cards
        "TIME_000" | "TIME_001" | "TIME_002" | "TIME_003" | "TIME_004" | "TIME_008"
        | "TIME_014" | "TIME_018" | "TIME_033" | "TIME_034" | "TIME_433" | "TIME_441"
        | "TIME_602" | "TIME_610" => 1,
        // ×3 — Mister Clocksworth stacks three Rewind instances
        "TIME_038" => 3,
        // TIME_035 (Time Machine) and END_036 (Morchie) reference Rewind
        // without carrying an instance of their own — their interplay is
        // W2's concern.
        "TIME_035" | "END_036" => 0,
        // M3-W3 — END_000p Blessing of the Bronze (the imbued Rogue hero
        // power, The End of Time miniset): "Rewind. Get a random minion
        // from another class. It costs less." Hero powers never hook the
        // rewind play path — the replay resolves INSIDE the Rogue imbued
        // arm (trigger.rs), which reads this count; documented §22.
        "END_000p" => 1,
        _ => 0,
    }
}

/// The ids of every card that references the Rewind keyword (2025–2026
/// expansions M3-W1): TIME_035 Time Machine's random-get in W2 picks from
/// this pool, and the F5 scenarios key their fixture CardDefs on the table
/// entries.
pub const REWIND_CARD_IDS: &[&str] = &[
    "TIME_000", "TIME_001", "TIME_002", "TIME_003", "TIME_004", "TIME_008", "TIME_014", "TIME_018",
    "TIME_033", "TIME_034", "TIME_035", "TIME_038", "TIME_433", "TIME_441", "TIME_602", "TIME_610",
    "END_036",
];
