//! Rule engine — validation, event enqueueing, event application.
//!
//! Three core functions:
//! - `validate()` — read-only check of an action's legality in the current state
//! - `enqueue()` — converts an action into initial events and enqueues them
//! - `apply_event()` — applies a single event, possibly enqueueing new events
//!
//! All functions are pure-functional in style, interacting with state via a `GameState` parameter.

use crate::core::action::Action;
use crate::core::component::{
    Attack, AttacksUsed, CardType, Cost, DarkGiftKind, Durability, EnchantmentExpiry,
    HandTurnCounter, Health, HeroPowerUsed, Secret, TriggerEvent, TriggerTiming,
};
use crate::core::entity::Entity;
use crate::core::event::{Event, EventQueue, Priority};
use crate::core::player::PlayerId;
use crate::core::small_list::SmallList;
use crate::core::state::{ChoiceKind, GameState, Step};
use crate::core::zone::Zone;
use crate::engine::secret;
use crate::engine::trigger::{self, MAX_HAND_SIZE};

/// Engine error — why an action could not be executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineError {
    /// Not your turn
    NotYourTurn,
    /// Trying to play the opponent's card
    NotYourCard,
    /// Card is not in hand
    CardNotInHand,
    /// Not a minion or weapon (PlayCard can only play minions or weapons)
    NotPlayable,
    /// Board is full (7 minion limit)
    BoardFull,
    /// Invalid target (attacking own side or target does not exist)
    InvalidTarget,
    /// Attacker is not on the board
    NotOnBoard,
    /// Attack count exhausted this turn
    AttacksExhausted,
    /// Entity has been destroyed (stale handle)
    EntityGone(Entity),
    /// Game is already over
    GameAlreadyOver,
    /// Not enough mana
    NotEnoughMana,
    /// Must attack a taunt minion first
    MustAttackTaunt,
    /// Hero power already used this turn
    HeroPowerAlreadyUsed,
    /// No choice is pending (roadmap G6)
    NoPendingChoice,
    /// The choice id does not match the pending choice, or the option is out of range
    InvalidChoice,
    /// A choice is pending — only `Action::Choose` is legal until it resolves
    /// (2025–2026 expansions M1-W3, P3 real choice resolution)
    ChoicePending,
    /// Feature not yet implemented (Phase 2+)
    Unimplemented,
}

/// Maximum number of minions on the battlefield.
pub const MAX_BOARD_SIZE: usize = 7;

/// Branch labels for a card entity's pending Choose One choice
/// (2025–2026 expansions M1-W3, P3): resolves the entity's card definition
/// and asks the cards-side `choose_one_option_names` table; falls back to
/// the generic labels for unknown/off-chain cards.
fn choose_one_labels(state: &GameState, card: Entity) -> [&'static str; 2] {
    state
        .world()
        .card_id(card)
        .and_then(|c| crate::cards::def::card_by_id(c.0))
        .map(crate::cards::choose_one_option_names)
        .unwrap_or(["First option", "Second option"])
}

/// The full branch label list for a choose-one card — 2 options for the
/// standard cards, 3 for the M2-W4a trio (Ancient Stegodon / Ancient
/// Raptor / Ancient Pterrordax, whose third branch lives in the
/// cards-side `choose_one_three_branch` table).
fn choose_one_labels_all(state: &GameState, card: Entity) -> Vec<&'static str> {
    let mut labels: Vec<&'static str> = choose_one_labels(state, card).to_vec();
    if let Some(third) = state
        .world()
        .card_id(card)
        .and_then(|c| crate::cards::def::card_by_id(c.0))
        .and_then(crate::cards::choose_one_three_option_names)
    {
        labels.push(third);
    }
    labels
}

/// How many times a choose-one choice surfaces for the given card —
/// 1 for every card except Forest Lord Cenarius (EDR_209, "Choose Thrice",
/// M1-W4b: the two options may be picked up to three times in total).
fn choose_one_repeat(state: &GameState, card: Entity) -> u8 {
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "EDR_209")
    {
        3
    } else {
        1
    }
}

/// Validates an action's legality in the current state (read-only).
///
/// Returns `Ok(())` or `Err(EngineError)`.
pub fn validate(state: &GameState, action: Action) -> Result<(), EngineError> {
    // Reject all actions once the game is over
    if matches!(state.step(), Step::GameOver { .. }) {
        return Err(EngineError::GameAlreadyOver);
    }
    // While a choice (Choose One / Discover / Mulligan) is pending, the only
    // legal action is resolving it (2025–2026 expansions M1-W3, P3). The
    // default policy never trips this gate: `GameEngine::apply` re-applies
    // only the `Action::Choose` it synthesized, never the original action.
    if state.pending_choice().is_some() && !matches!(action, Action::Choose { .. }) {
        return Err(EngineError::ChoicePending);
    }

    match action {
        Action::PlayCard {
            card,
            target: _,
            position: _,
        } => validate_play_card(state, card),
        Action::Attack { attacker, defender } => validate_attack(state, attacker, defender),
        Action::EndTurn => validate_end_turn(state),
        Action::HeroPower { hero, target: _ } => validate_hero_power(state, hero),
        Action::ActivateLocation { location, .. } => validate_activate_location(state, location),
        Action::Choose { choice_id, option } => validate_choose(state, choice_id, option),
        Action::TradeCard { card } => validate_trade_card(state, card),
        Action::Prepare { card } => validate_prepare(state, card),
    }
}

/// Validates a Prepare action (M5-W1 — Escape from Violet Hold, the
/// Prepare keyword of JAIL_407/721/906, pinned §27): the card must be a
/// Prepare card in the active player's hand, the player must not have
/// already prepared this turn, the card must not have been prepared
/// before, and the player must hold at least 1 mana to spend.
fn validate_prepare(state: &GameState, card: Entity) -> Result<(), EngineError> {
    let world = state.world();
    let active = state.active_player();
    check_entity(world, card)?;
    let card_player = world.player(card).ok_or(EngineError::EntityGone(card))?;
    if card_player != active {
        return Err(EngineError::NotYourCard);
    }
    if world.zone(card) != Some(Zone::Hand) {
        return Err(EngineError::CardNotInHand);
    }
    if !crate::cards::prepare::is_prepare_card(state, card) {
        return Err(EngineError::InvalidTarget);
    }
    // Once per turn, once per card (the official constraints).
    if state.player(active).prepare_used_this_turn {
        return Err(EngineError::InvalidTarget);
    }
    if state.player(active).prepared_cards.contains(&card) {
        return Err(EngineError::InvalidTarget);
    }
    // Cannot prepare with 0 mana.
    if state.player(active).current_mana < 1 {
        return Err(EngineError::NotEnoughMana);
    }
    Ok(())
}

/// Validates a location activation (Core Set W8): the entity must be a
/// friendly Location on the board, past its play cooldown, not yet
/// activated this turn, and still holding durability charges.
fn validate_activate_location(state: &GameState, location: Entity) -> Result<(), EngineError> {
    let world = state.world();
    let active = state.active_player();
    check_entity(world, location)?;
    if world.card_type(location) != Some(CardType::Location) {
        return Err(EngineError::InvalidTarget);
    }
    let owner = world
        .player(location)
        .ok_or(EngineError::EntityGone(location))?;
    if owner != active {
        return Err(EngineError::NotYourTurn);
    }
    if state.player(active).location != Some(location) {
        return Err(EngineError::InvalidTarget);
    }
    // Cooldown: a location cannot be activated the turn it was played
    if state.player(active).location_played_turn >= state.turn() {
        return Err(EngineError::InvalidTarget);
    }
    // One activation per turn (attacks_used doubles as the used marker —
    // it resets for all Play entities at the owner's turn start)
    if world.attacks_used(location).is_some_and(|u| u.0 > 0) {
        return Err(EngineError::InvalidTarget);
    }
    if world.durability(location).is_none_or(|d| d.0 == 0) {
        return Err(EngineError::InvalidTarget);
    }
    Ok(())
}

/// Validates a Tradeable trade (Core Set W2): the card must be a Tradeable
/// hand card and the player must afford the 1-mana trade.
fn validate_trade_card(state: &GameState, card: Entity) -> Result<(), EngineError> {
    let world = state.world();
    let active = state.active_player();
    check_entity(world, card)?;
    let card_player = world.player(card).ok_or(EngineError::EntityGone(card))?;
    if card_player != active {
        return Err(EngineError::NotYourCard);
    }
    if world.zone(card) != Some(Zone::Hand) {
        return Err(EngineError::CardNotInHand);
    }
    if world.tradeable(card).is_none() {
        return Err(EngineError::InvalidTarget);
    }
    if state.player(active).current_mana < 1 {
        return Err(EngineError::NotEnoughMana);
    }
    Ok(())
}

/// Validates a choice resolution (roadmap G6): a choice must be pending and
/// the id/option must match.
fn validate_choose(state: &GameState, choice_id: u64, option: u8) -> Result<(), EngineError> {
    let pending = state.pending_choice().ok_or(EngineError::NoPendingChoice)?;
    if pending.id != choice_id {
        return Err(EngineError::InvalidChoice);
    }
    if option as usize >= pending.options.len() {
        return Err(EngineError::InvalidChoice);
    }
    Ok(())
}

/// Validates a play-card action.
fn validate_play_card(state: &GameState, card: Entity) -> Result<(), EngineError> {
    let world = state.world();
    let active = state.active_player();

    // Check entity liveness and component presence
    check_entity(world, card)?;

    let card_player = world.player(card).ok_or(EngineError::EntityGone(card))?;
    if card_player != active {
        return Err(EngineError::NotYourCard);
    }

    // Must be a minion, weapon, spell, hero (Core Set W8), or location
    // (Core Set W8). ENCHANTMENT tokens are never playable.
    let card_type = world.card_type(card).ok_or(EngineError::NotPlayable)?;
    if card_type != CardType::Minion
        && card_type != CardType::Weapon
        && card_type != CardType::Spell
        && card_type != CardType::Hero
        && card_type != CardType::Location
    {
        return Err(EngineError::NotPlayable);
    }

    // Must be in hand
    let zone = world.zone(card).ok_or(EngineError::CardNotInHand)?;
    if zone != Zone::Hand {
        return Err(EngineError::CardNotInHand);
    }

    // CantPlayNextTurn (2025–2026 expansions M4-W4 — CATA_158 Windwalker's
    // "It can't be played next turn"): the marker survives the zone move
    // home and blocks the play until the owner's next turn start clears it.
    if world.cant_play_next_turn(card).is_some() {
        return Err(EngineError::InvalidTarget);
    }

    // LockedUntilCardPlayed (M5-W2 — JAIL_987 Low Security Wing: "It can't
    // be played until you play a card"): the marker is cleared by the
    // CardPlayed handler, so the lock only ever blocks the first play
    // after the buff landed.
    if world.locked_until_card_played(card).is_some() {
        return Err(EngineError::InvalidTarget);
    }

    // Check mana (single cost composition — roadmap G5)
    let cost = crate::engine::cost::play_cost(state, card, active);
    if cost.0 > state.player(active).current_mana {
        return Err(EngineError::NotEnoughMana);
    }

    // Reanimated Pterrordax (M2-W4a): "Costs Corpses instead of Mana" — the
    // play needs 5 Corpses (the printed Cost), which are spent at play.
    if world.card_id(card).is_some_and(|c| c.0 == "TLC_436") && state.player(active).corpses < 5 {
        return Err(EngineError::NotEnoughMana);
    }

    // Check board size limit (weapons do not occupy a minion slot)
    if card_type == CardType::Minion {
        let board_count = count_board_minions(world, active);
        if board_count >= MAX_BOARD_SIZE {
            return Err(EngineError::BoardFull);
        }
    }

    Ok(())
}

/// Validates an attack action.
fn validate_attack(
    state: &GameState,
    attacker: Entity,
    defender: Entity,
) -> Result<(), EngineError> {
    let world = state.world();
    let active = state.active_player();

    // Check phase
    if state.step() != Step::Main {
        return Err(EngineError::InvalidTarget);
    }

    // Check entity liveness
    check_entity(world, attacker)?;
    check_entity(world, defender)?;

    // Attacker must be a friendly character (minion or hero) on the board
    let attacker_player = world
        .player(attacker)
        .ok_or(EngineError::EntityGone(attacker))?;
    if attacker_player != active {
        return Err(EngineError::InvalidTarget);
    }
    let attacker_type = world.card_type(attacker).ok_or(EngineError::NotOnBoard)?;
    match attacker_type {
        CardType::Minion => {}
        CardType::Hero => {
            // Hero attacks require a weapon or a temporary attack enchantment
            // (Heroic Strike-style buffs make the hero able to attack)
            let has_weapon = state.player(attacker_player).weapon.is_some();
            let has_attack = state
                .world()
                .effective_attack(attacker)
                .is_some_and(|a| a.0 > 0);
            if !has_weapon && !has_attack {
                return Err(EngineError::InvalidTarget);
            }
        }
        _ => return Err(EngineError::NotOnBoard),
    }
    let attacker_zone = world.zone(attacker).ok_or(EngineError::NotOnBoard)?;
    if attacker_zone != Zone::Play {
        return Err(EngineError::NotOnBoard);
    }

    // Freeze (engine-mechanics roadmap M2): a frozen character cannot attack.
    // With the thaw moved to the turn-end wrap-up the freeze is actually
    // present during the owner's next turn, so legal actions must exclude it.
    if world.freeze(attacker).is_some() {
        return Err(EngineError::InvalidTarget);
    }

    // Check attack count (accounting for windfury)
    let max_atks = world.max_attacks(attacker);
    if world
        .attacks_used(attacker)
        .is_some_and(|a| a.is_exhausted_with(max_atks))
    {
        return Err(EngineError::AttacksExhausted);
    }

    // Minions that cannot attack (e.g. Ragnaros)
    if world.cant_attack(attacker).is_some() {
        return Err(EngineError::InvalidTarget);
    }

    // CantAttackThisTurn (2025–2026 expansions M4-W4 — CATA_496 Cursed
    // Chains: "It can't attack this turn"): the temporary restriction
    // blocks attacks until the turn-end wrap-up clears it.
    if world.cant_attack_this_turn(attacker).is_some() {
        return Err(EngineError::InvalidTarget);
    }

    // Dormant minions cannot attack (M3-W2a).
    if world.dormant(attacker).is_some() {
        return Err(EngineError::InvalidTarget);
    }

    // Attack must be > 0 (considering weapon and auras)
    let total_atk = compute_attacker_damage(state, attacker);
    if total_atk <= 0 {
        return Err(EngineError::InvalidTarget);
    }

    // Defender must be an enemy target (own hero or own minion is not allowed)
    let defender_player = world.player(defender).ok_or(EngineError::InvalidTarget)?;
    if defender_player == active {
        return Err(EngineError::InvalidTarget);
    }

    // Defender must be a minion or hero on the battlefield
    let defender_type = world
        .card_type(defender)
        .ok_or(EngineError::InvalidTarget)?;
    let defender_zone = world.zone(defender).ok_or(EngineError::InvalidTarget)?;
    if defender_zone != Zone::Play {
        return Err(EngineError::InvalidTarget);
    }
    match defender_type {
        CardType::Minion | CardType::Hero => {}
        _ => return Err(EngineError::InvalidTarget),
    }

    // Rush (Core Set W1): a minion with Rush cannot attack the enemy HERO
    // on the turn it was summoned (it may attack enemy minions). The
    // SummonedThisTurn marker is cleared at the start of the next turn.
    if defender_type == CardType::Hero
        && attacker_type == CardType::Minion
        && world.rush(attacker).is_some()
        && world.summoned_this_turn(attacker).is_some()
    {
        return Err(EngineError::InvalidTarget);
    }

    // M3-W3 — Hand of Infinity (END_012): "Can't attack heroes" — the
    // equipped hero wielding the weapon may attack minions only. NOT the
    // `cant_attack` component (that blocks ALL attacks); the restriction
    // is an ID-keyed check at validation, the same shape as the
    // Rush-on-hero check (§22).
    if defender_type == CardType::Hero
        && attacker_type == CardType::Hero
        && state
            .player(attacker_player)
            .weapon
            .is_some_and(|w| world.card_id(w).is_some_and(|c| c.0 == "END_012"))
    {
        return Err(EngineError::InvalidTarget);
    }

    // Taunt check: if the enemy board has taunt minions, a taunt must be
    // attacked — unless a friendly Kayn Sunfury is on the board (Core Set
    // W5: friendly attacks ignore Taunt). M4-W4 — the effective-Taunt
    // helper folds in Taunt auras (CATA_556 Carrier Whelp's
    // other-Dragons-Taunt).
    let enemy = active.opponent();
    let kayn_sunfury = world
        .zones()
        .iter(Zone::Play, active)
        .any(|e| world.card_id(e).is_some_and(|c| c.0 == "CORE_BT_187"));
    let has_taunt = !kayn_sunfury
        && world
            .zones()
            .iter(Zone::Play, enemy)
            .any(|e| world.effective_taunt(e) && world.dormant(e).is_none());
    if has_taunt {
        // defender must be a taunt minion
        if !world.effective_taunt(defender) {
            return Err(EngineError::MustAttackTaunt);
        }
    }

    // Stealth / Dormant check: cannot attack enemy stealthed or dormant
    // characters (M3-W2a — dormant is untargetable like Stealth)
    if (world.stealth(defender).is_some() || world.dormant(defender).is_some())
        && defender_player != active
    {
        return Err(EngineError::InvalidTarget);
    }

    Ok(())
}

/// Validates an end-turn action.
fn validate_end_turn(state: &GameState) -> Result<(), EngineError> {
    if state.step() != Step::Main {
        return Err(EngineError::NotYourTurn);
    }
    Ok(())
}

/// Validates a hero power use.
fn validate_hero_power(state: &GameState, hero: Entity) -> Result<(), EngineError> {
    let world = state.world();
    let active = state.active_player();

    // Check entity liveness
    check_entity(world, hero)?;

    // Must be a hero
    if world.card_type(hero) != Some(CardType::Hero) {
        return Err(EngineError::InvalidTarget);
    }

    // Must be own hero
    let hero_player = world.player(hero).ok_or(EngineError::EntityGone(hero))?;
    if hero_player != active {
        return Err(EngineError::NotYourTurn);
    }

    // Phase check
    if state.step() != Step::Main {
        return Err(EngineError::NotYourTurn);
    }

    // Check whether already used this turn
    if world.hero_power_used(hero).is_some_and(|u| u.0) {
        return Err(EngineError::HeroPowerAlreadyUsed);
    }

    // Check mana (Dreambound Disciple — M1-W4a: the next Hero Power costs
    // (0), so the affordability check is skipped when the flag is set;
    // M4-W4 — CATA_615t Genn's upgraded power costs (1), §26)
    let hero_power = world.hero_power(hero);
    let cost = if state.player(active).hero_power_cost_1 {
        1
    } else {
        hero_power.map(|hp| hp.cost).unwrap_or(2)
    };
    // M3-W2a (TIME_606 Quel'dorei Fletcher): "Your Hero Power costs (0)
    // while your hand has 3 or less cards" — the affordability check is
    // skipped when a friendly Quel'dorei is on the board and the hand is
    // small enough (the cost deduction itself happens in the HeroPowerUse
    // handler, mirroring the cost-pipeline entries).
    let fletcher_free = world
        .zones()
        .iter(Zone::Play, active)
        .any(|e| world.card_id(e).is_some_and(|c| c.0 == "TIME_606"))
        && world.zones().iter(Zone::Hand, active).count() <= 3;
    if !state.player(active).next_hero_power_free
        && !fletcher_free
        && cost > state.player(active).current_mana
    {
        return Err(EngineError::NotEnoughMana);
    }

    // M5-W1 — Blood Doctor Thal'ena (JAIL_446, §27): the swapped-in
    // Vampyr's Kiss hero power costs 3 Corpses instead of Mana — the
    // affordability check reads the Corpses counter (spent at the
    // HeroPowerActivated handler).
    if state.player(active).thalena_corpses_hero_power
        && !state.player(active).next_hero_power_free
        && state.player(active).corpses < 3
    {
        return Err(EngineError::NotEnoughMana);
    }

    Ok(())
}

/// M3-W2b — TIME_064 Deios, the Unstoppable Force: "Your Battlecries
/// trigger twice" as an `AuraEffect::DoubleTriggers` aura carried in the
/// minion's Aura component (silenceable — `silence_entity` drops the aura
/// with the rest). Returns whether the player controls an active Deios;
/// the call sites re-resolve the pre-captured effect exactly once
/// (fidelity-debt §21: one re-resolve, never four under a stacked
/// BattlecryTwice dark gift).
pub(crate) fn deios_doubling(state: &GameState, player: PlayerId) -> bool {
    state.world().zones().iter(Zone::Play, player).any(|e| {
        state.world().card_id(e).is_some_and(|c| c.0 == "TIME_064")
            && state
                .world()
                .aura(e)
                .is_some_and(|a| a.effect == crate::core::component::AuraEffect::DoubleTriggers)
    })
}

/// Computes the attacker's total damage (base attack + auras + weapon bonus).
/// `pub(crate)` — the forced-attack effect (Mythical Terror, Core Set W1)
/// enqueues attacks from the trigger resolver.
pub(crate) fn compute_attacker_damage(state: &GameState, attacker: Entity) -> i32 {
    let world = state.world();
    let mut base = world.effective_attack(attacker).unwrap_or(Attack(0));
    // Tar Tyrant (M2-W4a): "Has +6 Attack during your opponent's turn" —
    // the minion attacks on the opponent's turn, i.e. whenever the attacker
    // is not the active player's minion.
    if world.card_id(attacker).is_some_and(|c| c.0 == "TLC_605")
        && world
            .player(attacker)
            .is_some_and(|pid| pid != state.active_player())
    {
        return base.0 + 6;
    }
    // Bladed Gauntlet (Core Set W3c): the weapon's Attack equals the
    // owner's armor (the effective_attack base is 0)
    let hero_has_gauntlet = world.card_type(attacker) == Some(CardType::Hero)
        && world.player(attacker).is_some_and(|pid| {
            state
                .player(pid)
                .weapon
                .is_some_and(|w| world.card_id(w).is_some_and(|c| c.0 == "CORE_LOOT_044"))
        });
    if hero_has_gauntlet {
        if let Some(pid) = world.player(attacker) {
            base = Attack(state.player(pid).armor);
        }
    }
    // Small-Time Buccaneer (Core Set W3c): +2 Attack while the owner has a
    // weapon equipped
    if world
        .card_id(attacker)
        .is_some_and(|c| c.0 == "CORE_WON_351")
        && world
            .player(attacker)
            .is_some_and(|pid| state.player(pid).weapon.is_some())
    {
        base = Attack(base.0 + 2);
    }
    // `effective_attack` rather than the raw component so weapon enchantments
    // and Spiteful Smith's Enrage (+2 Attack to your weapon while damaged)
    // reach the hero's swing. Auras never apply to weapons.
    let weapon_bonus = if world.card_type(attacker) == Some(CardType::Hero) {
        world
            .player(attacker)
            .and_then(|pid| state.player(pid).weapon)
            .and_then(|w| world.effective_attack(w))
            .unwrap_or(Attack(0))
            .0
    } else {
        0
    };
    base.0 + weapon_bonus
}

/// Checks whether an entity is alive, returning `EntityGone` otherwise.
fn check_entity(world: &crate::core::world::World, entity: Entity) -> Result<(), EngineError> {
    if world.is_alive(entity) {
        Ok(())
    } else {
        Err(EngineError::EntityGone(entity))
    }
}

/// Counts the minions on a player's battlefield.
fn count_board_minions(world: &crate::core::world::World, player: PlayerId) -> usize {
    world
        .zones()
        .iter(Zone::Play, player)
        .filter(|&e| world.card_type(e) == Some(CardType::Minion))
        .count()
}

// ============================================================
// Event enqueueing
// ============================================================

/// Generates initial events from an action and enqueues them (read-only).
pub fn enqueue(
    state: &GameState,
    action: Action,
    queue: &mut EventQueue,
) -> Result<(), EngineError> {
    match action {
        Action::PlayCard {
            card,
            target,
            position,
        } => {
            let player = state.active_player();
            queue.push(Event::CardPlayed {
                player,
                card,
                target,
                position,
            });
            let card_type = state.world().card_type(card);
            if card_type == Some(CardType::Minion) {
                queue.push(Event::MinionSummoned {
                    player,
                    minion: card,
                    target,
                });
            }
        }
        Action::Attack { attacker, defender } => {
            let world = state.world();
            queue.push(Event::AttackDeclared { attacker, defender });
            // The attack resolution is enqueued as a single pipeline event:
            // the damage value is computed when `ResolveAttack` is processed,
            // but the attacker's damage must be fixed at enqueue time — the
            // weapon may be destroyed in `AttackDeclared`, so the attack
            // damage must include the weapon bonus.
            let attacker_total_atk = compute_attacker_damage(state, attacker);
            // Retaliation immunity (Gladiator's Longbow: the hero is immune while attacking, no retaliation)
            let retaliation_immune = world.card_type(attacker) == Some(CardType::Hero)
                && world.player(attacker).is_some_and(|pid| {
                    state.player(pid).weapon.is_some_and(|w| {
                        world.card_id(w).is_some_and(|c| {
                            c.0 == crate::cards::classic_hunter::GLADIATORS_LONGBOW_ID
                        })
                    })
                });
            queue.push(Event::ResolveAttack {
                attacker,
                defender,
                attacker_damage: attacker_total_atk,
                retaliation_immune,
            });
        }
        Action::EndTurn => {
            // Only the TurnEnded event is enqueued here: the step machine runs
            // the end sequence (EndTriggers → WrapUp) and enqueues the next
            // player's TurnStarted itself at the wrap-up boundary.
            let active = state.active_player();
            queue.push(Event::TurnEnded { player: active });
        }
        Action::TradeCard { card } => {
            // Tradeable (Core Set W2): the state changes run in the
            // event handler (`TradeCardExecuted`); enqueue is read-only.
            queue.push(Event::TradeCardExecuted { card });
        }
        Action::Prepare { card } => {
            // Prepare (M5-W1 — Escape from Violet Hold): the state changes
            // run in the event handler (`PrepareCardExecuted`); enqueue is
            // read-only, mirroring the TradeCard arm.
            queue.push(Event::PrepareCardExecuted { card });
        }
        Action::HeroPower { hero, target } => {
            let active = state.active_player();
            queue.push(Event::HeroPowerActivated {
                player: active,
                hero,
                target,
            });
        }
        Action::ActivateLocation { location, target } => {
            let active = state.active_player();
            queue.push(Event::LocationActivated {
                player: active,
                location,
                target,
            });
        }
        Action::Choose { choice_id, option } => {
            queue.push(Event::ChoiceResolved { choice_id, option });
        }
    }
    Ok(())
}

// ============================================================
// Event application (mutating state)
// ============================================================

/// Lifesteal (Core Set W1) — a Lifesteal damage source heals its owner's
/// hero for the damage dealt. Only characters with damage accumulate a heal
/// (overhealing triggers nothing, matching the heal pipeline); a healed hero
/// fires `CharacterHealed` triggers (Lightwarden sees lifesteal heals).
fn resolve_lifesteal_heal(
    state: &mut GameState,
    queue: &mut EventQueue,
    source: Entity,
    amount: i32,
) {
    // The source itself carries Lifesteal (minion, spell entity, weapon-hit
    // source), or the source is a hero whose equipped weapon has it
    // (Aldrachi Warblades — hero attacks heal; Core Set W1).
    let source_heals = state.world().lifesteal(source).is_some()
        || (state.world().card_type(source) == Some(CardType::Hero)
            && state
                .world()
                .player(source)
                .and_then(|p| state.player(p).weapon)
                .is_some_and(|w| state.world().lifesteal(w).is_some()));
    if !source_heals {
        return;
    }
    let Some(owner) = state.world().player(source) else {
        return;
    };
    let hero = state.player(owner).hero;
    let dmg = state
        .world()
        .damage(hero)
        .unwrap_or(crate::core::component::Damage(0))
        .0;
    if dmg <= 0 {
        return;
    }
    let new_dmg = (dmg - amount).max(0);
    if new_dmg > 0 {
        state
            .world_mut()
            .set_damage(hero, crate::core::component::Damage(new_dmg));
    } else {
        state.world_mut().remove_damage(hero);
    }
    fire_triggers(
        state,
        queue,
        TriggerEvent::CharacterHealed,
        owner,
        Some(hero),
        None,
    );
}

/// Death check — the last step of the damage pipeline (`DamageDealt`).
///
/// When the target's effective health is ≤ 0: heroes enqueue a game-over
/// (highest priority); minions are marked pending death (roadmap G3) — they
/// stay on the battlefield until the death step processes them, so healing can
/// rescue them before their death resolves.
fn queue_death_events(
    state: &mut GameState,
    queue: &mut EventQueue,
    target: Entity,
    card_type: Option<CardType>,
) {
    let effective_hp = state.world().effective_health(target);
    if !effective_hp.is_some_and(|h| h.is_dead()) {
        return;
    }
    match card_type {
        Some(CardType::Hero) => {
            let loser = state.world().player(target);
            if let Some(player) = loser {
                let winner = player.opponent();
                queue.push_with_priority(Event::GameOver { winner }, Priority::Highest);
            }
        }
        Some(CardType::Minion) => {
            let inner = state.make_mut();
            inner.pending_deaths.push(target);
            // M4-W1 Colossal (2025–2026 expansions): a dying Colossal
            // main takes its attached body parts with it — they join this
            // same death batch, so their deathrattles fire in the death
            // phase and the parts die in play order (the main is left of
            // its parts, so its own death processes first).
            crate::cards::colossal::cascade_part_deaths(state, target);
        }
        _ => {}
    }
}

/// Emberroot Destroyer (M1-W5) — "Whenever your hero takes damage on your
/// turn, deal 3 damage to a random enemy minion." A damage-pipeline hook at
/// the point where a hero's health is actually reduced (armor-absorbed
/// damage does not count; there is no HeroDamaged trigger event). Fires on
/// the owner's turn only, one random enemy-minion ping per friendly
/// Emberroot Destroyer on the board.
/// M3-W2a hero-damage counters — fires at the exact point the hero's
/// Health is actually reduced (the Emberroot site): bumps the damaged
/// hero's owner's `hero_damaged_this_turn` flag and the opponent's
/// `enemy_hero_damaged_this_turn` count (the opponent is the player whose
/// "enemy hero" this hero is — their Devious Coyotes read the count).
fn fire_hero_damage_counters(state: &mut GameState, target: Entity) {
    if state.world().card_type(target) != Some(CardType::Hero) {
        return;
    }
    let Some(pid) = state.world().player(target) else {
        return;
    };
    let inner = state.make_mut();
    inner.players[pid.index()].hero_damaged_this_turn = true;
    inner.players[pid.opponent().index()].enemy_hero_damaged_this_turn += 1;
}

/// M5-W1 — Warptooth (JAIL_421): "Charge. If four friendly characters
/// take damage on one of your turns, summon this from your hand or
/// deck." A damage-pipeline hook at the two real-loss sites (the Emberroot
/// convention: armor-absorbed damage does not count). Four DISTINCT
/// friendly characters — any character owned by the active player, i.e.
/// the owner's own turn — must take real damage; the summon consumes the
/// trigger, so a second crossing of four does not summon a further copy
/// in the same turn (the per-turn count, §27). The damaged list clears at
/// the owner's turn start.
fn fire_warptooth_hook(state: &mut GameState, target: Entity) {
    let active = state.active_player();
    let Some(owner) = state.world().player(target) else {
        return;
    };
    if owner != active {
        return;
    }
    if state.player(active).warptooth_damaged_ids.contains(&target) {
        return;
    }
    state.make_mut().players[active.index()]
        .warptooth_damaged_ids
        .push(target);
    if state.player(active).warptooth_damaged_ids.len() < 4 {
        return;
    }
    // Four distinct friendly characters damaged this turn: summon
    // Warptooth from the hand, else from the deck — the actual card
    // moves to the battlefield (no fresh copy; the official "summon
    // this from your hand or deck"). The count then clears — the summon
    // consumed the trigger, so a later crossing of four in the same turn
    // finds no hand/deck copy (the card is already in play, §27).
    let summoned = state
        .world()
        .zones()
        .iter(Zone::Hand, active)
        .find(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "JAIL_421"))
        .or_else(|| {
            state
                .world()
                .zones()
                .iter(Zone::Deck, active)
                .find(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "JAIL_421"))
        });
    if let Some(e) = summoned {
        state
            .world_mut()
            .move_to_zone(e, Zone::Play)
            .expect("Warptooth must be movable from hand or deck to play");
        // The canonical summon-sickness shape (the MinionSummoned
        // convention): Charge minions skip it entirely and may attack
        // the turn they arrive; Rush minions are tracked by
        // SummonedThisTurn (hero-attack ban) but may attack minions;
        // everything else gets attacks_used(1).
        if !state.world().effective_charge(e) {
            state
                .world_mut()
                .set_summoned_this_turn(e, crate::core::component::SummonedThisTurn);
            if state.world().rush(e).is_none() {
                state.world_mut().set_attacks_used(e, AttacksUsed(1));
            }
        }
        state.make_mut().players[active.index()]
            .warptooth_damaged_ids
            .clear();
    }
}

fn fire_emberroot_hook(state: &mut GameState, queue: &mut EventQueue, target: Entity) {
    if state.world().card_type(target) != Some(CardType::Hero) {
        return;
    }
    let Some(pid) = state.world().player(target) else {
        return;
    };
    if state.active_player() != pid {
        return;
    }
    let emberroots: SmallList<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, pid)
        .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "FIR_955"))
        .collect();
    if emberroots.is_empty() {
        return;
    }
    let enemy = pid.opponent();
    let enemies: SmallList<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, enemy)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect();
    if enemies.is_empty() {
        return;
    }
    for emberroot in emberroots {
        let idx = state.rng_mut().next_usize(enemies.len());
        queue.push(Event::DamageDealt {
            source: emberroot,
            target: enemies[idx],
            amount: 3,
        });
    }
}

/// Applies a single event, possibly enqueueing new events.
///
/// This is the core of the event loop. Each event is processed exactly once;
/// if an event produces new events, they are added to the queue and processed in turn.
pub fn apply_event(
    state: &mut GameState,
    event: Event,
    queue: &mut EventQueue,
) -> Result<(), EngineError> {
    match event {
        Event::TurnStarted { player } => {
            // M3-W3 — Endtime Murozond (END_037): "Skip your next turn" —
            // the flag set by the battlecry is consumed here: the turn is
            // ended immediately (the TurnEnded handler runs the normal end
            // sequence and hands the turn to the opponent), so the skipped
            // player never refills mana, draws, or enters Main (§22).
            if state.player(player).skip_next_turn {
                state.make_mut().players[player.index()].skip_next_turn = false;
                // The skipped turn is treated as this player's "active" turn
                // while it ends, so the wrap-up boundary (which starts the
                // turn of `active_player().opponent()`) hands the initiative
                // straight to the opponent.
                state.set_active_player(player);
                queue.push(Event::TurnEnded { player });
                return Ok(());
            }
            // Ravenous Flock (M2-W4a): "At the start of your next turn, summon
            // three 2/1 Hatchlings" — the flag set by the play effect fires
            // here, once, at the owner's next turn start.
            if state.player(player).flock_pending {
                state.make_mut().players[player.index()].flock_pending = false;
                let hero = state.player(player).hero;
                for _ in 0..3 {
                    trigger::resolve_summon(state, queue, hero, player, "TLC_237t");
                }
            }
            // Petrified Ogre (M2-W4a): "Starts Dormant. While Dormant, gain
            // +2/+2 at the start of your turn. (50% chance to awaken
            // instead.)" — Dormant is modeled as can't-attack (the
            // established convention, see §17); each of the owner's turn
            // starts flips the official 50%: an awake outcome clears the
            // Dormant marker, a sleeping outcome stacks +2/+2.
            if state
                .world()
                .zones()
                .iter(Zone::Play, player)
                .any(|e| state.world().card_id(e).is_some_and(|c| c.0 == "TLC_253"))
            {
                let awaken = state.rng_mut().next_u32() % 2 == 0;
                let ogres: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Play, player)
                    .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "TLC_253"))
                    .collect();
                if awaken {
                    for e in &ogres {
                        state.world_mut().remove_cant_attack(*e);
                    }
                } else {
                    for e in &ogres {
                        state.world_mut().add_enchantment(
                            *e,
                            crate::core::component::Enchantment {
                                attack: 2,
                                health: 2,
                                cost: 0,
                                expiry: crate::core::component::EnchantmentExpiry::Permanent,
                            },
                        );
                    }
                }
            }
            // Corruption: destroy corrupted minions at the start of your turn
            // (mem::take swaps in an empty Vec — zero allocation, unlike drain().collect())
            let inner = state.make_mut();
            let corrupted = std::mem::take(&mut inner.players[player.index()].corrupted);
            for entity in corrupted {
                if !state.world().is_alive(entity)
                    || state.world().card_type(entity) != Some(CardType::Minion)
                {
                    continue;
                }
                let hp = state.world().health(entity).unwrap_or(Health(1));
                queue.push(Event::DamageDealt {
                    source: entity,
                    target: entity,
                    amount: hp.0.max(1),
                });
            }

            // First collect entities whose attack counts need resetting (requires a read-only
            // borrow). Only board characters hold attack state, so restrict to Zone::Play —
            // this also bounds the list (hero + up to 7 minions + weapon) for the stack buffer.
            let player_entities: SmallList<Entity> = state
                .world()
                .iter_player()
                .filter(|(e, pid)| **pid == player && state.world().zone(*e) == Some(Zone::Play))
                .map(|(e, _)| e)
                .collect();
            let new_turn = state.turn() + 1;
            // Freeze timing (engine-mechanics roadmap M2): entities frozen at
            // the START of this turn (i.e. during the opponent's turn) keep
            // Freeze through the whole turn — the AttackDeclared check blocks
            // their attacks — and thaw in the turn-end wrap-up. Entities
            // frozen DURING this turn (by this player's own actions, e.g.
            // Icicle) are not in the snapshot and stay frozen into the next
            // turn, matching HS. The snapshot is taken before the attack
            // resets below.
            let frozen_at_start: SmallList<Entity> = player_entities
                .iter()
                .copied()
                .filter(|&e| state.world().freeze(e).is_some())
                .collect();

            // Then perform all modifications step by step
            {
                let world = state.world_mut();
                for &entity in player_entities.iter() {
                    world.set_attacks_used(entity, AttacksUsed(0));
                    // Reset the hero-power-used flag
                    world.set_hero_power_used(entity, HeroPowerUsed(false));
                    // SummonedThisTurn expires: a Rush minion may attack the
                    // hero from this turn on (Core Set W1)
                    world.remove_summoned_this_turn(entity);
                }
            }
            {
                let inner = state.make_mut();
                inner.players[player.index()].frozen_at_turn_start =
                    frozen_at_start.iter().copied().collect();
                // Core Set W3a — the healed-this-turn marker expires at turn start
                inner.players[player.index()].healed_this_turn = false;
                // Core Set W4b — the enemy-spell cost tax expires at turn start
                inner.players[player.index()].enemy_spell_cost_more = 0;
                // M2-W4a: "until the start of your next turn" effects expire at
                // the owner's turn start — Wilted Shadow's heal-block on the
                // opponent hero and Wave of Tar's enemy-minion cost tax.
                inner.players[player.index()].enemy_hero_cant_be_healed = false;
                inner.players[player.index()].minions_cost_more = false;
                // M3-W2a — per-turn hero-damage counters expire at turn start
                // (Devious Coyote's discount, Liferender's battlecry check).
                inner.players[player.index()].enemy_hero_damaged_this_turn = 0;
                inner.players[player.index()].hero_damaged_this_turn = false;
                // M3-W2a — Clockwork Rager counts the turns taken this game.
                inner.players[player.index()].turns_taken += 1;
                // M5-W1 — Chef Neth'rek (JAIL_860): "If your deck only has
                // cards that cost (3) or less, set your Mana to 10 after
                // five turns!" — the StartOfGame deck check armed
                // `nethrek_mana_after_five`; after the fifth own turn start
                // the crystals cap at 10 (the ManaRefill step that follows
                // this handler refills the current mana from the crystals).
                if inner.players[player.index()].nethrek_mana_after_five
                    && inner.players[player.index()].turns_taken >= 5
                {
                    inner.players[player.index()].mana_crystals = 10;
                }
                // M5-W1 — per-turn resets: Warptooth's four-friendly-damage
                // counter (JAIL_421), the Prepare once-per-turn guard
                // (JAIL_407/721/906) and Zee's Might's fifth-minion counter
                // (JAIL_800hp2) all clear at the owner's turn start.
                inner.players[player.index()].warptooth_damaged_ids.clear();
                inner.players[player.index()].prepare_used_this_turn = false;
                inner.players[player.index()].zee_might_counter = 0;
                // M3-W2a — TIME_716 Slow Motion's tax applied to the
                // opponent's cards during their (just-finished) turn:
                // the caster's tax expires at the caster's next turn start.
                inner.players[player.index()].next_turn_enemy_cards_cost_more = 0;
                // M3-W2b — Chrono-Lord Epoch's per-turn minion-play list
                // resets at the owner's turn start (the TurnEnded handler
                // snapshotted it into last_turn_minion_play_ids for the
                // opponent to destroy).
                inner.players[player.index()]
                    .minions_played_this_turn_ids
                    .clear();
                // M4-W4 — per-turn counters reset at the owner's turn start
                // (the mana-spent tally behind CATA_130/132/135's
                // spend-condition arms, the spell-damage tally behind
                // CATA_483/487, the once-per-turn spell-damage guard and
                // the Fire-spell marker behind CATA_584).
                inner.players[player.index()].mana_spent_this_turn = 0;
                inner.players[player.index()].spell_damage_dealt_this_turn = 0;
                inner.players[player.index()].first_spell_damage_gain_used = false;
                inner.players[player.index()].fire_spell_played_this_turn = false;
                // M5-W2 — per-turn resets: the elemental-chain spell
                // counter (JAIL_801 Molten Gold / JAIL_803 Frostshatter /
                // JAIL_805 Stormfury / JAIL_735 Code Violet / JAIL_321
                // Tricksy Improviser read "3 other spells this turn"), the
                // rewind mark for Slice and Dice (JAIL_500 — the replay
                // starts from the plays recorded after this mark) and the
                // Murloc Holmes suspect (JAIL_851 — the investigation
                // window is the suspect's next turn; the record lives on
                // the investigator, whose turn this is — by now the
                // suspect's turn has passed).
                inner.players[player.index()].spells_cast_this_turn = 0;
                inner.players[player.index()].rewind_turn_start_len =
                    inner.players[player.index()].last_played.len();
                inner.players[player.index()].murloc_holmes_suspect = None;
            }
            // M3-W2a — Dormant countdown: at the owner's turn start each
            // dormant minion sleeps one less turn; at 0 it awakens (the
            // component is removed and the aura index is restored by
            // `remove_dormant`). Runs before the StartTriggers step, so an
            // awakening minion's triggers fire the same turn.
            {
                let slumbering: SmallList<(Entity, u32)> = state
                    .world()
                    .zones()
                    .iter(Zone::Play, player)
                    .filter_map(|e| state.world().dormant(e).map(|d| (e, d.turns)))
                    .collect();
                for (entity, turns) in slumbering {
                    if turns <= 1 {
                        state.world_mut().remove_dormant(entity);
                    } else {
                        state.world_mut().set_dormant(
                            entity,
                            crate::core::component::Dormant { turns: turns - 1 },
                        );
                    }
                }
            }
            // M3-W2a — the played-this-turn markers expire at the owner's
            // turn start (the TIME_620 secret predicate: "the turn after it
            // was played" is approximated as "any later turn" — the marker
            // set at CardPlayed lasts until the play's turn ends, so a
            // same-turn death also fires the secret, §20).
            {
                let marked: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Play, player)
                    .filter(|&e| state.world().played_this_turn(e).is_some())
                    .collect();
                for e in marked {
                    state.world_mut().remove_played_this_turn(e);
                }
            }
            // M3-W2a — TIME_876 Shapeshifter: "At the start of your turn,
            // transform into a random minion in your opponent's hand" — a
            // hand card marked by the card id (pool-open read, §20).
            {
                let shapeshifters: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Hand, player)
                    .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "TIME_876"))
                    .collect();
                for e in shapeshifters {
                    trigger::resolve_effect(
                        state,
                        queue,
                        e,
                        player,
                        crate::core::effect::CardEffect::TransformHandSelfToRandomEnemyHandMinion,
                        None,
                        None,
                    );
                }
            }
            // M3-W2b — TIME_024 Murozond, Unbounded: "At the start of your
            // next turn, this minion's Attack is infinite" — the play
            // effect armed the flag; this turn start consumes it and sets
            // the Attack to the INFINITY cap (an unbounded value does not
            // exist in a 32-bit engine, fidelity-debt §21; the constant is
            // shared with W3).
            if let Some(murozond) = state.player(player).murozond_infinite_pending {
                state.make_mut().players[player.index()].murozond_infinite_pending = None;
                if state.world().is_alive(murozond)
                    && state.world().zone(murozond) == Some(Zone::Play)
                {
                    state.world_mut().set_attack(
                        murozond,
                        crate::core::component::Attack(
                            crate::cards::exp_tmw_w2b::INFINITY_ATTACK_CAP,
                        ),
                    );
                }
            }
            // M4-W4 — CATA_528 Sigil of the Seas: "At the start of your
            // next turn, summon a 3/3 Naga with Taunt" — the pending
            // summon armed by the play resolves once at the owner's turn
            // start.
            if let Some(id) = state.player(player).next_turn_summon.clone() {
                state.make_mut().players[player.index()].next_turn_summon = None;
                let hero = state.player(player).hero;
                trigger::resolve_summon(state, queue, hero, player, &id);
            }
            // M4-W4 — CATA_498 Rafaam's Last Stand: "(Upgrades each turn!)"
            // — the hand card's in-hand turn counter ticks at each owner
            // turn start (the effect arm reads it as bonus damage).
            {
                let marked: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Hand, player)
                    .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "CATA_498"))
                    .collect();
                for e in marked {
                    let cur = state
                        .world()
                        .hand_turn_counter(e)
                        .unwrap_or(HandTurnCounter(0))
                        .0;
                    state
                        .world_mut()
                        .set_hand_turn_counter(e, HandTurnCounter(cur + 1));
                }
            }
            // M4-W4 — CATA_615 Genn, Cursed King: "While holding this, if
            // the rest of your hand is all even or all odd, transform into
            // the 6/5 Worgen King." The official continuous check is
            // approximated at each own turn start (an empty "rest" hand
            // vacuously qualifies; the transform is permanent once
            // triggered, §26).
            {
                let genn: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Hand, player)
                    .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "CATA_615"))
                    .collect();
                for e in genn {
                    let rest: Vec<Entity> = state
                        .world()
                        .zones()
                        .iter(Zone::Hand, player)
                        .filter(|&h| h != e)
                        .collect();
                    let all_even = rest.iter().all(|&h| {
                        state
                            .world()
                            .effective_cost(h)
                            .is_some_and(|c| c.0 % 2 == 0)
                    });
                    let all_odd = rest.iter().all(|&h| {
                        state
                            .world()
                            .effective_cost(h)
                            .is_some_and(|c| c.0 % 2 == 1)
                    });
                    if rest.is_empty() || all_even || all_odd {
                        if let Some(def) = crate::cards::def::card_by_id("CATA_615t") {
                            trigger::resolve_transform_to_def(state, e, def);
                        }
                    }
                }
            }
            // M5-W1 — Godfrey the Betrayer (JAIL_509): at the owner's turn
            // start, overdrawn cards return to the hand (oldest first) while
            // there is space, each costing (1) less — the F-A11 override
            // armed at StartOfGame (the draw path parked the cards in the
            // SetAside zone instead of burning them).
            {
                let held: Vec<(Entity, i32)> = state.player(player).godfrey_held_cards.to_vec();
                let room =
                    MAX_HAND_SIZE.saturating_sub(state.world().zones().len(Zone::Hand, player));
                if !held.is_empty() && room > 0 {
                    for (e, _held_cost) in held.into_iter().take(room) {
                        state
                            .world_mut()
                            .move_to_zone(e, Zone::Hand)
                            .expect("parked card should be movable to hand");
                        trigger::reduce_hand_card_cost(state, e, 1);
                    }
                    state.make_mut().players[player.index()]
                        .godfrey_held_cards
                        .drain(..room);
                }
            }
            // M4-W4 — CantPlayNextTurn expires at the owner's turn start
            // (CATA_186t Sabotage!'s "It can't be played until your next
            // turn" — and M5-W1's Prepare lock, which reuses the same
            // component: a prepared card stays unplayable exactly until
            // its owner's next turn start).
            {
                let marked: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Hand, player)
                    .filter(|&e| state.world().cant_play_next_turn(e).is_some())
                    .collect();
                for e in marked {
                    state.world_mut().remove_cant_play_next_turn(e);
                }
            }
            state.set_active_player(player);
            state.set_turn(new_turn);
            // Enter the start-of-turn sequence. The mana refill and the draw are
            // NOT done here: the step machine runs StartTriggers (start-of-turn
            // secrets already fired on this event via check_secrets, start-turn
            // card effects fire next) → ManaRefill → TurnCostReduce → DrawStep →
            // Main, so that start-of-turn triggers resolve before the refill
            // and the draw.
            state.set_step(Step::StartTriggers);
        }
        Event::TurnEnded { player } => {
            // Quest progress (M2-W1): TLC_602 — "Survive 10 turns" — fires
            // for the player ending their turn while their hero is alive
            // (W2 pins the official timing).
            if state
                .world()
                .health(state.player(player).hero)
                .is_some_and(|h| h.0 > 0)
            {
                crate::engine::quest::progress(
                    state,
                    queue,
                    player,
                    crate::cards::quest::QuestCondition::SurviveTurns,
                    1,
                    None,
                );
            }
            // Story of Lakkari (M2-W4a): "At the end of your turn, discard a
            // card and fill your board with 3/2 Imps. Lasts 3 turns." — the
            // tick counter was set by the play effect; each of the owner's
            // turn ends consumes one tick (registered simplification: the
            // discard is random — the official card discards a random card —
            // and "fill your board" summons an Imp into every free slot).
            if state.player(player).lakkari_ticks > 0 {
                {
                    let p = &mut state.make_mut().players[player.index()];
                    p.lakkari_ticks -= 1;
                }
                trigger::resolve_discard_random(state, queue, player);
                let board_count = state
                    .world()
                    .zones()
                    .iter(Zone::Play, player)
                    .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                    .count();
                for _ in board_count..crate::engine::rules::MAX_BOARD_SIZE {
                    trigger::resolve_summon(
                        state,
                        queue,
                        state.player(player).hero,
                        player,
                        "TLC_466t",
                    );
                }
            }
            // M5-W2 — Reinforcement Aura (JAIL_327): "At the end of your
            // turns for the next 3 turns, summon a random minion costing
            // (2) or less from your deck" — the tick counter armed by the
            // spell's cast is consumed here (the Lakkari pattern); the
            // summon follows the Finja convention (§28: a fresh copy —
            // the deck card itself stays).
            if state.player(player).reinforcement_aura_ticks > 0 {
                {
                    let p = &mut state.make_mut().players[player.index()];
                    p.reinforcement_aura_ticks -= 1;
                }
                let deck_minions: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Deck, player)
                    .filter(|&e| {
                        state
                            .world()
                            .card_id(e)
                            .and_then(|c| crate::cards::def::card_by_id(c.0))
                            .is_some_and(|d| d.card_type == CardType::Minion && d.cost <= 2)
                    })
                    .collect();
                if let Some(&e) = deck_minions.get(state.rng_mut().next_usize(deck_minions.len())) {
                    if let Some(def) = state
                        .world()
                        .card_id(e)
                        .and_then(|c| crate::cards::def::card_by_id(c.0))
                    {
                        let _ = trigger::resolve_summon(
                            state,
                            queue,
                            state.player(player).hero,
                            player,
                            def.id,
                        );
                    }
                }
            }
            // Hatching Ceremony (M2-W4c): the +2/+2 to the owner's minions
            // lands at the end of the owner's NEXT turn after the cast — a
            // two-tick countdown armed by the spell's cast (a one-shot
            // armed at this turn's end would fire a full turn early — the
            // flag is armed during the caster's own turn, whose end comes
            // before the opponent's turn).
            if state.player(player).hatching_pending > 0 {
                {
                    let p = &mut state.make_mut().players[player.index()];
                    p.hatching_pending -= 1;
                }
                if state.player(player).hatching_pending == 0 {
                    let minions: SmallList<Entity> = state
                        .world()
                        .zones()
                        .iter(Zone::Play, player)
                        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                        .collect();
                    for e in minions {
                        state.world_mut().add_enchantment(
                            e,
                            crate::core::component::Enchantment {
                                attack: 2,
                                health: 2,
                                cost: 0,
                                expiry: crate::core::component::EnchantmentExpiry::Permanent,
                            },
                        );
                    }
                }
            }
            // Soulrest Ceremony (M2-W4c): the marked minions ("they die at
            // the end of your turn") die at the end of the owner's turn —
            // damaged to death through the normal death path (the
            // Corruption pattern, so deathrattles fire).
            {
                let inner = state.make_mut();
                let marked = std::mem::take(&mut inner.players[player.index()].soulrest_marked);
                for entity in marked {
                    if !state.world().is_alive(entity)
                        || state.world().card_type(entity) != Some(CardType::Minion)
                    {
                        continue;
                    }
                    let hp = state.world().health(entity).unwrap_or(Health(1));
                    queue.push(Event::DamageDealt {
                        source: entity,
                        target: entity,
                        amount: hp.0.max(1),
                    });
                }
            }
            // End-of-turn effects fire in the EndTriggers step — before the
            // wrap-up cleanup — so effects resolve at full strength and deaths
            // they cause are processed before "until end of turn" buffs expire.
            state.set_step(Step::EndTriggers);
            // Alexandros Mograine (Core Set W4a): ongoing end-of-turn damage
            // to the opponent
            let ongoing = state.player(player).ongoing_end_turn_damage;
            if ongoing > 0 {
                let enemy_hero = state.player(player.opponent()).hero;
                queue.push(Event::DamageDealt {
                    source: enemy_hero,
                    target: enemy_hero,
                    amount: ongoing,
                });
            }
            // 2025–2026 expansions M1-W4a pending end-of-turn timers: Rotten
            // Apple's self-damage and Fractured Power's delayed Mana Crystals
            // tick at the END of each of the player's own turns (registered
            // simplification — the real timing is "for the next 2 turns" from
            // cast, see §14.3).
            let hero = state.player(player).hero;
            let (self_dmg, self_ticks) = {
                let p = state.player(player);
                (p.self_damage_pending, p.self_damage_turns)
            };
            if self_ticks > 0 {
                {
                    let p = &mut state.make_mut().players[player.index()];
                    p.self_damage_turns -= 1;
                    if p.self_damage_turns == 0 {
                        p.self_damage_pending = 0;
                    }
                }
                queue.push(Event::DamageDealt {
                    source: hero,
                    target: hero,
                    amount: self_dmg,
                });
            }
            let (crystal_gain, crystal_ticks) = {
                let p = state.player(player);
                (p.crystal_gain_pending, p.crystal_gain_turns)
            };
            if crystal_ticks > 0 {
                {
                    let p = &mut state.make_mut().players[player.index()];
                    p.crystal_gain_turns -= 1;
                    if p.crystal_gain_turns == 0 {
                        p.crystal_gain_pending = 0;
                    }
                    p.mana_crystals = (p.mana_crystals + crystal_gain).min(10);
                }
            }
            // M3-W2b — Chrono-Lord Epoch: snapshot this turn's minion
            // plays into the "last turn" list — TIME_714's battlecry
            // destroys the opponent's minions matching it. The list is
            // mem::take'd (an empty Vec swaps in, no allocation).
            {
                let played = std::mem::take(
                    &mut state.make_mut().players[player.index()].minions_played_this_turn_ids,
                );
                state.make_mut().players[player.index()].last_turn_minion_play_ids = played;
            }
            // M3-W3 — Eternal Firebolt (END_025): "If it dies, return this
            // to your hand at the end of your turn" — the recorded target's
            // death returns a fresh copy of the spell to the owner's hand
            // (the played spell card itself sits in the graveyard; the
            // returned copy is the base card, F-A11 hand cap applies). The
            // record is consumed either way.
            if let Some(flame_target) = state.player(player).eternal_flame_target {
                state.make_mut().players[player.index()].eternal_flame_target = None;
                let dead = state.world().zone(flame_target) != Some(Zone::Play)
                    || state
                        .world()
                        .effective_health(flame_target)
                        .is_some_and(|h| h.is_dead());
                if dead {
                    if let Some(def) = crate::cards::def::card_by_id("END_025") {
                        trigger::add_card_to_hand(state, player, def);
                    }
                }
            }
            // M4-W4 — CATA_480 Sandfury Aura: "Your minions' end of turn
            // effects trigger twice. Lasts 3 turns." — each of the owner's
            // turn ends consumes one tick (the EndTriggers step re-fires
            // the full TurnEnd trigger set while the counter is positive).
            if state.player(player).end_turn_effects_twice_turns > 0 {
                let p = &mut state.make_mut().players[player.index()];
                p.end_turn_effects_twice_turns -= 1;
            }
            // M4-W4 — CATA_301 Ruby Sanctum's "Your next Healing effect
            // this turn deals damage instead" — the leftover flag clears at
            // the owner's turn end (a backstop; the funnel already clears
            // it on conversion).
            if state.player(player).next_heal_deals_damage {
                state.make_mut().players[player.index()].next_heal_deals_damage = false;
            }
        }
        Event::CardPlayed {
            player,
            card,
            target,
            position,
        } => {
            // Deduct mana (single cost composition — roadmap G5). Death
            // Metal Knight (Core Set W3a) pays Health instead of Mana when
            // the hero was healed this turn.
            let cost = crate::engine::cost::play_cost(state, card, player);
            let card_type = state.world().card_type(card);
            let is_minion = state.world().card_type(card) == Some(CardType::Minion);
            // Twisted Webweaver (M1-W4a): the played-minion log — every
            // minion play is recorded so a later play of the same card ID can
            // be detected ("another minion you've already played")
            let played_minion_id = if is_minion {
                state.world().card_id(card).map(|c| c.0)
            } else {
                None
            };
            // Naralex (M1-W4b): whether the played card is a Dragon — the
            // first Dragon played each turn costs (1) while he is on the board
            let is_dragon = is_minion
                && state
                    .world()
                    .has_race(card, crate::core::component::Race::Dragon);
            // M3-W3 — whether the played card is an Undead (Blessing of the
            // Infinite's per-turn counter, END_003p) — captured before the
            // player block below.
            let is_undead = is_minion
                && state
                    .world()
                    .has_race(card, crate::core::component::Race::Undead);
            // M2-W4a: the CostHealth marker (Whispering Stone's gotten Fel
            // spells) pays Health instead of Mana, like Death Metal Knight's
            // id-keyed pay-health branch.
            let pay_health = (state.player(player).healed_this_turn
                && state
                    .world()
                    .card_id(card)
                    .is_some_and(|c| c.0 == "CORE_ETC_523"))
                || state.world().cost_health(card).is_some();
            // M4-W4 — CATA_180 War'loc: "Your next Murloc that costs
            // (3) or less costs Health instead of Mana" — the one-time
            // flag pays Health for the first eligible Murloc played
            // (§26: the "next" window opens from the battlecry and the
            // eligibility reads the play cost).
            let war_loc = state.player(player).next_murloc_costs_health
                && is_minion
                && state
                    .world()
                    .has_race(card, crate::core::component::Race::Murloc)
                && cost.0 <= 3;
            if war_loc {
                state.make_mut().players[player.index()].next_murloc_costs_health = false;
            }
            if pay_health || war_loc {
                let hero = state.player(player).hero;
                queue.push(Event::DamageDealt {
                    source: hero,
                    target: hero,
                    amount: cost.0,
                });
            }
            // M2-W4a Map cards (Crypt/Submerged/Mountain/Cultist/Odd/Hive
            // Map): playing the discovered card this turn also adds one random
            // other option to hand (the simplified "pick one of the others"
            // chain, fidelity-debt §17). The pending entry is consumed here —
            // once, by the discovered card's play.
            if let Some((discovered, others)) = state.player(player).map_pending.clone() {
                if discovered == card && !others.is_empty() {
                    state.make_mut().players[player.index()].map_pending = None;
                    if let Some(other) = others.get(state.rng_mut().next_usize(others.len())) {
                        if let Some(other_def) = crate::cards::def::card_by_id(other.as_str()) {
                            trigger::add_card_to_hand(state, player, other_def);
                        }
                    }
                }
            }
            // Reanimated Pterrordax (M2-W4a): "Costs Corpses instead of Mana"
            // — the 5 Corpses are spent at play (play_cost returns Cost(0);
            // the validate gate below blocks the play without enough Corpses).
            if state
                .world()
                .card_id(card)
                .is_some_and(|c| c.0 == "TLC_436")
            {
                let inner = state.make_mut();
                let p = &mut inner.players[player.index()];
                p.corpses = p.corpses.saturating_sub(5);
            }
            // Hot Spring Glider (M2-W3): "your next Murloc costs (1) less
            // / gains Divine Shield" — one-time flags, consumed by the
            // next Murloc play (the cost discount was already applied by
            // play_cost above; the Divine Shield lands on the entity here
            // and carries into play).
            let is_murloc_play = state
                .world()
                .card_id(card)
                .and_then(|cid| crate::cards::def::card_by_id(cid.0))
                .is_some_and(|def| def.race == Some(crate::core::component::Race::Murloc));
            if is_murloc_play && state.player(player).next_murloc_divine_shield {
                state
                    .world_mut()
                    .set_divine_shield(card, crate::core::component::DivineShield);
            }
            // Spelunker (M2-W4a): whether the played card carries the
            // Temporary marker — the next-Temporary discount is consumed by
            // its play (captured before the player block below).
            let is_temporary_play = state.world().temporary(card).is_some();
            // Kindred (M2-W3): the played card's type — spells push `Spell`,
            // minions their primary (first) race; captured before the
            // player block below (the push happens there; play_cost ran
            // before it, so cost-time Kindred discounts count earlier
            // same-type cards only).
            let kindred_type_played = if card_type == Some(CardType::Spell) {
                Some(crate::cards::kindred::KindredType::Spell)
            } else if is_minion {
                state
                    .world()
                    .race(card)
                    .and_then(|r| r.first())
                    .map(|&race| crate::cards::kindred::KindredType::Minion(race))
            } else {
                None
            };
            // M4-W4 (2025–2026 expansions, Cataclysm §26) reads captured
            // before the player block below (the block holds the only
            // mutable borrow): CATA_560 Confront the Tol'vir replays the
            // 1-Cost cards played this game; CATA_557 Sylvanas's Triumph
            // checks the played-copy flag.
            let played_card_id = state.world().card_id(card).map(|c| c.0.to_string());
            let is_sylvanas_triumph = played_card_id.as_deref() == Some("CATA_557");
            {
                let inner = state.make_mut();
                let p = &mut inner.players[player.index()];
                if !pay_health {
                    p.current_mana -= cost.0;
                    // "spent X Mana while holding this" cards (CATA_131
                    // Felwood Treant / CATA_132 Broodwatcher / CATA_140
                    // Merithra) accumulate every mana deduction here.
                    p.mana_spent_this_turn += cost.0;
                    // CATA_130 Crystalspine Cub "Whenever you spend your
                    // last Mana Crystal" — flagged here, the trigger fires
                    // after the borrow block ends (in play order).
                    p.last_mana_crystal_spent_pending = p.current_mana == 0;
                }
                // CATA_616 Gronn Giant "Cost reduced by the Cost of the
                // last card you played": recorded before the CardPlayed
                // triggers fire (their own plays update it again).
                p.last_played_card_cost = cost.0;
                // CATA_560: the played 1-Cost card's id list.
                if cost.0 == 1 {
                    if let Some(cid) = played_card_id {
                        p.played_one_cost_cards.push(cid);
                    }
                }
                // M5-W2 — Jade Guardians (JAIL_474): the per-game counter
                // of cards played that cost (2) — the discount the two
                // 8-Cost minions get.
                if cost.0 == 2 {
                    p.cards_played_cost_2 += 1;
                }
                if is_sylvanas_triumph {
                    p.sylvanas_triumph_played = true;
                }
                p.cards_played_this_turn += 1;
                if is_murloc_play {
                    p.next_murloc_discount = 0;
                    p.next_murloc_divine_shield = false;
                }
                // One-time discounts are consumed on play (Core Set W4a)
                p.next_demon_discount = 0;
                p.next_outcast_discount = 0; // Illidari Studies (W6)
                p.next_combo_discount = 0;
                // Spelunker (M2-W4a): the next-Temporary discount is one-time,
                // consumed by the next Temporary card play (its cost already
                // included the discount)
                if is_temporary_play {
                    p.next_temporary_discount = 0;
                }
                // Agamaggan (M1-W4b): the next-card-costs-zero flag is
                // one-time, consumed on play (its cost already included it)
                p.next_card_costs_zero = false;
                if is_minion {
                    p.minions_played_this_turn += 1;
                    if let Some(id) = played_minion_id {
                        p.played_minion_ids.push(id.to_string());
                    }
                    // M5-W1 — Zee's Might (JAIL_800hp2, the Mug'Zee hero
                    // power): "Every 5th minion you play triggers its
                    // Battlecry twice." — the counter arms the readiness
                    // flag, consumed by the played minion's battlecry
                    // resolution (the MinionSummoned site below, gated on
                    // `played_minion`). Resets at the owner's turn start.
                    if p.mugzee_zee_might {
                        p.zee_might_counter += 1;
                        if p.zee_might_counter >= 5 {
                            p.zee_might_counter = 0;
                            p.zee_might_ready = true;
                        }
                    }
                    // Naralex (M1-W4b): the per-turn Dragon play counter —
                    // "your first Dragon each turn costs (1)"
                    if is_dragon {
                        p.dragons_played_this_turn += 1;
                    }
                    // M3-W3 — Blessing of the Infinite (END_003p, the
                    // Death Knight imbued hero power): the per-turn Undead
                    // play counter. Incremented BEFORE the CardPlayed
                    // triggers fire (the passive's trigger reads it — a
                    // counter above 1 means the first Undead already
                    // played, §22). Reset at the owner's ManaRefill step.
                    if is_undead {
                        p.undead_played_this_turn += 1;
                    }
                }
                // Preparation (W11): the discount is one-time — the first
                // spell played consumes it (its cost already included it)
                if card_type == Some(CardType::Spell) {
                    p.next_spell_discount = 0;
                }
                // Kindred (M2-W3): push the played card's type — the
                // activation check of a Kindred card counts its own push
                // plus any earlier same-type card (>= 2) at OnPlay /
                // battlecry time, and only earlier cards (>= 1) at cost
                // time (play_cost runs before this push). The push is the
                // only way a card enters the list, so effect-summoned
                // copies of a Kindred card never re-fire (Conjured
                // Bookkeeper's copy does not loop).
                if let Some(kindred_type) = kindred_type_played {
                    p.kindred_played.push(kindred_type);
                }
            }
            // CATA_130 Crystalspine Cub "Whenever you spend your last Mana
            // Crystal" (M4-W4, §26): the play-cost deduction set the flag —
            // the trigger fires after the player block, in play order.
            if state.make_mut().players[player.index()].last_mana_crystal_spent_pending {
                state.make_mut().players[player.index()].last_mana_crystal_spent_pending = false;
                fire_triggers(
                    state,
                    queue,
                    crate::core::component::TriggerEvent::LastManaCrystalSpent,
                    player,
                    None,
                    None,
                );
            }
            // M4-W4 — CATA_585 Torch: "Return this to hand with any
            // excess damage" — the die-check during the effect resolution
            // set the pending flag; a fresh copy returns to the hand (the
            // played entity still moves to the graveyard below, F-A11 hand
            // cap applies — §26's die-check approximation of the excess
            // damage return).
            if state.player(player).torch_return_pending {
                state.make_mut().players[player.index()].torch_return_pending = false;
                if let Some(def) = crate::cards::def::card_by_id("CATA_585") {
                    trigger::add_card_to_hand(state, player, def);
                }
            }
            // M4-W4 — the Dragon-in-hand transform (CATA_551 Stonetalon
            // Striker / CATA_552 Ebonscale Scout / CATA_553 Ebyssian):
            // "While in hand, play a Dragon to become a ... Dragon!" —
            // each transformed hand entity swaps its card id for the
            // token id at the Dragon's play (the token def carries the
            // new stats, race and type — §26).
            if is_dragon {
                let transforming: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Hand, player)
                    .filter(|&e| {
                        state
                            .world()
                            .card_id(e)
                            .is_some_and(|c| matches!(c.0, "CATA_551" | "CATA_552" | "CATA_553"))
                    })
                    .collect();
                for e in transforming {
                    let Some(id) = state.world().card_id(e).map(|c| c.0) else {
                        continue;
                    };
                    let token = match id {
                        "CATA_551" => "CATA_551t",
                        "CATA_552" => "CATA_552t",
                        _ => "CATA_553t",
                    };
                    // The full transform (shared primitive): the token's
                    // stats, race and keywords replace the base card's —
                    // the Dragon form is played as its own card (6/6
                    // Taunt for CATA_551t, and so on).
                    if let Some(def) = crate::cards::def::card_by_id(token) {
                        trigger::resolve_transform_to_def(state, e, def);
                    }
                }
            }
            // Outcast (Core Set W2): a card played from the leftmost or
            // rightmost hand position carries the Outcast marker — read
            // BEFORE the card leaves the hand. A one-card hand counts as
            // both edges (official rule).
            let hand_len = state.world().zones().len(Zone::Hand, player);
            let hand_index = state
                .world()
                .zones()
                .iter(Zone::Hand, player)
                .position(|e| e == card);
            if hand_index == Some(0) || hand_index == Some(hand_len - 1) {
                state
                    .world_mut()
                    .set_outcast_played(card, crate::core::component::OutcastPlayed);
            }
            // M2-W4c Skittish Saucier: the played card's ORIGINAL hand
            // position is recorded for the battlecry (which reduces the
            // Cost of the adjacent hand cards — after the card leaves the
            // hand, the left neighbor sits at k-1 and the right neighbor
            // slid to k). The battlecry resolves within this same play
            // burst, so no later play can stale the record.
            if let Some(k) = hand_index {
                state.make_mut().players[player.index()].last_played_hand_index = Some(k);
            }
            // M3-W2a Precise Shot (TIME_600): "If this is EXACTLY in the
            // center of your hand" — the center exists only for odd-sized
            // hands (the middle card). Captured here before the card
            // leaves the hand, like the Skittish Saucier position above.
            state.make_mut().players[player.index()].last_played_hand_center =
                hand_len >= 1 && hand_len % 2 == 1 && hand_index == Some((hand_len - 1) / 2);
            // M5-W2 — Unshackle Soul / Mind Sweeper: the "played a copy of
            // an opponent's card" game-scoped flag reads the
            // `CopiedFromOpponent` marker at play (the "while holding
            // this" nuance is dropped, §28).
            if state.world().copied_from_opponent(card).is_some() {
                state.make_mut().players[player.index()].copied_from_opponent_played = true;
            }
            // M5-W2 — Inspector Murloc Holmes (JAIL_851): when a card
            // whose NAME matches the suspect is played on the suspect's
            // next turn, the investigator receives 3 Coins. The suspect
            // record lives on the OTHER player (the investigator).
            {
                let investigator = player.opponent();
                let suspect = state.player(investigator).murloc_holmes_suspect.clone();
                if let Some((name, expected_turn)) = suspect {
                    if expected_turn == state.turn() {
                        let played_name = state
                            .world()
                            .card_id(card)
                            .and_then(|c| crate::cards::def::card_by_id(c.0))
                            .map(|d| d.name);
                        if played_name == Some(name.as_str()) {
                            state.make_mut().players[investigator.index()].murloc_holmes_suspect =
                                None;
                            if let Some(coin) = crate::cards::def::card_by_id("GAME_005") {
                                for _ in 0..3 {
                                    trigger::add_card_to_hand(state, investigator, coin);
                                }
                            }
                        }
                    }
                }
            }
            // M5-W2 — Low Security Wing (JAIL_987): "It can't be played
            // until you play a card" — any play clears the lock on every
            // hand card.
            {
                let locked: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Hand, player)
                    .filter(|&e| state.world().locked_until_card_played(e).is_some())
                    .collect();
                for e in locked {
                    state.world_mut().remove_locked_until_card_played(e);
                }
            }

            // Detect combo: another card was played this turn (cards_played > 1 because it was just incremented)
            let combo_active = state.player(player).cards_played_this_turn > 1;
            // Quest progress (M2-W2): TLC_446 — "Play 6 Temporary cards" —
            // a played card carrying the Temporary marker counts for its
            // owner. No W2 card creates Temporary cards (the creators are
            // W4); the F5 scenarios inject the marker directly.
            if state.world().temporary(card).is_some() {
                crate::engine::quest::progress(
                    state,
                    queue,
                    player,
                    crate::cards::quest::QuestCondition::PlayTemporaryCards,
                    1,
                    None,
                );
            }
            // Quest card (2025–2026 expansions M2-W1): quests are 1-cost
            // legendary spells played into the per-player quest slot instead
            // of the spell path — no move-to-Play, no secret handling, no
            // SpellCast event, no graveyard move. The generic played-card
            // side effects (mana deduction above, Overload, CardPlayed
            // triggers below) still apply.
            if let Some(qdef) = state
                .world()
                .card_id(card)
                .and_then(|cid| crate::cards::quest::quest_def(cid.0))
            {
                // M2-W4a: "If you played a Quest this game" (Questing
                // Assistant) — the played-a-quest flag, set at the quest
                // play-path diversion, persists for the game. A SIDEQUEST
                // (TLC_EVENT_400) is not a Quest — it never sets the flag
                // (official classification: sidequests occupy their own
                // slot and don't count for "played a Quest" effects).
                if !state
                    .world()
                    .card_id(card)
                    .is_some_and(|c| c.0 == "TLC_EVENT_400")
                {
                    state.make_mut().players[player.index()].quest_played = true;
                }
                // One quest per player: an occupied slot's quest is destroyed
                // (progress lost, no reward — the official replace rule).
                for old in state.world().zones().entities(Zone::Quest, player) {
                    state
                        .world_mut()
                        .move_to_zone(old, Zone::Graveyard)
                        .map_err(|_| EngineError::EntityGone(old))?;
                }
                state
                    .world_mut()
                    .move_to_zone(card, Zone::Quest)
                    .map_err(|_| EngineError::EntityGone(card))?;
                state.world_mut().set_quest(
                    card,
                    crate::core::component::Quest {
                        progress: 0,
                        target: qdef.target,
                        repeatable: qdef.repeatable,
                        markers: Vec::new(),
                        // M2-W2: dual-bar quests (TLC_817) mirror their
                        // second bar in the component state.
                        second: qdef
                            .second
                            .map(|s| crate::core::component::QuestSecondState {
                                progress: 0,
                                target: s.target,
                                markers: Vec::new(),
                            }),
                    },
                );
            } else if card_type == Some(CardType::Spell) {
                // A played spell leaves the hand immediately (HS: the card is
                // in play while its effect resolves). The hand-size cap
                // (F-A11) counts it outside the hand, so a cast at 9 cards
                // can still fit generated copies / draws; the spell then
                // moves on to SetAside (secrets) or the graveyard below.
                state
                    .world_mut()
                    .move_to_zone(card, Zone::Play)
                    .expect("spell must leave the hand when played");
                // Secret cards: attach the Secret component and move to the
                // SetAside zone (revealed when the trigger condition is met).
                // Secret effects are stored in the battlecry slot (consistent
                // with the spell card convention).
                let secret_trigger = state
                    .world()
                    .card_id(card)
                    .and_then(|cid| crate::cards::def::card_by_id(cid.0))
                    .and_then(|def| def.secret);
                if let Some(trigger) = secret_trigger {
                    // Kirin Tor Mage's one-time free secret is consumed
                    let secret_effect = state.world().battlecry(card).map(|b| b.0);
                    let inner = state.make_mut();
                    inner.players[player.index()].next_secret_free = false;
                    // The secret is registered even when it has no effect
                    // (Counterspell — the reveal itself negates the spell)
                    inner.world.set_secret(
                        card,
                        Secret {
                            trigger,
                            effect: secret_effect,
                        },
                    );
                    state
                        .world_mut()
                        .move_to_zone(card, Zone::SetAside)
                        .map_err(|_| EngineError::EntityGone(card))?;
                    // Secret-played triggers (Secretkeeper — whenever a Secret
                    // is played; both players' secrets count)
                    fire_triggers(
                        state,
                        queue,
                        TriggerEvent::SecretPlayed,
                        player,
                        Some(card),
                        None,
                    );
                    // After-cast triggers fire at Lowest priority — after the
                    // spell's damage and the deaths it caused have resolved
                    // (HS: deaths process before "after you cast" triggers)
                    queue.push_with_priority(
                        Event::SpellCast {
                            player,
                            spell: card,
                            target,
                        },
                        Priority::Lowest,
                    );
                } else {
                    // Counter-secret interception (roadmap G8): WhenEnemySpellCast
                    // secrets fire BEFORE the spell's effect resolves.
                    let interception =
                        secret::intercept_counter_secrets(state, queue, card, player);
                    // The spell's effect as the play path would resolve it
                    // (combo-aware) — hoisted so the interception branches
                    // (and the counter-secret record below) share one shape.
                    let chosen_effect = if combo_active {
                        // Combo: prefer combo_effect
                        state
                            .world()
                            .combo_effect(card)
                            .map(|c| c.0)
                            .or_else(|| state.world().battlecry(card).map(|b| b.0))
                    } else {
                        state.world().battlecry(card).map(|b| b.0)
                    };
                    match interception {
                        secret::Interception::Countered => {
                            // The spell is negated but still cast and discarded
                            // After-cast triggers fire at Lowest priority — after the
                            // spell's damage and the deaths it caused have resolved
                            // (HS: deaths process before "after you cast" triggers)
                            queue.push_with_priority(
                                Event::SpellCast {
                                    player,
                                    spell: card,
                                    target,
                                },
                                Priority::Lowest,
                            );
                            state
                                .world_mut()
                                .move_to_zone(card, Zone::Graveyard)
                                .map_err(|_| EngineError::EntityGone(card))?;
                            // Rewind (M3-W1): the negated spell still records
                            // its would-be effect (it was played — the entry
                            // occupies a slot), but no replay fires: the
                            // counter negated the whole card.
                            crate::engine::rewind::record_play(state, card, player, chosen_effect);
                        }
                        secret::Interception::Spellbent(token) => {
                            // Spellbender: the spell's single-target effect is
                            // redirected to the 1/3 token
                            if let Some(effect) = chosen_effect {
                                trigger::resolve_effect(
                                    state,
                                    queue,
                                    card,
                                    player,
                                    effect,
                                    Some(token),
                                    None,
                                );
                            }
                            // After-cast triggers fire at Lowest priority — after the
                            // spell's damage and the deaths it caused have resolved
                            // (HS: deaths process before "after you cast" triggers)
                            queue.push_with_priority(
                                Event::SpellCast {
                                    player,
                                    spell: card,
                                    target,
                                },
                                Priority::Lowest,
                            );
                            state
                                .world_mut()
                                .move_to_zone(card, Zone::Graveyard)
                                .map_err(|_| EngineError::EntityGone(card))?;
                            // Rewind (M3-W1): the redirected effect was the
                            // play's resolution — record it; no replay fires
                            // (the redirect consumed the spell's own effect).
                            crate::engine::rewind::record_play(state, card, player, chosen_effect);
                        }
                        secret::Interception::None => {
                            // M4-W2 Herald (2025–2026 expansions): a
                            // Herald spell's keyword resolves BEFORE the
                            // spell's own effect — the official text
                            // order "Herald {0}. Deal $4 damage..." (CATA_
                            // 156/530/561/785). A countered / spellbent
                            // spell resolves nothing — the hook lives
                            // only in the un-intercepted arm (§24).
                            crate::cards::herald::resolve_herald(state, queue, card, player);
                            // Choose One always surfaces the branch choice in
                            // real Hearthstone, regardless of cards played
                            // earlier this turn (the old `!combo_active` guard
                            // wrongly resolved the battlecry branch when the
                            // spell was played after another card — Core Set
                            // W6 fidelity fix, pinned by w6_*_choose_one).
                            if state.world().choose_one_effect(card).is_some() {
                                // Choose One spell (roadmap G6): the branch choice
                                // surfaces as a pending choice; the effect, the
                                // SpellCast event, and the graveyard move resolve in
                                // ChoiceResolved. The default policy
                                // (GameEngine::apply) resolves it randomly via the
                                // embedded RNG, preserving the historical behavior.
                                // The option labels come from the cards-side table
                                // (M1-W3, P3 real choice resolution; M2-W4a appends
                                // the third branch of the Un'Goro beasts).
                                let labels = choose_one_labels_all(state, card);
                                let repeat = choose_one_repeat(state, card);
                                state.set_pending_choice_repeat(
                                    ChoiceKind::ChooseOne,
                                    card,
                                    labels.iter().map(|l| l.to_string()).collect(),
                                    Vec::new(),
                                    false,
                                    repeat,
                                );
                            } else {
                                // Spell card: resolve the effect (combo-aware),
                                // then move to the graveyard (`chosen_effect`
                                // is the hoisted combo-aware effect above).
                                // Combo bounce-back (Headcrack): the card stays in hand after the effect resolves instead of going to the graveyard
                                let returns_to_hand = matches!(
                                    chosen_effect,
                                    Some(crate::core::effect::CardEffect::DealDamageAndReturnToHand { .. })
                                );
                                if let Some(effect) = chosen_effect {
                                    trigger::resolve_effect(
                                        state, queue, card, player, effect, target, None,
                                    );
                                }
                                // Tyrande (M1-W4b): "the next 3 spells you cast
                                // cast twice" — the second resolution uses no
                                // explicit target and fires no second SpellCast
                                // event (registered timing simplification, §14.4).
                                if state.player(player).spells_cast_twice_pending > 0 {
                                    state.make_mut().players[player.index()]
                                        .spells_cast_twice_pending -= 1;
                                    if let Some(effect) = chosen_effect {
                                        trigger::resolve_effect(
                                            state, queue, card, player, effect, None, None,
                                        );
                                    }
                                }
                                // M4-W1 (CATA_154 Sinestra — 2025–2026
                                // expansions): "Your spells from other classes
                                // cast twice." A friendly Sinestra on the board
                                // makes this spell's effect re-resolve once —
                                // the Tyrande convention (no explicit target,
                                // no second SpellCast event). The class filter
                                // is dropped: the engine has no per-player or
                                // per-card class concept (fidelity-debt §23).
                                if state.world().zones().iter(Zone::Play, player).any(|e| {
                                    state.world().card_id(e).is_some_and(|c| c.0 == "CATA_154")
                                }) {
                                    if let Some(effect) = chosen_effect {
                                        trigger::resolve_effect(
                                            state, queue, card, player, effect, None, None,
                                        );
                                    }
                                }
                                // Kindred (M2-W3): a Kindred spell's OnPlay
                                // effect resolves after the base spell effect
                                // (and the Tyrande double), before the
                                // SpellCast event — the played-type push
                                // happened earlier in this handler, so the
                                // activation check counts this spell plus any
                                // earlier same-type card (>= 2).
                                crate::cards::kindred::resolve_on_play(state, queue, card, player);
                                // Rewind (M3-W1 — the Across the Timeways
                                // rewind primitive): the spell's own effect
                                // (and the Tyrande double) resolved above. A
                                // Rewind card now replays the effects recorded
                                // BEFORE this play, then its own entry is
                                // recorded — the card never replays itself.
                                crate::engine::rewind::hook_after_play(
                                    state,
                                    queue,
                                    card,
                                    player,
                                    chosen_effect,
                                );
                                // Push the spell-cast event
                                // After-cast triggers fire at Lowest priority — after the
                                // spell's damage and the deaths it caused have resolved
                                // (HS: deaths process before "after you cast" triggers)
                                queue.push_with_priority(
                                    Event::SpellCast {
                                        player,
                                        spell: card,
                                        target,
                                    },
                                    Priority::Lowest,
                                );
                                if returns_to_hand {
                                    // Headcrack bounce-back: the spell went to
                                    // Zone::Play at cast time (F-A11 hand cap)
                                    // and returns to the hand here.
                                    state
                                        .world_mut()
                                        .move_to_zone(card, Zone::Hand)
                                        .map_err(|_| EngineError::EntityGone(card))?;
                                } else {
                                    state
                                        .world_mut()
                                        .move_to_zone(card, Zone::Graveyard)
                                        .map_err(|_| EngineError::EntityGone(card))?;
                                }
                            }
                        }
                    }
                }
            } else if card_type == Some(CardType::Weapon) {
                // Weapon card: destroy the old weapon first, then equip the new one
                let old_weapon = state.player(player).weapon;
                if let Some(old) = old_weapon {
                    let inner = state.make_mut();
                    inner.players[player.index()].weapon = None;
                    queue.push(Event::WeaponDestroyed {
                        player,
                        weapon: old,
                    });
                }
                // Equip the new weapon: move the card to the battlefield as a weapon entity
                state
                    .world_mut()
                    .move_to_zone(card, Zone::Play)
                    .map_err(|_| EngineError::EntityGone(card))?;
                let inner = state.make_mut();
                inner.players[player.index()].weapon = Some(card);
                queue.push(Event::WeaponEquipped {
                    player,
                    weapon: card,
                });
                // M4-W2 Herald (2025–2026 expansions): the weapon's
                // "Battlecry: Herald {0}." (CATA_580 Cataclysmic War Axe)
                // resolves right after equipping, before any battlecry.
                crate::cards::herald::resolve_herald(state, queue, card, player);
                // Choose One weapons (Barbed Thorn, M1-W3): the branch choice
                // surfaces as a pending choice after equipping — ChoiceResolved
                // resolves the chosen branch (same pattern as spells/minions).
                if state.world().choose_one_effect(card).is_some() {
                    let labels = choose_one_labels_all(state, card);
                    let repeat = choose_one_repeat(state, card);
                    state.set_pending_choice_repeat(
                        ChoiceKind::ChooseOne,
                        card,
                        labels.iter().map(|l| l.to_string()).collect(),
                        Vec::new(),
                        false,
                        repeat,
                    );
                } else {
                    // Weapon battlecry (combo-aware): resolved after equipping, e.g. Perdition's Blade
                    let weapon_effect = if combo_active {
                        state
                            .world()
                            .combo_effect(card)
                            .map(|c| c.0)
                            .or_else(|| state.world().battlecry(card).map(|b| b.0))
                    } else {
                        state.world().battlecry(card).map(|b| b.0)
                    };
                    if let Some(effect) = weapon_effect {
                        trigger::resolve_effect(state, queue, card, player, effect, None, None);
                        // M3-W2b — Deios' "Battlecries trigger twice" aura
                        // also doubles weapon battlecries (§21).
                        if deios_doubling(state, player) {
                            trigger::resolve_effect(state, queue, card, player, effect, None, None);
                        }
                    }
                    // Rewind (M3-W1): the weapon's battlecry resolved above —
                    // the rewind replay + history record follow (the spell
                    // path's ordering).
                    crate::engine::rewind::hook_after_play(
                        state,
                        queue,
                        card,
                        player,
                        weapon_effect,
                    );
                }
            } else if card_type == Some(CardType::Location) {
                // Location card (Core Set W8): replace the current location
                // (one per side), enter the board, start the play cooldown.
                let old = state.player(player).location;
                if let Some(old_loc) = old {
                    let inner = state.make_mut();
                    inner.players[player.index()].location = None;
                    let _ = state.world_mut().move_to_zone(old_loc, Zone::Graveyard);
                }
                state
                    .world_mut()
                    .move_to_zone(card, Zone::Play)
                    .map_err(|_| EngineError::EntityGone(card))?;
                let turn = state.turn();
                let inner = state.make_mut();
                inner.players[player.index()].location = Some(card);
                inner.players[player.index()].location_played_turn = turn;
                // Rewind (M3-W1): a location play records no effect — its
                // battlecry is the ACTIVATE effect (LocationActivated),
                // never resolved by the play — but the play still occupies
                // a history slot.
                crate::engine::rewind::record_play(state, card, player, None);
                // Card-played triggers fire in the shared section below
                // (same as every other card type).
            } else if card_type == Some(CardType::Hero) {
                // Hero card (Core Set W8 — Lord Jaraxxus; 2025–2026
                // expansions M4-W4 — Deathwing, Worldbreaker): replace the
                // hero through the shared ReplaceHero primitive — the old
                // hero entity moves to the graveyard, the played entity
                // becomes the new hero (def health, damage and attack
                // cleared, hero power and armor from the cards-side table,
                // equipped weapon destroyed), then the hero's battlecry
                // resolves (M4-W4 — Deathwing's "Choose a Cataclysm to
                // unleash!").
                let Some(cid) = state.world().card_id(card).map(|c| c.0) else {
                    return Err(EngineError::EntityGone(card));
                };
                trigger::resolve_effect(
                    state,
                    queue,
                    card,
                    player,
                    crate::core::effect::CardEffect::ReplaceHero { card_id: cid },
                    None,
                    None,
                );
                if let Some(def) = crate::cards::def::card_by_id(cid) {
                    if let Some(bc) = def.battlecry {
                        trigger::resolve_effect(state, queue, card, player, bc, None, None);
                    }
                }
            } else {
                // Move the card from hand to the battlefield (summon position,
                // roadmap G6 — 0 = leftmost)
                state
                    .world_mut()
                    .move_to_zone(card, Zone::Play)
                    .map_err(|_| EngineError::EntityGone(card))?;
                if let Some(position) = position {
                    let world = state.world_mut();
                    world.zones_mut().remove(Zone::Play, player, card);
                    world
                        .zones_mut()
                        .insert_at(Zone::Play, player, card, position as usize);
                }
                // M4-W4 — CATA_493 Duke of Below: the discard-scaled
                // self-aura bakes at play with the current count.
                if state
                    .world()
                    .card_id(card)
                    .is_some_and(|c| c.0 == "CATA_493")
                {
                    crate::cards::bake_duke_of_below(state, card, player);
                }
                // Rewind (M3-W1): mark this minion as the one being played —
                // its MinionSummoned event (also enqueued for effect summons)
                // uses the marker to run the rewind replay + history record
                // after the battlecry.
                state.make_mut().players[player.index()].rewind_played_minion = Some(card);
                // Choose One minions (Cenarius, Keeper of the Grove): the branch
                // choice surfaces as a pending choice — MinionSummoned skips the
                // battlecry and ChoiceResolved resolves the chosen branch (G6).
                // The option labels come from the cards-side table (M1-W3, P3).
                if state.world().choose_one_effect(card).is_some() {
                    let labels = choose_one_labels_all(state, card);
                    let repeat = choose_one_repeat(state, card);
                    state.set_pending_choice_repeat(
                        ChoiceKind::ChooseOne,
                        card,
                        labels.iter().map(|l| l.to_string()).collect(),
                        Vec::new(),
                        false,
                        repeat,
                    );
                }
                // Kindred (M2-W3): a Kindred minion's OnPlay effect resolves
                // after the card enters play (its keywords and the base
                // resolution are done; the enqueued MinionSummoned event —
                // the battlecry — processes later). None of the OnPlay
                // kindred effects interact with their cards' battlecries
                // (Hot Spring Glider's battlecry sets the discount flag, its
                // kindred the Divine Shield flag — independent), so the
                // ordering is unobservable (§16). The played-type push
                // happened earlier in this handler — the check counts the
                // card itself plus any earlier same-type card (>= 2).
                crate::cards::kindred::resolve_on_play(state, queue, card, player);
                // M3-W2a — minions that enter play Dormant (Cyborg
                // Patriarch, Timelord Nozdormu): the dormant component is
                // applied right after the card enters the battlefield, so
                // the battlecry resolution below (MinionSummoned) skips
                // nothing — the card's own battlecry resolves normally
                // (none of the dormant-at-summon cards carry one), and the
                // countdown starts at the next turn start. The aura index
                // is maintained by `set_dormant`.
                if let Some(turns) = state
                    .world()
                    .card_id(card)
                    .and_then(|cid| crate::cards::dormant_at_summon(cid.0))
                {
                    state
                        .world_mut()
                        .set_dormant(card, crate::core::component::Dormant { turns });
                }
                // M3-W2a — the played-this-turn marker (TIME_620 Timecode
                // the End's secret predicate): set when a minion is played
                // from the hand, cleared at the owner's turn start.
                state
                    .world_mut()
                    .set_played_this_turn(card, crate::core::component::PlayedThisTurn);
                // Quest progress (M2-W1): minion plays feed the Un'Goro quest
                // conditions — TLC_229 (unique types: the primary race as the
                // marker, 0 for race-less minions), TLC_830 (Beast attack
                // values at play time), TLC_239 (full-board turns — fires
                // only when the board is full after the play; W2 pins the
                // official timing).
                let races = state.world().race(card).unwrap_or_default();
                let type_marker = races.first().map_or(0, |r| *r as u32 + 1);
                crate::engine::quest::progress(
                    state,
                    queue,
                    player,
                    crate::cards::quest::QuestCondition::PlayMinionsOfUniqueTypes,
                    1,
                    Some(type_marker),
                );
                if state
                    .world()
                    .has_race(card, crate::core::component::Race::Beast)
                {
                    let attack = state
                        .world()
                        .effective_attack(card)
                        .map_or(0, |a| a.0 as u32);
                    crate::engine::quest::progress(
                        state,
                        queue,
                        player,
                        crate::cards::quest::QuestCondition::PlayBeastsOfAttack,
                        1,
                        Some(attack),
                    );
                }
                // The sidequest (M2-W4a): TLC_EVENT_400 — "Play 3 Beasts
                // or Undead" — a played minion of either race counts.
                if state
                    .world()
                    .has_race(card, crate::core::component::Race::Beast)
                    || state
                        .world()
                        .has_race(card, crate::core::component::Race::Undead)
                {
                    crate::engine::quest::progress(
                        state,
                        queue,
                        player,
                        crate::cards::quest::QuestCondition::PlayBeastsOrUndead,
                        1,
                        None,
                    );
                }
                if count_board_minions(state.world(), player) == MAX_BOARD_SIZE {
                    crate::engine::quest::progress(
                        state,
                        queue,
                        player,
                        crate::cards::quest::QuestCondition::FillBoardOnTurns,
                        1,
                        Some(state.turn()),
                    );
                }
            }
            // Overload (roadmap F1): playing an overload card locks mana for the
            // owner's next turn (applied at the ManaRefill step)
            if let Some(overload) = state.world().overload(card) {
                let inner = state.make_mut();
                inner.players[player.index()].overload_locked += overload.0;
                // M3-W3 — Haywire Hornswog (END_030) counts the Mana
                // Crystals Overloaded this GAME: the game-long counter
                // grows alongside the per-turn lock (the effect-based
                // overload of Winged Aberration END_032 bumps it in its
                // own resolution arm).
                inner.players[player.index()].overload_total += overload.0;
                // Registered FriendlyOverloadPlayed triggers fire (Unbound Elemental)
                fire_triggers(
                    state,
                    queue,
                    TriggerEvent::FriendlyOverloadPlayed,
                    player,
                    None,
                    None,
                );
            }
            // M3-W2b — Chrono-Lord Epoch's tracking: every minion play is
            // recorded in the per-turn list (the TurnEnded handler
            // snapshots it into last_turn_minion_play_ids).
            if let Some(cid) = played_minion_id {
                state.make_mut().players[player.index()]
                    .minions_played_this_turn_ids
                    .push(cid.to_string());
            }
            // M3-W2b — Timelooper Toki: playing one of the three tracked
            // "random spells from the past" removes it from the pending
            // list; when the list empties (all three played), a fresh
            // Toki joins the hand.
            {
                let cid = state.world().card_id(card).map(|c| c.0.to_string());
                if let Some(cid) = cid {
                    let emptied = {
                        let p = &mut state.make_mut().players[player.index()];
                        if let Some(pos) = p.toki_pending_spells.iter().position(|s| *s == cid) {
                            p.toki_pending_spells.remove(pos);
                            p.toki_pending_spells.is_empty()
                        } else {
                            false
                        }
                    };
                    if emptied {
                        if let Some(def) = crate::cards::def::card_by_id("TIME_861") {
                            trigger::add_card_to_hand(state, player, def);
                        }
                    }
                }
            }
            // M3-W2b — The Eternal Hold's one-time "your next Demon costs
            // (1)" flag: the discount was already applied by play_cost; the
            // flag is consumed by the next Demon played (any cost path).
            if state
                .world()
                .has_race(card, crate::core::component::Race::Demon)
            {
                state.make_mut().players[player.index()].next_demon_cost_one = false;
            }
            // M3-W3 — Battle at the End Time (END_017): "Fill your hand,
            // then empty it" — the emptied half fires when a play empties
            // the hand (the filled half fires at the draw/get funnels in
            // trigger.rs; the two markers dedup the sequence, §22).
            if state.world().zones().len(Zone::Hand, player) == 0 {
                crate::engine::quest::progress(
                    state,
                    queue,
                    player,
                    crate::cards::quest::QuestCondition::FillThenEmptyHand,
                    1,
                    Some(1),
                );
            }
            // Card-played triggers (Questing Adventurer — whenever YOU play a
            // card): fire after the card fully resolved (effect included).
            fire_triggers(
                state,
                queue,
                TriggerEvent::CardPlayed,
                player,
                Some(card),
                None,
            );
        }
        Event::MinionSummoned {
            player,
            minion,
            target,
        } => {
            // Quest progress (M2-W1): TLC_426 — "Summon 6 Murlocs" — fires
            // for every race of the summoned minion (progress() matches the
            // quest's exact race, so a race-less minion fires nothing). The
            // MinionSummoned event is the single funnel for ALL summons —
            // played minions route through it too (via enqueue), so played
            // Murlocs count, matching Hearthstone.
            let minion_races: Vec<crate::core::component::Race> = state
                .world()
                .race(minion)
                .map_or_else(Vec::new, |r| r.to_vec());
            for race in minion_races {
                crate::engine::quest::progress(
                    state,
                    queue,
                    player,
                    crate::cards::quest::QuestCondition::SummonMinionsOfRace { race },
                    1,
                    None,
                );
            }
            // M2-W2 (TLC_426 — Dive the Golakka Depths): the repeatable
            // quest's permanent "Murlocs you summon gain +1/+1". The flag
            // is set by the quest reward; every friendly Murloc summon
            // while it is active arrives with a +1/+1 enchantment (the
            // MinionSummoned event is the single funnel for ALL summons).
            if state.player(player).murloc_summon_buff
                && state
                    .world()
                    .has_race(minion, crate::core::component::Race::Murloc)
            {
                state.world_mut().add_enchantment(
                    minion,
                    crate::core::component::Enchantment {
                        attack: 1,
                        health: 1,
                        cost: 0,
                        expiry: crate::core::component::EnchantmentExpiry::Permanent,
                    },
                );
            }
            // Summoning sickness: minions without charge cannot attack this
            // turn (a Charge aura — Tundra Rhino — counts as charge). Rush
            // (Core Set W1) also skips sickness — it can attack enemy MINIONS
            // the turn it arrives — but is tracked by SummonedThisTurn so the
            // hero-attack ban (validate_attack) can be enforced until the
            // next turn.
            if !state.world().effective_charge(minion) {
                state
                    .world_mut()
                    .set_summoned_this_turn(minion, crate::core::component::SummonedThisTurn);
                if state.world().rush(minion).is_none() {
                    state.world_mut().set_attacks_used(minion, AttacksUsed(1));
                }
            }
            // Rewind (M3-W1): whether this MinionSummoned is a PLAYED minion
            // (the marker set in the CardPlayed path — effect summons never
            // record). The marker is consumed unconditionally: a played
            // minion's summon always processes it (Choose One plays skip the
            // hook but clear the marker here).
            let played_minion = state.player(player).rewind_played_minion == Some(minion);
            if played_minion {
                state.make_mut().players[player.index()].rewind_played_minion = None;
            }
            // M5-W1 — Zee's Might (JAIL_800hp2, the Mug'Zee hero power):
            // "Every 5th minion you play triggers its Battlecry twice."
            // The CardPlayed path armed the readiness flag for the fifth
            // played minion; this summon consumes it (unconditionally,
            // like the rewind marker — a Choose One fifth minion's
            // battlecry resolves through the choice system and is not
            // doubled, the corner registered in §27), and the battlecry
            // re-resolve fires below after Deios's.
            let zee_might_ready = played_minion && state.player(player).zee_might_ready;
            if zee_might_ready {
                state.make_mut().players[player.index()].zee_might_ready = false;
            }
            // Check battlecry component (combo-aware). Choose One minions
            // resolve their branch through the choice system (roadmap G6).
            if state.world().choose_one_effect(minion).is_none() {
                let combo_active = state.player(player).cards_played_this_turn > 1;
                let chosen_effect = if combo_active {
                    state
                        .world()
                        .combo_effect(minion)
                        .map(|c| c.0)
                        .or_else(|| state.world().battlecry(minion).map(|b| b.0))
                } else {
                    state.world().battlecry(minion).map(|b| b.0)
                };
                // M4-W2 Herald (2025–2026 expansions): a Herald minion's
                // battlecry IS the Herald keyword ("Battlecry: Herald
                // {0}." — CATA_160/525/565/722/725/780). The CardDefs
                // carry no battlecry; the hook (keyed by the herald
                // registry) increments the counter, summons the class
                // Soldier and applies the source-keyed add-ons. Sitting
                // in the battlecry conditions, an effect-summoned copy
                // Heralds too — the official "Battlecry:" text (§24);
                // Deios / BattlecryTwice do not double it (no battlecry
                // component).
                crate::cards::herald::resolve_herald(state, queue, minion, player);
                if let Some(effect) = chosen_effect {
                    // Kindred battlecry modifiers (M2-W3): the modifier
                    // either replaces the battlecry (TLC_454/463/829 —
                    // a different target set or a different hand) or adds
                    // an effect after it (TLC_482). The activation check
                    // runs against the played list (>= 2 — the card's own
                    // push happened at the CardPlayed path). The
                    // replacement effect re-resolves once under the
                    // BattlecryTwice dark gift, like a base battlecry.
                    let (effect, kindred_extra) = crate::cards::kindred::apply_battlecry_modifier(
                        state, minion, player, effect,
                    );
                    // Explicit battlecry target (engine-mechanics roadmap M1):
                    // forwarded from Action::PlayCard; re-validation stays G9 —
                    // a target that left the legal candidate set fizzles.
                    trigger::resolve_effect(state, queue, minion, player, effect, target, None);
                    if let Some(kindred_extra) = kindred_extra {
                        trigger::resolve_effect(
                            state,
                            queue,
                            minion,
                            player,
                            kindred_extra,
                            target,
                            None,
                        );
                    }
                    // Dark gift 6 (BattlecryTwice — 2025–2026 expansions
                    // M1-W2): "this minion's battlecries trigger twice" — the
                    // same effect re-resolves once for a minion carrying the
                    // gift (registered simplification: the official "only if
                    // it has a battlecry" filter is not applied — fidelity-debt
                    // §14). The re-resolution reads the pre-captured effect,
                    // so a battlecry that buffs or kills the minion mid-way
                    // still replays once, as in Hearthstone.
                    if state
                        .world()
                        .has_dark_gift(minion, DarkGiftKind::BattlecryTwice)
                    {
                        trigger::resolve_effect(state, queue, minion, player, effect, target, None);
                    }
                    // M3-W2b — TIME_064 Deios: "Your Battlecries trigger
                    // twice" — one re-resolve of the pre-captured effect,
                    // the same convention as the dark gift above (the two
                    // together resolve 3 times, never 4, §21).
                    if deios_doubling(state, player) {
                        trigger::resolve_effect(state, queue, minion, player, effect, target, None);
                    }
                    // M5-W1 — Zee's Might (see the consumption above): one
                    // more re-resolve of the pre-captured effect.
                    if zee_might_ready {
                        trigger::resolve_effect(state, queue, minion, player, effect, target, None);
                    }
                }
                // Rewind (M3-W1): the battlecry (and its re-resolutions)
                // resolved above — a played Rewind minion now replays the
                // effects recorded before this play, then its own entry is
                // recorded. Choose One minions skip both (their effect
                // resolves later via the choice system — unknowable here).
                if played_minion {
                    crate::engine::rewind::hook_after_play(
                        state,
                        queue,
                        minion,
                        player,
                        chosen_effect,
                    );
                }
            }
            // M4-W1 Colossal (2025–2026 expansions): a Colossal minion
            // played from hand summons its appendages AFTER its battlecry
            // resolves (Hearthstone order — the battlecry block above ran
            // first). The played-minion gate keeps effect summons
            // (resurrections, copies) from bringing parts along.
            if played_minion {
                crate::cards::colossal::summon_colossal_parts(state, queue, minion, player);
            }
            // Summon triggers: registered FriendlyMinionSummoned triggers fire in
            // play order (the summoned minion itself is excluded). The summoned
            // minion is the event subject — Sword of Justice buffs it via
            // EffectTarget::EventSubject.
            fire_triggers(
                state,
                queue,
                TriggerEvent::FriendlyMinionSummoned,
                player,
                Some(minion),
                Some(minion),
            );
            // Dark gift 5 (SummonCopyOnPlay — 2025–2026 expansions M1-W2):
            // "when you play this, summon a 2/2 copy of it". The copy is a
            // fresh entity from the base card definition (it carries no gifts
            // and no enchantments — a copied card is a plain card), forced to
            // 2/2 (registered: the engine-convention battlecry of effect
            // summons fires for the copy too — fidelity-debt §14).
            if state
                .world()
                .has_dark_gift(minion, DarkGiftKind::SummonCopyOnPlay)
            {
                if let Some(card_id) = state.world().card_id(minion) {
                    if let Some(copy) =
                        trigger::resolve_summon(state, queue, minion, player, card_id.0)
                    {
                        let world = state.world_mut();
                        world.set_attack(copy, Attack(2));
                        world.set_health(copy, Health(2));
                        world.remove_enchantments(copy);
                        world.remove_damage(copy);
                    }
                }
            }
        }
        Event::AttackDeclared { attacker, defender } => {
            // Lake Thresher (Core Set W3b): also damages the minions next
            // to whomever this attacks (the defender's board neighbors;
            // resolved here because the trigger effects only see the
            // attacker)
            if state
                .world()
                .card_id(attacker)
                .is_some_and(|c| c.0 == "CORE_SCH_605")
                && state.world().card_type(defender) == Some(CardType::Minion)
            {
                let atk = state
                    .world()
                    .effective_attack(attacker)
                    .unwrap_or(Attack(0))
                    .0;
                if atk > 0 {
                    if let Some(defender_player) = state.world().player(defender) {
                        let minions: Vec<Entity> = state
                            .world()
                            .zones()
                            .iter(Zone::Play, defender_player)
                            .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                            .collect();
                        if let Some(pos) = minions.iter().position(|&e| e == defender) {
                            if pos > 0 {
                                queue.push(Event::DamageDealt {
                                    source: attacker,
                                    target: minions[pos - 1],
                                    amount: atk,
                                });
                            }
                            if pos + 1 < minions.len() {
                                queue.push(Event::DamageDealt {
                                    source: attacker,
                                    target: minions[pos + 1],
                                    amount: atk,
                                });
                            }
                        }
                    }
                }
            }
            // Mayor Noggenfogger (Core Set W3a): all targets are chosen
            // randomly — the declared defender is replaced by a random enemy
            // character (hero or minion) when a Noggenfogger is on the
            // attacking player's board.
            let defender = if state
                .world()
                .zones()
                .iter(
                    Zone::Play,
                    state
                        .world()
                        .player(attacker)
                        .unwrap_or(state.active_player()),
                )
                .any(|e| {
                    state
                        .world()
                        .card_id(e)
                        .is_some_and(|c| c.0 == "CORE_CFM_670")
                }) {
                let attacker_player = state
                    .world()
                    .player(attacker)
                    .unwrap_or(state.active_player());
                let enemies: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Play, attacker_player.opponent())
                    .collect();
                if enemies.is_empty() {
                    defender
                } else {
                    let idx = state.rng_mut().next_usize(enemies.len());
                    enemies[idx]
                }
            } else {
                defender
            };
            // Freeze check: frozen characters cannot attack
            if state.world().freeze(attacker).is_some() {
                return Err(EngineError::InvalidTarget);
            }

            // Attack triggers (Blessing of Wisdom — the buffed minion draws
            // when IT attacks): pinned to the declared attacker
            fire_triggers(
                state,
                queue,
                TriggerEvent::Attacked,
                state
                    .world()
                    .player(attacker)
                    .unwrap_or(state.active_player()),
                Some(attacker),
                None,
            );
            // FriendlyMinionAttacked (M2-W4b — Archaios's "after a friendly
            // minion attacks, set its Health to this minion's Health"): a
            // friendly-scoped attack event for MINION attackers only, with
            // the attacker as the subject. Unlike `Attacked` it is not
            // pinned to the attacker — the trigger rides another friendly
            // minion (Archaios), which `trigger_applies` allows via the
            // default friendly-scope arm.
            if state.world().card_type(attacker) == Some(CardType::Minion) {
                fire_triggers(
                    state,
                    queue,
                    TriggerEvent::FriendlyMinionAttacked,
                    state
                        .world()
                        .player(attacker)
                        .unwrap_or(state.active_player()),
                    Some(attacker),
                    None,
                );
            }
            // ThisMinionAttacked (Core Set W3c — Wrathspike Brute): the
            // defender minion fires when it is attacked
            if state.world().card_type(defender) == Some(CardType::Minion) {
                if let Some(defender_player) = state.world().player(defender) {
                    fire_triggers(
                        state,
                        queue,
                        TriggerEvent::ThisMinionAttacked,
                        defender_player,
                        Some(attacker),
                        None,
                    );
                }
            }
            // HeroAttacked (Core Set W3b — Hench-Clan Thug): the hero
            // attacking fires a friendly-scoped trigger
            if state.world().card_type(attacker) == Some(CardType::Hero) {
                fire_triggers(
                    state,
                    queue,
                    TriggerEvent::HeroAttacked,
                    state
                        .world()
                        .player(attacker)
                        .unwrap_or(state.active_player()),
                    Some(attacker),
                    None,
                );
            }
            // Minion-hit attack triggers (Gorehowl — the weapon loses 1
            // Attack when the hero attacks a minion)
            if state.world().card_type(defender) == Some(CardType::Minion) {
                fire_triggers(
                    state,
                    queue,
                    TriggerEvent::AttackedMinion,
                    state
                        .world()
                        .player(attacker)
                        .unwrap_or(state.active_player()),
                    Some(attacker),
                    None,
                );
                // HeroAttackedMinion — the defender is the subject, so
                // splash effects can exclude the attacked minion (Defiled
                // Spear, M1-W4a)
                if state.world().card_type(attacker) == Some(CardType::Hero) {
                    fire_triggers(
                        state,
                        queue,
                        TriggerEvent::HeroAttackedMinion,
                        state
                            .world()
                            .player(attacker)
                            .unwrap_or(state.active_player()),
                        Some(defender),
                        None,
                    );
                }
                // AttackedEnemyMinion — a friendly MINION attacking a
                // minion, with the DEFENDER as the subject (The Great
                // Dracorex, M2-W4c: the splash must exclude the attacked
                // minion; friendly scope, not pinned — the trigger rides
                // the attacking Dracorex)
                if state.world().card_type(attacker) == Some(CardType::Minion) {
                    fire_triggers(
                        state,
                        queue,
                        TriggerEvent::AttackedEnemyMinion,
                        state
                            .world()
                            .player(attacker)
                            .unwrap_or(state.active_player()),
                        Some(defender),
                        None,
                    );
                }
            }

            // Read attacker type and weapon info first (read-only borrow)
            let is_hero = state.world().card_type(attacker) == Some(CardType::Hero);
            let attacker_player = state.world().player(attacker);
            let weapon_info: Option<(PlayerId, Entity)> = if is_hero {
                attacker_player.and_then(|pid| state.player(pid).weapon.map(|w| (pid, w)))
            } else {
                None
            };

            // Mark the attacker as having used an attack
            {
                let world = state.world_mut();
                let used = world.attacks_used(attacker).unwrap_or(AttacksUsed(0));
                world.set_attacks_used(attacker, AttacksUsed(used.0 + 1));
                // Stealth (roadmap F2): attacking breaks the attacker's stealth
                world.remove_stealth(attacker);
            }
            // M4-W4 — CATA_469 (friendly-attack counter): every attack by a
            // friendly character bumps the attacker's side's game-long
            // counter (consulted by the Attack-scaled arm).
            if let Some(pid) = state.world().player(attacker) {
                state.make_mut().players[pid.index()].friendly_attacks_this_game += 1;
            }

            // Decrement weapon durability by 1 — except Gorehowl attacking a
            // minion: the attack loss (triggered above) replaces the
            // durability loss
            if let Some((player, weapon)) = weapon_info {
                let gorehowl_minion_hit = state
                    .world()
                    .card_id(weapon)
                    .is_some_and(|c| c.0 == crate::cards::classic_warrior::GOREHOWL_ID)
                    && state.world().card_type(defender) == Some(CardType::Minion);
                if !gorehowl_minion_hit {
                    let dur = state.world().durability(weapon).unwrap_or(Durability(0));
                    let new_dur = Durability(dur.0 - 1);
                    state.world_mut().set_durability(weapon, new_dur);
                    if new_dur.0 <= 0 {
                        // Destroy the weapon
                        let inner = state.make_mut();
                        inner.players[player.index()].weapon = None;
                        queue.push(Event::WeaponDestroyed { player, weapon });
                    }
                }
            }
        }
        Event::ResolveAttack {
            attacker,
            defender,
            attacker_damage,
            retaliation_immune,
        } => {
            // Stormbrewer (M2-W3): "Whenever this attacks, deal 3 damage to
            // the target first" — the strike is enqueued BEFORE the attack
            // damage, so the queue resolves it first (the Lake Thresher
            // precedent — resolved here because the trigger effects only see
            // the attacker, not the defender). No defender-type check: the
            // strike hits whatever this attacks, hero included.
            if state
                .world()
                .card_id(attacker)
                .is_some_and(|c| c.0 == "TLC_107")
            {
                queue.push(Event::DamageDealt {
                    source: attacker,
                    target: defender,
                    amount: 3,
                });
            }
            // M5-W1 — The Living Plague (JAIL_443): "Instead of damaging
            // heroes, shuffle that many Blights into their deck that deal
            // 2 when drawn" — the hero-damage redirection: a Plague attack
            // on a hero enqueues no damage; `attacker_damage` Blights
            // (JAIL_443t, a playable 1-Cost spell dealing 2 to its own
            // hero — the cast-when-drawn simplification, §27) shuffle into
            // the hero's deck instead, at random positions (the
            // shuffle-into-deck convention).
            if state
                .world()
                .card_id(attacker)
                .is_some_and(|c| c.0 == "JAIL_443")
                && state.world().card_type(defender) == Some(CardType::Hero)
            {
                if let Some(owner) = state.world().player(defender) {
                    if let Some(def) = crate::cards::def::card_by_id("JAIL_443t") {
                        for _ in 0..attacker_damage.max(0) {
                            let e =
                                crate::cards::spawn_card_from_def(state.world_mut(), owner, def);
                            state.world_mut().set_zone(e, Zone::Deck);
                            let deck_count = state.world().zones().len(Zone::Deck, owner);
                            let position = if deck_count > 0 {
                                state.rng_mut().next_usize(deck_count + 1)
                            } else {
                                0
                            };
                            state
                                .world_mut()
                                .zones_mut()
                                .insert_at(Zone::Deck, owner, e, position);
                        }
                    }
                }
            } else {
                queue.push(Event::DamageDealt {
                    source: attacker,
                    target: defender,
                    amount: attacker_damage,
                });
            }
            // Defender retaliation: attack is computed from the current state
            // at resolution time — after an attack is redirected by a secret,
            // the new defender's retaliation applies automatically, without
            // per-card special handling.
            if !retaliation_immune && state.world().card_type(defender) == Some(CardType::Minion) {
                let atk = state
                    .world()
                    .effective_attack(defender)
                    .unwrap_or(Attack(0));
                if atk.0 > 0 {
                    queue.push(Event::DamageDealt {
                        source: defender,
                        target: attacker,
                        amount: atk.0,
                    });
                }
            }
        }
        Event::DamageDealt {
            target,
            amount,
            source,
        } => {
            // Quest progress (M2-W1): TLC_631 — "Deal exactly 2 damage to an
            // enemy on your turn" — fires when the active player damages a
            // target owned by the opponent; the condition is built from the
            // event's actual amount, so progress() only matches a quest whose
            // exact amount equals the damage dealt (a 3-damage hit does not
            // progress an exact-2 quest). The "on your turn" guard is
            // implicit: the quest owner is the active player (W2 pins the
            // official timing, e.g. shield absorption).
            if state
                .world()
                .player(target)
                .is_some_and(|owner| owner == state.active_player().opponent())
            {
                let active = state.active_player();
                crate::engine::quest::progress(
                    state,
                    queue,
                    active,
                    crate::cards::quest::QuestCondition::DealExactDamage {
                        amount: amount as u32,
                    },
                    1,
                    None,
                );
            }
            // M4-W4 — spell-damage counters and the FriendlySpellDealtDamage
            // event: a spell source bumps the caster's per-turn counter
            // (CATA_483 Unstable Spellcaster's "if you dealt damage with a
            // spell this turn" battlecry) and fires the friendly-scoped
            // event the CATA_487 Raincaller trigger rides (timing mirrors
            // the quest progress above — the damage is dealt regardless of
            // shield absorption).
            if state.world().card_type(source) == Some(CardType::Spell) {
                if let Some(pid) = state.world().player(source) {
                    state.make_mut().players[pid.index()].spell_damage_dealt_this_turn += 1;
                    fire_triggers(
                        state,
                        queue,
                        TriggerEvent::FriendlySpellDealtDamage,
                        pid,
                        Some(source),
                        None,
                    );
                }
            }
            // Unified damage pipeline: immune → dormant → divine shield → armor → health → death check
            // Immune: damage is completely ignored (the attack is still consumed).
            // M4-W4 — CATA_613 Survivalist's Immune-while-alone aura is
            // folded in here (the alone condition is evaluated by the
            // helper at the consult).
            if state.world().immune(target).is_some() || state.world().immune_while_alone(target) {
                return Ok(());
            }
            // Dormant minions take no damage while asleep (M3-W2a).
            if state.world().dormant(target).is_some() {
                return Ok(());
            }
            // Tichondrius (Core Set W4b): a friendly Tichondrius makes the
            // hero Immune while it is on the board
            let card_type = state.world().card_type(target);
            if card_type == Some(CardType::Hero) {
                if let Some(pid) = state.world().player(target) {
                    let tichondrius = state.world().zones().iter(Zone::Play, pid).any(|e| {
                        state
                            .world()
                            .card_id(e)
                            .is_some_and(|c| c.0 == "CORE_CATA_001")
                    });
                    if tichondrius {
                        return Ok(());
                    }
                }
            }
            // M4-W1 (CATA_155 Arisen Onyxia — 2025–2026 expansions):
            // "When your hero would lose Health on your turn, gain that
            // much max Health instead." A damage-pipeline redirect: the
            // hero's owner must be the active player and control a
            // friendly Arisen Onyxia — the damage is fully converted to a
            // permanent max-Health enchantment (no damage component, so
            // nothing reaches armor/weapon absorption — §23).
            if card_type == Some(CardType::Hero) && amount > 0 {
                if let Some(pid) = state.world().player(target) {
                    if pid == state.active_player() {
                        let onyxia =
                            state.world().zones().iter(Zone::Play, pid).any(|e| {
                                state.world().card_id(e).is_some_and(|c| c.0 == "CATA_155")
                            });
                        if onyxia {
                            state.world_mut().add_enchantment(
                                target,
                                crate::core::component::Enchantment {
                                    attack: 0,
                                    health: amount,
                                    cost: 0,
                                    expiry: crate::core::component::EnchantmentExpiry::Permanent,
                                },
                            );
                            return Ok(());
                        }
                    }
                }
            }
            // Water Elemental (W12, D2 — damage-pipeline check): freeze any
            // character damaged by this minion. Applied before the divine
            // shield absorption — HS freezes even when the shield absorbs.
            // The Core Set W6 token (CORE_BT_072t, Deep Freeze's summon) is
            // the same card and carries the same hook.
            let is_water_elemental = state.world().card_id(source).is_some_and(|c| {
                c.0 == crate::cards::classic_mage::WATER_ELEMENTAL_ID || c.0 == "CORE_BT_072t"
            });
            if is_water_elemental {
                state
                    .world_mut()
                    .set_freeze(target, crate::core::component::Freeze);
            }
            // Goldrinn, the Great Wolf (M1-W4b): your Beasts deal double
            // damage — a damage-pipeline hook at the entry point (the aura
            // approximation is registered in fidelity-debt §14.4). Any
            // damage source owned by the Goldrinn player with the Beast race
            // doubles while EDR_480 is on that player's board (the shield
            // absorption below is amount-agnostic, so doubling before it is
            // safe).
            let goldrinn_doubling = state.world().player(source).is_some_and(|pid| {
                state
                    .world()
                    .has_race(source, crate::core::component::Race::Beast)
                    && state
                        .world()
                        .zones()
                        .iter(Zone::Play, pid)
                        .any(|e| state.world().card_id(e).is_some_and(|c| c.0 == "EDR_480"))
            });
            let mut amount = if goldrinn_doubling {
                amount * 2
            } else {
                amount
            };
            // Bralma Searstone (M2-W4b): "Your Elementals deal 1 extra
            // damage" — a damage-pipeline hook at the same entry point as
            // Goldrinn (the aura approximation, registered §18): any
            // damage source owned by the Bralma player carrying the
            // Elemental race deals +1 while TLC_228 is alive on that
            // player's board. Elemental minion ATTACKS are the official
            // scope (spell damage from Elementals — the sources are the
            // spell entities themselves, which carry no race, so they are
            // naturally excluded).
            if let Some(src_owner) = state.world().player(source) {
                if state
                    .world()
                    .has_race(source, crate::core::component::Race::Elemental)
                    && state
                        .world()
                        .zones()
                        .iter(Zone::Play, src_owner)
                        .any(|e| state.world().card_id(e).is_some_and(|c| c.0 == "TLC_228"))
                {
                    amount += 1;
                }
            }
            // M2-W2 (TLC_631t Gorishi Colossus): the battlecry's permanent
            // "whenever you deal exactly 2 damage to an enemy, deal 2 more".
            // The flag is set by the quest reward's battlecry (fired when
            // the reward-summoned Gorishi resolves); the hook reuses the
            // DealExactDamage quest call site above. The bonus is applied
            // in-place (the Goldrinn pattern above) — an extra DamageDealt
            // event would re-enter this handler and recurse. The check runs
            // on the amount after Goldrinn doubling: the amount the target
            // actually takes is what the "exactly 2" refers to.
            if let Some(src_owner) = state.world().player(source) {
                if state.player(src_owner).deal_exact_2_bonus
                    && amount == 2
                    && state
                        .world()
                        .player(target)
                        .is_some_and(|owner| owner != src_owner)
                {
                    amount += 2;
                }
            }
            // M3-W2a (TIME_060 Quantum Destabilizer): "This minion takes
            // double damage from all sources" — a marker the battlecry/
            // summon paths apply; the doubling runs at the same entry point
            // as Goldrinn (before the divine-shield absorption).
            if state.world().double_damage_taken(target).is_some() {
                amount *= 2;
            }
            // M4-W4 — CATA_208 Selfless Protector "takes one extra damage
            // from all sources": the marked minion takes +1 (a permanent
            // marker on the minion, cleared when it leaves play or dies —
            // the move strips components; applied AFTER the doubling, like
            // the official "+1 extra damage" stacking).
            if state.world().bonus_damage_taken(target).is_some() {
                amount += 1;
            }
            // Divine shield absorbs: if the target has a divine shield, remove it and zero the damage
            if state.world().divine_shield(target).is_some() {
                state.world_mut().remove_divine_shield(target);
                // DivineShieldLost (Core Set W3b — Highlord Fordragon):
                // fires for the shield's owner when a friendly minion's
                // shield breaks
                if let Some(owner) = state.world().player(target) {
                    fire_triggers(
                        state,
                        queue,
                        TriggerEvent::DivineShieldLost,
                        owner,
                        Some(target),
                        None,
                    );
                }
                return Ok(());
            }

            // Get the target's card type
            let card_type = state.world().card_type(target);

            // Bulwark of Azzinoth (Core Set W3c): whenever the hero would
            // take damage, the weapon loses 1 Durability instead
            if card_type == Some(CardType::Hero) {
                let target_player = state.world().player(target);
                if let Some(pid) = target_player {
                    if let Some(weapon) = state.player(pid).weapon {
                        if state
                            .world()
                            .card_id(weapon)
                            .is_some_and(|c| c.0 == "CORE_BT_781")
                        {
                            let dur = state.world().durability(weapon).unwrap_or(Durability(0));
                            let new_dur = Durability(dur.0 - 1);
                            state.world_mut().set_durability(weapon, new_dur);
                            if new_dur.0 <= 0 {
                                let inner = state.make_mut();
                                inner.players[pid.index()].weapon = None;
                                queue.push(Event::WeaponDestroyed {
                                    player: pid,
                                    weapon,
                                });
                            }
                            return Ok(());
                        }
                    }
                    let armor = state.player(pid).armor;
                    if armor > 0 {
                        let absorbed = amount.min(armor);
                        let remaining = amount - absorbed;
                        let inner = state.make_mut();
                        inner.players[pid.index()].armor -= absorbed;
                        if remaining <= 0 {
                            // All damage absorbed by armor — the damage was
                            // dealt, so Lifesteal still heals the full amount
                            resolve_lifesteal_heal(state, queue, source, amount);
                            return Ok(());
                        }
                        // Remaining damage continues through to health — accumulate
                        // it on the damage component (roadmap G4)
                        let current_health = state.world().effective_health(target);
                        let Some(_hp) = current_health else {
                            return Ok(());
                        };
                        let damage_old = state
                            .world()
                            .damage(target)
                            .unwrap_or(crate::core::component::Damage(0))
                            .0;
                        let new_damage = damage_old + remaining;
                        state
                            .world_mut()
                            .set_damage(target, crate::core::component::Damage(new_damage));
                        fire_emberroot_hook(state, queue, target);
                        fire_hero_damage_counters(state, target);
                        fire_warptooth_hook(state, target);
                        queue_death_events(state, queue, target, card_type);
                        return Ok(());
                    }
                }
            }

            // No armor or non-hero: deduct health directly via the damage component
            let Some(cur) = state.world().effective_health(target) else {
                // Target does not exist (dead or removed); skip
                return Ok(());
            };
            let damage_old = state
                .world()
                .damage(target)
                .unwrap_or(crate::core::component::Damage(0))
                .0;
            // cur + damage_old = base + Σ health enchantments + aura (undamaged total)
            let mut new_damage = damage_old + amount;
            // Poison: a poisonous source destroys minions it damages (divine shield already handled above)
            if state.world().poison(source).is_some()
                && card_type == Some(CardType::Minion)
                && amount > 0
            {
                new_damage = cur.0 + damage_old;
            }
            // Commanding Shout: minions' health cannot drop below 1 this turn
            if card_type == Some(CardType::Minion)
                && state
                    .world()
                    .player(target)
                    .is_some_and(|pid| state.player(pid).minion_min_health > 0)
            {
                let min_hp = state
                    .world()
                    .player(target)
                    .map(|pid| state.player(pid).minion_min_health)
                    .unwrap_or(0);
                new_damage = new_damage.min(cur.0 + damage_old - min_hp);
            }

            // Mutate state via CoW
            state
                .world_mut()
                .set_damage(target, crate::core::component::Damage(new_damage));

            // Emberroot Destroyer (M1-W5) — "Whenever your hero takes damage
            // on your turn" fires here, where the hero's health is actually
            // reduced (armor-absorbed damage does not count).
            fire_emberroot_hook(state, queue, target);

            // M3-W2a per-turn hero-damage counters (Devious Coyote
            // TIME_047's "enemy hero took damage this turn" discount;
            // Liferender TIME_614's "hero's Health changed this turn"
            // battlecry check). Same site as the Emberroot hook: only
            // real health loss counts (armor absorption does not).
            fire_hero_damage_counters(state, target);

            // M5-W1 — Warptooth (JAIL_421): the four-friendly-characters
            // damage count (see the hook above; same real-loss site).
            fire_warptooth_hook(state, target);

            // Lifesteal (Core Set W1): damage dealt by a Lifesteal source
            // heals the source's owner hero for the damage dealt. Weapon,
            // minion and spell damage all count; the divine-shield/immune
            // branches returned early (no damage was dealt), and the
            // fully-armor-absorbed hero branch healed above.
            resolve_lifesteal_heal(state, queue, source, amount);

            // Damage triggers (roadmap G2): "whenever this minion takes damage"
            // (Acolyte of Pain) and "whenever a friendly minion takes damage"
            // (Frothing Berserker, Armorsmith) fire after the damage is applied
            // and before the death check — matching HS (Acolyte draws even if it
            // dies to the same damage).
            if amount > 0 && card_type == Some(CardType::Minion) {
                if let Some(owner) = state.world().player(target) {
                    fire_triggers(
                        state,
                        queue,
                        TriggerEvent::ThisMinionDamaged,
                        owner,
                        Some(target),
                        None,
                    );
                    fire_triggers(
                        state,
                        queue,
                        TriggerEvent::FriendlyMinionDamaged,
                        owner,
                        None,
                        None,
                    );
                }
            }

            // Death check (using effective health to account for aura bonuses)
            queue_death_events(state, queue, target, card_type);
            // M3-W2a — SurvivedDamage: "after this survives damage"
            // (TIME_050 Sentient Hourglass's stat swap, TIME_055 Unknown
            // Voyager's transform) fires when the damage was applied and
            // the minion is still alive (effective health not dead — the
            // Primal Sabretooth kill-check convention; a heal-rescued
            // target never fired, matching the "survives" wording). The
            // trigger is pinned to the damaged minion by subject.
            if amount > 0 && card_type == Some(CardType::Minion) {
                if let Some(owner) = state.world().player(target) {
                    if state
                        .world()
                        .effective_health(target)
                        .is_some_and(|h| !h.is_dead())
                    {
                        fire_triggers(
                            state,
                            queue,
                            TriggerEvent::SurvivedDamage,
                            owner,
                            Some(target),
                            None,
                        );
                    }
                }
            }
            // Primal Sabretooth (M2-W4a): "After this attacks and kills a
            // minion, get a copy of it" — the kill is detected right here
            // (the damage pipeline's death check, where the source of the
            // killing damage is in scope): a dead enemy minion whose death
            // came from a Sabretooth attack is copied to the Sabretooth's
            // owner's hand. A heal-rescued target never fired this (the
            // death check returned early), matching HS.
            if card_type == Some(CardType::Minion)
                && state
                    .world()
                    .effective_health(target)
                    .is_some_and(|h| h.is_dead())
                && state
                    .world()
                    .card_id(source)
                    .is_some_and(|c| c.0 == "TLC_247")
            {
                if let Some(killer_owner) = state.world().player(source) {
                    trigger::copy_card_to_hand(state, target, killer_owner);
                }
            }
        }
        Event::MinionDied { minion } => {
            // Death batching (roadmap G3): re-check the health — a minion healed
            // above 0 before its death was processed survives (the event is a no-op).
            let effective_hp = state.world().effective_health(minion);
            if !effective_hp.is_some_and(|h| h.is_dead()) {
                return Ok(());
            }
            let owner = state.world().player(minion);

            // M3-W3 — Splintered Reality (END_009): "They gain +1/+1 for
            // each friendly Treant that died this game" — the game-long
            // counter grows on every friendly Treant death (the
            // died-this-game tracking, the Umbra precedent).
            if let Some(owner) = owner {
                if state
                    .world()
                    .card_id(minion)
                    .is_some_and(|c| c.0 == "END_009t")
                {
                    state.make_mut().players[owner.index()].treants_died_total += 1;
                }
            }

            // Reborn (Core Set W1): the first death resurrects the minion as
            // a fresh 1/1 instead of dying — all buffs cleared, base stats
            // 1/1, Reborn spent, summoning sickness applied (a Rush/Charge
            // minion keeps its attack availability and can attack minions
            // again). The resurrection counts as a summon: on-summon
            // triggers fire (Sword of Justice), battlecries do NOT re-fire.
            if state.world().reborn(minion).is_some() {
                // A Reborn death still counts as a death for corpse purposes
                // (Core Set W1 — the Death-Knight corpse resource; the
                // resurrection itself produces none beyond this one).
                if let Some(owner) = owner {
                    let inner = state.make_mut();
                    inner.players[owner.index()].corpses += 1;
                }
                state.world_mut().remove_reborn(minion);
                if state
                    .world()
                    .has_dark_gift(minion, DarkGiftKind::RebornFull)
                {
                    // Dark gift 8 (RebornFull — 2025–2026 expansions M1-W2):
                    // the reborn minion keeps its enchantments (buffs and
                    // debuffs alike) and returns at full health — only the
                    // accumulated damage is cleared, no stat reset.
                    state.world_mut().remove_damage(minion);
                } else {
                    let world = state.world_mut();
                    world.remove_enchantments(minion);
                    world.set_attack(minion, Attack(1));
                    world.set_health(minion, Health(1));
                    world.remove_damage(minion);
                }
                if !state.world().effective_charge(minion) {
                    state
                        .world_mut()
                        .set_summoned_this_turn(minion, crate::core::component::SummonedThisTurn);
                    if state.world().rush(minion).is_none() {
                        state.world_mut().set_attacks_used(minion, AttacksUsed(1));
                    }
                }
                if let Some(owner) = owner {
                    fire_triggers(
                        state,
                        queue,
                        TriggerEvent::FriendlyMinionSummoned,
                        owner,
                        Some(minion),
                        Some(minion),
                    );
                }
                return Ok(());
            }

            // Move the minion to the graveyard FIRST (entity and components kept
            // for replay and graveyard effects) — death triggers must see the
            // dead minion already removed from the battlefield.
            state
                .world_mut()
                .move_to_zone(minion, Zone::Graveyard)
                .map_err(|_| EngineError::EntityGone(minion))?;

            // M4-W2 Herald (2025–2026 expansions): CATA_158 Maniacal
            // Follower's "Deathrattle: Herald {0}." — the dead minion's
            // card id stays readable in the graveyard; the hook is
            // id-keyed (the CardDef carries no deathrattle for this
            // card, §24).
            if let Some(owner) = owner {
                crate::cards::herald::resolve_herald(state, queue, minion, owner);
            }
            // Deathrattle effect (the entity is in the graveyard, as in HS)
            if let (Some(dr), Some(owner)) = (state.world().deathrattle(minion), owner) {
                trigger::resolve_effect(state, queue, minion, owner, dr.0, None, None);
                // M3-W2b — Deios' doubling covers deathrattles too (§21).
                if deios_doubling(state, owner) {
                    trigger::resolve_effect(state, queue, minion, owner, dr.0, None, None);
                }
            }

            // Death triggers: registered FriendlyMinionDied triggers fire in play
            // order (the dead minion itself is excluded). The dead minion is the
            // event subject — race-conditioned triggers (Scavenging Hyena — a
            // friendly Beast died) check its race.
            if let Some(owner) = owner {
                fire_triggers(
                    state,
                    queue,
                    TriggerEvent::FriendlyMinionDied,
                    owner,
                    Some(minion),
                    Some(minion),
                );
                // Any-minion-died triggers (Flesheathing Ghoul) fire for both
                // players' deaths
                fire_triggers(
                    state,
                    queue,
                    TriggerEvent::MinionDied,
                    owner,
                    Some(minion),
                    None,
                );
            }

            // Record the death for resurrection effects; the owner also
            // gains a corpse (Core Set W1 — Malignant Horror's end-of-turn
            // effect spends them; any friendly death produces one)
            let is_archmage = state
                .world()
                .card_id(minion)
                .is_some_and(|c| c.0 == "JAIL_974");
            if let Some(owner) = owner {
                let inner = state.make_mut();
                inner.players[owner.index()].died_this_turn.push(minion);
                inner.players[owner.index()].corpses += 1;
                // M5-W2 — JAIL_974 Captured Archmage: the per-card death
                // counter increments AFTER the deathrattle resolved above,
                // so the dying archmage's own deathrattle never counts
                // itself — the arm reads "4 OTHER Captured Archmages died
                // this game" (the Fireball condition).
                if is_archmage {
                    inner.players[owner.index()].jail974_deaths += 1;
                }
            }
        }
        Event::CardDrawn { .. } => {
            // Notification event — the card was already moved to hand in draw_card
        }
        Event::TradeCardExecuted { card } => {
            // Tradeable (Core Set W2): spend 1 mana, shuffle the card back
            // into the deck at a random position, draw a card.
            let active = state.active_player();
            {
                let inner = state.make_mut();
                inner.players[active.index()].current_mana =
                    (inner.players[active.index()].current_mana - 1).max(0);
            }
            let deck_count = state.world().zones().len(Zone::Deck, active);
            let position = if deck_count > 0 {
                state.rng_mut().next_usize(deck_count + 1)
            } else {
                0
            };
            // Shuffle the card into the deck at a random position (the
            // official trade semantics). Manually remove + insert: a
            // move_to_zone would append at the deck bottom, and a second
            // insert_at on top of it would duplicate the card.
            state
                .world_mut()
                .zones_mut()
                .remove(Zone::Hand, active, card);
            state.world_mut().set_zone(card, Zone::Deck);
            state
                .world_mut()
                .zones_mut()
                .insert_at(Zone::Deck, active, card, position);
            // draw_card pushes its own CardDrawn event for the drawn card
            trigger::draw_card(state, queue, active);
        }
        Event::PrepareCardExecuted { card } => {
            // Prepare (M5-W1 — Escape from Violet Hold, pinned §27): spend
            // ALL remaining mana, permanently reduce the card's cost by
            // (spent + 1), and lock the card in hand — it cannot be played
            // until its owner's next turn start (the CantPlayNextTurn
            // component, whose expiry lives in the TurnStarted handler —
            // CATA_186t Sabotage! precedent). Once per card and once per
            // turn (validated at the action site).
            let active = state.active_player();
            let spent = state.player(active).current_mana;
            {
                let inner = state.make_mut();
                let p = &mut inner.players[active.index()];
                p.current_mana = 0;
                p.prepare_used_this_turn = true;
                p.prepared_cards.push(card);
            }
            trigger::reduce_hand_card_cost(state, card, spent + 1);
            state
                .world_mut()
                .set_cant_play_next_turn(card, crate::core::component::CantPlayNextTurn);
            // M5-W2 — Jailbird (JAIL_453): "When you Prepare while holding
            // this, reduce this card's Cost by the same amount" — every
            // JAIL_453 in the hand gets the spent mana as a permanent
            // reduction (the same `reduce_hand_card_cost` path).
            {
                let jailbirds: SmallList<Entity> = state
                    .world()
                    .zones()
                    .iter(Zone::Hand, active)
                    .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "JAIL_453"))
                    .collect();
                for e in jailbirds {
                    trigger::reduce_hand_card_cost(state, e, spent);
                }
            }
        }
        Event::LocationActivated {
            player,
            location,
            target,
        } => {
            // M4-W2 Herald (2025–2026 expansions): CATA_492 Shrine of
            // Twilight's "Herald {0}. Draw a card." — the Herald sits in
            // the location's ACTIVATE text and resolves on activation,
            // before the draw (§24).
            crate::cards::herald::resolve_herald(state, queue, location, player);
            // Resolve the location's effect (stored in the battlecry slot,
            // the secret convention), consume one durability charge, and
            // mark it used this turn (attacks_used, reset at turn start).
            let effect = state.world().battlecry(location).map(|b| b.0);
            if let Some(effect) = effect {
                trigger::resolve_effect(state, queue, location, player, effect, target, None);
            }
            if let Some(d) = state.world().durability(location) {
                if d.0 > 1 {
                    state
                        .world_mut()
                        .set_durability(location, Durability(d.0 - 1));
                } else {
                    // Last charge spent: the location leaves the board
                    state.world_mut().set_durability(location, Durability(0));
                    let inner = state.make_mut();
                    if inner.players[player.index()].location == Some(location) {
                        inner.players[player.index()].location = None;
                    }
                    let _ = state.world_mut().move_to_zone(location, Zone::Graveyard);
                    // M4-W4 — CATA_527 Nespirah: a destroyed Location
                    // fires its Deathrattle ("Summon Nespirah,
                    // Unshackled") like any other board card.
                    if let Some(dr) = state.world().deathrattle(location) {
                        trigger::resolve_effect(state, queue, location, player, dr.0, None, None);
                    }
                }
            }
            state
                .world_mut()
                .set_attacks_used(location, crate::core::component::AttacksUsed(1));
        }
        Event::HeroPowerActivated {
            player,
            hero,
            target,
        } => {
            // Deduct mana (Blowtorch Saboteur — Core Set W4b: the opponent's
            // next Hero Power costs more; Dreambound Disciple — M1-W4a: the
            // next Hero Power costs (0), one-time, consumed here; M4-W4 —
            // CATA_615t Genn, Worgen King's "It costs (1)" flag — the
            // upgraded-power approximation, §26)
            let mut cost = state
                .world()
                .hero_power(hero)
                .map(|hp| {
                    let base = hp.cost + state.player(player).hero_power_cost_more;
                    if state.player(player).hero_power_cost_1 {
                        1
                    } else {
                        base
                    }
                })
                .unwrap_or(2 + state.player(player).hero_power_cost_more);
            // M3-W2a (TIME_606 Quel'dorei Fletcher): "Your Hero Power costs
            // (0) while your hand has 3 or less cards" — the deduction
            // mirrors the affordability check in validate_hero_power.
            let fletcher_free = state
                .world()
                .zones()
                .iter(Zone::Play, player)
                .any(|e| state.world().card_id(e).is_some_and(|c| c.0 == "TIME_606"))
                && state.world().zones().iter(Zone::Hand, player).count() <= 3;
            if fletcher_free {
                cost = 0;
            }
            let free = state.player(player).next_hero_power_free;
            // M5-W1 — Blood Doctor Thal'ena (JAIL_446, §27): the Vampyr's
            // Kiss hero power costs 3 Corpses instead of Mana (spent here;
            // the affordability check sits in validate_hero_power).
            let thalena_corpses = state.player(player).thalena_corpses_hero_power;
            {
                let inner = state.make_mut();
                let p = &mut inner.players[player.index()];
                if free {
                    p.next_hero_power_free = false;
                } else if thalena_corpses {
                    p.corpses = p.corpses.saturating_sub(3);
                } else {
                    p.current_mana = (p.current_mana - cost).max(0);
                }
                // Glowroot Lure (M1-W4a): the hero-power-use counter
                p.hero_power_uses += 1;
            }
            // Mark as used
            state
                .world_mut()
                .set_hero_power_used(hero, HeroPowerUsed(true));
            // Story of Sulfuras (M2-W4a): the swapped-in "Deal 8 damage to a
            // random enemy" hero power — each use ticks the counter, and the
            // second use swaps the original hero power back.
            if state.player(player).sulfuras_uses > 0 {
                let swap_back = {
                    let p = &mut state.make_mut().players[player.index()];
                    p.sulfuras_uses += 1;
                    if p.sulfuras_uses >= 2 {
                        p.sulfuras_uses = 0;
                        true
                    } else {
                        false
                    }
                };
                if swap_back {
                    let original = state.make_mut().players[player.index()]
                        .sulfuras_original
                        .take();
                    if let Some(original) = original {
                        state.world_mut().set_hero_power(hero, original);
                    }
                }
            }
            // Resolve the hero power effect (explicit target, roadmap G6)
            if let Some(hp_def) = state.world().hero_power(hero) {
                let effect = hp_def.effect;
                trigger::resolve_effect(state, queue, hero, player, effect, target, None);
                // M3-W2b — Deios doubles hero-power activations (§21).
                if deios_doubling(state, player) {
                    trigger::resolve_effect(state, queue, hero, player, effect, target, None);
                }
                // M3-W3 — HeroPowerUsed triggers (END_008 Enduring Roach:
                // "After you use your Hero Power, refresh 2 Mana Crystals")
                // fire after the power's effect resolved, with the hero as
                // the subject (§22).
                fire_triggers(
                    state,
                    queue,
                    TriggerEvent::HeroPowerUsed,
                    player,
                    Some(hero),
                    None,
                );
            }
        }
        Event::WeaponEquipped { .. } => {
            // Notification event — the weapon was already created in resolve_equip_weapon
        }
        Event::WeaponDestroyed { weapon, .. } => {
            // The weapon leaves play when destroyed — it must stop firing its
            // triggers (Sword of Justice's summon trigger). The player's weapon
            // pointer was already cleared by the destroying action.
            let _ = state.world_mut().move_to_zone(weapon, Zone::Graveyard);
            // The weapon's deathrattle fires when it breaks or is replaced
            // (Barbed Thorn choose branch 2, M1-W3) — mirrors the minion death
            // path: move to the graveyard first, then resolve the deathrattle
            // (the entity is in the graveyard, as in HS).
            if let (Some(dr), Some(owner)) = (
                state.world().deathrattle(weapon),
                state.world().player(weapon),
            ) {
                trigger::resolve_effect(state, queue, weapon, owner, dr.0, None, None);
            }
        }
        Event::SecretRevealed { .. } => {
            // Notification event — the secret effect is resolved through the trigger system
        }
        Event::SpellCast {
            player,
            spell,
            target,
        } => {
            // HS: deaths caused by the spell resolve BEFORE "after you cast"
            // triggers fire (a Wild Pyromancer killed by the spell must not
            // fire). When the spell left pending deaths, defer the trigger
            // firing: enqueue the death batch (Normal — processes first) and
            // re-push this event at Lowest; the second pass sees an empty
            // pending-death batch and fires the triggers.
            if !state.pending_deaths().is_empty() && process_pending_deaths(state, queue) {
                queue.push_with_priority(
                    Event::SpellCast {
                        player,
                        spell,
                        target,
                    },
                    Priority::Lowest,
                );
                return Ok(());
            }
            // New Moon upgrade counter (M1-W4a): one per cast spell — AFTER
            // the deferral branch so a re-pushed SpellCast event only counts
            // once (the Wish/Ritual upgrade condition, §14.3).
            state.make_mut().players[player.index()].spells_cast_total += 1;
            // M5-W2 — Molten Gold / Frostshatter / Stormfury / Code Violet /
            // Tricksy Improviser read the OTHER spells cast this turn, so
            // the counter increments here too — the SpellCast event fires
            // after the spell's own effect, which therefore never counts
            // itself. Cleared at the owner's turn start.
            state.make_mut().players[player.index()].spells_cast_this_turn += 1;
            // Quest progress (M2-W1): TLC_817 — "Cast 4 Holy spells" — the
            // cast spell's school comes from the quest registry's static
            // spell-school table (the 22 dump entries); school-less spells
            // fire nothing.
            if let Some(school) = state
                .world()
                .card_id(spell)
                .and_then(|cid| crate::cards::quest::spell_school(cid.0))
            {
                crate::engine::quest::progress(
                    state,
                    queue,
                    player,
                    crate::cards::quest::QuestCondition::CastSpellsOfSchool { school },
                    1,
                    None,
                );
                // M2-W4a per-turn school flags: Gladesong Siren's "costs (1)
                // if you cast a Holy and Shadow spell this turn" reads the
                // shadow marker; Creature of the Sacred Cave recasts a random
                // Holy spell cast this turn (the id list).
                match school {
                    crate::cards::quest::SpellSchool::Holy => {
                        let cid = state.world().card_id(spell).map(|c| c.0.to_string());
                        if let Some(cid) = cid {
                            state.make_mut().players[player.index()]
                                .holy_cast_ids
                                .push(cid);
                        }
                    }
                    crate::cards::quest::SpellSchool::Shadow => {
                        state.make_mut().players[player.index()].shadow_cast_this_turn = true;
                    }
                    crate::cards::quest::SpellSchool::Nature => {
                        // M3-W2a — Primordial Overseer TIME_213's battlecry
                        // scales with the Nature spells cast this game.
                        state.make_mut().players[player.index()].nature_spells_cast_total += 1;
                    }
                    // M4-W4 counters (2025–2026 expansions, Cataclysm §26):
                    // CATA_529 Ravenous Felfisher "Costs (1) less for each
                    // Fel spell you've cast this game" and CATA_584 Erupting
                    // Volcano "If you've played a Fire spell this turn" —
                    // both read at the cost / battlecry sites.
                    crate::cards::quest::SpellSchool::Fel => {
                        state.make_mut().players[player.index()].fel_spells_cast_this_game += 1;
                    }
                    crate::cards::quest::SpellSchool::Fire => {
                        state.make_mut().players[player.index()].fire_spell_played_this_turn = true;
                    }
                    _ => {}
                }
            }
            // Spell triggers: registered FriendlySpellCast triggers fire in
            // play order. The cast spell rides as the subject — behaviour-
            // neutral for the existing triggers (none of them read it; the
            // `po_spellcast_subject_is_behaviour_neutral` scenario pins the
            // values) and needed by Animated Moonwell (M1-W4a — gain Attack
            // equal to the spell's Cost).
            fire_triggers(
                state,
                queue,
                TriggerEvent::FriendlySpellCast,
                player,
                Some(spell),
                None,
            );
            // Global spell trigger (pool-open M1 — Lorewalker Cho): fires for
            // either player, with the spell entity as the subject so the
            // effect can hand the copy to the caster's opponent. The subject
            // is behaviour-neutral for FriendlySpellCast triggers (none of
            // them carry race/max_attack conditions) — pinned by
            // `po_spellcast_subject_is_behaviour_neutral`.
            fire_triggers(
                state,
                queue,
                TriggerEvent::AnySpellCast,
                player,
                Some(spell),
                None,
            );
            // M3-W3 — END_026 Fragment of Nothing: "after you cast a spell
            // ON A MINION, draw a card". The spell's explicit target (when
            // the cast had one — `Event::SpellCast.target`) identifies the
            // minion; the trigger is pinned to the Fragment, so this fires
            // whenever ANY friendly minion is the target of a friendly
            // spell. Fired after the FriendlySpellCast / AnySpellCast
            // triggers — same position as the after-cast block.
            if let Some(t) = target {
                if state.world().card_type(t) == Some(CardType::Minion) {
                    fire_triggers(
                        state,
                        queue,
                        TriggerEvent::FriendlySpellCastOnMinion,
                        player,
                        Some(t),
                        None,
                    );
                }
            }
            // M5-W1 — Jailhouse Manastorm (JAIL_122, §27): "After you cast
            // a spell this game, summon a random minion of the same Cost."
            // The battlecry armed `manastorm_after_spell`; while the
            // Manastorm is on the board (the while-alive simplification of
            // the game-long trigger — the flag persists after its death but
            // the board check gates it), every cast spell summons a random
            // minion of the spell's Cost from the `random_minion_of_cost`
            // pool (ALL_CARDS, tokens excluded). Fires after the
            // after-cast triggers, at the end of the handler.
            if state.player(player).manastorm_after_spell
                && state
                    .world()
                    .zones()
                    .iter(Zone::Play, player)
                    .any(|e| state.world().card_id(e).is_some_and(|c| c.0 == "JAIL_122"))
            {
                if let Some(cost) = state.world().effective_cost(spell) {
                    if let Some(def) = trigger::random_minion_of_cost(state, cost.0) {
                        trigger::resolve_summon(state, queue, spell, player, def.id);
                    }
                }
            }
        }
        Event::ChoiceResolved { choice_id, option } => {
            // Resolve the pending choice (roadmap G6). The choice was validated
            // and the pending entry is consumed here.
            let inner = state.make_mut();
            let Some(pending) = inner.pending_choice.take() else {
                return Ok(());
            };
            if pending.id != choice_id {
                inner.pending_choice = Some(pending);
                return Err(EngineError::InvalidChoice);
            }
            // Choose Thrice (Cenarius, M1-W4b): the remaining repeats of a
            // repeatable choose-one choice — captured before the match
            // consumes `pending` and re-surfaced after each branch resolves.
            let repeat = pending.repeat;
            match pending.kind {
                ChoiceKind::ChooseOne => {
                    let card = pending.card;
                    let player = state.world().player(card).unwrap_or(state.active_player());
                    let card_type = state.world().card_type(card);
                    let chosen_effect = if option == 0 {
                        state.world().battlecry(card).map(|b| b.0)
                    } else if option == 1 {
                        state.world().choose_one_effect(card).map(|c| c.0)
                    } else {
                        // M2-W4a third branch — the Un'Goro choose-one
                        // beasts (Ancient Stegodon / Ancient Raptor /
                        // Ancient Pterrordax), resolved from the
                        // cards-side three-branch table.
                        state
                            .world()
                            .card_id(card)
                            .and_then(|c| crate::cards::def::card_by_id(c.0))
                            .and_then(crate::cards::choose_one_three_branch)
                    };
                    let returns_to_hand = matches!(
                        chosen_effect,
                        Some(crate::core::effect::CardEffect::DealDamageAndReturnToHand { .. })
                    );
                    if let Some(effect) = chosen_effect {
                        trigger::resolve_effect(state, queue, card, player, effect, None, None);
                    }
                    // Tyrande (M1-W4b): the choose-one branch is also a spell
                    // cast — the "cast twice" re-resolution fires here too
                    // (no explicit target, no second SpellCast event, §14.4).
                    if state.player(player).spells_cast_twice_pending > 0 {
                        state.make_mut().players[player.index()].spells_cast_twice_pending -= 1;
                        if let Some(effect) = chosen_effect {
                            trigger::resolve_effect(state, queue, card, player, effect, None, None);
                        }
                    }
                    // M3-W2b — Deios: the choose-one branch is the card's
                    // battlecry — one re-resolve of the chosen branch (§21).
                    if deios_doubling(state, player) {
                        if let Some(effect) = chosen_effect {
                            trigger::resolve_effect(state, queue, card, player, effect, None, None);
                        }
                    }
                    if card_type == Some(CardType::Spell) {
                        // Spells: the SpellCast event and graveyard move complete
                        // the play that was deferred when the choice surfaced.
                        // After-cast triggers fire at Lowest priority — after the
                        // spell's damage and the deaths it caused have resolved
                        // (HS: deaths process before "after you cast" triggers)
                        queue.push_with_priority(
                            Event::SpellCast {
                                player,
                                spell: card,
                                // The choice deferred the spell's play; the
                                // original explicit target is lost by the time
                                // the branch resolves (M3-W3 — END_026's
                                // FriendlySpellCastOnMinion sees no target for
                                // choose-one spells, §22).
                                target: None,
                            },
                            Priority::Lowest,
                        );
                        if !returns_to_hand {
                            let _ = state.world_mut().move_to_zone(card, Zone::Graveyard);
                        }
                    }
                    // Choose Thrice (Cenarius, M1-W4b): re-surface the choice
                    // for the remaining picks — each pick resolves one branch
                    // (options may be mixed, like the official card).
                    if repeat > 1 {
                        let labels = choose_one_labels_all(state, card);
                        state.set_pending_choice_repeat(
                            ChoiceKind::ChooseOne,
                            card,
                            labels.iter().map(|l| l.to_string()).collect(),
                            Vec::new(),
                            false,
                            repeat - 1,
                        );
                    }
                }
                ChoiceKind::QonzuKeepOrTop => {
                    // Q'onzu (M1-W4b): option 0 keeps the discovered spell in
                    // hand (nothing to do — it already is there); option 1
                    // places it on top of the opponent's deck. The card's
                    // PLAYER component must switch to the enemy — zone
                    // movement (draws, discard) derives the owner from it.
                    if option == 1 {
                        let card = pending.card;
                        let player = state.world().player(card).unwrap_or(state.active_player());
                        let enemy = player.opponent();
                        let world = state.world_mut();
                        world.zones_mut().remove(Zone::Hand, player, card);
                        world.set_zone(card, Zone::Deck);
                        world.set_player(card, enemy);
                        world.zones_mut().insert_at(Zone::Deck, enemy, card, 0);
                    }
                }
                ChoiceKind::Discover => {
                    let player = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    // M2-W4a: "Discovered this turn" — the discover machinery
                    // sets the flag for every discover (Storage Scuffle's cost
                    // (0), Unearthed Artifacts' 4-Cost summon, Vault Breaker's
                    // discount all read it; cleared at the turn end).
                    state.make_mut().players[player.index()].discovered_this_turn = true;
                    if pending.discard_rest {
                        // Tracking (W10): the pool IS the deck's top cards —
                        // the picked card's existing entity moves to hand and
                        // the unpicked ones are discarded.
                        let deck: Vec<Entity> = state
                            .world()
                            .zones()
                            .iter(Zone::Deck, player)
                            .take(3)
                            .collect();
                        let Some(picked) = deck.get(option as usize).copied() else {
                            return Ok(());
                        };
                        let _ = state.world_mut().move_to_zone(picked, Zone::Hand);
                        for &e in &deck {
                            if e != picked {
                                let _ = state.world_mut().move_to_zone(e, Zone::Graveyard);
                            }
                        }
                    } else if let Some(card_id) = pending.pool.get(option as usize) {
                        // Add the picked pool card to the owner's hand
                        if let Some(card_def) = crate::cards::def::card_by_id(card_id) {
                            let picked_entity = trigger::add_card_to_hand(state, player, card_def);
                            if let Some(entity) = picked_entity {
                                apply_w4a_discover_modifiers(
                                    state,
                                    pending.temporary,
                                    pending.card,
                                    entity,
                                );
                                // M3-W2a: consume the discover-time modifiers
                                // stashed by the effect arms — Neon Innovation's
                                // +A/+H (TIME_016) and Alter Time's cost
                                // reduction (TIME_857) must land on the PICKED
                                // card, not on the effect's source.
                                let pending_bonus = state.make_mut().players[player.index()]
                                    .pending_discover_hand_bonus
                                    .take();
                                if let Some((attack, health)) = pending_bonus {
                                    state.world_mut().add_enchantment(
                                        entity,
                                        crate::core::component::Enchantment {
                                            attack,
                                            health,
                                            cost: 0,
                                            expiry: crate::core::component::EnchantmentExpiry::Permanent,
                                        },
                                    );
                                }
                                let pending_reduction = state.make_mut().players[player.index()]
                                    .pending_discover_cost_reduction
                                    .take();
                                if let Some(reduction) = pending_reduction {
                                    trigger::reduce_hand_card_cost(state, entity, reduction);
                                }
                                // The Map chain (M2-W4a, fidelity-debt §17): the
                                // OTHER options are stored so playing the
                                // discovered card this turn adds one of them.
                                if !pending.map_others.is_empty() {
                                    let others: Vec<String> = pending
                                        .pool
                                        .iter()
                                        .filter(|id| id.as_str() != card_id.as_str())
                                        .cloned()
                                        .collect();
                                    state.make_mut().players[player.index()].map_pending =
                                        Some((entity, others));
                                }
                            }
                        }
                    }
                    // Merchant of Legend (M2-W4a): shuffle the other two
                    // options into the deck.
                    if state
                        .world()
                        .card_id(pending.card)
                        .is_some_and(|c| c.0 == "TLC_514")
                    {
                        for (i, other) in pending.pool.iter().enumerate() {
                            if i == option as usize {
                                continue;
                            }
                            if let Some(def) = crate::cards::def::card_by_id(other) {
                                let deck_count = state.world().zones().len(Zone::Deck, player);
                                let position = if deck_count > 0 {
                                    state.rng_mut().next_usize(deck_count + 1)
                                } else {
                                    0
                                };
                                let world = state.world_mut();
                                let e = crate::cards::spawn_card_from_def(world, player, def);
                                world.set_zone(e, Zone::Deck);
                                world.zones_mut().insert_at(Zone::Deck, player, e, position);
                                state.make_mut().players[player.index()].shuffled_count += 1;
                            }
                        }
                    }
                    // Paleomancy (M2-W4a): spend 5 Corpses to keep all 3
                    // instead — the whole pool goes to hand.
                    if state
                        .world()
                        .card_id(pending.card)
                        .is_some_and(|c| c.0 == "TLC_434")
                        && state.player(player).corpses >= 5
                    {
                        let inner = state.make_mut();
                        inner.players[player.index()].corpses -= 5;
                        for (i, other) in pending.pool.iter().enumerate() {
                            if i == option as usize {
                                continue;
                            }
                            if let Some(def) = crate::cards::def::card_by_id(other) {
                                if let Some(entity) = trigger::add_card_to_hand(state, player, def)
                                {
                                    apply_w4a_discover_modifiers(
                                        state,
                                        false,
                                        pending.card,
                                        entity,
                                    );
                                }
                            }
                        }
                    }
                }
                ChoiceKind::DiscoverDeck => {
                    // Cursed Catacombs / Cultist Map (M2-W4a): the pool holds
                    // three random DISTINCT deck card ids (the source card
                    // excluded); the picked card is the deck entity moved to
                    // hand. Cursed Catacombs marks it Temporary; Cultist Map
                    // runs the Map chain (see fidelity-debt §17).
                    let player = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    state.make_mut().players[player.index()].discovered_this_turn = true;
                    let Some(picked_id) = pending.pool.get(option as usize) else {
                        return Ok(());
                    };
                    let deck: Vec<Entity> = state
                        .world()
                        .zones()
                        .iter(Zone::Deck, player)
                        .filter(|&e| {
                            state
                                .world()
                                .card_id(e)
                                .is_some_and(|c| c.0 == picked_id.as_str())
                        })
                        .collect();
                    let Some(picked) = deck.first().copied() else {
                        return Ok(());
                    };
                    let _ = state.world_mut().move_to_zone(picked, Zone::Hand);
                    if pending.temporary {
                        state
                            .world_mut()
                            .set_temporary(picked, crate::core::component::Temporary);
                    }
                    if !pending.map_others.is_empty() {
                        state.make_mut().players[player.index()].map_pending =
                            Some((picked, pending.map_others.clone()));
                    }
                }
                ChoiceKind::DiscoverEnemyDeckPutOnTop => {
                    // Eyes in the Sky (M2-W4a): look at 3 cards in the enemy's
                    // deck, pick one to put on top — the picked existing
                    // entity moves to the top (index 0) of the enemy deck.
                    let player = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    state.make_mut().players[player.index()].discovered_this_turn = true;
                    let Some(picked_id) = pending.pool.get(option as usize) else {
                        return Ok(());
                    };
                    let enemy = player.opponent();
                    let deck: Vec<Entity> = state
                        .world()
                        .zones()
                        .iter(Zone::Deck, enemy)
                        .filter(|&e| {
                            state
                                .world()
                                .card_id(e)
                                .is_some_and(|c| c.0 == picked_id.as_str())
                        })
                        .collect();
                    let Some(picked) = deck.first().copied() else {
                        return Ok(());
                    };
                    let world = state.world_mut();
                    world.zones_mut().remove(Zone::Deck, enemy, picked);
                    world.zones_mut().insert_at(Zone::Deck, enemy, picked, 0);
                }
                ChoiceKind::DiscoverEnemyHandCopy => {
                    // Deja Vu (M3-W2a — TIME_039): discover a COPY of a card
                    // in the opponent's hand (pool-open — the pool holds
                    // enemy-hand card ids; the pick adds the card's
                    // definition to the player's hand, the original stays).
                    let player = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    state.make_mut().players[player.index()].discovered_this_turn = true;
                    let Some(picked_id) = pending.pool.get(option as usize) else {
                        return Ok(());
                    };
                    if let Some(card_def) = crate::cards::def::card_by_id(picked_id) {
                        trigger::add_card_to_hand(state, player, card_def);
                    }
                }
                ChoiceKind::DiscoverDeckAndEnemyHandCopy => {
                    // Intertwined Fate (M3-W2a — TIME_432): discover a copy
                    // of a card from the player's deck and one from the
                    // opponent's hand. The pool holds three deck ids
                    // followed by three enemy-hand ids; the picked option's
                    // copy goes to the hand and a random copy from the
                    // OTHER pool follows (the §20 combined-choice shape).
                    let player = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    state.make_mut().players[player.index()].discovered_this_turn = true;
                    let Some(picked_id) = pending.pool.get(option as usize) else {
                        return Ok(());
                    };
                    if let Some(card_def) = crate::cards::def::card_by_id(picked_id) {
                        trigger::add_card_to_hand(state, player, card_def);
                    }
                    let (deck_pool, enemy_pool) = pending.pool.split_at(3);
                    let other_pool = if option < 3 { enemy_pool } else { deck_pool };
                    if let Some(other) =
                        other_pool.get(state.rng_mut().next_usize(other_pool.len()))
                    {
                        if let Some(other_def) = crate::cards::def::card_by_id(other) {
                            trigger::add_card_to_hand(state, player, other_def);
                        }
                    }
                }
                ChoiceKind::DiscoverDeckOthersBottom => {
                    // Waveshaping (M3-W2a — TIME_701): discover a card from
                    // the deck; the others are put on the bottom. The pool
                    // holds deck card ids; the picked card's EXISTING entity
                    // moves to hand and the unpicked ones move to the deck's
                    // bottom (the last positions — reverse draw order).
                    let player = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    state.make_mut().players[player.index()].discovered_this_turn = true;
                    let deck: Vec<Entity> = state
                        .world()
                        .zones()
                        .iter(Zone::Deck, player)
                        .filter(|&e| {
                            pending
                                .pool
                                .iter()
                                .any(|id| state.world().card_id(e).is_some_and(|c| c.0 == id))
                        })
                        .collect();
                    let Some(picked_id) = pending.pool.get(option as usize) else {
                        return Ok(());
                    };
                    let Some(picked) = deck
                        .iter()
                        .find(|&&e| {
                            state
                                .world()
                                .card_id(e)
                                .is_some_and(|c| c.0 == picked_id.as_str())
                        })
                        .copied()
                    else {
                        return Ok(());
                    };
                    let _ = state.world_mut().move_to_zone(picked, Zone::Hand);
                    // The unpicked entries move to the bottom — re-inserted
                    // after removal so they sit after the remaining deck
                    // cards (the deck's last positions).
                    for &e in &deck {
                        if e == picked {
                            continue;
                        }
                        let world = state.world_mut();
                        world.zones_mut().remove(Zone::Deck, player, e);
                        world.zones_mut().insert(Zone::Deck, player, e);
                    }
                }
                ChoiceKind::Cataclysm => {
                    // Deathwing's battlecry (M4-W4 — the C4 primitive): the
                    // pool holds the four data-defined Cataclysm spell ids;
                    // the pick resolves the Cataclysm's spell effect and is
                    // recorded in `Player::pending_cataclysms` so the choice
                    // re-surfaces (herald-scaled count of distinct picks)
                    // with the picked options removed.
                    let player = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    let Some((picks_left, picked)) =
                        state.player(player).pending_cataclysms.clone()
                    else {
                        return Ok(());
                    };
                    let Some(card_id) = pending.pool.get(option as usize) else {
                        return Ok(());
                    };
                    if let Some(def) = crate::cards::def::card_by_id(card_id) {
                        if let Some(effect) = def.spell_effect {
                            trigger::resolve_effect(
                                state,
                                queue,
                                pending.card,
                                player,
                                effect,
                                None,
                                None,
                            );
                        }
                    }
                    let mut picked = picked;
                    picked.push(card_id.clone());
                    let remaining = picks_left.saturating_sub(1);
                    if remaining > 0 {
                        let reduced: Vec<String> = pending
                            .pool
                            .iter()
                            .filter(|id| !picked.iter().any(|p| p == *id))
                            .cloned()
                            .collect();
                        state.make_mut().players[player.index()].pending_cataclysms =
                            Some((remaining, picked));
                        state.set_pending_choice_repeat(
                            ChoiceKind::Cataclysm,
                            pending.card,
                            reduced.clone(),
                            reduced,
                            false,
                            remaining as u8,
                        );
                    } else {
                        state.make_mut().players[player.index()].pending_cataclysms = None;
                    }
                }
                ChoiceKind::ChooseHandCard => {
                    // M4-W4 — choose a card in the player's hand (the
                    // options are the hand card ids in hand order); the
                    // pending kind stashed by the effect arm applies to the
                    // picked hand entity.
                    let player = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    let Some(kind) = state.player(player).pending_choose_hand else {
                        return Ok(());
                    };
                    state.make_mut().players[player.index()].pending_choose_hand = None;
                    let Some(picked_id) = pending.pool.get(option as usize) else {
                        return Ok(());
                    };
                    let hand: Vec<Entity> =
                        state.world().zones().iter(Zone::Hand, player).collect();
                    let Some(picked) = hand.iter().copied().find(|&e| {
                        state
                            .world()
                            .card_id(e)
                            .is_some_and(|c| c.0 == picked_id.as_str())
                    }) else {
                        return Ok(());
                    };
                    trigger::resolve_choose_hand_card(
                        state,
                        queue,
                        pending.card,
                        player,
                        kind,
                        picked,
                    );
                }
                ChoiceKind::TinyPalAmmo => {
                    // M5-W2 — JAIL_458 Tiny Pal's ammunition choice: the
                    // picked option is the ammunition weapon id; equipping
                    // it arms the attack trigger that fires the shot and
                    // re-surfaces the choice.
                    let player = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    let Some(weapon_id) = pending.pool.get(option as usize) else {
                        return Ok(());
                    };
                    trigger::resolve_equip_weapon(state, queue, player, weapon_id);
                }
                ChoiceKind::BloodClone => {
                    // M5-W2 — JAIL_451 Blood Clone: "Discover a 5-Cost
                    // minion. Spend 5 Corpses to summon a copy of it." —
                    // the choice only surfaces when the Corpses are
                    // affordable; the resolution spends them and summons
                    // the pick.
                    let player = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    if state.player(player).corpses >= 5 {
                        let inner = state.make_mut();
                        inner.players[player.index()].corpses -= 5;
                    }
                    crate::engine::quest::progress(
                        state,
                        queue,
                        player,
                        crate::cards::quest::QuestCondition::SpendCorpses,
                        5,
                        None,
                    );
                    if let Some(card_id) = pending.pool.get(option as usize) {
                        if let Some(def) = crate::cards::def::card_by_id(card_id) {
                            let _ =
                                trigger::resolve_summon(state, queue, pending.card, player, def.id);
                        }
                    }
                }
                ChoiceKind::PickEnemyHandCard => {
                    // M5-W2 — JAIL_303 Ancient Augur's secret
                    // investigation: the picked enemy hand card id is
                    // stashed; the deathrattle discards the matching card.
                    let player = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    if let Some(picked_id) = pending.pool.get(option as usize) {
                        state.make_mut().players[player.index()].augur_suspect =
                            Some(picked_id.clone());
                    }
                }
                ChoiceKind::MurlocHolmes => {
                    // M5-W2 — JAIL_851 Inspector Murloc Holmes: the
                    // investigation records the suspected card's NAME and
                    // the turn the other player would play it; the
                    // CardPlayed handler pays out 3 Coins on a match.
                    let player = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    if let Some(picked_id) = pending.pool.get(option as usize) {
                        let name = crate::cards::def::card_by_id(picked_id)
                            .map(|d| d.name.to_string())
                            .unwrap_or_else(|| picked_id.clone());
                        state.make_mut().players[player.index()].murloc_holmes_suspect =
                            Some((name, state.turn() + 1));
                    }
                }
                ChoiceKind::DiscoverDeckDestroyRest => {
                    // Commander Geddon (M4-W4): the DrawStep hook surfaces
                    // this choice instead of the turn's draw; the pool
                    // holds deck card ids. The picked card's EXISTING
                    // entity moves to hand with a (3) reduction; the
                    // unpicked pool entries are destroyed (§26 — the
                    // destroy scope is the two unpicked Discover options).
                    let player = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    let Some(picked_id) = pending.pool.get(option as usize) else {
                        return Ok(());
                    };
                    let deck: Vec<Entity> = state
                        .world()
                        .zones()
                        .iter(Zone::Deck, player)
                        .filter(|&e| {
                            state
                                .world()
                                .card_id(e)
                                .is_some_and(|c| c.0 == picked_id.as_str())
                        })
                        .collect();
                    let Some(picked) = deck.first().copied() else {
                        return Ok(());
                    };
                    let _ = state.world_mut().move_to_zone(picked, Zone::Hand);
                    let cur = state.world().effective_cost(picked).unwrap_or(Cost(0)).0;
                    state.world_mut().set_cost(picked, Cost((cur - 3).max(0)));
                    for id in &pending.pool {
                        if id == picked_id {
                            continue;
                        }
                        let doomed: Vec<Entity> = state
                            .world()
                            .zones()
                            .iter(Zone::Deck, player)
                            .filter(|&e| {
                                state.world().card_id(e).is_some_and(|c| c.0 == id.as_str())
                            })
                            .collect();
                        for e in doomed {
                            let _ = state.world_mut().move_to_zone(e, Zone::Graveyard);
                        }
                    }
                }
                ChoiceKind::Mulligan => {
                    // Opening mulligan (roadmap G7): "Keep all" (option 0) or
                    // replace the chosen starting card — the card returns to the
                    // deck, the deck is reshuffled, and a new card is drawn from
                    // the top. The Coin is not mulliganable.
                    let owner = state
                        .world()
                        .player(pending.card)
                        .unwrap_or(state.active_player());
                    if option > 0 {
                        let mulliganable: SmallList<Entity> = state
                            .world()
                            .zones()
                            .iter(Zone::Hand, owner)
                            .filter(|&e| state.world().card_id(e).is_none_or(|c| c.0 != "GAME_005"))
                            .collect();
                        if let Some(card) = mulliganable.get((option - 1) as usize).copied() {
                            let _ = state.world_mut().move_to_zone(card, Zone::Deck);
                            state.shuffle_decks();
                            crate::engine::trigger::draw_top_card_no_queue(state, owner);
                        }
                    }
                    state.make_mut().mulliganed[owner.index()] = true;
                    // Surface the opponent's mulligan; when both are resolved the
                    // opening is finished and the first player draws the 4th card
                    // as their turn 1 starts (official rule).
                    let next = owner.opponent();
                    if !state.make_mut().mulliganed[next.index()] {
                        state.surface_mulligan(next);
                    } else {
                        trigger::draw_top_card_no_queue(state, PlayerId::Player1);
                    }
                }
            }
        }
        Event::GameOver { winner } => {
            state.set_step(Step::GameOver { winner });
        }
    }
    Ok(())
}

// ============================================================
// Step state machine (roadmap G1)
// ============================================================

/// Advances the step state machine at an event-queue boundary.
///
/// Steps are entered by state: the turn-start sequence runs StartTriggers →
/// ManaRefill → TurnCostReduce → DrawStep → Main (start-of-turn card
/// effects fire before the mana refill, the Circadiamancer discount and the
/// draw); the end sequence runs EndTriggers → WrapUp → next player's
/// TurnStarted. Pending deaths (roadmap G3) enter the death step from any
/// boundary and return to the interrupted step afterwards.
/// Returns `false` when the machine is waiting for player input (Main step
/// with an empty queue) or the game is over — the engine loop should stop then.
pub fn advance_step(state: &mut GameState, queue: &mut EventQueue) -> bool {
    // The death phase takes precedence at any step boundary (HS: deaths resolve
    // before anything else proceeds). The interrupted step is saved so the
    // machine returns to it after the pending deaths are processed.
    if !state.pending_deaths().is_empty() && state.step() != Step::Death {
        state.make_mut().return_step = state.step();
        state.set_step(Step::Death);
        return true;
    }
    match state.step() {
        Step::GameOver { .. } => false,
        Step::StartTriggers => {
            // Start-of-turn triggers (CardDef::start_turn_effect) fire before
            // the mana refill and the draw
            let active = state.active_player();
            fire_triggers(state, queue, TriggerEvent::TurnStart, active, None, None);
            state.set_step(Step::ManaRefill);
            true
        }
        Step::ManaRefill => {
            // Mana crystal growth and refill. The overload mana lock (roadmap
            // F1) applies HERE: after the refill, mana locked by overload cards
            // played last turn is subtracted from current_mana.
            let active = state.active_player();
            let inner = state.make_mut();
            let p = &mut inner.players[active.index()];
            p.mana_crystals = (p.mana_crystals + 1).min(10);
            let locked = p.overload_locked;
            p.overload_locked = 0;
            p.current_mana = (p.mana_crystals - locked).max(0);
            // Emberscarred Whelp (M1-W5): "next turn only" Mana Crystals
            // granted last turn are added at this refill (simplification of
            // the real until-end-of-next-turn timing, §14.5)
            let temp = p.temp_mana_crystal_pending;
            p.temp_mana_crystal_pending = 0;
            p.current_mana = (p.current_mana + temp).min(10);
            // M3-W3 — Acceleration Aura (END_011): "At the start of your
            // turn, gain a temporary Mana Crystal. Lasts 3 turns." Each
            // own turn start consumes one tick and grants the +1 (capped
            // at 10 like the pending crystal above).
            if p.acceleration_aura_ticks > 0 {
                p.acceleration_aura_ticks -= 1;
                p.current_mana = (p.current_mana + 1).min(10);
            }
            // M3-W3 — the Blessing of the Infinite per-turn counter
            // (END_003p) resets with the turn.
            p.undead_played_this_turn = 0;
            p.cards_played_this_turn = 0;
            p.minions_played_this_turn = 0;
            // Naralex (M1-W4b): the per-turn Dragon counter resets with the
            // turn ("your first Dragon each turn costs (1)")
            p.dragons_played_this_turn = 0;
            // M3-W2a: chain into the TurnCostReduce step (Circadiamancer's
            // marked-hand discount) before the draw.
            state.set_step(Step::TurnCostReduce);
            true
        }
        Step::TurnCostReduce => {
            // M3-W2a (Circadiamancer TIME_102): "At the start of your turns,
            // reduce its Cost by (1)" — the marked hand card's accumulated
            // discount grows with every own turn start. Runs between the
            // mana refill and the draw, matching the card's wording.
            let active = state.active_player();
            let marked: SmallList<(Entity, u32)> = state
                .world()
                .zones()
                .iter(Zone::Hand, active)
                .filter_map(|e| state.world().turn_cost_reducer(e).map(|t| (e, t.0)))
                .collect();
            for (entity, count) in marked {
                state.world_mut().set_turn_cost_reducer(
                    entity,
                    crate::core::component::TurnCostReducer(count + 1),
                );
            }
            state.set_step(Step::DrawStep);
            true
        }
        Step::DrawStep => {
            // The turn's draw. Every player draws at the start of every turn,
            // including the first player's turn 1 (official rule) — the shipped
            // opening (sim::battle::build_game_state) deals that 4th card during
            // construction, so a DrawStep on turn 1 only appears in constructed
            // states, where the draw runs normally (no turn-1 skip).
            let active = state.active_player();
            // M3-W2a (TIME_617 Chronochiller): "You no longer draw a card
            // at the start of your turn" — an ID-based board check at the
            // draw point (the Tichondrius/Petrified-Ogre precedent).
            let skip_draw = state
                .world()
                .zones()
                .iter(Zone::Play, active)
                .any(|e| state.world().card_id(e).is_some_and(|c| c.0 == "TIME_617"));
            // M4-W4 — CATA_591 Commander Geddon: "Instead of drawing each
            // turn, Discover a card from your deck. It costs (3) less.
            // Destroy the others." — while the battlecry-set flag rides the
            // player, the DrawStep surfaces a DiscoverDeckDestroyRest choice
            // over 3 distinct deck card ids (sampled with the embedded RNG;
            // the picked entity moves to hand at (3) less, the unpicked
            // options are destroyed at resolution — §26).
            if state.player(active).geddon_discover_draw {
                let mut ids: Vec<String> = Vec::new();
                for e in state.world().zones().iter(Zone::Deck, active) {
                    if let Some(c) = state.world().card_id(e) {
                        if !ids.iter().any(|s| s.as_str() == c.0) {
                            ids.push(c.0.to_string());
                        }
                    }
                }
                for i in (1..ids.len()).rev() {
                    let j = state.rng_mut().next_usize(i + 1);
                    ids.swap(i, j);
                }
                ids.truncate(3);
                if !ids.is_empty() {
                    state.set_pending_choice(
                        ChoiceKind::DiscoverDeckDestroyRest,
                        state.player(active).hero,
                        ids.clone(),
                        ids,
                    );
                }
            } else if !skip_draw {
                trigger::draw_card(state, queue, active);
            }
            state.set_step(Step::Main);
            true
        }
        Step::EndTriggers => {
            // End-of-turn triggers fire before the wrap-up cleanup
            let active = state.active_player();
            fire_triggers(state, queue, TriggerEvent::TurnEnd, active, None, None);
            // M3-W2b — Deios' doubling covers end-of-turn trigger effects
            // (TIME_706's swap-back): the full trigger set fires twice (§21).
            if deios_doubling(state, active) {
                fire_triggers(state, queue, TriggerEvent::TurnEnd, active, None, None);
            }
            // M3-W2a — TIME_054 Time Skipper: "At the end of each player's
            // turn, that player gets a Coin" — a per-Skipper check across
            // BOTH boards (a Trigger component cannot express this: TurnEnd
            // triggers are owner-scoped, so an enemy Skipper would never
            // fire for the active player). One Coin per Skipper on either
            // board goes to the active player (§20).
            {
                let skippers = state
                    .world()
                    .zones()
                    .iter(Zone::Play, active)
                    .chain(state.world().zones().iter(Zone::Play, active.opponent()))
                    .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "TIME_054"))
                    .count();
                for _ in 0..skippers {
                    let def = crate::cards::def::card_by_id("GAME_005");
                    if let Some(def) = def {
                        trigger::add_card_to_hand(state, active, def);
                    }
                }
            }
            // M3-W2a — TIME_700 Chronological Aura: "At the end of your
            // turn, summon a 3/5 Dragon with Taunt. Lasts 3 turns" — the
            // tick counter rides the player (the effect set it); each own
            // turn end summons the drake and decrements while > 0 (§20).
            if state.player(active).chronological_aura_ticks > 0 {
                {
                    let p = &mut state.make_mut().players[active.index()];
                    p.chronological_aura_ticks -= 1;
                }
                trigger::resolve_summon(
                    state,
                    queue,
                    state.player(active).hero,
                    active,
                    "TIME_700t",
                );
            }
            // M4-W4 — CATA_480 Sandfury Aura: "Your minions' end of turn
            // effects trigger twice. Lasts 3 turns." — while the counter
            // is positive the full TurnEnd trigger set fires a second time
            // (the TurnEnded handler consumes one tick per own turn end).
            if state.player(active).end_turn_effects_twice_turns > 0 {
                fire_triggers(state, queue, TriggerEvent::TurnEnd, active, None, None);
            }
            state.set_step(Step::WrapUp);
            true
        }
        Step::WrapUp => {
            // Expire "until end of turn" effects, then start the opponent's turn
            wrap_up_turn(state);
            queue.push(Event::TurnStarted {
                player: state.active_player().opponent(),
            });
            state.set_step(Step::StartTriggers);
            true
        }
        Step::Death => {
            // Process the pending deaths (batch): enqueue MinionDied in play
            // order, current player first. Stay in the death step while more
            // deaths surface (deathrattles can kill); return to the interrupted
            // step when the batch drains.
            if process_pending_deaths(state, queue) {
                true
            } else {
                let return_step = state.make_mut().return_step;
                state.set_step(return_step);
                true
            }
        }
        Step::Main => false,
    }
}

/// Enqueues `MinionDied` for all pending deaths, in play order with the current
/// player's minions first (HS death-phase resolution order). Returns whether
/// any death was enqueued.
///
/// The `MinionDied` handler re-checks health, so a minion healed after being
/// marked survives (roadmap G3).
fn process_pending_deaths(state: &mut GameState, queue: &mut EventQueue) -> bool {
    let active = state.active_player();
    let inner = state.make_mut();
    let pending = std::mem::take(&mut inner.pending_deaths);
    let mut any = false;
    for player in [active, active.opponent()] {
        // Play order: the zone table holds the board in summon order
        for entity in inner.world.zones().iter(Zone::Play, player) {
            if pending.contains(&entity) {
                queue.push(Event::MinionDied { minion: entity });
                any = true;
            }
        }
    }
    any
}

/// Fires registered triggers for the given event (roadmap G2).
///
/// Replaces the ad-hoc per-class trigger scans with per-entity trigger
/// registration (the unified `Trigger` component). Triggers fire in **play
/// order** (summon order on each board), with the **current player's triggers
/// first** and the opponent's second — HS's simultaneous-trigger precedence.
/// "Whenever" triggers fire before "after" triggers (the HS whenever/after
/// timing classification).
///
/// `event_owner` is the player the event involves (the spell caster, the
/// minion's owner, the turn player); only triggers whose scope includes them
/// fire. `subject` pins the event to one entity (`ThisMinionDamaged` — the
/// damaged minion). `exclude` skips one entity (the summoned/died minion).
pub fn fire_triggers(
    state: &mut GameState,
    queue: &mut EventQueue,
    event: TriggerEvent,
    event_owner: PlayerId,
    subject: Option<Entity>,
    exclude: Option<Entity>,
) {
    let active = state.active_player();
    for timing in [TriggerTiming::Whenever, TriggerTiming::After] {
        for player in [active, active.opponent()] {
            let triggers: SmallList<(Entity, crate::core::effect::CardEffect), 8> = state
                .world()
                .zones()
                .iter(Zone::Play, player)
                .filter(|&e| {
                    Some(e) != exclude
                        && state.world().is_alive(e)
                        && state.world().dormant(e).is_none()
                        && state.world().trigger(e).is_some_and(|t| {
                            t.event == event
                                && t.timing == timing
                                && trigger_applies(state, event, player, event_owner, subject, e, t)
                        })
                })
                .map(|e| {
                    let t = state
                        .world()
                        .trigger(e)
                        .expect("filter guarantees a trigger");
                    (e, t.effect)
                })
                .collect();
            for (source, effect) in triggers {
                trigger::resolve_effect(state, queue, source, player, effect, None, subject);
            }
        }
    }
}

/// Whether a trigger owned by `trigger_player` fires for the event.
///
/// Most trigger classes are friendly-scoped (the trigger's owner must be the
/// event's player); `ThisMinionDamaged` is pinned to the damaged minion itself.
/// A race-conditioned trigger (fidelity-debt W1 — Murloc Tidecaller, Starving
/// Buzzard, Scavenging Hyena) additionally requires the event subject to have
/// the trigger's race.
fn trigger_applies(
    state: &GameState,
    event: TriggerEvent,
    trigger_player: PlayerId,
    event_owner: PlayerId,
    subject: Option<Entity>,
    entity: Entity,
    trigger: crate::core::component::Trigger,
) -> bool {
    match event {
        // Pinned to the entity the event happened to (SurvivedDamage — the
        // M3-W2a survives-damage trigger — rides the damaged minion like
        // ThisMinionDamaged)
        TriggerEvent::ThisMinionDamaged | TriggerEvent::SurvivedDamage => {
            if Some(entity) != subject {
                return false;
            }
        }
        // Pinned to the attacker itself (Blessing of Wisdom), or to the
        // weapon the attacking hero has equipped (Truesilver Champion,
        // Gorehowl — weapon-attack effects ride the weapon entity)
        TriggerEvent::Attacked | TriggerEvent::AttackedMinion => {
            let pinned = Some(entity) == subject
                || subject.is_some_and(|s| {
                    state.world().card_type(s) == Some(CardType::Hero)
                        && state
                            .world()
                            .player(s)
                            .and_then(|pid| state.player(pid).weapon)
                            == Some(entity)
                });
            if !pinned {
                return false;
            }
        }
        // HeroAttackedMinion — the defender is the subject, pinned to the
        // defender itself or the attacking hero's equipped weapon (Defiled
        // Spear's splash rides the weapon entity)
        TriggerEvent::HeroAttackedMinion => {
            let pinned =
                Some(entity) == subject || state.player(event_owner).weapon == Some(entity);
            if !pinned {
                return false;
            }
        }
        // Global classes — fire regardless of who owns the event
        // (Lightwarden: any character healed; Northshire Cleric: any minion
        // healed; Secretkeeper: any Secret played; Flesheating Ghoul: any
        // minion died; Lorewalker Cho: any player's spell cast)
        TriggerEvent::CharacterHealed
        | TriggerEvent::MinionHealed
        | TriggerEvent::SecretPlayed
        | TriggerEvent::MinionDied
        | TriggerEvent::AnySpellCast
        | TriggerEvent::CardDrawn
        | TriggerEvent::DivineShieldLost => {}
        _ => {
            if trigger_player != event_owner {
                return false;
            }
        }
    }
    if let Some(race) = trigger.race {
        // The subject must exist and carry the required race (a dead subject
        // keeps its components in the graveyard, so its race is still readable)
        if !subject.is_some_and(|s| state.world().has_race(s, race)) {
            return false;
        }
    }
    if let Some(max_attack) = trigger.max_attack {
        // Warsong Commander — the summoned minion must have at most this much
        // Attack. Read at fire time, so a minion summoned small and buffed
        // afterwards still qualifies (and vice versa), matching HS.
        if !subject.is_some_and(|s| {
            state
                .world()
                .effective_attack(s)
                .is_some_and(|a| a.0 <= max_attack)
        }) {
            return false;
        }
    }
    true
}

/// Turn wrap-up: expires "until end of turn" effects and clears per-turn state.
///
/// Runs after the EndTriggers step so that end-of-turn effects resolve at full
/// strength and deaths they cause are processed before buffs expire.
/// Applies the M2-W4a modifiers a Discovered card carries when it lands in
/// hand: the Temporary marker (Bloodpetal Biome / Cursed Catacombs — the
/// discard-at-turn-end behaviour landed in W2), Vault Breaker's "-1 Cost"
/// (TLC_483 on the board — "After you Discover a card, reduce its Cost by
/// (1)", folded into the get per fidelity-debt §17), and Relic of Kings'
/// "costs (1)" set (TLC_334).
fn apply_w4a_discover_modifiers(
    state: &mut GameState,
    temporary: bool,
    source: Entity,
    picked: Entity,
) {
    if temporary {
        state
            .world_mut()
            .set_temporary(picked, crate::core::component::Temporary);
    }
    let player = state
        .world()
        .player(source)
        .unwrap_or(state.active_player());
    if state
        .world()
        .card_id(source)
        .is_some_and(|c| c.0 == "TLC_334")
    {
        // Relic of Kings: the Discovered spell costs (1).
        state.world_mut().set_cost(picked, Cost(1));
    }
    if state
        .world()
        .zones()
        .iter(Zone::Play, player)
        .any(|e| state.world().card_id(e).is_some_and(|c| c.0 == "TLC_483"))
    {
        // Vault Breaker on the board: the Discovered card costs (1) less.
        let cur = state.world().cost(picked).unwrap_or(Cost(0)).0;
        state.world_mut().set_cost(picked, Cost((cur - 1).max(0)));
    }
}

fn wrap_up_turn(state: &mut GameState) {
    let player = state.active_player();
    // Freeze timing (engine-mechanics roadmap M2): the entities this player
    // had frozen at the start of their turn — their attacks were blocked by
    // the AttackDeclared check — thaw here, in the turn-end wrap-up, so they
    // are unfrozen for the opponent's turn (HS: thaw after the missed attack
    // opportunity). Dead entities are skipped; the snapshot is drained.
    {
        let inner = state.make_mut();
        let frozen = std::mem::take(&mut inner.players[player.index()].frozen_at_turn_start);
        for e in frozen {
            if inner.world.is_alive(e) {
                inner.world.remove_freeze(e);
            }
        }
    }
    // Temporary cards (M2-W2 — the Un'Goro quest wave primitive): every
    // card in the player's hand carrying the Temporary marker is discarded
    // at the end of their turn (official rule). The marker's creators are
    // W4 cards; W2's F5 scenarios inject it directly.
    {
        let inner = state.make_mut();
        let temporary_hand: SmallList<Entity> = inner
            .world
            .iter_temporary()
            .filter(|(e, _)| inner.world.zone(*e) == Some(Zone::Hand))
            .filter(|(e, _)| inner.world.player(*e) == Some(player))
            .map(|(e, _)| e)
            .collect();
        for &e in temporary_hand.iter() {
            // The filter above already guarantees hand + owner, so the move
            // cannot fail for an alive entity.
            let _ = inner.world.move_to_zone(e, Zone::Graveyard);
        }
    }
    // Expire "until end of turn" enchantments (temporary attack buffs/debuffs)
    // and clear per-turn state
    {
        let inner = state.make_mut();
        let p = &mut inner.players[player.index()];
        // M2-W4a: "Discovered this turn" flags expire at the turn end (Storage
        // Scuffle's cost (0), Unearthed Artifacts' 4-Cost summon, Vault
        // Breaker's discount all read it). The Map-card chain (play the
        // discovered card this turn → also pick one of the others) expires
        // with it; Cower in Fear's next-Beast discount is this-turn only; the
        // sacred-cave recast list is per-turn too.
        p.discovered_this_turn = false;
        p.holy_cast_ids.clear();
        p.shadow_cast_this_turn = false;
        p.next_beast_discount = 0;
        p.map_pending = None;
        p.died_this_turn.clear();
        // Kindred (M2-W3): the activation condition is "played a card of
        // the same type earlier THIS TURN" — the played-type list resets
        // at the player's own turn end (the next-murloc and next-kindred
        // flags deliberately survive: they are consumed by the next play,
        // whenever that is).
        p.kindred_played.clear();
        // Millhouse Manastorm's zero-spell-cost window lasts one turn
        p.spells_cost_zero = false;
        // Preparation's next-spell discount also expires at the turn end
        p.next_spell_discount = 0;
        // Stack-buffered snapshot of entities holding expiring enchantments
        let expiring: SmallList<Entity> = inner
            .world
            .iter_enchantments()
            .filter(|(_, list)| {
                list.iter()
                    .any(|e| e.expiry == EnchantmentExpiry::UntilEndOfTurn)
            })
            .map(|(e, _)| e)
            .collect();
        for &e in expiring.iter() {
            inner
                .world
                .retain_enchantments(e, |ench| ench.expiry != EnchantmentExpiry::UntilEndOfTurn);
        }
        // Clear temporary immunity on all entities (Bestial Wrath — until end of turn)
        let immune_entities: SmallList<Entity> =
            inner.world.iter_immune().map(|(e, _)| e).collect();
        for &e in immune_entities.iter() {
            inner.world.remove_immune(e);
        }
        // M3-W2a — "can't attack heroes this turn" (PMM Infinitizer) and
        // "takes double damage" (Quantum Destabilizer — its permanent
        // marker is kept; see the filter below) expire with the turn. The
        // TurnCostReducer marker only counts while the card sits in hand.
        let cant_attack_heroes: SmallList<Entity> = inner
            .world
            .iter_cant_attack_heroes_this_turn()
            .map(|(e, _)| e)
            .collect();
        for &e in cant_attack_heroes.iter() {
            inner.world.remove_cant_attack_heroes_this_turn(e);
        }
        let double_damage: SmallList<Entity> = inner
            .world
            .iter_double_damage_taken()
            // Quantum Destabilizer's marker is PERMANENT (id-keyed carve-out
            // like Tichondrius — its "takes double damage" trait never
            // expires; the marker is applied at summon).
            .filter(|(e, _)| inner.world.card_id(*e).is_some_and(|c| c.0 != "TIME_060"))
            .map(|(e, _)| e)
            .collect();
        for &e in double_damage.iter() {
            inner.world.remove_double_damage_taken(e);
        }
        let reducers: SmallList<Entity> = inner
            .world
            .iter_turn_cost_reducer()
            .filter(|(e, _)| inner.world.zone(*e) != Some(Zone::Hand))
            .map(|(e, _)| e)
            .collect();
        for &e in reducers.iter() {
            inner.world.remove_turn_cost_reducer(e);
        }
        // Barbed Thorn's "Poisonous this turn" expires at the turn end (M1-W3)
        if p.hero_poisonous_this_turn {
            inner.world.remove_poison(p.hero);
            p.hero_poisonous_this_turn = false;
        }
        // CATA_530 Fel Infusion's "hero Lifesteal this turn" expires at the
        // turn end (M4-W2 — the GrantPoisonousThisTurn convention)
        if p.hero_lifesteal_this_turn {
            inner.world.remove_lifesteal(p.hero);
            p.hero_lifesteal_this_turn = false;
        }
        // CATA_496 Cursed Chains' "It can't attack this turn" marker
        // expires at the wrap-up step (M4-W4 — both players' entities:
        // the controlled minion sits on the active player's board).
        let cant_attack_now: SmallList<Entity> = inner
            .world
            .iter_cant_attack_this_turn()
            .map(|(e, _)| e)
            .collect();
        for &e in cant_attack_now.iter() {
            inner.world.remove_cant_attack_this_turn(e);
        }
    }
    // Return temporarily controlled minions (Shadow Madness — until end of turn)
    // (mem::take — zero allocation)
    let inner = state.make_mut();
    let controlled = std::mem::take(&mut inner.players[player.index()].controlled_this_turn);
    for (entity, original_owner) in controlled {
        if state.world().is_alive(entity) {
            trigger::transfer_minion(state, entity, original_owner);
        }
    }
    // Clear the commanding-shout minimum health effect (until end of turn)
    {
        let inner = state.make_mut();
        inner.players[player.index()].minion_min_health = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::{
        Attack, CardType, Cost, CostModifier, CostModifierKind, Damage, Enchantment,
        EnchantmentExpiry, Health, Trigger,
    };
    use crate::core::entity::Entity;
    use crate::core::state::GameState;
    use crate::core::zone::Zone;

    /// Helper: create a fully-componented minion on the battlefield
    fn add_minion_to_board(state: &mut GameState, player: PlayerId, atk: i32, hp: i32) -> Entity {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(hp));
        world.set_attack(e, Attack(atk));
        world.set_cost(e, crate::core::component::Cost(0));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, player);
        world.set_attacks_used(e, AttacksUsed(0));
        world.set_zone(e, Zone::Play);
        world.zones_mut().insert(Zone::Play, player, e);
        e
    }

    /// Helper: add a minion to hand
    fn add_minion_to_hand(state: &mut GameState, player: PlayerId, atk: i32, hp: i32) -> Entity {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(hp));
        world.set_attack(e, Attack(atk));
        world.set_cost(e, crate::core::component::Cost(3));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, player);
        world.set_attacks_used(e, AttacksUsed(0));
        world.set_zone(e, Zone::Hand);
        world.zones_mut().insert(Zone::Hand, player, e);
        // Give enough mana for testing
        state.make_mut().players[player.index()].current_mana = 10;
        e
    }

    #[test]
    fn validate_play_card_legal() {
        let mut state = GameState::new();
        let card = add_minion_to_hand(&mut state, PlayerId::Player1, 2, 3);
        let result = validate(
            &state,
            Action::PlayCard {
                card,
                target: None,
                position: None,
            },
        );
        assert!(result.is_ok(), "valid play should succeed: {result:?}");
    }

    #[test]
    fn validate_play_card_not_your_card() {
        let mut state = GameState::new();
        let card = add_minion_to_hand(&mut state, PlayerId::Player2, 2, 3);
        let result = validate(
            &state,
            Action::PlayCard {
                card,
                target: None,
                position: None,
            },
        );
        assert_eq!(result, Err(EngineError::NotYourCard));
    }

    #[test]
    fn validate_play_card_not_in_hand() {
        let mut state = GameState::new();
        let card = add_minion_to_board(&mut state, PlayerId::Player1, 2, 3);
        // On the battlefield, not in hand
        let result = validate(
            &state,
            Action::PlayCard {
                card,
                target: None,
                position: None,
            },
        );
        assert_eq!(result, Err(EngineError::CardNotInHand));
    }

    #[test]
    fn validate_play_card_board_full() {
        let mut state = GameState::new();
        // Fill the board with 7 minions
        for _ in 0..7 {
            add_minion_to_board(&mut state, PlayerId::Player1, 1, 1);
        }
        let card = add_minion_to_hand(&mut state, PlayerId::Player1, 2, 3);
        let result = validate(
            &state,
            Action::PlayCard {
                card,
                target: None,
                position: None,
            },
        );
        assert_eq!(result, Err(EngineError::BoardFull));
    }

    #[test]
    fn validate_attack_legal() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 3);
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 2, 2);
        let result = validate(&state, Action::Attack { attacker, defender });
        assert!(result.is_ok(), "valid attack should succeed: {result:?}");
    }

    #[test]
    fn validate_attack_own_minion() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 3);
        let defender = add_minion_to_board(&mut state, PlayerId::Player1, 2, 2);
        let result = validate(&state, Action::Attack { attacker, defender });
        assert_eq!(result, Err(EngineError::InvalidTarget));
    }

    #[test]
    fn validate_attack_hero_legal() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 3);
        let defender = state.player(PlayerId::Player2).hero;
        let result = validate(&state, Action::Attack { attacker, defender });
        assert!(result.is_ok(), "attacking hero should be valid: {result:?}");
    }

    #[test]
    fn validate_attack_twice() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 3);
        state.world_mut().set_attacks_used(attacker, AttacksUsed(1));
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 2, 2);
        let result = validate(&state, Action::Attack { attacker, defender });
        assert_eq!(result, Err(EngineError::AttacksExhausted));
    }

    #[test]
    fn validate_stale_entity() {
        let state = GameState::new();
        let entity = Entity::new(999, 0); // Non-existent entity
        let result = validate(
            &state,
            Action::PlayCard {
                card: entity,
                target: None,
                position: None,
            },
        );
        assert_eq!(result, Err(EngineError::EntityGone(entity)));
    }

    #[test]
    fn apply_trade_deals_damage_both_ways() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 5);
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 2, 3);

        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(&mut state, Action::Attack { attacker, defender })
            .unwrap();

        // defender takes 3 damage and dies (damage is cleared when it moves to
        // the graveyard, so the dead minion reads its base health)
        assert_eq!(
            state.world().zone(defender),
            Some(Zone::Graveyard),
            "defender should be dead in graveyard"
        );

        // attacker takes 2 damage: 5 → 3, survives
        assert_eq!(
            state.world().effective_health(attacker),
            Some(Health(3)),
            "attacker should have taken 2 damage"
        );
        assert_eq!(
            state.world().attacks_used(attacker),
            Some(AttacksUsed(1)),
            "attacker should have used attack"
        );
    }

    #[test]
    fn damage_hero() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 5, 3);
        let hero = state.player(PlayerId::Player2).hero;

        let mut queue = EventQueue::new();
        enqueue(
            &state,
            Action::Attack {
                attacker,
                defender: hero,
            },
            &mut queue,
        )
        .unwrap();

        while let Some(event) = queue.pop_front() {
            apply_event(&mut state, event, &mut queue).unwrap();
        }

        assert_eq!(state.world().effective_health(hero), Some(Health(25)));
        // The attacker should not take damage from the hero (heroes do not retaliate)
        assert_eq!(state.world().effective_health(attacker), Some(Health(3)));
    }

    #[test]
    fn game_over_on_hero_death() {
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 30, 3);
        let hero = state.player(PlayerId::Player2).hero;

        let mut queue = EventQueue::new();
        enqueue(
            &state,
            Action::Attack {
                attacker,
                defender: hero,
            },
            &mut queue,
        )
        .unwrap();

        while let Some(event) = queue.pop_front() {
            apply_event(&mut state, event, &mut queue).unwrap();
        }

        assert_eq!(
            state.step(),
            Step::GameOver {
                winner: PlayerId::Player1
            }
        );
    }

    #[test]
    fn attacker_dies_still_deals_damage() {
        // The attacker dies in the trade, but damage is still dealt (simultaneous resolution)
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 4, 1);
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 5, 10);

        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(&mut state, Action::Attack { attacker, defender })
            .unwrap();

        // defender takes 4 damage: 10 → 6
        assert_eq!(state.world().effective_health(defender), Some(Health(6)));
        // attacker takes 5 damage: 1 → -4 → dies
        assert_eq!(state.world().zone(attacker), Some(Zone::Graveyard));
        assert_eq!(state.world().zone(defender), Some(Zone::Play)); // defender survives
    }

    #[test]
    fn reset_attacks_on_turn_start() {
        let mut state = GameState::new();
        let minion = add_minion_to_board(&mut state, PlayerId::Player1, 2, 3);
        state.world_mut().set_attacks_used(minion, AttacksUsed(1));

        let mut queue = EventQueue::new();
        queue.push(Event::TurnStarted {
            player: PlayerId::Player1,
        });

        // Apply events
        while let Some(event) = queue.pop_front() {
            apply_event(&mut state, event, &mut queue).unwrap();
        }

        assert_eq!(state.world().attacks_used(minion), Some(AttacksUsed(0)));
    }

    // ============================================================
    // G1 — step state machine
    // ============================================================

    #[test]
    fn first_player_turn_one_has_one_mana() {
        // The first player's opening turn starts with 1 mana crystal (their
        // turn-1 refill is built into GameState::new).
        let state = GameState::new();
        assert_eq!(state.player(PlayerId::Player1).mana_crystals, 1);
        assert_eq!(state.player(PlayerId::Player1).current_mana, 1);
        assert_eq!(state.player(PlayerId::Player2).mana_crystals, 0);
    }

    #[test]
    fn mana_refill_runs_on_turn_start() {
        // Player2's turn 1 (turn counter 2) goes through the ManaRefill step: 0 → 1.
        let mut state = GameState::new();
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        assert_eq!(state.player(PlayerId::Player2).mana_crystals, 1);
        assert_eq!(state.player(PlayerId::Player2).current_mana, 1);
        // The machine returns to Main, waiting for input
        assert_eq!(state.step(), Step::Main);
    }

    #[test]
    fn first_player_draws_on_turn_one() {
        // Official rule: the first player draws the 4th card as their turn 1
        // starts. The shipped opening (sim::battle::build_game_state) deals it
        // during construction; a state entering DrawStep on turn 1 (e.g. a
        // constructed state) must draw as well — there is no turn-1 skip.
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_deck(PlayerId::Player1, &BLOODFEN_RAPTOR);
        let mut state = builder.build();
        assert_eq!(state.turn(), 1);
        state.set_step(Step::DrawStep);
        let mut queue = crate::core::event::EventQueue::new();
        assert!(advance_step(&mut state, &mut queue));
        assert_eq!(state.step(), Step::Main);
        assert_eq!(state.world().zones().len(Zone::Hand, PlayerId::Player1), 1);
        assert_eq!(state.world().zones().len(Zone::Deck, PlayerId::Player1), 0);
    }

    #[test]
    fn second_player_draws_on_first_turn_first_player_from_turn_two() {
        // Step-machine draw schedule: the second player draws on their first
        // turn (entered via TurnStarted); the first player draws from turn 2 on
        // (its turn-1 card is dealt during the opening construction).
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_deck(PlayerId::Player1, &BLOODFEN_RAPTOR);
        builder.add_minion_to_deck(PlayerId::Player2, &BLOODFEN_RAPTOR);
        let mut state = builder.build();
        let engine = crate::engine::game::GameEngine::new();

        // End Player1's opening turn: Player2 draws on its first turn
        engine.apply(&mut state, Action::EndTurn).unwrap();
        assert_eq!(state.world().zones().len(Zone::Deck, PlayerId::Player1), 1);
        assert_eq!(state.world().zones().len(Zone::Deck, PlayerId::Player2), 0);

        // End Player2's turn: Player1 draws on turn 2
        engine.apply(&mut state, Action::EndTurn).unwrap();
        assert_eq!(state.world().zones().len(Zone::Deck, PlayerId::Player1), 0);
        assert_eq!(state.world().zones().len(Zone::Hand, PlayerId::Player1), 1);
    }

    #[test]
    fn start_of_turn_effect_fires_before_draw() {
        // A start-of-turn effect (CardDef::start_turn_effect, wired in G1)
        // resolves in the StartTriggers step — before the mana refill and the
        // turn's draw both happened afterwards.
        let mut state = GameState::new();
        let minion = add_minion_to_board(&mut state, PlayerId::Player2, 1, 2);
        state.world_mut().set_trigger(
            minion,
            Trigger {
                event: TriggerEvent::TurnStart,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: crate::core::effect::CardEffect::GainStats {
                    attack: 1,
                    health: 1,
                    target: crate::core::effect::EffectTarget::Self_,
                },
            },
        );
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        // The start effect resolved
        assert_eq!(state.world().effective_attack(minion), Some(Attack(2)));
        assert_eq!(state.world().effective_health(minion), Some(Health(3)));
    }

    #[test]
    fn end_of_turn_effect_fires_before_wrap_up() {
        // End-of-turn effects must resolve at full strength, before "until end
        // of turn" buffs expire. The hero has a temporary attack bonus of 5 and
        // an end-of-turn effect dealing the hero's attack as damage: the enemy
        // minion must take the full 5 (if wrap-up ran first, it would take 0).
        let mut state = GameState::new();
        // Heroic-Strike-style temporary bonus: a +5 until-end-of-turn attack enchantment
        let hero = state.player(PlayerId::Player1).hero;
        state.world_mut().add_enchantment(
            hero,
            crate::core::component::Enchantment {
                attack: 5,
                health: 0,
                cost: 0,
                expiry: crate::core::component::EnchantmentExpiry::UntilEndOfTurn,
            },
        );
        state.world_mut().set_trigger(
            hero,
            Trigger {
                event: TriggerEvent::TurnEnd,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: crate::core::effect::CardEffect::DealHeroAttackDamage {
                    target: crate::core::effect::EffectTarget::AnyEnemy,
                },
            },
        );
        let enemy_minion = add_minion_to_board(&mut state, PlayerId::Player2, 1, 2);
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();

        // The effect fired with the full attack (5 damage kills the 2-HP minion)
        assert_eq!(
            state.world().zone(enemy_minion),
            Some(Zone::Graveyard),
            "end-of-turn effect must resolve before the temporary buff expires"
        );
        // The temp enchantment expired afterwards — the hero is back to base attack
        assert_eq!(
            state.world().effective_attack(hero),
            Some(Attack(0)),
            "temporary attack bonus must expire at wrap-up"
        );
    }

    #[test]
    fn turn_start_secret_fires_before_draw() {
        // A start-of-turn secret (OnFriendlyTurnStart) fires when the
        // TurnStarted event is processed — before the ManaRefill and DrawStep.
        // Its discard effect must run against the empty hand, leaving exactly
        // the turn's drawn card (if the draw happened first, the discard would
        // remove it).
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::core::component::{CardId, Secret, SecretTrigger};
        use crate::core::effect::CardEffect;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_deck(PlayerId::Player2, &BLOODFEN_RAPTOR);
        let mut state = builder.build();
        // A Player2 secret that discards a random card on their turn start
        let secret_entity = {
            let world = state.world_mut();
            let e = world.spawn();
            world.set_card_id(e, CardId("CLASSIC_001"));
            world.set_card_type(e, CardType::Spell);
            world.set_cost(e, crate::core::component::Cost(0));
            world.set_player(e, PlayerId::Player2);
            world.set_secret(
                e,
                Secret {
                    trigger: SecretTrigger::OnFriendlyTurnStart,
                    effect: Some(CardEffect::DiscardRandomCard),
                },
            );
            world.set_zone(e, Zone::SetAside);
            world
                .zones_mut()
                .insert(Zone::SetAside, PlayerId::Player2, e);
            e
        };
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();

        // The secret fired (revealed) and the draw still happened afterwards
        assert_eq!(state.world().zone(secret_entity), Some(Zone::Graveyard));
        assert_eq!(
            state.world().zones().len(Zone::Hand, PlayerId::Player2),
            1,
            "turn-start secret must fire before the draw"
        );
    }

    #[test]
    fn death_step_processes_kills_and_returns_to_main() {
        // Combat deaths surface while in the Main step: the machine enters the
        // Death step, batch-processes the MinionDied events, and returns to Main.
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 4, 5);
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 1, 2);
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(&mut state, Action::Attack { attacker, defender })
            .unwrap();
        assert_eq!(state.world().zone(defender), Some(Zone::Graveyard));
        assert_eq!(state.step(), Step::Main);
    }

    // ============================================================
    // G2 — registered triggers
    // ============================================================

    #[test]
    fn acolyte_of_pain_draws_when_damaged() {
        // Acolyte of Pain is a damage trigger, not a deathrattle (the old
        // mis-modeling is removed): it draws when it takes damage.
        use crate::cards::def::ACOLYTE_OF_PAIN;
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_board(PlayerId::Player1, &ACOLYTE_OF_PAIN);
        builder.add_minion_to_deck(PlayerId::Player1, &BLOODFEN_RAPTOR);
        let mut state = builder.build();
        // P2's attacker damages the Acolyte on P2's turn
        let attacker = add_minion_to_board(&mut state, PlayerId::Player2, 2, 3);
        let acolyte = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .find(|&e| {
                state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| c.0 == "NEUTRAL_004")
            })
            .expect("acolyte on board");
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker,
                    defender: acolyte,
                },
            )
            .unwrap();
        assert_eq!(
            state.world().zones().len(Zone::Hand, PlayerId::Player1),
            1,
            "Acolyte of Pain must draw when damaged"
        );
        assert_eq!(state.world().effective_health(acolyte), Some(Health(1)));
    }

    #[test]
    fn acolyte_of_pain_draws_before_dying_to_the_damage() {
        // HS semantics: the draw fires when the damage is applied, before the
        // death check — a lethal hit still draws.
        use crate::cards::def::ACOLYTE_OF_PAIN;
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_board(PlayerId::Player1, &ACOLYTE_OF_PAIN);
        builder.add_minion_to_deck(PlayerId::Player1, &BLOODFEN_RAPTOR);
        let mut state = builder.build();
        let acolyte = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .find(|&e| {
                state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| c.0 == "NEUTRAL_004")
            })
            .expect("acolyte on board");
        let attacker = add_minion_to_board(&mut state, PlayerId::Player2, 5, 3);
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker,
                    defender: acolyte,
                },
            )
            .unwrap();
        assert_eq!(
            state.world().zones().len(Zone::Hand, PlayerId::Player1),
            1,
            "Acolyte draws even when the damage is lethal"
        );
        assert_eq!(state.world().zone(acolyte), Some(Zone::Graveyard));
    }

    #[test]
    fn frothing_berserker_gains_attack_when_friendly_minion_damaged() {
        use crate::cards::def::FROTHING_BERSERKER;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_board(PlayerId::Player1, &FROTHING_BERSERKER);
        let mut state = builder.build();
        let frothing = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .find(|&e| {
                state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| c.0 == "WARRIOR_007")
            })
            .expect("frothing on board");
        let friendly_minion = add_minion_to_board(&mut state, PlayerId::Player1, 1, 3);
        let attacker = add_minion_to_board(&mut state, PlayerId::Player2, 1, 3);
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker,
                    defender: friendly_minion,
                },
            )
            .unwrap();
        assert_eq!(
            state.world().effective_attack(frothing),
            Some(Attack(3)),
            "Frothing Berserker gains +1 attack when a friendly minion takes damage"
        );
    }

    #[test]
    fn armorsmith_gains_armor_when_friendly_minion_damaged() {
        use crate::cards::def::ARMORSMITH;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_board(PlayerId::Player1, &ARMORSMITH);
        let mut state = builder.build();
        let friendly_minion = add_minion_to_board(&mut state, PlayerId::Player1, 1, 3);
        let attacker = add_minion_to_board(&mut state, PlayerId::Player2, 1, 3);
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        engine
            .apply(
                &mut state,
                Action::Attack {
                    attacker,
                    defender: friendly_minion,
                },
            )
            .unwrap();
        assert_eq!(
            state.player(PlayerId::Player1).armor,
            1,
            "Armorsmith grants 1 armor when a friendly minion takes damage"
        );
    }

    #[test]
    fn triggers_fire_in_play_order() {
        // Two friendly-summon triggers with order-observable effects: the first
        // played (A draws) must fire before the second (B discards) — the drawn
        // card is discarded, leaving the hand empty.
        use crate::core::effect::CardEffect;
        let mut state = GameState::new();
        let a = add_minion_to_board(&mut state, PlayerId::Player1, 1, 1);
        let b = add_minion_to_board(&mut state, PlayerId::Player1, 1, 1);
        state.world_mut().set_trigger(
            a,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::DrawCard { count: 1 },
            },
        );
        state.world_mut().set_trigger(
            b,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::DiscardRandomCard,
            },
        );
        // A deck with one card so A's draw has something to draw
        {
            let world = state.world_mut();
            let e = world.spawn();
            world.set_card_type(e, CardType::Minion);
            world.set_cost(e, crate::core::component::Cost(0));
            world.set_player(e, PlayerId::Player1);
            world.set_zone(e, Zone::Deck);
            world.zones_mut().insert(Zone::Deck, PlayerId::Player1, e);
        }
        // Play a third minion from hand to trigger FriendlyMinionSummoned
        let hand_minion = add_minion_to_hand(&mut state, PlayerId::Player1, 1, 1);
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: hand_minion,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        assert_eq!(
            state.world().zones().len(Zone::Hand, PlayerId::Player1),
            0,
            "triggers must fire in play order: A draws first, B discards it"
        );
    }

    #[test]
    fn whenever_triggers_fire_before_after() {
        // HS timing classification: "whenever" triggers fire before "after"
        // triggers, even when the "after" trigger was played first.
        use crate::core::effect::CardEffect;
        let mut state = GameState::new();
        // B is played first but is an "after" trigger; A is a "whenever" trigger
        let b = add_minion_to_board(&mut state, PlayerId::Player1, 1, 1);
        let a = add_minion_to_board(&mut state, PlayerId::Player1, 1, 1);
        state.world_mut().set_trigger(
            b,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::After,
                race: None,
                max_attack: None,
                effect: CardEffect::DiscardRandomCard,
            },
        );
        state.world_mut().set_trigger(
            a,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::DrawCard { count: 1 },
            },
        );
        {
            let world = state.world_mut();
            let e = world.spawn();
            world.set_card_type(e, CardType::Minion);
            world.set_cost(e, crate::core::component::Cost(0));
            world.set_player(e, PlayerId::Player1);
            world.set_zone(e, Zone::Deck);
            world.zones_mut().insert(Zone::Deck, PlayerId::Player1, e);
        }
        let hand_minion = add_minion_to_hand(&mut state, PlayerId::Player1, 1, 1);
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: hand_minion,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        // Whenever (A) drew, then After (B) discarded it
        assert_eq!(
            state.world().zones().len(Zone::Hand, PlayerId::Player1),
            0,
            "whenever triggers must fire before after triggers"
        );
    }

    // ============================================================
    // G3 — death batching
    // ============================================================

    /// Plays a custom P1 minion whose battlecry deals `amount` damage to all
    /// enemy minions, returning the battlecry minion.
    fn play_aoe_battlecry(state: &mut GameState, amount: i32) -> Entity {
        use crate::core::component::Battlecry;
        use crate::core::effect::CardEffect;
        let aoe = {
            let world = state.world_mut();
            let e = world.spawn();
            world.set_health(e, Health(2));
            world.set_attack(e, Attack(1));
            world.set_cost(e, crate::core::component::Cost(1));
            world.set_card_type(e, CardType::Minion);
            world.set_player(e, PlayerId::Player1);
            world.set_attacks_used(e, AttacksUsed(0));
            world.set_battlecry(
                e,
                Battlecry(CardEffect::DealDamage {
                    amount,
                    target: crate::core::effect::EffectTarget::AllEnemyMinions,
                }),
            );
            world.set_zone(e, Zone::Hand);
            world.zones_mut().insert(Zone::Hand, PlayerId::Player1, e);
            e
        };
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                state,
                Action::PlayCard {
                    card: aoe,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        aoe
    }

    #[test]
    fn healed_pending_death_survives() {
        // Death batching (G3): a minion marked dead stays on the battlefield
        // until the death step processes it. B (played first, deathrattle heals
        // all friendly minions) dies before A; its deathrattle heals A above 0,
        // so A's death is re-checked and A survives.
        use crate::core::component::Deathrattle;
        use crate::core::effect::CardEffect;
        let mut state = GameState::new();
        // B (healer) is played first, A (victim) second — B's death processes first
        let b = add_minion_to_board(&mut state, PlayerId::Player2, 0, 1);
        state.world_mut().set_deathrattle(
            b,
            Deathrattle(CardEffect::RestoreHealth {
                amount: 10,
                target: crate::core::effect::EffectTarget::AllFriendlyMinions,
            }),
        );
        let a = add_minion_to_board(&mut state, PlayerId::Player2, 0, 1);
        // P1 plays an AOE battlecry dealing 1 damage to both
        play_aoe_battlecry(&mut state, 1);

        assert_eq!(
            state.world().zone(a),
            Some(Zone::Play),
            "A must survive: healed above 0 before its death was processed"
        );
        // Damage dropped A to 0; the heal restored it to full (heals cap at
        // full health in the damage model)
        assert_eq!(state.world().effective_health(a), Some(Health(1)));
        assert_eq!(state.world().zone(b), Some(Zone::Graveyard));
        assert_eq!(state.step(), Step::Main);
    }

    #[test]
    fn deaths_process_in_play_order() {
        // The death phase processes deaths in play order: A (played first,
        // deathrattle draws) dies before B (deathrattle discards), so the drawn
        // card is discarded — if B processed first, the hand would keep it.
        use crate::core::component::Deathrattle;
        use crate::core::effect::CardEffect;
        let mut state = GameState::new();
        let a = add_minion_to_board(&mut state, PlayerId::Player2, 0, 1);
        state
            .world_mut()
            .set_deathrattle(a, Deathrattle(CardEffect::DrawCard { count: 1 }));
        let b = add_minion_to_board(&mut state, PlayerId::Player2, 0, 1);
        state
            .world_mut()
            .set_deathrattle(b, Deathrattle(CardEffect::DiscardRandomCard));
        // P2's deck with one card for A's draw
        {
            let world = state.world_mut();
            let e = world.spawn();
            world.set_card_type(e, CardType::Minion);
            world.set_cost(e, crate::core::component::Cost(0));
            world.set_player(e, PlayerId::Player2);
            world.set_zone(e, Zone::Deck);
            world.zones_mut().insert(Zone::Deck, PlayerId::Player2, e);
        }
        play_aoe_battlecry(&mut state, 1);

        assert_eq!(
            state.world().zones().len(Zone::Hand, PlayerId::Player2),
            0,
            "deaths must process in play order: A's draw is discarded by B"
        );
        assert_eq!(state.world().zone(a), Some(Zone::Graveyard));
        assert_eq!(state.world().zone(b), Some(Zone::Graveyard));
    }

    #[test]
    fn deathrattle_resolves_after_minion_removed() {
        // Death triggers see the dead minion already removed: a minion with a
        // FriendlyMinionDied trigger on ITSELF does not fire (the dead minion is
        // excluded), and the deathrattle resolves after the move to the graveyard.
        use crate::core::component::Deathrattle;
        use crate::core::effect::CardEffect;
        let mut state = GameState::new();
        let victim = add_minion_to_board(&mut state, PlayerId::Player2, 0, 1);
        state.world_mut().set_deathrattle(
            victim,
            Deathrattle(CardEffect::SummonMinion {
                card_id: "CLASSIC_001",
            }),
        );
        let hand_before = state.world().zones().len(Zone::Hand, PlayerId::Player1);
        let _ = hand_before;
        play_aoe_battlecry(&mut state, 1);
        // The deathrattle fired (a minion was summoned) — and the dead minion is gone
        assert_eq!(state.world().zone(victim), Some(Zone::Graveyard));
        assert_eq!(
            state
                .world()
                .zones()
                .iter(Zone::Play, PlayerId::Player2)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .count(),
            1,
            "deathrattle summon must resolve after the minion is removed"
        );
    }

    // ============================================================
    // G4 — enchantment layer
    // ============================================================

    /// Plays a custom P1 minion whose battlecry is `effect`, returning it.
    fn play_battlecry_minion(
        state: &mut GameState,
        effect: crate::core::effect::CardEffect,
    ) -> Entity {
        use crate::core::component::Battlecry;
        let e = {
            let world = state.world_mut();
            let e = world.spawn();
            world.set_health(e, Health(2));
            world.set_attack(e, Attack(1));
            world.set_cost(e, crate::core::component::Cost(1));
            world.set_card_type(e, CardType::Minion);
            world.set_player(e, PlayerId::Player1);
            world.set_attacks_used(e, AttacksUsed(0));
            world.set_battlecry(e, Battlecry(effect));
            world.set_zone(e, Zone::Hand);
            world.zones_mut().insert(Zone::Hand, PlayerId::Player1, e);
            e
        };
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                state,
                Action::PlayCard {
                    card: e,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        e
    }

    #[test]
    fn buffs_and_damage_are_enchantments_and_damage_component() {
        // G4: base stats stay untouched; buffs attach enchantments, damage
        // accumulates on the damage component.
        let mut state = GameState::new();
        let minion = add_minion_to_board(&mut state, PlayerId::Player1, 2, 5);
        state.world_mut().add_enchantment(
            minion,
            Enchantment {
                attack: 3,
                health: 2,
                cost: 0,
                expiry: EnchantmentExpiry::Permanent,
            },
        );
        state.world_mut().set_damage(minion, Damage(2));
        // Base values untouched
        assert_eq!(state.world().attack(minion), Some(Attack(2)));
        assert_eq!(state.world().health(minion), Some(Health(5)));
        // Effective values: base + enchantments − damage
        assert_eq!(state.world().effective_attack(minion), Some(Attack(5)));
        assert_eq!(state.world().effective_health(minion), Some(Health(5)));
    }

    #[test]
    fn silence_strips_enchantments_but_keeps_damage() {
        // G4: silence removes enchantments, keeping base stats and accumulated damage
        let mut state = GameState::new();
        let target = add_minion_to_board(&mut state, PlayerId::Player2, 2, 5);
        state.world_mut().add_enchantment(
            target,
            Enchantment {
                attack: 3,
                health: 3,
                cost: 0,
                expiry: EnchantmentExpiry::Permanent,
            },
        );
        state.world_mut().set_damage(target, Damage(2));
        assert_eq!(state.world().effective_attack(target), Some(Attack(5)));
        assert_eq!(state.world().effective_health(target), Some(Health(6)));
        // Silence via a custom battlecry
        play_battlecry_minion(
            &mut state,
            crate::core::effect::CardEffect::SilenceMinion {
                target: crate::core::effect::EffectTarget::AnyEnemyMinion,
            },
        );
        assert_eq!(
            state.world().effective_attack(target),
            Some(Attack(2)),
            "silence must strip enchantments, revealing the base attack"
        );
        assert_eq!(
            state.world().effective_health(target),
            Some(Health(3)),
            "silence keeps the accumulated damage (5 − 2)"
        );
        assert!(state.world().enchantments(target).is_none());
    }

    #[test]
    fn until_end_of_turn_enchantments_expire_at_wrap_up() {
        // G4: "until end of turn" enchantments expire at wrap-up; permanent
        // enchantments survive it.
        let mut state = GameState::new();
        let minion = add_minion_to_board(&mut state, PlayerId::Player1, 2, 3);
        state.world_mut().add_enchantment(
            minion,
            Enchantment {
                attack: 3,
                health: 0,
                cost: 0,
                expiry: EnchantmentExpiry::UntilEndOfTurn,
            },
        );
        state.world_mut().add_enchantment(
            minion,
            Enchantment {
                attack: 1,
                health: 0,
                cost: 0,
                expiry: EnchantmentExpiry::Permanent,
            },
        );
        assert_eq!(state.world().effective_attack(minion), Some(Attack(6)));
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        assert_eq!(
            state.world().effective_attack(minion),
            Some(Attack(3)),
            "the until-end-of-turn enchantment must expire at wrap-up"
        );
    }

    #[test]
    fn leaving_play_clears_enchantments_and_damage() {
        // G4: a minion leaving the battlefield loses enchantments and damage —
        // bounced minions return at full health with base stats.
        let mut state = GameState::new();
        let minion = add_minion_to_board(&mut state, PlayerId::Player1, 2, 5);
        state.world_mut().add_enchantment(
            minion,
            Enchantment {
                attack: 2,
                health: 3,
                cost: 0,
                expiry: EnchantmentExpiry::Permanent,
            },
        );
        state.world_mut().set_damage(minion, Damage(2));
        assert_eq!(state.world().effective_attack(minion), Some(Attack(4)));
        state
            .world_mut()
            .move_to_zone(minion, Zone::Hand)
            .expect("bounce");
        assert_eq!(state.world().effective_attack(minion), Some(Attack(2)));
        assert_eq!(state.world().effective_health(minion), Some(Health(5)));
        assert!(state.world().enchantments(minion).is_none());
        assert!(state.world().damage(minion).is_none());
    }

    #[test]
    fn shadowstep_cost_enchantment_applies_after_bounce() {
        // G4 end-to-end: Shadowstep bounces a buffed, damaged minion — the
        // bounce clears enchantments and damage, then the bounce effect
        // re-applies its cost reduction as a cost enchantment.
        use crate::cards::def::SHADOWSTEP;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_board(PlayerId::Player1, &crate::cards::def::BLOODFEN_RAPTOR);
        builder.add_minion_to_hand(PlayerId::Player1, &SHADOWSTEP);
        let mut state = builder.build();
        let minion = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .find(|&e| {
                state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| c.0 == "CLASSIC_001")
            })
            .expect("raptor on board");
        let shadowstep = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .find(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "ROGUE_019"))
            .expect("shadowstep in hand");
        // Buff +2/+2 and damage the minion
        state.world_mut().add_enchantment(
            minion,
            Enchantment {
                attack: 2,
                health: 2,
                cost: 0,
                expiry: EnchantmentExpiry::Permanent,
            },
        );
        state.world_mut().set_damage(minion, Damage(1));
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: shadowstep,
                    target: Some(minion),
                    position: None,
                },
            )
            .unwrap();
        assert_eq!(state.world().zone(minion), Some(Zone::Hand));
        assert_eq!(
            state.world().effective_attack(minion),
            Some(Attack(3)),
            "bounced minion must lose its buffs (raptor base attack)"
        );
        assert_eq!(
            state.world().effective_health(minion),
            Some(Health(2)),
            "bounced minion must be at full health (raptor base health)"
        );
        assert_eq!(
            state.world().effective_cost(minion),
            Some(crate::core::component::Cost(0)),
            "Shadowstep's cost reduction persists in hand (3 − 2)"
        );
    }

    // ============================================================
    // G5 — cost manager
    // ============================================================

    #[test]
    fn cost_modifier_stack_set_and_floor() {
        // G5: the cost stack composes base + enchantment deltas, then applies
        // set-to-value and floor modifiers.
        let mut state = GameState::new();
        let card = add_minion_to_hand(&mut state, PlayerId::Player1, 2, 3); // cost 3
        assert_eq!(state.world().effective_cost(card), Some(Cost(3)));
        // −2 enchantment → 1
        state.world_mut().add_enchantment(
            card,
            Enchantment {
                attack: 0,
                health: 0,
                cost: -2,
                expiry: EnchantmentExpiry::Permanent,
            },
        );
        assert_eq!(state.world().effective_cost(card), Some(Cost(1)));
        // Min(3) floor raises it back to 3
        state.world_mut().add_cost_modifier(
            card,
            CostModifier {
                kind: CostModifierKind::Min(3),
            },
        );
        assert_eq!(state.world().effective_cost(card), Some(Cost(3)));
        // Set(5) overrides the composed value
        state.world_mut().add_cost_modifier(
            card,
            CostModifier {
                kind: CostModifierKind::Set(5),
            },
        );
        assert_eq!(state.world().effective_cost(card), Some(Cost(5)));
        // Removing the stack restores the enchantment-composed cost
        state.world_mut().remove_cost_modifiers(card);
        assert_eq!(state.world().effective_cost(card), Some(Cost(1)));
    }

    #[test]
    fn cost_cannot_go_below_zero() {
        let mut state = GameState::new();
        let card = add_minion_to_hand(&mut state, PlayerId::Player1, 2, 3); // cost 3
        state.world_mut().add_enchantment(
            card,
            Enchantment {
                attack: 0,
                health: 0,
                cost: -10,
                expiry: EnchantmentExpiry::Permanent,
            },
        );
        assert_eq!(
            state.world().effective_cost(card),
            Some(Cost(0)),
            "costs cannot go below 0"
        );
    }

    #[test]
    fn play_cost_composes_player_level_modifiers() {
        // G5: cost::play_cost is the single composition — Kirin Tor Mage's
        // one-time free secret applies at the player level.
        use crate::cards::def::EXPLOSIVE_TRAP;
        use crate::cards::def::KIRIN_TOR_MAGE;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &EXPLOSIVE_TRAP);
        builder.add_minion_to_hand(PlayerId::Player1, &KIRIN_TOR_MAGE);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let mut state = builder.build();
        let trap = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .find(|&e| {
                state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| c.0 == "HUNTER_T01")
            })
            .expect("trap in hand");
        assert_eq!(
            crate::engine::cost::play_cost(&state, trap, PlayerId::Player1),
            Cost(2)
        );
        // Kirin Tor's one-time free secret (played first)
        let kirin = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .find(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "MAGE_021"))
            .expect("kirin in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: kirin,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        assert!(state.player(PlayerId::Player1).next_secret_free);
        assert_eq!(
            crate::engine::cost::play_cost(&state, trap, PlayerId::Player1),
            Cost(0),
            "the next secret costs 0"
        );
    }

    // ============================================================
    // G6 — choice system
    // ============================================================

    #[test]
    fn choose_one_choice_surfaces_and_resolves_branch() {
        // G6: a Choose One card surfaces a pending choice via apply_choices;
        // Action::Choose resolves the chosen branch.
        use crate::cards::def::CENARIUS;
        use crate::engine::game::Resolution;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &CENARIUS);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let mut state = builder.build();
        let cenarius = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("cenarius in hand");
        let engine = crate::engine::game::GameEngine::new();
        let resolution = engine
            .apply_choices(
                &mut state,
                Action::PlayCard {
                    card: cenarius,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        let Resolution::NeedsChoice { choice } = resolution else {
            panic!("a choose-one card must surface a choice");
        };
        assert_eq!(choice.kind, ChoiceKind::ChooseOne);
        assert_eq!(choice.card, cenarius);
        assert_eq!(choice.options.len(), 2);
        // Resolve the second branch (summon two treants)
        let resolution = engine
            .apply_choices(
                &mut state,
                Action::Choose {
                    choice_id: choice.id,
                    option: 1,
                },
            )
            .unwrap();
        assert!(matches!(resolution, Resolution::Done(_)));
        let minions: SmallList<Entity> = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
            .collect();
        assert_eq!(
            minions.len(),
            3,
            "the second branch summons two treants next to Cenarius"
        );
    }

    #[test]
    fn choose_one_invalid_choice_rejected() {
        use crate::cards::def::CENARIUS;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &CENARIUS);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let mut state = builder.build();
        let cenarius = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("cenarius in hand");
        let engine = crate::engine::game::GameEngine::new();
        let _ = engine
            .apply_choices(
                &mut state,
                Action::PlayCard {
                    card: cenarius,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        // A stale choice id is rejected
        let err = engine
            .apply_choices(
                &mut state,
                Action::Choose {
                    choice_id: 999,
                    option: 0,
                },
            )
            .unwrap_err();
        assert_eq!(err, EngineError::InvalidChoice);
    }

    #[test]
    fn discover_choice_picks_a_pool_card() {
        // G6: an AddRandomCardToHand effect surfaces a Discover choice with the
        // pool as options; the chosen card is added to hand.
        use crate::engine::game::Resolution;
        let mut state = GameState::new();
        let source = add_minion_to_board(&mut state, PlayerId::Player1, 2, 2);
        let dream_pool = crate::cards::pool::pool_cards(crate::core::effect::RandomPool::Dream);
        assert!(!dream_pool.is_empty());
        let chosen = dream_pool[0];
        // Simulate the discover flow: set the pending choice as the resolver would
        let choice_id = state.set_pending_choice(
            ChoiceKind::Discover,
            source,
            dream_pool
                .iter()
                .map(|c| format!("{} ({})", c.name, c.id))
                .collect(),
            dream_pool.iter().map(|c| c.id.to_string()).collect(),
        );
        let engine = crate::engine::game::GameEngine::new();
        let resolution = engine
            .apply_choices(
                &mut state,
                Action::Choose {
                    choice_id,
                    option: 0,
                },
            )
            .unwrap();
        assert!(matches!(resolution, Resolution::Done(_)));
        let hand: SmallList<Entity> = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .collect();
        assert_eq!(hand.len(), 1);
        assert_eq!(
            state.world().card_id(hand[0]),
            Some(crate::core::component::CardId(chosen.id)),
            "the chosen pool card is added to hand"
        );
    }

    #[test]
    fn summon_position_places_minion() {
        // G6: PlayCard's position picks the board slot (0 = leftmost)
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_board(PlayerId::Player1, &BLOODFEN_RAPTOR);
        builder.add_minion_to_hand(PlayerId::Player1, &BLOODFEN_RAPTOR);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let mut state = builder.build();
        let existing = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .next()
            .expect("board minion");
        let card = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("hand minion");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card,
                    target: None,
                    position: Some(0),
                },
            )
            .unwrap();
        let order: SmallList<Entity> = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .collect();
        assert_eq!(order[0], card, "position 0 places the minion leftmost");
        assert_eq!(order[1], existing);
    }

    #[test]
    fn hero_power_explicit_target() {
        // G6: Action::HeroPower carries an explicit target (engine-random otherwise)
        use crate::core::effect::CardEffect;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.set_hero_power(
            PlayerId::Player1,
            2,
            CardEffect::DealDamage {
                amount: 2,
                target: crate::core::effect::EffectTarget::AnyEnemy,
            },
        );
        builder.set_mana(PlayerId::Player1, 10, 10);
        let mut state = builder.build();
        let hero = state.player(PlayerId::Player1).hero;
        let enemy_minion = add_minion_to_board(&mut state, PlayerId::Player2, 2, 4);
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::HeroPower {
                    hero,
                    target: Some(enemy_minion),
                },
            )
            .unwrap();
        assert_eq!(
            state.world().effective_health(enemy_minion),
            Some(Health(2)),
            "the explicit hero power target takes the damage"
        );
    }

    // ============================================================
    // G7 — opening flow
    // ============================================================

    /// Spawns a deck card with a known identity for the player (no shuffle).
    fn add_raw_deck_card(state: &mut GameState, player: PlayerId, card_id: &'static str) -> Entity {
        let world = state.world_mut();
        let e = world.spawn();
        world.set_card_id(e, crate::core::component::CardId(card_id));
        world.set_card_type(e, CardType::Minion);
        world.set_attack(e, Attack(1));
        world.set_health(e, Health(1));
        world.set_cost(e, crate::core::component::Cost(1));
        world.set_player(e, player);
        world.set_zone(e, Zone::Deck);
        world.zones_mut().insert(Zone::Deck, player, e);
        e
    }

    #[test]
    fn top_draw_uses_deck_order() {
        // G7: draws take the top card of the ordered deck (the random element
        // moved to the game-start shuffle).
        let mut state = GameState::new();
        add_raw_deck_card(&mut state, PlayerId::Player2, "NEUTRAL_001");
        add_raw_deck_card(&mut state, PlayerId::Player2, "NEUTRAL_002");
        let engine = crate::engine::game::GameEngine::new();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        // P2 drew the top card (first spawned)
        let hand: SmallList<Entity> = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player2)
            .collect();
        assert_eq!(hand.len(), 1);
        assert_eq!(
            state.world().card_id(hand[0]),
            Some(crate::core::component::CardId("NEUTRAL_001"))
        );
    }

    #[test]
    fn begin_game_deals_starting_hands_and_coin() {
        // G7: 3 cards to the first player, 4 + The Coin to the second
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        for _ in 0..10 {
            builder.add_minion_to_deck(PlayerId::Player1, &BLOODFEN_RAPTOR);
            builder.add_minion_to_deck(PlayerId::Player2, &BLOODFEN_RAPTOR);
        }
        let mut state = builder.build();
        assert!(state.pending_choice().is_none());
        state.begin_game();
        assert_eq!(state.world().zones().len(Zone::Hand, PlayerId::Player1), 3);
        assert_eq!(state.world().zones().len(Zone::Hand, PlayerId::Player2), 5);
        let coin = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player2)
            .find(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "GAME_005"));
        assert!(coin.is_some(), "the second player gets The Coin");
        // P1's mulligan surfaces as a pending choice
        let choice = state.pending_choice().expect("mulligan pending");
        assert_eq!(choice.kind, ChoiceKind::Mulligan);
        assert_eq!(choice.options.len(), 4, "3 replace options + keep all");
    }

    #[test]
    fn mulligan_replaces_a_card() {
        // G7: resolving the mulligan replaces the chosen card — the hand keeps
        // its size and the replaced card returns to the deck.
        use crate::cards::def::BLOODFEN_RAPTOR;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        for _ in 0..10 {
            builder.add_minion_to_deck(PlayerId::Player1, &BLOODFEN_RAPTOR);
            builder.add_minion_to_deck(PlayerId::Player2, &BLOODFEN_RAPTOR);
        }
        let mut state = builder.build();
        state.begin_game();
        let choice_id = state.pending_choice().expect("mulligan").id;
        let replaced = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("starting card");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply_choices(
                &mut state,
                Action::Choose {
                    choice_id,
                    option: 1, // replace the first starting card
                },
            )
            .unwrap();
        // Hand size unchanged; the replaced card went back to the deck
        assert_eq!(state.world().zones().len(Zone::Hand, PlayerId::Player1), 3);
        assert_eq!(state.world().zone(replaced), Some(Zone::Deck));
        // P2's mulligan surfaces next
        assert_eq!(
            state.pending_choice().map(|c| c.kind),
            Some(ChoiceKind::Mulligan)
        );
        // Resolve P2's mulligan (keep all) — the opening finishes
        let choice_id = state.pending_choice().expect("mulligan").id;
        engine
            .apply_choices(
                &mut state,
                Action::Choose {
                    choice_id,
                    option: 0,
                },
            )
            .unwrap();
        assert!(state.pending_choice().is_none(), "opening complete");
        assert_eq!(state.step(), Step::Main);
        // Official rule: as the opening finishes, the first player draws the
        // 4th card as their turn 1 starts
        assert_eq!(state.world().zones().len(Zone::Hand, PlayerId::Player1), 4);
    }

    #[test]
    fn coin_gives_mana_this_turn_only() {
        // G7: The Coin adds +1 mana this turn without a permanent crystal
        use crate::cards::def::THE_COIN;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &THE_COIN);
        builder.set_mana(PlayerId::Player1, 1, 1);
        let mut state = builder.build();
        let coin = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("coin in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: coin,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        assert_eq!(state.player(PlayerId::Player1).current_mana, 2);
        assert_eq!(
            state.player(PlayerId::Player1).mana_crystals,
            1,
            "the coin does not add a permanent crystal"
        );
    }

    // ============================================================
    // G8 — secret interception at step boundaries
    // ============================================================

    /// Adds a custom spell to the player's hand (type + effect set manually).
    fn add_custom_spell(
        state: &mut GameState,
        player: PlayerId,
        effect: crate::core::effect::CardEffect,
    ) -> Entity {
        use crate::core::component::Battlecry;
        let world = state.world_mut();
        let e = world.spawn();
        world.set_card_type(e, CardType::Spell);
        world.set_cost(e, crate::core::component::Cost(1));
        world.set_player(e, player);
        world.set_battlecry(e, Battlecry(effect));
        world.set_zone(e, Zone::Hand);
        world.zones_mut().insert(Zone::Hand, player, e);
        e
    }

    #[test]
    fn counterspell_negates_enemy_spell() {
        // G8: Counterspell intercepts the enemy spell BEFORE its effect resolves
        use crate::cards::def::COUNTERSPELL;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &COUNTERSPELL);
        builder.set_mana(PlayerId::Player1, 10, 10);
        builder.set_mana(PlayerId::Player2, 10, 10);
        let minion = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 3, 3);
        let mut state = builder.build();
        let counterspell = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("counterspell in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: counterspell,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        // P2 casts a damaging spell — it is negated
        state.set_active_player(PlayerId::Player2);
        let spell = add_custom_spell(
            &mut state,
            PlayerId::Player2,
            crate::core::effect::CardEffect::DealDamage {
                amount: 5,
                target: crate::core::effect::EffectTarget::AnyEnemyMinion,
            },
        );
        let log = engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: spell,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        assert!(
            log.iter()
                .any(|e| matches!(e, Event::SecretRevealed { .. })),
            "counterspell must be revealed"
        );
        assert_eq!(
            state.world().effective_health(minion),
            Some(Health(3)),
            "the countered spell's effect must not resolve"
        );
        assert_eq!(
            state.world().zone(spell),
            Some(Zone::Graveyard),
            "the countered spell still goes to the graveyard"
        );
    }

    #[test]
    fn spellbender_does_not_redirect_aoe_spells() {
        // G8: Spellbender only redirects single-target effects — AOE spells hit
        // normally (including the token), matching HS
        use crate::cards::def::SPELLBENDER;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &SPELLBENDER);
        builder.set_mana(PlayerId::Player1, 10, 10);
        builder.set_mana(PlayerId::Player2, 10, 10);
        let minion = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 3, 3);
        let mut state = builder.build();
        let spellbender = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("spellbender in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: spellbender,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        state.set_active_player(PlayerId::Player2);
        let spell = add_custom_spell(
            &mut state,
            PlayerId::Player2,
            crate::core::effect::CardEffect::DealDamage {
                amount: 2,
                target: crate::core::effect::EffectTarget::AllEnemyMinions,
            },
        );
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: spell,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        assert_eq!(
            state.world().effective_health(minion),
            Some(Health(1)),
            "AOE spells are not redirected by Spellbender"
        );
    }

    // ============================================================
    // G9 — target legality at resolution
    // ============================================================

    #[test]
    fn stealthed_minion_cannot_be_explicitly_targeted() {
        // G9: single-target effects cannot target stealthed characters — an
        // explicit stealthed target makes the effect fizzle (no random fallback)
        use crate::cards::def::FROSTBOLT;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &FROSTBOLT);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let stealthed = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 5, 3);
        let visible = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 5, 3);
        let mut state = builder.build();
        state
            .world_mut()
            .set_stealth(stealthed, crate::core::component::Stealth);
        let frostbolt = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("frostbolt in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: frostbolt,
                    target: Some(stealthed),
                    position: None,
                },
            )
            .unwrap();
        assert_eq!(
            state.world().effective_health(stealthed),
            Some(Health(5)),
            "the stealthed minion cannot be targeted — the spell fizzles"
        );
        assert_eq!(
            state.world().effective_health(visible),
            Some(Health(5)),
            "no random fallback to another target"
        );
    }

    // ============================================================
    // F1 — overload mana lock
    // ============================================================

    #[test]
    fn overload_locks_mana_on_next_turn() {
        // F1: Lightning Bolt (overload 1) locks 1 mana on the owner's NEXT
        // turn — applied at the ManaRefill step and cleared afterwards.
        use crate::cards::def::LIGHTNING_BOLT;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &LIGHTNING_BOLT);
        // Turn 1: 1 mana
        let mut state = builder.build();
        let bolt = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("lightning bolt in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: bolt,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        assert_eq!(
            state.player(PlayerId::Player1).overload_locked,
            1,
            "playing an overload card locks mana for the next turn"
        );
        // P2's turn, then P1's next turn (turn 3): crystals 2, mana 2 − 1 lock
        engine.apply(&mut state, Action::EndTurn).unwrap();
        engine.apply(&mut state, Action::EndTurn).unwrap();
        assert_eq!(state.player(PlayerId::Player1).mana_crystals, 2);
        assert_eq!(
            state.player(PlayerId::Player1).current_mana,
            1,
            "the overload lock reduces the next turn's mana"
        );
        assert_eq!(
            state.player(PlayerId::Player1).overload_locked,
            0,
            "the lock is consumed at the refill"
        );
    }

    // ============================================================
    // F2 — stealth fidelity
    // ============================================================

    #[test]
    fn attacking_breaks_stealth() {
        // F2: a stealthed minion loses stealth when it attacks
        let mut state = GameState::new();
        let attacker = add_minion_to_board(&mut state, PlayerId::Player1, 3, 3);
        let defender = add_minion_to_board(&mut state, PlayerId::Player2, 2, 5);
        state
            .world_mut()
            .set_stealth(attacker, crate::core::component::Stealth);
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(&mut state, Action::Attack { attacker, defender })
            .unwrap();
        assert!(
            state.world().stealth(attacker).is_none(),
            "attacking breaks the attacker's stealth"
        );
    }

    // ============================================================
    // F4 — per-effect fidelity audit
    // ============================================================

    #[test]
    fn single_target_destroy_hits_one_random_minion() {
        // F4: Assassinate-style destroy with no explicit target destroys ONE
        // random matching minion — never all of them.
        use crate::cards::def::ASSASSINATE;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &ASSASSINATE);
        builder.set_mana(PlayerId::Player1, 10, 10);
        builder.add_minion_to_board(PlayerId::Player2, &crate::cards::def::BLOODFEN_RAPTOR);
        builder.add_minion_to_board(PlayerId::Player2, &crate::cards::def::BLOODFEN_RAPTOR);
        builder.add_minion_to_board(PlayerId::Player2, &crate::cards::def::BLOODFEN_RAPTOR);
        let mut state = builder.build();
        let assassinate = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("assassinate in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: assassinate,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        assert_eq!(
            state
                .world()
                .zones()
                .iter(Zone::Play, PlayerId::Player2)
                .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
                .count(),
            2,
            "a single-target destroy removes exactly one minion"
        );
    }

    #[test]
    fn execute_destroys_one_damaged_minion() {
        // F4: Execute (destroy a damaged enemy minion) hits exactly one damaged
        // minion; undamaged minions are untouched.
        use crate::cards::def::EXECUTE;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &EXECUTE);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let damaged = builder.add_custom_minion_to_board(PlayerId::Player2, 2, 4, 3);
        builder.add_minion_to_board(PlayerId::Player2, &crate::cards::def::BLOODFEN_RAPTOR);
        let mut state = builder.build();
        state
            .world_mut()
            .set_damage(damaged, crate::core::component::Damage(1));
        let execute = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .expect("execute in hand");
        let engine = crate::engine::game::GameEngine::new();
        engine
            .apply(
                &mut state,
                Action::PlayCard {
                    card: execute,
                    target: None,
                    position: None,
                },
            )
            .unwrap();
        assert_eq!(
            state.world().zone(damaged),
            Some(Zone::Graveyard),
            "the damaged minion is destroyed"
        );
        let remaining = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player2)
            .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
            .count();
        assert_eq!(remaining, 1, "only the damaged minion is destroyed");
    }

    #[test]
    fn aura_stacking_is_additive() {
        // F4: multiple auras stack additively (two Stormwind Champions: +2/+2)
        use crate::cards::def::STORMWIND_CHAMPION;
        use crate::sim::game::GameBuilder;
        let mut builder = GameBuilder::new();
        builder.add_minion_to_board(PlayerId::Player1, &STORMWIND_CHAMPION);
        builder.add_minion_to_board(PlayerId::Player1, &STORMWIND_CHAMPION);
        builder.add_minion_to_board(PlayerId::Player1, &crate::cards::def::BLOODFEN_RAPTOR);
        let state = builder.build();
        let target = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player1)
            .find(|&e| {
                state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| c.0 == "CLASSIC_001")
            })
            .expect("raptor on board");
        assert_eq!(
            state.world().effective_attack(target),
            Some(Attack(5)),
            "two champions grant +2/+2 (additive aura stacking)"
        );
        assert_eq!(
            state.world().effective_health(target),
            Some(Health(4)),
            "two champions grant +2/+2 (additive aura stacking)"
        );
    }
}
