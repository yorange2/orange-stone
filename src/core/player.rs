//! Player definitions — PlayerId and Player state.
//!
//! Each player has a `PlayerId` (mounted on entities via a component)
//! and a `Player` struct (stored in `GameState`).
use serde::{Deserialize, Serialize};

/// Player identifier.
///
/// Uses `#[repr(u8)]` so arrays (`[T; 2]`) can be indexed efficiently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PlayerId {
    /// The first player
    Player1 = 0,
    /// The second player
    Player2 = 1,
}

impl PlayerId {
    /// Number of players.
    pub const COUNT: usize = 2;

    /// Returns the `usize` index for array subscripting.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Returns the opponent's `PlayerId`.
    #[must_use]
    pub const fn opponent(self) -> Self {
        match self {
            Self::Player1 => Self::Player2,
            Self::Player2 => Self::Player1,
        }
    }
}

/// Future Animal Companion replacement (MEND W2 — Tame Pet, Migrating
/// Elekk, Roam Free): while set, every Animal Companion summon is swapped
/// for a random Beast of cost `3 + cost_bump` instead (§30 simplification:
/// official picks among three fixed Beasts and upgrades them on repeated
/// casts; we re-sample per summon). Multiple cards that set the flag
/// accumulate the bump (official behaviour: each cast upgrades the trio).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanionReplacement {
    /// Extra mana cost added to the replacement Beast's base cost of 3.
    pub cost_bump: u32,
}

/// Player state — player data that is not at the entity level.
///
/// The hero itself is an entity (`CardType::Hero`) stored in the World.
/// `Player` holds a reference to the hero entity plus state such as mana crystals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// The player ID
    pub id: PlayerId,
    /// Handle to the hero entity
    pub hero: crate::core::entity::Entity,
    /// Total mana crystals (max 10, +1 at the start of each turn)
    pub mana_crystals: i32,
    /// Current available mana (spent when playing cards, refilled at turn start)
    pub current_mana: i32,
    /// The currently equipped weapon entity (`None` means no weapon)
    pub weapon: Option<crate::core::entity::Entity>,
    /// The current location entity (Core Set W8 — at most one location per
    /// side; playing a new one replaces the old)
    pub location: Option<crate::core::entity::Entity>,
    /// The turn the current location was played (Core Set W8 cooldown: a
    /// location cannot be activated the turn it was played)
    pub location_played_turn: u32,
    /// Hero armor
    pub armor: i32,
    /// Fatigue counter (1-based): the damage dealt by the next empty-deck draw
    /// attempt (official HS rule — docs/fatigue-roadmap.md). Starts at 1 (the
    /// first hit deals 1) and increments by 1 after each fatigue hit.
    pub fatigue: u32,
    /// Cards played this turn (for the Combo mechanic)
    pub cards_played_this_turn: u32,
    /// Minions played this turn (Pint-Sized Summoner — the first minion
    /// each turn costs less)
    pub minions_played_this_turn: u8,
    /// Mana locked by overload cards for the next turn (roadmap F1)
    pub overload_locked: i32,
    /// Friendly minions that died this turn (for resurrection effects)
    pub died_this_turn: Vec<crate::core::entity::Entity>,
    /// Entities frozen at the start of this player's turn (engine-mechanics
    /// roadmap M2 — freeze timing): a character frozen during the opponent's
    /// turn keeps Freeze through its owner's next turn (its attack is
    /// blocked), then thaws in the turn-end wrap-up of that turn
    pub frozen_at_turn_start: Vec<crate::core::entity::Entity>,
    /// The next secret costs 0 (Kirin Tor Mage, one-time)
    pub next_secret_free: bool,
    /// All spells cost 0 this turn (Millhouse Manastorm — the opponent's
    /// spells cost 0 next turn); cleared at the owner's turn start
    pub spells_cost_zero: bool,
    /// The next spell cast this turn costs this much less (Preparation —
    /// one-time, consumed by the first spell played); cleared at turn end
    pub next_spell_discount: i32,
    /// Enemy minions temporarily controlled (entity, original owner) — returned at end of turn (Shadow Madness)
    pub controlled_this_turn: Vec<(crate::core::entity::Entity, PlayerId)>,
    /// Corrupted enemy minions — destroyed at the owner's turn start (Corruption)
    pub corrupted: Vec<crate::core::entity::Entity>,
    /// Minimum minion health this turn (Commanding Shout, 0 means no limit)
    pub minion_min_health: i32,
    /// Corpses (Core Set W1 — data-driven: Malignant Horror's end-of-turn
    /// effect spends them). Gained when a friendly minion dies; any player
    /// can hold them, only Death-Knight-style cards spend them.
    pub corpses: u32,
    /// Whether the hero was healed this turn (Core Set W3a — Death Metal
    /// Knight pays Health instead of Mana when true); cleared at turn start
    pub healed_this_turn: bool,
    /// How many times the enemy hero actually lost Health this turn
    /// (2025–2026 expansions M3-W2a — Devious Coyote TIME_047's "Costs (1)
    /// less for each time the enemy hero took damage this turn").
    /// Incremented in the damage pipeline where the hero's health drops
    /// (armor absorption does not count); cleared at turn start.
    pub enemy_hero_damaged_this_turn: u32,
    /// Whether this player's hero Health changed this turn (2025–2026
    /// expansions M3-W2a — Liferender TIME_614's battlecry check; either
    /// direction, damage or heal). Set in the damage pipeline and by the
    /// heal path; cleared at turn start.
    pub hero_damaged_this_turn: bool,
    /// Turns the player has taken this game (2025–2026 expansions M3-W2a —
    /// Clockwork Rager TIME_048 "Gain +1 Health for each turn you've
    /// taken"); incremented at the player's turn start.
    pub turns_taken: u32,
    /// The opponent's cards cost this much more during the OPPONENT's next
    /// turn (2025–2026 expansions M3-W2a — TIME_716 Slow Motion "Your
    /// opponent's cards cost (1) more next turn"). The tax sits on the
    /// caster and is applied by the cost pipeline when the OTHER player
    /// plays; it expires at the caster's next turn start (after the
    /// opponent's taxed turn has passed).
    pub next_turn_enemy_cards_cost_more: i32,
    /// Nature spells cast by this player this game (2025–2026 expansions
    /// M3-W2a — Primordial Overseer TIME_213's battlecry scales with it).
    /// Incremented in the spell-cast path where the school is resolved.
    pub nature_spells_cast_total: u32,
    /// The next Demon costs this much less (Raging Felscreamer — Core Set
    /// W4a, one-time, consumed on play)
    pub next_demon_discount: i32,
    /// The next Outcast card costs this much less (Illidari Studies — Core
    /// Set W6, one-time, consumed on play)
    pub next_outcast_discount: i32,
    /// The next Combo card costs this much less this turn (Foxy Fraud —
    /// Core Set W4a, one-time)
    pub next_combo_discount: i32,
    /// Ongoing end-of-turn damage to the opponent (Alexandros Mograine —
    /// Core Set W4a, game-long)
    pub ongoing_end_turn_damage: i32,
    /// The next Hero Power costs more (Blowtorch Saboteur — Core Set W4b,
    /// one-time)
    pub hero_power_cost_more: i32,
    /// Enemy spells cost more this turn (Cult Neophyte — Core Set W4b,
    /// cleared at the owner's turn start)
    pub enemy_spell_cost_more: i32,
    /// Imbue count (2025–2026 expansions M1-W1 — the Emerald Dream imbue
    /// mechanic): every played imbue card increments it. The first imbue
    /// replaces the hero power with the class's imbued form (cost 2) when
    /// the hero is one of the six imbuing classes; later imbues scale the
    /// imbued powers' numbers — level L = imbue count (the count is the
    /// single source of truth, so a replaced-then-replaced hero power keeps
    /// its level).
    pub imbue_count: i32,
    /// Friendly spells cast while Hamuul Runetotem (EDR_845) is in play
    /// (2025–2026 expansions M1-W1): his "Repeat this every 3 spells you
    /// cast" fires an extra imbue when this hits a multiple of 3.
    pub hamuul_spells_cast: i32,
    /// Dark gifts given to this player's minions (2025–2026 expansions
    /// M1-W2 — the Emerald Dream dark-gift mechanic): every applied gift is
    /// logged in application order. Wallow, the Wretched (EDR_487) reads
    /// this list to copy every gift onto himself while in the hand or deck
    /// (registered simplification: the log records gift kinds only, not
    /// targets — fidelity-debt §14).
    pub dark_gifts_given: Vec<crate::core::component::DarkGiftKind>,
    /// "Poisonous this turn" (Barbed Thorn — 2025–2026 expansions M1-W3): the
    /// hero carries the Poison component while this is true; cleared in the
    /// turn-end wrap-up (the flag and the component expire together).
    pub hero_poisonous_this_turn: bool,
    /// Spells cast this game (2025–2026 expansions M1-W4a): the New Moon
    /// upgrade condition ("Cast 3 spells") is approximated by this total —
    /// the per-card tracking of the real set is simplified, see §14.3.
    pub spells_cast_total: u32,
    /// Hero Power uses this game (2025–2026 expansions M1-W4a): Glowroot
    /// Lure's cost reduction keyed off this count.
    pub hero_power_uses: u32,
    /// The next Hero Power costs (0) (Dreambound Disciple — 2025–2026
    /// expansions M1-W4a, one-time, consumed at the hero-power activation)
    pub next_hero_power_free: bool,
    /// Pending self-damage from Rotten Apple (2025–2026 expansions M1-W4a):
    /// damage to the hero at the END of this many of the player's own turns,
    /// and how many ticks remain (simplification: the real timing is "for
    /// the next 2 turns" from cast, see §14.3).
    pub self_damage_pending: i32,
    /// Remaining Rotten Apple ticks
    pub self_damage_turns: u8,
    /// Pending Mana Crystal gain from Fractured Power (2025–2026 expansions
    /// M1-W4a): crystals granted at the END of this many of the player's own
    /// turns, and how many ticks remain (simplification, see §14.3).
    pub crystal_gain_pending: i32,
    /// Remaining Fractured Power ticks
    pub crystal_gain_turns: u8,
    /// Temporary Mana Crystals granted "next turn only" (Emberscarred
    /// Whelp, 2025–2026 expansions M1-W5): granted at the player's next
    /// ManaRefill and spent at the same time (simplification of the real
    /// "until the end of your next turn" timing, see §14.5).
    pub temp_mana_crystal_pending: i32,
    /// Minion card IDs played this game (2025–2026 expansions M1-W4a):
    /// Twisted Webweaver's "another minion you've already played" log.
    /// `String` (not `&'static str`) so the player state stays (de)serializable.
    pub played_minion_ids: Vec<String>,
    /// Omen's attack counter (2025–2026 expansions M1-W4b): each attack adds
    /// 1 to the deathrattle's "Improves" damage (§14.4 interpretation —
    /// the official enchantment-per-minion is approximated per-player).
    pub omen_attacks: u32,
    /// How many of Tyrande's "next 3 spells cast twice" charges remain
    /// (2025–2026 expansions M1-W4b); consumed at the spell play path.
    pub spells_cast_twice_pending: u32,
    /// Dragons played this turn (2025–2026 expansions M1-W4b — Naralex's
    /// "your first Dragon each turn costs (1)"); cleared at the owner's
    /// turn start (ManaRefill).
    pub dragons_played_this_turn: u8,
    /// Card IDs of the minions Ursoc's battlecry killed (2025–2026
    /// expansions M1-W4b): the deathrattle resurrects them. Overwritten by
    /// each Ursoc battlecry; consumed by the deathrattle.
    pub ursoc_killed_ids: Vec<String>,
    /// All the player's cards cost (1) this game (2025–2026 expansions
    /// M1-W4b — Aviana's simplified immediate effect, §14.4).
    pub cards_cost_1: bool,
    /// The next card the player plays costs (0) (2025–2026 expansions
    /// M1-W4b — Agamaggan's simplified cost, §14.4; one-time, consumed on
    /// play).
    pub next_card_costs_zero: bool,
    /// Murlocs the player summons gain +1/+1 (2025–2026 expansions M2-W2 —
    /// Dive the Golakka Depths's repeatable quest reward). Set by
    /// `CardEffect::SetMurlocSummonBuff`; the friendly-summon hook in
    /// rules.rs applies a +1/+1 enchantment to every friendly Murloc
    /// summon while it is set. Game-long, permanent.
    pub murloc_summon_buff: bool,
    /// Whenever the player deals exactly 2 damage to an enemy, deal 2 more
    /// (2025–2026 expansions M2-W2 — Gorishi Colossus's battlecry). Set by
    /// `CardEffect::SetDealExact2Bonus`; the damage hook in rules.rs
    /// (the DealExactDamage quest call site) applies the bonus in-place.
    /// Game-long, permanent.
    pub deal_exact_2_bonus: bool,
    /// Kindred (2025–2026 expansions M2-W3) — every card the player played
    /// this turn pushed its kindred type (`KindredType::Spell` for spells,
    /// `KindredType::Minion(race)` for minions whose CardDef carries a
    /// race). A Kindred card's activation condition is a matching EARLIER
    /// play ("another card of the same type" — the card itself counts only
    /// because the play path pushes its type before the check). Cleared at
    /// the player's own turn end — the condition is "this turn".
    pub kindred_played: Vec<crate::cards::kindred::KindredType>,
    /// Kindred (M2-W3): TLC_251 Primalfin Challenger — "your next Kindred
    /// triggers twice". Set by the battlecry; the next OnPlay Kindred
    /// resolution resolves its effect twice and clears the flag (cost
    /// discounts and battlecry modifiers do not consume it — the W3
    /// decision, see fidelity-debt §16). Not cleared at turn end (the
    /// official flag persists until consumed).
    pub next_kindred_twice: bool,
    /// Kindred (M2-W3): TLC_428 Hot Spring Glider's battlecry — "your next
    /// Murloc costs (1) less". Applied by the cost pipeline and consumed by
    /// the next Murloc the player plays, whenever that is — no turn-end
    /// clear (the official flag persists until a Murloc is played).
    pub next_murloc_discount: i32,
    /// Herald (2025–2026 expansions M4-W2 — the Cataclysm Herald wave,
    /// exp_cata_w2.rs): the number of Herald cards this player has played
    /// this game. Incremented by the Herald resolution (cards::herald)
    /// BEFORE the class Soldier is summoned; never resets. W4's Deathwing
    /// reads it directly ("Cataclysms scale with the Herald counter").
    pub herald_count: u32,
    /// Whether the hero has Lifesteal until the end of this turn
    /// (2025–2026 expansions M4-W2 — CATA_530 Fel Infusion; the Lifesteal
    /// component rides the hero entity, the flag expires in the turn-end
    /// wrap-up, the hero_poisonous_this_turn convention)
    pub hero_lifesteal_this_turn: bool,
    /// Kindred (M2-W3): TLC_428's Kindred add-on — "your next Murloc gains
    /// Divine Shield". Consumed together with `next_murloc_discount` by
    /// the next Murloc play; the shield is applied to the played Murloc.
    pub next_murloc_divine_shield: bool,
    /// Whether the player Discovered this turn (2025–2026 expansions
    /// M2-W4a): set by the discover machinery, cleared at the turn end.
    /// Read by Storage Scuffle (costs (0)), Unearthed Artifacts (summon a
    /// 4-Cost minion instead) and Vault Breaker (discount the discovered
    /// card).
    pub discovered_this_turn: bool,
    /// Whether the player played a Quest this game (2025–2026 expansions
    /// M2-W4a): set by the quest play-path diversion; Questing Assistant
    /// (TLC_987) deals 3 damage to an enemy minion when it is set.
    pub quest_played: bool,
    /// The next Temporary card costs this much less (2025–2026 expansions
    /// M2-W4a — Spelunker's battlecry, one-time, consumed by the next
    /// Temporary card the player plays).
    pub next_temporary_discount: i32,
    /// How many times the player shuffled cards into their deck
    /// (2025–2026 expansions M2-W4a): Underbrush Tracker costs (1) less
    /// per shuffle; Knockback's damage improves per shuffle. Incremented
    /// by every shuffle-into-deck resolution.
    pub shuffled_count: u32,
    /// The Map-card chain (2025–2026 expansions M2-W4a — registered
    /// simplification, fidelity-debt §17): after a Map discover resolves,
    /// holds the discovered card entity plus the other options' ids. If
    /// the discovered card is played this turn, one random other option
    /// is added to the hand. Cleared at the turn end.
    pub map_pending: Option<(crate::core::entity::Entity, Vec<String>)>,
    /// Holy spells the player cast this turn (2025–2026 expansions
    /// M2-W4a): the cast-spell ids in cast order, from the quest
    /// registry's spell-school table. Gladesong Siren's cost condition
    /// reads the list; Creature of the Sacred Cave recasts a random entry
    /// at the turn end. Cleared at the turn end.
    pub holy_cast_ids: Vec<String>,
    /// Whether the player cast a Shadow spell this turn (2025–2026
    /// expansions M2-W4a — Gladesong Siren's cost condition); cleared at
    /// the turn end.
    pub shadow_cast_this_turn: bool,
    /// The enemy hero cannot be healed (2025–2026 expansions M2-W4a —
    /// Crater Gator's battlecry: "Until the start of your next turn, the
    /// enemy hero can't be healed"). Set on the caster; cleared at the
    /// caster's turn start.
    pub enemy_hero_cant_be_healed: bool,
    /// Ravenous Flock's pending Hatchlings (2025–2026 expansions M2-W4a):
    /// set by the spell's cast, resolved at the caster's next turn start
    /// (three 2/1 Hatchlings summoned).
    pub flock_pending: bool,
    /// Remaining Story of Lakkari activations (2025–2026 expansions
    /// M2-W4a): at the end of the owner's turn while this is > 0, discard
    /// a random card and fill the board with 3/2 Imps; decremented after
    /// each activation ("Lasts 3 turns").
    pub lakkari_ticks: u8,
    /// Hatching Ceremony's pending buff (2025–2026 expansions M2-W4c):
    /// armed at 2 by the spell's cast, decremented at each of the owner's
    /// turn ends; the +2/+2 to the owner's minions lands when it reaches 0
    /// — the end of the owner's NEXT turn after the cast (a one-shot armed
    /// at 1 would fire at the end of the cast's own turn, a full turn
    /// early).
    pub hatching_pending: u8,
    /// Soulrest Ceremony's marked minions (2025–2026 expansions M2-W4c):
    /// the friendly minions buffed by the spell ("they die at the end of
    /// your turn"); the TurnEnded handler damages each marked minion
    /// through the normal death path (deathrattles fire) and clears the
    /// list.
    pub soulrest_marked: Vec<crate::core::entity::Entity>,
    /// The hand position of the card the player most recently played
    /// (2025–2026 expansions M2-W4c — Skittish Saucier's battlecry reads
    /// it to reduce the Cost of the adjacent hand cards). Recorded at
    /// CardPlayed before the card leaves the hand; the battlecry resolves
    /// within the same play burst, so no later play can stale it.
    pub last_played_hand_index: Option<usize>,
    /// Whether the card the player most recently played was EXACTLY in
    /// the center of the hand (2025–2026 expansions M3-W2a — Precise Shot
    /// TIME_600: "If this is EXACTLY in the center of your hand, deal $5
    /// instead" — the center exists only for odd-sized hands). Captured
    /// at CardPlayed alongside `last_played_hand_index`.
    pub last_played_hand_center: bool,
    /// Chronological Aura's remaining activations (2025–2026 expansions
    /// M3-W2a — TIME_700 "At the end of your turn, summon a 3/5 Dragon
    /// with Taunt. Lasts 3 turns"): the tick counter rides the player;
    /// the end-of-turn hook summons the drake and decrements while > 0.
    pub chronological_aura_ticks: i32,
    /// The Timeway Warden's imprisonments (2025–2026 expansions M3-W2a —
    /// TIME_442): `(imprisoned minion, warden)` pairs; the warden's
    /// deathrattle awakens the imprisoned minion (removes its Dormant
    /// marker). A dead warden leaves a dangling pair until the game ends
    /// (the imprisoned minion stays dormant — the registered §20 shape).
    pub timeway_imprisoned: Vec<(crate::core::entity::Entity, crate::core::entity::Entity)>,
    /// Pending hand-stats bonus from a discover effect (2025–2026
    /// expansions M3-W2a — TIME_039 "Discover a minion. Give it +2/+2":
    /// the discover pick would be applied to the wrong card if buffed
    /// eagerly, so the bonus rides the player until the picked card is
    /// added to hand in the ChoiceResolved path).
    pub pending_discover_hand_bonus: Option<(i32, i32)>,
    /// Pending hand-cost reduction from a discover effect (2025–2026
    /// expansions M3-W2a — TIME_036 "Discover a minion. Reduce its cost
    /// by (2)"; consumed exactly like `pending_discover_hand_bonus`).
    pub pending_discover_cost_reduction: Option<i32>,
    /// Story of Sulfuras (2025–2026 expansions M2-W4a): how many times the
    /// swapped "Deal 8 damage to a random enemy" hero power has been used;
    /// after 2 uses the original hero power is restored.
    pub sulfuras_uses: u8,
    /// The hero power replaced by Story of Sulfuras (2025–2026 expansions
    /// M2-W4a) — restored after 2 uses of the swapped power.
    pub sulfuras_original: Option<crate::core::component::HeroPowerDef>,
    /// Platysaur's draw-then-discard link (2025–2026 expansions M2-W4a):
    /// (platysaur entity, drawn card entity) pairs; the deathrattle
    /// discards the linked card.
    pub platysaur_drawn: Vec<(crate::core::entity::Entity, crate::core::entity::Entity)>,
    /// The player's minions cost (2) more this turn (2025–2026 expansions
    /// M2-W4a — Wave of Tar sets this on the OPPONENT); cleared at the
    /// affected player's turn start.
    pub minions_cost_more: bool,
    /// The next Beast the player plays this turn costs this much less
    /// (2025–2026 expansions M2-W4a — Cower in Fear, one-time, cleared at
    /// the turn end).
    pub next_beast_discount: i32,
    /// The card ids of the player's deck at game start (2025–2026
    /// expansions M2-W4a — Story of the Waygate's "didn't start in your
    /// deck" set; snapshotted by GameBuilder).
    pub starting_deck: Vec<String>,
    /// The player's minions cost (5) for the rest of the game (2025–2026
    /// expansions M2-W4b — Loh's battlecry; a SET read by the play-cost
    /// pipeline).
    pub minions_cost_5: bool,
    /// The crafted-location marker of Elise the Navigator (2025–2026
    /// expansions M2-W4b — registered simplification §18: the starting-deck
    /// check sets this; no custom-location machinery exists yet).
    pub elise_location_crafted: bool,
    /// The player's rewind history (2025–2026 expansions M3-W1 — the
    /// Across the Timeways rewind primitive): one entry per card play —
    /// the played card's id and the effect the play resolved (battlecry /
    /// spell effect, combo-aware), capped at
    /// [`crate::cards::rewind::MAX_REWIND_HISTORY`]. Quest, hero, and
    /// Choose One plays record nothing; plays without an effect record
    /// `effect: None` but still occupy a slot. See
    /// `crate::cards::rewind` for the record semantics.
    pub last_played: Vec<crate::cards::rewind::RewindEntry>,
    /// The played minion whose play burst is in progress (2025–2026
    /// expansions M3-W1 — the rewind primitive): set in the CardPlayed
    /// path when a minion is played, consumed by that minion's
    /// MinionSummoned event — which is also enqueued for effect-summoned
    /// minions, so the marker distinguishes a PLAYED minion (records into
    /// the rewind history) from a summoned one (never records). `None`
    /// between plays; a stale marker (e.g. a Choose One minion whose play
    /// records nothing) is inert — the equality check cannot match a
    /// future summon, and the next play overwrites it.
    pub rewind_played_minion: Option<crate::core::entity::Entity>,
    /// Murozond, Unbounded's INFINITY arming (2025–2026 expansions M3-W2b —
    /// TIME_024): the battlecry records the minion here; the start of the
    /// owner's NEXT turn sets its Attack to the INFINITY cap (the
    /// documented `exp_tmw_w2b::INFINITY_ATTACK_CAP`) and clears the flag.
    /// None between plays; a stale marker is inert — the next arming
    /// overwrites it and the turn-start hook checks the minion is still on
    /// the board.
    pub murozond_infinite_pending: Option<crate::core::entity::Entity>,
    /// Timelooper Toki's pending spells (2025–2026 expansions M3-W2b —
    /// TIME_861 "Get 3 random spells from the past. When all 3 are played,
    /// add another Timelooper Toki to your hand"): the ids of the 3 spells
    /// the battlecry generated; the CardPlayed path removes a played id,
    /// and when the list empties adds TIME_861 to the hand. §21 — the
    /// official "from the past" pool is approximated by the whole active
    /// spell window (the SpellCostGE8 precedent).
    pub toki_pending_spells: Vec<String>,
    /// The Fins Beyond Time's hand snapshot (2025–2026 expansions M3-W2b —
    /// TIME_706): the battlecry stores the hand's card ids here, shuffles
    /// them into the deck and draws replacements; the end-of-turn restore
    /// returns the snapshot (or is a no-op when None — a silenced/removed
    /// battlecry never snapshots).
    pub hand_swap_snapshot: Option<Vec<String>>,
    /// The minion card ids the OPPONENT played last turn (2025–2026
    /// expansions M3-W2b — Chrono-Lord Epoch TIME_714 "Destroy all minions
    /// your opponent played last turn"): snapshotted from
    /// `minions_played_this_turn_ids` at the opponent's turn end.
    pub last_turn_minion_play_ids: Vec<String>,
    /// The minion card ids THIS player played this turn (2025–2026
    /// expansions M3-W2b — the per-turn counterpart of `played_minion_ids`:
    /// pushed at CardPlayed, cleared at the owner's turn start, snapshotted
    /// into `last_turn_minion_play_ids` at the owner's turn end).
    pub minions_played_this_turn_ids: Vec<String>,
    /// The next Demon the player plays costs (1) (2025–2026 expansions
    /// M3-W2b — TIME_446 The Eternal Hold's deck-no-minions fallback;
    /// one-time, consumed by the next Demon play in the CardPlayed path).
    pub next_demon_cost_one: bool,
    /// The player's own Overload total this game (2025–2026 expansions
    /// M3-W3 — END_030 Haywire Hornswog "costs (1) less for each Overload
    /// you've been dealt"): every overloaded card play bumps this (the
    /// existing overload lock site); never resets — game-long).
    pub overload_total: i32,
    /// The player skips their NEXT turn (2025–2026 expansions M3-W3 —
    /// END_037 Endtime Murozond's battlecry): consumed at the start of the
    /// player's next turn — the TurnStarted handler clears the flag and
    /// immediately pushes `Event::TurnEnded` so the normal turn-end
    /// sequence (triggers, wrap-up, pass to the opponent) runs.
    pub skip_next_turn: bool,
    /// A hand card whose cost is set to Infinity (2025–2026 expansions
    /// M3-W3 — END_034 Crumblecrusher "set a random card in your hand's
    /// cost to Infinity"): the affected card entity; the cost layer
    /// reports the INFINITY cap while set. `None` when no card is set.
    pub hand_card_infinity: Option<crate::core::entity::Entity>,
    /// Undead minions this player played this turn (2025–2026 expansions
    /// M3-W3 — END_003 Finality / END_003p imbued Death Knight hero power
    /// "your first Undead each turn has +N Attack"): incremented at
    /// CardPlayed for Undead minions, reset at the owner's ManaRefill step.
    pub undead_played_this_turn: u32,
    /// The Eternal Firebolt copy pending the player's next turn end
    /// (2025–2026 expansions M3-W3 — END_025 "if it kills a minion, add
    /// another Eternal Firebolt to your hand"): set by the predicted-death
    /// split, checked and cleared at the owner's turn end — if the target
    /// died the copy is added (a fresh END_025), otherwise the flag just
    /// clears (the same one-card replay pattern as §22).
    pub eternal_flame_target: Option<crate::core::entity::Entity>,
    /// Remaining Chronikar ticks (2025–2026 expansions M3-W3 — END_006
    /// Chronikar: +3 Attack to your hero this turn and the next two
    /// turns — 3 total buffs): the battlecry sets 2 (the immediate buff
    /// is the 1st), each start of turn decrements and re-buffs while > 0.
    pub chronikar_ticks: i32,
    /// Remaining Acceleration Aura turns (2025–2026 expansions M3-W3 —
    /// END_011: your next three turns start with an extra Mana Crystal):
    /// the ManaRefill hook grants +1 temporary mana per owner turn start
    /// while this is > 0, then decrements. Set to 3 by the spell.
    pub acceleration_aura_ticks: i32,
    /// Splintered Reality Treants that died this game (2025–2026
    /// expansions M3-W3 — END_009 "each of your Treants that died while
    /// this is on the board made it cost (1) less"): bumped by the
    /// MinionDied handler when the END_009t Treant dies, read by the
    /// END_009 cost layer. Game-long, never resets.
    pub treants_died_total: u32,
    /// Deathwing's cost reduction (2025–2026 expansions M4-W4 — CATA_497
    /// Ultraxion "Reduce Deathwing's Cost by ({1})"): accumulated per
    /// Ultraxion herald resolution (the herald count at play time); the
    /// cost pipeline subtracts it from CATA_190h's play cost.
    pub deathwing_cost_reduction: i32,
    /// Mana spent this turn (2025–2026 expansions M4-W4 — the "spent X
    /// Mana while holding this" cards CATA_131/132/140 and CATA_130
    /// Crystalspine Cub's LastManaCrystalSpent trigger): every mana
    /// deduction adds the spent amount here; cleared at the owner's turn
    /// start.
    pub mana_spent_this_turn: i32,
    /// Spell damage this player dealt this turn (2025–2026 expansions
    /// M4-W4 — CATA_452 Spellweaver's Brilliance costs (1) less per,
    /// CATA_483 Unstable Spellcaster checks it, CATA_487 Raincaller's
    /// first-damage trigger): incremented by the damage pipeline when a
    /// friendly spell deals damage; cleared at the owner's turn start.
    pub spell_damage_dealt_this_turn: i32,
    /// Whether Raincaller's +2 Attack has been consumed this turn
    /// (2025–2026 expansions M4-W4 — CATA_487 "The first time you deal
    /// damage with a spell each turn"): set when the FriendlySpellDealtDamage
    /// trigger resolves; cleared at the owner's turn start.
    pub first_spell_damage_gain_used: bool,
    /// Healing bonus from friendly effects (2025–2026 expansions M4-W4 —
    /// CATA_216 Cleansing Cleric "Your healing effects restore 2 more
    /// Health this game"): added to every friendly heal; game-long.
    pub healing_bonus: i32,
    /// The next Healing effect this turn deals damage instead (2025–2026
    /// expansions M4-W4 — CATA_301 Ruby Sanctum): set by the location
    /// activation, consumed by the first friendly heal, cleared at the
    /// owner's turn end.
    pub next_heal_deals_damage: bool,
    /// The next Murloc that costs (3) or less costs Health instead of
    /// Mana (2025–2026 expansions M4-W4 — CATA_180 War'loc): consumed by
    /// the next qualifying Murloc play (the CostHealth convention — the
    /// play validation still requires the mana, the payment diverts).
    pub next_murloc_costs_health: bool,
    /// Cards this player discarded this game (2025–2026 expansions M4-W4 —
    /// CATA_493 Duke of Below "Has +2/+2 for each card you've discarded
    /// this game"): incremented by the discard sites; the Duke's aura
    /// re-bakes on change.
    pub discarded_this_game: u32,
    /// Fel spells this player cast this game (2025–2026 expansions M4-W4 —
    /// CATA_529 Ravenous Felfisher "Costs (1) less for each Fel spell
    /// you've cast this game"): incremented at the spell-cast site where
    /// the school is resolved; never resets.
    pub fel_spells_cast_this_game: u32,
    /// Times a friendly character attacked this game (2025–2026 expansions
    /// M4-W4 — CATA_568 Muradin's Last Stand "Costs (1) less for each
    /// time a friendly character attacked this game"): incremented by the
    /// attack resolution; never resets.
    pub friendly_attacks_this_game: u32,
    /// Whether the player played a Fire spell this turn (2025–2026
    /// expansions M4-W4 — CATA_584 Erupting Volcano "If you've played a
    /// Fire spell this turn, deal 3 more"): set at the spell-cast site;
    /// cleared at the owner's turn start.
    pub fire_spell_played_this_turn: bool,
    /// Whether the player's Dragons have Rush this game (2025–2026
    /// expansions M4-W4 — CATA_553 Ebyssian "Your Dragons have Rush this
    /// game"): consulted by the effective-Rush helper; game-long.
    pub dragons_have_rush: bool,
    /// Whether the player's hero power costs (1) (2025–2026 expansions
    /// M4-W4 — CATA_615t Genn, Worgen King "Upgrade your starting Hero
    /// Power. It costs (1)"): consulted by the hero-power activation cost;
    /// game-long.
    pub hero_power_cost_1: bool,
    /// The Cost of the last card the player played (2025–2026 expansions
    /// M4-W4 — CATA_616 Gronn Giant "This minion's Cost is reduced by the
    /// Cost of the last card you played"): recorded at the CardPlayed
    /// path; 0 before the first play.
    pub last_played_card_cost: i32,
    /// Whether the last mana deduction emptied the pool (2025–2026
    /// expansions M4-W4 — CATA_130 Crystalspine Cub "Whenever you spend
    /// your last Mana Crystal"): set by the play-cost deduction, consumed
    /// by the CardPlayed handler right after the player block (the
    /// trigger fires after the borrow ends); cleared every play.
    pub last_mana_crystal_spent_pending: bool,
    /// The 1-Cost card ids the player played this game (2025–2026
    /// expansions M4-W4 — CATA_560 Confront the Tol'vir "Replay each
    /// 1-Cost card you've played this game"): pushed at CardPlayed for
    /// 1-Cost plays, in play order; never resets.
    pub played_one_cost_cards: Vec<String>,
    /// Remaining turns of doubled friendly end-of-turn effects (2025–2026
    /// expansions M4-W4 — CATA_480 Sandfury Aura "Your minions' end of
    /// turn effects trigger twice. Lasts 3 turns"): while > 0, the
    /// EndTriggers step resolves end-of-turn effects twice; decremented
    /// at each of the owner's turn ends.
    pub end_turn_effects_twice_turns: i32,
    /// Whether Sylvanas's Triumph was played earlier this game (2025–2026
    /// expansions M4-W4 — CATA_557 "If you've played another copy of
    /// this, hit all enemies instead"): set at the cast site; never
    /// resets.
    pub sylvanas_triumph_played: bool,
    /// Commander Geddon's draw replacement (2025–2026 expansions M4-W4 —
    /// CATA_591 "Instead of drawing each turn, Discover a card from your
    /// deck. It costs (3) less. Destroy the others"): while set, the
    /// DrawStep hooks into the discover machinery instead of a normal
    /// draw; game-long.
    pub geddon_discover_draw: bool,
    /// The spell absorbed by Crackling Cloudstrider (2025–2026 expansions
    /// M4-W4 — CATA_563 "Choose a spell in your hand that costs (4) or
    /// less to absorb. Deathrattle: Cast it"): the chosen spell entity;
    /// `None` when nothing is absorbed.
    pub absorbed_spell: Option<crate::core::entity::Entity>,
    /// The card Gemstone Hoarder chose to discard (2025–2026 expansions
    /// M4-W4 — CATA_897 "Choose a card in your hand to discard.
    /// Deathrattle: Get it back. It costs (1) less"): the chosen card
    /// entity; the deathrattle returns and discounts it.
    pub hoarded_card: Option<crate::core::entity::Entity>,
    /// The cards Iso'rath devoured (2025–2026 expansions M4-W4 — CATA_481
    /// "Devour 2 random cards from the opponent's hand... Deathrattle:
    /// Return them"): the devoured card entities (removed from the
    /// opponent's hand); the deathrattle returns them to the owner.
    pub devoured_cards: Vec<crate::core::entity::Entity>,
    /// The card summoned at the start of the player's next turn
    /// (2025–2026 expansions M4-W4 — CATA_528 Sigil of the Seas "At the
    /// start of your next turn, summon a 3/3 Naga with Taunt"): the
    /// card id; consumed by the TurnStarted handler. `String` (not
    /// `&'static str`) so the player state stays (de)serializable.
    pub next_turn_summon: Option<String>,
    /// The Dragon Breath damage stored by Blackwing Experiment (2025–2026
    /// expansions M4-W4 — CATA_464 "Deathrattle: Get a 2-Cost spell that
    /// deals this minion's Attack damage"): the attack value captured at
    /// the deathrattle; the gotten spell's damage is fixed by it.
    pub dragon_breath_damage: i32,
    /// Torch's pending return (2025–2026 expansions M4-W4 — CATA_585
    /// "Deal 8 damage to a damaged minion. Return this to hand with any
    /// excess damage"): set when the spell is played, consumed by the
    /// play path if the target dies (the registered §26 approximation —
    /// the official excess-damage return is a die-check).
    pub torch_return_pending: bool,
    /// Alexstrasza's pending full-health payoff (2025–2026 expansions
    /// M4-W4 — CATA_307 "Set your remaining Health to 15. When you reach
    /// full Health, deal 15 damage to your opponent"): the damage amount;
    /// the heal pipeline checks it when the hero reaches full health and
    /// clears it.
    pub alexstrasza_full_health_pending: Option<i32>,
    /// The pending "choose a card in your hand" action (2025–2026
    /// expansions M4-W4 — CATA_200/209/477/490/563/566/697/721/979):
    /// created by the `CardEffect::ChooseHandCard` resolution, consumed
    /// by the ChoiceResolved handler; `None` between choices.
    pub pending_choose_hand: Option<crate::cards::choose_hand_card::ChooseHandCardKind>,
    /// The pending Cataclysm choice state (2025–2026 expansions M4-W4 —
    /// Deathwing's battlecry): the number of picks left and the
    /// already-picked Cataclysm ids (picks are distinct); `None` outside
    /// the Deathwing play burst.
    pub pending_cataclysms: Option<(u32, Vec<String>)>,
    /// Escape from Violet Hold W1 — JAIL_509 Godfrey the Betrayer's
    /// StartOfGame override is active for this player: overdrawn cards
    /// return to the hand instead of burning (F-A11 override).
    pub godfrey_overdraw_return: bool,
    /// Escape from Violet Hold W1 — JAIL_509: the cards that would have
    /// been burned sit in the SetAside zone (entity, original cost) until
    /// the hand has room; they return costing (1) less.
    pub godfrey_held_cards: Vec<(crate::core::entity::Entity, i32)>,
    /// Escape from Violet Hold W1 — JAIL_860 Chef Neth'rek's StartOfGame
    /// check passed (starting deck is all ≤3-cost cards): set Mana to 10
    /// after five turns have passed.
    pub nethrek_mana_after_five: bool,
    /// Escape from Violet Hold W1 — JAIL_860: how many of the player's
    /// own turns have elapsed since the game started (Neth'rek's timer).
    pub nethrek_turns_elapsed: u32,
    /// Escape from Violet Hold W1 — JAIL_421 Warptooth: the distinct
    /// friendly characters that lost Health this turn (cleared at the
    /// owner's turn start). At 4 distinct characters the card summons
    /// itself from hand or deck.
    pub warptooth_damaged_ids: Vec<crate::core::entity::Entity>,
    /// Escape from Violet Hold W1 — JAIL_122 Jailhouse Manastorm: while
    /// this is on the board, each spell this player casts summons a
    /// random minion of the same Cost. (Game-long flag on the player —
    /// set by the battlecry, cleared when the minion dies.)
    pub manastorm_after_spell: bool,
    /// Escape from Violet Hold W1 — JAIL_504 Aya, Lotus Kingpin: the card
    /// id that replaces THE_COIN in this player's hand (the upgraded
    /// counterfeit token chosen at start of game); `None` = normal Coin.
    pub coin_replacement: Option<String>,
    /// Escape from Violet Hold W1 — JAIL_800 Mug'Zee: the player received
    /// Mug's Hero Power (deck has no other minions).
    pub mugzee_mug_magic: bool,
    /// Escape from Violet Hold W1 — JAIL_800 Mug'Zee: the player received
    /// Zee's Might (deck has no spells): minions cost (2) less while none
    /// have been played this turn, and five minion plays in one turn
    /// empower the next summoned minion's battlecry.
    pub mugzee_zee_might: bool,
    /// Escape from Violet Hold W1 — JAIL_800: how many minions the player
    /// played this turn (Zee's Might counter; reset at turn start).
    pub zee_might_counter: u8,
    /// Escape from Violet Hold W1 — JAIL_800: five minions played in one
    /// turn — the next minion summoned this turn has its battlecry
    /// resolved a second time.
    pub zee_might_ready: bool,
    /// Escape from Violet Hold W1 — the Prepare keyword: a Prepare card
    /// was dragged onto the deck this turn (once per turn).
    pub prepare_used_this_turn: bool,
    /// Escape from Violet Hold W1 — the Prepare keyword: cards that have
    /// been prepared (their one-time Prepare discount has been applied).
    pub prepared_cards: Vec<crate::core::entity::Entity>,
    /// Escape from Violet Hold W1 — JAIL_446 Blood Doctor Thal'ena: the
    /// hero power swapped in by the battlecry costs Corpses instead of
    /// Mana (2 Corpses per use).
    pub thalena_corpses_hero_power: bool,
    /// The spells cast this turn (2025–2026 expansions M5-W2 — the Violet
    /// Hold closing wave: the Molten Gold / Frostshatter / Stormfury
    /// elemental-chain counter; incremented in the SpellCast handler and
    /// cleared at the owner's turn start).
    pub spells_cast_this_turn: u32,
    /// The rewind history length at this turn's start (M5-W2 — Slice and
    /// Dice: replayed plays start from this mark; set at the owner's turn
    /// start).
    pub rewind_turn_start_len: usize,
    /// The card ids discarded by Zuramat's Prison (M5-W2 — the freed
    /// Zuramat the Obliterator plays one per turn).
    pub zuramat_discarded: Vec<String>,
    /// The suspect of Inspector Murloc Holmes' investigation: the card id
    /// and the turn it was investigated (M5-W2 — if the other player plays
    /// that id during the suspect's next turn, the caster gets 3 Coins).
    pub murloc_holmes_suspect: Option<(String, u32)>,
    /// The suspect card id of Ancient Augur's investigation (M5-W2 — the
    /// matching enemy hand card is discarded by the deathrattle).
    pub augur_suspect: Option<String>,
    /// The card ids Irida Sinseeker sent to the Void (M5-W2 — two random
    /// ones return to hand at the start of the owner's turns).
    pub void_cards: Vec<String>,
    /// The Reinforcement Aura tick count (M5-W2 — at each end of the
    /// owner's turn, a random deck minion costing 2 or less is summoned
    /// while the count is above zero).
    pub reinforcement_aura_ticks: u8,
    /// The Void Soul level (M5-W2 — each cast summons a random Demon of
    /// cost (1 + level) and increments the level; §28 approximation).
    pub void_soul_level: i32,
    /// The cards played this game that cost (2) (M5-W2 — Jade Guardians'
    /// cost-reduction counter; incremented at the play-cost deduction
    /// site).
    pub cards_played_cost_2: u32,
    /// Whether a card copied from the opponent was played (M5-W2 — the
    /// Unshackle Soul cost read; set in the CardPlayed handler when the
    /// played card carries the `CopiedFromOpponent` marker).
    pub copied_from_opponent_played: bool,
    /// The Captured Archmage deaths this game (M5-W2 — JAIL_974's
    /// deathrattle fires Fireball once the count reaches 4; incremented in
    /// the MinionDied handler, so the dying archmage itself is never
    /// counted — the deathrattle resolves before the increment).
    pub jail974_deaths: u32,
    /// Future Animal Companion replacement (MEND W2): when `Some`, Animal
    /// Companion summons resolve to a random Beast of cost
    /// `3 + cost_bump` instead of the fixed Huffer/Leokk/Misha trio.
    pub companion_replacement: Option<CompanionReplacement>,
    /// Extra Animal Companions per summon (MEND W2 — Talya Earthstrider):
    /// `RandomPool::Companion` resolution summons `1 + companion_bonus`
    /// Beasts, each independently subject to `companion_replacement`.
    pub companion_bonus: u32,
    /// The Leyline play-cost discount (MEND W3 — Ley Walker / The
    /// Arcanomicon): "Your Leylines cost (1) less this game." The cost
    /// pipeline (`engine/cost.rs`) subtracts the accumulated discount
    /// from every card in the `cards::leyline` registry.
    pub leyline_discount: u32,
    /// Extra Leyline activations (MEND W3 — Surge Needle / The
    /// Arcanomicon): "Your Leylines trigger an additional time this
    /// game." The three Leyline cards' {1} scalars (hit / summon / draw
    /// counts) read this at resolution time.
    pub leyline_extra_trigger: u32,
    /// The Leyline effect-magnitude bonus (MEND W3 — Mystic Runesaber /
    /// The Arcanomicon): "Increase the effects of your Leylines by 1
    /// this game." The three Leyline cards' {0} scalars (damage,
    /// summoned-minion cost, cost reduction) read this at resolution.
    pub leyline_effect_bonus: u32,
    /// The Silver Hand Recruit game-long Attack bonus (MEND W4 — the
    /// Paladin class-set wave: Brash Battlemaster MEND_800 / Emboldening
    /// Blade MEND_803): "Give your Silver Hand Recruits +1 Attack this
    /// game." Applied at every CORE_GVG_061t creation point — the summon
    /// resolution (`resolve_summon_doubled`) and the hand-add path
    /// (`add_card_to_hand`) — so board, hand and played-from-hand
    /// Recruits all honor it for the rest of the game (§32).
    pub silver_hand_attack_bonus: u32,
    /// The Silver Hand Recruit game-long Health bonus (MEND W4 — the
    /// Paladin class-set wave: Resilient Savior MEND_801 / Emboldening
    /// Blade MEND_803): "Give your Silver Hand Recruits +1 Health this
    /// game." Applied alongside `silver_hand_attack_bonus` at every
    /// CORE_GVG_061t creation point (§32).
    pub silver_hand_health_bonus: u32,
}

impl Player {
    /// Create a new player state.
    #[must_use]
    pub const fn new(id: PlayerId, hero: crate::core::entity::Entity, mana_crystals: i32) -> Self {
        Self {
            id,
            hero,
            mana_crystals,
            current_mana: mana_crystals,
            weapon: None,
            location: None,
            location_played_turn: 0,
            armor: 0,
            fatigue: 1,
            cards_played_this_turn: 0,
            minions_played_this_turn: 0,
            overload_locked: 0,
            died_this_turn: Vec::new(),
            next_secret_free: false,
            spells_cost_zero: false,
            next_spell_discount: 0,
            controlled_this_turn: Vec::new(),
            corrupted: Vec::new(),
            minion_min_health: 0,
            frozen_at_turn_start: Vec::new(),
            corpses: 0,
            healed_this_turn: false,
            enemy_hero_damaged_this_turn: 0,
            hero_damaged_this_turn: false,
            turns_taken: 0,
            nature_spells_cast_total: 0,
            next_turn_enemy_cards_cost_more: 0,
            next_demon_discount: 0,
            next_outcast_discount: 0,
            next_combo_discount: 0,
            ongoing_end_turn_damage: 0,
            hero_power_cost_more: 0,
            enemy_spell_cost_more: 0,
            imbue_count: 0,
            hamuul_spells_cast: 0,
            dark_gifts_given: Vec::new(),
            hero_poisonous_this_turn: false,
            spells_cast_total: 0,
            hero_power_uses: 0,
            next_hero_power_free: false,
            self_damage_pending: 0,
            self_damage_turns: 0,
            crystal_gain_pending: 0,
            crystal_gain_turns: 0,
            temp_mana_crystal_pending: 0,
            played_minion_ids: Vec::new(),
            omen_attacks: 0,
            spells_cast_twice_pending: 0,
            dragons_played_this_turn: 0,
            ursoc_killed_ids: Vec::new(),
            cards_cost_1: false,
            next_card_costs_zero: false,
            murloc_summon_buff: false,
            deal_exact_2_bonus: false,
            kindred_played: Vec::new(),
            next_kindred_twice: false,
            next_murloc_discount: 0,
            next_murloc_divine_shield: false,
            herald_count: 0,
            hero_lifesteal_this_turn: false,
            discovered_this_turn: false,
            quest_played: false,
            next_temporary_discount: 0,
            shuffled_count: 0,
            map_pending: None,
            holy_cast_ids: Vec::new(),
            shadow_cast_this_turn: false,
            enemy_hero_cant_be_healed: false,
            flock_pending: false,
            lakkari_ticks: 0,
            hatching_pending: 0,
            soulrest_marked: Vec::new(),
            last_played_hand_index: None,
            last_played_hand_center: false,
            chronological_aura_ticks: 0,
            timeway_imprisoned: Vec::new(),
            pending_discover_hand_bonus: None,
            pending_discover_cost_reduction: None,
            sulfuras_uses: 0,
            sulfuras_original: None,
            platysaur_drawn: Vec::new(),
            minions_cost_more: false,
            next_beast_discount: 0,
            starting_deck: Vec::new(),
            minions_cost_5: false,
            elise_location_crafted: false,
            last_played: Vec::new(),
            rewind_played_minion: None,
            murozond_infinite_pending: None,
            toki_pending_spells: Vec::new(),
            hand_swap_snapshot: None,
            last_turn_minion_play_ids: Vec::new(),
            minions_played_this_turn_ids: Vec::new(),
            next_demon_cost_one: false,
            overload_total: 0,
            skip_next_turn: false,
            hand_card_infinity: None,
            undead_played_this_turn: 0,
            eternal_flame_target: None,
            chronikar_ticks: 0,
            acceleration_aura_ticks: 0,
            treants_died_total: 0,
            deathwing_cost_reduction: 0,
            mana_spent_this_turn: 0,
            spell_damage_dealt_this_turn: 0,
            first_spell_damage_gain_used: false,
            healing_bonus: 0,
            next_heal_deals_damage: false,
            next_murloc_costs_health: false,
            discarded_this_game: 0,
            fel_spells_cast_this_game: 0,
            friendly_attacks_this_game: 0,
            fire_spell_played_this_turn: false,
            dragons_have_rush: false,
            hero_power_cost_1: false,
            last_played_card_cost: 0,
            last_mana_crystal_spent_pending: false,
            played_one_cost_cards: Vec::new(),
            end_turn_effects_twice_turns: 0,
            sylvanas_triumph_played: false,
            geddon_discover_draw: false,
            absorbed_spell: None,
            hoarded_card: None,
            devoured_cards: Vec::new(),
            next_turn_summon: None,
            dragon_breath_damage: 0,
            torch_return_pending: false,
            alexstrasza_full_health_pending: None,
            pending_choose_hand: None,
            pending_cataclysms: None,
            godfrey_overdraw_return: false,
            godfrey_held_cards: Vec::new(),
            nethrek_mana_after_five: false,
            nethrek_turns_elapsed: 0,
            warptooth_damaged_ids: Vec::new(),
            manastorm_after_spell: false,
            coin_replacement: None,
            mugzee_mug_magic: false,
            mugzee_zee_might: false,
            zee_might_counter: 0,
            zee_might_ready: false,
            prepare_used_this_turn: false,
            prepared_cards: Vec::new(),
            thalena_corpses_hero_power: false,
            spells_cast_this_turn: 0,
            rewind_turn_start_len: 0,
            zuramat_discarded: Vec::new(),
            murloc_holmes_suspect: None,
            augur_suspect: None,
            void_cards: Vec::new(),
            reinforcement_aura_ticks: 0,
            void_soul_level: 0,
            cards_played_cost_2: 0,
            copied_from_opponent_played: false,
            jail974_deaths: 0,
            companion_replacement: None,
            companion_bonus: 0,
            leyline_discount: 0,
            leyline_extra_trigger: 0,
            leyline_effect_bonus: 0,
            silver_hand_attack_bonus: 0,
            silver_hand_health_bonus: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_id_index() {
        assert_eq!(PlayerId::Player1.index(), 0);
        assert_eq!(PlayerId::Player2.index(), 1);
    }

    #[test]
    fn player_id_opponent() {
        assert_eq!(PlayerId::Player1.opponent(), PlayerId::Player2);
        assert_eq!(PlayerId::Player2.opponent(), PlayerId::Player1);
    }
}
