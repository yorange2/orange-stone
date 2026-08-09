//! GameBuilder — a flexible game builder for tests.
//!
//! `GameBuilder` lets you set up game state directly without rule validation,
//! and is a core tool for unit and integration tests.
//! Future-phase RL environments will also use it to reset games.

use crate::cards::def::CardDef;
use crate::core::component::{
    Attack, AttacksUsed, Aura, CardType, Cost, Health, HeroPowerDef, Secret,
};
use crate::core::entity::Entity;
use crate::core::player::PlayerId;
use crate::core::small_list::SmallList;
use crate::core::state::{GameState, Step};
use crate::core::zone::Zone;

/// Game builder — used to create custom game states.
///
/// All methods return `&mut Self` for chained calls.
///
/// # Example
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
    /// Creates a new builder whose initial state has two 30 HP heroes.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
        }
    }

    /// Consumes the builder and returns the built `GameState` (decks shuffled —
    /// roadmap G7 opening).
    #[must_use]
    pub fn build(mut self) -> GameState {
        // M5-W1 — Aya's turn-order override (JAIL_504, "You always go
        // second") resolves FIRST, before the Start-of-Game phase: when
        // only Player1's deck holds Aya the seats flip — Player1 becomes
        // the second player (0/0 mana, the turn-1 crystal moves to
        // Player2), so the SOG phase, the battle-path deal and the step
        // machine all see the final turn order. Both decks holding Aya
        // (or neither) keeps the default order (§27).
        let p1_has_aya = self
            .state
            .world()
            .zones()
            .iter(Zone::Deck, PlayerId::Player1)
            .any(|e| {
                self.state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| c.0 == "JAIL_504")
            });
        let p2_has_aya = self
            .state
            .world()
            .zones()
            .iter(Zone::Deck, PlayerId::Player2)
            .any(|e| {
                self.state
                    .world()
                    .card_id(e)
                    .is_some_and(|c| c.0 == "JAIL_504")
            });
        if crate::cards::start_of_game::aya_flip(p1_has_aya, p2_has_aya) {
            self.state.set_active_player(PlayerId::Player2);
            let inner = self.state.make_mut();
            inner.players[0].mana_crystals = 0;
            inner.players[0].current_mana = 0;
            inner.players[1].mana_crystals = 1;
            inner.players[1].current_mana = 1;
        }
        // M5-W1 — the Start-of-Game phase (the V1 rule-override hook; the
        // EDR §14 Hamuul row's "engine has no StartOfGame event" is now
        // resolved — the EDR card itself stays as-is, annotated in the
        // ledger). Starting-deck cards with registered effects resolve in
        // deck order, per player (Player1 first, then Player2 — the fixed
        // seat order), BEFORE the starting-deck snapshot and the shuffle:
        // the deck mutations (Hogger's duplicates, Azalina's 40-card
        // rebuild, Beatrix's ten copies) land in the pre-shuffle deck. A
        // scratch queue carries each effect's enqueued events; the only
        // lasting surface is the state itself (Aya's pending choice
        // survives — the first action of the game resolves it).
        let sog_triples: SmallList<(PlayerId, Entity, crate::core::effect::CardEffect)> = {
            let mut triples = SmallList::new();
            for player in [PlayerId::Player1, PlayerId::Player2] {
                for e in self.state.world().zones().iter(Zone::Deck, player) {
                    let Some(cid) = self.state.world().card_id(e) else {
                        continue;
                    };
                    if let Some(effect) = crate::cards::start_of_game::start_of_game_effect(cid.0) {
                        triples.push((player, e, effect));
                    }
                }
            }
            triples
        };
        for (player, source, effect) in sog_triples {
            let mut queue = crate::core::event::EventQueue::new();
            crate::engine::trigger::resolve_effect(
                &mut self.state,
                &mut queue,
                source,
                player,
                effect,
                None,
                None,
            );
        }
        // M2-W4a: snapshot each player's starting deck (Story of the
        // Waygate's "didn't start in your deck" set) before the shuffle —
        // every game construction path (battle / batch / env / tests)
        // funnels through here, and the mulligan reshuffle must not
        // overwrite the pre-game contents. The snapshot runs AFTER the
        // Start-of-Game phase, so SOG deck mutations are part of the
        // starting deck.
        for player in [PlayerId::Player1, PlayerId::Player2] {
            let ids: Vec<String> = self
                .state
                .world()
                .zones()
                .iter(Zone::Deck, player)
                .filter_map(|e| self.state.world().card_id(e).map(|c| c.0.to_string()))
                .collect();
            self.state.make_mut().players[player.index()].starting_deck = ids;
        }
        self.state.shuffle_decks();
        self.state
    }

    /// Raw mutable access to the in-progress state (for test setups that need
    /// components without a dedicated builder method).
    pub fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    /// Sets the active player.
    pub fn active_player(&mut self, player: PlayerId) -> &mut Self {
        self.state.set_active_player(player);
        self
    }

    /// Sets the current turn number.
    pub fn turn(&mut self, turn: u32) -> &mut Self {
        self.state.set_turn(turn);
        self
    }

    /// Sets the game phase.
    pub fn step(&mut self, step: Step) -> &mut Self {
        self.state.set_step(step);
        self
    }

    /// Sets the hero's health.
    pub fn hero_health(&mut self, player: PlayerId, hp: i32) -> &mut Self {
        let hero = self.state.player(player).hero;
        let world = self.state.world_mut();
        world.set_health(hero, Health(hp));
        self
    }

    /// Sets the player's mana crystals.
    pub fn set_mana(&mut self, player: PlayerId, crystals: i32, current: i32) -> &mut Self {
        let inner = self.state.make_mut();
        let p = &mut inner.players[player.index()];
        p.mana_crystals = crystals;
        p.current_mana = current;
        self
    }

    /// Sets the RNG seed (rebuilds the RNG).
    pub fn with_rng_seed(&mut self, seed: u64) -> &mut Self {
        self.state.make_mut().rng = crate::sim::rng::GameRng::new(seed);
        self
    }

    /// Creates a minion from a `CardDef` and puts it in the given player's deck.
    pub fn add_minion_to_deck(&mut self, player: PlayerId, card: &CardDef) -> &mut Self {
        let e = self.spawn_minion(player, card);
        let world = self.state.world_mut();
        world.set_zone(e, Zone::Deck);
        world.zones_mut().insert(Zone::Deck, player, e);
        self
    }

    /// Creates a minion from a `CardDef` and puts it in the given player's hand.
    pub fn add_minion_to_hand(&mut self, player: PlayerId, card: &CardDef) -> &mut Self {
        let e = self.spawn_minion(player, card);
        let world = self.state.world_mut();
        world.set_zone(e, Zone::Hand);
        world.zones_mut().insert(Zone::Hand, player, e);
        self
    }

    /// Creates a minion from a `CardDef` and puts it on the given player's board.
    pub fn add_minion_to_board(&mut self, player: PlayerId, card: &CardDef) -> &mut Self {
        let e = self.spawn_minion(player, card);
        let world = self.state.world_mut();
        world.set_zone(e, Zone::Play);
        world.zones_mut().insert(Zone::Play, player, e);
        self
    }

    /// Spawns a minion entity with basic components, puts it in the given player's hand, and returns the entity handle.
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

    /// Spawns a minion entity with basic components, puts it on the given player's board, and returns the entity handle.
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

    /// Equips a weapon to the hero.
    pub fn equip_weapon(&mut self, player: PlayerId, card: &CardDef) -> &mut Self {
        // The full component set comes from `spawn_card_from_def` — weapon
        // triggers (Sword of Justice) must be registered for the test harness
        // too, not only for weapons played from hand.
        let weapon = crate::cards::spawn_card_from_def(self.state.world_mut(), player, card);
        let inner = self.state.make_mut();
        let world = &mut inner.world;
        world.set_zone(weapon, Zone::Play);
        world.zones_mut().insert(Zone::Play, player, weapon);
        inner.players[player.index()].weapon = Some(weapon);
        self
    }

    /// Sets the hero's armor.
    pub fn hero_armor(&mut self, player: PlayerId, armor: i32) -> &mut Self {
        let inner = self.state.make_mut();
        inner.players[player.index()].armor = armor;
        self
    }

    /// Sets the hero power for the hero.
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

    /// Sets an aura effect on an entity.
    pub fn set_aura_on_entity(&mut self, entity: Entity, aura: Aura) -> &mut Self {
        self.state.world_mut().set_aura(entity, aura);
        self
    }

    /// Sets a secret component on an entity.
    pub fn set_secret_on_entity(&mut self, entity: Entity, secret: Secret) -> &mut Self {
        self.state.world_mut().set_secret(entity, secret);
        self
    }

    /// Internal helper: spawns a card entity from a `CardDef` (without setting the zone).
    ///
    /// Component setup is centralized in `crate::cards::spawn_card_from_def`.
    fn spawn_minion(&mut self, player: PlayerId, card: &CardDef) -> Entity {
        crate::cards::spawn_card_from_def(self.state.world_mut(), player, card)
    }
}
