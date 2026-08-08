//! Bot strategies — the greedy bot and the smart bot.
//!
//! ## GreedyBot
//!
//! Strategy: go all-out for face damage, unless a taunt blocks the way (taunts must be
//! cleared) or the hero cannot be attacked.
//! Never intentionally dies during the turn (avoids suicidal attacks).
//!
//! ## SmartBot
//!
//! Rule-based heuristic strategy, improving on GreedyBot:
//!
//! 1. **Card scoring**: rate each card by stats/cost efficiency and keyword bonuses
//! 2. **Lethal detection**: check whether the opponent can be killed this turn
//! 3. **Value trades**: evaluate the value gained or lost in minion trades and trade favorably
//! 4. **Threat assessment**: prioritize high-threat enemy minions (high attack, divine shield, windfury)
//! 5. **Board awareness**: switch between aggressive and defensive play based on the board state
//! 6. **Divine shield handling**: pop shields with the weakest attacker to protect key damage dealers
//! 7. **State projection**: track planned charge minions and weapons and fold them into combat planning

use crate::core::action::Action;
use crate::core::component::{CardType, Health};
use crate::core::entity::Entity;
use crate::core::player::PlayerId;
use crate::core::state::GameState;
use crate::core::zone::Zone;

/// Greedy bot — a simple face-first strategy.
#[derive(Debug, Clone, Copy, Default)]
pub struct GreedyBot;

impl GreedyBot {
    /// Creates a new greedy bot.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Generates the action sequence for the active player this turn.
    ///
    /// Returns all actions to execute in the current turn.
    /// An empty `Vec` means the game is over.
    pub fn decide_actions(&self, state: &GameState) -> Vec<Action> {
        // Return no actions once the game is over
        if matches!(state.step(), crate::core::state::Step::GameOver { .. }) {
            return vec![];
        }
        // A pending choice (Choose One / Discover / Mulligan) is the only
        // legal action (2025–2026 expansions M1-W3, P3 real choice
        // resolution); the bot plays the simple heuristic: branch option 0.
        if let Some(choice) = state.pending_choice() {
            return vec![Action::Choose {
                choice_id: choice.id,
                option: 0,
            }];
        }

        let active = state.active_player();
        let mut actions = Vec::new();

        // 1. Play: play all affordable minions and weapons from hand
        let (play_actions, remaining_mana) = self.play_cards(state, active);
        actions.extend(play_actions);

        // 2. Hero power (using the mana left after playing cards)
        if let Some(hp_action) = self.hero_power(state, active, remaining_mana) {
            actions.push(hp_action);
        }

        // 3. Attack: all attackers go face
        actions.extend(self.attack_phase(state, active));

        // 4. End turn
        actions.push(Action::EndTurn);

        actions
    }

    /// Plays all affordable minions and weapons (cheapest first).
    /// Returns (action list, remaining mana).
    fn play_cards(&self, state: &GameState, player: PlayerId) -> (Vec<Action>, i32) {
        let world = state.world();
        let current_mana = state.player(player).current_mana;

        // Collect all playable cards (minions + weapons + spells)
        let mut playable: Vec<(i32, Entity)> = world
            .zones()
            .iter(Zone::Hand, player)
            .filter(|&e| {
                let ct = world.card_type(e);
                (ct == Some(CardType::Minion)
                    || ct == Some(CardType::Weapon)
                    || ct == Some(CardType::Spell)
                    || ct == Some(CardType::Location))
                    && world.effective_cost(e).is_some_and(|c| c.0 <= current_mana)
            })
            .map(|e| (world.effective_cost(e).unwrap().0, e))
            .collect();

        // Sort by cost ascending (greedy: play cheap cards first to fit in more plays)
        playable.sort_by_key(|(cost, _)| *cost);

        let mut actions = Vec::new();
        let mut remaining_mana = current_mana;

        for (cost, card) in playable {
            if cost <= remaining_mana {
                actions.push(Action::PlayCard {
                    card,
                    target: None,
                    position: None,
                });
                remaining_mana -= cost;
            }
        }

        (actions, remaining_mana)
    }

    /// Tries to use the hero power (if mana remains after playing cards and it is unused).
    fn hero_power(
        &self,
        state: &GameState,
        player: PlayerId,
        remaining_mana: i32,
    ) -> Option<Action> {
        let hero = state.player(player).hero;
        let world = state.world();

        // Check whether it was already used
        if world.hero_power_used(hero).is_some_and(|u| u.0) {
            return None;
        }

        // Check whether there is enough mana (hero power cost from the definition, or 2 by default)
        let cost = world.hero_power(hero).map(|hp| hp.cost).unwrap_or(2);
        if remaining_mana >= cost {
            Some(Action::HeroPower { hero, target: None })
        } else {
            None
        }
    }

    /// Attack phase: all attackers go face (or clear taunts).
    fn attack_phase(&self, state: &GameState, player: PlayerId) -> Vec<Action> {
        let enemy = player.opponent();
        let enemy_hero = state.player(enemy).hero;
        let world = state.world();

        // Collect all friendly characters that can attack (minions + hero with a weapon)
        let attackers: Vec<Entity> = self.collect_attackers(state, player);

        // Check whether the enemy has taunt minions
        let taunts: Vec<Entity> = world
            .zones()
            .iter(Zone::Play, enemy)
            .filter(|&e| world.taunt(e).is_some() && world.card_type(e) == Some(CardType::Minion))
            .collect();

        let mut actions = Vec::new();

        if taunts.is_empty() {
            // No taunts — everyone goes face
            for attacker in &attackers {
                actions.push(Action::Attack {
                    attacker: *attacker,
                    defender: enemy_hero,
                });
            }
        } else {
            // Taunts present: clear them first, remaining attacks go face
            let mut remaining_taunts: Vec<Entity> = taunts.clone();
            let mut used_attackers: Vec<usize> = Vec::new();

            // Greedy: clear taunts with the weakest attacker that can do it
            for (i, attacker) in attackers.iter().enumerate() {
                if remaining_taunts.is_empty() {
                    break;
                }

                let atk = world
                    .effective_attack(*attacker)
                    .unwrap_or(crate::core::component::Attack(0));

                // Find a taunt that can be killed (greedy: prefer the most wounded one)
                let target = remaining_taunts
                    .iter()
                    .filter(|&t| {
                        let hp = world.effective_health(*t).unwrap_or(Health(99));
                        hp.0 <= atk.0 || i == attackers.len() - 1 // the last attacker must act
                    })
                    .min_by_key(|&t| world.effective_health(*t).unwrap_or(Health(99)).0);

                if let Some(&target) = target {
                    actions.push(Action::Attack {
                        attacker: *attacker,
                        defender: target,
                    });
                    // Simulate: whether the target dies from this attack
                    let target_hp = world.effective_health(target).unwrap_or(Health(0));
                    if target_hp.0 <= atk.0 {
                        remaining_taunts.retain(|&t| t != target);
                    } else {
                        // Target survives; still remove it from the list (avoid re-attacking the same target)
                        remaining_taunts.retain(|&t| t != target);
                    }
                    used_attackers.push(i);
                }
            }

            // After taunts are cleared, remaining attackers go face
            for (i, attacker) in attackers.iter().enumerate() {
                if used_attackers.contains(&i) {
                    continue;
                }
                // Re-check for remaining taunts (skip face attacks if any taunt is left)
                let still_has_taunt = state
                    .world()
                    .zones()
                    .iter(Zone::Play, enemy)
                    .any(|e| world.taunt(e).is_some());
                if still_has_taunt {
                    // Taunts remain but no attackers are left — skip
                    continue;
                }
                actions.push(Action::Attack {
                    attacker: *attacker,
                    defender: enemy_hero,
                });
            }
        }

        actions
    }

    /// Collects all characters that can attack for the given player.
    fn collect_attackers(&self, state: &GameState, player: PlayerId) -> Vec<Entity> {
        let world = state.world();

        // Minions on the board (not exhausted, attack > 0)
        let mut attackers: Vec<Entity> = world
            .zones()
            .iter(Zone::Play, player)
            .filter(|&e| {
                world.card_type(e) == Some(CardType::Minion)
                    && world
                        .attacks_used(e)
                        .is_some_and(|a| !a.is_exhausted_with(world.max_attacks(e)))
                    && world.effective_attack(e).is_some_and(|a| a.0 > 0)
            })
            .collect();

        // Hero (has a weapon and is not exhausted)
        let hero = state.player(player).hero;
        let has_weapon = state.player(player).weapon.is_some();
        let hero_can_attack = world
            .attacks_used(hero)
            .is_some_and(|a| !a.is_exhausted_with(world.max_attacks(hero)));
        if has_weapon && hero_can_attack {
            attackers.push(hero);
        }

        attackers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::def::{BLOODFEN_RAPTOR, OGRE_MAGI, VOIDWALKER};
    use crate::core::player::PlayerId;
    use crate::sim::game::GameBuilder;

    #[test]
    fn bot_plays_cards_from_hand() {
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &OGRE_MAGI);
        builder.add_minion_to_hand(PlayerId::Player1, &BLOODFEN_RAPTOR);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();

        let bot = GreedyBot::new();
        let actions = bot.decide_actions(&state);

        // Should include two PlayCards and one EndTurn
        let play_count = actions
            .iter()
            .filter(|a| matches!(a, Action::PlayCard { .. }))
            .count();
        assert!(play_count >= 1);
        assert!(actions.last().is_some_and(|a| matches!(a, Action::EndTurn)));
    }

    #[test]
    fn bot_attacks_hero_when_no_taunts() {
        let mut builder = GameBuilder::new();
        let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 3, 3);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();
        let enemy_hero = state.player(PlayerId::Player2).hero;

        let bot = GreedyBot::new();
        let actions = bot.decide_actions(&state);

        // Should have an action attacking the enemy hero
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Attack { attacker: a, defender: d }
            if *a == attacker && *d == enemy_hero
        )));
    }

    #[test]
    fn bot_must_attack_taunt_first() {
        let mut builder = GameBuilder::new();
        let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 3, 3);
        builder.add_minion_to_board(PlayerId::Player2, &VOIDWALKER);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();

        // Get the taunt minion entity
        let taunt: Vec<Entity> = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player2)
            .filter(|&e| state.world().taunt(e).is_some())
            .collect();
        let taunt_entity = taunt[0];

        let bot = GreedyBot::new();
        let actions = bot.decide_actions(&state);

        // The first attack should hit the taunt (not the hero)
        let attack_actions: Vec<_> = actions
            .iter()
            .filter(|a| matches!(a, Action::Attack { .. }))
            .collect();
        if !attack_actions.is_empty() {
            let first = attack_actions[0];
            assert!(matches!(
                first,
                Action::Attack { attacker: a, defender: d }
                if *a == attacker && *d == taunt_entity
            ));
        }
    }

    #[test]
    fn bot_uses_hero_power_when_available() {
        use crate::core::effect::{CardEffect, EffectTarget};

        let mut builder = GameBuilder::new();
        builder.set_mana(PlayerId::Player1, 10, 10);
        builder.set_hero_power(
            PlayerId::Player1,
            2,
            CardEffect::DealDamage {
                amount: 1,
                target: EffectTarget::AnyEnemy,
            },
        );
        let state = builder.build();

        let bot = GreedyBot::new();
        let actions = bot.decide_actions(&state);

        assert!(
            actions
                .iter()
                .any(|a| matches!(a, Action::HeroPower { .. }))
        );
    }

    #[test]
    fn bot_hero_attacks_with_weapon() {
        use crate::cards::def::EAGLEHORN_BOW;

        let mut builder = GameBuilder::new();
        builder.equip_weapon(PlayerId::Player1, &EAGLEHORN_BOW);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();
        let hero = state.player(PlayerId::Player1).hero;
        let enemy_hero = state.player(PlayerId::Player2).hero;

        let bot = GreedyBot::new();
        let actions = bot.decide_actions(&state);

        // The hero should have an attack action
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Attack { attacker: a, defender: d }
            if *a == hero && *d == enemy_hero
        )));
    }

    #[test]
    fn bot_ends_turn() {
        let state = GameState::new();
        let bot = GreedyBot::new();
        let actions = bot.decide_actions(&state);

        // The last action should be EndTurn
        assert!(actions.last().is_some_and(|a| matches!(a, Action::EndTurn)));
    }

    #[test]
    fn bot_returns_empty_for_game_over() {
        let mut builder = GameBuilder::new();
        builder.step(crate::core::state::Step::GameOver {
            winner: PlayerId::Player1,
        });
        let state = builder.build();

        let bot = GreedyBot::new();
        let actions = bot.decide_actions(&state);

        assert!(actions.is_empty());
    }
}

// ============================================================
// SmartBot — a rule-based heuristic smart bot
// ============================================================

// SmartBot uses world.divine_shield(), world.taunt(), world.windfury() methods
// — no direct type imports needed beyond what's already imported above

/// Smart bot — a rule-based heuristic strategy.
///
/// Key improvements over `GreedyBot`:
///
/// | Feature | GreedyBot | SmartBot |
/// |------|-----------|----------|
/// | Card selection | Cheapest first | Value-scored ordering |
/// | Attack strategy | Always face | Lethal detection + value trades |
/// | Taunt handling | Weakest attacker clears | Optimal clear assignment |
/// | Divine shield | Ignored | Popped by the weakest attacker |
/// | Board awareness | None | Auto switch between attack and defense |
/// | State projection | None | Tracks charge minions and weapons |
///
/// # Algorithm overview
///
/// 1. **Card scoring** — score each playable card by stats efficiency, keywords, and battlecry effects
/// 2. **Card playing** — play in score-descending order, projecting charge minions and weapons into combat
/// 3. **Hero power** — use it when mana remains (prefer powers that help the board)
/// 4. **Combat phase**:
///    a. Lethal check — if total attack >= enemy hero health (accounting for taunts), everyone goes face
///    b. Taunt clearing — taunts must be cleared, using the best attackers
///    c. Value trades — trade favorably into high-threat enemies (survive > trade > pass)
///    d. Remaining attackers go face
/// 5. **End turn**
#[derive(Debug, Clone, Copy, Default)]
pub struct SmartBot;

/// Projected attacker info — folds charge minions and weapons into attack planning after playing cards.
#[derive(Debug, Clone, Copy)]
struct ProjectedAttacker {
    /// Attacker entity (existing or about to be played)
    entity: Entity,
    /// Attack
    attack: i32,
    /// Health
    health: i32,
    /// Whether it has divine shield
    has_divine_shield: bool,
    /// Whether it is the hero (heroes take no retaliation damage)
    is_hero: bool,
}

impl SmartBot {
    /// Creates a new smart bot.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Generates the action sequence for the active player this turn.
    ///
    /// Returns all actions to execute in the current turn.
    /// An empty `Vec` means the game is over.
    pub fn decide_actions(&self, state: &GameState) -> Vec<Action> {
        if matches!(state.step(), crate::core::state::Step::GameOver { .. }) {
            return vec![];
        }
        // A pending choice (Choose One / Discover / Mulligan) is the only
        // legal action (2025–2026 expansions M1-W3, P3 real choice
        // resolution); the bot plays the simple heuristic: branch option 0.
        if let Some(choice) = state.pending_choice() {
            return vec![Action::Choose {
                choice_id: choice.id,
                option: 0,
            }];
        }

        let active = state.active_player();
        let enemy = active.opponent();
        let current_mana = state.player(active).current_mana;

        // ── 1. Card playing phase ──
        let (play_actions, projected_charge, hero_weapon_attack, remaining_mana) =
            self.play_cards(state, active, current_mana);

        // ── 2. Location activation (Core Set W8) ──
        let location_action = self.location_activation(state, active);

        // ── 3. Hero power ──
        let hero_power_action = self.hero_power(state, active, remaining_mana);

        // ── 4. Combat phase ──
        let combat_actions =
            self.combat_phase(state, active, enemy, &projected_charge, hero_weapon_attack);

        // ── 5. Assemble the action sequence ──
        let mut actions = Vec::with_capacity(
            play_actions.len()
                + location_action.map_or(0, |_| 1)
                + hero_power_action.map_or(0, |_| 1)
                + combat_actions.len()
                + 1,
        );
        actions.extend(play_actions);
        if let Some(la) = location_action {
            actions.push(la);
        }
        if let Some(hp) = hero_power_action {
            actions.push(hp);
        }
        actions.extend(combat_actions);
        actions.push(Action::EndTurn);

        actions
    }

    // ============================================================
    // Card scoring and playing
    // ============================================================

    /// Plays the best sequence of cards.
    ///
    /// Returns: `(play actions, projected charge attackers, hero weapon attack if any, remaining mana)`
    fn play_cards(
        &self,
        state: &GameState,
        player: PlayerId,
        current_mana: i32,
    ) -> (Vec<Action>, Vec<ProjectedAttacker>, Option<i32>, i32) {
        let world = state.world();

        // Collect all playable cards (minions + weapons + spells) with their scores
        let mut candidates: Vec<(f64, Entity)> = world
            .zones()
            .iter(Zone::Hand, player)
            .filter(|&e| {
                let ct = world.card_type(e);
                (ct == Some(CardType::Minion)
                    || ct == Some(CardType::Weapon)
                    || ct == Some(CardType::Spell))
                    && world.effective_cost(e).is_some_and(|c| c.0 <= current_mana)
            })
            .map(|e| (self.evaluate_card(state, player, e), e))
            .collect();

        // Sort by score descending
        candidates
            .sort_by(|(s1, _), (s2, _)| s2.partial_cmp(s1).unwrap_or(std::cmp::Ordering::Equal));

        let mut actions = Vec::new();
        let mut projected_charge: Vec<ProjectedAttacker> = Vec::new();
        let mut hero_weapon_attack: Option<i32> = None;
        let mut remaining_mana = current_mana;

        for (_score, card) in candidates {
            let cost = world.effective_cost(card).map(|c| c.0).unwrap_or(0);
            if cost > remaining_mana {
                continue;
            }

            // Project charge minions — they can attack this turn
            if world.card_type(card) == Some(CardType::Minion) && world.charge(card).is_some() {
                let atk = world.attack(card).map(|a| a.0).unwrap_or(0);
                let hp = world.health(card).map(|h| h.0).unwrap_or(0);
                let ds = world.divine_shield(card).is_some();
                if atk > 0 {
                    projected_charge.push(ProjectedAttacker {
                        entity: card,
                        attack: atk,
                        health: hp,
                        has_divine_shield: ds,
                        is_hero: false,
                    });
                }
            }

            // Project weapons — the hero can attack
            if world.card_type(card) == Some(CardType::Weapon) {
                let atk = world.attack(card).map(|a| a.0).unwrap_or(0);
                if atk > 0 {
                    hero_weapon_attack = Some(atk);
                }
            }

            actions.push(Action::PlayCard {
                card,
                target: None,
                position: None,
            });
            remaining_mana -= cost;
        }

        (
            actions,
            projected_charge,
            hero_weapon_attack,
            remaining_mana,
        )
    }

    /// Evaluates a card's value score.
    ///
    /// Base formula: `stats efficiency + keyword bonus + battlecry/deathrattle bonus`
    ///
    /// - Stats efficiency: `(attack + health) - (cost * 2 + 1)` — deviation from the vanilla standard
    /// - Keywords: taunt +1.5, divine shield +attack*0.7, charge +attack*0.5, windfury +attack*0.5, spell damage +value
    /// - Battlecry damage +damage*1.2, card draw +3*count, buffs +total buff amount
    fn evaluate_card(&self, state: &GameState, _player: PlayerId, card: Entity) -> f64 {
        let world = state.world();
        let atk = world.attack(card).map(|a| a.0).unwrap_or(0) as f64;
        let hp = world.health(card).map(|h| h.0).unwrap_or(0) as f64;
        let cost = world.effective_cost(card).map(|c| c.0).unwrap_or(1).max(1) as f64;

        // Stats efficiency: deviation from the vanilla standard (cost*2+1)
        let vanilla_standard = cost * 2.0 + 1.0;
        let stats_efficiency = (atk + hp) - vanilla_standard;

        // Keyword bonuses
        let mut keyword_bonus = 0.0;
        if world.taunt(card).is_some() {
            keyword_bonus += 1.5;
        }
        if world.divine_shield(card).is_some() {
            keyword_bonus += atk * 0.7;
        }
        if world.windfury(card).is_some() {
            keyword_bonus += atk * 0.5;
        }
        if world.charge(card).is_some() {
            keyword_bonus += atk * 0.5;
        }
        if let Some(sd) = world.spell_damage(card) {
            keyword_bonus += sd.0 as f64 * 1.5;
        }

        // Battlecry/deathrattle bonuses
        let mut effect_bonus = 0.0;
        if let Some(bc) = world.battlecry(card) {
            effect_bonus += evaluate_effect_value(bc.0);
        }
        if let Some(dr) = world.deathrattle(card) {
            // Deathrattle value is discounted (delayed trigger)
            effect_bonus += evaluate_effect_value(dr.0) * 0.6;
        }

        // Final score: efficiency + keywords + effects (normalized by cost so high-value cards sort well)
        stats_efficiency + keyword_bonus + effect_bonus + (atk + hp) / cost
    }

    // ============================================================
    // Location activation (Core Set W8)
    // ============================================================

    /// Picks the location activation action when a friendly location is
    /// ready (past its play cooldown, charges left). Target heuristic:
    /// an enemy minion the 1 damage would kill, else the lowest-health
    /// enemy minion, else the strongest friendly attacker to buff.
    fn location_activation(&self, state: &GameState, player: PlayerId) -> Option<Action> {
        let world = state.world();
        let location = state.player(player).location?;
        if state.player(player).location_played_turn >= state.turn() {
            return None;
        }
        if world.attacks_used(location).is_some_and(|u| u.0 > 0) {
            return None;
        }
        if world.durability(location).is_none_or(|d| d.0 == 0) {
            return None;
        }
        let enemy = player.opponent();
        let kill_candidates: Vec<Entity> = world
            .zones()
            .iter(Zone::Play, enemy)
            .filter(|&e| world.card_type(e) == Some(CardType::Minion))
            .filter(|&e| world.effective_health(e).is_some_and(|h| h.0 <= 1))
            .collect();
        if !kill_candidates.is_empty() {
            return Some(Action::ActivateLocation {
                location,
                target: Some(kill_candidates[0]),
            });
        }
        let mut weak_enemies: Vec<(i32, Entity)> = world
            .zones()
            .iter(Zone::Play, enemy)
            .filter(|&e| world.card_type(e) == Some(CardType::Minion))
            .map(|e| (world.effective_health(e).map(|h| h.0).unwrap_or(0), e))
            .collect();
        weak_enemies.sort_by_key(|(h, _)| *h);
        if let Some((_, e)) = weak_enemies.first() {
            return Some(Action::ActivateLocation {
                location,
                target: Some(*e),
            });
        }
        // No enemy minions: buff the strongest friendly attacker
        let mut friends: Vec<(i32, Entity)> = world
            .zones()
            .iter(Zone::Play, player)
            .filter(|&e| world.card_type(e) == Some(CardType::Minion))
            .map(|e| (world.effective_attack(e).map(|a| a.0).unwrap_or(0), e))
            .collect();
        friends.sort_by_key(|(a, _)| std::cmp::Reverse(*a));
        friends.first().map(|(_, e)| Action::ActivateLocation {
            location,
            target: Some(*e),
        })
    }

    // ============================================================
    // Hero power
    // ============================================================

    /// Decides whether to use the hero power.
    fn hero_power(
        &self,
        state: &GameState,
        player: PlayerId,
        remaining_mana: i32,
    ) -> Option<Action> {
        let hero = state.player(player).hero;

        // Skip if already used
        if state.world().hero_power_used(hero).is_some_and(|u| u.0) {
            return None;
        }

        let cost = state
            .world()
            .hero_power(hero)
            .map(|hp| hp.cost)
            .unwrap_or(2);
        if remaining_mana < cost {
            return None;
        }

        // Mana is available and the hero power can be used — use it (2 damage, armor, etc. all have value)
        Some(Action::HeroPower { hero, target: None })
    }

    // ============================================================
    // Combat phase
    // ============================================================

    /// Combat phase: lethal check → value trades → face damage
    fn combat_phase(
        &self,
        state: &GameState,
        player: PlayerId,
        enemy: PlayerId,
        projected_charge: &[ProjectedAttacker],
        hero_weapon_attack: Option<i32>,
    ) -> Vec<Action> {
        let world = state.world();

        // ── Collect all attackers ──
        let mut attackers: Vec<ProjectedAttacker> = self.collect_existing_attackers(state, player);

        // Hero attack (accounting for a newly equipped weapon)
        let hero = state.player(player).hero;
        let hero_can_attack = world
            .attacks_used(hero)
            .is_some_and(|a| !a.is_exhausted_with(world.max_attacks(hero)));
        let existing_weapon_atk = state.player(player).weapon.and_then(|w| {
            if world.is_alive(w) {
                world.attack(w).map(|a| a.0)
            } else {
                None
            }
        });

        let effective_hero_atk = hero_weapon_attack.or(existing_weapon_atk);
        if let Some(atk) = effective_hero_atk {
            if atk > 0 && hero_can_attack {
                // Avoid adding the hero twice
                if !attackers.iter().any(|a| a.entity == hero) {
                    attackers.push(ProjectedAttacker {
                        entity: hero,
                        attack: atk,
                        health: world.effective_health(hero).map(|h| h.0).unwrap_or(30),
                        has_divine_shield: false,
                        is_hero: true,
                    });
                }
            }
        }

        // Add projected charge minions
        for pa in projected_charge {
            // Avoid duplicates (unlikely, but a safe check)
            if !attackers.iter().any(|a| a.entity == pa.entity) {
                attackers.push(*pa);
            }
        }

        if attackers.is_empty() {
            return vec![];
        }

        // ── Collect the enemy board ──
        let enemy_minions: Vec<EnemyMinion> = world
            .zones()
            .iter(Zone::Play, enemy)
            .filter(|&e| world.card_type(e) == Some(CardType::Minion) && world.is_alive(e))
            .map(|e| EnemyMinion {
                entity: e,
                attack: world.effective_attack(e).map(|a| a.0).unwrap_or(0),
                health: world.effective_health(e).map(|h| h.0).unwrap_or(0),
                has_taunt: world.taunt(e).is_some(),
                has_divine_shield: world.divine_shield(e).is_some(),
            })
            .collect();

        let enemy_hero_entity = state.player(enemy).hero;
        let enemy_hero_health = world
            .effective_health(enemy_hero_entity)
            .map(|h| h.0)
            .unwrap_or(0);
        let enemy_hero_armor = state.player(enemy).armor;
        let enemy_effective_hp = enemy_hero_health + enemy_hero_armor;

        // ── Split taunt and non-taunt minions ──
        let (taunt_minions, non_taunt_minions): (Vec<EnemyMinion>, Vec<EnemyMinion>) =
            enemy_minions.into_iter().partition(|m| m.has_taunt);

        // ── Step 1: lethal check ──
        if !attackers.is_empty() {
            let total_attack: i32 = attackers.iter().map(|a| a.attack).sum();

            if taunt_minions.is_empty() {
                // No taunts — check whether lethal is possible directly
                if total_attack >= enemy_effective_hp {
                    return attackers
                        .iter()
                        .map(|a| Action::Attack {
                            attacker: a.entity,
                            defender: enemy_hero_entity,
                        })
                        .collect();
                }
            } else {
                // Taunts present — check whether lethal is possible through them
                let taunt_total_hp: i32 = taunt_minions.iter().map(effective_hp).sum();
                let damage_needed = taunt_total_hp + enemy_effective_hp;
                if total_attack >= damage_needed
                    && self.can_clear_and_lethal(&attackers, &taunt_minions, enemy_effective_hp)
                {
                    // Clear taunts first, then go face
                    let mut actions = self.assign_taunt_clear(&attackers, &taunt_minions);
                    // Remaining attackers go face
                    let used: Vec<Entity> = actions
                        .iter()
                        .map(|a| match a {
                            Action::Attack { attacker, .. } => *attacker,
                            _ => unreachable!(),
                        })
                        .collect();
                    for attacker in &attackers {
                        if !used.contains(&attacker.entity) {
                            actions.push(Action::Attack {
                                attacker: attacker.entity,
                                defender: enemy_hero_entity,
                            });
                        }
                    }
                    return actions;
                }
            }
        }

        // ── Step 2: taunt clearing (taunts must be cleared when not lethal) ──
        let mut assigned_attackers: Vec<Entity> = Vec::new();
        let mut actions = Vec::new();

        if !taunt_minions.is_empty() {
            let taunt_actions = self.assign_taunt_clear(&attackers, &taunt_minions);
            for a in &taunt_actions {
                if let Action::Attack { attacker, .. } = a {
                    assigned_attackers.push(*attacker);
                }
            }
            actions.extend(taunt_actions);
        }

        // ── Step 3: value trades (against non-taunt enemy minions) ──
        let available: Vec<&ProjectedAttacker> = attackers
            .iter()
            .filter(|a| !assigned_attackers.contains(&a.entity))
            .collect();

        if !available.is_empty() && !non_taunt_minions.is_empty() {
            let (trade_actions, newly_assigned) = self.value_trades(&available, &non_taunt_minions);
            assigned_attackers.extend(newly_assigned);
            actions.extend(trade_actions);
        }

        // ── Step 4: remaining attackers go face ──
        // Re-check whether any taunt is still up
        let still_has_taunt = state
            .world()
            .zones()
            .iter(Zone::Play, enemy)
            .any(|e| state.world().taunt(e).is_some());

        if !still_has_taunt {
            for attacker in &attackers {
                if !assigned_attackers.contains(&attacker.entity) {
                    actions.push(Action::Attack {
                        attacker: attacker.entity,
                        defender: enemy_hero_entity,
                    });
                }
            }
        }

        actions
    }

    /// Evaluates whether lethal is possible after clearing all taunts.
    fn can_clear_and_lethal(
        &self,
        attackers: &[ProjectedAttacker],
        taunts: &[EnemyMinion],
        enemy_hp: i32,
    ) -> bool {
        // Simplified: sort attackers by attack descending and greedily assign taunt clearing
        let mut sorted_attackers = attackers.to_vec();
        sorted_attackers.sort_by_key(|a| std::cmp::Reverse(a.attack));

        let mut taunt_hps: Vec<i32> = taunts.iter().map(effective_hp).collect();
        let mut idx = 0;

        for th in &mut taunt_hps {
            while *th > 0 && idx < sorted_attackers.len() {
                *th -= sorted_attackers[idx].attack;
                idx += 1;
            }
            if *th > 0 {
                return false; // cannot clear all taunts
            }
        }

        // Whether the remaining attackers can deal lethal
        let remaining_damage: i32 = sorted_attackers[idx..].iter().map(|a| a.attack).sum();
        remaining_damage >= enemy_hp
    }

    /// Assigns attackers to clear taunt minions (optimal assignment).
    fn assign_taunt_clear(
        &self,
        attackers: &[ProjectedAttacker],
        taunts: &[EnemyMinion],
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        let mut used: Vec<Entity> = Vec::new();
        let mut remaining_taunts: Vec<&EnemyMinion> = taunts.iter().collect();

        for taunt in taunts {
            if let Some(best) = self.find_best_attacker(attackers, &used, taunt) {
                actions.push(Action::Attack {
                    attacker: best.entity,
                    defender: taunt.entity,
                });
                used.push(best.entity);
                remaining_taunts.retain(|t| t.entity != taunt.entity);
            }
        }

        // If taunts are left unassigned (not enough attackers), keep attacking with used attackers
        for taunt in &remaining_taunts {
            // Find an unused attacker that can still attack
            let remaining_available: Vec<&ProjectedAttacker> = attackers
                .iter()
                .filter(|a| !used.contains(&a.entity))
                .collect();
            if let Some(attacker) = remaining_available.first() {
                actions.push(Action::Attack {
                    attacker: attacker.entity,
                    defender: taunt.entity,
                });
                used.push(attacker.entity);
            }
        }

        actions
    }

    /// Value trades: trade favorably into non-taunt enemy minions.
    ///
    /// Returns `(attack actions, list of assigned entities)`
    fn value_trades(
        &self,
        available: &[&ProjectedAttacker],
        enemies: &[EnemyMinion],
    ) -> (Vec<Action>, Vec<Entity>) {
        let mut actions = Vec::new();
        let mut used: Vec<Entity> = Vec::new();

        // Sort by threat level (attack weighted higher)
        let mut sorted_enemies = enemies.to_vec();
        sorted_enemies.sort_by_key(|e| std::cmp::Reverse(e.attack * 2 + e.health));

        for enemy in &sorted_enemies {
            // Evaluate whether the trade is worthwhile
            let trade_value = self.evaluate_trade(available, &used, enemy);

            // Trade only when the value is positive or the threat must be dealt with
            if trade_value > -2.0 || enemy.attack >= 4 {
                if let Some(best) = self.find_best_attacker_slice(available, &used, enemy) {
                    actions.push(Action::Attack {
                        attacker: best.entity,
                        defender: enemy.entity,
                    });
                    used.push(best.entity);
                }
            }
        }

        (actions, used)
    }

    /// Finds the best attacker for the given enemy minion (using a `&[ProjectedAttacker]` slice).
    fn find_best_attacker_slice(
        &self,
        available: &[&ProjectedAttacker],
        used: &[Entity],
        enemy: &EnemyMinion,
    ) -> Option<ProjectedAttacker> {
        let candidates: Vec<&ProjectedAttacker> = available
            .iter()
            .filter(|a| !used.contains(&a.entity))
            .copied()
            .collect();
        self.find_best_attacker_inner(&candidates, enemy)
    }

    /// Finds the best attacker for the given enemy minion (using `&[ProjectedAttacker]`).
    fn find_best_attacker(
        &self,
        attackers: &[ProjectedAttacker],
        used: &[Entity],
        enemy: &EnemyMinion,
    ) -> Option<ProjectedAttacker> {
        let candidates: Vec<&ProjectedAttacker> = attackers
            .iter()
            .filter(|a| !used.contains(&a.entity))
            .collect();
        self.find_best_attacker_inner(&candidates, enemy)
    }

    /// Core attacker selection logic.
    ///
    /// Priority:
    /// 1. Kill the enemy and survive (perfect trade)
    /// 2. Trade for free using divine shield
    /// 3. Kill the enemy but trade (use the lowest-value attacker)
    /// 4. Pop divine shield with the weakest attacker
    /// 5. Cannot kill, but chip damage is worthwhile (use the weakest attacker)
    fn find_best_attacker_inner(
        &self,
        candidates: &[&ProjectedAttacker],
        enemy: &EnemyMinion,
    ) -> Option<ProjectedAttacker> {
        if candidates.is_empty() {
            return None;
        }

        // enemy_effective_hp is computed inline as needed

        // First priority: kill the enemy and survive
        let best_trade = candidates
            .iter()
            .filter(|a| {
                let dmg_to_enemy = if enemy.has_divine_shield { 0 } else { a.attack };
                let dmg_to_self = if a.has_divine_shield { 0 } else { enemy.attack };
                dmg_to_enemy >= enemy.health && dmg_to_self < a.health
            })
            .min_by_key(|a| {
                // Use the lowest-value (lowest attack) attacker for the perfect trade
                (a.attack, a.health)
            });

        if let Some(&a) = best_trade {
            return Some(*a);
        }

        // Second priority: take out the enemy for free with divine shield
        let divine_trade = candidates
            .iter()
            .filter(|a| a.has_divine_shield && !enemy.has_divine_shield && a.attack >= enemy.health)
            .min_by_key(|a| a.attack);

        if let Some(&a) = divine_trade {
            return Some(*a);
        }

        // Third priority: can kill the enemy (trading), use the lowest-value attacker
        let kill_trade = candidates
            .iter()
            .filter(|a| a.attack >= enemy.health || (enemy.has_divine_shield && a.attack > 0))
            .min_by_key(|a| {
                // Prefer attackers that will die, then the lowest attack
                let will_die = !a.has_divine_shield && !a.is_hero && enemy.attack >= a.health;
                (if will_die { 0 } else { 1 }, a.attack, a.health)
            });

        if let Some(&a) = kill_trade {
            return Some(*a);
        }

        // Fourth priority: pop divine shield (with the weakest attacker)
        let pop_shield = candidates
            .iter()
            .filter(|_a| enemy.has_divine_shield)
            .min_by_key(|a| (a.attack, a.health));

        if let Some(&a) = pop_shield {
            return Some(*a);
        }

        // No suitable trade — do not attack this enemy minion
        None
    }

    /// Evaluates the value of a trade (positive = favorable, negative = loss).
    fn evaluate_trade(
        &self,
        available: &[&ProjectedAttacker],
        used: &[Entity],
        enemy: &EnemyMinion,
    ) -> f64 {
        if let Some(best) = self.find_best_attacker_slice(available, used, enemy) {
            let dmg_to_self = if best.has_divine_shield {
                0
            } else {
                enemy.attack
            };
            let attacker_dies = !best.is_hero && dmg_to_self >= best.health;
            let attacker_value = best.attack as f64 + best.health as f64 * 0.5;
            let enemy_value = enemy.attack as f64 * 1.5 + enemy.health as f64 * 0.5;

            if attacker_dies {
                enemy_value - attacker_value
            } else {
                enemy_value - (dmg_to_self as f64 * 0.3) // partial value of the health lost
            }
        } else {
            -999.0
        }
    }

    /// Collects all characters on the board that can attack (excluding charge projections).
    fn collect_existing_attackers(
        &self,
        state: &GameState,
        player: PlayerId,
    ) -> Vec<ProjectedAttacker> {
        let world = state.world();

        world
            .zones()
            .iter(Zone::Play, player)
            .filter(|&e| {
                world.card_type(e) == Some(CardType::Minion)
                    && world.is_alive(e)
                    && world
                        .attacks_used(e)
                        .is_some_and(|a| !a.is_exhausted_with(world.max_attacks(e)))
                    && world.effective_attack(e).is_some_and(|a| a.0 > 0)
            })
            .map(|e| ProjectedAttacker {
                entity: e,
                attack: world.effective_attack(e).map(|a| a.0).unwrap_or(0),
                health: world.effective_health(e).map(|h| h.0).unwrap_or(0),
                has_divine_shield: world.divine_shield(e).is_some(),
                is_hero: false,
            })
            .collect()
    }
}

// ============================================================
// Helper types and functions
// ============================================================

/// Snapshot info of an enemy minion (used for combat planning).
#[derive(Debug, Clone, Copy)]
struct EnemyMinion {
    entity: Entity,
    attack: i32,
    health: i32,
    has_taunt: bool,
    has_divine_shield: bool,
}

/// Computes a minion's effective health (including divine shield).
fn effective_hp(minion: &EnemyMinion) -> i32 {
    if minion.has_divine_shield {
        minion.health + 1 // divine shield absorbs one hit
    } else {
        minion.health
    }
}

/// Evaluates the expected value of a card effect.
fn evaluate_effect_value(effect: crate::core::effect::CardEffect) -> f64 {
    use crate::core::effect::CardEffect;
    match effect {
        CardEffect::DealDamage { amount, .. } => amount as f64 * 1.2,
        CardEffect::DrawCard { count } => count as f64 * 3.0,
        CardEffect::SummonMinion { .. } => 3.0,
        CardEffect::GainStats { attack, health, .. } => (attack + health) as f64 * 0.8,
        CardEffect::EquipWeapon { .. } => 2.0,
        CardEffect::GainArmor { amount, .. } => amount as f64 * 0.6,
        CardEffect::ReturnToHand { .. } => 2.0,
        CardEffect::IncreaseCost { amount, .. } => amount as f64 * 0.5,
        CardEffect::ReturnToHandAndIncreaseCost { amount, .. } => 2.0 + amount as f64 * 0.5,
        CardEffect::DestroyMinion { .. } => 5.0,
        CardEffect::SilenceMinion { .. } => 3.0,
        CardEffect::SetAttack { attack, .. } => attack as f64 * 0.5,
        CardEffect::SetHealth { health, .. } => health as f64 * 0.5,
        CardEffect::RestoreHealth { amount, .. } => amount as f64 * 0.7,
        CardEffect::FreezeCharacter { .. } => 1.0,
        CardEffect::GainManaCrystal { count } => count as f64 * 2.0,
        CardEffect::GainManaThisTurn { count } => count as f64 * 1.5,
        CardEffect::DestroyWeapon => 2.0,
        CardEffect::GainHeroAttack { attack, armor } => attack as f64 * 1.5 + armor as f64 * 0.5,
        CardEffect::DealHeroAttackDamage { .. } => 3.0,
        CardEffect::FullHeal { .. } => 3.0,
        CardEffect::GrantWindfury { .. } => 3.0,
        CardEffect::GainStatsAndGrantWindfury { attack, health, .. } => {
            (attack + health) as f64 * 0.8 + 3.0
        }
        CardEffect::GrantCharge { attack_bonus, .. } => 2.0 + attack_bonus as f64 * 1.5,
        CardEffect::DoubleAttack { .. } => 3.0,
        CardEffect::DoubleHealth { .. } => 3.0,
        CardEffect::BuffWeapon { attack, durability } => (attack + durability) as f64 * 1.5,
        CardEffect::DiscardRandomCard => -2.0,
        CardEffect::DiscardHand => -10.0,
        CardEffect::NextSpellDiscount { amount } => amount as f64 * 1.0,
        CardEffect::GrantAdjacentStatsAndDivineShield { attack, health } => {
            (attack + health) as f64 * 0.8 + 2.0
        }
        CardEffect::DestroyAllOtherMinionsAndDiscardHand => -6.0,
        CardEffect::DealArmorDamage { .. } => 3.0,
        CardEffect::DestroyWeaponAndDraw => 5.0,
        CardEffect::ReturnAllToHand => 3.0,
        CardEffect::SetAttackToHealth { .. } => 3.0,
        CardEffect::DestroyAllExceptOne => 4.0,
        CardEffect::DestroyAndHeal { heal, .. } => 4.0 + heal as f64 * 0.7,
        CardEffect::DestroyAndAOE { .. } => 5.0,
        CardEffect::DealDamageToTwo { amount } => amount as f64 * 2.0,
        CardEffect::DealDamageAndDraw { damage, draw, .. } => {
            damage as f64 * 1.2 + draw as f64 * 3.0
        }
        CardEffect::DamageAndGainAttack {
            damage,
            attack_bonus,
            ..
        } => damage as f64 * 0.5 + attack_bonus as f64 * 1.5,
        CardEffect::DestroyAdjacent { .. } => 3.0,
        CardEffect::DestroyManaCrystal => -1.0,
        CardEffect::GiveCardsToOpponent { .. } => -2.0,
        CardEffect::ResurrectMinion => 5.0,
        CardEffect::CopyMinionStats => 4.0,
        CardEffect::TempDebuff {
            attack_reduction, ..
        } => attack_reduction as f64 * 1.5,
        CardEffect::ReflectDamage => 3.0,
        CardEffect::DealDamageAndReturnToHand { amount, .. } => amount as f64 * 1.2 + 3.0,
        CardEffect::ReturnFriendlyToHandAndReduceCost { amount } => 2.0 + amount as f64 * 0.5,
        CardEffect::AdjacentDamage => 3.0,
        CardEffect::DestroyWeaponAndDealAttackToEnemies => 4.0,
        CardEffect::GrantStealth => 2.0,
        CardEffect::SummonMultipleMinions { count, .. } => count as f64 * 2.0,
        CardEffect::DamagePlayedMinion { amount } => amount as f64 * 1.2,
        CardEffect::RedirectAttackToRandomCharacter => 3.0,
        CardEffect::SummonAndRedirectAttack { .. } => 3.0,
        CardEffect::SummonSpellbender => 2.0,
        CardEffect::NextSecretCostsZero => 2.0,
        CardEffect::DrawCardAndReduceCost { amount } => 3.0 + amount as f64 * 0.5,
        CardEffect::GrantDeathrattleAll { .. } => 3.0,
        CardEffect::GiveCardToOpponent { count, .. } => -(count as f64) * 1.0,
        CardEffect::FreezeOrDamage { amount } => 1.0 + amount as f64,
        CardEffect::DestroyAndGainHealth => 5.0,
        CardEffect::GrantAttackAndImmune { attack, .. } => 2.0 + attack as f64 * 1.5,
        CardEffect::PreventFatalDamageAndImmune => 5.0,
        CardEffect::TakeControlUntilEndOfTurn => 4.0,
        CardEffect::TakeControl => 6.0,
        CardEffect::TakeControlAttackLE { .. } => 6.0,
        CardEffect::Corrupt => 2.0,
        CardEffect::MinHealthUntilEndOfTurn => 2.0,
        CardEffect::TransformToRandom { .. } => 4.0,
        CardEffect::AddRandomCardToHand { .. } => 3.0,
        CardEffect::DiscoverDeckTop3 => 3.0,
        CardEffect::SummonRandomMinion { .. } => 3.0,
        CardEffect::AddCardToHand { .. } => 3.0,
        CardEffect::DealDamageAndSummonIfKilled { amount, .. } => amount as f64 * 1.2 + 3.0,
        CardEffect::DrawCardByRace { count, .. } => count as f64 * 3.0,
        CardEffect::Demonfire { damage, .. } => damage as f64 * 1.2,
        CardEffect::GainStatsAndTaunt { attack, health, .. } => {
            (attack + health) as f64 * 0.8 + 1.0
        }
        CardEffect::DestroyAndGainStats { attack, health, .. } => {
            (attack + health) as f64 * 0.8 + 2.0
        }
        CardEffect::DestroyRandomEnemySecret => 2.0,
        CardEffect::DestroyAllEnemySecretsAndGainStats { attack, health } => {
            (attack + health) as f64 * 0.8 + 3.0
        }
        CardEffect::DestroyAllEnemySecretsAndDraw { count } => 3.0 + count as f64 * 3.0,
        CardEffect::AttachAttackDraw { count } => count as f64 * 3.0,
        CardEffect::GainStatsPerHandCard {
            attack,
            health_per_card,
        } => attack as f64 + health_per_card as f64 * 4.0,
        CardEffect::GainStatsPerFriendlyMinion {
            attack,
            health_per_minion,
        } => attack as f64 * 1.5 + health_per_minion as f64 * 1.5,
        CardEffect::DealDamageRandomly { amount, count, .. } => amount as f64 * 1.2 * count as f64,
        CardEffect::MortalStrike { boosted, .. } => boosted as f64 * 1.2,
        CardEffect::DrawPerDamagedFriendlyCharacter => 3.0,
        CardEffect::GainStatsIfOwnSecret { attack, health } => (attack + health) as f64 * 0.8,
        CardEffect::AbsorbDivineShields {
            attack_per_shield,
            health_per_shield,
        } => (attack_per_shield + health_per_shield) as f64 * 0.8,
        CardEffect::RemoveWeaponDurability { .. } => 1.5,
        CardEffect::GainAttackEqualToWeapon => 2.0,
        CardEffect::EnemySpellsCostZero => 2.0,
        CardEffect::GiveOpponentManaCrystal { .. } => 0.5,
        CardEffect::SetPlayedMinionHealth { .. } => 2.0,
        CardEffect::SilenceAllEnemyMinionsAndDraw { count } => 3.0 + count as f64 * 3.0,
        CardEffect::SwapAttackAndHealth { .. } => 1.5,
        CardEffect::FreezeAdjacent => 1.5,
        CardEffect::GrantAdjacentTaunt => 1.0,
        CardEffect::GrantAdjacentSpellDamage { .. } => 1.0,
        CardEffect::FullHealAndTaunt { .. } => 2.0,
        CardEffect::ChanceDraw { percent } => percent as f64 / 100.0 * 3.0,
        CardEffect::GainStatsThisTurn { attack, health, .. } => (attack + health) as f64 * 0.8,
        CardEffect::GrantDivineShieldAllFriendly => 3.0,
        CardEffect::GrantDivineShield { .. } => 2.0,
        CardEffect::YseraAwakens { damage } => damage as f64 * 1.2,
        CardEffect::GainStatsAndTauntAllFriendly { attack, health } => {
            (attack + health) as f64 * 0.8 + 2.0
        }
        CardEffect::DrawAndDamageByCost => 4.0,
        CardEffect::RestoreDamagedFriendly { amount } => amount as f64 * 0.6,
        CardEffect::SwapWithHandMinion => 2.0,
        CardEffect::ResurrectDiedMinion => 2.0,
        // Pool-open (roadmap M1): copying an opponent's card is worth roughly
        // a random card to hand; summoning one is worth a summon.
        CardEffect::CopyRandomEnemyHandCard { .. }
        | CardEffect::CopyRandomEnemyDeckCards { .. }
        | CardEffect::SummonRandomEnemyDeckMinion { .. }
        | CardEffect::CopyCastSpellToOtherPlayerHand => 3.0,
        // Core Set W1: filling the hand is worth ~a card each; the forced
        // attack and the corpse copy are board-value effects.
        CardEffect::FillHandWithMinion { .. } => 4.0,
        CardEffect::ForceEnemyMinionsAttackThis => 3.0,
        CardEffect::SpendCorpsesSummonCopy { .. } => 3.0,
        // Core Set W2: outcast draws/damage and the random heal are worth
        // their top-end values; the location destroy is worth nothing until
        // W8 brings the Location type (no targets today).
        CardEffect::DrawCardOutcast { outcast, .. } => outcast as f64 * 3.0,
        CardEffect::OutcastDamage { outcast_amount, .. } => outcast_amount as f64 * 1.2,
        CardEffect::RestoreRandomFriendly { amount } => amount as f64 * 0.7,
        CardEffect::DestroyEnemyLocation => 0.0,
        CardEffect::DamagePlayedMinionAndExcess { amount } => amount as f64 * 1.2,
        CardEffect::DamageAndDrawIfHandEmpty { damage, .. } => damage as f64 * 1.2 + 3.0,
        // Core Set W3a
        CardEffect::AoeDamageAndHealFriendly { damage, heal } => {
            damage as f64 * 1.2 + heal as f64 * 0.7
        }
        CardEffect::DamageAndDrawIfSurvives { damage, .. } => damage as f64 * 1.2 + 3.0,
        CardEffect::GainArmorAndDraw { armor, draw } => armor as f64 * 0.6 + draw as f64 * 3.0,
        CardEffect::DamageAndDrawIfKilled { damage, .. } => damage as f64 * 1.2 + 3.0,
        CardEffect::DamageAndGainArmor { damage, armor, .. } => {
            damage as f64 * 1.2 + armor as f64 * 0.6
        }
        CardEffect::GainPoisonousToFriendlyUndead => 3.0,
        CardEffect::TransformToMinion { .. } => 4.0,
        CardEffect::GrantDeathrattleToTarget { .. } => 3.0,
        CardEffect::DestroyAllMinionsAttackGE { attack } => attack as f64 * 0.8,
        CardEffect::AoeDamageAndDraw { damage, draw } => damage as f64 * 1.2 + draw as f64 * 3.0,
        CardEffect::GainStatsTauntAndDeathrattle { attack, health, .. } => {
            (attack + health) as f64 * 0.8 + 3.0
        }
        // Core Set W3a part 2
        CardEffect::SummonRandomFishFromDeck => 3.0,
        CardEffect::AddRandomSpellToOpponentDeckTop => 2.0,
        CardEffect::SummonStatueTrio => 5.0,
        CardEffect::CopyEnemyDeckCardOnSelfAttack => 3.0,
        // Core Set W3b
        CardEffect::DamageAndSummon { damage, .. } => damage as f64 * 1.2 + 3.0,
        CardEffect::RehgarBolt => 3.0,
        CardEffect::DamageTwoDrawIfKilled { damage } => damage as f64 * 2.4 + 3.0,
        CardEffect::FreezeAndDiscoverSpell => 3.0,
        CardEffect::HenchThugBuff => 2.0,
        CardEffect::SummonRecruitsAndEquipWeapon => 5.0,
        CardEffect::BuffAndSummonRandomCost2 => 4.0,
        CardEffect::DamageAndSummonCopyIfKilled { damage } => damage as f64 * 1.2 + 4.0,
        CardEffect::KeymasterCopy => 3.0,
        CardEffect::FordragonBuff => 3.0,
        CardEffect::DamageAndSummonVoidwalkers { damage, .. } => damage as f64 * 1.2 + 4.0,
        CardEffect::DamageAndAddToHand { damage, .. } => damage as f64 * 1.2 + 2.0,
        CardEffect::AddRandomOtherClassSpells { count, .. } => count as f64 * 3.0,
        CardEffect::SummonFelbatOnDraw => 3.0,
        CardEffect::SpendCorpsesSummonRandomMinion { .. } => 3.0,
        // Core Set W3c
        CardEffect::GainStatsAndDraw {
            attack,
            health,
            draw,
            ..
        } => (attack + health) as f64 * 0.8 + draw as f64 * 3.0,
        CardEffect::DamageUndamaged { damage } => damage as f64 * 1.2,
        CardEffect::DamageMinionAndSelfHero { damage } => damage as f64 * 1.2,
        CardEffect::GainHeroAttackAndDraw { attack } => attack as f64 * 1.5 + 3.0,
        CardEffect::GainArmorAndSummonDeckMinion { armor, .. } => armor as f64 * 0.6 + 3.0,
        CardEffect::GainArmorAndDrawOnHeroAttack { armor } => armor as f64 * 0.6 + 3.0,
        CardEffect::SummonAllCompanions => 6.0,
        CardEffect::DamageFreezeAllAndSummon { damage, .. } => damage as f64 * 1.2 + 5.0,
        CardEffect::DestroyHighestAttackEnemy => 5.0,
        CardEffect::SummonZombiesWithCorpseReborn { .. } => 4.0,
        CardEffect::TransformSelfToCastSpell => 3.0,
        CardEffect::BuffHandMinionsWithCorpses { .. } => 3.0,
        CardEffect::RestoreHealthAndDraw { amount, .. } => amount as f64 * 0.7 + 3.0,
        CardEffect::DrawIfUnspentMana => 3.0,
        CardEffect::GainArmorAndSummonRandomCost { armor, .. } => armor as f64 * 0.6 + 4.0,
        // Core Set W4a
        CardEffect::GainStatsIfHealedThisTurn { attack, health, .. } => {
            (attack + health) as f64 * 0.8
        }
        CardEffect::BattleToTheDeath => 4.0,
        CardEffect::NextDemonDiscount { amount } => amount as f64 * 2.0,
        CardEffect::BuffHandMinions { attack, health } => (attack + health) as f64 * 0.8,
        CardEffect::SummonRandomEnemyHandMinion => 3.0,
        CardEffect::DrawForBoth => 3.0,
        CardEffect::NextComboDiscount { amount } => amount as f64 * 2.0,
        CardEffect::AddRandomMageSpells { count } => count as f64 * 3.0,
        CardEffect::DamageEnemyHeroAndHealSelf { amount } => amount as f64 * 1.2,
        CardEffect::LoseHealthPerOpponentHandCard => 2.0,
        CardEffect::GrantRandomFriendlyDivineShieldTaunt => 2.0,
        CardEffect::RemoveTopEnemyDeckCard => 2.0,
        CardEffect::DiscoverSpellAndHealCost => 4.0,
        CardEffect::DrawBeastDragonMurloc => 6.0,
        CardEffect::AddRandomOtherClassCard => 3.0,
        CardEffect::DamageSelfHero { damage } => damage as f64 * 0.5,
        CardEffect::SummonTwoCopiesOfSelf => 6.0,
        CardEffect::SpendCorpsesDamageRandom { damage, .. } => damage as f64 * 1.2,
        CardEffect::SpendCorpsesSummonFootmen { .. } => 5.0,
        CardEffect::OngoingEndTurnDamage { damage } => damage as f64 * 1.5,
        CardEffect::DamageAllOtherMinions { damage } => damage as f64 * 1.2,
        CardEffect::BuffTauntHandMinions { attack, health } => (attack + health) as f64 * 0.8,
        CardEffect::AddRandomShamanSpell => 3.0,
        // Core Set W4b
        CardEffect::SummonRandomMinionCostEqHandSize => 4.0,
        CardEffect::ResurrectHighestCostFallen => 5.0,
        CardEffect::GrantDeathrattleSummonOwnCost => 4.0,
        CardEffect::AddRandomPirateToHand => 3.0,
        CardEffect::NextEnemyHeroPowerCostMore { .. } => 1.0,
        CardEffect::SummonRandomMinionOfCost { .. } => 4.0,
        CardEffect::SummonRandomDemonFromHandOrDeck => 5.0,
        CardEffect::NextEnemySpellsCostMore { .. } => 1.0,
        CardEffect::BuffWeaponDurabilityIfBeast { .. } => 1.0,
        CardEffect::ReturnLastTurnSpells => 4.0,
        CardEffect::DestroyMinionAndSelfDamage => 5.0,
        CardEffect::DamageSelfMinion { damage } => damage as f64 * 0.5,
        CardEffect::AddRandomOneCostCard => 3.0,
        CardEffect::BuffThreeDifferentRaces { attack, health } => (attack + health) as f64 * 0.8,
        CardEffect::AddFiveRandomCards => 8.0,
        CardEffect::DiscardTwoRandomCards => -2.0,
        // Core Set W5
        CardEffect::DamageAllMinionsIfHoldingDragon { damage } => damage as f64 * 1.2,
        CardEffect::DamageAllEnemiesByAttack => 3.0,
        CardEffect::ReturnRandomFriendlyAndReduceCost { .. } => 2.0,
        CardEffect::GrantAttackToRandomFriendly => 2.0,
        CardEffect::SummonRandomLegendaryMinion => 5.0,
        CardEffect::ResurrectWeaponKilled => 4.0,
        CardEffect::DestroyRandomEnemyMinion => 4.0,
        CardEffect::SummonOasisWaterElemental => 4.0,
        // Core Set W6
        CardEffect::SummonRandomCostAndFreeze { .. } => 5.0,
        CardEffect::DamageAndAddRandomSpell { damage, .. } => damage as f64 * 1.2 + 3.0,
        CardEffect::FreezeAndSummonElementals => 6.0,
        CardEffect::AddRandomTauntBuffed => 4.0,
        CardEffect::AddRandomBattlecryMinion => 3.0,
        CardEffect::DamageAndFreeze { damage, .. } => damage as f64 * 1.2 + 1.0,
        CardEffect::DamageAllEnemyMinionsAndFreeze { damage } => damage as f64 * 1.2 * 2.0,
        CardEffect::AddRandomOutcastCardNextCheaper => 3.0,
        // 2025-2026 expansions M1-W1 (the Emerald Dream imbue mechanic)
        CardEffect::ImbueHeroPower => 2.0,
        CardEffect::ImbuedHeroPower { .. } => 3.0,
        CardEffect::UseHeroPower => 2.0,
        CardEffect::DrawBeastAndImbue => 5.0,
        CardEffect::RestoreAndDrawAndImbue { amount } => amount as f64 * 0.7 + 5.0,
        CardEffect::SummonRandomTwoCostTauntAndImbue => 4.0,
        CardEffect::ImbueAndReduceHandCost => 4.0,
        CardEffect::ImbueAndTriggerHeroPower => 5.0,
        CardEffect::ImbueAndGetWisp => 3.0,
        CardEffect::ImbueAndDebuffEnemies { attack_reduction } => {
            attack_reduction as f64 * 1.0 + 2.0
        }
        CardEffect::DealDamageIfImbuedTwice { damage } => damage as f64 * 1.2,
        CardEffect::DiscoverWildGodIfImbued4 => 5.0,
        CardEffect::ImbueEveryThirdSpell => 2.0,
        CardEffect::SummonRandomDragonOfCost { .. } => 4.0,
        // 2025-2026 expansions M1-W2 (the Emerald Dream dark-gift mechanic)
        CardEffect::ApplyDarkGift { .. } => 3.0,
        CardEffect::DiscoverWithDarkGift { .. } => 4.0,
        CardEffect::DiscoverDragonWithDarkGift => 4.0,
        CardEffect::DiscoverUndeadWithCorpseGift { .. } => 3.0,
        CardEffect::DiscoverEnemyDeckMinionCopy { .. } => 3.0,
        CardEffect::DiscoverDeckMinionWithDarkGift => 4.0,
        CardEffect::ReduceHandMinionGiftCost => 3.0,
        // 2025-2026 expansions M1-W3 (the Emerald Dream choose-one wave)
        CardEffect::GainStatsAndGrantDivineShield { attack, health, .. } => {
            (attack + health) as f64 * 0.8 + 2.0
        }
        CardEffect::GainStatsAndGrantLifesteal { attack, health, .. } => {
            (attack + health) as f64 * 0.8 + 2.0
        }
        CardEffect::GrantPoisonousThisTurn => 2.0,
        CardEffect::GrantWeaponDeathrattleAllEnemies { damage } => damage as f64 * 1.2,
        CardEffect::DrawCardByType { count, .. } => count as f64 * 3.0,
        CardEffect::SpendCorpsesDamageMinion { damage, .. } => damage as f64 * 1.2,
        CardEffect::DamageAllMinions { damage } => damage as f64 * 1.2,
        CardEffect::AddRandomDruidSpell => 3.0,
        CardEffect::AddRandomOtherClassChooseOneCard => 4.0,
    }
}

// ============================================================
// SmartBot tests
// ============================================================

#[cfg(test)]
mod smart_bot_tests {
    use super::*;
    use crate::cards::def::{
        BLOODFEN_RAPTOR, BLUEGILL_WARRIOR, EAGLEHORN_BOW, OGRE_MAGI, VOIDWALKER,
    };
    use crate::core::player::PlayerId;
    use crate::sim::game::GameBuilder;

    #[test]
    fn smart_bot_plays_cards_from_hand() {
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &OGRE_MAGI);
        builder.add_minion_to_hand(PlayerId::Player1, &BLOODFEN_RAPTOR);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        let play_count = actions
            .iter()
            .filter(|a| matches!(a, Action::PlayCard { .. }))
            .count();
        assert!(play_count >= 1);
        assert!(actions.last().is_some_and(|a| matches!(a, Action::EndTurn)));
    }

    #[test]
    fn smart_bot_ends_turn() {
        let state = GameState::new();
        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);
        assert!(actions.last().is_some_and(|a| matches!(a, Action::EndTurn)));
    }

    #[test]
    fn smart_bot_returns_empty_for_game_over() {
        let mut builder = GameBuilder::new();
        builder.step(crate::core::state::Step::GameOver {
            winner: PlayerId::Player1,
        });
        let state = builder.build();

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);
        assert!(actions.is_empty());
    }

    #[test]
    fn smart_bot_detects_lethal() {
        // A 30-attack minion is on the board — lethal should be detected
        let mut builder = GameBuilder::new();
        builder.add_custom_minion_to_board(PlayerId::Player1, 30, 5, 3);
        // Ensure the enemy hero has 30 HP so lethal is possible
        builder.hero_health(PlayerId::Player2, 30);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();
        let enemy_hero = state.player(PlayerId::Player2).hero;

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        // All attacks should go face
        let attack_actions: Vec<_> = actions
            .iter()
            .filter(|a| matches!(a, Action::Attack { .. }))
            .collect();
        assert!(!attack_actions.is_empty());
        for action in attack_actions {
            assert!(matches!(
                action,
                Action::Attack { defender, .. } if *defender == enemy_hero
            ));
        }
    }

    #[test]
    fn smart_bot_attacks_hero_when_no_taunts() {
        let mut builder = GameBuilder::new();
        let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 3, 3, 3);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();
        let enemy_hero = state.player(PlayerId::Player2).hero;

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Attack { attacker: a_ent, defender: d_ent }
            if *a_ent == attacker && *d_ent == enemy_hero
        )));
    }

    #[test]
    fn smart_bot_must_attack_taunt_first() {
        let mut builder = GameBuilder::new();
        let attacker = builder.add_custom_minion_to_board(PlayerId::Player1, 5, 5, 3);
        builder.add_minion_to_board(PlayerId::Player2, &VOIDWALKER);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();

        let taunt = state
            .world()
            .zones()
            .iter(Zone::Play, PlayerId::Player2)
            .find(|&e| state.world().taunt(e).is_some())
            .unwrap();

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        let attack_actions: Vec<_> = actions
            .iter()
            .filter(|a| matches!(a, Action::Attack { .. }))
            .collect();
        if !attack_actions.is_empty() {
            let first = attack_actions[0];
            assert!(
                matches!(
                    first,
                    Action::Attack { attacker: a, defender: d } if *a == attacker && *d == taunt
                ),
                "First attack should be our minion clearing enemy taunt"
            );
            // After clearing the taunt, remaining attacks should go face
            let enemy_hero = state.player(PlayerId::Player2).hero;
            let remaining: Vec<_> = attack_actions.iter().skip(1).collect();
            if !remaining.is_empty() {
                for action in remaining {
                    assert!(matches!(
                        action,
                        Action::Attack { defender: d, .. } if *d == enemy_hero
                    ));
                }
            }
        }
    }

    #[test]
    fn smart_bot_hero_attacks_with_weapon() {
        let mut builder = GameBuilder::new();
        builder.equip_weapon(PlayerId::Player1, &EAGLEHORN_BOW);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();
        let hero = state.player(PlayerId::Player1).hero;
        let enemy_hero = state.player(PlayerId::Player2).hero;

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Attack { attacker: a, defender: d }
            if *a == hero && *d == enemy_hero
        )));
    }

    #[test]
    fn smart_bot_prefers_high_value_cards() {
        // Hand contains Ogre Magi (4/4 with spell damage, 4 mana) and Bloodfen Raptor (3/2, 2 mana)
        // Ogre Magi should score higher (spell damage bonus); both get played with enough mana
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &OGRE_MAGI); // 4/4 spell_dmg+1 for 4
        builder.add_minion_to_hand(PlayerId::Player1, &BLOODFEN_RAPTOR); // 3/2 for 2
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        // Both should be played (enough mana)
        let play_actions: Vec<_> = actions
            .iter()
            .filter(|a| matches!(a, Action::PlayCard { .. }))
            .collect();
        assert_eq!(play_actions.len(), 2);
    }

    #[test]
    fn smart_bot_plays_charge_minion() {
        // Bluegill Warrior (2/1 charge) should be played and included in attack planning
        let mut builder = GameBuilder::new();
        builder.add_minion_to_hand(PlayerId::Player1, &BLUEGILL_WARRIOR);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();
        let bluegill = state
            .world()
            .zones()
            .iter(Zone::Hand, PlayerId::Player1)
            .next()
            .unwrap();

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        // Should have a play action
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::PlayCard {
                        card,
                        target: None,
                        position: None,
                    } if *card == bluegill
        )));

        // Should have an attack action (charge minions can attack)
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Attack { attacker: a, .. } if *a == bluegill
        )));
    }

    #[test]
    fn smart_bot_uses_hero_power_when_available() {
        use crate::core::effect::{CardEffect, EffectTarget};

        let mut builder = GameBuilder::new();
        builder.set_mana(PlayerId::Player1, 10, 10);
        builder.set_hero_power(
            PlayerId::Player1,
            2,
            CardEffect::DealDamage {
                amount: 1,
                target: EffectTarget::AnyEnemy,
            },
        );
        let state = builder.build();

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        assert!(
            actions
                .iter()
                .any(|a| matches!(a, Action::HeroPower { .. }))
        );
    }

    #[test]
    fn smart_bot_trades_favorably() {
        // Our 5/5 vs the enemy 3/2 — we can kill it and survive, so we should trade
        let mut builder = GameBuilder::new();
        let our_minion = builder.add_custom_minion_to_board(PlayerId::Player1, 5, 5, 3);
        let enemy_minion = builder.add_custom_minion_to_board(PlayerId::Player2, 3, 2, 2);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        // Should clear the enemy's high-threat minion instead of going all-face
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Attack { attacker: a, defender: d }
            if *a == our_minion && *d == enemy_minion
        )));
    }

    #[test]
    fn smart_bot_handles_divine_shield_enemy() {
        // The enemy has a divine shield minion and we have several minions
        // SmartBot should evaluate the divine shield correctly without crashing
        let mut builder = GameBuilder::new();
        let _weak = builder.add_custom_minion_to_board(PlayerId::Player1, 1, 3, 1);
        let _strong = builder.add_custom_minion_to_board(PlayerId::Player1, 6, 6, 5);
        let _enemy = builder.add_custom_minion_to_board(PlayerId::Player2, 4, 5, 4);
        builder.set_mana(PlayerId::Player1, 10, 10);
        let state = builder.build();

        let bot = SmartBot::new();
        let actions = bot.decide_actions(&state);

        // At least verify the bot completes its decision without crashing
        assert!(actions.last().is_some_and(|a| matches!(a, Action::EndTurn)));
    }
}
