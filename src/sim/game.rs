//! GameBuilder — 用于测试的灵活对局构建器。
//!
//! `GameBuilder` 允许绕过规则验证直接设置游戏状态，
//! 是单元测试和集成测试的核心工具。
//! 未来 Phase 的 RL 环境也会使用它来重置对局。

use crate::cards::def::CardDef;
use crate::core::component::{
    Attack, AttacksUsed, Aura, CardType, Cost, Durability, Health, HeroPowerDef, Secret,
};
use crate::core::entity::Entity;
use crate::core::player::PlayerId;
use crate::core::state::{GameState, Phase};
use crate::core::zone::Zone;

/// 对局构建器 — 用于创建自定义游戏状态。
///
/// 所有方法返回 `&mut Self`，支持链式调用。
///
/// # 示例
///
/// ```rust
/// use orange_stone::sim::game::GameBuilder;
/// use orange_stone::cards::def::{OGRE_MAGI, ARCHMAGE};
/// use orange_stone::core::player::PlayerId;
///
/// let mut builder = GameBuilder::new();
/// builder.add_minion_to_hand(PlayerId::Player1, &OGRE_MAGI);
/// builder.add_minion_to_board(PlayerId::Player2, &ARCHMAGE);
/// let state = builder.build();
/// ```
#[derive(Debug, Default)]
pub struct GameBuilder {
    state: GameState,
}

impl GameBuilder {
    /// 创建一个新的构建器，初始状态包含两个 30 HP 英雄。
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
        }
    }

    /// 消耗构建器，返回构建好的 `GameState`。
    #[must_use]
    pub fn build(self) -> GameState {
        self.state
    }

    /// 设置当前行动玩家。
    pub fn active_player(&mut self, player: PlayerId) -> &mut Self {
        self.state.set_active_player(player);
        self
    }

    /// 设置当前回合数。
    pub fn turn(&mut self, turn: u32) -> &mut Self {
        self.state.set_turn(turn);
        self
    }

    /// 设置游戏阶段。
    pub fn phase(&mut self, phase: Phase) -> &mut Self {
        self.state.set_phase(phase);
        self
    }

    /// 设置英雄的生命值。
    pub fn hero_health(&mut self, player: PlayerId, hp: i32) -> &mut Self {
        let hero = self.state.player(player).hero;
        let world = self.state.world_mut();
        world.set_health(hero, Health(hp));
        self
    }

    /// 设置玩家的法力水晶。
    pub fn set_mana(&mut self, player: PlayerId, crystals: i32, current: i32) -> &mut Self {
        let inner = self.state.make_mut();
        let p = &mut inner.players[player.index()];
        p.mana_crystals = crystals;
        p.current_mana = current;
        self
    }

    /// 设置 RNG seed（重建 RNG）。
    pub fn with_rng_seed(&mut self, seed: u64) -> &mut Self {
        self.state.make_mut().rng = crate::sim::rng::GameRng::new(seed);
        self
    }

    /// 根据 CardDef 创建一个随从并放入指定玩家的牌库。
    pub fn add_minion_to_deck(&mut self, player: PlayerId, card: &CardDef) -> &mut Self {
        let e = self.spawn_minion(player, card);
        let world = self.state.world_mut();
        world.set_zone(e, Zone::Deck);
        world.zones_mut().insert(Zone::Deck, player, e);
        self
    }

    /// 根据 `CardDef` 创建一个随从并放入指定玩家的手牌。
    pub fn add_minion_to_hand(&mut self, player: PlayerId, card: &CardDef) -> &mut Self {
        let e = self.spawn_minion(player, card);
        let world = self.state.world_mut();
        world.set_zone(e, Zone::Hand);
        world.zones_mut().insert(Zone::Hand, player, e);
        self
    }

    /// 根据 `CardDef` 创建一个随从并放入指定玩家的战场。
    pub fn add_minion_to_board(&mut self, player: PlayerId, card: &CardDef) -> &mut Self {
        let e = self.spawn_minion(player, card);
        let world = self.state.world_mut();
        world.set_zone(e, Zone::Play);
        world.zones_mut().insert(Zone::Play, player, e);
        self
    }

    /// 生成一个随从实体并设置基本组件，放入指定玩家的手牌并返回实体句柄。
    pub fn add_custom_minion_to_hand(
        &mut self,
        player: PlayerId,
        attack: i32,
        health: i32,
        cost: i32,
    ) -> Entity {
        let world = self.state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(health));
        world.set_attack(e, Attack(attack));
        world.set_cost(e, Cost(cost));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, player);
        world.set_attacks_used(e, AttacksUsed(0));
        world.set_zone(e, Zone::Hand);
        world.zones_mut().insert(Zone::Hand, player, e);
        e
    }

    /// 生成一个随从实体并设置基本组件，放入指定玩家的战场并返回实体句柄。
    pub fn add_custom_minion_to_board(
        &mut self,
        player: PlayerId,
        attack: i32,
        health: i32,
        cost: i32,
    ) -> Entity {
        let world = self.state.world_mut();
        let e = world.spawn();
        world.set_health(e, Health(health));
        world.set_attack(e, Attack(attack));
        world.set_cost(e, Cost(cost));
        world.set_card_type(e, CardType::Minion);
        world.set_player(e, player);
        world.set_attacks_used(e, AttacksUsed(0));
        world.set_zone(e, Zone::Play);
        world.zones_mut().insert(Zone::Play, player, e);
        e
    }

    /// 给英雄装备武器。
    pub fn equip_weapon(&mut self, player: PlayerId, card: &CardDef) -> &mut Self {
        let inner = self.state.make_mut();
        let world = &mut inner.world;
        let weapon = world.spawn();
        world.set_attack(weapon, Attack(card.attack));
        world.set_durability(weapon, Durability(card.durability));
        world.set_cost(weapon, Cost(card.cost));
        world.set_card_type(weapon, CardType::Weapon);
        world.set_player(weapon, player);
        world.set_zone(weapon, Zone::Play);
        world.zones_mut().insert(Zone::Play, player, weapon);
        inner.players[player.index()].weapon = Some(weapon);
        self
    }

    /// 设置英雄护甲。
    pub fn hero_armor(&mut self, player: PlayerId, armor: i32) -> &mut Self {
        let inner = self.state.make_mut();
        inner.players[player.index()].armor = armor;
        self
    }

    /// 给英雄设置英雄技能。
    pub fn set_hero_power(
        &mut self,
        player: PlayerId,
        cost: i32,
        effect: crate::core::effect::CardEffect,
    ) -> &mut Self {
        let hero = self.state.player(player).hero;
        let world = self.state.world_mut();
        world.set_hero_power(hero, HeroPowerDef { cost, effect });
        self
    }

    /// 给随从设置光环效果。
    pub fn set_aura_on_entity(&mut self, entity: Entity, aura: Aura) -> &mut Self {
        self.state.world_mut().set_aura(entity, aura);
        self
    }

    /// 给随从设置奥秘组件。
    pub fn set_secret_on_entity(&mut self, entity: Entity, secret: Secret) -> &mut Self {
        self.state.world_mut().set_secret(entity, secret);
        self
    }

    /// 内部辅助：根据 CardDef 生成一个卡牌实体（不设置 Zone）。
    ///
    /// 组件设置逻辑统一在 `crate::cards::spawn_card_from_def` 中。
    fn spawn_minion(&mut self, player: PlayerId, card: &CardDef) -> Entity {
        crate::cards::spawn_card_from_def(self.state.world_mut(), player, card)
    }
}
