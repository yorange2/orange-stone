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
