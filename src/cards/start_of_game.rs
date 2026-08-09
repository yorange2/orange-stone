//! Start-of-Game definitions (M5-W1 — Escape from Violet Hold, the
//! rule-override framework V1): the id-keyed registry of the starting-deck
//! cards whose effects fire in the game-setup phase, before turn 1.
//!
//! The engine previously had NO StartOfGame event (the EDR §14 Hamuul row
//! — "the engine has no StartOfGame event" — was the standing debt). This
//! wave lands the hook in `GameBuilder::build`: after the turn-order
//! override (Aya's flip), the starting decks are scanned in order, and
//! every card with a registered effect resolves it per player (Player1
//! first, then Player2) through the normal `trigger::resolve_effect`
//! pipeline with a scratch event queue — before the starting-deck snapshot
//! and the shuffle.
//!
//! The registry is the id-keyed analogue of `CardDef` fields — the
//! start-of-game data deliberately does NOT live in the `CardDef`
//! literals; the setup phase looks it up via `start_of_game_effect`
//! instead (mirror of `cards::quest`, `cards::kindred`, `cards::rewind`).

use crate::core::effect::CardEffect;

/// Returns the Start-of-Game effect for a starting-deck card, if any
/// (M5-W1 — the V1 rule-override registry). The effects resolve in deck
/// order per player during `GameBuilder::build`.
pub(crate) fn start_of_game_effect(card_id: &str) -> Option<CardEffect> {
    match card_id {
        // JAIL_384 Chainbreaker Hogger: "Start of Game: Duplicate all
        // other Legendary cards in your deck."
        "JAIL_384" => Some(CardEffect::HoggerStartOfGame),
        // JAIL_430 Azalina Soulsever: "Start of Game: Your starting
        // Health is 40. Your deck is 20 cards, plus 20 copied from your
        // enemy."
        "JAIL_430" => Some(CardEffect::AzalinaStartOfGame),
        // JAIL_509 Godfrey the Betrayer: "Start of Game: Overdrawn cards
        // return to your hand when you have space. They cost (1) less."
        "JAIL_509" => Some(CardEffect::GodfreyStartOfGame),
        // JAIL_860 Chef Neth'rek: "Start of Game: If your deck only has
        // cards that cost (3) or less, set your Mana to 10 after five
        // turns!"
        "JAIL_860" => Some(CardEffect::NethrekStartOfGame),
        // JAIL_800 Mug'Zee: "Start of Game: If your deck has no other
        // minions, get Mug's Hero Power. If it has no spells, get
        // Zee's!"
        "JAIL_800" => Some(CardEffect::MugzeeStartOfGame),
        // JAIL_397 Commander Beatrix (simplified, §27): the deck-building
        // pick becomes a Start-of-Game random 2-Cost minion — ten copies
        // join the starting deck.
        "JAIL_397" => Some(CardEffect::BeatrixStartOfGame),
        // NOTE: JAIL_504 Aya, Lotus Kingpin has NO Start-of-Game effect —
        // "You always go second" is the passive handled by `aya_flip` (the
        // builder-level turn-order override), and the counterfeit pick is
        // her BATTLECRY (a choose-one three-branch minion, resolved by the
        // standard play path).
        _ => None,
    }
}

/// M5-W1 — Aya, Lotus Kingpin's turn-order override: "You always go
/// second." Whether the first seat flips to second — Player1's deck holds
/// Aya while Player2's does not (both or neither → no flip, the symmetric
/// corner §27). The builder and the battle path both scan the decks with
/// this predicate, so the Coin placement and the SOG phase see the same
/// final turn order.
pub(crate) fn aya_flip(p1_has_aya: bool, p2_has_aya: bool) -> bool {
    p1_has_aya && !p2_has_aya
}
