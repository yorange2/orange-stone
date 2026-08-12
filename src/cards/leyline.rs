//! Leyline registry (MEND W3 — the Cataclysm Mage class-set wave,
//! `exp_cata_w7.rs`): the id-keyed registry for the three Leyline cards.
//!
//! **"Leyline"** is a card-group keyword on three Mage cards — Bursting
//! Leyline (MEND_500), Crystallized Leyline (MEND_502) and Leyline Nexus
//! (MEND_504) — that the rest of the wave upgrades for the rest of the
//! game:
//!
//! - MEND_501 Ley Walker — "Battlecry: Your Leylines cost (1) less this
//!   game. Deathrattle: Get a random Leyline."
//! - MEND_503 Surge Needle — "Battlecry: Your Leylines trigger an
//!   additional time this game."
//! - MEND_506 Mystic Runesaber — "Elusive. Battlecry: Increase the
//!   effects of your Leylines by 1 this game."
//! - MEND_505 The Arcanomicon — "Get all 3 Leylines. Choose an upgrade
//!   for your Leylines." (one of the three upgrades above)
//!
//! The three upgrades live as persistent per-player flags on the `Player`
//! (`leyline_discount`, `leyline_extra_trigger`, `leyline_effect_bonus`);
//! the Leyline cards' resolve arms (`engine/trigger.rs`) read the flags,
//! and `engine/cost.rs::play_cost` applies the discount keyed by this
//! registry. Upgrades stack and last for the rest of the game (D2 note
//! §31 — the official texts carry no refresh/cap constraint, so the
//! flags are cumulative; a second Surge Needle adds another trigger).
//!
//! "Trigger an additional time" resolves as one extra REPETITION of the
//! card's effect (an extra hit / an extra summoned minion / an extra
//! drawn card), and "effect +1" as one extra unit of the card's scalar
//! ({0} — the damage, the summoned minion's Cost, the cost reduction):
//! both readings are the plain-text reading of the {1} "times" / "{0}"
//! scalars (D2 note §31, semantics verified 2026-08-12).

/// The three Leyline cards (the full official set — the registry drives
/// the cost discount, the random Leyline the Ley Walker deathrattle gets,
/// and The Arcanomicon's "Get all 3 Leylines").
pub const LEYLINE_CARD_IDS: &[&str] = &[
    "MEND_500", // Bursting Leyline
    "MEND_502", // Crystallized Leyline
    "MEND_504", // Leyline Nexus
];

/// Whether a card id is a Leyline (2025–2026 expansions M4-W3).
#[must_use]
pub fn is_leyline(card_id: &str) -> bool {
    matches!(card_id, "MEND_500" | "MEND_502" | "MEND_504")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::def::card_by_id;

    /// Every registry id resolves to a real CardDef (the hand-written
    /// `exp_cata_w7.rs` defs) — the registries must never dangle.
    #[test]
    fn leyline_ids_resolve() {
        for id in LEYLINE_CARD_IDS {
            card_by_id(id).unwrap_or_else(|| panic!("no CardDef for {id}"));
        }
    }

    /// The registry is exactly the three Leylines: the wave's other
    /// upgrade/support cards are not Leylines.
    #[test]
    fn leyline_membership_is_exact() {
        assert!(is_leyline("MEND_500"));
        assert!(is_leyline("MEND_502"));
        assert!(is_leyline("MEND_504"));
        assert!(!is_leyline("MEND_501"));
        assert!(!is_leyline("MEND_503"));
        assert!(!is_leyline("MEND_505"));
        assert!(!is_leyline("MEND_506"));
    }
}
