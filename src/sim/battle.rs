//! 战斗模拟 — 机器人对战、卡牌覆盖追踪与对局统计。
//!
//! 提供完整的双人对战循环、随机牌组生成、卡牌覆盖率跟踪
//! 以及对局结果统计，用于大规模自动化测试。
//!
//! # 示例
//!
//! ```rust,ignore
//! use orange_stone::sim::battle::{BattleRunner, CardTracker, BotType};
//!
//! let mut runner = BattleRunner::new(BotType::Greedy, 12345);
//! let result = runner.run_battle(30);
//! println!("胜者: {:?}", result.winner);
//! ```

use crate::cards::def::CardDef;
use crate::cards::sets::ALL_CARDS;
use crate::core::action::Action;
use crate::core::component::CardType;
use crate::core::player::PlayerId;
use crate::core::state::{GameState, Phase};
use crate::core::zone::Zone;
use crate::engine::game::GameEngine;
use crate::sim::bot::{GreedyBot, SmartBot};
use crate::sim::game::GameBuilder;
use crate::sim::rng::GameRng;
use std::collections::HashMap;

// ============================================================
// 卡牌覆盖追踪
// ============================================================

/// 追踪每张卡牌在对局中的使用情况。
#[derive(Debug, Clone)]
pub struct CardTracker {
    /// 每张卡牌被纳入牌组的次数
    pub deck_count: HashMap<&'static str, usize>,
    /// 每张卡牌被打出的次数
    pub play_count: HashMap<&'static str, usize>,
    /// 唯一卡牌总数
    unique_count: usize,
}

impl CardTracker {
    /// 创建新的卡牌追踪器，初始化所有 ALL_CARDS 中的卡牌。
    pub fn new() -> Self {
        // 去重
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

    /// 返回唯一卡牌总数。
    pub fn unique_cards(&self) -> usize {
        self.unique_count
    }

    /// 标记一张卡牌被纳入牌组。
    pub fn record_in_deck(&mut self, card: &CardDef) {
        *self.deck_count.entry(card.id).or_insert(0) += 1;
    }

    /// 标记一批卡牌被纳入牌组。
    pub fn record_deck(&mut self, cards: &[&CardDef]) {
        for card in cards {
            self.record_in_deck(card);
        }
    }

    /// 标记一张卡牌被打出。
    pub fn record_played(&mut self, card_id: &'static str) {
        *self.play_count.entry(card_id).or_insert(0) += 1;
    }

    /// 检查卡牌覆盖率：返回 (已使用的卡牌数, 总数)。
    pub fn coverage(&self) -> (usize, usize) {
        let used = self.deck_count.values().filter(|&&c| c > 0).count();
        (used, self.unique_count)
    }

    /// 返回最少被使用的卡牌及其次数。
    pub fn least_used(&self) -> Vec<(&'static str, usize)> {
        let min = self.deck_count.values().min().copied().unwrap_or(0);
        self.deck_count
            .iter()
            .filter(|(_, c)| **c == min)
            .map(|(&id, &c)| (id, c))
            .collect()
    }

    /// 返回最多被使用的卡牌及其次数。
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
// 机器人委托 — 统一 GreedyBot 和 SmartBot 的接口
// ============================================================

/// 机器人类型枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotType {
    /// 贪心机器人
    Greedy,
    /// 智能机器人
    Smart,
}

/// 统一的机器人委托，消除 trait object 依赖。
#[derive(Debug, Clone, Copy)]
pub enum BotDelegate {
    /// 贪心机器人变体
    Greedy(GreedyBot),
    /// 智能机器人变体
    Smart(SmartBot),
}

impl BotDelegate {
    /// 从 BotType 创建对应的机器人委托。
    pub fn new(bot_type: BotType) -> Self {
        match bot_type {
            BotType::Greedy => Self::Greedy(GreedyBot::new()),
            BotType::Smart => Self::Smart(SmartBot::new()),
        }
    }

    /// 生成当前回合的动作序列。
    pub fn decide_actions(&self, state: &GameState) -> Vec<Action> {
        match self {
            Self::Greedy(bot) => bot.decide_actions(state),
            Self::Smart(bot) => bot.decide_actions(state),
        }
    }
}

// ============================================================
// 对局结果
// ============================================================

/// 单场对局的结果摘要。
#[derive(Debug, Clone)]
pub struct BattleResult {
    /// 胜者（None 表示达到最大回合数限制）
    pub winner: Option<PlayerId>,
    /// 对局使用的回合数
    pub turns: u32,
    /// Player1 剩余生命值
    pub p1_hp: i32,
    /// Player2 剩余生命值
    pub p2_hp: i32,
    /// 动作总数
    pub total_actions: usize,
    /// 本局中发生的引擎错误（含上下文信息）
    pub errors: Vec<BattleError>,
    /// 使用不同 seed 的回放检查（可选）
    pub end_phase: Phase,
}

/// 对局中的错误记录。
#[derive(Debug, Clone)]
pub struct BattleError {
    /// 发生在哪个玩家的回合
    pub player: PlayerId,
    /// 回合数
    pub turn: u32,
    /// 错误信息
    pub error: String,
    /// 触发错误的操作
    pub action: String,
}

// ============================================================
// 对局运行器
// ============================================================

/// 对局运行器 — 管理牌组生成、对局循环和统计。
#[derive(Debug)]
pub struct BattleRunner {
    /// 机器人类型
    bot_type: BotType,
    /// 随机数生成器
    rng: GameRng,
    /// 卡牌追踪器
    pub tracker: CardTracker,
    /// 对局统计
    pub stats: BattleStats,
}

/// 累加的对局统计。
#[derive(Debug, Clone, Default)]
pub struct BattleStats {
    /// 已完成对局数
    pub games_played: usize,
    /// Player1 获胜次数
    pub p1_wins: usize,
    /// Player2 获胜次数
    pub p2_wins: usize,
    /// 达到回合上限的次数
    pub turn_limit_hits: usize,
    /// 累计动作总数
    pub total_actions: usize,
    /// 累计回合总数
    pub total_turns: u64,
    /// 记录的所有引擎错误
    pub all_errors: Vec<BattleError>,
    /// 每次对局的回合数分布
    pub turn_distribution: HashMap<u32, usize>,
}

impl BattleRunner {
    /// 创建新的对局运行器。
    ///
    /// `bot_type` — 双方使用的机器人类型。
    /// `seed` — 主 RNG seed（每局会在此 seed 上偏移）。
    pub fn new(bot_type: BotType, seed: u64) -> Self {
        Self {
            bot_type,
            rng: GameRng::new(seed),
            tracker: CardTracker::new(),
            stats: BattleStats::default(),
        }
    }

    /// 运行一场对局。
    ///
    /// 自动生成随机牌组，执行双人对战循环，更新追踪器和统计。
    /// `deck_size` — 每个玩家的牌组大小（通常为 30）。
    /// `max_turns` — 回合数上限（防止无限循环）。
    /// `hand_size` — 初始手牌数（默认 3）。
    pub fn run_battle(&mut self, deck_size: usize) -> BattleResult {
        // 生成牌组
        let (p1_deck, p2_deck) = self.generate_random_decks(deck_size);
        self.tracker.record_deck(&p1_deck);
        self.tracker.record_deck(&p2_deck);

        // 构建初始对局
        let mut state = self.build_game_state(&p1_deck, &p2_deck, 3);

        // 记录本局种子（用于复现）
        let game_seed = self.rng.next_u32() as u64;
        state.make_mut().rng = GameRng::new(game_seed);

        // 运行对局
        self.run_game_loop(&mut state, 60)
    }

    /// 生成两个随机牌组（每个 player 的牌组内不重复）。
    fn generate_random_decks(&mut self, deck_size: usize) -> (Vec<&'static CardDef>, Vec<&'static CardDef>) {
        let unique_cards: Vec<&CardDef> = ALL_CARDS.iter().collect();
        let total = unique_cards.len();

        // Fisher-Yates 洗牌
        let mut indices: Vec<usize> = (0..total).collect();
        for i in (1..total).rev() {
            let j = self.rng.next_usize(i + 1);
            indices.swap(i, j);
        }

        // 优先选择使用次数较少的卡牌：
        // 对每个位置，在洗牌后的几个候选中选使用次数最少的
        let pick_card = |rng: &mut GameRng, tracker: &CardTracker, used: &mut Vec<bool>| -> &'static CardDef {
            // 在未使用的卡牌中，随机选几个，挑使用次数最少的
            let candidates: Vec<usize> = (0..total)
                .filter(|&idx| !used[indices[idx]])
                .collect();
            if candidates.is_empty() {
                // 去重卡牌全部用了，允许复用
                let pick = rng.next_usize(total);
                return unique_cards[pick];
            }
            // 取 5 个候选，选 deck_count 最少的
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
            // 找到对应卡牌
            let idx = (0..total).find(|&i| unique_cards[indices[i]].id == card_id).unwrap();
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

    /// 构建初始 GameState — 牌库、初始手牌和初始法力。
    fn build_game_state(
        &mut self,
        deck1: &[&'static CardDef],
        deck2: &[&'static CardDef],
        hand_size: usize,
    ) -> GameState {
        let mut builder = GameBuilder::new();
        // 牌库
        for card in deck1 {
            builder.add_minion_to_deck(PlayerId::Player1, card);
        }
        for card in deck2 {
            builder.add_minion_to_deck(PlayerId::Player2, card);
        }
        builder.set_mana(PlayerId::Player1, 0, 0);
        builder.set_mana(PlayerId::Player2, 0, 0);
        let mut state = builder.build();

        // 初始手牌
        for &pid in &[PlayerId::Player1, PlayerId::Player2] {
            let deck_count = state.world().zones().len(Zone::Deck, pid);
            let draw_count = hand_size.min(deck_count);
            for _ in 0..draw_count {
                let idx = state.rng_mut().next_usize(deck_count);
                let Some(card) = state.world().zones().iter(Zone::Deck, pid).nth(idx) else {
                    continue;
                };
                let _ = state.world_mut().move_to_zone(card, Zone::Hand);
            }
        }

        state
    }

    /// 运行完整的对局循环。
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

            if matches!(state.phase(), Phase::GameOver { .. }) {
                break;
            }

            let active = state.active_player();
            let actions = bot.decide_actions(state);

            for action in &actions {
                total_actions += 1;

                // 追踪打出卡牌
                if let Action::PlayCard { card } = action {
                    if let Some(card_id) = get_card_id(state, *card) {
                        self.tracker.record_played(card_id);
                    }
                }

                match engine.apply(state, *action) {
                    Ok(_events) => {
                        // 检查基本不变量
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

                // 游戏结束后跳出
                if matches!(state.phase(), Phase::GameOver { .. }) {
                    break;
                }
            }

            if matches!(state.phase(), Phase::GameOver { .. }) {
                break;
            }
        }

        let winner = match state.phase() {
            Phase::GameOver { winner } => Some(winner),
            _ => None,
        };

        let p1_hp = state
            .world()
            .health(state.player(PlayerId::Player1).hero)
            .map(|h| h.0)
            .unwrap_or(0);
        let p2_hp = state
            .world()
            .health(state.player(PlayerId::Player2).hero)
            .map(|h| h.0)
            .unwrap_or(0);

        // 更新统计
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
            end_phase: state.phase(),
        }
    }
}

// ============================================================
// 辅助函数
// ============================================================

/// 获取实体的卡牌 ID（从 hand/deck 阶段记录的 entity）。
///
/// 当前实现：遍历 ALL_CARDS 根据 cost/attack/health 近似匹配。
/// 注意：这是一个启发式匹配，有多个卡牌同身材时可能不准确。
fn get_card_id(state: &GameState, entity: crate::core::entity::Entity) -> Option<&'static str> {
    let world = state.world();
    let cost = world.cost(entity)?;
    let atk = world.attack(entity).unwrap_or_default();
    let hp = world.health(entity).unwrap_or_default();
    let ct = world.card_type(entity)?;

    // 精确匹配
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

/// 检查基本游戏不变量，发现异常则返回错误。
fn check_invariants(state: &GameState, _player: PlayerId, _turn: u32) -> Option<BattleError> {
    let world = state.world();

    // 检查英雄不死于非伤害
    for &pid in &[PlayerId::Player1, PlayerId::Player2] {
        let hero = state.player(pid).hero;
        if let Some(hp) = world.health(hero) {
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

    // 检查场上随从数量
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

    // 检查法力值
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
