//! Component definitions — data types in the ECS.
//!
//! Every component is a Copy type stored in its corresponding `SparseSet`.
//! The newtype wrappers ensure type safety (an Attack cannot be used as a Health).
use serde::{Deserialize, Serialize};

use std::ops::{Add, AddAssign, Sub, SubAssign};

macro_rules! impl_arith {
    ($t:ty) => {
        impl Add for $t {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }
        }
        impl AddAssign for $t {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }
        impl Sub for $t {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                Self(self.0 - rhs.0)
            }
        }
        impl SubAssign for $t {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }
    };
}

/// Health component.
///
/// When Health ≤ 0 the entity is considered dead (hero death → game over, minion death → moved to the graveyard).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct Health(pub i32);
impl From<i32> for Health {
    fn from(v: i32) -> Self {
        Self(v)
    }
}
impl Health {
    /// Returns `true` if health ≤ 0 (the entity is dead).
    #[must_use]
    pub const fn is_dead(self) -> bool {
        self.0 <= 0
    }
}
impl_arith!(Health);

/// Attack component.
///
/// Heroes and some minions can have 0 attack.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct Attack(pub i32);
impl From<i32> for Attack {
    fn from(v: i32) -> Self {
        Self(v)
    }
}
impl_arith!(Attack);

/// Mana cost component.
///
/// Reserved in Phase 1; the mana crystal system is not used yet.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct Cost(pub i32);
impl From<i32> for Cost {
    fn from(v: i32) -> Self {
        Self(v)
    }
}
impl_arith!(Cost);

/// Card type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardType {
    /// Minion — can be on the board, attack, and be attacked
    Minion,
    /// Hero — one per player; its death ends the game
    Hero,
    /// Weapon (Phase 2+)
    Weapon,
    /// Spell (Phase 2+)
    Spell,
}

/// Number of attacks already used this turn.
///
/// Each minion/hero can attack at most once per turn (Windfury in Phase 2+ raises this to 2).
/// Reset to 0 by the `TurnStarted` event at the start of a turn.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct AttacksUsed(pub u8);
impl From<u8> for AttacksUsed {
    fn from(v: u8) -> Self {
        Self(v)
    }
}
impl AttacksUsed {
    /// Returns `true` if the attack budget for this turn is exhausted (1 by default).
    /// Pass `max_attacks` to support Windfury (2 attacks).
    #[must_use]
    pub const fn is_exhausted_with(self, max_attacks: u8) -> bool {
        self.0 >= max_attacks
    }
}

/// Battlecry component — triggered when a minion is summoned.
///
/// `Battlecry` is detected while handling the `MinionSummoned` event;
/// the triggered effect is defined by `CardEffect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Battlecry(pub crate::core::effect::CardEffect);

/// Deathrattle component — triggered when a minion dies.
///
/// `Deathrattle` is detected while handling the `MinionDied` event (before moving to the graveyard);
/// the triggered effect is defined by `CardEffect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Deathrattle(pub crate::core::effect::CardEffect);

/// Taunt component — the enemy must attack this minion first.
///
/// If the enemy board has any minion with `Taunt`,
/// the attacker cannot target heroes or non-Taunt minions.
/// With multiple Taunt minions the attacker is free to choose which to attack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Taunt;

/// Weapon durability — consumes 1 on each hero attack; destroys the weapon when it reaches 0.
///
/// Durability lives in the weapon entity's `Weapon` component. After a hero attack is declared,
/// weapon durability decreases by 1; when it reaches 0, the weapon is destroyed.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct Durability(pub i32);
impl From<i32> for Durability {
    fn from(v: i32) -> Self {
        Self(v)
    }
}
impl_arith!(Durability);

/// Hero armor — absorbs damage before health is reduced.
///
/// When a hero takes damage, armor is consumed first. After armor reaches 0,
/// remaining damage is taken from health. Armor cannot go negative.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct Armor(pub i32);
impl From<i32> for Armor {
    fn from(v: i32) -> Self {
        Self(v)
    }
}
impl_arith!(Armor);

/// Hero power definition — an active ability the hero can use.
///
/// Most hero powers cost 2 mana and are limited to once per turn.
/// The effect is defined by `CardEffect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HeroPowerDef {
    /// Mana cost (usually 2)
    pub cost: i32,
    /// The effect
    pub effect: crate::core::effect::CardEffect,
}

/// Whether the hero power has been used this turn.
///
/// Reset to `false` at the start of a turn and set to `true` after using the hero power.
/// A `bool` is used instead of a counter because the hero power can be used only once per turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct HeroPowerUsed(pub bool);

/// Aura effect — a passive effect that continuously affects matching entities.
///
/// Auras do not modify base attributes; instead they apply buffs dynamically at query time.
/// When the aura source dies or leaves the battlefield, the effect disappears automatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Aura {
    /// Aura effect kind
    pub effect: AuraEffect,
    /// Target scope
    pub target: AuraTarget,
}

/// Aura effect kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuraEffect {
    /// +N/+M stat buff
    GainStats {
        /// Attack increase
        attack: i32,
        /// Health increase
        health: i32,
    },
    /// +N attack
    GainAttack(i32),
    /// +N health
    GainHealth(i32),
    /// Spell cost reduction (Sorcerer's Apprentice — friendly spells in hand cost less)
    ReduceSpellCost(i32),
    /// Minion cost reduction (Summoning Portal — friendly minions in hand cost less, with a floor)
    ReduceMinionCost {
        /// Cost reduction
        amount: i32,
        /// Cost floor
        min: i32,
    },
}

/// Aura target scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuraTarget {
    /// Adjacent minions (one on each side)
    AdjacentMinions,
    /// Other friendly minions (excluding itself)
    OtherFriendlyMinions,
    /// All friendly minions (including itself)
    AllFriendlyMinions,
    /// All enemy minions
    AllEnemyMinions,
}

/// Secret — a face-down, passively triggered spell.
///
/// After being played, a secret card enters the `SetAside` zone (hidden from the opponent).
/// When its trigger condition is met, the secret is revealed and its effect executes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Secret {
    /// Trigger condition
    pub trigger: SecretTrigger,
    /// Effect executed after triggering
    pub effect: crate::core::effect::CardEffect,
}

/// Secret trigger condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecretTrigger {
    /// After a friendly character (hero or minion) is attacked
    AfterFriendlyAttacked,
    /// After an enemy minion is played
    AfterEnemyMinionPlayed,
    /// After the enemy hero attacks
    AfterEnemyHeroAttacks,
    /// At the start of the friendly turn
    OnFriendlyTurnStart,
    /// After a minion dies
    AfterMinionDied,
    /// After the enemy casts a spell (Counterspell-type secrets)
    WhenEnemySpellCast,
    /// When an enemy minion attacks the friendly hero (Vaporize-type secrets)
    WhenEnemyMinionAttacksHero,
    /// When the enemy attacks the friendly hero (Misdirection-type secrets — attacker may be a minion or hero)
    WhenEnemyAttacksHero,
    /// When the enemy attacks (Noble Sacrifice-type secrets)
    WhenEnemyAttacks,
    /// When a friendly minion takes damage (Snake Trap-type secrets)
    WhenFriendlyMinionDamaged,
}

/// Divine Shield — absorbs one instance of damage, then disappears.
///
/// When a character with Divine Shield takes damage, the shield is removed and the damage is fully absorbed.
/// Divine Shields do not stack (an entity can have at most one).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct DivineShield;

/// Windfury — can attack twice per turn.
///
/// A character with Windfury can attack up to 2 times per turn (instead of the default 1).
/// The `AttacksUsed.is_exhausted()` check must account for whether Windfury is present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Windfury;

/// Charge — can attack on the turn it is summoned.
///
/// When a minion is summoned, if it has the Charge component it is not marked as having attacked
/// (i.e. `AttacksUsed` stays 0, allowing an immediate attack).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Charge;

/// Spell damage — increases the damage dealt by spells.
///
/// All friendly `SpellDamage` values on the board are summed and added to spell damage.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct SpellDamage(pub i32);
impl From<i32> for SpellDamage {
    fn from(v: i32) -> Self {
        Self(v)
    }
}
impl_arith!(SpellDamage);

/// Freeze — the character is frozen and skips its next attack opportunity.
///
/// A frozen character thaws at the start of its turn (the Freeze component is removed).
/// While frozen, the character cannot attack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Freeze;

/// Cannot attack — this minion cannot initiate attacks (e.g. Ragnaros, Ancient Watcher).
///
/// `CantAttack` is checked during attack validation.
/// Unlike Freeze, which is a temporary state (cleared at turn start), CantAttack is permanent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct CantAttack;

/// End-of-turn effect — triggered at the end of every turn.
///
/// The effect is defined by `CardEffect` and detected while handling the `TurnEnded` event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EndTurnEffect(pub crate::core::effect::CardEffect);

/// Spell-cast trigger effect — triggered when a friendly player casts a spell.
///
/// The effect is defined by `CardEffect` and detected while handling the `SpellCast` event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpellTrigger(pub crate::core::effect::CardEffect);

/// Minion-death trigger effect — triggered when a friendly minion dies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeathTrigger(pub crate::core::effect::CardEffect);

/// Minion-summon trigger effect — triggered when a friendly minion is summoned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SummonTrigger(pub crate::core::effect::CardEffect);

/// Choose One effect — alternative effects for Druid Choose One cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChooseOneEffect(pub crate::core::effect::CardEffect);

/// Combo effect — the alternative effect of Rogue Combo cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComboEffect(pub crate::core::effect::CardEffect);

/// Attack equals health — this minion's attack always equals its current health (Lightspawn).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct AttackEqualsHealth;

/// Temporary attack debuff — cleared at the end of the turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct TempAttackDebuff(pub i32);

/// Card ID — records the original card definition ID of the entity.
///
/// Used to look up `CardDef` at runtime (transform, secret mounting, random card pools, etc.).
/// Set when minion/weapon/spell entities are created (summoned, equipped, built).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct CardId(pub &'static str);

impl<'de> serde::Deserialize<'de> for CardId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        // All card IDs come from the static card library — resolve back to &'static str on deserialization
        crate::cards::def::card_by_id(&s)
            .map(|def| CardId(def.id))
            .ok_or_else(|| serde::de::Error::custom(format!("unknown card id: {s}")))
    }
}

/// Poison — damage dealt to a minion kills it outright.
///
/// When a Poison character deals damage to a minion, the target's health is set to 0 directly (bypassing normal damage reduction).
/// Divine Shield still absorbs Poison damage (the Divine Shield check comes first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Poison;

/// Stealth — the enemy cannot attack this character or target it with single-target effects.
///
/// A Stealth character is still affected by AOE (all-enemy damage). Stealth is not permanent:
/// it is removed when the character attacks (this engine simplifies it to permanent Stealth; removal logic is left for later).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Stealth;

/// Immune — this character cannot take any damage.
///
/// Damage taken by an Immune character is completely ignored (attacks are still consumed and weapon durability still drops).
/// Immune is a temporary state (Bestial Wrath — until end of turn), cleared at the end of the turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Immune;

/// Overload marker — triggers friendly minions' overload triggers when this card is played (Unbound Elemental).
///
/// This engine does not simulate overload's mana locking; it only serves as a trigger marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Overload;

/// Overload trigger effect — triggered when a friendly player plays a card with overload (Unbound Elemental).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OverloadTrigger(pub crate::core::effect::CardEffect);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_is_dead() {
        assert!(Health(0).is_dead());
        assert!(Health(-1).is_dead());
        assert!(!Health(1).is_dead());
    }

    #[test]
    fn health_arithmetic() {
        let mut h = Health(10);
        h += Health(5);
        assert_eq!(h, Health(15));
        h -= Health(3);
        assert_eq!(h, Health(12));
        let diff = Health(10) - Health(3);
        assert_eq!(diff, Health(7));
    }

    #[test]
    fn attacks_used_is_exhausted() {
        assert!(AttacksUsed(1).is_exhausted_with(1));
        assert!(AttacksUsed(2).is_exhausted_with(2));
        assert!(!AttacksUsed(1).is_exhausted_with(2));
        assert!(!AttacksUsed(0).is_exhausted_with(1));
    }
}
