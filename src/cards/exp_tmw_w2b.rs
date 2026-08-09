//! M3-W2b cards (exp_tmw_w2b) — Across the Timeways sub-roadmap W2, second
//! split: the 25 Across-the-Timeways legendaries implemented in this wave
//! (TIME_038 Mister Clocksworth and TIME_063 Timelord Nozdormu are W2a
//! shapes shipped in PR #152 and are NOT here), plus the 12 tokens they
//! produce.
//!
//! Implementation decisions (per the M3-W2b spec, verified against
//! `cards/data/TIME_TRAVEL.json` and `/tmp/hs_full.json` 2026-08-09; the
//! fidelity rows are fidelity-debt.md §21, en + zh):
//! - **Fabled is NOT implemented** (W2a decision, consistent with
//!   TIME_870/850 shapes): TIME_005 Rafaam, TIME_009 Gelbin, TIME_020
//!   Broxigar, TIME_209 Muradin, TIME_211 Azshara, TIME_609 Sylvanas,
//!   TIME_619 Talanji, TIME_850 Lo'Gosh, TIME_852 Sindragosa, TIME_875
//!   Garona and TIME_890 Medivh carry the keyword in the official text
//!   only. The Fabled companion mechanics are simplified: TIME_005
//!   (Fabled+ — a 40-card deck of 10 Rafaams) is a plain 10/10;
//!   TIME_020's start-of-game disappear/reappear is a plain 12/12 with
//!   Charge.
//! - **INFINITY** (TIME_024 Murozond): a 32-bit engine has no unbounded
//!   value — the play effect arms a player flag and the start-of-turn hook
//!   sets the Attack to `INFINITY_ATTACK_CAP` (100; shared with W3).
//! - **Deios (TIME_064)** — official text "Your Battlecries, Deathrattles,
//!   Hero Power, and end of turn effects trigger twice": an
//!   `AuraEffect::DoubleTriggers` aura (silenceable). The rules hooks
//!   re-resolve the pre-captured effect ONCE per site (a stacked
//!   BattlecryTwice dark gift resolves 3 times, never 4, §21).
//! - **The played-log cards** read `Player::played_minion_ids` (all-time
//!   minion play log) / `last_played` (rewind history): TIME_609 Sylvanas
//!   counts played Alleria/Vereesa (TIME_609t1/t2) and deals 2 damage
//!   once per sister; TIME_103 Chromie's deathrattle adds a copy of each
//!   of the last `count` distinct played cards (the official "cards
//!   you've played this game" is bounded by MAX_REWIND_HISTORY, count 10).
//! - **The sisters' own battlecries are simplified (§21)**: TIME_609t1
//!   Alleria discovers a random spell (the repeat rider is approximated by
//!   Sylvanas' log); TIME_609t2 Vereesa is a plain 2/4 (no deck-buff
//!   variant exists).
//! - **Locations** (CardType::Location, the Core Set W8 representation):
//!   TIME_446 The Eternal Hold, TIME_211t1 The Well of Eternity,
//!   TIME_211t2 Zin-Azshari and TIME_890t2 Karazhan carry their activation
//!   effects in the battlecry slot. Karazhan's "Costs (0) if you're
//!   wielding Atiesh" rider is simplified — Atiesh (TIME_890t) is skipped
//!   (§21); the MEDIVH side of the pair IS implemented (play_cost: Medivh
//!   costs 0 while Karazhan is your location).
//! - **Bwonsamdi boons (TIME_619)**: the choose-one branches grant Taunt /
//!   Lifesteal / Rush (option labels "Boon of Power"/"Boon of Longevity"/
//!   "Boon of Speed"); the official boon's "+2/+2 and minions-summoned-
//!   cost-more" riders are simplified away (§21).
//! - **D2 random picks**: every targetable/choice pick that would be a
//!   player choice in the real game is a D2 random pick (the established
//!   convention) — TIME_435 Eternus' target, TIME_713's Chest summon,
//!   TIME_850's attacked enemy, TIME_211t2's copied minion.
//! - **Stats** follow the official JSON data (`/tmp/hs_full.json`
//!   2026-08-09), including TIME_852 Sindragosa 2/8 and TIME_435 Eternus
//!   6/2.
//!
//! The 25 cards and 12 tokens register in `sets::HANDWRITTEN_EXPANSION_CARDS`
//! (the M3-W2b block); TIME_875 Garona is also pool-open
//! (`POOL_OPEN_CARDS` — it reads the opponent's hand for King Llane).

use crate::cards::def::CardDef;
use crate::core::component::AuraEffect;
use crate::core::component::AuraTarget;
use crate::core::component::CardType;
use crate::core::component::Race;
use crate::core::effect::CardEffect;
use crate::core::effect::DiscoverPool;
use crate::core::effect::EffectTarget;
use crate::core::effect::KeywordKind;

/// The engine's finite stand-in for "INFINITY" (TIME_024 Murozond, W2b —
/// shared with the W3 wave): an Attack of 100 wins every trade and ends
/// every game in one swing; there is no unbounded value in a 32-bit
/// engine (fidelity-debt §21).
pub const INFINITY_ATTACK_CAP: i32 = 100;

/// TIME_005 Timethief Rafaam — 10-cost 10/10 (MINION). Fabled+ — "Your deck
/// size is 40, but has 10 Rafaams! Battlecry: If you played the rest,
/// destroy the enemy hero" — simplified to a plain 10/10 (§21: the
/// Fabled+ deck construction and the played-all check do not exist in the
/// engine).
pub const RAFAM_THE_UNBOUNDED: CardDef = CardDef {
    id: "TIME_005",
    name: "Timethief Rafaam",
    card_type: CardType::Minion,
    cost: 10,
    attack: 10,
    health: 10,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_009 Gelbin of Tomorrow — 8-cost 6/6 (MINION, Mechanical). "Put one
/// of each Aura from your deck into the battlefield" — the Fabled aura
/// collection is simplified to summoning ONE random deck minion (§21; the
/// summon is battlecry-free — effect summons never run battlecries).
pub const GELBIN_OF_TOMORROW: CardDef = CardDef {
    id: "TIME_009",
    name: "Gelbin of Tomorrow",
    card_type: CardType::Minion,
    cost: 8,
    attack: 6,
    health: 6,
    durability: 0,
    battlecry: Some(CardEffect::SummonRandomMinionFromDeck),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Mechanical),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_013 Farseer Wo — 4-cost 2/6 (MINION, Elusive). "After you cast a
/// spell, Discover a Nature spell from the past" — the FriendlySpellCast
/// trigger surfaces the real three-option discover over the in-window
/// Nature-spell pool.
pub const FARSEER_WO: CardDef = CardDef {
    id: "TIME_013",
    name: "Farseer Wo",
    card_type: CardType::Minion,
    cost: 4,
    attack: 2,
    health: 6,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: true,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: Some(CardEffect::DiscoverPool {
        pool: DiscoverPool::NatureSpell,
    }),
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_020 Broxigar — 2-cost 12/12 (MINION, Charge). Fabled — "Start of
/// Game: Disappear. Kill all 4 Demons from Argus to reappear in hand" —
/// simplified to a plain 12/12 with Charge (§21).
pub const BROXIGAR_THE_UNBROKEN: CardDef = CardDef {
    id: "TIME_020",
    name: "Broxigar",
    card_type: CardType::Minion,
    cost: 2,
    attack: 12,
    health: 12,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: true,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_024 Murozond, Unbounded — 9-cost 8/8 (MINION, Dragon). "Battlecry:
/// At the start of your next turn, set this minion's Attack to INFINITY!" —
/// the battlecry arms the owner's player flag; the start-of-turn hook sets
/// the Attack to `INFINITY_ATTACK_CAP` (the finite engine stand-in, §21).
pub const MUROZOND_UNBOUNDED: CardDef = CardDef {
    id: "TIME_024",
    name: "Murozond, Unbounded",
    card_type: CardType::Minion,
    cost: 9,
    attack: 8,
    health: 8,
    durability: 0,
    battlecry: Some(CardEffect::MurozondPrepareInfiniteAttack),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_032 Chronogor — 6-cost 6/7 (MINION, Dragon). "Battlecry: You draw
/// your 2 highest Cost cards. Your opponent draws your 2 lowest Cost
/// cards." — both draws come from the OWNER's deck (the scan-draw
/// convention, no fatigue on a scan).
pub const CHRONOGOR: CardDef = CardDef {
    id: "TIME_032",
    name: "Chronogor",
    card_type: CardType::Minion,
    cost: 6,
    attack: 6,
    health: 7,
    durability: 0,
    battlecry: Some(CardEffect::ChronogorDrawsHighestLowest),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_042 King Maluk — 4-cost 5/6 (MINION, Beast). "Battlecry: Discard
/// your hand. Get an Infinite Banana." — the Banana (TIME_042t) is a plain
/// 1-Cost spell (the "Infinite" stays-in-hand keyword is not implemented,
/// §21).
pub const KING_MALUK: CardDef = CardDef {
    id: "TIME_042",
    name: "King Maluk",
    card_type: CardType::Minion,
    cost: 4,
    attack: 5,
    health: 6,
    durability: 0,
    battlecry: Some(CardEffect::DiscardHandAndAddInfiniteBanana),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Beast),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_064 Chrono-Lord Deios — 7-cost 4/8 (MINION). "Your Battlecries,
/// Deathrattles, Hero Power, and end of turn effects trigger twice." — an
/// `AuraEffect::DoubleTriggers` aura carried by the minion (silenceable);
/// the rules.rs `deios_doubling` hook re-resolves each pre-captured effect
/// once (§21). The AuraTarget is nominal — the doubling check reads the
/// aura component directly, not the stats buckets.
pub const DEIOS_THE_UNSTOPPABLE: CardDef = CardDef {
    id: "TIME_064",
    name: "Chrono-Lord Deios",
    card_type: CardType::Minion,
    cost: 7,
    attack: 4,
    health: 8,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: Some((AuraEffect::DoubleTriggers, AuraTarget::AllFriendlyMinions)),
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_103 Chromie — 6-cost 4/6 (MINION). "Deathrattle: Draw another copy
/// of cards you've played this game." — one copy of each of the last 10
/// distinct played cards (the official unbounded count is approximated by
/// MAX_REWIND_HISTORY, §21).
pub const CHROMIE: CardDef = CardDef {
    id: "TIME_103",
    name: "Chromie",
    card_type: CardType::Minion,
    cost: 6,
    attack: 4,
    health: 6,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::AddCopiesOfLastPlayedCards { count: 10 }),
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_209 Muradin, High King — 5-cost 3/2 (MINION). Fabled, Rush.
/// "Battlecry: Bring the High King's Hammer to ME! Deathrattle: Add it to
/// your hand." — the battlecry equips TIME_209t, the deathrattle adds it
/// back to hand. (Rush via the apply_card_keywords id-table; the Fabled
/// keyword itself is not implemented, §21.)
pub const MURADIN_HIGH_KING: CardDef = CardDef {
    id: "TIME_209",
    name: "Muradin, High King",
    card_type: CardType::Minion,
    cost: 5,
    attack: 3,
    health: 2,
    durability: 0,
    battlecry: Some(CardEffect::EquipWeapon {
        card_id: "TIME_209t",
    }),
    deathrattle: Some(CardEffect::AddCardToHand {
        card_id: "TIME_209t",
    }),
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_211 Lady Azshara — 5-cost 5/5 (MINION). Fabled. "Choose One —
/// Empower Zin-Azshari; or The Well of Eternity." — option 0 replaces the
/// location with TIME_211t2 Zin-Azshari, option 1 with TIME_211t1 The Well
/// of Eternity (the official "the other gets destroyed" is the location
/// replacement rule; §21).
pub const LADY_AZSHARA: CardDef = CardDef {
    id: "TIME_211",
    name: "Lady Azshara",
    card_type: CardType::Minion,
    cost: 5,
    attack: 5,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::SummonLocationForPlayer {
        card_id: "TIME_211t2",
    }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: Some(CardEffect::SummonLocationForPlayer {
        card_id: "TIME_211t1",
    }),
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_435 Eternus — 6-cost 6/2 (MINION, Dragon). "Battlecry: Take control
/// of an enemy minion with this minion's Health or less." — the threshold
/// is the source's effective Health at resolution (the unit
/// TakeControlEnemyMinionHealthLE reads it live; Mind Control precedent —
/// permanent control).
pub const ETERNUS: CardDef = CardDef {
    id: "TIME_435",
    name: "Eternus",
    card_type: CardType::Minion,
    cost: 6,
    attack: 6,
    health: 2,
    durability: 0,
    battlecry: Some(CardEffect::TakeControlEnemyMinionHealthLE),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_446 The Eternal Hold — 6-cost 0/0 (LOCATION, 3 durability).
/// "Discover any Demon that costs (5) or more. If your deck has no
/// minions, your next one costs (1)." — the activation surfaces the real
/// three-option discover over the full-catalog DemonCostGE5 pool; the
/// deck-no-minions check arms the one-time next-demon-cost-one flag
/// (fidelity-debt §21).
pub const THE_ETERNAL_HOLD: CardDef = CardDef {
    id: "TIME_446",
    name: "The Eternal Hold",
    card_type: CardType::Location,
    cost: 6,
    attack: 0,
    health: 0,
    durability: 3,
    battlecry: Some(CardEffect::DiscoverDemonGE5AndSetNextDemonCostOne),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_609 Ranger General Sylvanas — 3-cost 2/4 (MINION). Fabled.
/// "Battlecry: Deal 2 damage to all enemies. If you've played Alleria or
/// Vereesa, repeat for each." — the repeat count reads the all-time
/// played-minion log for the two sisters (TIME_609t1/t2); the Fabled
/// keyword is not implemented (§21).
pub const RANGER_GENERAL_SYLVANAS: CardDef = CardDef {
    id: "TIME_609",
    name: "Ranger General Sylvanas",
    card_type: CardType::Minion,
    cost: 3,
    attack: 2,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::SylvanasDealToAllEnemiesRepeated { damage: 2 }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_618 Husk, Eternal Reaper — 4-cost 5/3 (MINION, Undead). "Battlecry:
/// Give your hero 'Deathrattle: Spend up to 20 Corpses to resurrect with
/// that much Health'" — approximated as an immediate battlecry: spend up
/// to 20 Corpses to restore that much Health to the hero (the hero
/// deathrattle does not exist in the engine, §21).
pub const HUSK_ETERNAL_REAPER: CardDef = CardDef {
    id: "TIME_618",
    name: "Husk, Eternal Reaper",
    card_type: CardType::Minion,
    cost: 4,
    attack: 5,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::SpendCorpsesRestoreHeroHealth { max: 20 }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Undead),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_619 Talanji of the Graves — 4-cost 4/5 (MINION, Undead). Fabled.
/// "Battlecry: Draw Bwonsamdi (or resurrect him if he has died). Choose a
/// Boon to give him." — the three choose-one branches each run the
/// draw-or-resurrect (exactly once, whichever option) then grant the boon
/// keyword: Taunt (Boon of Power) / Lifesteal (Boon of Longevity) / Rush
/// (Boon of Speed, the cards-side third branch). The boons' "+2/+2 and
/// deathrattle-minions cost more" riders are simplified away (§21).
pub const TALANJI_OF_THE_GRAVES: CardDef = CardDef {
    id: "TIME_619",
    name: "Talanji of the Graves",
    card_type: CardType::Minion,
    cost: 4,
    attack: 4,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::DrawOrResurrectBwonsamdiAndGrantBoon {
        keyword: KeywordKind::Taunt,
    }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Undead),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: Some(CardEffect::DrawOrResurrectBwonsamdiAndGrantBoon {
        keyword: KeywordKind::Lifesteal,
    }),
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_705 Krona, Keeper of Eons — 6-cost 4/7 (MINION, Taunt). "Battlecry:
/// Set the Costs of the bottom 5 cards of your deck to (1)." — Set-to-1
/// cost modifiers on the last 5 deck entries (deck index 0 is the top).
pub const KRONA_KEEPER_OF_EONS: CardDef = CardDef {
    id: "TIME_705",
    name: "Krona, Keeper of Eons",
    card_type: CardType::Minion,
    cost: 6,
    attack: 4,
    health: 7,
    durability: 0,
    battlecry: Some(CardEffect::SetDeckBottomCostsOne { count: 5 }),
    deathrattle: None,
    taunt: true,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_706 The Fins Beyond Time — 2-cost 2/3 (MINION, Murloc). "Battlecry:
/// Replace your hand with your starting hand. Swap back at the end of your
/// turn." — §21: the starting hand is approximated by fresh draws — the
/// current hand is shuffled into the deck BOTTOM, the same number of cards
/// are drawn, and the end-of-turn `RestoreHandSnapshot` swaps the hand
/// back.
pub const THE_FINS_BEYOND_TIME: CardDef = CardDef {
    id: "TIME_706",
    name: "The Fins Beyond Time",
    card_type: CardType::Minion,
    cost: 2,
    attack: 2,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::ReplaceHandAndSwapBackAtTurnEnd),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Murloc),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: Some(CardEffect::RestoreHandSnapshot),
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_713 Time Adm'ral Hooktail — 5-cost 4/6 (MINION, Pirate). "Battlecry:
/// Summon a 0/8 Chest for your opponent. It's FULL of Coins!" — the
/// TIMELESS_CHEST token (TIME_713t) summons onto the opponent's board.
pub const TIME_ADMIRAL_HOOKTAIL: CardDef = CardDef {
    id: "TIME_713",
    name: "Time Adm'ral Hooktail",
    card_type: CardType::Minion,
    cost: 5,
    attack: 4,
    health: 6,
    durability: 0,
    battlecry: Some(CardEffect::SummonChestForOpponent),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Pirate),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_714 Chrono-Lord Epoch — 6-cost 7/5 (MINION, Dragon). "Battlecry:
/// Destroy all minions that your opponent played last turn." — the
/// opponent's per-turn play list (maintained by the CardPlayed/TurnEnded
/// hooks) matched against the current enemy board; deathrattles fire
/// normally.
pub const CHRONO_LORD_EPOCH: CardDef = CardDef {
    id: "TIME_714",
    name: "Chrono-Lord Epoch",
    card_type: CardType::Minion,
    cost: 6,
    attack: 7,
    health: 5,
    durability: 0,
    battlecry: Some(CardEffect::DestroyAllMinionsOpponentPlayedLastTurn),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_850 Lo'Gosh, Blood Fighter — 7-cost 7/7 (MINION). Fabled, Rush.
/// "Deathrattle: Summon a Blood Fighter from your hand. It gains +5/+5 and
/// attacks a random enemy." — the deathrattle consumes either Blood
/// Fighter in the hand (TIME_850t Broll / TIME_850t1 Valeera), summons a
/// fresh copy, buffs it +5/+5 permanently and forces the attack through
/// the AttackDeclared/ResolveAttack path. (Rush via the
/// apply_card_keywords id-table.)
pub const LO_GOSH_BLOOD_FIGHTER: CardDef = CardDef {
    id: "TIME_850",
    name: "Lo'Gosh, Blood Fighter",
    card_type: CardType::Minion,
    cost: 7,
    attack: 7,
    health: 7,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::SummonBloodFighterFromHandBuffAndAttack),
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_852 Azure Queen Sindragosa — 5-cost 2/8 (MINION, Dragon). Fabled.
/// "If you control another Dragon, your Arcane spells cost (2) less." —
/// an id-keyed play_cost arm (Spell + Arcane school + another controlled
/// Dragon, excluding this card's own discount — see `engine::cost`).
pub const SINDRAGOSA: CardDef = CardDef {
    id: "TIME_852",
    name: "Azure Queen Sindragosa",
    card_type: CardType::Minion,
    cost: 5,
    attack: 2,
    health: 8,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Dragon),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_861 Timelooper Toki — 3-cost 3/3 (MINION). "Battlecry: Get 3 random
/// spells from the past. When you play ALL 3, get another Timelooper
/// Toki." — the generated spell ids are tracked per player; the CardPlayed
/// hook removes a played id and adds a fresh TIME_861 when the list
/// empties (§21 — "from the past" is the active spell window).
pub const TIMELOOPER_TOKI: CardDef = CardDef {
    id: "TIME_861",
    name: "Timelooper Toki",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 3,
    durability: 0,
    battlecry: Some(CardEffect::GetThreeRandomSpellsFromPastTracked),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_875 Garona Halforcen — 4-cost 5/4 (MINION). Fabled. "Battlecry: If
/// your opponent is holding King Llane, destroy him and cut their Health
/// in half." — pool-open (reads the opponent's hand, POOL_OPEN_CARDS): the
/// held King Llane (TIME_875t) is destroyed and the enemy hero's max
/// Health is halved (rounding the max up, keeping the current damage;
/// §21).
pub const GARONA_HALFORCEN: CardDef = CardDef {
    id: "TIME_875",
    name: "Garona Halforcen",
    card_type: CardType::Minion,
    cost: 4,
    attack: 5,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::DestroyHeldKingLlaneAndHalveEnemyHealth),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_890 Medivh the Hallowed — 10-cost 7/7 (MINION). Fabled. "Costs (0)
/// if you control Karazhan. Battlecry: Silence and destroy all other
/// minions." — the cost arm lives in `engine::cost` (0 while TIME_890t2 is
/// the owner's location); the battlecry silences every other minion on
/// BOTH boards first (deathrattles removed — they must not fire) and then
/// destroys them through the normal death path.
pub const MEDIVH_THE_HALLOWED: CardDef = CardDef {
    id: "TIME_890",
    name: "Medivh the Hallowed",
    card_type: CardType::Minion,
    cost: 10,
    attack: 7,
    health: 7,
    durability: 0,
    battlecry: Some(CardEffect::SilenceAndDestroyAllOtherMinions),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

// ============================================================
// Tokens
// ============================================================

/// TIME_209t High King's Hammer — 6-cost 3/4 (WEAPON, Windfury). The
/// official "Deathrattle: Shuffle this into your deck with +2 Attack
/// permanently" is simplified away — Muradin's deathrattle already returns
/// the hammer to hand, and a shuffle-back would loop (§21).
pub const MURADINS_HAMMER: CardDef = CardDef {
    id: "TIME_209t",
    name: "High King's Hammer",
    card_type: CardType::Weapon,
    cost: 6,
    attack: 3,
    health: 0,
    durability: 4,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: true,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_211t1 The Well of Eternity — 4-cost 0/0 (LOCATION, 3 durability).
/// "Fill your hand with random Temporary spells." — the activation fills
/// the hand (cap 10) with in-window random spells, each marked Temporary.
pub const THE_WELL_OF_ETERNITY: CardDef = CardDef {
    id: "TIME_211t1",
    name: "The Well of Eternity",
    card_type: CardType::Location,
    cost: 4,
    attack: 0,
    health: 0,
    durability: 3,
    battlecry: Some(CardEffect::FillHandWithRandomTemporarySpells),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_211t2 Zin-Azshari — 4-cost 0/0 (LOCATION, 3 durability). "Summon a
/// copy of a friendly minion." — the activation copies a random friendly
/// minion (D2 pick; an empty board fizzles).
pub const ZIN_AZSHARI: CardDef = CardDef {
    id: "TIME_211t2",
    name: "Zin-Azshari",
    card_type: CardType::Location,
    cost: 4,
    attack: 0,
    health: 0,
    durability: 3,
    battlecry: Some(CardEffect::SummonCopyOfFriendlyMinion),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_619t Bwonsamdi — 6-cost 6/6 (MINION, Undead). "Deathrattle: Summon
/// a random 4-Cost minion." — the deathrattle (any boons granted by
/// Talanji ride on the entity; the "+2/+2" boon riders are simplified
/// away, §21).
pub const BWONSAMDI: CardDef = CardDef {
    id: "TIME_619t",
    name: "Bwonsamdi",
    card_type: CardType::Minion,
    cost: 6,
    attack: 6,
    health: 6,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::SummonRandomCostMinion { cost: 4 }),
    taunt: false,
    stealth: false,
    elusive: false,
    race: Some(Race::Undead),
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_042t Infinite Banana — 1-cost SPELL. "Give a minion +1/+1." — the
/// "Infinite" (stays in hand) keyword is not implemented — the Banana is a
/// plain 1-Cost spell (§21).
pub const INFINITE_BANANA: CardDef = CardDef {
    id: "TIME_042t",
    name: "Infinite Banana",
    card_type: CardType::Spell,
    cost: 1,
    attack: 0,
    health: 0,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: Some(CardEffect::GainStats {
        attack: 1,
        health: 1,
        target: EffectTarget::AnyMinion,
    }),
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_713t Timeless Chest — 3-cost 0/8 (MINION). "Deathrattle: Fill your
/// opponent's hand with Coins." — the hand (cap 10) fills with THE_COIN
/// (GAME_005).
pub const TIMELESS_CHEST: CardDef = CardDef {
    id: "TIME_713t",
    name: "Timeless Chest",
    card_type: CardType::Minion,
    cost: 3,
    attack: 0,
    health: 8,
    durability: 0,
    battlecry: None,
    deathrattle: Some(CardEffect::FillOpponentHandWithCoins),
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_875t King Llane — 3-cost 3/3 (MINION). "Start of Game: Hide from
/// Garona in the enemy's deck. Battlecry: Draw a card. Shuffle this back
/// into your deck." — Garona's trigger searches the opponent's HAND for
/// the official hide-out (the deck-hide and self-shuffle battlecry are
/// simplified to a plain 3/3, §21).
pub const KING_LLANE: CardDef = CardDef {
    id: "TIME_875t",
    name: "King Llane",
    card_type: CardType::Minion,
    cost: 3,
    attack: 3,
    health: 3,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_890t2 Karazhan the Sanctum — 10-cost 0/0 (LOCATION, 2 durability).
/// "Costs (0) if you're wielding Atiesh. Summon two random 8-Cost
/// minions." — the Atiesh rider is simplified away (Atiesh itself is
/// skipped, §21); the activation summons two random 8-Cost minions (the
/// fixed 8/8 summon — the SummonTwoRandomCostMinions upgrade check reads
/// spells-cast and passes 8 for both).
pub const KARAZHAN_THE_SANCTUM: CardDef = CardDef {
    id: "TIME_890t2",
    name: "Karazhan the Sanctum",
    card_type: CardType::Location,
    cost: 10,
    attack: 0,
    health: 0,
    durability: 2,
    battlecry: Some(CardEffect::SummonTwoRandomCostMinions {
        base_cost: 8,
        upgraded_cost: 8,
    }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_609t1 Ranger Captain Alleria — 3-cost 2/4 (MINION). "Battlecry:
/// Discover a spell. If you've played Sylvanas or Vereesa, repeat for
/// each." — the discover is real (the in-window spell pool); the repeat
/// rider is approximated by Ranger General Sylvanas' played-log count and
/// does not repeat here (§21).
pub const RANGER_CAPTAIN_ALLERIA: CardDef = CardDef {
    id: "TIME_609t1",
    name: "Ranger Captain Alleria",
    card_type: CardType::Minion,
    cost: 3,
    attack: 2,
    health: 4,
    durability: 0,
    battlecry: Some(CardEffect::DiscoverPool {
        pool: DiscoverPool::Spell,
    }),
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_609t2 Ranger Initiate Vereesa — 3-cost 2/4 (MINION). "Battlecry:
/// Give minions in your deck +1/+1. If you've played Alleria or Sylvanas,
/// repeat for each." — the deck-buff has no engine variant: Vereesa is a
/// plain 2/4 whose play still counts in Sylvanas' repeat log (§21).
pub const RANGER_INITIATE_VEREESA: CardDef = CardDef {
    id: "TIME_609t2",
    name: "Ranger Initiate Vereesa",
    card_type: CardType::Minion,
    cost: 3,
    attack: 2,
    health: 4,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_850t Broll, Blood Fighter — 7-cost 7/7 (MINION, Taunt). The other
/// Blood Fighter form; summoned and buffed by TIME_850's deathrattle. The
/// token's own deathrattle chain is simplified away (§21 — only Lo'Gosh
/// triggers the summon-buff-attack chain).
pub const BROLL_BLOOD_FIGHTER: CardDef = CardDef {
    id: "TIME_850t",
    name: "Broll, Blood Fighter",
    card_type: CardType::Minion,
    cost: 7,
    attack: 7,
    health: 7,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: true,
    stealth: false,
    elusive: false,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};

/// TIME_850t1 Valeera, Blood Fighter — 7-cost 7/7 (MINION, Elusive). The
/// other Blood Fighter form; summoned and buffed by TIME_850's deathrattle
/// (the token's own deathrattle chain simplified away, §21).
pub const VALEERA_BLOOD_FIGHTER: CardDef = CardDef {
    id: "TIME_850t1",
    name: "Valeera, Blood Fighter",
    card_type: CardType::Minion,
    cost: 7,
    attack: 7,
    health: 7,
    durability: 0,
    battlecry: None,
    deathrattle: None,
    taunt: false,
    stealth: false,
    elusive: true,
    race: None,
    hero_power: None,
    aura: None,
    secret: None,
    divine_shield: false,
    windfury: false,
    charge: false,
    spell_damage: 0,
    cant_attack: false,
    end_turn_effect: None,
    start_turn_effect: None,
    spell_effect: None,
    spell_trigger: None,
    death_trigger: None,
    summon_trigger: None,
    choose_one_effect: None,
    combo_effect: None,
    attack_equals_health: false,
};
