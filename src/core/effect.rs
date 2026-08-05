//! Card effect definitions — compile-time constant CardEffect and EffectTarget.
//!
//! Phase 2 card effects: damage, card draw, summon, buff.
//! Effects are stored as `Copy` enum constants in `CardDef` and the `Battlecry`/`Deathrattle` components.
use serde::{Deserialize, Serialize};

/// Effect target selector.
///
/// When an effect is executed, the engine selects target entities based on this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectTarget {
    /// A random enemy character (hero or minion)
    AnyEnemy,
    /// A random enemy minion
    AnyEnemyMinion,
    /// All enemy minions
    AllEnemyMinions,
    /// All enemy characters (enemy hero + all enemy minions)
    AllEnemies,
    /// The enemy hero
    EnemyHero,
    /// Self (buff-type effects)
    Self_,
    /// All friendly minions
    AllFriendlyMinions,
    /// All minions (friend or foe)
    AllMinions,
    /// All characters (heroes + minions, friend or foe)
    AllCharacters,
    /// The friendly hero
    FriendlyHero,
    /// A damaged enemy minion
    DamagedEnemyMinion,
    /// A random friendly minion
    FriendlyMinion,
    /// A random enemy Taunt minion
    TauntEnemyMinion,
}

/// Card effect — an action executed when triggered.
///
/// Implements `Copy` so it can be stored as a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum CardEffect {
    /// Deal N damage
    DealDamage {
        /// Damage amount
        amount: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Draw N cards
    DrawCard {
        /// Number of cards to draw
        count: u32,
    },
    /// Summon a minion
    SummonMinion {
        /// Card ID to summon
        card_id: &'static str,
    },
    /// Grant +N/+M (buff self or a friendly minion)
    GainStats {
        /// Attack increase
        attack: i32,
        /// Health increase
        health: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Equip a weapon
    EquipWeapon {
        /// Weapon card ID to equip
        card_id: &'static str,
    },
    /// Gain armor
    GainArmor {
        /// Armor amount
        amount: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Return a minion to hand
    ReturnToHand {
        /// Target selection
        target: EffectTarget,
    },
    /// Increase a minion's mana cost (Freezing Trap effect)
    IncreaseCost {
        /// Mana cost increase
        amount: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Return a minion to hand and increase its mana cost (full Freezing Trap effect)
    ReturnToHandAndIncreaseCost {
        /// Mana cost increase
        amount: i32,
    },
    /// Destroy a minion (Shadow Word: Death, Assassinate)
    DestroyMinion {
        /// Target selection
        target: EffectTarget,
    },
    /// Silence a minion — remove all effect components
    SilenceMinion {
        /// Target selection
        target: EffectTarget,
    },
    /// Set attack to a fixed value
    SetAttack {
        /// Target attack value
        attack: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Restore health
    RestoreHealth {
        /// Amount to restore
        amount: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Freeze a character
    FreezeCharacter {
        /// Target selection
        target: EffectTarget,
    },
    /// Gain empty mana crystals
    GainManaCrystal {
        /// Number to gain
        count: i32,
    },
    /// Gain mana this turn only (The Coin) — does not add a permanent crystal
    GainManaThisTurn {
        /// Mana to gain this turn
        count: i32,
    },
    /// Destroy the enemy weapon
    DestroyWeapon,
    /// Give the hero temporary attack and optional armor (effective this turn; attack bonus cleared at end of turn)
    GainHeroAttack {
        /// Attack bonus
        attack: i32,
        /// Armor bonus (0 means no armor)
        armor: i32,
    },
    /// Deal damage equal to the hero's attack to the target
    DealHeroAttackDamage {
        /// Target selection
        target: EffectTarget,
    },
    /// Fully restore a minion's health
    FullHeal {
        /// Target selection
        target: EffectTarget,
    },
    /// Grant a minion Windfury
    GrantWindfury {
        /// Target selection
        target: EffectTarget,
    },
    /// Grant a minion Charge and optional attack
    GrantCharge {
        /// Target selection
        target: EffectTarget,
        /// Extra attack (0 means no attack bonus)
        attack_bonus: i32,
    },
    /// Double a minion's attack
    DoubleAttack {
        /// Target selection
        target: EffectTarget,
    },
    /// Double a minion's health
    DoubleHealth {
        /// Target selection
        target: EffectTarget,
    },
    /// Increase a friendly weapon's attack and durability
    BuffWeapon {
        /// Attack increase
        attack: i32,
        /// Durability increase
        durability: i32,
    },
    /// Discard a random card from hand
    DiscardRandomCard,
    /// Deal damage equal to the friendly hero's armor to the target
    DealArmorDamage {
        /// Target selection
        target: EffectTarget,
    },
    /// Destroy the enemy weapon and draw cards equal to its durability
    DestroyWeaponAndDraw,
    /// Return all minions to their owners' hands
    ReturnAllToHand,
    /// Set a minion's attack equal to its current health
    SetAttackToHealth {
        /// Target selection
        target: EffectTarget,
    },
    /// Destroy all minions except one random one
    DestroyAllExceptOne,
    /// Destroy a minion and restore health to the friendly hero
    DestroyAndHeal {
        /// Target selection
        target: EffectTarget,
        /// Amount to restore
        heal: i32,
    },
    /// Destroy a friendly minion and deal AOE damage equal to its attack
    DestroyAndAOE {
        /// Target: all enemy minions / all enemy characters
        target: EffectTarget,
    },
    /// Deal damage to two random enemy minions
    DealDamageToTwo {
        /// Damage amount
        amount: i32,
    },
    /// Deal damage + draw cards
    DealDamageAndDraw {
        /// Damage amount
        damage: i32,
        /// Target selection
        target: EffectTarget,
        /// Number of cards to draw
        draw: u32,
    },
    /// Deal damage to a minion and gain attack
    DamageAndGainAttack {
        /// Damage amount
        damage: i32,
        /// Attack bonus
        attack_bonus: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Destroy a friendly minion and gain its attack and health
    DestroyAdjacent {
        /// Whether to gain the stats
        gain_stats: bool,
    },
    /// Destroy a mana crystal
    DestroyManaCrystal,
    /// Add cards to the opponent's hand
    GiveCardsToOpponent {
        /// Number of cards to add
        count: u32,
    },
    /// Resurrect a friendly minion that died this turn with 1 health
    ResurrectMinion,
    /// Copy a random friendly minion's attack and health
    CopyMinionStats,
    /// Give an enemy minion -2 attack (effective this turn)
    TempDebuff {
        /// Attack reduction
        attack_reduction: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Reflect damage taken back to the attacker (secret effect)
    ReflectDamage,
    /// Deal damage to the target and return the caster to hand on combo (Headcrack)
    DealDamageAndReturnToHand {
        /// Damage amount
        amount: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Return a friendly minion to hand and reduce its cost (Shadowstep)
    ReturnFriendlyToHandAndReduceCost {
        /// Cost reduction
        amount: i32,
    },
    /// Deal damage equal to the target's attack to its adjacent minions (Betrayal)
    AdjacentDamage,
    /// Destroy your weapon and deal damage equal to its attack to all enemies (Blade Flurry)
    DestroyWeaponAndDealAttackToEnemies,
    /// Grant a friendly minion Stealth (Master of Disguise)
    GrantStealth,
    /// Summon multiple minions (Snake Trap's three snakes, Cenarius's two treants)
    SummonMultipleMinions {
        /// Card ID to summon
        card_id: &'static str,
        /// Summon count
        count: u32,
    },
    /// Deal damage to the enemy minion just played (Snipe — handled by secret.rs, needs event context)
    DamagePlayedMinion {
        /// Damage amount
        amount: i32,
    },
    /// Redirect the attack to another random character (Misdirection — handled by secret.rs)
    RedirectAttackToRandomCharacter,
    /// Summon a minion as the attack's new target (Noble Sacrifice — handled by secret.rs)
    SummonAndRedirectAttack {
        /// Defender card ID to summon
        card_id: &'static str,
    },
    /// Summon a 1/3 Spellbender and redirect spell damage (Spellbender — handled by secret.rs)
    SummonSpellbender,
    /// Your next secret costs (0) (Kirin Tor Mage)
    NextSecretCostsZero,
    /// Draw a card and reduce its cost (Far Sight)
    DrawCardAndReduceCost {
        /// Cost reduction
        amount: i32,
    },
    /// Grant all friendly minions "Deathrattle: summon the specified minion" (Soul of the Forest)
    GrantDeathrattleAll {
        /// Card ID to summon on deathrattle
        card_id: &'static str,
    },
    /// Add the specified card to the opponent's hand (King Mukla's Banana)
    GiveCardToOpponent {
        /// Card ID to give
        card_id: &'static str,
        /// Count
        count: u32,
    },
    /// Freeze a minion; if it is already frozen, deal damage instead (Ice Lance)
    FreezeOrDamage {
        /// Damage amount
        amount: i32,
    },
    /// Destroy a minion and gain its health (Natalie Seline)
    DestroyAndGainHealth,
    /// Grant a friendly minion an attack bonus and Immune until end of turn (Bestial Wrath)
    GrantAttackAndImmune {
        /// Attack bonus
        attack: i32,
    },
    /// Temporarily take control of an enemy minion until end of turn (Shadow Madness, attack ≤ 3)
    TakeControlUntilEndOfTurn,
    /// Permanently take control of an enemy minion (Mind Control)
    TakeControl,
    /// Corrupt an enemy minion — destroy it at the start of your turn (Corruption)
    Corrupt,
    /// Minions cannot drop below 1 health this turn (Commanding Shout)
    MinHealthUntilEndOfTurn,
    /// Transform the target minion into one of two alternatives (Tinkmaster Overspark)
    TransformToRandom {
        /// Alternative card A (5/5 devilsaur)
        card_a: &'static str,
        /// Alternative card B (1/1 squirrel)
        card_b: &'static str,
    },
    /// Add a random card to your hand (Tier 3 random generation)
    AddRandomCardToHand {
        /// Pool type
        pool: RandomPool,
    },
    /// Summon a random minion (Animal Companion/Barrens Stablehand)
    SummonRandomMinion {
        /// Pool type
        pool: RandomPool,
    },
    /// Add the specified card to your hand (Archmage Antonidas's Fireball)
    AddCardToHand {
        /// Card ID
        card_id: &'static str,
    },
    /// Deal damage; if the target dies, summon a random minion (Bane of Doom)
    DealDamageAndSummonIfKilled {
        /// Damage amount
        amount: i32,
        /// Pool summoned after the target dies
        pool: RandomPool,
    },
}

/// Deserialization mirror of CardEffect (owns all fields, no &'static str references).
#[derive(serde::Deserialize)]
enum CardEffectDe {
    DealDamage {
        amount: i32,
        target: EffectTarget,
    },
    DrawCard {
        count: u32,
    },
    SummonMinion {
        card_id: String,
    },
    GainStats {
        attack: i32,
        health: i32,
        target: EffectTarget,
    },
    EquipWeapon {
        card_id: String,
    },
    GainArmor {
        amount: i32,
        target: EffectTarget,
    },
    ReturnToHand {
        target: EffectTarget,
    },
    IncreaseCost {
        amount: i32,
        target: EffectTarget,
    },
    ReturnToHandAndIncreaseCost {
        amount: i32,
    },
    DestroyMinion {
        target: EffectTarget,
    },
    SilenceMinion {
        target: EffectTarget,
    },
    SetAttack {
        attack: i32,
        target: EffectTarget,
    },
    RestoreHealth {
        amount: i32,
        target: EffectTarget,
    },
    FreezeCharacter {
        target: EffectTarget,
    },
    GainManaCrystal {
        count: i32,
    },
    GainManaThisTurn {
        count: i32,
    },
    DestroyWeapon,
    GainHeroAttack {
        attack: i32,
        armor: i32,
    },
    DealHeroAttackDamage {
        target: EffectTarget,
    },
    FullHeal {
        target: EffectTarget,
    },
    GrantWindfury {
        target: EffectTarget,
    },
    GrantCharge {
        target: EffectTarget,
        attack_bonus: i32,
    },
    DoubleAttack {
        target: EffectTarget,
    },
    DoubleHealth {
        target: EffectTarget,
    },
    BuffWeapon {
        attack: i32,
        durability: i32,
    },
    DiscardRandomCard,
    DealArmorDamage {
        target: EffectTarget,
    },
    DestroyWeaponAndDraw,
    ReturnAllToHand,
    SetAttackToHealth {
        target: EffectTarget,
    },
    DestroyAllExceptOne,
    DestroyAndHeal {
        target: EffectTarget,
        heal: i32,
    },
    DestroyAndAOE {
        target: EffectTarget,
    },
    DealDamageToTwo {
        amount: i32,
    },
    DealDamageAndDraw {
        damage: i32,
        target: EffectTarget,
        draw: u32,
    },
    DamageAndGainAttack {
        damage: i32,
        attack_bonus: i32,
        target: EffectTarget,
    },
    DestroyAdjacent {
        gain_stats: bool,
    },
    DestroyManaCrystal,
    GiveCardsToOpponent {
        count: u32,
    },
    ResurrectMinion,
    CopyMinionStats,
    TempDebuff {
        attack_reduction: i32,
        target: EffectTarget,
    },
    ReflectDamage,
    DealDamageAndReturnToHand {
        amount: i32,
        target: EffectTarget,
    },
    ReturnFriendlyToHandAndReduceCost {
        amount: i32,
    },
    AdjacentDamage,
    DestroyWeaponAndDealAttackToEnemies,
    GrantStealth,
    SummonMultipleMinions {
        card_id: String,
        count: u32,
    },
    DamagePlayedMinion {
        amount: i32,
    },
    RedirectAttackToRandomCharacter,
    SummonAndRedirectAttack {
        card_id: String,
    },
    SummonSpellbender,
    NextSecretCostsZero,
    DrawCardAndReduceCost {
        amount: i32,
    },
    GrantDeathrattleAll {
        card_id: String,
    },
    GiveCardToOpponent {
        card_id: String,
        count: u32,
    },
    FreezeOrDamage {
        amount: i32,
    },
    DestroyAndGainHealth,
    GrantAttackAndImmune {
        attack: i32,
    },
    TakeControlUntilEndOfTurn,
    TakeControl,
    Corrupt,
    MinHealthUntilEndOfTurn,
    TransformToRandom {
        card_a: String,
        card_b: String,
    },
    AddRandomCardToHand {
        pool: RandomPool,
    },
    SummonRandomMinion {
        pool: RandomPool,
    },
    AddCardToHand {
        card_id: String,
    },
    DealDamageAndSummonIfKilled {
        amount: i32,
        pool: RandomPool,
    },
}

impl<'de> serde::Deserialize<'de> for CardEffect {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let de = CardEffectDe::deserialize(d)?;
        // All card IDs come from the static card library — resolve back to &'static str on deserialization
        let intern = |s: String| -> Result<&'static str, D::Error> {
            crate::cards::def::card_by_id(&s)
                .map(|def| def.id)
                .ok_or_else(|| serde::de::Error::custom(format!("unknown card id: {s}")))
        };
        Ok(match de {
            CardEffectDe::DealDamage { amount, target } => {
                CardEffect::DealDamage { amount, target }
            }
            CardEffectDe::DrawCard { count } => CardEffect::DrawCard { count },
            CardEffectDe::SummonMinion { card_id } => CardEffect::SummonMinion {
                card_id: intern(card_id)?,
            },
            CardEffectDe::GainStats {
                attack,
                health,
                target,
            } => CardEffect::GainStats {
                attack,
                health,
                target,
            },
            CardEffectDe::EquipWeapon { card_id } => CardEffect::EquipWeapon {
                card_id: intern(card_id)?,
            },
            CardEffectDe::GainArmor { amount, target } => CardEffect::GainArmor { amount, target },
            CardEffectDe::ReturnToHand { target } => CardEffect::ReturnToHand { target },
            CardEffectDe::IncreaseCost { amount, target } => {
                CardEffect::IncreaseCost { amount, target }
            }
            CardEffectDe::ReturnToHandAndIncreaseCost { amount } => {
                CardEffect::ReturnToHandAndIncreaseCost { amount }
            }
            CardEffectDe::DestroyMinion { target } => CardEffect::DestroyMinion { target },
            CardEffectDe::SilenceMinion { target } => CardEffect::SilenceMinion { target },
            CardEffectDe::SetAttack { attack, target } => CardEffect::SetAttack { attack, target },
            CardEffectDe::RestoreHealth { amount, target } => {
                CardEffect::RestoreHealth { amount, target }
            }
            CardEffectDe::FreezeCharacter { target } => CardEffect::FreezeCharacter { target },
            CardEffectDe::GainManaCrystal { count } => CardEffect::GainManaCrystal { count },
            CardEffectDe::GainManaThisTurn { count } => CardEffect::GainManaThisTurn { count },
            CardEffectDe::DestroyWeapon => CardEffect::DestroyWeapon,
            CardEffectDe::GainHeroAttack { attack, armor } => {
                CardEffect::GainHeroAttack { attack, armor }
            }
            CardEffectDe::DealHeroAttackDamage { target } => {
                CardEffect::DealHeroAttackDamage { target }
            }
            CardEffectDe::FullHeal { target } => CardEffect::FullHeal { target },
            CardEffectDe::GrantWindfury { target } => CardEffect::GrantWindfury { target },
            CardEffectDe::GrantCharge {
                target,
                attack_bonus,
            } => CardEffect::GrantCharge {
                target,
                attack_bonus,
            },
            CardEffectDe::DoubleAttack { target } => CardEffect::DoubleAttack { target },
            CardEffectDe::DoubleHealth { target } => CardEffect::DoubleHealth { target },
            CardEffectDe::BuffWeapon { attack, durability } => {
                CardEffect::BuffWeapon { attack, durability }
            }
            CardEffectDe::DiscardRandomCard => CardEffect::DiscardRandomCard,
            CardEffectDe::DealArmorDamage { target } => CardEffect::DealArmorDamage { target },
            CardEffectDe::DestroyWeaponAndDraw => CardEffect::DestroyWeaponAndDraw,
            CardEffectDe::ReturnAllToHand => CardEffect::ReturnAllToHand,
            CardEffectDe::SetAttackToHealth { target } => CardEffect::SetAttackToHealth { target },
            CardEffectDe::DestroyAllExceptOne => CardEffect::DestroyAllExceptOne,
            CardEffectDe::DestroyAndHeal { target, heal } => {
                CardEffect::DestroyAndHeal { target, heal }
            }
            CardEffectDe::DestroyAndAOE { target } => CardEffect::DestroyAndAOE { target },
            CardEffectDe::DealDamageToTwo { amount } => CardEffect::DealDamageToTwo { amount },
            CardEffectDe::DealDamageAndDraw {
                damage,
                target,
                draw,
            } => CardEffect::DealDamageAndDraw {
                damage,
                target,
                draw,
            },
            CardEffectDe::DamageAndGainAttack {
                damage,
                attack_bonus,
                target,
            } => CardEffect::DamageAndGainAttack {
                damage,
                attack_bonus,
                target,
            },
            CardEffectDe::DestroyAdjacent { gain_stats } => {
                CardEffect::DestroyAdjacent { gain_stats }
            }
            CardEffectDe::DestroyManaCrystal => CardEffect::DestroyManaCrystal,
            CardEffectDe::GiveCardsToOpponent { count } => {
                CardEffect::GiveCardsToOpponent { count }
            }
            CardEffectDe::ResurrectMinion => CardEffect::ResurrectMinion,
            CardEffectDe::CopyMinionStats => CardEffect::CopyMinionStats,
            CardEffectDe::TempDebuff {
                attack_reduction,
                target,
            } => CardEffect::TempDebuff {
                attack_reduction,
                target,
            },
            CardEffectDe::ReflectDamage => CardEffect::ReflectDamage,
            CardEffectDe::DealDamageAndReturnToHand { amount, target } => {
                CardEffect::DealDamageAndReturnToHand { amount, target }
            }
            CardEffectDe::ReturnFriendlyToHandAndReduceCost { amount } => {
                CardEffect::ReturnFriendlyToHandAndReduceCost { amount }
            }
            CardEffectDe::AdjacentDamage => CardEffect::AdjacentDamage,
            CardEffectDe::DestroyWeaponAndDealAttackToEnemies => {
                CardEffect::DestroyWeaponAndDealAttackToEnemies
            }
            CardEffectDe::GrantStealth => CardEffect::GrantStealth,
            CardEffectDe::SummonMultipleMinions { card_id, count } => {
                CardEffect::SummonMultipleMinions {
                    card_id: intern(card_id)?,
                    count,
                }
            }
            CardEffectDe::DamagePlayedMinion { amount } => {
                CardEffect::DamagePlayedMinion { amount }
            }
            CardEffectDe::RedirectAttackToRandomCharacter => {
                CardEffect::RedirectAttackToRandomCharacter
            }
            CardEffectDe::SummonAndRedirectAttack { card_id } => {
                CardEffect::SummonAndRedirectAttack {
                    card_id: intern(card_id)?,
                }
            }
            CardEffectDe::SummonSpellbender => CardEffect::SummonSpellbender,
            CardEffectDe::NextSecretCostsZero => CardEffect::NextSecretCostsZero,
            CardEffectDe::DrawCardAndReduceCost { amount } => {
                CardEffect::DrawCardAndReduceCost { amount }
            }
            CardEffectDe::GrantDeathrattleAll { card_id } => CardEffect::GrantDeathrattleAll {
                card_id: intern(card_id)?,
            },
            CardEffectDe::GiveCardToOpponent { card_id, count } => CardEffect::GiveCardToOpponent {
                card_id: intern(card_id)?,
                count,
            },
            CardEffectDe::FreezeOrDamage { amount } => CardEffect::FreezeOrDamage { amount },
            CardEffectDe::DestroyAndGainHealth => CardEffect::DestroyAndGainHealth,
            CardEffectDe::GrantAttackAndImmune { attack } => {
                CardEffect::GrantAttackAndImmune { attack }
            }
            CardEffectDe::TakeControlUntilEndOfTurn => CardEffect::TakeControlUntilEndOfTurn,
            CardEffectDe::TakeControl => CardEffect::TakeControl,
            CardEffectDe::Corrupt => CardEffect::Corrupt,
            CardEffectDe::MinHealthUntilEndOfTurn => CardEffect::MinHealthUntilEndOfTurn,
            CardEffectDe::TransformToRandom { card_a, card_b } => CardEffect::TransformToRandom {
                card_a: intern(card_a)?,
                card_b: intern(card_b)?,
            },
            CardEffectDe::AddRandomCardToHand { pool } => CardEffect::AddRandomCardToHand { pool },
            CardEffectDe::SummonRandomMinion { pool } => CardEffect::SummonRandomMinion { pool },
            CardEffectDe::AddCardToHand { card_id } => CardEffect::AddCardToHand {
                card_id: intern(card_id)?,
            },
            CardEffectDe::DealDamageAndSummonIfKilled { amount, pool } => {
                CardEffect::DealDamageAndSummonIfKilled { amount, pool }
            }
        })
    }
}

/// Random pool type — Tier 3 random generation.
///
/// Satisfies pool closure: every sampling pool is a filtered subset of the Classic pool
/// (see `src/cards/pool.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RandomPool {
    /// A random Legendary minion (Brightwing)
    Legendary,
    /// A random Beast (Barrens Stablehand)
    Beast,
    /// A random Mage spell (Tome of Intellect)
    MageSpell,
    /// A random shadow spell (Xavius)
    ShadowSpell,
    /// A random Demon (Voidcaller/Bane of Doom)
    Demon,
    /// A random card of another class (Pilfer — simplified to non-Rogue cards)
    OtherClass,
    /// A random Dream card (Ysera)
    Dream,
    /// A random Animal Companion (Huffer/Leokk/Misha)
    Companion,
}
