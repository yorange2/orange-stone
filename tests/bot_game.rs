//! 机器人对战集成测试 — 两个 GreedyBot 互相博弈。
//!
//! 运行时输出详细的每步日志（action + event），
//! 通过 `cargo test bot_game -- --nocapture` 查看完整输出。

use orange_stone::cards::def::{
    ACIDIC_SWAMP_OOZE, ARCHMAGE, BLOODFEN_RAPTOR, EAGLEHORN_BOW, KOBOLD_GEOMANCER,
    MURLOC_RAIDER, MURLOC_TIDEHUNTER, OGRE_MAGI, VOIDWALKER, VOODOO_DOCTOR,
};
use orange_stone::core::action::Action;
use orange_stone::core::component::{CardType, Health};
use orange_stone::core::player::PlayerId;
use orange_stone::core::state::Phase;
use orange_stone::core::zone::Zone;
use orange_stone::engine::game::GameEngine;
use orange_stone::sim::bot::{GreedyBot, SmartBot};
use orange_stone::sim::game::GameBuilder;

/// 辅助：获取玩家名称
fn player_name(p: PlayerId) -> &'static str {
    match p {
        PlayerId::Player1 => "玩家1",
        PlayerId::Player2 => "玩家2",
    }
}

/// 辅助：格式化实体信息
fn entity_info(
    state: &orange_stone::core::state::GameState,
    e: orange_stone::core::entity::Entity,
) -> String {
    let w = state.world();
    let ct = w.card_type(e);
    let name = match ct {
        Some(CardType::Hero) => "英雄".to_string(),
        Some(CardType::Minion) => format!(
            "随从({}/{})",
            w.effective_attack(e).map(|a| a.0).unwrap_or(0),
            w.effective_health(e).map(|h| h.0).unwrap_or(0)
        ),
        Some(CardType::Weapon) => format!(
            "武器({}攻/{}耐)",
            w.attack(e).map(|a| a.0).unwrap_or(0),
            w.durability(e).map(|d| d.0).unwrap_or(0)
        ),
        _ => "?".to_string(),
    };
    format!("实体#{} {}", e.index, name)
}

/// 辅助：打印当前战况
fn print_board(state: &orange_stone::core::state::GameState) {
    let active = state.active_player();
    println!("══════════════════════════════════════════");
    println!(
        "回合 {} | {} 的回合 | 阶段: {:?}",
        state.turn(),
        player_name(active),
        state.phase()
    );

    for &pid in &[PlayerId::Player1, PlayerId::Player2] {
        let p = state.player(pid);
        let hero = p.hero;
        let hp = state.world().health(hero).unwrap_or(Health(0));
        let armor = p.armor;
        let weapon_str = p
            .weapon
            .map(|w| {
                let atk = state.world().attack(w).unwrap_or_default();
                let dur = state.world().durability(w).unwrap_or_default();
                format!(" [武器 {}/{}]", atk.0, dur.0)
            })
            .unwrap_or_default();

        println!(
            "  {}: 💚{} 🛡️{} 💎{}/{} {}",
            player_name(pid),
            hp.0,
            armor,
            p.current_mana,
            p.mana_crystals,
            weapon_str
        );

        // 手牌
        let hand: Vec<_> = state.world().zones().iter(Zone::Hand, pid).collect();
        println!("    手牌 ({} 张):", hand.len());
        for e in &hand {
            let cost = state.world().cost(*e).unwrap_or_default();
            let atk = state.world().attack(*e).unwrap_or_default();
            let hp = state.world().health(*e).unwrap_or_default();
            let taunt = if state.world().taunt(*e).is_some() {
                " [嘲讽]"
            } else {
                ""
            };
            println!("      费用{}: {}/{} {}", cost.0, atk.0, hp.0, taunt);
        }

        // 战场
        let board: Vec<_> = state
            .world()
            .zones()
            .iter(Zone::Play, pid)
            .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
            .collect();
        println!("    战场 ({} 个随从):", board.len());
        for e in &board {
            let atk = state.world().effective_attack(*e).unwrap_or_default();
            let hp = state.world().effective_health(*e).unwrap_or_default();
            let base_hp = state.world().health(*e).unwrap_or_default();
            let taunt = if state.world().taunt(*e).is_some() {
                " [嘲讽]"
            } else {
                ""
            };
            let aura = if state.world().aura(*e).is_some() {
                " [光环]"
            } else {
                ""
            };
            let used = if state
                .world()
                .attacks_used(*e)
                .is_some_and(|a| a.is_exhausted_with(state.world().max_attacks(*e)))
            {
                " (已攻击)"
            } else {
                ""
            };
            println!(
                "      {}/{} (基础{}) {}{}{}",
                atk.0, hp.0, base_hp.0, taunt, aura, used
            );
        }
    }
    println!("══════════════════════════════════════════");
}

#[test]
fn two_bots_battle() {
    let engine = GameEngine::new();
    let bot = GreedyBot::new();

    // ===== 构建初始对局 =====
    let mut builder = GameBuilder::new();

    // --- Player1 牌库 (10 张) ---
    let p1_cards = [
        &MURLOC_RAIDER, // 1费 2/1
        &MURLOC_RAIDER, // 1费 2/1
        &VOODOO_DOCTOR, // 1费 2/1 战吼:回2
        &VOIDWALKER, // 1费 1/3 嘲讽
        &BLOODFEN_RAPTOR, // 2费 3/2
        &BLOODFEN_RAPTOR, // 2费 3/2
        &KOBOLD_GEOMANCER, // 2费 2/2 法伤+1
        &MURLOC_TIDEHUNTER, // 2费 2/1 战吼:召唤1/1
        &OGRE_MAGI, // 4费 4/4
        &ARCHMAGE, // 6费 4/7
    ];
    for card in &p1_cards {
        builder.add_minion_to_deck(PlayerId::Player1, card);
    }

    // --- Player2 牌库 (10 张) ---
    let p2_cards = [
        &MURLOC_RAIDER, // 1费 2/1
        &MURLOC_RAIDER, // 1费 2/1
        &VOODOO_DOCTOR,         // 1费 1/1 战吼:1伤
        &VOIDWALKER,    // 1费 1/2 嘲讽
        &BLOODFEN_RAPTOR,      // 2费 2/3
        &ACIDIC_SWAMP_OOZE, // 2费 3/2 战吼:摧毁武器
        &ACIDIC_SWAMP_OOZE, // 2费 3/2 战吼:摧毁武器
        &OGRE_MAGI,       // 4费 4/5
        &EAGLEHORN_BOW, // 3费 3/2 武器
        &ARCHMAGE,     // 6费 6/7
    ];
    for card in &p2_cards {
        builder.add_minion_to_deck(PlayerId::Player2, card);
    }

    // 初始抽牌（每人 3 张起手）
    builder.set_mana(PlayerId::Player1, 0, 0);
    builder.set_mana(PlayerId::Player2, 0, 0);
    let mut state = builder.build();

    // 手动给每人抽 3 张起手牌
    for &pid in &[PlayerId::Player1, PlayerId::Player2] {
        for _ in 0..3 {
            let deck_len = state.world().zones().len(Zone::Deck, pid);
            if deck_len > 0 {
                let idx = state.rng_mut().next_usize(deck_len);
                let card = state
                    .world()
                    .zones()
                    .iter(Zone::Deck, pid)
                    .nth(idx)
                    .expect("deck card");
                state
                    .world_mut()
                    .move_to_zone(card, Zone::Hand)
                    .expect("move to hand");
            }
        }
    }

    // ===== 对局循环 =====
    println!("\n🎮 === 贪心机器人对战开始! ===");
    let max_turns = 30; // 防止无限循环
    let mut turn_count = 0;

    loop {
        turn_count += 1;
        if turn_count > max_turns {
            println!("\n⏰ 达到最大回合数限制 ({})，强制结束", max_turns);
            break;
        }

        if matches!(state.phase(), Phase::GameOver { .. }) {
            break;
        }

        let active = state.active_player();
        print_board(&state);

        let actions = bot.decide_actions(&state);
        println!(
            "\n🤖 {} 决定执行 {} 个动作:",
            player_name(active),
            actions.len()
        );

        for (i, action) in actions.iter().enumerate() {
            let action_desc = match action {
                Action::PlayCard { card } => {
                    format!("打出 {}", entity_info(&state, *card))
                }
                Action::Attack { attacker, defender } => {
                    format!(
                        "{} 攻击 {}",
                        entity_info(&state, *attacker),
                        entity_info(&state, *defender)
                    )
                }
                Action::EndTurn => "结束回合".to_string(),
                Action::HeroPower { .. } => "使用英雄技能".to_string(),
            };
            println!("  [{}/{}] {}", i + 1, actions.len(), action_desc);

            match engine.apply(&mut state, *action) {
                Ok(events) => {
                    for event in &events {
                        let evt_str = match event {
                            orange_stone::core::event::Event::TurnStarted { player } => {
                                format!("  ↪ 回合开始: {}", player_name(*player))
                            }
                            orange_stone::core::event::Event::TurnEnded { player } => {
                                format!("  ↪ 回合结束: {}", player_name(*player))
                            }
                            orange_stone::core::event::Event::CardPlayed { player, .. } => {
                                format!("  ↪ 卡牌打出: {}", player_name(*player))
                            }
                            orange_stone::core::event::Event::MinionSummoned {
                                player,
                                minion,
                                ..
                            } => {
                                format!(
                                    "  ↪ 随从召唤: {} → {}",
                                    player_name(*player),
                                    entity_info(&state, *minion)
                                )
                            }
                            orange_stone::core::event::Event::AttackDeclared {
                                attacker,
                                defender,
                            } => {
                                format!(
                                    "  ↪ 攻击宣言: {} → {}",
                                    entity_info(&state, *attacker),
                                    entity_info(&state, *defender)
                                )
                            }
                            orange_stone::core::event::Event::DamageDealt {
                                target,
                                amount,
                                ..
                            } => {
                                let hp = state.world().health(*target).unwrap_or(Health(0));
                                format!(
                                    "  ↪ 受到 {} 点伤害: {} (剩余 {} HP)",
                                    amount,
                                    entity_info(&state, *target),
                                    hp.0
                                )
                            }
                            orange_stone::core::event::Event::MinionDied { minion } => {
                                format!("  ↪ 💀 随从死亡: {}", entity_info(&state, *minion))
                            }
                            orange_stone::core::event::Event::CardDrawn { player, .. } => {
                                format!("  ↪ 抽牌: {}", player_name(*player))
                            }
                            orange_stone::core::event::Event::GameOver { winner } => {
                                format!("  ↪ 🏆 游戏结束! 胜者: {}", player_name(*winner))
                            }
                            orange_stone::core::event::Event::WeaponEquipped {
                                player,
                                weapon,
                                ..
                            } => {
                                format!(
                                    "  ↪ 武器装备: {} → {}",
                                    player_name(*player),
                                    entity_info(&state, *weapon)
                                )
                            }
                            orange_stone::core::event::Event::WeaponDestroyed {
                                player,
                                weapon,
                                ..
                            } => {
                                format!(
                                    "  ↪ 💥 武器摧毁: {} → {}",
                                    player_name(*player),
                                    entity_info(&state, *weapon)
                                )
                            }
                            orange_stone::core::event::Event::HeroPowerActivated {
                                player, ..
                            } => {
                                format!("  ↪ 英雄技能: {}", player_name(*player))
                            }
                            orange_stone::core::event::Event::SecretRevealed { player, .. } => {
                                format!("  ↪ 奥秘揭示: {}", player_name(*player))
                            }
                            orange_stone::core::event::Event::SpellCast { player, .. } => {
                                format!("  ↪ 法术施放: {}", player_name(*player))
                            }
                        };
                        println!("{evt_str}");
                    }
                    // 游戏结束后停止处理
                    if matches!(state.phase(), Phase::GameOver { .. }) {
                        break;
                    }
                }
                Err(err) => {
                    println!("  ❌ 动作失败: {err:?}");
                }
            }
        }

        // 检查游戏是否结束
        if matches!(state.phase(), Phase::GameOver { .. }) {
            break;
        }
    }

    // ===== 最终结果 =====
    println!("\n📊 === 对战结束 ===");
    print_board(&state);

    match state.phase() {
        Phase::GameOver { winner } => {
            println!("\n🏆 胜者: {}!", player_name(winner));
            let loser = winner.opponent();
            let winner_hp = state.world().health(state.player(winner).hero);
            let loser_hp = state.world().health(state.player(loser).hero);
            println!(
                "   {} 英雄 HP: {}",
                player_name(winner),
                winner_hp.map(|h| h.0).unwrap_or(0)
            );
            println!(
                "   {} 英雄 HP: {}",
                player_name(loser),
                loser_hp.map(|h| h.0).unwrap_or(0)
            );
            println!("  总回合数: {}", state.turn());
        }
        _ => {
            println!("\n⚠️ 游戏未正常结束 (达到最大回合数)");
        }
    }
}

#[test]
fn two_smart_bots_battle() {
        let engine = GameEngine::new();
        let bot = SmartBot::new();

        // ===== 构建初始对局 =====
        let mut builder = GameBuilder::new();

        // --- Player1 牌库 (10 张) ---
        let p1_cards = [
            &MURLOC_RAIDER,         // 1费 2/1
            &MURLOC_RAIDER,         // 1费 2/1
            &VOODOO_DOCTOR,         // 1费 2/1 战吼:回2
            &VOIDWALKER,            // 1费 1/3 嘲讽
            &BLOODFEN_RAPTOR,       // 2费 3/2
            &BLOODFEN_RAPTOR,       // 2费 3/2
            &KOBOLD_GEOMANCER,      // 2费 2/2 法伤+1
            &MURLOC_TIDEHUNTER,     // 2费 2/1 战吼:召唤1/1
            &OGRE_MAGI,             // 4费 4/4
            &ARCHMAGE,              // 6费 4/7
        ];
        for card in &p1_cards {
            builder.add_minion_to_deck(PlayerId::Player1, card);
        }

        // --- Player2 牌库 (10 张) ---
        let p2_cards = [
            &MURLOC_RAIDER,
            &MURLOC_RAIDER,
            &VOODOO_DOCTOR,
            &VOIDWALKER,
            &BLOODFEN_RAPTOR,
            &ACIDIC_SWAMP_OOZE,
            &ACIDIC_SWAMP_OOZE,
            &OGRE_MAGI,
            &EAGLEHORN_BOW,
            &ARCHMAGE,
        ];
        for card in &p2_cards {
            builder.add_minion_to_deck(PlayerId::Player2, card);
        }

        builder.set_mana(PlayerId::Player1, 0, 0);
        builder.set_mana(PlayerId::Player2, 0, 0);
        let mut state = builder.build();

        // 手动给每人抽 3 张起手牌
        for &pid in &[PlayerId::Player1, PlayerId::Player2] {
            for _ in 0..3 {
                let deck_len = state.world().zones().len(Zone::Deck, pid);
                if deck_len > 0 {
                    let idx = state.rng_mut().next_usize(deck_len);
                    let card = state
                        .world()
                        .zones()
                        .iter(Zone::Deck, pid)
                        .nth(idx)
                        .expect("deck card");
                    state
                        .world_mut()
                        .move_to_zone(card, Zone::Hand)
                        .expect("move to hand");
                }
            }
        }

        // ===== 对局循环 =====
        println!("\n🧠 === SmartBot 对战开始! ===");
        let max_turns = 60; // SmartBot 可能更聪明，给更多回合
        let mut turn_count = 0;

        loop {
            turn_count += 1;
            if turn_count > max_turns {
                println!("\n⏰ 达到最大回合数限制 ({})，强制结束", max_turns);
                break;
            }

            if matches!(state.phase(), Phase::GameOver { .. }) {
                break;
            }

            let active = state.active_player();
            print_board(&state);

            let actions = bot.decide_actions(&state);
            println!(
                "\n🧠 {} (SmartBot) 决定执行 {} 个动作:",
                player_name(active),
                actions.len()
            );

            for (i, action) in actions.iter().enumerate() {
                let action_desc = match action {
                    Action::PlayCard { card } => {
                        format!("打出 {}", entity_info(&state, *card))
                    }
                    Action::Attack { attacker, defender } => {
                        format!(
                            "{} 攻击 {}",
                            entity_info(&state, *attacker),
                            entity_info(&state, *defender)
                        )
                    }
                    Action::EndTurn => "结束回合".to_string(),
                    Action::HeroPower { .. } => "使用英雄技能".to_string(),
                };
                println!("  [{}/{}] {}", i + 1, actions.len(), action_desc);

                match engine.apply(&mut state, *action) {
                    Ok(events) => {
                        for event in &events {
                            let evt_str = match event {
                                orange_stone::core::event::Event::TurnStarted { player } => {
                                    format!("  ↪ 回合开始: {}", player_name(*player))
                                }
                                orange_stone::core::event::Event::TurnEnded { player } => {
                                    format!("  ↪ 回合结束: {}", player_name(*player))
                                }
                                orange_stone::core::event::Event::CardPlayed { player, .. } => {
                                    format!("  ↪ 卡牌打出: {}", player_name(*player))
                                }
                                orange_stone::core::event::Event::MinionSummoned {
                                    player,
                                    minion,
                                    ..
                                } => {
                                    format!(
                                        "  ↪ 随从召唤: {} → {}",
                                        player_name(*player),
                                        entity_info(&state, *minion)
                                    )
                                }
                                orange_stone::core::event::Event::AttackDeclared {
                                    attacker,
                                    defender,
                                } => {
                                    format!(
                                        "  ↪ 攻击宣言: {} → {}",
                                        entity_info(&state, *attacker),
                                        entity_info(&state, *defender)
                                    )
                                }
                                orange_stone::core::event::Event::DamageDealt {
                                    target,
                                    amount,
                                    ..
                                } => {
                                    let hp = state.world().health(*target).unwrap_or(Health(0));
                                    format!(
                                        "  ↪ 受到 {} 点伤害: {} (剩余 {} HP)",
                                        amount,
                                        entity_info(&state, *target),
                                        hp.0
                                    )
                                }
                                orange_stone::core::event::Event::MinionDied { minion } => {
                                    format!("  ↪ 💀 随从死亡: {}", entity_info(&state, *minion))
                                }
                                orange_stone::core::event::Event::CardDrawn { player, .. } => {
                                    format!("  ↪ 抽牌: {}", player_name(*player))
                                }
                                orange_stone::core::event::Event::GameOver { winner } => {
                                    format!("  ↪ 🏆 游戏结束! 胜者: {}", player_name(*winner))
                                }
                                orange_stone::core::event::Event::WeaponEquipped {
                                    player,
                                    weapon,
                                    ..
                                } => {
                                    format!(
                                        "  ↪ 武器装备: {} → {}",
                                        player_name(*player),
                                        entity_info(&state, *weapon)
                                    )
                                }
                                orange_stone::core::event::Event::WeaponDestroyed {
                                    player,
                                    weapon,
                                    ..
                                } => {
                                    format!(
                                        "  ↪ 💥 武器摧毁: {} → {}",
                                        player_name(*player),
                                        entity_info(&state, *weapon)
                                    )
                                }
                                orange_stone::core::event::Event::HeroPowerActivated {
                                    player, ..
                                } => {
                                    format!("  ↪ 英雄技能: {}", player_name(*player))
                                }
                                orange_stone::core::event::Event::SecretRevealed { player, .. } => {
                                    format!("  ↪ 奥秘揭示: {}", player_name(*player))
                                }
                                orange_stone::core::event::Event::SpellCast { player, .. } => {
                                    format!("  ↪ 法术施放: {}", player_name(*player))
                                }
                            };
                            println!("{evt_str}");
                        }
                        if matches!(state.phase(), Phase::GameOver { .. }) {
                            break;
                        }
                    }
                    Err(err) => {
                        println!("  ❌ 动作失败: {err:?}");
                    }
                }
            }

            if matches!(state.phase(), Phase::GameOver { .. }) {
                break;
            }
        }

        // ===== 最终结果 =====
        println!("\n📊 === SmartBot 对战结束 ===");
        print_board(&state);

        match state.phase() {
            Phase::GameOver { winner } => {
                println!("\n🧠🏆 胜者: {}!", player_name(winner));
                let loser = winner.opponent();
                let winner_hp = state.world().health(state.player(winner).hero);
                let loser_hp = state.world().health(state.player(loser).hero);
                println!(
                    "   {} 英雄 HP: {}",
                    player_name(winner),
                    winner_hp.map(|h| h.0).unwrap_or(0)
                );
                println!(
                    "   {} 英雄 HP: {}",
                    player_name(loser),
                    loser_hp.map(|h| h.0).unwrap_or(0)
                );
                println!("  总回合数: {}", state.turn());
            }
            _ => {
                println!("\n⚠️ 游戏未正常结束 (达到最大回合数)");
            }
        }
    }
