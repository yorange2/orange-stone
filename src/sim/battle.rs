//! Battle simulation — bot vs bot games, card coverage tracking, and game statistics.
//!
//! Provides a full two-player game loop, random deck generation, card coverage
//! tracking, and game result statistics for large-scale automated testing.
//!
//! # Example
//!
//! ```rust,ignore
//! use orange_stone::sim::battle::{BattleRunner, CardTracker, BotType};
//!
//! let mut runner = BattleRunner::new(BotType::Greedy, 12345);
//! let result = runner.run_battle(30);
//! println!("winner: {:?}", result.winner);
//! ```

use crate::cards::def::CardDef;
use crate::cards::sets::ALL_CARDS;
use crate::core::action::Action;
use crate::core::component::CardType;
use crate::core::player::PlayerId;
use crate::core::state::{GameState, Step};
use crate::core::zone::Zone;
use crate::engine::game::GameEngine;
use crate::sim::bot::{GreedyBot, SmartBot};
use crate::sim::game::GameBuilder;
use crate::sim::rng::GameRng;
use std::collections::HashMap;

// ============================================================
// Card coverage tracking
// ============================================================

/// Tracks how often each card is used in games.
#[derive(Debug, Clone)]
pub struct CardTracker {
    /// How many times each card was included in a deck
    pub deck_count: HashMap<&'static str, usize>,
    /// How many times each card was played
    pub play_count: HashMap<&'static str, usize>,
    /// Total number of unique cards
    unique_count: usize,
}

impl CardTracker {
    /// Creates a new card tracker, initializing all cards from ALL_CARDS.
    pub fn new() -> Self {
        // Deduplicate
        let mut seen: HashMap<&'static str, &'static CardDef> = HashMap::new();
        for card in ALL_CARDS {
            seen.entry(card.id).or_insert(card);
        }
        let unique_count = seen.len();
        let mut deck_count = HashMap::with_capacity(unique_count);
        let mut play_count = HashMap::with_capacity(unique_count);
        for id in seen.keys() {
            deck_count.insert(*id, 0);
            play_count.insert(*id, 0);
        }
        Self {
            deck_count,
            play_count,
            unique_count,
        }
    }

    /// Returns the total number of unique cards.
    pub fn unique_cards(&self) -> usize {
        self.unique_count
    }

    /// Records that a card was included in a deck.
    pub fn record_in_deck(&mut self, card: &CardDef) {
        *self.deck_count.entry(card.id).or_insert(0) += 1;
    }

    /// Records that a batch of cards was included in a deck.
    pub fn record_deck(&mut self, cards: &[&CardDef]) {
        for card in cards {
            self.record_in_deck(card);
        }
    }

    /// Records that a card was played.
    pub fn record_played(&mut self, card_id: &'static str) {
        *self.play_count.entry(card_id).or_insert(0) += 1;
    }

    /// Checks card coverage: returns (number of used cards, total).
    pub fn coverage(&self) -> (usize, usize) {
        let used = self.deck_count.values().filter(|&&c| c > 0).count();
        (used, self.unique_count)
    }

    /// Returns the least used cards and their counts.
    pub fn least_used(&self) -> Vec<(&'static str, usize)> {
        let min = self.deck_count.values().min().copied().unwrap_or(0);
        self.deck_count
            .iter()
            .filter(|(_, c)| **c == min)
            .map(|(&id, &c)| (id, c))
            .collect()
    }

    /// Returns the most used cards and their counts.
    pub fn most_used(&self) -> Vec<(&'static str, usize)> {
        let max = self.deck_count.values().max().copied().unwrap_or(0);
        self.deck_count
            .iter()
            .filter(|(_, c)| **c == max)
            .map(|(&id, &c)| (id, c))
            .collect()
    }
}

impl Default for CardTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Bot delegation — a unified interface over GreedyBot and SmartBot
// ============================================================

/// Enum of bot types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotType {
    /// Greedy bot
    Greedy,
    /// Smart bot
    Smart,
    /// No bot — both players are controlled externally (roadmap M1-G4,
    /// used by the RL env for arena / self-play)
    None,
}

/// Unified bot delegation that avoids trait object dependencies.
#[derive(Debug, Clone, Copy)]
pub enum BotDelegate {
    /// Greedy bot variant
    Greedy(GreedyBot),
    /// Smart bot variant
    Smart(SmartBot),
    /// No bot — returns no actions (external control)
    None,
}

impl BotDelegate {
    /// Creates the bot delegate for the given BotType.
    pub fn new(bot_type: BotType) -> Self {
        match bot_type {
            BotType::Greedy => Self::Greedy(GreedyBot::new()),
            BotType::Smart => Self::Smart(SmartBot::new()),
            BotType::None => Self::None,
        }
    }

    /// Generates the action sequence for the current turn.
    pub fn decide_actions(&self, state: &GameState) -> Vec<Action> {
        match self {
            Self::Greedy(bot) => bot.decide_actions(state),
            Self::Smart(bot) => bot.decide_actions(state),
            Self::None => Vec::new(),
        }
    }
}

// ============================================================
// Battle results
// ============================================================

/// Summary of a single battle.
#[derive(Debug, Clone)]
pub struct BattleResult {
    /// Winner (None means the max turn limit was reached)
    pub winner: Option<PlayerId>,
    /// Turns used by the battle
    pub turns: u32,
    /// Player1's remaining health
    pub p1_hp: i32,
    /// Player2's remaining health
    pub p2_hp: i32,
    /// Total number of actions
    pub total_actions: usize,
    /// Engine errors that occurred in this battle (with context)
    pub errors: Vec<BattleError>,
    /// Replay check with a different seed (optional)
    pub end_step: Step,
}

/// An error record from a battle.
#[derive(Debug, Clone)]
pub struct BattleError {
    /// Which player's turn it happened on
    pub player: PlayerId,
    /// Turn number
    pub turn: u32,
    /// Error message
    pub error: String,
    /// The action that triggered the error
    pub action: String,
}

// ============================================================
// Battle runner
// ============================================================

/// Battle runner — manages deck generation, the game loop, and statistics.
#[derive(Debug)]
pub struct BattleRunner {
    /// Bot type
    bot_type: BotType,
    /// Random number generator
    rng: GameRng,
    /// Card tracker
    pub tracker: CardTracker,
    /// Battle statistics
    pub stats: BattleStats,
}

/// Accumulated battle statistics.
#[derive(Debug, Clone, Default)]
pub struct BattleStats {
    /// Number of completed battles
    pub games_played: usize,
    /// Player1 wins
    pub p1_wins: usize,
    /// Player2 wins
    pub p2_wins: usize,
    /// Number of times the turn limit was reached
    pub turn_limit_hits: usize,
    /// Total actions across all battles
    pub total_actions: usize,
    /// Total turns across all battles
    pub total_turns: u64,
    /// All engine errors recorded
    pub all_errors: Vec<BattleError>,
    /// Distribution of turns per battle
    pub turn_distribution: HashMap<u32, usize>,
}

impl BattleRunner {
    /// Creates a new battle runner.
    ///
    /// `bot_type` — the bot type used by both sides.
    /// `seed` — the master RNG seed (each battle is offset from this seed).
    pub fn new(bot_type: BotType, seed: u64) -> Self {
        Self {
            bot_type,
            rng: GameRng::new(seed),
            tracker: CardTracker::new(),
            stats: BattleStats::default(),
        }
    }

    /// Runs a single battle.
    ///
    /// Automatically generates random decks, runs the two-player loop, and updates the tracker and stats.
    /// `deck_size` — the deck size for each player (usually 30).
    /// `max_turns` — the turn limit (prevents infinite loops).
    /// `hand_size` — the initial hand size (default 3).
    pub fn run_battle(&mut self, deck_size: usize) -> BattleResult {
        // Build the initial game (decks + seed)
        let mut state = self.create_game_state(deck_size, 3, false);

        // Run the battle
        self.run_game_loop(&mut state, 60)
    }

    /// Generates random decks and builds an initial `GameState` (shared by single and batched simulation).
    ///
    /// Each call consumes one deck seed and one game seed, so consecutive calls
    /// on the same `BattleRunner` (same initial seed) produce different battles.
    /// `hand_size`/`second_player_coin` shape the opening (roadmap M1-G6).
    pub fn create_game_state(
        &mut self,
        deck_size: usize,
        hand_size: usize,
        second_player_coin: bool,
    ) -> GameState {
        let (p1_deck, p2_deck) = self.generate_random_decks(deck_size);
        self.create_game_state_with_decks(&p1_deck, &p2_deck, hand_size, second_player_coin)
    }

    /// Builds an initial `GameState` from explicit decks (roadmap M1-G2).
    ///
    /// Used by the RL environment for fixed-pool mirror matchups; the decks are
    /// recorded in the tracker and shuffled by the builder (deterministic per seed).
    /// The game RNG is reseeded from the runner's RNG, keeping per-battle reproducibility.
    ///
    /// `hand_size` — cards dealt to the first player (roadmap M1-G6); with
    /// `second_player_coin` the second player draws one extra card and gets
    /// The Coin (the official opening shape).
    pub fn create_game_state_with_decks(
        &mut self,
        deck1: &[&'static CardDef],
        deck2: &[&'static CardDef],
        hand_size: usize,
        second_player_coin: bool,
    ) -> GameState {
        self.tracker.record_deck(deck1);
        self.tracker.record_deck(deck2);

        // Build the initial game
        let mut state = self.build_game_state(deck1, deck2, hand_size, second_player_coin);

        // Record this battle's seed (for reproduction)
        let game_seed = self.rng.next_u32() as u64;
        state.make_mut().rng = GameRng::new(game_seed);
        state
    }

    /// Generates two random decks (no duplicates within each player's deck).
    fn generate_random_decks(
        &mut self,
        deck_size: usize,
    ) -> (Vec<&'static CardDef>, Vec<&'static CardDef>) {
        let unique_cards: Vec<&CardDef> = ALL_CARDS.iter().collect();
        let total = unique_cards.len();

        // Fisher-Yates shuffle
        let mut indices: Vec<usize> = (0..total).collect();
        for i in (1..total).rev() {
            let j = self.rng.next_usize(i + 1);
            indices.swap(i, j);
        }

        // Prefer cards that have been used less:
        // for each slot, pick the least-used card among a few shuffled candidates
        let pick_card =
            |rng: &mut GameRng, tracker: &CardTracker, used: &mut Vec<bool>| -> &'static CardDef {
                // Among unused cards, sample a few and pick the least used
                let candidates: Vec<usize> =
                    (0..total).filter(|&idx| !used[indices[idx]]).collect();
                if candidates.is_empty() {
                    // All unique cards are used up — allow reuse
                    let pick = rng.next_usize(total);
                    return unique_cards[pick];
                }
                // Take up to 5 candidates and pick the one with the lowest deck_count
                let n = (candidates.len()).min(5);
                let start = rng.next_usize(candidates.len());
                let mut best: Option<(usize, &'static str)> = None;
                for i in 0..n {
                    let ci = candidates[(start + i) % candidates.len()];
                    let card = unique_cards[indices[ci]];
                    let cnt = *tracker.deck_count.get(card.id).unwrap_or(&0);
                    if best.is_none_or(|(b_cnt, _)| cnt < b_cnt) {
                        best = Some((cnt, card.id));
                    }
                }
                let card_id = best.unwrap().1;
                // Find the matching card
                let idx = (0..total)
                    .find(|&i| unique_cards[indices[i]].id == card_id)
                    .unwrap();
                used[indices[idx]] = true;
                unique_cards[indices[idx]]
            };

        let mut used1 = vec![false; total];
        let mut used2 = vec![false; total];

        let mut deck1 = Vec::with_capacity(deck_size);
        let mut deck2 = Vec::with_capacity(deck_size);

        for _ in 0..deck_size {
            deck1.push(pick_card(&mut self.rng, &self.tracker, &mut used1));
            deck2.push(pick_card(&mut self.rng, &self.tracker, &mut used2));
        }

        (deck1, deck2)
    }

    /// Builds the initial GameState — deck, opening hand, and starting mana.
    ///
    /// The state RNG is seeded from the runner's RNG, so the deck shuffle and
    /// opening hands differ per seed. This matters for explicit decks (M1-G2):
    /// with a fixed deck the deck content alone no longer varies the opening.
    ///
    /// Opening shape (roadmap M1-G6): the first player draws `hand_size`;
    /// with `second_player_coin` the second player draws `hand_size + 1` and
    /// receives The Coin as an extra hand card (official second-player shape).
    fn build_game_state(
        &mut self,
        deck1: &[&'static CardDef],
        deck2: &[&'static CardDef],
        hand_size: usize,
        second_player_coin: bool,
    ) -> GameState {
        let opening_seed = self.rng.next_u32() as u64;
        let mut builder = GameBuilder::new();
        builder.with_rng_seed(opening_seed);
        // Deck
        for card in deck1 {
            builder.add_minion_to_deck(PlayerId::Player1, card);
        }
        for card in deck2 {
            builder.add_minion_to_deck(PlayerId::Player2, card);
        }
        // The Coin joins the second player's hand before the deal (it is not
        // part of the deck, so the shuffle never touches it)
        if second_player_coin {
            builder.add_minion_to_hand(PlayerId::Player2, &crate::cards::def::THE_COIN);
        }
        // Mana: GameState::new() already gives Player1 the turn-1 crystal (1/1,
        // HS official); Player2 starts at 0/0 and gets its crystal on its first
        // ManaRefill. Do NOT reset P1 to 0/0 — that clobbers the turn-1 crystal
        // and leaves the first player a full crystal behind for the whole game.
        builder.set_mana(PlayerId::Player2, 0, 0);
        let mut state = builder.build();

        // Opening hand — draw from the *current* deck length (the deck shrinks
        // with every draw, so an index into the original length can miss).
        let p2_bonus = if second_player_coin { 1 } else { 0 };
        for (pid, draw_count) in [
            (PlayerId::Player1, hand_size),
            (PlayerId::Player2, hand_size + p2_bonus),
        ] {
            let deck_count = state.world().zones().len(Zone::Deck, pid);
            for _ in 0..draw_count.min(deck_count) {
                let current = state.world().zones().len(Zone::Deck, pid);
                if current == 0 {
                    break;
                }
                let idx = state.rng_mut().next_usize(current);
                let Some(card) = state.world().zones().iter(Zone::Deck, pid).nth(idx) else {
                    continue;
                };
                let _ = state.world_mut().move_to_zone(card, Zone::Hand);
            }
        }

        state
    }

    /// Runs the full battle loop.
    fn run_game_loop(&mut self, state: &mut GameState, max_turns: u32) -> BattleResult {
        let engine = GameEngine::new();
        let bot = BotDelegate::new(self.bot_type);

        let mut total_actions = 0;
        let mut errors = Vec::new();
        let mut turn_count = 0;

        loop {
            turn_count += 1;
            if turn_count > max_turns {
                break;
            }

            if matches!(state.step(), Step::GameOver { .. }) {
                break;
            }

            let active = state.active_player();
            let actions = bot.decide_actions(state);
            if actions.is_empty() {
                // No actions (e.g. BotType::None): nothing to do this turn
                break;
            }

            for action in &actions {
                total_actions += 1;

                // Track played cards
                if let Action::PlayCard {
                    card,
                    target: None,
                    position: None,
                } = action
                {
                    if let Some(card_id) = get_card_id(state, *card) {
                        self.tracker.record_played(card_id);
                    }
                }

                match engine.apply(state, *action) {
                    Ok(_events) => {
                        // Check basic invariants
                        if let Some(err) = check_invariants(state, active, state.turn()) {
                            errors.push(err);
                        }
                    }
                    Err(err) => {
                        errors.push(BattleError {
                            player: active,
                            turn: state.turn(),
                            error: format!("{err:?}"),
                            action: format!("{action:?}"),
                        });
                    }
                }

                // Break out once the game is over
                if matches!(state.step(), Step::GameOver { .. }) {
                    break;
                }
            }

            if matches!(state.step(), Step::GameOver { .. }) {
                break;
            }
        }

        let winner = match state.step() {
            Step::GameOver { winner } => Some(winner),
            _ => None,
        };

        let p1_hp = state
            .world()
            .effective_health(state.player(PlayerId::Player1).hero)
            .map(|h| h.0)
            .unwrap_or(0);
        let p2_hp = state
            .world()
            .effective_health(state.player(PlayerId::Player2).hero)
            .map(|h| h.0)
            .unwrap_or(0);

        // Update statistics
        self.stats.games_played += 1;
        self.stats.total_actions += total_actions;
        self.stats.total_turns += u64::from(state.turn());
        *self
            .stats
            .turn_distribution
            .entry(state.turn())
            .or_insert(0) += 1;

        match winner {
            Some(PlayerId::Player1) => self.stats.p1_wins += 1,
            Some(PlayerId::Player2) => self.stats.p2_wins += 1,
            None => self.stats.turn_limit_hits += 1,
        }
        self.stats.all_errors.extend(errors.clone());

        BattleResult {
            winner,
            turns: state.turn(),
            p1_hp,
            p2_hp,
            total_actions,
            errors,
            end_step: state.step(),
        }
    }
}

// ============================================================
// Helper functions
// ============================================================

/// Gets the card ID for an entity (for entities recorded during the hand/deck phase).
///
/// Current implementation: scans ALL_CARDS and matches approximately by cost/attack/health.
/// Note: this is a heuristic match and may be inaccurate when several cards share the same stats.
fn get_card_id(state: &GameState, entity: crate::core::entity::Entity) -> Option<&'static str> {
    let world = state.world();
    let cost = world.cost(entity)?;
    let atk = world.attack(entity).unwrap_or_default();
    let hp = world.effective_health(entity).unwrap_or_default();
    let ct = world.card_type(entity)?;

    // Exact match
    ALL_CARDS
        .iter()
        .find(|c| {
            c.card_type == ct
                && c.cost == cost.0
                && (c.attack == atk.0 || ct == CardType::Spell)
                && (c.health == hp.0 || ct == CardType::Spell)
        })
        .map(|c| c.id)
}

/// Checks basic game invariants, returning an error if something is wrong.
fn check_invariants(state: &GameState, _player: PlayerId, _turn: u32) -> Option<BattleError> {
    let world = state.world();

    // Check that the hero did not die from non-damage effects
    for &pid in &[PlayerId::Player1, PlayerId::Player2] {
        let hero = state.player(pid).hero;
        if let Some(hp) = world.effective_health(hero) {
            if hp.0 < -100 {
                return Some(BattleError {
                    player: pid,
                    turn: state.turn(),
                    error: format!("英雄 HP 异常：{}", hp.0),
                    action: "(invariant check)".to_string(),
                });
            }
        }
    }

    // Check the minion count on the board
    for &pid in &[PlayerId::Player1, PlayerId::Player2] {
        let count = world
            .zones()
            .iter(Zone::Play, pid)
            .filter(|&e| world.card_type(e) == Some(CardType::Minion))
            .count();
        if count > 7 {
            return Some(BattleError {
                player: pid,
                turn: state.turn(),
                error: format!("场上随从数量超限：{count} > 7"),
                action: "(invariant check)".to_string(),
            });
        }
    }

    // Check mana values
    for &pid in &[PlayerId::Player1, PlayerId::Player2] {
        let p = state.player(pid);
        if p.current_mana < 0 || p.current_mana > 10 {
            return Some(BattleError {
                player: pid,
                turn: state.turn(),
                error: format!("法力值异常：current={}", p.current_mana),
                action: "(invariant check)".to_string(),
            });
        }
        if p.mana_crystals < 0 || p.mana_crystals > 10 {
            return Some(BattleError {
                player: pid,
                turn: state.turn(),
                error: format!("法力水晶异常：crystals={}", p.mana_crystals),
                action: "(invariant check)".to_string(),
            });
        }
    }

    None
}
