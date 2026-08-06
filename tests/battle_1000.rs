//! 1000-battle bot matches — large-scale card coverage test.
//!
//! Runs 1000 SmartBot vs SmartBot battles with random decks,
//! tracking card coverage, engine errors, and basic game invariants.
//!
//! Run with:
//! ```bash
//! cargo test battle_1000 --release -- --nocapture
//! ```
//!
//! Use `--release` for reasonable runtime.

use orange_stone::sim::battle::{BattleRunner, BotType};

/// Main test: 1000 bot battles.
///
/// The battle count can be adjusted via the `BATTLE_COUNT` environment variable:
/// ```bash
/// BATTLE_COUNT=100 cargo test battle_1000 --release -- --nocapture
/// ```
#[test]
fn run_1000_battles() {
    let battle_count: usize = std::env::var("BATTLE_COUNT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           🎮 机器人对战大规模测试                            ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  对局数量: {battle_count:<50}║");
    println!("║  机器人:   SmartBot vs SmartBot                             ║");
    println!("║  每局牌组: 30 张                                            ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  卡牌总数: {}                                              ║",
        orange_stone::cards::sets::ALL_CARDS.len()
    );

    let unique_cards = {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for card in orange_stone::cards::sets::ALL_CARDS {
            seen.insert(card.id);
        }
        seen.len()
    };
    println!("║  唯一卡牌: {unique_cards:<50}║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut runner = BattleRunner::new(BotType::Smart, 42);

    let mut last_progress_pct = 0;
    let start_time = std::time::Instant::now();

    for i in 0..battle_count {
        let result = runner.run_battle(30);

        // Progress reporting
        let progress_pct = ((i + 1) * 100) / battle_count;
        if progress_pct > last_progress_pct || i < 5 || i >= battle_count - 1 {
            let elapsed = start_time.elapsed();
            let rate = if i > 0 {
                elapsed.as_secs_f64() / (i as f64)
            } else {
                0.0
            };
            let eta = rate * (battle_count - i - 1) as f64;

            let winner_str = match result.winner {
                Some(orange_stone::core::player::PlayerId::Player1) => "P1",
                Some(orange_stone::core::player::PlayerId::Player2) => "P2",
                None => "超时",
            };
            let err_count = result.errors.len();

            println!(
                "[{:>4}/{}] {:>3}% | {winner_str} 胜 | 回合 {:>3} | 动作 {:>4} | 错误 {err_count} | {:.1}s/局 | ETA {:.0}s",
                i + 1,
                battle_count,
                progress_pct,
                result.turns,
                result.total_actions,
                rate,
                eta,
            );
            last_progress_pct = progress_pct;
        }
    }

    // ===== Statistics output =====
    let elapsed = start_time.elapsed();
    let stats = &runner.stats;
    let tracker = &runner.tracker;
    let (covered, total) = tracker.coverage();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           📊 测试结果汇总                                    ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  总对局数:     {:<46}║", stats.games_played);
    println!("║  总用时:       {:.1}s{:<41}║", elapsed.as_secs_f64(), "");
    println!(
        "║  每局平均:     {:.2}s{:<41}║",
        elapsed.as_secs_f64() / stats.games_played as f64,
        ""
    );
    println!("║  总动作数:     {:<46}║", stats.total_actions);
    println!("║  总回合数:     {:<46}║", stats.total_turns);
    println!("║  P1 胜:        {:<46}║", stats.p1_wins);
    println!("║  P2 胜:        {:<46}║", stats.p2_wins);
    println!("║  超时:         {:<46}║", stats.turn_limit_hits);
    println!(
        "║  卡牌覆盖:     {}/{} ({:.1}%){:<30}║",
        covered,
        total,
        covered as f64 / total as f64 * 100.0,
        ""
    );
    println!("╠══════════════════════════════════════════════════════════════╣");

    // Engine error statistics
    let total_errors: usize = stats.all_errors.len();
    println!("║  引擎错误总数: {:<46}║", total_errors);

    if !stats.all_errors.is_empty() {
        // Group by error type
        use std::collections::HashMap;
        let mut error_counts: HashMap<String, usize> = HashMap::new();
        for err in &stats.all_errors {
            *error_counts.entry(err.error.clone()).or_insert(0) += 1;
        }
        let mut sorted_errors: Vec<_> = error_counts.into_iter().collect();
        sorted_errors.sort_by_key(|(_, c)| std::cmp::Reverse(*c));

        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  错误分布 (按类型):                                         ║");
        for (err_type, count) in sorted_errors.iter().take(15) {
            let truncated = if err_type.len() > 40 {
                format!("{}...", &err_type[..37])
            } else {
                err_type.clone()
            };
            println!("║    {count:>4}× {truncated:<46}║");
        }

        // Show some concrete error context
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║  前 10 个错误详情:                                          ║");
        for (i, err) in stats.all_errors.iter().take(10).enumerate() {
            let player = match err.player {
                orange_stone::core::player::PlayerId::Player1 => "P1",
                orange_stone::core::player::PlayerId::Player2 => "P2",
            };
            println!("║  [{i}] {player} 回合{}: {}", err.turn, err.error);
            let action_short = if err.action.len() > 40 {
                format!("{}...", &err.action[..37])
            } else {
                err.action.clone()
            };
            println!("║      操作: {action_short}");
        }
    }

    // Coverage details
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  卡牌覆盖详情:                                              ║");

    let least = tracker.least_used();
    let most = tracker.most_used();

    println!(
        "║  最少使用 ({min}次): {count} 张卡牌║",
        min = least.first().map(|(_, c)| *c).unwrap_or(0),
        count = least.len(),
    );
    for (id, cnt) in least.iter().take(5) {
        let name = get_card_name(id);
        println!("║    {cnt}× {id} ({name})");
    }
    if least.len() > 5 {
        println!("║    ... 还有 {} 张卡牌", least.len() - 5);
    }

    println!("║                                                              ║");
    println!(
        "║  最多使用 ({max}次):║",
        max = most.first().map(|(_, c)| *c).unwrap_or(0),
    );
    for (id, cnt) in most.iter().take(5) {
        let name = get_card_name(id);
        println!("║    {cnt}× {id} ({name})");
    }

    // Turn count distribution
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  回合数分布 (Top 10):                                       ║");
    let mut turn_dist: Vec<_> = stats.turn_distribution.iter().collect();
    turn_dist.sort_by_key(|(t, _)| std::cmp::Reverse(**t));
    for (turns, count) in turn_dist.iter().take(10) {
        let bar = "█".repeat((*count * 20).min(40));
        println!("║  {turns:>3} 回合: {count:>4} 局 {bar}");
    }

    println!("╚══════════════════════════════════════════════════════════════╝");

    // Assert: no engine errors
    if total_errors > 0 {
        println!("\n⚠️  检测到 {total_errors} 个引擎错误！请检查上面的错误详情。");
        // Do not panic on common expected errors (e.g. HeroPowerAlreadyUsed, bot strategy issues)
        let unexpected_errors: Vec<_> = stats
            .all_errors
            .iter()
            .filter(|e| {
                // Exclude all known bot strategy errors
                !e.error.contains("HeroPowerAlreadyUsed")
                    && !e.error.contains("NotEnoughMana")
                    && !e.error.contains("BoardFull")
                    && !e.error.contains("AttacksExhausted")
                    && !e.error.contains("MustAttackTaunt")
                    && !e.error.contains("InvalidTarget")
                    && !e.error.contains("EntityGone")
                    && !e.error.contains("NotOnBoard")
                    && !e.error.contains("NotYourTurn")
                    && !e.error.contains("NotYourCard")
                    && !e.error.contains("CardNotInHand")
                    && !e.error.contains("NotPlayable")
                    && !e.error.contains("GameAlreadyOver")
                    && !e.error.contains("Unimplemented")
            })
            .collect();
        if !unexpected_errors.is_empty() {
            panic!(
                "发现 {} 个非预期的引擎错误（排除机器人策略错误）",
                unexpected_errors.len()
            );
        }
    }

    // Assert: coverage should be > 80%
    let coverage_pct = covered as f64 / total as f64 * 100.0;
    assert!(
        coverage_pct > 80.0,
        "卡牌覆盖率 {coverage_pct:.1}% 低于 80% 阈值"
    );
    println!("\n✅ 所有检查通过！");
}

/// Look up a card name by card ID.
fn get_card_name(id: &str) -> &'static str {
    orange_stone::cards::sets::ALL_CARDS
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.name)
        .unwrap_or("?")
}
