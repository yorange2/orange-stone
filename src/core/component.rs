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
    /// Location (Core Set W8) — sits on the board, is not attackable or
    /// targetable, and is activated once per turn (durability charges)
    Location,
    /// Enchantment (Core Set W8) — a buff-instance token card, never
    /// playable (the engine models buffs as Enchantment components; the
    /// card definitions exist for data completeness)
    Enchantment,
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

/// Minion race / tribe (fidelity-debt W1; Core Set W0 added the five
/// non-Classic races) — Beast, Murloc, Demon, Dragon, Elemental, Mechanical,
/// Pirate, Totem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Race {
    /// Beast
    Beast,
    /// Murloc
    Murloc,
    /// Demon
    Demon,
    /// Dragon (Core Set W0)
    Dragon,
    /// Elemental (Core Set W0)
    Elemental,
    /// Mechanical (Core Set W0)
    Mechanical,
    /// Pirate (Core Set W0)
    Pirate,
    /// Totem (Core Set W0)
    Totem,
    /// Undead (Core Set W1 — data-driven: Underking, Malignant Horror and
    /// the March-of-the-Lich-King reprints carry the tribe)
    Undead,
    /// Quilboar (Core Set W3b — Hench-Clan Thug)
    Quilboar,
    /// Draenei (Core Set W4a — Battle Vicar)
    Draenei,
    /// Naga (2025–2026 expansions M0.2 — Gladesong Siren, Azshara Ocean Lord, …)
    Naga,
}

/// Imbue hero-power classes (2025–2026 expansions M1-W1 — the Emerald Dream
/// imbue mechanic): the six classes whose hero powers are replaced by their
/// imbued forms on the first imbue. Class detection reads the hero's card ID
/// prefix — `HERO_01` = Warrior, `HERO_02` = Shaman, `HERO_03` = Rogue,
/// `HERO_04` = Paladin, `HERO_05` = Hunter, `HERO_06` = Druid, `HERO_07` =
/// Warlock, `HERO_08` = Mage, `HERO_09` = Priest, `HERO_10` = Death Knight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImbueClass {
    /// Druid — Blessing of the Golem (EDR_847p)
    Druid,
    /// Hunter — Blessing of the Wolf (EDR_850p)
    Hunter,
    /// Mage — Blessing of the Wisp (EDR_851p)
    Mage,
    /// Paladin — Blessing of the Dragon (EDR_445p)
    Paladin,
    /// Priest — Blessing of the Moon (EDR_449p)
    Priest,
    /// Shaman — Blessing of the Wind (EDR_448p)
    Shaman,
    /// Rogue — the Timeways miniset hero power (END_000p, from the END_000
    /// Eventuality battlecry / imbue); costs 1, draws an other-class minion
    /// and Rewinds (2025–2026 expansions M3-W3)
    Rogue,
    /// Death Knight — the Timeways miniset hero power (END_003p, from the
    /// END_003 Finality battlecry / imbue); costs 0, buffs the first Undead
    /// played each turn (2025–2026 expansions M3-W3)
    DeathKnight,
}

impl ImbueClass {
    /// Maps a hero card ID to the imbue class: the six imbuing classes
    /// return `Some`, the other four classes and unknown IDs return `None`
    /// (a hero without a card ID — e.g. the `GameState::new()` defaults —
    /// counts imbues without ever replacing the hero power).
    #[must_use]
    pub fn from_hero_card_id(id: &str) -> Option<Self> {
        match id {
            "HERO_02" => Some(Self::Shaman),
            "HERO_04" => Some(Self::Paladin),
            "HERO_05" => Some(Self::Hunter),
            "HERO_06" => Some(Self::Druid),
            "HERO_08" => Some(Self::Mage),
            "HERO_09" => Some(Self::Priest),
            "HERO_03" => Some(Self::Rogue),
            "HERO_10" => Some(Self::DeathKnight),
            _ => None,
        }
    }
}

/// Dark gifts (2025–2026 expansions M1-W2 — the Emerald Dream dark-gift
/// mechanic): the ten card-level upgrades applied by the EDR dark-gift
/// Discover cards. A gift is attached to a minion card and persists across
/// zones (hand / deck / play) — the `World` `dark_gifts` component carries
/// the markers and `engine::trigger::apply_dark_gift` applies the static
/// effects (enchantments + keyword components). The variant order IS the
/// fixed gift pool (index 0..=9 drives `ALL_DARK_GIFTS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DarkGiftKind {
    /// +3 attack and Lifesteal
    AttackLifesteal,
    /// +2/+2 and Elusive
    StatsElusive,
    /// Cost (2) less, -2 attack (the official "only if attack stays at
    /// least 1" filter is not applied — registered simplification)
    CostDiscount,
    /// Charge
    Charge,
    /// When you play this, summon a 2/2 copy of it
    SummonCopyOnPlay,
    /// Your Battlecries trigger twice (official "only if it has a battlecry"
    /// filter is not applied — registered simplification)
    BattlecryTwice,
    /// +4 health and Taunt
    HealthTaunt,
    /// Reborn, and reborn at full health with enchantments
    RebornFull,
    /// +4/+5 and place this on top of your deck
    DeckTopBuff,
    /// Divine Shield and Windfury
    ShieldWindfury,
}

/// The fixed dark-gift pool — index order defines the random gift pick
/// (`engine::trigger` resolves gifts by this order).
pub const ALL_DARK_GIFTS: [DarkGiftKind; 10] = [
    DarkGiftKind::AttackLifesteal,
    DarkGiftKind::StatsElusive,
    DarkGiftKind::CostDiscount,
    DarkGiftKind::Charge,
    DarkGiftKind::SummonCopyOnPlay,
    DarkGiftKind::BattlecryTwice,
    DarkGiftKind::HealthTaunt,
    DarkGiftKind::RebornFull,
    DarkGiftKind::DeckTopBuff,
    DarkGiftKind::ShieldWindfury,
];

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
    /// Charge aura (Tundra Rhino — friendly Beasts have Charge). Consulted via
    /// `World::effective_charge` (base Charge component or an applying aura).
    GrantCharge,
    /// First-minion cost reduction marker (Pint-Sized Summoner): while the
    /// source is on the board, the owner's first minion each turn costs
    /// `amount` less. Consulted by `engine::cost::play_cost`; silencing the
    /// source removes the aura and the discount.
    FirstMinionDiscount {
        /// Cost reduction for the first minion played this turn
        amount: i32,
    },
    /// Hand-cost increase for ALL minions (Mana Wraith — all minions cost 1
    /// more; affects both players' hands). Consulted by
    /// `World::effective_cost`.
    IncreaseMinionCost {
        /// Cost increase
        amount: i32,
    },
    /// Hand-cost increase for the OWNER's minions only (Venture Co.
    /// Mercenary — your minions cost 3 more).
    IncreaseMinionCostFriendly {
        /// Cost increase
        amount: i32,
    },
    /// Conditional Charge marker (Southsea Deckhand — has Charge while you
    /// have a weapon equipped). Consulted by `World::effective_charge`.
    ChargeWithWeapon,
    /// Triggers of the owner's cards fire twice while the source is on the
    /// board (2025–2026 expansions M3-W2b — TIME_064 Deios the Betrayer).
    /// Unit marker: the aura's presence is the whole effect; the doubling
    /// is wired at the battlecry / weapon play / choose-one / deathrattle /
    /// hero-power / turn-end trigger sites. Silenceable like every aura.
    DoubleTriggers,
    /// Rewind replays are doubled while the source is on the board
    /// (2025–2026 expansions M3-W3 — END_036 Morchie: while this is on the
    /// board, Rewind triggers twice; consulted by `engine::rewind` — the
    /// same unit-marker pattern as `DoubleTriggers`, §22)
    RewindKeepsBothOutcomes,
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
    /// Friendly minions of the given race, including the source (Tundra Rhino)
    FriendlyRace(Race),
    /// Friendly minions of the given race, excluding the source
    /// (Murloc Warleader, Siegebreaker)
    OtherFriendlyRace(Race),
}

/// Secret — a face-down, passively triggered spell.
///
/// After being played, a secret card enters the `SetAside` zone (hidden from the opponent).
/// When its trigger condition is met, the secret is revealed and its effect executes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Secret {
    /// Trigger condition
    pub trigger: SecretTrigger,
    /// Effect executed after triggering (None for negation-only secrets such
    /// as Counterspell — the reveal itself is the effect)
    pub effect: Option<crate::core::effect::CardEffect>,
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
    /// When the friendly hero takes damage (Eye for an Eye)
    WhenFriendlyHeroDamaged,
    /// When the friendly hero takes fatal damage (Ice Block — the hit would kill)
    WhenFriendlyHeroFatallyDamaged,
    /// After the opponent plays three cards in a turn (Rat Trap — Core Set W5)
    AfterEnemyPlaysThreeCards,
    /// After a friendly minion that was played this turn dies (2025–2026
    /// expansions M3-W2a — TIME_620 Timecode the End; the "turn after it
    /// was played" wording is approximated as "any later turn" — the
    /// PlayedThisTurn marker is set at play and cleared at turn start, so
    /// a same-turn death also fires the secret, §20)
    WhenFriendlyMinionDiedTurnAfterPlayed,
    /// At the end of the ENEMY's turn (2025–2026 expansions M3-W3 —
    /// END_024 Flames of Infinity: at the end of your opponent's turn,
    /// deal infinite damage to their highest-health minion; the enemy
    /// turn's `Event::TurnEnded` is the event, matching `Event::TurnEnded {
    /// player }` against the opponent)
    WhenEnemyTurnEnds,
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
/// Freeze timing (engine-mechanics roadmap M2): a character frozen during
/// the opponent's turn keeps Freeze through its owner's next turn — the
/// `AttackDeclared`/validation freeze check blocks its attacks — and thaws
/// in the turn-end wrap-up of that turn (after the missed attack
/// opportunity), matching HS. The turn-start snapshot
/// (`Player::frozen_at_turn_start`) distinguishes "frozen at the start of
/// the owner's turn" (thaws this wrap-up) from "frozen during the turn"
/// (stays frozen into the next turn).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Freeze;

/// Cannot attack — this minion cannot initiate attacks (e.g. Ragnaros, Ancient Watcher).
///
/// `CantAttack` is checked during attack validation.
/// Unlike Freeze, which is a temporary state (thawed in the turn-end
/// wrap-up), CantAttack is permanent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct CantAttack;

/// Trigger event class — what a registered trigger responds to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriggerEvent {
    /// A friendly minion is summoned
    FriendlyMinionSummoned,
    /// A friendly minion dies
    FriendlyMinionDied,
    /// A friendly player casts a spell
    FriendlySpellCast,
    /// ANY player casts a spell (Lorewalker Cho — copies the cast spell to
    /// the other player's hand; global scope, fires for both players).
    /// `Event::SpellCast` threads the spell entity as the event subject.
    AnySpellCast,
    /// A friendly player plays a card with Overload
    FriendlyOverloadPlayed,
    /// This minion takes damage (Acolyte of Pain)
    ThisMinionDamaged,
    /// A friendly minion takes damage (Frothing Berserker, Armorsmith)
    FriendlyMinionDamaged,
    /// At the start of the owner's turn
    TurnStart,
    /// At the end of the owner's turn
    TurnEnd,
    /// Any character is healed (Lightwarden — gain +2 Attack whenever a
    /// character is healed; fires for friendly and enemy heals alike)
    CharacterHealed,
    /// A minion is healed (Northshire Cleric — draw a card whenever a minion
    /// is healed). Unlike `CharacterHealed` this excludes heroes, and like it
    /// the scope is global: either player's minion healing fires it.
    MinionHealed,
    /// A card is played (Questing Adventurer — whenever YOU play a card;
    /// friendly scope)
    CardPlayed,
    /// A Secret is played (Secretkeeper — whenever a Secret is played; fires
    /// for both players)
    SecretPlayed,
    /// A friendly minion lost its Divine Shield (Highlord Fordragon — Core
    /// Set W3b; the shield-absorbing damage is the subject)
    DivineShieldLost,
    /// A card is drawn (Keymaster Alabaster, Eredar Deceptor — Core Set
    /// W3b; the drawn card is the subject)
    CardDrawn,
    /// Any minion dies (Flesheathing Ghoul — gain +1 Attack whenever a minion
    /// dies; fires for both players)
    MinionDied,
    /// The entity itself attacks (Blessing of Wisdom — the buffed minion
    /// draws a card whenever it attacks; pinned to the attacker)
    Attacked,
    /// The OWNER'S HERO attacked (Hench-Clan Thug — Core Set W3b; the hero
    /// is the subject, friendly scope)
    HeroAttacked,
    /// THIS minion was attacked (Wrathspike Brute — Core Set W3c; the
    /// attacker is the subject, the pinned minion is the defender)
    ThisMinionAttacked,
    /// The entity attacks a MINION (Gorehowl — the weapon loses 1 Attack
    /// when the hero attacks a minion; pinned to the attacker or the
    /// attacker's equipped weapon)
    AttackedMinion,
    /// The OWNER'S HERO attacked a minion, with the DEFENDER as the subject
    /// (Defiled Spear — the splash must exclude the attacked minion, so the
    /// trigger carries the defender rather than the attacker; pinned to the
    /// defender itself or the attacking hero's equipped weapon)
    HeroAttackedMinion,
    /// A secret owned by the trigger's player is revealed (Eaglehorn Bow —
    /// +1 Durability; friendly-scoped via the revealer, unlike the played
    /// event which fires when the secret is played)
    FriendlySecretRevealed,
    /// A friendly minion (not a hero) declared an attack (M2-W4b —
    /// Archaios's "after a friendly minion attacks"; friendly scope with
    /// the attacker as the subject — unlike `Attacked`/`AttackedMinion`
    /// this is not pinned to the attacker, so the trigger rides Archaios)
    FriendlyMinionAttacked,
    /// A friendly MINION attacked a MINION, with the DEFENDER as the
    /// subject (The Great Dracorex — M2-W4c: the splash must exclude the
    /// attacked minion, so the trigger carries the defender; friendly
    /// scope, not pinned — the trigger rides the attacking Dracorex)
    AttackedEnemyMinion,
    /// The entity itself survived damage — the damage was applied and it
    /// is still alive (2025–2026 expansions M3-W2a — TIME_050 Sentient
    /// Hourglass, TIME_055 Unknown Voyager; the survival is predicted
    /// before the damage resolves, the Slam convention, so the triggers
    /// are pinned to the damaged minion like ThisMinionDamaged)
    SurvivedDamage,
    /// A friendly spell was cast ON a friendly MINION, with the target as
    /// the subject (2025–2026 expansions M3-W3 — END_026 Fragment of
    /// Nothing: draw a card after you cast a spell on a minion; the
    /// trigger is pinned to the Fragment itself, so it fires when ANY
    /// friendly minion is the target of a friendly spell — the subject
    /// identifies the target). The `Event::SpellCast` site threads the
    /// target through `target: Option<Entity>`.
    FriendlySpellCastOnMinion,
    /// The OWNER activated their hero power, with the hero-power entity as
    /// the subject (2025–2026 expansions M3-W3 — END_008 Enduring Roach:
    /// refresh 2 Mana Crystals after you use your hero power; fired by the
    /// HeroPowerActivated handler after the power's effect resolves)
    HeroPowerUsed,
}

/// Trigger timing — Hearthstone's "whenever" / "after" classification.
///
/// "Whenever" triggers fire as the event is processed; "after" triggers fire
/// after all "whenever" triggers of the event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriggerTiming {
    /// Fires as the event is processed
    Whenever,
    /// Fires after all "whenever" triggers
    After,
}

/// A registered trigger — the per-entity trigger registration of roadmap G2
/// (RS `ITrigger` / SB `TriggerManager` analogue).
///
/// Replaces the ad-hoc per-class trigger components: an entity registers its
/// triggers with this component, and the engine fires them in play order with
/// current-player precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Trigger {
    /// The event class this trigger responds to
    pub event: TriggerEvent,
    /// "Whenever" or "after" timing
    pub timing: TriggerTiming,
    /// The effect resolved when the trigger fires
    pub effect: crate::core::effect::CardEffect,
    /// Optional race condition: the trigger only fires when the event's
    /// subject has this race (Murloc Tidecaller — a friendly Murloc was
    /// summoned; Scavenging Hyena — a friendly Beast died).
    pub race: Option<Race>,
    /// Optional attack ceiling: the trigger only fires when the event's subject
    /// has at most this much Attack (Warsong Commander — "whenever you summon a
    /// minion with 3 or less Attack"). Measured on the subject's effective
    /// attack at the moment the event fires.
    pub max_attack: Option<i32>,
}

/// Choose One effect — alternative effects for Druid Choose One cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChooseOneEffect(pub crate::core::effect::CardEffect);

/// Combo effect — the alternative effect of Rogue Combo cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComboEffect(pub crate::core::effect::CardEffect);

/// Attack equals health — this minion's attack always equals its current health (Lightspawn).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct AttackEqualsHealth;

/// Enchantment expiry — when the enchantment is removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum EnchantmentExpiry {
    /// Removed only by silence / transform / leaving the battlefield
    #[default]
    Permanent,
    /// Removed at the end of the owner's turn (wrap-up)
    UntilEndOfTurn,
}

/// A stat modifier on an entity — the enchantment layer (roadmap G4).
///
/// Effective stats are `base + Σ enchantments (+ auras)`, with damage
/// subtracted from effective health. Buffs, debuffs, and cost modifiers attach
/// enchantments instead of writing the base components, which makes silence
/// (strip enchantments), transform, copy (Faceless Manipulator), until-end-of-
/// turn expiry, and zone-change retention expressible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Enchantment {
    /// Attack delta
    pub attack: i32,
    /// Health delta
    pub health: i32,
    /// Cost delta (cost modifiers survive zone changes — e.g. Shadowstep)
    pub cost: i32,
    /// When the enchantment is removed
    pub expiry: EnchantmentExpiry,
}

/// Accumulated damage (roadmap G4): effective health = base + Σ health deltas − damage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Damage(pub i32);

/// Cost modifier kind — the non-delta classes of the cost stack (roadmap G5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CostModifierKind {
    /// Set the cost to a fixed value (overrides composed base + deltas)
    Set(i32),
    /// The cost cannot go below this floor
    Min(i32),
}

/// A cost modifier on an entity (roadmap G5) — set-to-value and floor classes
/// of the modifier stack. No classic card uses them yet; the engine supports
/// them so cost effects compose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CostModifier {
    /// The modifier kind
    pub kind: CostModifierKind,
}

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

/// Enrage — a conditional bonus that is active only while the minion is damaged.
///
/// Real Hearthstone models Enrage as a state, not as a one-shot buff: the bonus
/// applies while the minion sits below its maximum Health, it does **not** stack
/// across separate damage instances, and it disappears the moment the minion is
/// healed back to full. It is an ability, so Silence removes it.
///
/// The bonus is therefore resolved on read (`World::effective_attack` and
/// `World::max_attacks`) rather than written into an enchantment, which is what
/// keeps it from stacking or surviving a full heal.
///
/// Note that Gurubashi Berserker is *not* an Enrage minion — its real text is
/// "Whenever this minion takes damage, gain +3 Attack", a permanent stacking
/// buff — so it stays wired to the `ThisMinionDamaged` trigger instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Enrage {
    /// Attack granted to this minion while it is damaged.
    pub attack: i32,
    /// Whether the minion also has Windfury while damaged (Raging Worgen).
    pub windfury: bool,
    /// Attack granted to the owner's weapon while this minion is damaged
    /// (Spiteful Smith — the bonus lives on the smith, not on the weapon).
    pub weapon_attack: i32,
}

/// Stealth — the enemy cannot attack this character or target it with single-target effects.
///
/// A Stealth character is still affected by AOE (all-enemy damage). Stealth is not permanent:
/// it is removed when the character attacks (this engine simplifies it to permanent Stealth; removal logic is left for later).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Stealth;

/// Rush (Core Set W1) — can attack enemy MINIONS the turn it is summoned
/// (no summoning sickness), but cannot attack the enemy hero until the next
/// turn. The summon-turn restriction is tracked by `SummonedThisTurn`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Rush;

/// Lifesteal (Core Set W1) — damage dealt by this character heals the
/// owner's hero for the damage dealt. Weapon damage counts; spell damage
/// counts (the spell entity carries the component); divine-shield and
/// immune absorptions heal nothing (no damage was dealt).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Lifesteal;

/// Reborn (Core Set W1) — the first time this minion would die, it is
/// resurrected on the spot as a fresh 1/1 copy instead: all buffs cleared,
/// base stats set to 1/1, Reborn spent, and summoning sickness applied
/// (the resurrection counts as a summon — on-summon triggers fire, the
/// minion cannot attack until next turn unless it has Rush/Charge, and
/// battlecries do NOT re-fire).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Reborn;

/// SummonedThisTurn (Core Set W1) — set on minions when they enter the
/// battlefield, cleared at the start of each turn. Rush consults it to
/// forbid hero attacks on the summoning turn; Charge/Rush minions do not
/// get summoning sickness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct SummonedThisTurn;

/// Tradeable (Core Set W2) — a hand card with Tradeable may be shuffled
/// back into the deck for 1 mana to draw a card (`Action::TradeCard`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Tradeable;

/// OutcastPlayed (Core Set W2) — set on a card at the moment it is played
/// when it sat at the leftmost or rightmost position of the hand; the
/// outcast effects resolve against this marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct OutcastPlayed;

/// Elusive (扰咒) — cannot be targeted by spells or hero powers (M5).
/// Attacks and battlecries CAN target it; AOE still hits it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Elusive;

/// Immune — this character cannot take any damage.
///
/// Damage taken by an Immune character is completely ignored (attacks are still consumed and weapon durability still drops).
/// Immune is a temporary state (Bestial Wrath — until end of turn), cleared at the end of the turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Immune;

/// Temporary — the Temporary keyword (2025–2026 expansions M2-W2, the
/// Un'Goro quest wave): a card in hand carrying this marker is discarded
/// at the end of its owner's turn (official rule), and playing one
/// progresses the TLC_446 quest (PlayTemporaryCards). No W2 card creates
/// Temporary cards — the creators are W4 cards; the F5 scenarios inject
/// the marker directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Temporary;

/// Dormant (沉睡) — the Dormant keyword (2025–2026 expansions M3-W2a,
/// Across the Timeways). A dormant minion is on the battlefield but
/// asleep: it cannot attack, cannot be targeted (like Stealth), takes no
/// damage, and has no board presence (its auras and triggers do not
/// fire). `turns` counts down at the start of the owner's turn; at 0 the
/// component is removed and the minion awakens. Cards enter play dormant
/// via the `cards::dormant_at_summon` registry; TIME_442's imprisonment
/// applies it to enemy minions mid-game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Dormant {
    /// Remaining turns asleep, decremented at the owner's turn start.
    pub turns: u32,
}

/// CantAttackHeroesThisTurn (2025–2026 expansions M3-W2a) — a temporary
/// restriction: the minion may attack enemy minions but not the enemy
/// hero, until the end of the current turn (PMM Infinitizer TIME_043).
/// Cleared in the wrap-up step, mirroring the "until end of turn"
/// enchantment expiry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct CantAttackHeroesThisTurn;

/// TurnCostReducer (2025–2026 expansions M3-W2a) — a hand-card marker
/// that is reduced by (1) at the start of the owner's turn
/// (Circadiamancer TIME_102's "it costs (1) less each turn"). Read by
/// the cost pipeline; removed when the card leaves the hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct TurnCostReducer(pub u32);

/// DoubleDamageTaken (2025–2026 expansions M3-W2a) — a temporary marker
/// making the marked character take double damage (TIME_858 Temporal
/// Construct's "takes double damage").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct DoubleDamageTaken;

/// CostHealth — a hand card that costs Health instead of Mana
/// (2025–2026 expansions M2-W4a — Whispering Stone's gotten Fel spells:
/// "They cost Health instead of Mana"). The CardPlayed path reads the
/// marker to pay the card's cost from the hero's Health instead of mana.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct CostHealth;

/// PlayedThisTurn (2025–2026 expansions M3-W2a) — a minion-played-this-turn
/// marker on a friendly minion entity, set when a minion is played from
/// the hand and cleared at the turn start. Used by the TIME_620 secret
/// ("After a friendly minion dies the turn after it was played") — the
/// registered approximation models "the turn after" as "any later turn".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct PlayedThisTurn;

/// Overload amount — the mana locked on the owner's next turn (roadmap F1).
/// Also triggers friendly minions' overload triggers when the card is played
/// (Unbound Elemental).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Overload(pub i32);

/// Runtime state of a quest's optional second progress bar (2025–2026
/// expansions M2-W2 — TLC_817 Reach Equilibrium: cast 4 Holy spells AND
/// 4 Shadow spells on one card). The first bar's state lives directly on
/// `Quest`; this mirrors it for the second bar.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct QuestSecondState {
    /// Progress toward the second bar's target (0..=target).
    pub progress: u32,
    /// The second bar's completion target (mirrors `SecondQuestDef::target`).
    pub target: u32,
    /// Per-condition tracked values for the second bar (same semantics as
    /// `Quest::markers`).
    pub markers: Vec<u32>,
}

/// Quest progress (2025–2026 expansions M2-W1) — runtime state of a quest
/// card sitting in the player's `Zone::Quest` slot.
///
/// Static quest data (condition, target, reward) lives in the `cards::quest`
/// registry, keyed by card id; this component tracks the mutable half.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Quest {
    /// Progress toward the quest's target (0..=target).
    pub progress: u32,
    /// The quest's completion target (mirrors `QuestDef::target`).
    pub target: u32,
    /// Repeatable quests reset to 0 on completion and stay in the slot.
    pub repeatable: bool,
    /// Per-condition tracked values (unique races / distinct turns / attack
    /// values for set-based conditions); progress skips values already seen.
    pub markers: Vec<u32>,
    /// Optional second progress bar (TLC_817, M2-W2): the card completes —
    /// and leaves the quest slot — only when BOTH bars are done; each bar's
    /// reward resolves independently at its own target.
    pub second: Option<QuestSecondState>,
}

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

    /// The hero-card-ID → imbue class mapping (2025–2026 expansions M1-W1,
    /// extended M3-W3): the eight imbuing classes resolve — the six of
    /// M1-W1 plus Rogue and Death Knight (The End of Time miniset: END_000p
    /// and END_003p), the remaining two classes (Warrior, Warlock) and
    /// unknown IDs (no card ID — the `GameState::new()` defaults) resolve
    /// to `None`.
    #[test]
    fn imbue_class_from_hero_card_id_mapping() {
        use super::ImbueClass;
        assert_eq!(
            ImbueClass::from_hero_card_id("HERO_06"),
            Some(ImbueClass::Druid)
        );
        assert_eq!(
            ImbueClass::from_hero_card_id("HERO_05"),
            Some(ImbueClass::Hunter)
        );
        assert_eq!(
            ImbueClass::from_hero_card_id("HERO_08"),
            Some(ImbueClass::Mage)
        );
        assert_eq!(
            ImbueClass::from_hero_card_id("HERO_04"),
            Some(ImbueClass::Paladin)
        );
        assert_eq!(
            ImbueClass::from_hero_card_id("HERO_09"),
            Some(ImbueClass::Priest)
        );
        assert_eq!(
            ImbueClass::from_hero_card_id("HERO_02"),
            Some(ImbueClass::Shaman)
        );
        // M3-W3 — The End of Time: Rogue (END_000p Blessing of the Bronze)
        // and Death Knight (END_003p Blessing of the Infinite)
        assert_eq!(
            ImbueClass::from_hero_card_id("HERO_03"),
            Some(ImbueClass::Rogue)
        );
        assert_eq!(
            ImbueClass::from_hero_card_id("HERO_10"),
            Some(ImbueClass::DeathKnight)
        );
        // Warrior / Warlock — no imbued form
        assert_eq!(ImbueClass::from_hero_card_id("HERO_01"), None);
        assert_eq!(ImbueClass::from_hero_card_id("HERO_07"), None);
        assert_eq!(ImbueClass::from_hero_card_id("CORE_EX1_323"), None);
        assert_eq!(ImbueClass::from_hero_card_id(""), None);
    }
}
