//! PyO3 wrappers for the structured views (roadmap M1-G3).
//!
//! Thin conversion layer: the data mirrors live in `crate::rl::views` (pure
//! Rust, unit-tested); these classes just expose their fields to Python.
//!
//! ```python
//! obs = env.structured_observation()   # turn, my_turn, me/opponent, ...
//! acts = env.structured_legal_actions()  # kind, card_index, entity_id, ...
//! ```

use crate::rl::views as rviews;
use pyo3::prelude::*;

/// A hand card, minion, or other entity as seen from one perspective.
#[pyclass(name = "EntityView", module = "orange_stone")]
#[derive(Clone)]
pub struct PyEntityView {
    /// Entity slot index — matches `ActionView.entity_id` / `target_id`
    #[pyo3(get)]
    pub entity_id: u32,
    /// Card ID (empty for generated entities without a CardId)
    #[pyo3(get)]
    pub card_id: String,
    /// Display name (empty if unknown)
    #[pyo3(get)]
    pub name: String,
    /// Current (effective) cost
    #[pyo3(get)]
    pub cost: i32,
    /// Current (effective) attack
    #[pyo3(get)]
    pub attack: i32,
    /// Current (effective) health
    #[pyo3(get)]
    pub health: i32,
    /// Whether the entity can attack this turn
    #[pyo3(get)]
    pub can_attack: bool,
    /// Taunt keyword
    #[pyo3(get)]
    pub taunt: bool,
    /// Divine Shield keyword
    #[pyo3(get)]
    pub divine_shield: bool,
    /// Stealth keyword
    #[pyo3(get)]
    pub stealth: bool,
    /// Elusive keyword (M5)
    #[pyo3(get)]
    pub elusive: bool,
    /// Windfury keyword
    #[pyo3(get)]
    pub windfury: bool,
    /// Charge keyword (effective — a Charge aura counts)
    #[pyo3(get)]
    pub charge: bool,
    /// Race / tribe: 0=none, 1=Beast, 2=Murloc, 3=Demon (fidelity-debt W1)
    #[pyo3(get)]
    pub race: i32,
    /// Frozen this turn
    #[pyo3(get)]
    pub frozen: bool,
    /// Hand cards only: affordable with the owner's current mana
    #[pyo3(get)]
    pub playable: bool,
    /// Card type: 0=Minion, 1=Spell, 2=Weapon, 3=Hero (M5 card text)
    #[pyo3(get)]
    pub card_type: i32,
    /// Has a battlecry (spell effects also live in this slot)
    #[pyo3(get)]
    pub has_battlecry: bool,
    /// Has a deathrattle
    #[pyo3(get)]
    pub has_deathrattle: bool,
    /// Has an aura
    #[pyo3(get)]
    pub has_aura: bool,
    /// Has a trigger
    #[pyo3(get)]
    pub has_trigger: bool,
    /// Battlecry/spell magnitudes (M5 card text)
    #[pyo3(get)]
    pub bc_damage: i32,
    #[pyo3(get)]
    pub bc_draw: i32,
    #[pyo3(get)]
    pub bc_summon: i32,
    #[pyo3(get)]
    pub bc_buff: i32,
    #[pyo3(get)]
    pub bc_heal: i32,
    #[pyo3(get)]
    pub bc_freeze: i32,
    #[pyo3(get)]
    pub bc_destroy: i32,
    /// Deathrattle magnitudes
    #[pyo3(get)]
    pub dr_damage: i32,
    #[pyo3(get)]
    pub dr_draw: i32,
    #[pyo3(get)]
    pub dr_summon: i32,
    /// Aura magnitudes
    #[pyo3(get)]
    pub aura_attack: i32,
    #[pyo3(get)]
    pub aura_health: i32,
}

impl From<&rviews::EntityView> for PyEntityView {
    fn from(v: &rviews::EntityView) -> Self {
        Self {
            entity_id: v.entity_id,
            card_id: v.card_id.clone(),
            name: v.name.clone(),
            cost: v.cost,
            attack: v.attack,
            health: v.health,
            can_attack: v.can_attack,
            taunt: v.taunt,
            divine_shield: v.divine_shield,
            stealth: v.stealth,
            elusive: v.elusive,
            windfury: v.windfury,
            charge: v.charge,
            race: v.race,
            frozen: v.frozen,
            playable: v.playable,
            card_type: v.card_type,
            has_battlecry: v.has_battlecry,
            has_deathrattle: v.has_deathrattle,
            has_aura: v.has_aura,
            has_trigger: v.has_trigger,
            bc_damage: v.bc_damage,
            bc_draw: v.bc_draw,
            bc_summon: v.bc_summon,
            bc_buff: v.bc_buff,
            bc_heal: v.bc_heal,
            bc_freeze: v.bc_freeze,
            bc_destroy: v.bc_destroy,
            dr_damage: v.dr_damage,
            dr_draw: v.dr_draw,
            dr_summon: v.dr_summon,
            aura_attack: v.aura_attack,
            aura_health: v.aura_health,
        }
    }
}

/// One player's view: hero, mana, zones, and entities.
#[pyclass(name = "PlayerView", module = "orange_stone")]
#[derive(Clone)]
pub struct PyPlayerView {
    /// Hero remaining health
    #[pyo3(get)]
    pub hero_health: i32,
    /// Hero armor
    #[pyo3(get)]
    pub hero_armor: i32,
    /// Hero attack (weapon + buffs)
    #[pyo3(get)]
    pub hero_attack: i32,
    /// Remaining mana this turn
    #[pyo3(get)]
    pub remaining_mana: i32,
    /// Total mana crystals
    #[pyo3(get)]
    pub total_mana: i32,
    /// Cards in hand
    #[pyo3(get)]
    pub hand_count: usize,
    /// Cards in deck
    #[pyo3(get)]
    pub deck_count: usize,
    /// Equipped weapon attack (`0` without a weapon)
    #[pyo3(get)]
    pub weapon_attack: i32,
    /// Equipped weapon durability (`0` without a weapon)
    #[pyo3(get)]
    pub weapon_durability: i32,
    /// Whether the hero power can be used this turn
    #[pyo3(get)]
    pub hero_power_usable: bool,
    /// Hero power cost (`0` without a hero power)
    #[pyo3(get)]
    pub hero_power_cost: i32,
    /// Minions on the battlefield (left to right)
    #[pyo3(get)]
    pub field: Vec<PyEntityView>,
    /// Cards in hand (only for the perspective player; opponent's is empty)
    #[pyo3(get)]
    pub hand: Vec<PyEntityView>,
}

impl From<&rviews::PlayerView> for PyPlayerView {
    fn from(v: &rviews::PlayerView) -> Self {
        Self {
            hero_health: v.hero_health,
            hero_armor: v.hero_armor,
            hero_attack: v.hero_attack,
            remaining_mana: v.remaining_mana,
            total_mana: v.total_mana,
            hand_count: v.hand_count,
            deck_count: v.deck_count,
            weapon_attack: v.weapon_attack,
            weapon_durability: v.weapon_durability,
            hero_power_usable: v.hero_power_usable,
            hero_power_cost: v.hero_power_cost,
            field: v.field.iter().map(PyEntityView::from).collect(),
            hand: v.hand.iter().map(PyEntityView::from).collect(),
        }
    }
}

/// The full structured observation, from the perspective player's side.
#[pyclass(name = "Observation", module = "orange_stone")]
#[derive(Clone)]
pub struct PyObservation {
    /// Current turn number (1-based)
    #[pyo3(get)]
    pub turn: u32,
    /// Whether it is the perspective player's turn
    #[pyo3(get)]
    pub my_turn: bool,
    /// Whether the game has ended
    #[pyo3(get)]
    pub done: bool,
    /// Winner: `0` = not over / draw, `1` = player 1, `2` = player 2
    #[pyo3(get)]
    pub winner: u8,
    /// Whether the engine is waiting for a choice (mulligan, choose-one, …)
    #[pyo3(get)]
    pub awaiting_choice: bool,
    /// The perspective player
    #[pyo3(get)]
    pub me: PyPlayerView,
    /// The opponent (hand hidden)
    #[pyo3(get)]
    pub opponent: PyPlayerView,
}

impl From<&rviews::Observation> for PyObservation {
    fn from(v: &rviews::Observation) -> Self {
        Self {
            turn: v.turn,
            my_turn: v.my_turn,
            done: v.done,
            winner: v.winner,
            awaiting_choice: v.awaiting_choice,
            me: PyPlayerView::from(&v.me),
            opponent: PyPlayerView::from(&v.opponent),
        }
    }
}

/// Structured metadata of one legal action.
#[pyclass(name = "ActionView", module = "orange_stone")]
#[derive(Clone)]
pub struct PyActionView {
    /// Index into `legal_actions()` / `step(action_index)`
    #[pyo3(get)]
    pub index: usize,
    /// Action kind: `"end_turn" | "play" | "attack" | "hero_power" | "choose"`
    #[pyo3(get)]
    pub kind: String,
    /// Hand index of the card for `play`, else `-1`
    #[pyo3(get)]
    pub card_index: i32,
    /// Entity id: card (`play`), attacker (`attack`), hero (`hero_power`), else `-1`
    #[pyo3(get)]
    pub entity_id: i32,
    /// Target entity id (play target / defender / hero power target), else `-1`
    #[pyo3(get)]
    pub target_id: i32,
    /// Readable description (kept for `play.py`)
    #[pyo3(get)]
    pub description: String,
}

impl From<&rviews::ActionView> for PyActionView {
    fn from(v: &rviews::ActionView) -> Self {
        Self {
            index: v.index,
            kind: v.kind.to_string(),
            card_index: v.card_index,
            entity_id: v.entity_id,
            target_id: v.target_id,
            description: v.description.clone(),
        }
    }
}
