//! Card effect definitions — compile-time constant CardEffect and EffectTarget.
//!
//! Phase 2 card effects: damage, card draw, summon, buff.
//! Effects are stored as `Copy` enum constants in `CardDef` and the `Battlecry`/`Deathrattle` components.
use serde::{Deserialize, Serialize};

use crate::core::component::{CardType, DarkGiftKind, ImbueClass, LeylineUpgrade};

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
    /// A random friendly minion other than the effect source (Young Priestess)
    OtherFriendlyMinion,
    /// All friendly minions other than the effect source (2025–2026
    /// expansions M4-W4 — CATA_133 Iridescent Flitterwing's end-of-turn
    /// "give your other minions +1/+1")
    AllOtherFriendlyMinions,
    /// A random enemy Taunt minion
    TauntEnemyMinion,
    /// The entity the triggering event happened to (Sword of Justice — the
    /// just-summoned minion). Resolved from the trigger's event subject; a
    /// no-op when the subject is gone (dead / left play).
    EventSubject,
    /// A random friendly minion of the given race (Houndmaster — a friendly Beast)
    FriendlyRace(crate::core::component::Race),
    /// All friendly minions of the given race, excluding the effect source
    /// (Coldlight Seer — all other Murlocs)
    AllOtherFriendlyRace(crate::core::component::Race),
    /// A random minion of the given race on either side of the board
    /// (Hungry Crab — destroy a Murloc)
    AnyRace(crate::core::component::Race),
    /// A random enemy minion with attack ≤ N (Stampeding Kodo)
    EnemyMinionAttackLE(i32),
    /// A random minion on either side with attack ≥ N (Big Game Hunter)
    AnyMinionAttackGE(i32),
    /// A random minion on either side with attack ≤ N (Twilight Influence —
    /// "destroy a minion with 3 or less Attack" targets either side)
    AnyMinionAttackLE(i32),
    /// A random damaged friendly minion (Rampage)
    DamagedFriendlyMinion,
    /// A random damaged minion on either side (Rampage)
    DamagedMinion,
    /// A random minion on either side (Crazed Alchemist, Ancestral Healing)
    AnyMinion,
    /// A single character (hero or minion) on either side — Stormpike
    /// Commando, Elven Archer, Fire Elemental, SI:7 Agent, Earthen Ring
    /// Farseer, Voodoo Doctor
    AnyCharacter,
    /// Either hero — Alexstrasza
    AnyHero,
    /// A random ENEMY minion with attack ≥ N (Big Game Hunter — enemy-only,
    /// mirroring `EnemyMinionAttackLE`)
    EnemyMinionAttackGE(i32),
    /// All friendly characters, hero included (Darkscale Healer)
    AllFriendlyCharacters,
    /// A random friendly DAMAGED minion other than the source (Stonecarver
    /// — 2025–2026 expansions M2-W4a, "another friendly damaged minion")
    DamagedOtherFriendlyMinion,
    /// A random enemy minion with a minion type (Bugsquasher —
    /// 2025–2026 expansions M2-W4a, "an enemy minion with a minion type")
    EnemyMinionWithRace,
}

/// A keyword a `GrantKeyword` effect can bestow (2025–2026 expansions
/// M2-W4a — the choose-one branches of Ancient Stegodon / Ancient Raptor /
/// Ancient Pterrordax and Resuscitate's Reborn grant).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeywordKind {
    /// Taunt
    Taunt,
    /// Poisonous
    Poisonous,
    /// Elusive
    Elusive,
    /// Reborn
    Reborn,
    /// Stealth
    Stealth,
    /// Windfury (M2-W4a — Ancient Pterrordax's third choose-one branch)
    Windfury,
    /// Lifesteal (2025–2026 expansions M3-W2b — TIME_619 Talanji of the
    /// Graves' Boon of Longevity for Bwonsamdi)
    Lifesteal,
    /// Rush (2025–2026 expansions M3-W2b — TIME_619 Talanji of the Graves'
    /// Boon of Speed for Bwonsamdi)
    Rush,
    /// Divine Shield (2025–2026 expansions M4-W1 — Chromatus's Bronze
    /// Head: the head carries Divine Shield and its deathrattle removes
    /// the keyword from the main Chromatus)
    DivineShield,
}

/// Discover pool — the source of a `DiscoverPool` discover (2025–2026
/// expansions M2-W4a — the D2 random simplification, fidelity-debt §17):
/// every pool maps to a filtered sampling procedure in
/// `cards::pool::discover_pool_cards`, mirroring the RandomPool pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscoverPool {
    /// A Legendary minion (Merchant of Legend)
    LegendaryMinion,
    /// An Undead (Paleomancy)
    UndeadMinion,
    /// A Frost Rune card (Crypt Map — the fixed W1–W3 Frost Rune table)
    FrostRune,
    /// A Murloc (Submerged Map)
    Murloc,
    /// A minion with a minion type the player hasn't played this game
    /// (Mountain Map)
    MinionOfUnplayedType,
    /// A Beast with odd Attack (Odd Map)
    BeastOddAttack,
    /// A Fel spell (Hive Map)
    FelSpell,
    /// A spell from any class that costs (8) or more (Relic of Kings)
    SpellCostGE8,
    /// A card with Cost equal to the player's remaining Mana Crystals
    /// (Scrappy Scavenger — the pool is resolved at call time)
    CostEqualRemainingMana,
    /// A Temporary 1-Cost minion (Bloodpetal Biome)
    TemporaryOneCostMinion,
    /// A spell that costs (7) or more from the past (2025–2026 expansions
    /// M3-W2a — TIME_704 Highborne Mentor; the SpellCostGE8 precedent)
    SpellCostGE7,
    /// An Arcane spell from the past (2025–2026 expansions M3-W2a —
    /// TIME_857 Alter Time; the school filter is the FelSpell precedent)
    ArcaneSpell,
    /// A Paladin Mech from the past (2025–2026 expansions M3-W2a —
    /// TIME_016 Neon Innovation; Paladin class Mechs, §20)
    PaladinMech,
    /// Any spell from the past (2025–2026 expansions M3-W2a — TIME_612
    /// Blood Draw; the pool spans the whole active window, §20)
    Spell,
    /// A Nature spell from the past (2025–2026 expansions M3-W2b —
    /// TIME_013 Chromie's discovery branch; the school filter is the
    /// ArcaneSpell precedent)
    NatureSpell,
    /// Any Demon costing (5) or more from the past (2025–2026 expansions
    /// M3-W2b — TIME_446 The Eternal Hold; full-catalog like SpellCostGE8)
    DemonCostGE5,
    /// A spell that costs (1) (2025–2026 expansions M4-W4 — CATA_484
    /// Winter's Answer; the pool spans the whole active window, §26)
    OneCostSpell,
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
    /// Set health to a fixed value (Alexstrasza — a hero's Health to 15)
    SetHealth {
        /// Target health value
        health: i32,
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
    /// Gain stats AND Windfury in one effect (Raging Worgen's Enrage — +1
    /// Attack and Windfury while damaged)
    GainStatsAndGrantWindfury {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
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
    /// Discard the whole hand (Deathwing's battlecry — combined with the
    /// destroy-all-other-minions part in DestroyAllOtherMinionsAndDiscardHand)
    DiscardHand,
    /// The next spell cast this turn costs this much less (Preparation)
    NextSpellDiscount {
        /// Discount amount
        amount: i32,
    },
    /// Give the source's adjacent minions stats and Divine Shield (Defender
    /// of Argus — +1/+1 and Divine Shield)
    GrantAdjacentStatsAndDivineShield {
        /// Attack gain
        attack: i32,
        /// Health gain
        health: i32,
    },
    /// Destroy all other minions and discard your hand (Deathwing)
    DestroyAllOtherMinionsAndDiscardHand,
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
        /// Target selection (Bestial Wrath — a friendly Beast)
        target: EffectTarget,
    },
    /// Prevent the fatal damage that triggered this secret and become Immune
    /// until the end of the turn (Ice Block — the hero survives at 1 health)
    PreventFatalDamageAndImmune,
    /// Temporarily take control of an enemy minion until end of turn (Shadow Madness, attack ≤ 3)
    TakeControlUntilEndOfTurn,
    /// Permanently take control of an enemy minion (Mind Control)
    TakeControl,
    /// Take permanent control of a random enemy minion with at most this
    /// attack (Cabal Shadow Priest — attack ≤ 2)
    TakeControlAttackLE {
        /// Maximum attack of the controlled minion
        max_attack: i32,
    },
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
    /// Discover over the top 3 cards of the deck (Tracking): pick one into
    /// hand, discard the rest
    DiscoverDeckTop3,
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
    /// Draw cards of the given race from the deck (Sense Demons — draw two Demons)
    DrawCardByRace {
        /// Number of cards to draw
        count: u32,
        /// Required race of the drawn cards
        race: crate::core::component::Race,
    },
    /// Deal damage to a minion; if it is a friendly Demon, buff it instead
    /// (Demonfire — 2 damage, or +2/+2 to a friendly Demon)
    Demonfire {
        /// Damage amount (non-Demon targets)
        damage: i32,
        /// Attack gained by a friendly Demon target
        attack_bonus: i32,
        /// Health gained by a friendly Demon target
        health_bonus: i32,
    },
    /// Gain stats AND grant Taunt to the target (Houndmaster — +2/+2 and Taunt)
    GainStatsAndTaunt {
        /// Attack gain
        attack: i32,
        /// Health gain
        health: i32,
        /// Target scope
        target: EffectTarget,
    },
    /// Destroy a minion of the target scope, then gain fixed stats
    /// (Hungry Crab — destroy a Murloc and gain +2/+2)
    DestroyAndGainStats {
        /// Attack gained by the source
        attack: i32,
        /// Health gained by the source
        health: i32,
        /// Destroy target scope
        target: EffectTarget,
    },
    /// Destroy one random enemy Secret (SI:7 Infiltrator)
    DestroyRandomEnemySecret,
    /// Destroy ALL enemy Secrets and gain stats (Eater of Secrets)
    DestroyAllEnemySecretsAndGainStats {
        /// Attack gained by the source
        attack: i32,
        /// Health gained by the source
        health: i32,
    },
    /// Destroy ALL enemy Secrets and draw cards (Flare)
    DestroyAllEnemySecretsAndDraw {
        /// Number of cards to draw
        count: u32,
    },
    /// Give a minion "Whenever this minion attacks, draw a card"
    /// (Blessing of Wisdom — attaches an attack trigger to the target)
    AttachAttackDraw {
        /// Number of cards drawn per attack
        count: u32,
    },
    /// Gain stats equal to the number of cards in hand (Twilight Drake —
    /// +1 Health per card in hand)
    GainStatsPerHandCard {
        /// Flat attack gain
        attack: i32,
        /// Health gained per hand card
        health_per_card: i32,
    },
    /// Gain stats per OTHER friendly minion on the board (Frostwolf Warlord
    /// — +1/+1 for each other friendly minion; the source is excluded)
    GainStatsPerFriendlyMinion {
        /// Attack gained per other friendly minion
        attack: i32,
        /// Health gained per other friendly minion
        health_per_minion: i32,
    },
    /// Deal N damage COUNT times, each ping at a random character of the
    /// target scope (Mad Bomber — 3 random 1-damage pings across all OTHER
    /// characters; the same target can be hit repeatedly)
    DealDamageRandomly {
        /// Damage per ping
        amount: i32,
        /// Number of pings
        count: i32,
        /// Target scope
        target: EffectTarget,
    },
    /// Deal damage; boosted when the caster's hero has ≤ N health
    /// (Mortal Strike — 4 damage, or 6 at 12 or less health)
    MortalStrike {
        /// Normal damage
        damage: i32,
        /// Damage at the health threshold
        boosted: i32,
        /// Hero-health threshold
        threshold: i32,
    },
    /// Draw a card for each damaged friendly character (Battle Rage)
    DrawPerDamagedFriendlyCharacter,
    /// Gain stats at end of turn only while the owner controls a Secret
    /// (Ethereal Arcanist)
    GainStatsIfOwnSecret {
        /// Attack gain
        attack: i32,
        /// Health gain
        health: i32,
    },
    /// Absorb all Divine Shields on the board and gain stats per shield
    /// (Blood Knight — +3/+3 for each destroyed shield)
    AbsorbDivineShields {
        /// Attack gained per absorbed shield
        attack_per_shield: i32,
        /// Health gained per absorbed shield
        health_per_shield: i32,
    },
    /// Remove durability from the opponent's weapon (Bloodsail Corsair —
    /// remove 1 Durability); a weapon at 0 durability is destroyed.
    RemoveWeaponDurability {
        /// Durability removed
        amount: i32,
    },
    /// Gain attack equal to the owner's weapon attack (Bloodsail Raider)
    GainAttackEqualToWeapon,
    /// The opponent's spells cost 0 next turn (Millhouse Manastorm)
    EnemySpellsCostZero,
    /// Give the opponent an empty mana crystal (Arcane Golem)
    GiveOpponentManaCrystal {
        /// Number of crystals
        count: i32,
    },
    /// Set a played enemy minion's health to a value (Repentance — 1);
    /// resolved with the secret event's played minion
    SetPlayedMinionHealth {
        /// Health value the minion is set to
        health: i32,
    },
    /// Silence all enemy minions and draw cards (Mass Dispel)
    SilenceAllEnemyMinionsAndDraw {
        /// Number of cards to draw
        count: u32,
    },
    /// Swap a minion's Attack and Health (Crazed Alchemist)
    SwapAttackAndHealth {
        /// Target scope
        target: EffectTarget,
    },
    /// Freeze a random enemy minion and its neighbors (Cone of Cold)
    FreezeAdjacent,
    /// Give the source's adjacent minions Taunt (Sunfury Protector)
    GrantAdjacentTaunt,
    /// Give the source's adjacent minions Spell Damage (Ancient Mage)
    GrantAdjacentSpellDamage {
        /// Spell damage granted
        amount: i32,
    },
    /// Restore a minion to full Health and give it Taunt (Ancestral Healing)
    FullHealAndTaunt {
        /// Target scope
        target: EffectTarget,
    },
    /// Draw a card with the given probability at the end of the owner's turn
    /// (Nat Pagle — 50% chance)
    ChanceDraw {
        /// Draw chance in percent (0-100)
        percent: u32,
    },
    /// Gain stats THIS TURN (Mana Addict — +2 Attack this turn after casting
    /// a spell; the enchantment expires at the end of the turn)
    GainStatsThisTurn {
        /// Attack gain
        attack: i32,
        /// Health gain
        health: i32,
        /// Target scope
        target: EffectTarget,
    },
    /// Give all friendly minions Divine Shield (Righteousness)
    GrantDivineShieldAllFriendly,
    /// Give a friendly minion Divine Shield (Argent Protector — Battlecry)
    GrantDivineShield {
        /// Target selection
        target: EffectTarget,
    },
    /// Deal damage to all characters except Ysera (Ysera Awakens — a Dream
    /// card that spares its generator)
    YseraAwakens {
        /// Damage amount
        damage: i32,
    },
    /// Give all friendly minions stats AND Taunt (Gift of the Wild)
    GainStatsAndTauntAllFriendly {
        /// Attack gain
        attack: i32,
        /// Health gain
        health: i32,
    },
    /// Draw a card and deal damage equal to its mana cost (Holy Wrath)
    DrawAndDamageByCost,
    /// Restore health to a random damaged friendly character (Lightwell)
    RestoreDamagedFriendly {
        /// Amount to restore
        amount: i32,
    },
    /// Swap this minion with a random minion in your hand (Alarm-o-Bot)
    SwapWithHandMinion,
    /// Resummon the minion that just died with 1 Health (Redemption — a
    /// secret effect resolved with the death event)
    ResurrectDiedMinion,
    // ----------------------------------------------------------------
    // Pool-open effects (pool-open-cards roadmap M1) — read the
    // opponent's actual zones instead of sampling a pool. Enforced to
    // appear only on cards registered in `sets::POOL_OPEN_CARDS` by
    // `pool_open_effects_require_registry` (see docs/pool-openness.md).
    // ----------------------------------------------------------------
    /// Copy `count` random cards from the enemy hand into this player's
    /// hand (Mind Vision — sampling without replacement over hand entities)
    CopyRandomEnemyHandCard {
        /// Number of cards to copy
        count: u32,
    },
    /// Copy `count` random cards from the enemy deck into this player's
    /// hand (Thoughtsteal — sampling without replacement over deck
    /// entities; two copies of the same card are two distinct entities
    /// and may both be picked, the same entity may not)
    CopyRandomEnemyDeckCards {
        /// Number of cards to copy
        count: u32,
    },
    /// Summon a copy of a random minion from the enemy deck (Mindgames).
    /// The enemy deck is not modified; when the deck holds no minions,
    /// the `fallback_card_id` token is summoned instead (Shadow of Nothing)
    SummonRandomEnemyDeckMinion {
        /// Token card ID to summon when the enemy deck has no minions
        fallback_card_id: &'static str,
    },
    /// Copy the just-cast spell into the other player's hand (Lorewalker
    /// Cho — the subject is the cast spell; the copy goes to the caster's
    /// opponent, whoever that is)
    CopyCastSpellToOtherPlayerHand,
    // ----------------------------------------------------------------
    // Core Set W1 effects (core-set-roadmap W1) — attack-pipeline
    // primitives. RUSH / LIFESTEAL / REBORN themselves are components
    // applied by `apply_card_keywords`; these are the scripted effects
    // the W1 cards need.
    // ----------------------------------------------------------------
    /// Fill the owner's hand with the given card (Halazzi, the Lynx — 1/1
    /// Lynxes with Rush; stops at the 10-card hand cap, F-A11)
    FillHandWithMinion {
        /// Card ID of the token to fill the hand with
        card_id: &'static str,
    },
    /// Force all enemy minions that can attack to attack this character
    /// (Mythical Terror — end-of-turn; ignores Taunt, respects Frozen and
    /// exhausted attackers)
    ForceEnemyMinionsAttackThis,
    /// Spend `cost` corpses to summon a copy of this minion (Malignant
    /// Horror — end-of-turn; does nothing with fewer corpses)
    SpendCorpsesSummonCopy {
        /// Corpses to spend
        cost: u32,
    },
    // ----------------------------------------------------------------
    // Core Set W2 effects (core-set-roadmap W2) — hand/spell-pipeline
    // primitives: TRADEABLE / OUTCAST / spell-power exemption. TRADEABLE
    // itself is a component (`Action::TradeCard`); OUTCAST effects are
    // these two variants, resolved against the OutcastPlayed marker.
    // ----------------------------------------------------------------
    /// Draw `normal` cards, or `outcast` when the card was played from the
    /// leftmost/rightmost hand position (Spectral Sight 1/2, Crimson Sigil
    /// Runner 0/1 — Core Set W2)
    DrawCardOutcast {
        /// Cards drawn in the normal case
        normal: u32,
        /// Cards drawn when played from the hand edge (Outcast)
        outcast: u32,
    },
    /// Deal `amount` damage, or `outcast_amount` when played from the hand
    /// edge (Eye Beam 3/6 — Core Set W2; the Lifesteal half is the W1
    /// component)
    OutcastDamage {
        /// Damage in the normal case
        amount: i32,
        /// Damage when played from the hand edge (Outcast)
        outcast_amount: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Restore `amount` health in 1-point pings randomly spread across all
    /// friendly characters, hero included (Healing Rain — Core Set W2;
    /// overhealing is wasted, matching HS)
    RestoreRandomFriendly {
        /// Total health restored (as 1-point pings)
        amount: i32,
    },
    /// Destroy an enemy location (Demolition Renovator — Core Set W2; the
    /// engine has no Location card type until W8, so the effect resolves
    /// against no targets and fizzles for now)
    DestroyEnemyLocation,
    /// Deal damage to the just-played enemy minion, excess carries to the
    /// enemy hero (Explosive Runes — Core Set W2; resolved by the secret
    /// system with the played-minion event)
    DamagePlayedMinionAndExcess {
        /// Damage to the played minion
        amount: i32,
    },
    /// Deal damage; when the caster's hand is empty, also draw a card
    /// (Quick Shot — Core Set W2; the official AFFECTED_BY_SPELL_POWER
    /// marker needs no special handling — spells are spell-powered by
    /// default in this engine)
    DamageAndDrawIfHandEmpty {
        /// Damage amount
        damage: i32,
        /// Target selection
        target: EffectTarget,
    },
    // ----------------------------------------------------------------
    // Core Set W3a effects (core-set-roadmap W3a) — faithful versions of
    // Classic cards the Classic pool simplified: Holy Nova's heal half,
    // Slam's survive-conditioned draw, Shield Block's draw, Mortal Coil's
    // kill-conditioned draw, plus Bash's damage+armor and Hex's real
    // transform.
    // ----------------------------------------------------------------
    /// Deal `damage` to all enemy minions and restore `heal` to all
    /// friendly characters, hero included (Holy Nova — Core Set W3a)
    AoeDamageAndHealFriendly {
        /// Damage to all enemy minions
        damage: i32,
        /// Heal to all friendly characters
        heal: i32,
    },
    /// Deal damage; when the target SURVIVES, draw a card (Slam — Core
    /// Set W3a; the Classic pool's unconditional draw is a simplification)
    DamageAndDrawIfSurvives {
        /// Damage amount
        damage: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Gain armor and draw cards (Shield Block — Core Set W3a; the
    /// Classic pool's armor-only version is a simplification)
    GainArmorAndDraw {
        /// Armor gained
        armor: i32,
        /// Cards drawn
        draw: u32,
    },
    /// Deal damage; when the target DIES, draw a card (Mortal Coil —
    /// Core Set W3a; the Classic pool's damage-only version is a
    /// simplification)
    DamageAndDrawIfKilled {
        /// Damage amount
        damage: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Deal damage and gain armor (Bash — Core Set W3a)
    DamageAndGainArmor {
        /// Damage amount
        damage: i32,
        /// Armor gained by the friendly hero
        armor: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Give a friendly Undead minion Poisonous (Poison Breath — Core Set
    /// W3a)
    GainPoisonousToFriendlyUndead,
    /// Transform a minion into the given card (Hex — Core Set W3a; the
    /// Classic pool's Polymorph-as-destroy is a simplification — a real
    /// transform clears effects and does not trigger deathrattles)
    TransformToMinion {
        /// Card ID of the transform target (the 0/1 Frog)
        card_id: &'static str,
    },
    /// Give the target minion "Deathrattle: summon the given card"
    /// (Spikeridged Steed — Core Set W3a, a single-target version of
    /// GrantDeathrattleAll)
    GrantDeathrattleToTarget {
        /// Card ID summoned on deathrattle
        card_id: &'static str,
    },
    /// Destroy all minions with at least this Attack (Shadow Word: Ruin —
    /// Core Set W3a; both sides)
    DestroyAllMinionsAttackGE {
        /// Minimum attack of destroyed minions
        attack: i32,
    },
    /// Deal damage to all enemy minions and draw cards (Fan of Knives —
    /// Core Set W3a)
    AoeDamageAndDraw {
        /// Damage to all enemy minions
        damage: i32,
        /// Cards drawn
        draw: u32,
    },
    /// Give the target +attack/+health, Taunt, and "Deathrattle: summon
    /// the given card" (Spikeridged Steed — Core Set W3a)
    GainStatsTauntAndDeathrattle {
        /// Attack gained
        attack: i32,
        /// Health gained
        health: i32,
        /// Card ID summoned on deathrattle
        card_id: &'static str,
    },
    // ----------------------------------------------------------------
    // Core Set W3a part 2 (core-set-roadmap W3a) — the complex batch:
    // Finja/Shaku attack triggers, Merch Seller's deck-topping spell,
    // Immortalized in Stone's statue trio. Noggenfogger's target
    // randomization, Khadgar's summon doubling and Death Metal Knight's
    // health payment are engine hooks, not effects.
    // ----------------------------------------------------------------
    /// Summon a random Murloc from the owner's deck when THIS minion
    /// attacks (Finja — Core Set W3a; resolved by the attack trigger, the
    /// event subject must be the source)
    SummonRandomFishFromDeck,
    /// Put a random spell on top of the opponent's deck (Merch Seller —
    /// Core Set W3a; end-of-turn effect)
    AddRandomSpellToOpponentDeckTop,
    /// Summon the three statues with Taunt: a 4/8, a 2/4 and a 1/2
    /// Elemental (Immortalized in Stone — Core Set W3a)
    SummonStatueTrio,
    /// Copy a random card from the opponent's deck to hand when THIS
    /// minion attacks (Shaku, the Collector — Core Set W3a; pool-open, the
    /// event subject must be the source)
    CopyEnemyDeckCardOnSelfAttack,
    // ----------------------------------------------------------------
    // Core Set W3b effects (core-set-roadmap W3b) — 27 confirmed cards.
    // ----------------------------------------------------------------
    /// Deal damage and summon the given token (Wound Prey — 1 damage +
    /// a 1/1 Hyena with Rush)
    DamageAndSummon {
        /// Damage amount
        damage: i32,
        /// Target selection
        target: EffectTarget,
        /// Token card ID to summon
        card_id: &'static str,
    },
    /// Get a Lightning Bolt when THIS or an ADJACENT friendly minion
    /// attacks (Rehgar Earthfury — the event subject must be self or a
    /// neighbor)
    RehgarBolt,
    /// Deal damage to two random enemy minions; draw a card for each that
    /// dies (Consumption — prediction-based like Bane of Doom)
    DamageTwoDrawIfKilled {
        /// Damage per target
        damage: i32,
    },
    /// Freeze a character and Discover a spell (Death's Advance — the
    /// Discover surfaces as a pending choice; the default policy picks
    /// randomly)
    FreezeAndDiscoverSpell,
    /// Gain +1/+1 when the OWNER'S HERO attacked (Hench-Clan Thug — the
    /// event subject must be the owner's hero)
    HenchThugBuff,
    /// Summon three 1/1 Silver Hand Recruits and equip a 1/4 weapon
    /// (Muster for Battle)
    SummonRecruitsAndEquipWeapon,
    /// Give a minion +2/+2 and summon a random 2-cost minion (Silvermoon
    /// Portal)
    BuffAndSummonRandomCost2,
    /// Deal damage to a minion; when it dies, summon a fresh copy of it
    /// (Initiation — prediction-based)
    DamageAndSummonCopyIfKilled {
        /// Damage amount
        damage: i32,
    },
    /// When the opponent draws a card, add a 1-cost copy of it to hand
    /// (Keymaster Alabaster)
    KeymasterCopy,
    /// When a friendly minion loses Divine Shield, give a minion in hand
    /// +5/+5 (Highlord Fordragon — resolved by the DivineShieldLost event)
    FordragonBuff,
    /// Deal damage and summon two 1/3 Voidwalkers with Taunt (Demonic
    /// Assault)
    DamageAndSummonVoidwalkers {
        /// Damage amount
        damage: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Deal damage to a minion and add the given card to hand (First
    /// Flame — Second Flame)
    DamageAndAddToHand {
        /// Damage amount
        damage: i32,
        /// Card ID added to hand
        card_id: &'static str,
    },
    /// Add `count` random spells costing at least `min_cost` from OTHER
    /// classes to hand (Jackpot!)
    AddRandomOtherClassSpells {
        /// Number of spells
        count: u32,
        /// Minimum cost
        min_cost: i32,
    },
    /// When the owner draws a card, summon a 1/1 Demon with Rush (Eredar
    /// Deceptor)
    SummonFelbatOnDraw,
    /// Spend up to `max` corpses to summon a random minion of that cost
    /// (Corpse Farm)
    SpendCorpsesSummonRandomMinion {
        /// Maximum corpses spent
        max: u32,
    },
    // ----------------------------------------------------------------
    // Core Set W3c effects (core-set-roadmap W3c) — 38 cards.
    // ----------------------------------------------------------------
    /// Give a minion +attack/+health and draw (Power Word: Shield, Hand of
    /// A'dal — the Classic pool's buff-only versions were simplifications)
    GainStatsAndDraw {
        /// Attack gained
        attack: i32,
        /// Health gained
        health: i32,
        /// Target selection
        target: EffectTarget,
        /// Cards drawn
        draw: u32,
    },
    /// Deal damage to an UNDAMAGED minion only (Backstab — the Classic
    /// pool's unconditional version was a simplification)
    DamageUndamaged {
        /// Damage amount
        damage: i32,
    },
    /// Deal damage to a minion AND the caster's hero (Spirit Bomb)
    DamageMinionAndSelfHero {
        /// Damage to both
        damage: i32,
    },
    /// Give the hero +attack this turn and draw (Chaos Strike)
    GainHeroAttackAndDraw {
        /// Attack this turn
        attack: i32,
    },
    /// Gain armor and summon a random minion costing at most `max_cost`
    /// from the deck (Oaken Summons)
    GainArmorAndSummonDeckMinion {
        /// Armor gained
        armor: i32,
        /// Maximum cost of the summoned minion
        max_cost: i32,
    },
    /// Gain armor and draw when the OWNER'S HERO attacks (Hookfist-3000)
    GainArmorAndDrawOnHeroAttack {
        /// Armor gained
        armor: i32,
    },
    /// Summon all three Animal Companions (Call of the Wild)
    SummonAllCompanions,
    /// Deal damage to a character, freeze all enemy minions and summon a
    /// 5/5 Frostwyrm (Frostwyrm's Fury)
    DamageFreezeAllAndSummon {
        /// Damage to the target
        damage: i32,
        /// Token card ID (the Frostwyrm)
        card_id: &'static str,
    },
    /// Destroy the enemy minion with the highest Attack (Asphyxiate; ties
    /// resolve randomly)
    DestroyHighestAttackEnemy,
    /// Summon two 2/2 Zombies with Taunt; spend `corpses` to give them
    /// Reborn (Tomb Guardians)
    SummonZombiesWithCorpseReborn {
        /// Corpses spent for Reborn
        corpses: u32,
    },
    /// Transform this hand card into a copy of the just-cast spell
    /// (Shadow of Demise)
    TransformSelfToCastSpell,
    /// Give all minions in hand +1/+1; spend `corpses` for another +1/+1
    /// (Blood Tap)
    BuffHandMinionsWithCorpses {
        /// Corpses spent for the extra buff
        corpses: u32,
    },
    /// Restore health and draw (Flash of Light)
    RestoreHealthAndDraw {
        /// Health restored
        amount: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Draw a card at end of turn when the owner has unspent mana
    /// (Crystal Merchant)
    DrawIfUnspentMana,
    /// Gain armor and summon a random minion of exactly `cost` (Ironforge
    /// Portal)
    GainArmorAndSummonRandomCost {
        /// Armor gained
        armor: i32,
        /// Cost of the summoned minion
        cost: i32,
    },
    // ----------------------------------------------------------------
    // Core Set W4a effects (core-set-roadmap W4a) — the battlecry batch.
    // Discover cards are simplified to random generation (the engine has no
    // Discover system — registered in the fidelity ledger).
    // ----------------------------------------------------------------
    /// Gain +attack/+health when the owner restored Health this turn (Priest
    /// of An'she)
    GainStatsIfHealedThisTurn {
        /// Attack gained
        attack: i32,
        /// Health gained
        health: i32,
    },
    /// Battle the chosen enemy minion to the death (Warmaul Challenger —
    /// both deal their attack to each other)
    BattleToTheDeath,
    /// The next Demon played costs less (Raging Felscreamer)
    NextDemonDiscount {
        /// Cost reduction
        amount: i32,
    },
    /// Give all minions in hand +attack/+health (Grimestreet Outfitter)
    BuffHandMinions {
        /// Attack gained
        attack: i32,
        /// Health gained
        health: i32,
    },
    /// The opponent summons a random minion from THEIR HAND (Dirty Rat —
    /// pool-open)
    SummonRandomEnemyHandMinion,
    /// Each player draws a card (Prize Vendor)
    DrawForBoth,
    /// The next Combo card costs less this turn (Foxy Fraud)
    NextComboDiscount {
        /// Cost reduction
        amount: i32,
    },
    /// Add random Mage spells to hand (Babbling Bookcase)
    AddRandomMageSpells {
        /// Number of spells
        count: u32,
    },
    /// Deal damage to the enemy hero and restore to the friendly hero
    /// (Lifedrinker)
    DamageEnemyHeroAndHealSelf {
        /// Damage and heal amount
        amount: i32,
    },
    /// Lose 1 Health for each card in the opponent's hand (Witchwood
    /// Grizzly — reads the opponent's hand count only)
    LoseHealthPerOpponentHandCard,
    /// Give a random friendly minion Divine Shield and Taunt (Coghammer)
    GrantRandomFriendlyDivineShieldTaunt,
    /// Remove the top card of the opponent's deck (Gnomeferatu — pool-open)
    RemoveTopEnemyDeckCard,
    /// Discover a spell and restore health equal to its cost (Ivory Knight
    /// — the Discover is simplified to a random spell)
    DiscoverSpellAndHealCost,
    /// Draw a Beast, a Dragon and a Murloc from the deck (The Curator)
    DrawBeastDragonMurloc,
    /// Add a random card from another class to hand (Swashburglar)
    AddRandomOtherClassCard,
    /// Add a random Shaman spell to hand (Witch's Apprentice)
    AddRandomShamanSpell,
    /// Deal damage to the friendly hero (Vulgar Homunculus)
    DamageSelfHero {
        /// Damage amount
        damage: i32,
    },
    /// Summon two copies of this minion (Nerubian Swarmguard)
    SummonTwoCopiesOfSelf,
    /// Spend up to `max` corpses; deal `damage` to a random enemy for each
    /// (Marrow Manipulator)
    SpendCorpsesDamageRandom {
        /// Maximum corpses
        max: u32,
        /// Damage per corpse
        damage: i32,
    },
    /// Raise up to `max` corpses as 1/3 Risen Footmen with Taunt (Boneguard
    /// Commander)
    SpendCorpsesSummonFootmen {
        /// Maximum corpses
        max: u32,
    },
    /// For the rest of the game, deal damage to the opponent at the end of
    /// your turns (Alexandros Mograine)
    OngoingEndTurnDamage {
        /// Damage to the opponent
        damage: i32,
    },
    /// Deal damage to all OTHER minions (Primordial Drake)
    DamageAllOtherMinions {
        /// Damage amount
        damage: i32,
    },
    /// Give Taunt minions in hand +attack/+health (Detonation Juggernaut)
    BuffTauntHandMinions {
        /// Attack gained
        attack: i32,
        /// Health gained
        health: i32,
    },
    // ----------------------------------------------------------------
    // Core Set W4b effects (core-set-roadmap W4b) — the second battlecry
    // batch. Simplified where flagged: no choice system (Siamat random),
    // no pack-opening flow (Avatar adds cards to hand), Steamcleaner has no
    // card-origin tracking (no-op).
    // ----------------------------------------------------------------
    /// Summon a random minion with Cost equal to the owner's hand size
    /// (Astromancer)
    SummonRandomMinionCostEqHandSize,
    /// Resurrect the highest-Cost friendly minion that died this game
    /// (Calia Menethil)
    ResurrectHighestCostFallen,
    /// Give other friendly minions "Deathrattle: summon a random minion of
    /// this minion's Cost" (Ulfar)
    GrantDeathrattleSummonOwnCost,
    /// Add a random Pirate to hand (Sky Raider)
    AddRandomPirateToHand,
    /// The opponent's next Hero Power costs more (Blowtorch Saboteur)
    NextEnemyHeroPowerCostMore {
        /// Cost increase
        amount: i32,
    },
    /// Summon a random minion of the given cost (Maze Guide)
    SummonRandomMinionOfCost {
        /// Cost of the summoned minion
        cost: i32,
    },
    /// Summon a random Demon from hand and deck (Archwitch Willow)
    SummonRandomDemonFromHandOrDeck,
    /// The opponent's spells cost more next turn (Cult Neophyte)
    NextEnemySpellsCostMore {
        /// Cost increase
        amount: i32,
    },
    /// If the owner controls a Beast, buff the weapon durability (Headhunter's
    /// Hatchet)
    BuffWeaponDurabilityIfBeast {
        /// Durability gained
        amount: i32,
    },
    /// Return all spells played last turn to hand (Krag'wa, the Frog)
    ReturnLastTurnSpells,
    /// Destroy a minion; the hero takes damage equal to its Health
    /// (Riftcleaver)
    DestroyMinionAndSelfDamage,
    /// Deal damage to this minion (Injured Tol'vir)
    DamageSelfMinion {
        /// Damage amount
        damage: i32,
    },
    /// Add a random 1-Cost card to hand (Dark Peddler — Discover simplified)
    AddRandomOneCostCard,
    /// Give 3 random friendly minions of different races +attack/+health
    /// (Menagerie Mug)
    BuffThreeDifferentRaces {
        /// Attack gained
        attack: i32,
        /// Health gained
        health: i32,
    },
    /// Add 5 random cards to hand (Avatar of Hearthstone — the pack-opening
    /// flow is simplified to card generation)
    AddFiveRandomCards,
    /// Discard two random cards (Doomguard)
    DiscardTwoRandomCards,
    // ----------------------------------------------------------------
    // Core Set W5 effects (core-set-roadmap W5) — deathrattle/secret/aura
    // batch. Simplifications flagged: Fandral's Choose-One combining is not
    // modelled (whiteboard 3/6), Frostmourne's weapon-kill tracking is
    // approximated by a generic resurrect, Vibrant Squirrel's acorn
    // draw-summon is approximated by shuffling inert acorns.
    // ----------------------------------------------------------------
    /// Deal damage to all minions when the owner holds a Dragon (Chillmaw)
    DamageAllMinionsIfHoldingDragon {
        /// Damage amount
        damage: i32,
    },
    /// Deal the source's Attack randomly split among all enemies
    /// (Augmented Porcupine)
    DamageAllEnemiesByAttack,
    /// Return a random friendly minion to hand and reduce its cost
    /// (Waggle Pick)
    ReturnRandomFriendlyAndReduceCost {
        /// Cost reduction
        amount: i32,
    },
    /// Give the source's Attack to a random friendly minion (Fiendish
    /// Servant)
    GrantAttackToRandomFriendly,
    /// Summon a random legendary minion (Sneed's Old Shredder)
    SummonRandomLegendaryMinion,
    /// Resurrect a random minion killed by this weapon this game
    /// (Frostmourne — approximated by the generic resurrect)
    ResurrectWeaponKilled,
    /// Destroy a random enemy minion (Pressure Plate secret effect)
    DestroyRandomEnemyMinion,
    /// Summon a 3/6 Water Elemental (Oasis Ally secret effect)
    SummonOasisWaterElemental,
    // ----------------------------------------------------------------
    // Core Set W6 effects (core-set-roadmap W6) — discover/choose-one/
    // combo/overload/freeze batch. Discover is simplified to random
    // generation (fidelity ledger).
    // ----------------------------------------------------------------
    /// Summon a random 8-Cost minion and Freeze it (Glaciate)
    SummonRandomCostAndFreeze {
        /// Cost of the summoned minion
        cost: i32,
    },
    /// Deal damage and add a random spell to hand (Runed Orb)
    DamageAndAddRandomSpell {
        /// Damage amount
        damage: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Freeze an enemy and summon two 3/6 Water Elementals (Deep Freeze)
    FreezeAndSummonElementals,
    /// Add a random Taunt minion to hand with +1/+2 (I Know a Guy —
    /// Discover simplified)
    AddRandomTauntBuffed,
    /// Add a random Battlecry minion to hand (Blazing Invocation —
    /// Discover simplified)
    AddRandomBattlecryMinion,
    /// Deal damage and freeze the target (Frostbolt — the Classic pool's
    /// damage-only version was a simplification)
    DamageAndFreeze {
        /// Damage amount
        damage: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Deal damage to all enemy minions and freeze them (Blizzard — the
    /// Classic pool's damage-only version was a simplification)
    DamageAllEnemyMinionsAndFreeze {
        /// Damage amount
        damage: i32,
    },
    /// Add a random Outcast card to hand and make the next Outcast card
    /// cost (1) less (Illidari Studies — Discover simplified to random
    /// generation)
    AddRandomOutcastCardNextCheaper,
    // ----------------------------------------------------------------
    // 2025–2026 expansions M1-W1 effects (exp_edr_w1) — the Emerald Dream
    // imbue mechanic: an imbue card increments the player's imbue count;
    // the first imbue replaces the hero power with the class's imbued form
    // (cost 2), later imbues scale the imbued powers' numbers (level =
    // imbue count, read at resolution time — the count is the single source
    // of truth). Imbued hero powers are spell-powered like any hero power
    // (the source is the hero entity, CardType::Hero).
    // ----------------------------------------------------------------
    /// Imbue: the player's imbue count +1; on the first imbue, a hero of one
    /// of the six imbuing classes gets its imbued hero power (cost 2)
    /// instead of the base one (EDR_226, EDR_227, EDR_449, EDR_451, EDR_800,
    /// EDR_845, EDR_852, EDR_970)
    ImbueHeroPower,
    /// The imbued hero power of the given class — resolved at level L = the
    /// owner's current imbue count (the six EDR_*p hero powers)
    ImbuedHeroPower {
        /// The hero power's class
        class: ImbueClass,
    },
    /// Free resolution of the owner's current hero power effect — no mana
    /// spent, the hero power is not marked used (Wisprider's "trigger it")
    UseHeroPower,
    /// Draw a Beast, then imbue (Exotic Houndmaster)
    DrawBeastAndImbue,
    /// Restore health to the friendly hero, draw a card, then imbue
    /// (Aspect's Embrace)
    RestoreAndDrawAndImbue {
        /// Health restored
        amount: i32,
    },
    /// Summon a random 2-Cost minion, give it Taunt, then imbue (Aegis of
    /// Light)
    SummonRandomTwoCostTauntAndImbue,
    /// Imbue, then reduce the cost of a random minion in hand by (1)
    /// (Living Garden)
    ImbueAndReduceHandCost,
    /// Imbue, then trigger the (possibly just-replaced) hero power once
    /// (Wisprider)
    ImbueAndTriggerHeroPower,
    /// Imbue, then add a Wisp token to hand (Spirit Gatherer)
    ImbueAndGetWisp,
    /// Give all enemy minions -attack until your next turn, then imbue
    /// (Kaldorei Priestess — the TempDebuff precedent)
    ImbueAndDebuffEnemies {
        /// Attack reduction
        attack_reduction: i32,
    },
    /// If the player has imbued at least twice, deal damage to a minion
    /// (Resplendent Dreamweaver)
    DealDamageIfImbuedTwice {
        /// Damage amount
        damage: i32,
    },
    /// Add a random Wild God to hand; if the player has imbued at least 4
    /// times, set its Cost to (1) (Malorne the Waywatcher — Discover
    /// simplified to a random pick over the fixed WILD_GOD_POOL)
    DiscoverWildGodIfImbued4,
    /// Hamuul Runetotem: count friendly spells cast while in play; every
    /// third cast imbues again
    ImbueEveryThirdSpell,
    /// Summon a random minion of the given cost with the Dragon race (the
    /// Emerald Portal token — Casts-When-Drawn simplified to a playable
    /// spell that summons a random 1-Cost Dragon)
    SummonRandomDragonOfCost {
        /// Cost of the summoned dragon
        cost: i32,
    },
    // ----------------------------------------------------------------
    // 2025–2026 expansions M1-W2 effects (exp_edr_w2) — the Emerald Dream
    // dark-gift mechanic: the dark-gift Discover cards draw a random
    // qualifying minion and attach one of the ten dark gifts. A gift is a
    // card-level upgrade that persists across zones (hand / deck / play);
    // the marker rides the World `dark_gifts` component and the static
    // effects (enchantments + keyword components) are applied by
    // `engine::trigger::apply_dark_gift`.
    // ----------------------------------------------------------------
    /// Apply a dark gift to the subject (used by the trigger hooks; the
    /// W2 cards resolve the gift through the dedicated Discover variants
    /// below)
    ApplyDarkGift {
        /// The gift to apply
        gift: DarkGiftKind,
    },
    /// Discover a random minion of the given pool and give it a dark gift
    /// (Treacherous Tormentor — Legendary pool, Avant-Gardening —
    /// Deathrattle pool, Jumpscare! — Demons costing 5+ pool; the real
    /// Discover lets the player choose, simplified to random)
    DiscoverWithDarkGift {
        /// The pool the discovered minion is drawn from
        pool: RandomPool,
    },
    /// If the player is holding a Dragon, discover a random Dragon and give
    /// it a dark gift (Darkrider — the "holding a Dragon" condition gates
    /// the whole effect)
    DiscoverDragonWithDarkGift,
    /// Discover a random Undead; if the player can spend the given corpses,
    /// give it a dark gift (Rite of Atrocity — spends 2 corpses; without
    /// the corpses the Undead is added ungifted)
    DiscoverUndeadWithCorpseGift {
        /// Corpses spent for the gift
        corpses: u32,
    },
    /// Copy a random minion from the opponent's deck into the player's hand
    /// (Nightmare Fuel — **pool-open**: reads the enemy deck, registered in
    /// `sets::POOL_OPEN_CARDS`); with the Combo branch the copy receives a
    /// dark gift
    DiscoverEnemyDeckMinionCopy {
        /// Whether the Combo branch gives the copy a dark gift
        with_gift: bool,
    },
    /// Move a random minion from the player's own deck to hand and give it
    /// a dark gift (Nightmare Lord Xavius — the player's own deck is
    /// in-pool, not pool-open)
    DiscoverDeckMinionWithDarkGift,
    /// Reduce the Cost of the player's hand minions carrying a dark gift
    /// by (2) (Overgrown Horror)
    ReduceHandMinionGiftCost,
    // ----------------------------------------------------------------
    // 2025–2026 expansions M1-W3 effects (exp_edr_w3) — the Emerald Dream
    // choose-one cards: real Choose One resolution surfaces the branch
    // options through `legal_actions` (P3); the branch effects below cover
    // the two non-trivial shape families (buff + keyword, weapon-upgrade,
    // draw-by-type, corpse-spend damage, board-wide damage, simplified
    // Discover→random).
    // ----------------------------------------------------------------
    /// Give the target minion the given stats and Divine Shield (Lightmender
    /// choose branch 1)
    GainStatsAndGrantDivineShield {
        /// Attack gained
        attack: i32,
        /// Health gained
        health: i32,
        /// The buffed minion
        target: EffectTarget,
    },
    /// Give the target minion the given stats and Lifesteal (Lightmender
    /// choose branch 2)
    GainStatsAndGrantLifesteal {
        /// Attack gained
        attack: i32,
        /// Health gained
        health: i32,
        /// The buffed minion
        target: EffectTarget,
    },
    /// The player's hero gains Poisonous until the end of this turn (Barbed
    /// Thorn choose branch 1 — weapon attacks poison what they hit)
    GrantPoisonousThisTurn,
    /// The player's hero gains Lifesteal until the end of this turn
    /// (2025–2026 expansions M4-W2 — CATA_530 Fel Infusion's "Your hero
    /// has Lifesteal this turn"; the per-player flag expires in the
    /// turn-end wrap-up, the GrantPoisonousThisTurn convention)
    GrantHeroLifestealThisTurn,
    /// The player's weapon gains "Deathrattle: deal damage to all enemies"
    /// (Barbed Thorn choose branch 2 — the deathrattle fires when the weapon
    /// breaks or is replaced)
    GrantWeaponDeathrattleAllEnemies {
        /// Damage dealt to all enemies
        damage: i32,
    },
    /// Draw the given number of cards of the given type from the player's
    /// deck (Reforestation — "draw a spell" / "draw a minion"; the real
    /// card's 蓄力/Forge-esque hold mechanic is omitted, see §14.2)
    DrawCardByType {
        /// Cards drawn
        count: u32,
        /// The drawn card type (Spell or Minion)
        card_type: CardType,
    },
    /// Spend the given corpses to deal damage to a minion (Morbid Swarm
    /// choose branch 2 — the corpseless branch is a no-op, matching the
    /// spend-corpses precedent)
    SpendCorpsesDamageMinion {
        /// Corpses spent
        cost: u32,
        /// Damage dealt
        damage: i32,
    },
    /// Deal damage to ALL minions (Ominous Nightmares choose branch 1,
    /// Wyvern's Slumber choose branch 2)
    DamageAllMinions {
        /// Damage amount
        damage: i32,
    },
    /// Add a random Druid spell to hand (Spark of Life choose branch 2 —
    /// the real Discover lets the player choose, simplified to random)
    AddRandomDruidSpell,
    /// Add a random Choose One card of another class to hand (Symbiosis —
    /// the real Discover lets the player choose, simplified to a random
    /// pick over the fixed OTHER_CLASS_CHOOSE_ONE_POOL, see §14.2)
    AddRandomOtherClassChooseOneCard,
    /// Attack two random enemy minions if this card costs (n) or less
    /// (Verdant Dreamsaber — the attack is modeled as direct damage, like
    /// the "excess damage" family; the cost condition reads the played card)
    AttackTwoRandomEnemyMinionsIfCostLE {
        /// The cost threshold
        cost: u8,
    },
    /// Gain Armor and summon a random minion of the given cost, giving it
    /// Taunt (Ward of Earth)
    GainArmorSummonCostTaunt {
        /// Armor gained
        armor: i32,
        /// Cost of the summoned minion
        cost: u8,
    },
    /// Add a random minion of the given cost with a Dark Gift to hand
    /// (Creature of Madness — the real Discover lets the player choose,
    /// simplified to random)
    AddRandomCostMinionWithDarkGift {
        /// The minion's cost
        cost: u8,
    },
    /// Give the given stats to the top n minions in the player's deck
    /// (Beanstalk Brute — the deck is ordered, so the "top minions" are the
    /// first ones in deck order; enchantments persist deck → hand → play)
    BuffTopDeckMinions {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
        /// Number of top minions
        count: u8,
    },
    /// Shuffle each minion into a random player's deck (Typhoon)
    ShuffleAllMinionsIntoDecks,
    /// Draw a spell from the player's deck and add a random spell to hand
    /// (Dragonscale Armaments — the engine does not track card origins, so
    /// the "one that didn't start there" half is a random spell, see §14.3)
    DrawDeckSpellAndAddRandomSpell,
    /// Set a minion's stats depending on who owns it (Mark of Ursol — enemy
    /// targets become the enemy stat block, friendly ones the friendly one)
    SetStatsByFriendlyTarget {
        /// Enemy target attack
        enemy_attack: i32,
        /// Enemy target health
        enemy_health: i32,
        /// Friendly target attack
        friendly_attack: i32,
        /// Friendly target health
        friendly_health: i32,
    },
    /// Gain Attack equal to the Cost of a friendly spell just cast
    /// (Animated Moonwell)
    GainAttackEqualSpellCost,
    /// Deal damage to the lowest-Health enemy, twice (Renewing Flames)
    DamageLowestHealthEnemyTwice {
        /// Damage per hit
        amount: i32,
    },
    /// Draw the top card and gain the given stats (Dreamwarden — the real
    /// card's "didn't start there" condition is dropped, see §14.3)
    DrawAndGainStats {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// Shuffle the given number of copies of a card into the player's deck
    /// (Illusory Greenwing)
    ShuffleCardIntoDeck {
        /// Card to shuffle
        card_id: &'static str,
        /// Number of copies
        count: u8,
    },
    /// Give a minion stats and "Deathrattle: give a friendly minion stats
    /// and this Deathrattle" (Amphibian's Spirit — the engine stores one
    /// Deathrattle per entity, so an existing one is replaced, see §14.3)
    AmphibianSpiritBuff {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// Deal damage to a minion; if it dies, summon a Wolf token (Spirit Bond)
    DamageAndSummonWolfIfKilled {
        /// Damage dealt
        damage: i32,
    },
    /// Add a random spell to hand, reduced by the given amount (Horn of
    /// Plenty — the Nature school is not modeled, so the pool is any spell,
    /// and Discover is random, see §14.3)
    AddRandomSpellCostsLess {
        /// Cost reduction
        reduction: u8,
    },
    /// Summon a Treant token whose deathrattle adds a random spell to hand
    /// (Grove Shaper — the Nature school is not modeled, so the trigger
    /// fires on any friendly spell; the treant cannot remember the cast
    /// spell, see §14.3)
    SummonTreantCopyingSpell,
    /// Summon an Egg token that hatches a copy of a random friendly Dragon
    /// when it dies (Clutch of Corruption — the egg cannot remember which
    /// Dragon was chosen, see §14.3)
    SummonEggHatchingDragon,
    /// Resurrect a random friendly Dragon from the graveyard (Succumb to
    /// Madness — the real Discover lets the player choose, simplified to
    /// random over the death record)
    ResurrectRandomFallenDragon,
    /// Equip a Sword token if holding a Dragon (Brood Keeper)
    EquipSwordIfHoldingDragon,
    /// Deal damage to all other friendly minions (Afflicted Devastator)
    DamageAllOtherFriendlyMinions {
        /// Damage dealt
        damage: i32,
    },
    /// Deal damage to a minion, gaining Lifesteal if the player has cast 3
    /// or more spells this game (Wish of the New Moon — the per-card New
    /// Moon counter is approximated by the player's total spells cast,
    /// see §14.3)
    DamageMinionWithMoonLifesteal {
        /// Damage dealt
        amount: i32,
    },
    /// Summon two random minions of the given cost, upgraded after 3 spells
    /// cast (Ritual of the New Moon — see the Wish of the New Moon note)
    SummonTwoRandomCostMinions {
        /// Base summoned cost
        base_cost: u8,
        /// Upgraded summoned cost
        upgraded_cost: u8,
    },
    /// Deal damage to the enemy hero if holding a spell that costs (5) or
    /// more (Weaver of the Cycle)
    DamageIfHoldingSpell5Plus {
        /// Damage dealt
        amount: i32,
    },
    /// Summon a copy of the player's own minion if its Attack is at least
    /// the threshold (Mythical Runebear)
    SummonCopyIfAttackGE {
        /// Attack threshold
        attack: i32,
    },
    /// Restore Health to the hero and queue self-damage for the next n of
    /// the player's turns (Rotten Apple — the damage ticks at the END of
    /// each of the next turns, see §14.3)
    RestoreHealthAndPendingSelfDamage {
        /// Health restored
        heal: i32,
        /// Self damage per turn
        damage: i32,
        /// Number of affected turns
        turns: u8,
    },
    /// Destroy a Mana Crystal and gain some after n of the player's turns
    /// (Fractured Power — the gain lands at the END of the nth turn, see
    /// §14.3)
    DestroyCrystalGainCrystalsLater {
        /// Crystals gained later
        gain: i32,
        /// Number of affected turns
        turns: u8,
    },
    /// Draw a minion that costs (n) or more from the deck (Rotheart Dryad)
    DrawMinionCostGE {
        /// Minimum cost
        cost: u8,
    },
    /// Gain the Deathrattle of a friendly minion that died this turn
    /// (Archdruid of Thorns — the engine stores one Deathrattle per entity,
    /// so the most recently died minion's is used, see §14.3)
    GainDeathrattleOfDiedThisTurn,
    /// Add a random minion from the deck to hand (Hungering Ancient's
    /// deathrattle — the eaten minion's identity cannot be stored per
    /// instance, see §14.3)
    AddRandomDeckMinionToHand,
    /// Eat the first minion in the deck and gain its stats (Hungering
    /// Ancient's end-of-turn effect)
    EatDeckMinionGainStats,
    /// Reduce a random minion's Attack in each player's hand (Twisted
    /// Treant — the debuff enchantment persists from hand into play)
    DebuffRandomHandMinionBoth {
        /// Attack reduction
        attack_reduction: i32,
    },
    /// Spend all Mana to cast a random spell of that cost (Forbidden Shrine
    /// — "cast" is a direct effect resolution, see §14.3)
    SpendAllManaCastRandomSpell,
    /// Get a copy of the lowest-Cost card in the opponent's hand (Tricky
    /// Satyr)
    CopyLowestCostEnemyHandCard,
    /// The opponent draws two cards; the player gets copies of them
    /// (Mimicry)
    OpponentDrawsTwoAndCopies,
    /// Return a friendly minion to hand and summon a Spider token
    /// (Web of Deception)
    ReturnFriendlyMinionSummonSpider,
    /// Shuffle a card the player holds that the opponent also holds into
    /// the opponent's deck (Shadowcloaked Assailant — when several match,
    /// one random one is shuffled, see §14.3)
    ShuffleMatchingEnemyHandCardIntoDeck,
    /// Destroy a friendly minion to gain Armor (Siphoning Growth)
    DestroyFriendlyMinionGainArmor {
        /// Armor gained
        armor: i32,
    },
    /// Draw a spell that costs (n) or more from the deck (Fae Trickster)
    DrawSpellCostGE {
        /// Minimum cost
        cost: u8,
    },
    /// Draw the given number of Dragons, reducing their Cost (Tormented
    /// Dreadwing)
    DrawDragonsReduced {
        /// Cards drawn
        count: u8,
        /// Cost reduction
        reduction: u8,
    },
    /// Summon a copy of the player's own minion (Bloodthistle Illusionist;
    /// also 2025–2026 expansions M4-W4 — CATA_586 Destructive Blaze's
    /// "after this survives damage, summon a copy" trigger).
    SummonCopyOfSelf,
    /// Destroy a friendly Wisp to draw the given number of cards
    /// (Divination — the engine's Wisp is the EDR_851t token)
    DestroyFriendlyWispDraw {
        /// Cards drawn
        count: u8,
    },
    /// Draw the given number of cards and summon that many Leeches
    /// (Sanguine Infestation)
    DrawAndSummonLeeches {
        /// Cards drawn
        draw: u8,
    },
    /// Draw the given number of cards and summon a Dreadseed token (Grim
    /// Harvest — Dormant is not modeled, the Dreadseed is the W3
    /// can't-attack token, see §14.3)
    DrawAndSummonDreadseed {
        /// Cards drawn
        draw: u8,
    },
    /// Make the player's next Hero Power cost (0) (Dreambound Disciple —
    /// the flag is consumed at the hero-power activation)
    NextHeroPowerCostsZero,
    /// Permanently, Murlocs the player summons gain +1/+1 (TLC_426's
    /// repeatable quest reward, 2025–2026 expansions M2-W2 — sets the
    /// player's `murloc_summon_buff` flag; the friendly-summon hook in
    /// rules.rs applies the buff)
    SetMurlocSummonBuff,
    /// Permanently, whenever the player deals exactly 2 damage to an enemy,
    /// deal 2 more (Gorishi Colossus's battlecry, 2025–2026 expansions
    /// M2-W2 — sets the player's `deal_exact_2_bonus` flag; the damage
    /// hook in rules.rs applies the bonus)
    SetDealExact2Bonus,
    /// Restore Health to the hero and add random Druid spells to hand
    /// (Photosynthesis)
    RestoreHealthAndGetDruidSpells {
        /// Health restored
        amount: i32,
        /// Spells added
        count: u8,
    },
    /// Both players gain the given number of empty Mana Crystals (Tranquil
    /// Treant)
    GainManaCrystalBoth {
        /// Crystals gained
        count: i32,
    },
    /// Transform all Neutral cards in the deck into random Druid ones
    /// (Envoy of the Glade)
    TransformNeutralDeckToDruid,
    /// Add a Moonfire and a Starfire to hand, both with Spell Damage +1
    /// (Stellar Balance — the spell entity's own Spell Damage applies to
    /// itself when cast)
    AddMoonfireAndStarfireWithSpellDamage,
    /// Give stats to another random friendly Dragon (Petal Peddler)
    BuffAnotherRandomFriendlyDragon {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// Reduce the Cost of the right-most card in hand (Nightmare Dragonkin)
    ReduceRightmostHandCardCost {
        /// Cost reduction
        reduction: u8,
    },
    /// Resurrect a friendly Deathrattle minion that costs (n) or less and
    /// summon a copy of it (Ravenous Felhunter — the death record reads the
    /// graveyard, like Calia Menethil)
    ResurrectDeathrattleMinionCostLE {
        /// Maximum cost
        cost: u8,
    },
    /// Resurrect a friendly Deathrattle minion that costs (n) or more and
    /// summon a copy of it, excluding the dying minion itself (Ferocious
    /// Felbat)
    ResurrectDeathrattleMinionCostGE {
        /// Minimum cost
        cost: u8,
    },
    /// Gain Armor, plus more per friendly Wisp (Merry Moonkin — the
    /// engine's Wisp is the EDR_851t token)
    GainArmorPerWisp {
        /// Base armor
        base: i32,
    },
    /// Deal damage to a minion, plus more per friendly minion that died
    /// this game (Starsurge — the fallen count reads the graveyard)
    DamageMinionScaledByFallen {
        /// Base damage
        base: i32,
    },
    /// Give the hero Divine Shield (Curious Cumulus)
    GrantHeroDivineShield,
    /// Restore Health to both heroes (Critter Caretaker)
    RestoreBothHeroes {
        /// Health restored
        amount: i32,
    },
    /// Put a copy of the player's own card on the bottom of the deck at a
    /// reduced Cost (Meadowstrider — the deck's bottom is the end of the
    /// deck zone order)
    AddSelfToDeckBottomCost {
        /// Reduced cost
        cost: u8,
    },
    /// Summon a copy of a random friendly Dragon (the Clutch of Corruption
    /// Egg's hatch, see §14.3)
    SummonCopyOfRandomFriendlyDragon,
    /// Gain Health if the hero power was used this turn (Barkshield
    /// Sentinel — there is no hero-power-use trigger event, so the buff
    /// fires at the end of the turn instead, see §14.3)
    GainHealthIfHeroPowerUsed {
        /// Health gained
        amount: i32,
    },
    /// Attack a random enemy minion, dealing excess damage to the enemy
    /// hero (Briarspawn Drake — the attack is modeled as direct damage)
    AttackRandomEnemyMinionExcess,
    /// Deal the attacking hero's Attack as damage to a random enemy minion
    /// (Defiled Spear — the "another enemy" exclusion is dropped, §14.3)
    SplashHeroAttackToRandomEnemy,
    /// Gain the Attack of the minion that just died (Scavenging Flytrap —
    /// the graveyard wipe means the base Attack is gained, §14.3)
    GainDeadMinionAttack,
    /// Draw a card if the played minion was already played earlier this game
    /// (Twisted Webweaver — the played-minion log lives on the Player)
    DrawIfMinionPlayedBefore,
    /// Give the just-played minion a random Bonus Effect (Dreambound Raptor
    /// — the official pool is approximated by a fixed keyword pool, §14.3)
    GrantRandomBonusEffect,
    /// Ysera, Emerald Aspect (M1-W4b): the Start of Game effect ("increase
    /// both players' maximum Mana by 5") fires when Ysera is played (the
    /// engine has no StartOfGame event, §14.4), then the battlecry grants 3
    /// filled Mana Crystals
    YseraEmeraldAspect,
    /// Resurrect all different friendly minions that cost (8) or more
    /// (Merithra — "different" deduplicates by card ID, the graveyard is
    /// the death record)
    ResurrectAllDifferentFriendlyCostGE {
        /// Minimum base cost
        cost: u8,
    },
    /// Cast the highest-Cost spell from the hand as a normal spell (Ursol —
    /// the aura-ization is simplified to a direct cast, §14.4)
    CastHighestCostSpellFromHand,
    /// Omen's attack counter trigger (M1-W4b): each attack adds 1 to the
    /// deathrattle's "Improves" damage (§14.4 interpretation)
    IncrementOmenAttack,
    /// Omen's deathrattle (M1-W4b): deal 1 plus the recorded attack bonus
    /// to all enemies
    OmenDeathrattle,
    /// Deal `amount` damage split among all enemies as 1-damage hits, only
    /// when at least `threshold` friendly minions died this game (Aessina —
    /// the graveyard is the game-long death record)
    SplitDamageAmongAllEnemiesIfFallen {
        /// Total damage split
        amount: u8,
        /// Friendly deaths required
        threshold: u8,
    },
    /// The next `count` spells the player plays cast twice (Tyrande — the
    /// doubled cast re-resolves the same effect at play time; after-cast
    /// triggers fire once per play, §14.4)
    NextSpellsCastTwice {
        /// How many spells cast twice
        count: u8,
    },
    /// Summon a random Dragon for each time this minion died this game
    /// (Ysondre — the death count reads the graveyard, which includes the
    /// current death)
    SummonRandomDragonPerSelfDeath,
    /// Gain Armor and give this minion +Attack (Tortolla — fired by its
    /// ThisMinionDamaged trigger)
    GainArmorAndSelfAttack {
        /// Armor gained by the owner
        armor: i32,
        /// Attack gained by the minion
        attack: i32,
    },
    /// The next card the player plays costs (0) (Agamaggan — the
    /// opponent's-Health cost is simplified to 0, §14.4)
    NextCardCostsZero,
    /// Transform all minions in the hand into random Demons, keeping their
    /// original stats and Cost (Alara'shi)
    TransformHandMinionsToRandomDemons,
    /// Add a random spell to the hand, then surface a keep-or-put-on-top
    /// choice (Q'onzu — the Discover is simplified to a random spell,
    /// §14.4)
    DiscoverSpellKeepOrTop,
    /// Discard a random card from the opponent's hand (Renferal — the
    /// one-turn trap is simplified to a discard, §14.4)
    DiscardRandomEnemyHandCard,
    /// Fill the hand with copies of the opponent's deck cards, each costing
    /// `reduction` less (Ashamane — pool-open, reads the opponent's deck)
    FillHandWithEnemyDeckCopies {
        /// Cost reduction on each copy
        reduction: u8,
    },
    /// Summon `count` 1/1 Beetles (Nythendra — the split/reform cycle is
    /// simplified to a deathrattle summon, §14.4)
    SummonBeetles {
        /// Number of 1/1 Beetles
        count: u8,
    },
    /// Attack ALL other minions, recording the kills for the deathrattle
    /// (Ursoc — the attacks are direct damage; the kill record is exact
    /// because the damage resolves one target at a time)
    UrsocBattlecry,
    /// Resurrect every minion the battlecry killed (Ursoc)
    UrsocDeathrattle,
    /// Give all OTHER friendly minions +Attack/+Health (Forest Lord
    /// Cenarius — Choose Thrice branch 1)
    GainStatsAllOtherFriendlyMinions {
        /// Attack gained
        attack: i32,
        /// Health gained
        health: i32,
    },
    /// Summon a random Animal Companion (Broll Bearmantle — fired by its
    /// FriendlySpellCast trigger)
    SummonRandomAnimalCompanion,
    /// Add all 5 Dream cards to the hand (Shaladrassil — the corruption
    /// clause is unmodeled, §14.4)
    AddAllDreamCards,
    /// Your cards cost (1) this game (Aviana — the three-turn lunar cycle
    /// is simplified to an immediate effect, §14.4)
    CardsCostOneThisGame,
    /// Gain +Attack/+Health when the owner used their Hero Power this turn
    /// (Spirit of the Kaldorei — 2025–2026 expansions M1-W5; buffs the
    /// source itself)
    GainStatsIfHeroPowerUsed {
        /// Attack gained
        attack: i32,
        /// Health gained
        health: i32,
    },
    /// Give a friendly minion +Attack/+Health and Rush when the owner used
    /// their Hero Power this turn (Charred Chameleon — M1-W5)
    GiveMinionStatsRushIfHeroPowerUsed {
        /// Attack gained
        attack: i32,
        /// Health gained
        health: i32,
    },
    /// Draw N cards when the owner has Imbued their Hero Power at least
    /// twice (Petal Picker — M1-W5; the W1 DealDamageIfImbuedTwice
    /// threshold pattern)
    DrawIfImbuedTwice {
        /// Cards drawn
        count: u32,
    },
    /// Deal N damage to ALL enemy minions (Avatar of Destruction's
    /// deathrattle and Smoldering Ascent — M1-W5)
    DealDamageToAllEnemyMinions {
        /// Damage to each enemy minion
        damage: i32,
    },
    /// Add a random qualifying minion to the hand with a random dark gift,
    /// then reduce its Cost by `reduction` (Cremate — M1-W5, the
    /// Discover→random simplification)
    DiscoverWithDarkGiftCostReduction {
        /// Cost reduction on the added minion
        reduction: u8,
    },
    /// Summon two 4/4 Dragon Broodlings with Taunt while the owner holds a
    /// minion with a dark gift (Frostburn Matriarch — M1-W5)
    SummonBroodlingsIfHoldingGift,
    /// Destroy this and deal N damage to all enemies (Felfire Blaze — the
    /// Fel-spell filter is unmodeled: fires on ANY friendly spell cast,
    /// §14.5)
    FelfireBlazeTrigger {
        /// Damage to all enemies
        damage: i32,
    },
    /// Give all friendly minions +A/+H, then discard a random hand spell
    /// for an extra +A2/+H2 (Overheat — the Nature-spell filter is
    /// unmodeled, §14.5)
    BuffFriendlyMinionsDiscardBonus {
        /// Base attack gained
        attack: i32,
        /// Base health gained
        health: i32,
        /// Bonus attack gained on the discard
        bonus_attack: i32,
        /// Bonus health gained on the discard
        bonus_health: i32,
    },
    /// Summon a random 1-Cost minion, gain 1 Armor, draw 1 card and refresh
    /// 1 Mana (Amirdrassil — the "Improves each use" escalation is
    /// unmodeled, the fixed effect applies every use, §14.5)
    AmirdrassilActivate,
    /// Get a random Elemental and reduce its Cost by `reduction`
    /// (Inferno Herald — the Fire-spell filter is unmodeled: fires on ANY
    /// friendly spell cast, §14.5)
    InfernoHeraldTrigger {
        /// Cost reduction on the added Elemental
        reduction: u8,
    },
    /// Give a minion +A/+H; once the owner has cast `threshold` spells, add
    /// a Light of the New Moon back to hand (the official per-play
    /// counter is approximated by the player spell total, §14.5)
    BuffMinionReturnIfSpellsCast {
        /// Attack gained
        attack: i32,
        /// Health gained
        health: i32,
        /// Spell-cast threshold for the return
        threshold: u32,
    },
    /// The equipped weapon gains +N Attack while the owner holds a minion
    /// with a dark gift (Cindersword — M1-W5)
    GainWeaponAttackIfHoldingGift {
        /// Attack gained
        amount: i32,
    },
    /// Deal `base` damage to a random enemy minion, or `upgraded` while the
    /// owner holds a card costing at least the threshold (Flames of the
    /// Firelord — M1-W5)
    DamageRandomEnemyMinionHoldingCostGE {
        /// Base damage
        base: i32,
        /// Upgraded damage
        upgraded: i32,
        /// Cost threshold for the upgrade
        threshold: i32,
    },
    /// Add a random Combo/Battlecry/Stealth minion with a random dark gift
    /// to the hand (Smoke Bomb — the Discover→random simplification, §14.5)
    DiscoverComboBattlecryStealthWithDarkGift,
    /// Add a random Demon with a random dark gift to the hand, then a copy
    /// of it (Shadowflame Stalker — the Discover→random simplification,
    /// §14.5)
    DiscoverDemonWithDarkGiftCopy,
    /// Add a random card of the Cost to the hand and set a temporary mana
    /// crystal for the owner's next turn only (Emberscarred Whelp — M1-W5)
    DiscoverCostCardGainTempMana {
        /// Cost of the discovered card
        cost: u8,
        /// Temporary mana crystals for the next turn
        mana: u8,
    },
    /// Deal N damage (the explicit target or a random enemy) and add a
    /// random Warrior minion with a random dark gift to the hand
    /// (Shadowflame Suffusion — M1-W5, the Discover→random simplification)
    DamageAndDiscoverWarriorWithGift {
        /// Damage dealt
        damage: i32,
    },
    /// Reduce the Cost of every hand card by `reduction` while all hand
    /// cards cost differently (Zaqali Flamemancer — M1-W5)
    ReduceHandCostIfAllDistinct {
        /// Cost reduction
        reduction: u8,
    },
    /// Draw a minion (deck scan) and summon an 8/8 copy of it with Divine
    /// Shield (Searing Reflection — the tutor is a first-match scan, §14.5)
    DrawMinionSummonDivineShieldCopy,
    /// Spend the largest affordable 10/20/30 Corpses to gain that many
    /// stats on this minion (Volcoross — the 3-way choose is unmodeled:
    /// the largest affordable option is picked automatically, §14.5)
    VolcorossBattlecry,
    /// Add a random spell to the hand and reduce the Cost of every hand
    /// spell by `reduction` (Scorchreaver — the Fel-spell filter is
    /// unmodeled, §14.5)
    DiscoverSpellReduceHandSpells {
        /// Cost reduction on hand spells
        reduction: u8,
    },
    /// Deal this minion's Attack damage split among all enemies after it
    /// attacks a minion and survives (Magma Hound — M1-W5)
    MagmaHoundSplash,
    /// Deal N damage to a minion; its owner draws a card (Conflagrate —
    /// M1-W5)
    DamageMinionOwnerDraws {
        /// Damage dealt
        damage: i32,
    },
    /// Deal `base` damage to all enemies, or `boosted` when it is the
    /// opponent's turn (Tindral Sageswift's deathrattle — M1-W5)
    DeathrattleDamageAllEnemiesTurnScaled {
        /// Damage on the owner's turn
        base: i32,
        /// Damage on the opponent's turn
        boosted: i32,
    },
    /// Deal N damage randomly split among all enemies — N independent
    /// 1-damage pings (Fyrakk's "cast 15 Mana worth of Fire spells" is
    /// approximated as 15 split damage, §14.5)
    DealDamageSplitAmongAllEnemies {
        /// Total damage dealt as 1-damage pings
        amount: i32,
    },
    /// Copy the lowest-Cost Beast in the hand (Tending Dragonkin — M1-W5)
    CopyLowestCostBeastInHand,
    /// Gain Divine Shield and Lifesteal while the owner holds a spell
    /// costing at least the threshold (Ashleaf Pixie — M1-W5)
    GainDivineShieldLifestealIfHoldingSpellGE {
        /// Cost threshold of the held spell
        cost: i32,
    },
    /// Give the hero +N Attack this turn and M Armor while the owner holds
    /// a minion with a dark gift (Dragon Turtle — M1-W5)
    GainHeroAttackArmorIfHoldingGift {
        /// Hero Attack gained this turn
        attack: i32,
        /// Armor gained
        armor: i32,
    },
    /// Deal N damage, then discard a random hand spell to deal N more
    /// (Scorching Winds — the Fire-spell filter is unmodeled, §14.5)
    DamageAndDiscardSpellMore {
        /// Base damage
        base: i32,
        /// Bonus damage on the discard
        bonus: i32,
    },
    /// Give all minions in the hand +N/+M (Keeper of Flame — M1-W5; the
    /// "destroyed in 3 turns" clause is unmodeled, §14.5)
    BuffAllHandMinions {
        /// Attack increase
        attack: i32,
        /// Health increase
        health: i32,
    },
    /// Give the source minion Rush (Stormbrewer's Kindred — 2025–2026
    /// expansions M2-W3)
    GainRush {
        /// Target selection
        target: EffectTarget,
    },
    /// Give the source minion Immune until the end of the turn (Whirling
    /// Stormdrake's Kindred — M2-W3; the temporary immunity clears in the
    /// turn-end wrap-up)
    GainImmuneThisTurn {
        /// Target selection
        target: EffectTarget,
    },
    /// The player's next Murloc costs `amount` less (Hot Spring Glider's
    /// battlecry — M2-W3; one-time, applied by the cost pipeline and
    /// consumed by the next Murloc play, no turn-end clear)
    NextMurlocCostsLess {
        /// Discount amount
        amount: i32,
    },
    /// The player's next Murloc gains Divine Shield (Hot Spring Glider's
    /// Kindred — M2-W3; consumed together with `NextMurlocCostsLess` by
    /// the next Murloc play)
    GiveNextMurlocDivineShield,
    /// The player's next Kindred triggers twice (Primalfin Challenger's
    /// battlecry — M2-W3; consumed by the next OnPlay Kindred resolution)
    SetNextKindredTwice,
    /// Draw the first Kindred card from the deck, then the first remaining
    /// card of the same Kindred type (Torga's battlecry — M2-W3; an empty
    /// match draws nothing)
    DrawKindredAndActivator,
    /// Draw a Fire spell (Volcanic Thrasher's battlecry — M2-W3); when
    /// the Kindred condition holds, the drawn spell gains Spell Damage +2
    DrawSpellGiveSpellDamage {
        /// Spell Damage granted by the Kindred half
        amount: i32,
    },
    /// Draw a minion of each cost from 1 to `up_to` (Hybridization's
    /// battlecry — M2-W3); when the Kindred condition holds, each drawn
    /// card costs (1) less
    DrawMinionsOfEachCost {
        /// Highest cost drawn (draws one of each cost 1..=up_to)
        up_to: i32,
    },
    /// Draw a Deathrattle minion costing at most `max_cost` (Dread
    /// Raptor's battlecry — M2-W3); when the Kindred condition holds, the
    /// drawn card costs (0)
    DrawDeathrattleMinionCostLE {
        /// Maximum cost of the drawn minion
        max_cost: i32,
    },
    /// Destroy the enemy minion with the lowest Attack (Scalehide Kodo's
    /// battlecry — M2-W3; the Kindred half switches it to the highest)
    DestroyLowestAttackEnemy,
    /// Trigger the Deathrattles of all friendly Sizzling Cinders
    /// (Slagclaw's Kindred add-on — M2-W3)
    TriggerFriendlyCinderDeathrattles,
    /// Destroy a minion and give the source its stats (Ravenous
    /// Devilsaur's Kindred — M2-W3; the stats are read before the destroy)
    DestroyMinionAndGainItsStats {
        /// Target selection
        target: EffectTarget,
    },
    /// Deal damage equal to the source's Attack to the target (Ravasaur
    /// Matriarch's Kindred — M2-W3)
    DealSelfAttackDamage {
        /// Target selection
        target: EffectTarget,
    },
    /// Summon a random minion of the given cost and give it Taunt
    /// (Gravedawn Voidbulb — M2-W3; the random pool follows the D2
    /// simplification — a filtered ALL_CARDS window, token-excluded, §16)
    SummonRandomMinionCostTaunt {
        /// Cost of the summoned minion
        cost: i32,
    },
    // ===== 2025–2026 expansions M2-W4a (the Un'Goro main set) =====
    /// Discover a card from a pool (M2-W4a — the D2 random simplification,
    /// fidelity-debt §17): builds the three-option choice from
    /// `cards::pool::discover_pool_cards`; the unpicked options feed the
    /// Map-chain / Merchant-of-Legend / Paleomancy resolutions keyed by the
    /// source card in ChoiceResolved.
    DiscoverPool {
        /// The pool to discover from
        pool: DiscoverPool,
    },
    /// Add `count` copies of a random pool card to the hand (M2-W4a —
    /// Whispering Stone's Fel spells use the CostHealth variant below;
    /// this is the plain count form)
    AddRandomCardToHandCount {
        /// Pool type
        pool: RandomPool,
        /// Number of cards
        count: u8,
    },
    /// Add `count` copies of a fixed card to the hand (M2-W4a —
    /// Infestation's two Gorishi Stingers)
    AddCardToHandCount {
        /// Card ID
        card_id: &'static str,
        /// Number of cards
        count: u8,
    },
    /// Add `count` random Temporary minions of the given cost to the hand
    /// (M2-W4a — Tunnel Terror's deathrattle; the Temporary marker is set
    /// on each added card, discarded at the end of the turn)
    AddTemporaryRandomMinionsCost {
        /// Cost of the Temporary minions
        cost: i32,
        /// Number of cards
        count: u8,
    },
    /// Add `count` random Fel spells to the hand marked as costing Health
    /// instead of Mana (M2-W4a — Whispering Stone's deathrattle)
    AddRandomFelSpellsCostHealth {
        /// Number of spells
        count: u8,
    },
    /// Add a random Holy and a random Shadow spell to the hand (M2-W4a —
    /// Twilight Mender's deathrattle)
    AddRandomHolyAndShadowSpell,
    /// Add a random 1-Cost Holy spell that gives +2 or -2 Health to the
    /// hand (M2-W4a — Glade Ecologist's deathrattle)
    AddRandomHolySpellCost1,
    /// Add a copy of another Elemental or Dragon in the hand (M2-W4a —
    /// Cloud Serpent's battlecry; no eligible card copies nothing)
    CopyRandomHandElementalOrDragon,
    /// Reduce the Cost of a random minion in the opponent's hand (M2-W4a —
    /// Curious Explorer's deathrattle)
    ReduceRandomEnemyHandMinionCost {
        /// Cost reduction
        amount: i32,
    },
    /// Reduce the Cost of a random Beast in the owner's hand (M2-W4a —
    /// Dinositter's end-of-turn effect)
    ReduceRandomBeastHandCost {
        /// Cost reduction
        amount: i32,
    },
    /// Reduce the Cost of every hand card that did not start in the deck
    /// by `amount` (M2-W4a — Story of the Waygate; the starting deck is
    /// snapshotted by GameBuilder, see fidelity-debt §17)
    ReduceNonStartingHandCost {
        /// Cost reduction
        amount: i32,
    },
    /// Summon four 2/2 Treants that each attack a chosen minion (M2-W4a —
    /// TREEEES!!!; the target is chosen by the explicit-target resolution,
    /// the attacks run through the normal attack pipeline)
    SummonTreantsAttackMinion,
    /// Deal `amount` damage to a random enemy and summon that many 2/1
    /// Sizzling Cinders (M2-W4a — Sizzling Swarm; the summon count equals
    /// the damage dealt — the official card's damage is a fixed 3)
    DealDamageSummonCinders {
        /// Damage dealt (also the Cinder summon count)
        amount: i32,
    },
    /// Deal `amount` damage to the enemy with the lowest Health, `times`
    /// times (M2-W4a — Lava Flow; ties resolve randomly)
    DealDamageLowestHealthEnemyRepeated {
        /// Damage per hit
        amount: i32,
        /// Number of hits
        times: i32,
    },
    /// Deal `amount` damage to `count` random DISTINCT enemies (M2-W4a —
    /// Bonechill Stegodon's deathrattle)
    DealDamageRandomEnemies {
        /// Damage per enemy
        amount: i32,
        /// Number of enemies hit
        count: i32,
    },
    /// Draw `count` minions of different minion types and give them stats
    /// (M2-W4a — Flight of the Firehawk; draws until `count` distinct
    /// types or the deck runs out)
    DrawMinionsDifferentTypesBuff {
        /// Minions drawn
        count: u8,
        /// Attack bonus on each drawn minion
        attack: i32,
        /// Health bonus on each drawn minion
        health: i32,
    },
    /// Draw a minion; if it has `min_attack` or more Attack, give it
    /// `buff_health` Health and gain `armor` Armor (M2-W4a — Story of
    /// Barnabus)
    DrawMinionBuffArmorIfAttackGE {
        /// Attack threshold
        min_attack: i32,
        /// Health bonus when the threshold is met
        buff_health: i32,
        /// Armor gained when the threshold is met
        armor: i32,
    },
    /// Summon three 2/1 Hatchlings at the start of the owner's NEXT turn
    /// (M2-W4a — Ravenous Flock; resolved by the turn-start hook)
    SetFlockPending,
    /// Give the owner's OTHER minions with at most `max_attack` Attack
    /// stats and Taunt (M2-W4a — Hatchery Helper's battlecry)
    GiveBuffOtherMinionsAttackLE {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
        /// Attack ceiling
        max_attack: i32,
    },
    /// Destroy a minion and summon a random minion of the same Cost to
    /// replace it (M2-W4a — Life Cycle; the cost is read before the
    /// destroy, the replacement is summoned to the same board)
    DestroyMinionSummonRandomSameCost {
        /// Target selection
        target: EffectTarget,
    },
    /// Summon `count` copies of a minion and grant each a random Bonus
    /// Effect (M2-W4a — Tyrannogill's three Dinolocs)
    SummonMinionsGrantRandomBonus {
        /// Card ID to summon
        card_id: &'static str,
        /// Summon count
        count: u8,
    },
    /// Summon one copy of each of two tokens (M2-W4a — Blob of Tar's two
    /// Blobs)
    SummonMinionPair {
        /// First token
        a: &'static str,
        /// Second token
        b: &'static str,
    },
    /// Summon a random minion of `cost`; if the player Discovered this
    /// turn, of `escalated_cost` instead (M2-W4a — Unearthed Artifacts)
    SummonRandomMinionCostOrEscalated {
        /// Base cost
        cost: i32,
        /// Cost after a Discover this turn
        escalated_cost: i32,
    },
    /// Deal `amount` damage to an enemy minion; if it dies, gain `armor`
    /// Armor (M2-W4a — Latorvian Armorer's battlecry)
    DealDamageGainArmorIfKilled {
        /// Damage amount
        amount: i32,
        /// Armor gained on kill
        armor: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Deal `damage` to all enemy minions; the enemy's minions cost (2)
    /// more next turn (M2-W4a — Wave of Tar; the cost-more flag lands on
    /// the CASTER — play_cost reads `player.opponent().minions_cost_more`
    /// — and clears at the caster's next turn start)
    DealDamageAllEnemyMinionsSetMinionsCostMore {
        /// Damage to each enemy minion
        damage: i32,
    },
    /// Give the target stats, then give the same stats to the owner's
    /// other minions that share a minion type with it (M2-W4a — Ready the
    /// Fleet)
    GiveBuffSameType {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Grant `count` random Bonus Effects to the target (M2-W4a — Story
    /// of Galvadon's three to one minion)
    GrantRandomBonusEffects {
        /// Number of effects
        count: u8,
        /// Target selection
        target: EffectTarget,
    },
    /// Give a random friendly minion a random Bonus Effect and this
    /// Deathrattle (M2-W4a — Stranglevine; the granted deathrattle is the
    /// same effect, so the chain recurs)
    GrantRandomBonusEffectAndDeathrattle,
    /// Activate the Story of Lakkari end-of-turn loop for `ticks` turns
    /// (M2-W4a — discard a card and fill the board with 3/2 Imps)
    SetLakkariTicks {
        /// Activations remaining
        ticks: u8,
    },
    /// Give the target stats and "Deathrattle: Summon a random minion of
    /// `summon_cost`" (M2-W4a — Threshrider's Blessing)
    GiveBuffAndSummonDeathrattle {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
        /// Cost of the deathrattle's summoned minion
        summon_cost: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Deal `amount` damage improved by the number of times the player
    /// shuffled cards into their deck this game (M2-W4a — Knockback)
    DealDamageImprovedByShuffles {
        /// Base damage
        amount: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Draw a card and link it to this minion (M2-W4a — Platysaur's
    /// battlecry; the linked card is discarded by `DiscardLinkedDrawnCard`
    /// when the minion dies)
    DrawCardLinkDeathrattle,
    /// Discard the card linked by `DrawCardLinkDeathrattle` (M2-W4a —
    /// Platysaur's deathrattle)
    DiscardLinkedDrawnCard,
    /// Gain `armor` Armor, then deal damage equal to the owner's Armor to
    /// an enemy minion (M2-W4a — Fortify)
    GainArmorDealDamageEqual {
        /// Armor gained first
        armor: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Destroy the top `count` cards of the owner's deck (M2-W4a —
    /// Willful Watcher's deathrattle)
    DestroyDeckTop {
        /// Cards destroyed
        count: u8,
    },
    /// Resurrect a 1-, 2-, and 3-Cost minion and give them Reborn (M2-W4a
    /// — Resuscitate; one fallen minion per cost from the graveyard,
    /// missing costs resurrect nothing)
    ResurrectOneOfEachCostGiveReborn {
        /// Highest cost resurrected (resurrects one of each cost 1..=max)
        max_cost: i32,
    },
    /// Deal `amount` damage to a minion; the next Beast the owner plays
    /// this turn costs `discount` less (M2-W4a — Cower in Fear)
    DealDamageSetNextBeastDiscount {
        /// Damage amount
        amount: i32,
        /// Cost reduction on the next Beast
        discount: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Give all Beasts in the owner's hand, deck, and battlefield stats
    /// (M2-W4a — Supreme Dinomancy)
    BuffAllBeastsEverywhere {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// Deal `amount` damage to a minion and all other minions of the same
    /// minion type (M2-W4a — Fumigate)
    DealDamageSameType {
        /// Damage amount
        amount: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// The next Temporary card costs `amount` less (M2-W4a — Spelunker's
    /// battlecry, one-time, consumed by the next Temporary play)
    SetNextTemporaryDiscount {
        /// Cost reduction
        amount: i32,
    },
    /// Until the start of the owner's next turn, the enemy hero can't be
    /// healed (M2-W4a — Crater Gator's battlecry)
    SetEnemyHeroCantBeHealed,
    /// Destroy a friendly minion and add the Bones of its Attack and
    /// Health to the hand (M2-W4a — Dissolving Ooze; two Bone tokens whose
    /// Attack/Health copy the destroyed minion's, see fidelity-debt §17)
    DestroyFriendlyMinionAddBones {
        /// Target selection
        target: EffectTarget,
    },
    /// Gain empty Mana Crystals until both players have the same Mana
    /// (M2-W4a — Crystal Tender's battlecry)
    GainManaCrystalsMatchOpponent,
    /// Give stats to each friendly minion whose minion type appears
    /// exactly once among friendly minions (M2-W4a — Tortollan
    /// Storyteller's end-of-turn effect, the "different type" reading)
    GiveBuffDifferentTypeMinions {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// If the player played a Quest this game, deal `amount` damage to an
    /// enemy minion (M2-W4a — Questing Assistant's battlecry)
    DealDamageIfQuestPlayed {
        /// Damage amount
        amount: i32,
        /// Target selection
        target: EffectTarget,
    },
    /// Swap the hero power to "Deal 8 damage to a random enemy"; after 2
    /// uses it swaps back (M2-W4a — Story of Sulfuras)
    SwapHeroPowerToDeal8Random,
    /// Recast a random Holy spell the owner cast this turn (M2-W4a —
    /// Creature of the Sacred Cave's end-of-turn effect, "targets this if
    /// possible" — the recast resolves on the creature, see §17)
    RecastRandomHolySpellThisTurn,
    /// Cast a random spell from the owner's deck costing at most
    /// `max_cost` (M2-W4a — Violet Treasuregill's battlecry, "targets
    /// this if possible")
    CastRandomSpellFromDeckCostLE {
        /// Maximum cost of the cast spell
        max_cost: i32,
    },
    /// Spend up to `max_spend` Armor; deal that much damage to all minions
    /// (M2-W4a — Shellnado)
    SpendArmorDealDamageAllMinions {
        /// Maximum Armor spent
        max_spend: i32,
    },
    /// Destroy the top card of the deck and Discover a card of the same
    /// Rarity (M2-W4a — Relic Miner's battlecry; the pool is resolved from
    /// the destroyed card's rarity)
    DestroyTopCardDiscoverSameRarity,
    /// Grant a keyword to the target (M2-W4a — the choose-one branches of
    /// Ancient Stegodon / Ancient Raptor / Ancient Pterrordax)
    GrantKeyword {
        /// The keyword to grant
        keyword: KeywordKind,
        /// Target selection
        target: EffectTarget,
    },
    /// Grant the target "Deathrattle: Summon `count` copies of a card"
    /// (M2-W4a — Ancient Raptor's third choose-one branch)
    GrantDeathrattleSummon {
        /// Card ID the deathrattle summons
        card_id: &'static str,
        /// Summon count
        count: u8,
        /// Target selection
        target: EffectTarget,
    },
    /// Add a random weapon from another class to the hand; while the Combo
    /// condition holds it enters with `combo_attack` extra Attack
    /// (M2-W4a — Neferset Weaponsmith)
    AddRandomWeaponAnotherClassComboAttack {
        /// Attack bonus under Combo
        combo_attack: i32,
    },
    /// Drain: deal `amount` damage to all other minions and restore that
    /// much Health to this minion per minion damaged (M2-W4a — Juvenile
    /// Pterrordax's "steal 1 Health from all other minions")
    Drain {
        /// Amount stolen per minion
        amount: i32,
    },
    /// Gain stats equal to the Cost of the Fire spell that was just cast
    /// (M2-W4a — Mechanized Magma's "Whenever you play a Fire spell, gain
    /// stats equal to its Cost"; the spell school is read from the card
    /// definition, the cost is the spell's effective cost at cast time)
    GainStatsEqualFireSpellCost,
    /// Deal `amount` damage to the target and summon one copy of `card_id`
    /// (M2-W4a — Gorishi Stinger's "Deal 2 damage. Summon a 2/1 Grub with
    /// Rush"; the damage targets the spell's explicit target, or a random
    /// enemy when none was chosen)
    DealDamageAndSummon {
        /// Damage amount
        amount: i32,
        /// Card ID of the summoned minion
        card_id: &'static str,
    },
    /// Discover a card from the owner's deck (M2-W4a — Cursed Catacombs
    /// marks the picked card Temporary; Cultist Map runs the Map chain):
    /// the pool is three random distinct deck card ids, the source card
    /// excluded, surfaced as a `ChoiceKind::DiscoverDeck` choice
    DiscoverDeckCard,
    /// Look at the top 3 cards of the enemy's deck and pick one to put on
    /// top (M2-W4a — Eyes in the Sky; surfaced as a
    /// `ChoiceKind::DiscoverEnemyDeckPutOnTop` choice — the deck is
    /// untouched otherwise)
    DiscoverEnemyDeckTop,
    /// Summon a random Fel Beast (M2-W4a — Deathrot Maw's deathrattle;
    /// the random pool is `RandomPool::FelBeast`, the D2 simplification)
    SummonRandomFelBeast,
    /// Add a random Beast to the hand costing `amount` less (M2-W4a —
    /// Storm the Gates' reward Zombeast, "crafted" as a random Beast with
    /// the (3) cost reduction applied; see fidelity-debt §17)
    AddRandomBeastCostLess {
        /// Cost reduction on the added Beast
        amount: i32,
    },
    // ----------------------------------------------------------------
    // M2-W4b — the Un'Goro legendary wave (src/cards/exp_tlc_w4b.rs): the
    // 14 TLC legendary cards.
    // ----------------------------------------------------------------
    /// Trigger the Deathrattles of up to `count` friendly minions that died
    /// this game (M2-W4b — Endbringer Umbra's battlecry; the friendly
    /// graveyard is the died-this-game log — the W3
    /// `TriggerFriendlyCinderDeathrattles` scan generalized)
    TriggerFriendlyDeadDeathrattles {
        /// Number of deathrattles to trigger
        count: i32,
    },
    /// City Chief Esho's battlecry: if every minion in the owner's deck
    /// shares a minion type, give the owner's other minions stats wherever
    /// they are (M2-W4b — the M2-W4a `BuffAllBeastsEverywhere` pattern
    /// generalized to every minion, the source excluded; the deck-check
    /// semantics are pinned in the resolve arm)
    EshoDeckCheckBuffEverywhere {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// Set the Attack and Health of all enemy minions to fixed values
    /// (M2-W4b — Krog's end-of-turn "set the Attack and Health of all enemy
    /// minions to 1"; permanent enchantments on the affected minions are
    /// removed and damage is cleared, matching the real set semantics)
    SetStatsAllEnemyMinions {
        /// Attack value
        attack: i32,
        /// Health value
        health: i32,
    },
    /// Summon a fresh copy of each damaged friendly minion; the copies gain
    /// Rush (M2-W4b — Nablya's battlecry; the copies are base-stat entities
    /// per the engine copy convention)
    SummonDamagedCopiesRush,
    /// Summon two random Deathrattle minions from the deck (as copies — the
    /// deck is untouched) and make them fight each other (M2-W4b — High
    /// Cultist Herenn; the registered simplification §18: "they fight" is
    /// each dealing its Attack damage to the other once)
    SummonTwoDeathrattleMinionsAndFight,
    /// Set the owner's `minions_cost_5` flag — all their minions cost (5)
    /// for the rest of the game (M2-W4b — Loh's battlecry; read by the
    /// play-cost pipeline)
    LohMinionsCost5,
    /// Elise the Navigator's deck-composition check (M2-W4b — registered
    /// simplification §18: the starting-deck check ("10 cards of different
    /// costs") runs against the `Player::starting_deck` snapshot and sets
    /// the `elise_location_crafted` marker; no custom-location machinery)
    EliseCraftLocation,
    /// Niri of the Crater's played-card trigger (M2-W4b): the CardPlayed
    /// event fires after any card fully resolved (minion plays and spell
    /// casts alike — the single trigger the per-ID registration can hold),
    /// so the effect branches on the subject's card type. A 1-Cost minion
    /// doubles its stats (an enchantment equal to the current stats); a
    /// 1-Cost spell casts twice (the second resolution uses no explicit
    /// target and fires no second SpellCast event, the Tyrande timing
    /// simplification §14.4). "1-Cost" reads the effective cost at trigger
    /// time.
    NiriOfTheCrater,
    /// Set the event subject's Health equal to the source's effective
    /// Health (M2-W4b — Archaios's "after a friendly minion attacks, set
    /// its Health to this minion's Health"; the set clears damage and
    /// permanent enchantments so the effective Health matches the source)
    SetEventSubjectHealthToSource,
    /// Deal damage to the opponent's left- and right-most minions — the
    /// two board edge positions (M2-W4c — Diabolus Rex's Kindred OnPlay; a
    /// one-minion board is both edges and is hit once)
    DealDamageToLeftRightEnemyMinions {
        /// Damage amount
        amount: i32,
    },
    /// Give the owner's OTHER minions Rush (M2-W4c — Firegill's Kindred
    /// OnPlay; the source — the Kindred card itself — is excluded)
    GiveOtherFriendlyMinionsRush,
    /// Deal damage to two random enemy minions and Freeze them (M2-W4c —
    /// Chillspine Stegodon's Kindred battlecry modifier, replace: true, so
    /// the freeze lands on the SAME two minions the damage hit)
    DealDamageToTwoAndFreeze {
        /// Damage amount
        amount: i32,
    },
    /// Set a friendly minion's stats to fixed values and fill the board
    /// with copies of it (M2-W4c — Bat Mask; the set writes the base
    /// stats, clears damage and strips permanent enchantments — the W4b
    /// set semantics — and each copy is itself set to the same stats)
    SetStatsAndFillBoardWithCopies {
        /// Attack value
        attack: i32,
        /// Health value
        health: i32,
    },
    /// Set a minion's stats to fixed values and give it Charge (M2-W4c —
    /// Devilsaur Mask; either side's minions can be chosen)
    SetStatsAndGrantCharge {
        /// Attack value
        attack: i32,
        /// Health value
        health: i32,
        /// The target set (any minion on either side)
        target: EffectTarget,
    },
    /// Set a minion's stats to fixed values, give it Lifesteal, and force
    /// a random enemy minion that can attack to attack it (M2-W4c —
    /// Behemoth Mask; the forced attack runs through the normal attack
    /// pipeline, the Mythical Terror convention)
    SetStatsGrantLifestealForceAttack {
        /// Attack value
        attack: i32,
        /// Health value
        health: i32,
        /// The target set (any minion on either side)
        target: EffectTarget,
    },
    /// Set a minion's stats to fixed values and attach the deathrattle
    /// "deal 2 damage to all minions" (M2-W4c — Sheep Mask)
    SetStatsAttachDamageAllDeathrattle {
        /// Attack value
        attack: i32,
        /// Health value
        health: i32,
        /// The target set (any minion on either side)
        target: EffectTarget,
    },
    /// Set a minion's stats to fixed values, give it Stealth, and draw
    /// cards (M2-W4c — Panther Mask)
    SetStatsGrantStealthAndDraw {
        /// Attack value
        attack: i32,
        /// Health value
        health: i32,
        /// Cards drawn
        draw: i32,
        /// The target set (any minion on either side)
        target: EffectTarget,
    },
    /// Summon a minion and give the owner's minions (the new one
    /// included) a stat buff (M2-W4c — Longneck Egg's deathrattle: summon
    /// a 3/3 Beast, give your minions +1/+1)
    SummonMinionAndBuffFriendlyMinions {
        /// The summoned card
        card_id: &'static str,
        /// Attack bonus for the friendly minions
        attack: i32,
        /// Health bonus for the friendly minions
        health: i32,
    },
    /// Summon a random Beast from the owner's deck as a fresh copy and
    /// give it Lifesteal (M2-W4c — Possessed Animancer's deathrattle; the
    /// deck-untouched copy-summon is the Herenn §18 convention)
    SummonRandomDeckBeastGiveLifesteal,
    /// Summon raptor tokens; when the spell was Outcast, give them Immune
    /// for the rest of the turn (M2-W4c — Horn of Feasting; the official
    /// "Immune while attacking" is the Immune-until-end-of-turn shape)
    SummonRaptorsOutcast {
        /// Raptor count
        count: i32,
    },
    /// Reduce the Cost of the adjacent hand cards — the cards at the
    /// played card's hand positions minus one and plus one at play time —
    /// by a fixed amount (M2-W4c — Skittish Saucier's battlecry; the play
    /// path records the played card's hand index on the player)
    ReduceAdjacentHandCardCost {
        /// Discount amount
        amount: i32,
    },
    /// The Great Dracorex's after-attack splash (M2-W4c): the source
    /// deals its Attack damage to every enemy minion except the attacked
    /// one (the attack-declaration trigger fires with the defender as the
    /// event subject, so the splash can exclude it)
    DracorexSplash,
    /// Set the owner's `hatching_pending` flag — the +2/+2 to the owner's
    /// minions fires at the end of the owner's NEXT turn (M2-W4c —
    /// Hatching Ceremony; the flock_pending turn-start flag precedent)
    SetHatchingPending,
    /// Deal damage to a target and give the owner's Elementals a stat
    /// buff (M2-W4c — Fire Breath)
    DealDamageAndBuffFriendlyElementals {
        /// Damage amount (spell-power scaled)
        damage: i32,
        /// Attack bonus for the friendly Elementals
        attack: i32,
        /// Health bonus for the friendly Elementals
        health: i32,
    },
    /// Shuffle the left-most card in the owner's hand into their deck at
    /// a random position (M2-W4c — Crystal Tusk's battlecry; the
    /// Tradeable shuffle pattern)
    ShuffleLeftmostHandCardIntoDeck,
    /// Draw the first 0-Attack minion in the owner's deck (M2-W4c — Holy
    /// Eggbearer's battlecry; a scan draw, the Sense Demons pattern — a
    /// deck with no 0-Attack minion draws nothing)
    DrawZeroAttackMinion,
    /// Transform a random minion into a random minion (M2-W4c — Tribute
    /// Dance, registered simplification §19: the official two-choice
    /// transform resolves as two random picks)
    TransformRandomMinionIntoRandomMinion,
    /// Summon a random Deathrattle minion costing at least `min_cost` and
    /// trigger its Deathrattle (M2-W4c — Story of Umbra; the D2 random
    /// Discover simplification, §19)
    SummonRandomDeathrattleMinionCostGEAndTrigger {
        /// Minimum cost
        min_cost: i32,
    },
    /// Spend Corpses to gain Reborn on the source (M2-W4c — Hollow
    /// Direhorn's after-a-friendly-minion-dies trigger; does nothing with
    /// fewer Corpses, the SpendCorpsesSummonCopy pattern)
    SpendCorpsesGainReborn {
        /// Corpses spent
        amount: i32,
    },
    /// Give the owner's minions +1 Attack and Rush, marking them to die
    /// at the end of the owner's turn (M2-W4c — Soulrest Ceremony; the
    /// turn-end sweep kills the marked minions through the normal death
    /// path so deathrattles fire)
    SoulrestMarkAndBuff,
    /// Give a target minion a stat buff and Rush (M2-W4c — Herbivore
    /// Assistant's battlecry, a friendly Beast)
    GainStatsAndGrantRush {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
        /// The target set (a friendly Beast)
        target: EffectTarget,
    },
    /// Give all minions in the owner's hand and deck a stat buff (M2-W4c —
    /// Seismopod's deathrattle; the wherever-buff machinery from W4b's
    /// Esho — base stat writes, the Grimestreet convention)
    BuffHandAndDeckMinions {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// Summon two random Beasts of the given Cost; each attacks a random
    /// enemy character through the normal attack pipeline (M2-W4c —
    /// Ankylodon's deathrattle)
    SummonTwoRandomCostBeastsAttackRandomEnemies {
        /// The exact Cost of the summoned Beasts
        cost: i32,
    },
    /// Summon a random Legendary minion and set its stats to fixed values
    /// (M2-W4c — Hero's Welcome; the D2 random Discover simplification,
    /// §19 — the set is the W4b set semantics)
    SummonRandomLegendaryMinionSetStats {
        /// Attack value
        attack: i32,
        /// Health value
        health: i32,
    },
    /// Summon a random minion of the given Cost and set its stats to
    /// fixed values (M2-W4c — Ritual of Life; the D2 random Discover
    /// simplification, §19)
    SummonRandomCostMinionSetStats {
        /// The exact Cost of the summoned minion
        cost: i32,
        /// Attack value
        attack: i32,
        /// Health value
        health: i32,
    },
    /// Get a random Mask from another class; the Combo discount reduces
    /// the added Mask's Cost (M2-W4c — Costume Merchant; all five Masks
    /// are non-Rogue, so the fixed MASK_POOL is the full mask set)
    AddRandomMaskCombo {
        /// Cost reduction applied to the added Mask
        reduction: i32,
    },
    /// Gain the stats of a random Legendary Beast (M2-W4c — Beast Speaker
    /// Taka's battlecry, registered simplification §19: the official
    /// Discover is a random pick)
    GainStatsOfRandomLegendaryBeast,
    /// Summon a random Legendary Beast (M2-W4c — Beast Speaker Taka's
    /// deathrattle, registered simplification §19: the official summons
    /// the exact discovered Beast — an independent random pick)
    SummonRandomLegendaryBeast,
    /// Summon a random Taunt minion costing at least `min_cost` (M2-W4c —
    /// Atlasaurus's deathrattle; the D2 random Discover simplification,
    /// §19)
    SummonRandomTauntMinionCostGE {
        /// Minimum cost
        min_cost: i32,
    },
    /// Summon a random Taunt minion of each of three exact Costs (M2-W4c —
    /// Guard Duty's 6/4/2-Cost Taunts; the D2 random simplification, §19)
    SummonRandomTauntMinionsOfCosts {
        /// Highest Cost
        a: i32,
        /// Middle Cost
        b: i32,
        /// Lowest Cost
        c: i32,
    },
    /// Get a random 1-Cost minion (M2-W4c — Raptor-Nest Nurse's battlecry;
    /// the D2 random simplification, §19)
    AddRandomOneCostMinion,
    /// Get a random 1-Cost spell (M2-W4c — Raptor-Nest Nurse's deathrattle;
    /// the D2 random simplification, §19)
    AddRandomOneCostSpell,
    /// Get a random minion with multiple minion types (M2-W4c —
    /// Tortotem's end-of-turn effect; the fixed MULTI_TRIBE_MINION_POOL is
    /// the D2 simplification, §19)
    AddRandomMultiTribeMinion,
    // ----------------------------------------------------------------
    // M3-W2a (Across the Timeways — the 120 non-legendary TIME cards).
    // New CardEffect variants keep the bincode CardEffectDe mirror, the
    // Deserialize conversion and the bot scoring arms in lockstep.
    // ----------------------------------------------------------------
    /// Get a random minion that costs (N) less (TIME_000 Semi-Stable
    /// Portal — the CostModifier rides the drawn card)
    AddRandomMinionCostsLess {
        /// The cost reduction
        reduction: i32,
    },
    /// Get N random spells from the player's class (TIME_002 Aeon Wizard;
    /// the class is approximated by the union of the class card groups —
    /// the engine has no per-player class, §20)
    AddRandomSpellsFromClass {
        /// Number of spells
        count: i32,
    },
    /// Draw a random minion and give it +A/+H (TIME_003 Portal Vanguard —
    /// the draw filter picks a random minion card from the deck)
    DrawRandomMinionGiveStats {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// Both players discard a random card (TIME_008 Bygone Doomspeaker)
    BothPlayersDiscardRandomCard,
    /// Summon N Mana worth of random minions (TIME_014 Instant Multiverse
    /// — 12 Mana: random minions until the spent Mana reaches N)
    SummonManaWorthRandomMinions {
        /// Total Mana worth
        total: i32,
    },
    /// Get 2 random Holy spells and restore Health to the hero equal to
    /// their Costs (TIME_018 Mend the Timeline)
    GetHolySpellsRestoreHealthEqualCosts,
    /// Cast N random Nature spells (TIME_033 Druid of Regrowth — each cast
    /// runs the spell's effect against random targets where needed, §20)
    CastRandomNatureSpells {
        /// Number of spells to cast
        count: i32,
    },
    /// Both players equip a random weapon; the player's weapon gains
    /// +A/+H (TIME_034 Stadium Announcer)
    BothPlayersEquipRandomWeaponBuffOurs {
        /// Attack bonus on the player's weapon
        attack: i32,
        /// Health bonus on the player's weapon
        health: i32,
    },
    /// Get a random Rewind card (TIME_035 Time Machine — the
    /// REWIND_CARD_IDS pool, D2)
    AddRandomRewindCardToHand,
    /// Look at the right-most card in the opponent's hand; get a copy of
    /// it OR increase its Cost by 2 (TIME_036 Royal Informant — the
    /// either/or is a random pick, the established pool-open
    /// simplification, §20; **pool-open**, POOL_OPEN_CARDS)
    CopyRightmostEnemyHandCardOrIncreaseCost,
    /// Summon 2 random Legendary minions (TIME_038 Mister Clocksworth —
    /// the legendary pool spans the full active window, §20)
    SummonTwoRandomLegendaryMinions,
    /// Discover a copy of a card in the opponent's hand (TIME_039 Deja Vu
    /// — a real three-option choice over enemy hand cards; **pool-open**,
    /// POOL_OPEN_CARDS)
    DiscoverEnemyHandCardCopy,
    /// Summon a 0/4 Taunt, and another if holding a Dragon (TIME_006
    /// Mirror Dimension)
    SummonTauntAndIfHoldingDragonAgain {
        /// The Taunt token's card id
        card_id: &'static str,
    },
    /// Restore N Health to the hero and give them Divine Shield (TIME_015
    /// Hardlight Protector)
    RestoreAndGrantHeroDivineShield {
        /// Health restored
        amount: i32,
    },
    /// Discover a Paladin Mech from the past and give it +A/+H (TIME_016
    /// Neon Innovation — the pool is Paladin class Mechs, §20)
    DiscoverPaladinMechPastGiveStats {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// Deal N damage to all enemies if the player controls an Aura
    /// (TIME_019 Manifested Timeways — the check scans aura sources)
    DealDamageAllEnemiesIfControllingAura {
        /// Damage amount
        amount: i32,
    },
    /// The hero is Immune until the end of the turn (TIME_021 Doomsday
    /// Prepper's Outcast — the "until your next turn" expiry is the
    /// registered Kaldorei precedent, §20)
    GiveHeroImmuneThisTurn,
    /// Draw the bottom N cards of the deck (TIME_023 Contingency)
    DrawBottomCards {
        /// Number of cards
        count: i32,
    },
    /// Give all friendly minions +A/+H and shuffle 2 Shreds of Time into
    /// the deck (TIME_026 Entropic Continuity)
    BuffAllFriendlyMinionsShuffleShreds {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// Deal N damage split among all enemies and shuffle 2 Shreds of Time
    /// into the deck (TIME_027 Tachyon Barrage — split = N one-damage
    /// pings, the W4a split machinery)
    DealDamageSplitAmongAllEnemiesShuffleShreds {
        /// Total damage
        amount: i32,
    },
    /// Cast a Shred of Time from the deck to gain +A/+H (TIME_028
    /// Fatebreaker — "cast from deck" simplified: scan the deck for a
    /// Shred, remove it and apply its 3 damage to the hero, §20)
    CastShredFromDeckGainStats {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// Cast a Shred of Time from the deck to summon a copy of this
    /// (TIME_029 Ruinous Velocidrake — same cast-from-deck shape as
    /// TIME_028, §20)
    CastShredFromDeckSummonCopy,
    /// Copy a random minion in the player's hand (TIME_030 Divergence —
    /// the split-card halves mechanic is approximated by a copy, §20)
    CopyRandomHandMinion,
    /// Draw N cards of different Costs (TIME_031 RAFAAM LADDER!!)
    DrawCardsOfDifferentCosts {
        /// Number of cards
        count: i32,
    },
    /// Draw a minion and give minions in the hand +H Health (TIME_037
    /// Disciple of the Dove)
    DrawMinionAndBuffHandMinionsHealth {
        /// Health bonus
        health: i32,
    },
    /// Set a friendly minion's stats to A/H; it can't attack heroes this
    /// turn (TIME_043 PMM Infinitizer)
    SetStatsAndCantAttackHeroesThisTurn {
        /// Attack value
        attack: i32,
        /// Health value
        health: i32,
    },
    /// Gain +H Health for each turn taken this game (TIME_048 Clockwork
    /// Rager — the player turns_taken counter)
    GainStatsPerTurnTaken {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// At the start of the player's turn, transform into a random N-Cost
    /// minion (TIME_049 Dangerous Variant — the W4c transform shape)
    TransformSelfToRandomMinionOfCost {
        /// The target Cost
        cost: i32,
    },
    /// After this survives damage, swap its stats (TIME_050 Sentient
    /// Hourglass — the survival is predicted before the damage resolves,
    /// the Slam convention)
    SwapStatsIfSurvivesDamage,
    /// At the end of each player's turn, that player gets a Coin
    /// (TIME_054 Time Skipper)
    GiveCoin,
    /// After this survives damage, transform into a random N-Cost minion
    /// (TIME_055 Unknown Voyager)
    TransformSelfIfSurvivesDamageToRandomCost {
        /// The target Cost
        cost: i32,
    },
    /// Set the Cost of every card in both hands back to its original Cost
    /// (TIME_057 Wizened Truthseeker — cost enchantments are stripped)
    ResetBothHandsCosts,
    /// Summon a random N-Cost minion that is Dormant for T turns
    /// (TIME_058 Paltry Flutterwing)
    SummonRandomMinionOfCostDormant {
        /// The Cost of the summoned minion
        cost: i32,
        /// Dormant countdown
        turns: u32,
    },
    /// Summon a random Dragon that costs N or more (TIME_436 Past Conflux
    /// — the pool spans the active window, tokens excluded)
    SummonRandomDragonCostGE {
        /// The minimum Cost
        min_cost: i32,
    },
    /// Reverse the order of the player's deck (TIME_061 Timeless
    /// Causality)
    ReverseDeckOrder,
    /// If holding a Dragon, gain Taunt and Divine Shield (TIME_062
    /// Chronicle Keeper)
    GainTauntAndDivineShieldIfHoldingDragon,
    /// Add a random N-Cost minion to hand, marked to cost (1) less at
    /// each own turn start (TIME_102 Circadiamancer — the
    /// TurnCostReducer marker)
    AddRandomCostMinionMarkedTurnDiscount {
        /// The Cost of the minion
        cost: i32,
    },
    /// Deal D damage to a friendly minion to deal A damage to a random
    /// enemy minion (TIME_212 Lightning Rod)
    DealDamageFriendlyMinionToRandomEnemy {
        /// Damage to the friendly minion
        damage: i32,
        /// Damage to the random enemy minion
        amount: i32,
    },
    /// If the player has cast a Nature spell, gain +A/+H and draw a card
    /// (TIME_213 Primordial Overseer — the "while holding this" wording
    /// is approximated by the game-wide nature spell counter, §20)
    GainStatsAndDrawIfNatureSpellCast {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// Deal N damage to all minions and add the given card to hand
    /// (TIME_215 Thunderquake)
    DamageAllMinionsAndAddCardToHand {
        /// Damage amount
        amount: i32,
        /// The card id added to hand
        card_id: &'static str,
    },
    /// Deal N damage to a minion; if it survives, draw 2 cards (TIME_216
    /// Nascent Bolt)
    DamageAndDrawTwoIfSurvives {
        /// Damage amount
        damage: i32,
        /// The target set
        target: EffectTarget,
    },
    /// Deal N damage to a minion and give the hero +A Attack this turn
    /// (TIME_218 Static Shock — the hero attack is an until-end-of-turn
    /// enchantment)
    DamageMinionGiveHeroAttack {
        /// Damage amount
        damage: i32,
        /// Hero Attack gained
        attack: i32,
    },
    /// Deal damage to an enemy minion equal to this minion's Health
    /// (TIME_427 Cleansing Lightspawn)
    DealDamageEnemyMinionEqualToSourceHealth,
    /// Set the Attack and Health of every hand minion to the higher of
    /// the two stats (TIME_429 Divine Augur)
    SetHandMinionStatsToHigher,
    /// Restore Health to a character equal to this minion's Health
    /// (TIME_431 Amber Priestess)
    RestoreHealthEqualToSourceHealth,
    /// Discover a copy of a card from the player's deck and one from the
    /// opponent's hand (TIME_432 Intertwined Fate — one choice over both
    /// pools; the unpicked side is a random copy, §20; **pool-open**,
    /// POOL_OPEN_CARDS)
    DiscoverDeckAndEnemyHandCardCopy,
    /// Silence and destroy a random enemy minion (TIME_433 Cease to
    /// Exist)
    SilenceAndDestroyRandomEnemyMinion,
    /// Summon a shadow that attacks a random enemy minion (TIME_434
    /// Temporal Traveler's deathrattle)
    SummonShadowAttacksRandomEnemy {
        /// The shadow token's card id
        card_id: &'static str,
    },
    /// Summon two 3/2 Demons; if the deck has no minions they attack the
    /// lowest Health enemy (TIME_443 Hounds of Fury)
    SummonTwoDemonsAttackLowestHealthIfDeckNoMinions,
    /// Give a character Divine Shield and give hand minions +H Health
    /// (TIME_447 Power Word: Barrier)
    GrantDivineShieldAndBuffHandMinionsHealth {
        /// Hand minion Health bonus
        health: i32,
    },
    /// Discover a minion; if the deck has no minions, reduce the Cost of
    /// hand minions by N (TIME_448 Solitude — the discover-two is
    /// simplified to one pick, §20)
    DiscoverMinionReduceHandCostsIfDeckNoMinions {
        /// The cost reduction
        reduction: i32,
    },
    /// Give the hero +A Attack this turn; if the deck has no minions,
    /// give hand minions +A Attack (TIME_449 Lasting Legacy)
    GainHeroAttackAndBuffHandMinionsIfDeckNoMinions {
        /// Attack value
        attack: i32,
    },
    /// Deal N damage, or C if this is exactly in the center of the hand
    /// (TIME_600 Precise Shot — the center position is captured at play)
    PreciseShot {
        /// Base damage
        amount: i32,
        /// Center-of-hand damage
        center_amount: i32,
    },
    /// Draw until the hand has N cards (TIME_601 Arrow Retriever)
    DrawUntilHandSize {
        /// Target hand size
        size: i32,
    },
    /// Summon a random N-Cost Beast; it attacks a random enemy (TIME_602
    /// Wormhole — the forced attack runs through the normal pipeline)
    SummonRandomCostBeastAttackRandomEnemy {
        /// The Beast's Cost
        cost: i32,
    },
    /// Summon N copies of the given token; each gains two random Bonus
    /// Effects (TIME_610 Shadows of Yesterday)
    SummonMinionsGrantTwoRandomBonus {
        /// The token's card id
        card_id: &'static str,
        /// Number of copies
        count: i32,
    },
    /// Get a random Legendary minion with its Cost reduced by N (TIME_613
    /// Cryofrozen Champion's deathrattle)
    AddRandomLegendaryMinionCostReduced {
        /// The cost reduction
        reduction: i32,
    },
    /// If the hero's Health changed this turn, deal N damage to an enemy
    /// minion (TIME_614 Liferender)
    DealDamageEnemyMinionIfHeroHealthChanged {
        /// Damage amount
        amount: i32,
    },
    /// Fill the hand with random Undead that cost Health instead of Mana
    /// (TIME_615 Forgotten Millennium — the "this turn" expiry of the
    /// CostHealth marker is approximated, §20)
    FillHandWithRandomUndeadCostHealth,
    /// Summon the highest Cost friendly Undead that died this game
    /// (TIME_616 Memoriam Manifest)
    SummonHighestCostFallenUndead,
    /// At the end of the player's turn, summon a 3/5 Dragon with Taunt;
    /// the aura lasts N turns (TIME_700 Chronological Aura — the tick
    /// counter rides the player)
    SetChronologicalAura {
        /// Number of turns the aura lasts
        ticks: i32,
    },
    /// Discover a card from the deck; the others are put on the bottom
    /// (TIME_701 Waveshaping)
    DiscoverDeckCardOthersBottom,
    /// Deal N damage; if the player played a minion this turn, gain A
    /// Armor (TIME_702 Ebb and Flow — the "while holding this" wording
    /// is approximated by any-minion-played-this-turn, §20)
    DamageAndGainArmorIfMinionPlayedWhileHeld {
        /// Damage amount
        damage: i32,
        /// Armor gained
        armor: i32,
    },
    /// If the hero has N or less Health, gain +A/+H and summon a copy of
    /// this (TIME_703 Endangered Dodo)
    GainStatsAndSummonCopyIfHeroHealthLE {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
        /// The hero Health threshold
        threshold: i32,
    },
    /// Get a 2/2 Pupil and discover a spell that costs N or more from the
    /// past to teach it (TIME_704 Highborne Mentor — the teach link is
    /// approximated, §20)
    GetPupilAndDiscoverSpellCostGE {
        /// The pupil token's card id
        pupil_card_id: &'static str,
        /// The minimum spell Cost
        min_cost: i32,
    },
    /// Replace the hand and deck with random Choose One cards that cost
    /// (1) less (TIME_707 Alternate Reality — the pool spans the
    /// active-window choose-one cards, §20)
    ReplaceHandAndDeckWithRandomChooseOne,
    /// Summon two random N-Cost minions with +B Attack (TIME_711
    /// Flashback — the Combo bonus rides the same variant)
    SummonTwoRandomCostMinionsWithAttack {
        /// The Cost of the summoned minions
        cost: i32,
        /// Attack bonus on each
        bonus: i32,
    },
    /// Destroy a minion and summon a random N-Cost minion (TIME_712
    /// Dethrone's Combo branch)
    DestroyMinionAndSummonRandomCost {
        /// The Cost of the summoned minion
        cost: i32,
    },
    /// The opponent's cards cost N more next turn (TIME_716 Slow Motion —
    /// the tax rides the caster's player field, applied to the
    /// opponent's plays and cleared at the caster's next turn start)
    NextTurnEnemyCardsCostMore {
        /// The cost increase
        amount: i32,
    },
    /// Put N random Beasts on the bottom of the deck with +A/+H (TIME_730
    /// Kaldorei Cultivator — the discover-two is simplified to random
    /// adds, §20)
    AddRandomBeastsToBottomDeckWithStats {
        /// Number of Beasts
        count: i32,
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// Deal N damage; if holding a minion that costs C or more, draw a
    /// minion (TIME_750 Precursory Strike)
    DamageAndDrawMinionIfHoldingCostGE {
        /// Damage amount
        damage: i32,
        /// The hand minion's minimum Cost
        cost: i32,
    },
    /// Draw 2 cards; one is randomly picked to have its Cost reduced by N
    /// (TIME_770 Fast Forward — the pick is random, §20)
    DrawTwoReduceRandomCost {
        /// The cost reduction
        reduction: i32,
    },
    /// Deal P damage to an enemy and S damage to two other random enemies
    /// (TIME_855 Arcane Barrage)
    DealDamagePrimaryAndSplash {
        /// Primary damage
        primary: i32,
        /// Splash damage
        splash: i32,
    },
    /// Discover two Arcane spells from the past that cost (N) less
    /// (TIME_857 Alter Time — one pick, §20)
    DiscoverArcaneSpellsReduced {
        /// The cost reduction
        reduction: i32,
    },
    /// Deal N damage to an enemy minion and draw cards equal to the
    /// excess damage (TIME_858 Temporal Construct — the excess is
    /// computed after the damage resolves)
    DealDamageAndDrawExcess {
        /// Damage amount
        amount: i32,
    },
    /// Summon a random N-Cost and C-Cost minion; scramble their stats
    /// (TIME_859 Anomalize — each summoned minion's Attack and Health are
    /// swapped, the §20 shape)
    SummonPairScrambleStats {
        /// The first minion's Cost
        first_cost: i32,
        /// The second minion's Cost
        second_cost: i32,
    },
    /// Look at 2 random Secrets; one is cast for the player, the other
    /// for the opponent (TIME_860 Faceless Enigma — the pick is random,
    /// §20)
    LookAtSecretsGiveRandom,
    /// Summon a random minion from the deck and a tiger for the opponent
    /// (TIME_870 Gladiatorial Combat)
    SummonRandomDeckMinionAndTigerForOpponent {
        /// The tiger token's card id
        card_id: &'static str,
    },
    /// Gain +A/+H for each damaged minion (TIME_871 Heir of Hereafter)
    GainStatsPerDamagedMinion {
        /// Attack bonus
        attack: i32,
        /// Health bonus
        health: i32,
    },
    /// Fill the opponent's board with random 1-Cost minions (TIME_872
    /// Undefeated Champion)
    FillEnemyBoardWithRandomCost1Minions,
    /// Gain N Armor and summon two Beasts for the opponent (TIME_873
    /// Unleash the Crocolisks)
    GainArmorAndSummonTwoBeastsForOpponent {
        /// Armor gained
        armor: i32,
        /// The Beast token's card id
        card_id: &'static str,
    },
    /// Imprison an enemy minion — it goes Dormant for 10,000 turns
    /// (TIME_442 Timeway Warden's battlecry; the imprison/awaken link
    /// rides the player field, §20)
    ImprisonEnemyMinion,
    /// Awaken the minion this minion imprisoned (TIME_442 Timeway
    /// Warden's deathrattle)
    AwakenImprisonedMinion,
    /// Look at a random card in the opponent's hand and gain +H Health —
    /// the "guess" is always right because the looked-at card IS in the
    /// hand (TIME_041 Futuristic Forefather; **pool-open**, POOL_OPEN_CARDS,
    /// the §20 always-right simplification)
    GuessEnemyHandGainHealth {
        /// Health gained
        health: i32,
    },
    /// Transform this hand card into a random enemy-hand minion at the
    /// start of the owner's turn (TIME_876 Shapeshifter; **pool-open**,
    /// POOL_OPEN_CARDS — the transformation happens in the hand, the
    /// TurnStarted hook resolves the effect)
    TransformHandSelfToRandomEnemyHandMinion,
    /// Secret-context effect — resummon the minion that died (TIME_620
    /// Timecode the End; resolved by the secret system like
    /// ResurrectDiedMinion but WITHOUT the 1-Health damage — the
    /// official "resurrect it" resummons at full Health)
    ResurrectDiedMinionFull,
    // ===== M3-W2b — Across the Timeways legendary wave
    // (src/cards/exp_tmw_w2b.rs) — every variant stays in lockstep order
    // with the CardEffectDe mirror below (the
    // card_effect_de_mirror_order_matches guard).
    /// Summon a random minion from the owner's deck (TIME_009 Gelbin of
    /// Tomorrow — "put one of each Aura from your deck into the
    /// battlefield" approximated as one random deck minion, §21)
    SummonRandomMinionFromDeck,
    /// Deal N damage to all enemies, once plus once per Alleria/Vereesa
    /// played this game (TIME_609 Ranger General Sylvanas — the repeat
    /// count reads the played-minion log)
    SylvanasDealToAllEnemiesRepeated {
        /// Damage amount
        damage: i32,
    },
    /// The owner draws their 2 highest-Cost cards; the opponent draws the
    /// owner's 2 lowest-Cost cards (TIME_032 Chronogor — filtered deck
    /// draws from the owner's deck)
    ChronogorDrawsHighestLowest,
    /// Discard the owner's hand and add an Infinite Banana to it (TIME_042
    /// King Maluk)
    DiscardHandAndAddInfiniteBanana,
    /// Arm the owner's player record: at the start of the owner's NEXT
    /// turn, set this minion's Attack to the INFINITY cap (TIME_024
    /// Murozond, Unbounded — the two-tick hatching precedent, the flag is
    /// consumed by the TurnStarted hook)
    MurozondPrepareInfiniteAttack,
    /// Add a copy of the last N cards the owner played this game to the
    /// hand (TIME_103 Chromie — the official "draw another copy of each
    /// card you've played this game" approximated by the rewind history,
    /// §21)
    AddCopiesOfLastPlayedCards {
        /// How many played cards to copy
        count: i32,
    },
    /// Summon a Location card into the owner's location slot, replacing
    /// the current location (TIME_211 Lady Azshara's choose-one branches)
    SummonLocationForPlayer {
        /// The Location card's id
        card_id: &'static str,
    },
    /// Summon a copy of a random friendly minion (TIME_211t2 Zin-Azshari
    /// location activation)
    SummonCopyOfFriendlyMinion,
    /// Fill the owner's hand with random Temporary spells (TIME_211t1 The
    /// Well of Eternity location activation)
    FillHandWithRandomTemporarySpells,
    /// Take permanent control of an enemy minion whose Health is at most
    /// the SOURCE's Health (TIME_435 Eternus — "an enemy minion with this
    /// minion's Health or less"; the threshold is read from the source at
    /// resolution, the Mind Control precedent)
    TakeControlEnemyMinionHealthLE,
    /// Discover any Demon costing (5) or more; if the owner's deck has no
    /// minions, the next Demon the owner plays costs (1) (TIME_446 The
    /// Eternal Hold)
    DiscoverDemonGE5AndSetNextDemonCostOne,
    /// Spend up to N Corpses to restore that much Health to the owner's
    /// hero (TIME_618 Husk, Eternal Reaper — the hero-deathrattle
    /// resurrection approximated as an immediate spend, §21)
    SpendCorpsesRestoreHeroHealth {
        /// The corpse spend cap
        max: i32,
    },
    /// Draw Bwonsamdi from the deck, or resurrect him if he has died, and
    /// grant him the chosen Boon keyword (TIME_619 Talanji of the Graves
    /// — one instance per choose-one branch, so the draw+grant runs once)
    DrawOrResurrectBwonsamdiAndGrantBoon {
        /// The boon keyword granted
        keyword: KeywordKind,
    },
    /// Summon a random minion of the exact Cost (TIME_619t Bwonsamdi's
    /// deathrattle — "Summon a random 4-Cost minion"; the active-window
    /// filter is the Ritual of Life precedent)
    SummonRandomCostMinion {
        /// The exact cost of the summoned minion
        cost: i32,
    },
    /// Set the Costs of the bottom N cards of the owner's deck to (1)
    /// (TIME_705 Krona, Keeper of Eons)
    SetDeckBottomCostsOne {
        /// How many bottom cards to set
        count: i32,
    },
    /// Snapshot the owner's hand and replace it with draws from the deck;
    /// the end-of-turn restore swaps it back (TIME_706 The Fins Beyond
    /// Time — "your starting hand" approximated as fresh draws, §21)
    ReplaceHandAndSwapBackAtTurnEnd,
    /// Restore the hand snapshot taken by `ReplaceHandAndSwapBackAtTurnEnd`
    /// (TIME_706 The Fins Beyond Time end-of-turn effect)
    RestoreHandSnapshot,
    /// Summon a 0/8 Timeless Chest for the OPPONENT (TIME_713 Time
    /// Adm'ral Hooktail)
    SummonChestForOpponent,
    /// Fill the opponent's hand with Coins (TIME_713t Timeless Chest
    /// deathrattle)
    FillOpponentHandWithCoins,
    /// Destroy all minions the opponent played last turn (TIME_714
    /// Chrono-Lord Epoch — reads the opponent's per-turn played-id
    /// snapshot)
    DestroyAllMinionsOpponentPlayedLastTurn,
    /// Summon a Blood Fighter from the owner's hand, give it +5/+5, and it
    /// attacks a random enemy (TIME_850 Lo'Gosh, Blood Fighter)
    SummonBloodFighterFromHandBuffAndAttack,
    /// Get 3 random spells from the past, tracked per player; when all 3
    /// are played, add another TIME_861 Timelooper Toki to the hand
    /// (TIME_861 Toki)
    GetThreeRandomSpellsFromPastTracked,
    /// If the opponent is holding King Llane, destroy him and cut the
    /// enemy hero's Health in half (TIME_875 Garona Halforcen;
    /// **pool-open**, POOL_OPEN_CARDS)
    DestroyHeldKingLlaneAndHalveEnemyHealth,
    /// Silence and destroy all other minions (TIME_890 Medivh the
    /// Hallowed)
    SilenceAndDestroyAllOtherMinions,
    // ----------------------------------------------------------------
    // M3-W3 — the Across the Timeways closing wave (The End of Time
    // miniset, 38 END_ cards; fidelity-debt §22, en + zh).
    /// Deal damage to a target, then imbue the hero power (END_000
    /// Eventuality — "Deal 2 damage. Imbue your Hero Power"; the
    /// unqualified damage text follows the Elven Archer AnyCharacter
    /// convention)
    DealDamageAndImbue {
        /// Damage amount
        amount: i32,
        /// Damage target
        target: EffectTarget,
    },
    /// Draw an Undead, then imbue the hero power twice (END_003 Finality)
    DrawUndeadAndImbueTwice,
    /// Equip a 1/2 Dagger; when the owner already wields a weapon, give it
    /// +2 Attack instead (END_002 Wicked Blightspawn deathrattle)
    EquipDaggerOrBuffWeapon,
    /// Summon a random 4-Cost minion; spend 4 Corpses to summon another;
    /// when the spell was Outcast, summon a third (END_005 Bygone Echoes —
    /// the Outcast and corpse branches stack)
    BygoneEchoesSummon,
    /// Give the hero +3 Attack this turn, next turn, and the turn after
    /// (END_006 Chronikar battlecry — the battlecry applies the current
    /// turn's buff and arms the two-turn `chronikar_ticks` counter)
    ChronikarHeroAttackBuff,
    /// The start-of-turn half of END_006 Chronikar: while
    /// `chronikar_ticks` > 0, decrement it and re-apply the +3 Attack
    /// until end of turn (the third turn's buff)
    ChronikarRebuff,
    /// Deal 1 damage, give the hero +1 Attack this turn, draw a card, and
    /// gain 1 Armor (END_007 Press the Advantage — the four parts in one
    /// resolution)
    PressTheAdvantage,
    /// Refresh the owner's Mana Crystals by the given amount (END_008
    /// Enduring Roach — "After you use your Hero Power, refresh 2 Mana
    /// Crystals")
    RefreshManaCrystals {
        /// Mana refreshed
        amount: i32,
    },
    /// Summon two 2/2 Treants that gain +1/+1 for each friendly Treant
    /// that died this game (END_009 Splintered Reality — the per-player
    /// `treants_died_total` counter)
    SummonTwoTreantsScaling,
    /// Set the Attack of all other minions to the given value (END_010
    /// Twilight Timereaver choose-one branch 0 — "all other minions" =
    /// every minion on both boards except the source minion itself)
    SetAllOtherMinionsAttack {
        /// Attack value
        attack: i32,
    },
    /// Set the Health of all other minions to the given value (END_010
    /// Twilight Timereaver choose-one branch 1)
    SetAllOtherMinionsHealth {
        /// Health value
        health: i32,
    },
    /// Arm the owner's `acceleration_aura_ticks` counter to 3 (END_011
    /// Acceleration Aura — the ManaRefill step grants one temporary Mana
    /// Crystal and decrements at each of the owner's next three turn
    /// starts)
    ArmAccelerationAura,
    /// Set the equipped weapon's Attack to the INFINITY cap for this turn
    /// (END_012 Hand of Infinity — "Set this weapon's Attack to INFINITY
    /// this turn"; the cap is `exp_tmw_w2b::INFINITY_ATTACK_CAP`, §22)
    SetWeaponAttackInfinityThisTurn,
    /// Deal damage to an enemy; when it dies, give a random friendly
    /// minion +A/+H (END_014 Synchronized Spark — the predicted-death
    /// convention of DealDamageGainArmorIfKilled)
    DamageAndBuffFriendlyIfKilled {
        /// Damage amount
        amount: i32,
        /// Buff attack
        attack: i32,
        /// Buff health
        health: i32,
    },
    /// Get a random Deathrattle minion; it costs (2) less (END_015
    /// Triennium Rex — the shared Kindred OnPlay and Deathrattle effect)
    AddRandomDeathrattleMinionCostsLess,
    /// Discard the owner's highest-Cost hand card (END_016 Chronoclaws —
    /// "After your hero attacks, discard your highest Cost card")
    DiscardHighestCostCard,
    /// Draw cards until the hand is full (END_017t Tick and Tock
    /// battlecry and JAIL_430 Azalina Soulsever's battlecry, M5-W1 —
    /// the 10-card hand cap, F-A11)
    DrawUntilHandFull,
    /// Empty the opponent's hand — every card is destroyed (END_017t Tick
    /// and Tock deathrattle)
    EmptyOpponentHand,
    /// Set the Cost of a random card in the owner's hand to the INFINITY
    /// cap, recorded in `Player::hand_card_infinity` (END_018 Acolyte of
    /// Infinity battlecry — §22)
    SetRandomHandCardCostInfinity,
    /// Restore the recorded `hand_card_infinity` card's Cost to its def
    /// value when it is still in the hand (END_018 Acolyte of Infinity
    /// deathrattle)
    RestoreInfinityHandCardCost,
    /// Gain +A/+H when the owner's hero took damage this turn (END_019
    /// Endtime Survivor battlecry — reads `hero_damaged_this_turn`)
    GainStatsIfHeroDamagedThisTurn {
        /// Attack gain
        attack: i32,
        /// Health gain
        health: i32,
    },
    /// Deal 1 damage to a minion; when it survives, draw a card; when it
    /// dies, summon a random 1-Cost minion (END_020 Eternal Toil — the
    /// predicted-death convention)
    DamageMinionDrawIfSurvivesSummonIfDies {
        /// Damage amount
        amount: i32,
    },
    /// Give all minions and weapons in the owner's hand +A Attack (END_021
    /// Dimensional Weaponsmith battlecry)
    BuffHandMinionsAndWeapons {
        /// Attack bonus
        attack: i32,
    },
    /// Freeze a minion and its neighbors; destroy any that are damaged
    /// (END_023 Bitter End — the neighbors are the adjacent board slots)
    FreezeMinionAndNeighborsDestroyDamaged,
    /// Deal the INFINITY cap of damage to the enemy minion with the
    /// highest Health (END_024 Flames of Infinity secret — resolved by
    /// `secret.rs` with the TurnEnded context; §22)
    InfiniteDamageToHighestHealthEnemyMinion,
    /// Deal damage to a minion; when it dies, record it in
    /// `Player::eternal_flame_target` — the owner's turn end adds a fresh
    /// END_025 copy to the hand (the return-to-hand shape, §22)
    DamageMinionEternalFirebolt {
        /// Damage amount
        amount: i32,
    },
    /// Destroy all minions with 4 or less Attack (END_028 For All Time)
    DestroyAllMinionsWith4OrLessAttack,
    /// Add a random Shadow spell to the owner's hand (END_029 Voodoo
    /// Totem end-of-turn effect)
    AddRandomShadowSpell,
    /// Lock the given Overload for the owner's next turn, then gain
    /// Immune this turn and Windfury (END_032 Winged Aberration combo —
    /// the lock mirrors the overload play site, §22)
    OverloadForAndGainImmuneWindfury {
        /// Overload locked
        overload: i32,
    },
    /// Destroy a random enemy minion, the enemy's location, and the
    /// enemy's weapon (END_034 Crumblecrusher battlecry)
    DestroyRandomEnemyMinionLocationWeapon,
    /// Destroy the top 5 cards of the enemy deck when the owner's deck is
    /// empty (END_035 Omen of the End battlecry)
    DestroyTopFiveEnemyDeckIfOwnEmpty,
    /// Fill the owner's board with random Dragons, fully heal the hero,
    /// and skip the owner's next turn (END_037 Endtime Murozond battlecry
    /// — the skip is the `skip_next_turn` player flag, consumed at the
    /// owner's next TurnStarted)
    FillBoardRandomDragonsHealHeroSkipNextTurn,
    /// Get a random minion from another class; it costs the given amount
    /// less (END_000p Blessing of the Bronze Rogue hero power — the
    /// reduction scales with the imbue level; the official number is
    /// truncated in the dump, §22)
    GetRandomOtherClassMinionCostsLess {
        /// Cost reduction
        reduction: i32,
    },
    /// Buff the first Undead the owner plays this turn with +A Attack
    /// (END_003p Blessing of the Infinite Death Knight hero power — the
    /// hero-pinned trigger reads `undead_played_this_turn`, §22)
    BuffFirstUndeadPlayedEachTurn {
        /// Attack granted
        attack: i32,
    },
    /// Gain stats on the source AND on its Colossal main minion
    /// (2025–2026 expansions M4-W1 — Wickerfang's Legs: "At the end of
    /// your turn, gain +1/+1", copied to Wickerfang via the part→main
    /// link). Registered approximation: only the legs' own gains copy —
    /// an external buff on a leg does not (§23).
    GainStatsAndCopyToColossalMain {
        /// Attack gained on the part and the main
        attack: i32,
        /// Health gained on the part and the main
        health: i32,
    },
    /// Remove a keyword from the source's Colossal main minion
    /// (2025–2026 expansions M4-W1 — Chromatus's Heads: each head's
    /// deathrattle removes its own keyword from Chromatus, resolved via
    /// the part→main link).
    RemoveKeywordFromColossalMain {
        /// The keyword to remove from the main minion
        keyword: KeywordKind,
    },
    /// Get a random minion of the given cost that costs Health instead of
    /// Mana (2025–2026 expansions M4-W1 — Onyxia's Wing: "get a random
    /// {0}-Cost minion. It costs Health this turn" — the Herald {0} is
    /// fixed to 0, §23).
    AddRandomCostMinionCostsHealth {
        /// The minion's base cost
        cost: i32,
    },
    /// Destroy the friendly minion immediately to the right of the source
    /// to gain +A/+A (2025–2026 expansions M4-W1 — Cho's Arm / Gall's
    /// Arm, §23). While the owner has a friendly Cho'gall on the board,
    /// a random minion in the ENEMY's deck is destroyed instead.
    ColossalArmDestroyRight {
        /// Attack gained when a minion is destroyed
        attack: i32,
        /// Health gained when a minion is destroyed
        health: i32,
    },
    /// Get a random Fire spell; it costs `reduction` less (2025–2026
    /// expansions M4-W1 — Plume of Vulcanos: "Whenever this takes damage,
    /// get a random Fire spell. It costs (3) less." — the Fire filter
    /// reads the quest registry's official spell-school table).
    AddRandomFireSpellCostsLess {
        /// Cost reduction for the added spell
        reduction: i32,
    },
    /// Trigger the Deathrattles of ALL friendly on-board minions, once
    /// each, in play order (2025–2026 expansions M4-W1 — Ragnaros, the
    /// Great Fire: "At the end of your turn, trigger your minions'
    /// Deathrattles").
    TriggerFriendlyDeathrattles,
    /// Get `count` random minions whose base cost equals the source's
    /// attack; each costs (1) (2025–2026 expansions M4-W1 — Al'Akir,
    /// Lord of Storms' battlecry; the cost-set uses the roadmap G5
    /// set-to-value cost modifier).
    AddRandomMinionsCostEqualAttack {
        /// Number of minions to get
        count: i32,
    },
    /// Give a random friendly minion `attack` more Attack (2025–2026
    /// expansions M4-W1 — Magmaw's Body deathrattle, a fixed +2).
    GiveRandomFriendlyMinionAttack {
        /// Attack gained by the random friendly minion
        attack: i32,
    },
    /// Summon `count` copies of a minion, then grant ALL friendly minions
    /// "Deathrattle: summon the same minion" (2025–2026 expansions M4-W3
    /// — CATA_134 Wildwood Circle's combined form; the
    /// GrantDeathrattleAll Soul-of-the-Forest convention applied AFTER
    /// the summon, so the fresh tokens receive the deathrattle too).
    SummonMinionsAndGrantDeathrattleAll {
        /// Card ID to summon (and to attach as the deathrattle summon)
        card_id: &'static str,
        /// Number of minions to summon
        count: u32,
    },
    /// Give a friendly minion stats and Elusive, then summon a copy of it
    /// (2025–2026 expansions M4-W3 — CATA_306 Schism's combined form; one
    /// pick feeds all three parts, and the copy is the base card — the
    /// SummonCopyOfFriendlyMinion convention).
    GainStatsElusiveAndSummonCopy {
        /// Attack gained by the target
        attack: i32,
        /// Health gained by the target
        health: i32,
    },
    /// Summon `count` copies of a minion, then give every friendly minion
    /// extra Attack and Divine Shield (2025–2026 expansions M4-W3 —
    /// CATA_479 Flight Maneuvers' combined form; the buff applies after
    /// the summon, so the fresh Drakes receive it too — the
    /// Longneck-Egg convention).
    SummonMinionsAndGrantFriendlyAttackDivineShield {
        /// Card ID to summon
        card_id: &'static str,
        /// Number of minions to summon
        count: u32,
        /// Attack gained by every friendly minion
        attack: i32,
    },
    /// Deal damage to a character, then deal damage to all enemies
    /// (2025–2026 expansions M4-W3 — CATA_489 Arcane Flow's combined
    /// form; the primary part targets any character — the official
    /// "$4 damage" has no target filter, the Rite-of-Twilight §24 pin —
    /// and the splash hits the enemy hero too).
    DealDamageAndDamageAllEnemies {
        /// Primary damage (targeted)
        amount: i32,
        /// Damage dealt to all enemies
        aoe: i32,
    },
    /// Draw `count` random minions from the deck, then give every minion
    /// in hand stats (2025–2026 expansions M4-W3 — CATA_820 Supply Run's
    /// combined form; the draw-minions convention of
    /// DrawMinionAndBuffHandMinionsHealth, with the hand buff applied
    /// after the draws so the drawn minions receive it too).
    DrawMinionsAndBuffHandMinions {
        /// Number of minions to draw
        count: u32,
        /// Attack gained by every minion in hand
        attack: i32,
        /// Health gained by every minion in hand
        health: i32,
    },
    /// Get a random Shatter card from another class (2025–2026 expansions
    /// M4-W3 — CATA_202 Stolen Power; the fixed `SHATTER_POOL` — the
    /// other 5 Shatter cards, all non-Rogue — is the D2 random
    /// simplification, the Mask-pool convention, §25; the gotten card is
    /// the combined playable form).
    AddRandomShatterCardToHand,
    /// Replace the owner's hero with a fresh copy of `card_id` (2025–2026
    /// expansions M4-W4 — the hero-replacement primitive, pays the M2 §15
    /// Master Dusk "when real" note). The old hero entity moves to the
    /// graveyard; the new hero's health is the def's, accumulated damage
    /// and attack are cleared, the hero power comes from the def (or the
    /// cards-side table), the equipped weapon is destroyed, and the old
    /// hero's enchantments are left behind with it.
    ReplaceHero {
        /// Card ID of the replacement hero (a `CardType::Hero` def)
        card_id: &'static str,
    },
    /// "Choose a Cataclysm to unleash!" (2025–2026 expansions M4-W4 —
    /// CATA_190h Deathwing, Worldbreaker's battlecry). Surfaces a
    /// `ChoiceKind::Cataclysm` pending choice whose option count is the
    /// herald tier — `herald_number(1, herald_count)`: 1 pick at counter
    /// 0–1, 2 at 2–3, 4 at 4+ ("Herald twice to upgrade"). Distinct
    /// picks: each resolution re-surfaces the choice with the picked
    /// Cataclysm removed from the pool.
    ChooseCataclysms,
    /// "Choose a card in your hand" (2025–2026 expansions M4-W4 — the
    /// shared hand-pick machinery of CATA_200/209/477/490/563/566/697/
    /// 721/897/979). Surfaces a `ChoiceKind::ChooseHandCard` pending
    /// choice whose pool is the owner's hand card ids; the resolution
    /// dispatches to the cards-side `choose_hand_card` table keyed by the
    /// source card id (the Agent-of-the-Old-Ones convention).
    ChooseHandCard,
    /// Refresh Mana Crystals if the owner is holding a Dragon (2025–2026
    /// expansions M4-W4 — CATA_111 Darkscale Broodmother's "if you're
    /// holding a Dragon, refresh 2 Mana Crystals").
    RefreshManaIfHoldingDragon {
        /// Mana refreshed when holding a Dragon
        amount: i32,
    },
    /// Gain a temporary Mana Crystal — permanent if the owner spent the
    /// threshold Mana while holding this card (2025–2026 expansions M4-W4
    /// — CATA_131 Felwood Treant. Registered: the official "while holding
    /// this" accumulation is approximated by the per-turn spent counter,
    /// §26).
    GainTempManaOrPermanentIfSpent {
        /// Mana spent this turn that makes the crystal permanent
        threshold: i32,
    },
    /// Get two 3/3 Taunt Whelps — summon them instead if the owner spent
    /// the threshold Mana while holding this (2025–2026 expansions M4-W4
    /// — CATA_132 Broodwatcher; the spent-while-holding approximation of
    /// GainTempManaOrPermanentIfSpent, §26).
    GetOrSummonTauntWhelpsIfSpent {
        /// Mana spent this turn that summons instead of getting
        threshold: i32,
    },
    /// Summon two 1/2 Golems, then spend all the owner's Mana to give
    /// them +1/+1 for each Mana spent (2025–2026 expansions M4-W4 —
    /// CATA_135 Mossbinding).
    SummonGolemsSpendAllMana,
    /// Shuffle 5 random minions that cost (8) or more into the deck with
    /// doubled stats (2025–2026 expansions M4-W4 — CATA_136 Azshara's
    /// Triumph; the shuffled copies carry a doubled-stats enchantment).
    ShuffleRandomMinionsCostGE8DoubleStats,
    /// Give a friendly minion +1/+1 for each minion the owner controls
    /// (2025–2026 expansions M4-W4 — CATA_138 Forest's Gift; the
    /// targeted version of GainStatsPerFriendlyMinion — the count
    /// includes the target itself).
    GainStatsPerFriendlyMinionTargeted,
    /// Fill the owner's hand with random Dragons; if the owner spent the
    /// threshold Mana while holding this, they cost (1) (2025–2026
    /// expansions M4-W4 — CATA_140 Merithra of the Dream; the
    /// spent-while-holding approximation of §26).
    FillHandWithRandomDragons {
        /// Mana spent this turn that makes the Dragons cost (1)
        threshold: i32,
    },
    /// Give a friendly minion on the battlefield Attack equal to the
    /// source's Attack (2025–2026 expansions M4-W4 — CATA_161 Gruesome
    /// Nightmare. Registered: the official "minion in your hand or
    /// battlefield" target set is approximated by the battlefield side,
    /// §26).
    SetAttackEqualToSource,
    /// The owner's next Murloc that costs (3) or less costs Health
    /// instead of Mana (2025–2026 expansions M4-W4 — CATA_180 War'loc;
    /// the one-time flag is consumed by the next eligible Murloc play,
    /// the CostHealth convention, §26).
    NextMurlocCostsHealth,
    /// Transform a random enemy minion into a copy of the source
    /// (2025–2026 expansions M4-W4 — CATA_185 Faceless Replicator's
    /// deathrattle. Registered: the official "the minion that killed
    /// this" killer-identification is unmodeled — the deathrattle hits a
    /// random enemy minion instead, §26).
    TransformRandomEnemyMinionToSelf,
    /// Give the opponent a 2-Cost Sabotage (2025–2026 expansions M4-W4 —
    /// CATA_186 Stickybomb Saboteur. Registered: the official "cards
    /// next to it cost (1) more" hand-adjacency aura is unmodeled — the
    /// gotten Sabotage is a blank 2-cost spell, §26).
    GiveOpponentSabotage,
    /// Return an enemy minion to its owner's hand; it can't be played
    /// next turn (2025–2026 expansions M4-W4 — CATA_215 Daze; the
    /// returned entity carries the CantPlayNextTurn marker, cleared at
    /// the owner's turn end).
    ReturnEnemyMinionCantPlayNextTurn,
    /// The owner's healing effects restore `amount` more Health this game
    /// (2025–2026 expansions M4-W4 — CATA_216 Cleansing Cleric; the
    /// permanent heal bonus is added by the heal pipeline).
    SetHealingBonus {
        /// Bonus Health restored by every friendly heal
        amount: i32,
    },
    /// The owner's next Healing effect this turn deals damage instead
    /// (2025–2026 expansions M4-W4 — CATA_301 Ruby Sanctum; the flag is
    /// consumed by the next friendly heal and cleared at turn end).
    SetNextHealDealsDamage,
    /// Restore a minion to full Health, then draw a card (2025–2026
    /// expansions M4-W4 — CATA_302 Mend).
    FullHealMinionAndDraw,
    /// Deal `amount` damage to a minion; if it dies, restore `amount`
    /// Health to the enemy hero (2025–2026 expansions M4-W4 — CATA_303
    /// Purifying Breath; the kill check runs after the damage resolves).
    DamageMinionHealEnemyHeroIfKilled {
        /// Damage dealt to the minion (and the heal on its death)
        amount: i32,
    },
    /// Lifesteal Battlecry — deal `amount` damage to the source and heal
    /// the owner's hero for the same amount (2025–2026 expansions M4-W4 —
    /// CATA_304 Injured Attendant; registered: the official lifesteal
    /// heals for the damage actually dealt, simplified to the exact
    /// amount, §26).
    LifestealSelfDamage {
        /// Damage dealt to the source
        amount: i32,
    },
    /// At the end of the owner's turn, if the source is at full Health,
    /// gain `amount` Health (2025–2026 expansions M4-W4 — CATA_305
    /// Incensed Matriarch's end-of-turn effect).
    GainHealthIfFullHealth {
        /// Health gained at full Health
        amount: i32,
    },
    /// Set the owner's remaining Health to `health`; when the hero next
    /// reaches full Health, deal `damage` to the opponent (2025–2026
    /// expansions M4-W4 — CATA_307 Alexstrasza, Guardian of Life; the
    /// full-Health watcher is a player flag consumed by the heal
    /// pipeline's FriendlyHeroHealedToFull event).
    SetRemainingHealthAndFullHealDamage {
        /// Health the owner's hero is set to
        health: i32,
        /// Damage dealt to the opponent when the hero reaches full Health
        damage: i32,
    },
    /// Give all spells in the owner's hand and deck Spell Damage +1
    /// (2025–2026 expansions M4-W4 — CATA_458 Archmage Kalec; the spell
    /// entities carry the SpellDamage component, which the cast pipeline
    /// adds to the damage).
    BuffSpellDamageHandAndDeck,
    /// Get a 2-Cost spell that deals the source's Attack damage
    /// (2025–2026 expansions M4-W4 — CATA_464 Blackwing Experiment's
    /// deathrattle; the source's Attack is stashed in the player's
    /// `dragon_breath_damage` flag, read by DealDamageEqualDragonBreath —
    /// the baked-value convention, §26).
    AddDragonBreathScaledByAttack,
    /// Deal damage equal to the player's stashed `dragon_breath_damage`
    /// (2025–2026 expansions M4-W4 — CATA_464t Dragon Breath's spell
    /// effect; the flag is set by AddDragonBreathScaledByAttack).
    DealDamageEqualDragonBreath,
    /// Summon five 5/4 Undead Drakes; if the owner has 8 Corpses, spend
    /// them to give the Drakes Rush (2025–2026 expansions M4-W4 —
    /// CATA_465 Chow Down).
    SummonFiveHungryDrakesSpendCorpsesRush,
    /// Refresh Mana Crystals equal to the source's Attack (2025–2026
    /// expansions M4-W4 — CATA_469 Chromatic Broodmother's "whenever this
    /// attacks" trigger effect).
    RefreshManaEqualSelfAttack,
    /// Get the crafted Undead Dragon; if the owner is holding a Dragon,
    /// reduce its Cost by `reduction` (2025–2026 expansions M4-W4 —
    /// CATA_470 Victor Nefarius. Registered: the official "craft a custom
    /// Undead Dragon" choice is fixed to the Nefarian's Creation token,
    /// §26).
    AddNefariansCreation {
        /// Cost reduction when holding a Dragon
        reduction: i32,
    },
    /// Give every friendly minion "Deathrattle: summon a random minion
    /// that costs `cost`" (2025–2026 expansions M4-W4 — CATA_471
    /// Talanji's Last Stand; the Soul-of-the-Forest grant convention).
    GrantDeathrattleSummonRandomCostMinion {
        /// Cost of the deathrattle-summoned minion
        cost: i32,
    },
    /// Trigger a random friendly minion's end-of-turn effect once
    /// (2025–2026 expansions M4-W4 — CATA_472 Inspiring Maul's
    /// deathrattle; the triggered effect is the target's TurnEnd trigger
    /// effect, resolved with the target as the source).
    TriggerRandomFriendlyEndTurnEffect,
    /// Give every friendly minion Divine Shield; any that already had one
    /// gain +A/+H instead (2025–2026 expansions M4-W4 — CATA_473
    /// Nozdormu, Bronze Aspect's end-of-turn effect).
    GrantDivineShieldOrGainStats {
        /// Attack gained by minions that already had Divine Shield
        attack: i32,
        /// Health gained by minions that already had Divine Shield
        health: i32,
    },
    /// Get a random Holy spell; reduce its Cost by `reduction`
    /// (2025–2026 expansions M4-W4 — CATA_474 Spearheart Sentry's
    /// end-of-turn effect).
    AddRandomHolySpellCostReduced {
        /// Cost reduction for the added spell
        reduction: i32,
    },
    /// Summon a Dragon with stats equal to the source's (2025–2026
    /// expansions M4-W4 — CATA_478 Bronze Redeemer's end-of-turn effect;
    /// the token carries a copied-stats enchantment).
    SummonDragonWithSelfStats,
    /// The owner's minions' end-of-turn effects trigger twice, for
    /// `turns` turns (2025–2026 expansions M4-W4 — CATA_480 Sandfury
    /// Aura; the tick counter is consumed by the owner's turn ends).
    SetEndTurnEffectsTwice {
        /// Number of turns the doubling lasts
        turns: u32,
    },
    /// Devour 2 random cards from the opponent's hand, then the source
    /// goes Dormant for 2 turns (2025–2026 expansions M4-W4 — CATA_481
    /// Iso'rath's battlecry; the devoured entities are stashed in the
    /// player's `devoured_cards` list, returned by
    /// ReturnDevouredCards).
    DevourTwoEnemyHandCardsAndDormant,
    /// Return the devoured cards to the opponent's hand (2025–2026
    /// expansions M4-W4 — CATA_481 Iso'rath's deathrattle).
    ReturnDevouredCards,
    /// If the owner dealt damage with a spell this turn, summon a copy of
    /// the source (2025–2026 expansions M4-W4 — CATA_483 Unstable
    /// Spellcaster's battlecry).
    SummonCopyIfSpellDamageDealtThisTurn,
    /// Deal `amount` damage to a character, then deal `secondary` damage
    /// to a random enemy minion (2025–2026 expansions M4-W4 — CATA_485
    /// Sleet Storm's combined form; the primary target is any character,
    /// the Arcane Flow §24 pin).
    DealDamageAndDamageRandomEnemyMinion {
        /// Primary damage (targeted)
        amount: i32,
        /// Damage dealt to a random enemy minion
        secondary: i32,
    },
    /// The first time the owner deals damage with a spell each turn, gain
    /// `attack` Attack (2025–2026 expansions M4-W4 — CATA_487
    /// Raincaller's trigger effect; the once-per-turn guard rides the
    /// player's per-turn flag).
    GainAttackFirstSpellDamageThisTurn {
        /// Attack gained
        attack: i32,
    },
    /// Deal `amount` damage to all minions, repeating with 1 less damage
    /// until 0 (2025–2026 expansions M4-W4 — CATA_491 Eldritch
    /// Tentacles: "deal $3 damage to all minions. Repeat this with 1 less
    /// damage.").
    DamageAllMinionsRepeatDescending {
        /// Starting damage
        amount: i32,
    },
    /// Summon a copy of the discarded minion (2025–2026 expansions M4-W4
    /// — CATA_494 Maloriak's "after you discard a minion" trigger
    /// effect; the event subject is the discarded minion).
    SummonCopyOfDiscardedMinion,
    /// Take control of an enemy minion until the end of the owner's turn;
    /// it can't attack this turn (2025–2026 expansions M4-W4 — CATA_496
    /// Cursed Chains; registered: the official "until the end of THEIR
    /// turn" timing is approximated by the standard until-end-of-turn
    /// convention, §26).
    TakeControlUntilEndOfTurnCantAttack,
    /// Deal `base` plus the number of turns this card has spent in the
    /// owner's hand damage to two random enemy minions (2025–2026
    /// expansions M4-W4 — CATA_498 Rafaams' Last Stand's "(Upgrades each
    /// turn!)"; the hand-turn counter is bumped at the owner's turn start
    /// for the marked card).
    DealDamageToTwoScaledByHandTurns {
        /// Base damage before the hand-turn upgrade
        base: i32,
    },
    /// Deal `amount` damage to all minions; draw a card for each that
    /// died (2025–2026 expansions M4-W4 — CATA_526 Broxigar's Last
    /// Stand).
    DamageAllMinionsDrawPerDeath {
        /// Damage dealt to each minion
        amount: i32,
    },
    /// Reopen the location — it becomes ready again this turn (2025–2026
    /// expansions M4-W4 — CATA_527 Nespirah, Enthralled's "after you cast
    /// a Fel spell, reopen" trigger effect; the Fel check is id-keyed in
    /// the trigger resolution).
    ReopenLocation,
    /// At the start of the owner's next turn, summon the given card
    /// (2025–2026 expansions M4-W4 — CATA_528 Sigil of the Seas; the
    /// pending summon is stored in the player's `next_turn_summon` and
    /// consumed by the TurnStarted hook, the flock-pending pattern).
    SetNextTurnSummon {
        /// Card ID summoned at the owner's next turn start
        card_id: &'static str,
    },
    /// Deal `amount` damage to the opponent's left and right-most enemy
    /// minions; if the spell was played from the hand edge (Outcast), do
    /// it again (2025–2026 expansions M4-W4 — CATA_533 Flash Flood).
    DealDamageLeftRightOutcastAgain {
        /// Damage dealt to the two edge minions
        amount: i32,
    },
    /// Deal damage equal to the source's Attack to a random character
    /// (2025–2026 expansions M4-W4 — CATA_552 Ebonscale Scout's
    /// battlecry; the official text carries no target filter, the
    /// AnyCharacter Arcane Flow §24 pin).
    DealDamageEqualSelfAttack,
    /// The owner's Dragons have Rush this game (2025–2026 expansions
    /// M4-W4 — CATA_553 Ebyssian's battlecry; the permanent flag is read
    /// by the summon path).
    SetDragonsHaveRush,
    /// Set an enemy minion's Health to 1; if the owner is holding a
    /// Dragon, set another random enemy minion's Health to 1 too
    /// (2025–2026 expansions M4-W4 — CATA_554 Earthen Roar; the explicit
    /// target wins for the first pick).
    SetEnemyMinionHealthTo1IfHoldingDragon,
    /// Get a random Dragon that costs (3) or less (2025–2026 expansions
    /// M4-W4 — CATA_556 Carrier Whelp's battlecry; the D2 pool samples
    /// the active window).
    AddRandomDragonCostLE3,
    /// Deal `amount` damage to a character; if the owner has played
    /// another copy of this card this game, damage all enemies instead
    /// (2025–2026 expansions M4-W4 — CATA_557 Sylvanas's Triumph; the
    /// played-copy flag is set at the card's play).
    DealDamageOrDamageAllEnemiesIfCopyPlayed {
        /// Damage dealt
        amount: i32,
    },
    /// Replay each 1-Cost card the owner has played this game, targeting
    /// enemies if possible (2025–2026 expansions M4-W4 — CATA_560
    /// Confront the Tol'vir. Registered: spells resolve their spell
    /// effect against enemy targets, minions are summoned, §26).
    ReplayOneCostCardsPlayedThisGame,
    /// Cast the spell the source absorbed (2025–2026 expansions M4-W4 —
    /// CATA_563 Crackling Cloudstrider's deathrattle; the absorbed spell
    /// entity is stashed in the player's `absorbed_spell` field).
    CastAbsorbedSpell,
    /// Give a friendly minion Mega-Windfury; it can't attack heroes
    /// (2025–2026 expansions M4-W4 — CATA_564 Air Support's battlecry;
    /// the MegaWindfury marker grants 4 attacks, the CantAttackHeroes
    /// marker the restriction).
    GrantMegaWindfuryCantAttackHeroes,
    /// Transform all friendly minions into random minions that cost (1)
    /// more; the transformed minions summon the originals when they die
    /// (2025–2026 expansions M4-W4 — CATA_567 Ascendance. Registered: the
    /// transform sampling and the deathrattle grant are the D2
    /// approximation, §26).
    TransformFriendlyMinionsCost1MoreSummonOriginals,
    /// Summon a random 3-, 2-, and 1-Cost minion (2025–2026 expansions
    /// M4-W4 — CATA_569 Ceremonial Clash).
    SummonRandomThreeTwoOneCostMinions,
    /// Draw a card and reduce its Cost by `reduction`; repeat this with
    /// the excess Cost reduction (2025–2026 expansions M4-W4 — CATA_570
    /// Morchok's battlecry; the loop continues while the reduction
    /// remains and the deck has cards).
    DrawAndReduceCostRepeated {
        /// Cost reduction carried through the draws
        reduction: i32,
    },
    /// Deal `base` plus one per minion on the battlefield damage to all
    /// minions (2025–2026 expansions M4-W4 — CATA_581 Decimation's
    /// "(Improved for each minion on the battlefield)"; registered: the
    /// improvement counts ALL minions on both sides, §26).
    DamageAllMinionsScaledByBoard {
        /// Base damage before the battlefield improvement
        base: i32,
    },
    /// Deal `amount` damage to all minions and give the owner's hero
    /// `attack` Attack this turn (2025–2026 expansions M4-W4 — CATA_582
    /// Searing Fissure).
    DamageAllMinionsAndGainHeroAttack {
        /// Damage dealt to all minions
        amount: i32,
        /// Attack gained by the hero this turn
        attack: i32,
    },
    /// Deal `amount` damage randomly split among enemies; if the owner
    /// played a Fire spell this turn, deal `amount` more (2025–2026
    /// expansions M4-W4 — CATA_584 Erupting Volcano; the Fire check reads
    /// the quest registry's spell-school table).
    DealDamageSplitAmongEnemiesIfFireSpell {
        /// Damage randomly split among enemies
        amount: i32,
    },
    /// Deal `amount` damage to a damaged minion; if it dies, return this
    /// spell to the owner's hand (2025–2026 expansions M4-W4 — CATA_585
    /// Torch. Registered: the official "return with any excess damage" is
    /// approximated by the die-check, §26).
    DamageDamagedMinionReturnIfExcess {
        /// Damage dealt to the damaged minion
        amount: i32,
    },
    /// Instead of drawing each turn, Discover a card from the owner's
    /// deck; it costs (3) less and the others are destroyed (2025–2026
    /// expansions M4-W4 — CATA_591 Commander Geddon's battlecry; the
    /// game-long flag is read by the DrawStep hook, registered §26).
    SetGeddonDiscoverDraw,
    /// Give a minion "Deathrattle: summon a random minion from your hand"
    /// (2025–2026 expansions M4-W4 — CATA_610 Lo'Gosh's Last Stand.
    /// Registered: the official side-agnostic target is approximated by
    /// the friendly scope, §26).
    GrantDeathrattleSummonRandomHandMinion,
    /// Summon a random minion from the owner's hand (2025–2026 expansions
    /// M4-W4 — the deathrattle granted by CATA_610).
    SummonRandomMinionFromHand,
    /// Upgrade the owner's starting Hero Power; it costs (1) (2025–2026
    /// expansions M4-W4 — CATA_615t Genn, Worgen King's battlecry.
    /// Registered: the official upgrade replaces the hero power with the
    /// Worgen King version — approximated by the cost (1) flag, §26).
    UpgradeHeroPowerCost1,
    /// Get a random Paladin Aura; it lasts an additional turn (2025–2026
    /// expansions M4-W4 — CATA_621 Gelbin's Triumph. Registered: the
    /// aura pool is fixed to random Paladin spells and the extra turn is
    /// unmodeled, §26).
    AddRandomPaladinAura,
    /// Steal `amount` Health from the chosen enemy minion, three times
    /// (2025–2026 expansions M4-W4 — CATA_699 Dread Leviathan's battlecry
    /// with an explicit enemy-minion target. Registered: each steal is
    /// dealt as damage to the target and a matching heal on the source,
    /// §26).
    StealHealthThreeTimes {
        /// Health stolen per iteration
        amount: i32,
    },
    /// Destroy all cards that cost (2) or less in both players' decks
    /// (2025–2026 expansions M4-W4 — CATA_720 Warmaster Blackhorn's
    /// battlecry).
    DestroyDeckCardsCostLE2Both,
    /// Unlock the owner's Overloaded Mana Crystals (2025–2026 expansions
    /// M4-W4 — CATA_724 Stormbinder's deathrattle; the overload-lock
    /// field is cleared like the ManaRefill step does).
    UnlockOverloadedCrystals,
    /// After the owner casts a spell, cast a random spell of the same
    /// Cost from another class (2025–2026 expansions M4-W4 — CATA_786
    /// Chaos Supplicant's trigger effect; the D2 pool samples the active
    /// window's spells of the cast spell's cost from the other classes).
    CastRandomSpellSameCostOtherClass,
    /// Return the card the source discarded to the owner's hand; it costs
    /// (1) less (2025–2026 expansions M4-W4 — CATA_897 Gemstone Hoarder's
    /// deathrattle; the hoarded card is stashed in the player's
    /// `hoarded_card` field by the choose-hand-card discard).
    ReturnHoardCostLess,
    /// Deal `amount` damage to a minion; reduce the Cost of a random card
    /// in the owner's hand by the excess damage (2025–2026 expansions
    /// M4-W4 — CATA_978 Sindragosa's Triumph. Registered: the excess is
    /// the damage minus the target's remaining Health, §26).
    DamageMinionReduceHandCostByExcess {
        /// Damage dealt to the minion
        amount: i32,
    },
    /// Split the chosen spell into two random spells of the same Cost
    /// (2025–2026 expansions M4-W4 — CATA_979 Conjuration Specialist's
    /// choose-hand-card dispatch; the chosen spell leaves the hand and
    /// two random spells of its Cost are added, §26).
    AddTwoRandomSpellsSameCost {
        /// Cost of the two random replacement spells
        cost: i32,
    },
    /// If the total Cost of the minions in the owner's deck is at least
    /// the threshold, split 100 stats among minions in the deck
    /// (2025–2026 expansions M4-W4 — CATA_213 Vyranoth's battlecry.
    /// Registered: the official "starting minions" deck snapshot is
    /// approximated by the deck at play time, and the 100 stats are split
    /// as +5/+5 on ten random deck minions, §26).
    Split100StatsAmongDeckMinionsIfCostGE100 {
        /// Minimum total deck-minion Cost for the split
        threshold: i32,
    },
    /// Destroy the enemy minion with the highest Health (2025–2026
    /// expansions M4-W4 — CATA_190t11 Topple, one of Deathwing's four
    /// Cataclysms: "Destroy the highest-Health enemy minion"). Ties break
    /// by the leftmost (first-played) minion, like the engine's other
    /// highest-attribute picks.
    DestroyHighestHealthEnemyMinion,
    /// Shuffle five random Legendary Dragons into the deck, each costing
    /// (1) (2025–2026 expansions M4-W4 — CATA_190t13 Enthrall, one of
    /// Deathwing's four Cataclysms; the LegendaryDragon pool — §26).
    ShuffleRandomLegendaryDragonsCost1,
    /// Destroy a Legendary minion (2025–2026 expansions M4-W4 — CATA_203
    /// Garona's Last Stand; the Legendary filter follows the
    /// `LegendaryMinion` pool convention, §26).
    DestroyLegendaryMinion,
    /// Give a random friendly minion Attack (2025–2026 expansions M4-W4 —
    /// CATA_467 Command Claw's weapon trigger).
    GrantRandomFriendlyMinionAttack {
        /// Attack granted
        attack: u8,
    },
    /// Discover a spell that costs (1) (2025–2026 expansions M4-W4 —
    /// CATA_484 Winterspring Whelp; the `OneCostSpell` pool — §26).
    DiscoverOneCostSpell,
    /// Discover any spell (2025–2026 expansions M4-W4 — CATA_614
    /// Shadowed Informant; the `Spell` pool).
    DiscoverAnySpell,
    /// Summon two random minions that cost (1) (2025–2026 expansions
    /// M4-W4 — CATA_499 Disposable Acolytes' play half; the discard half
    /// lives at the discard chokepoint).
    SummonTwoRandomOneCostMinions,
    /// Reopen a friendly location if a Fel spell was just played
    /// (2025–2026 expansions M4-W4 — CATA_527 Nespirah's trigger; the
    /// location's deathrattle fires at destruction — §26).
    ReopenLocationIfFelSpell,
    /// Summon random minions of the given cost (2025–2026 expansions
    /// M4-W4 — CATA_723 Drakeadon Mongrel's deathrattle; the `MinionCost`
    /// pool).
    SummonRandomMinionsOfCost {
        /// The minions' cost
        cost: u8,
        /// How many to summon
        count: u8,
    },
    /// Get a random non-Colossal Naga; it costs (1) (2025–2026 expansions
    /// M4-W4 — CATA_527t2 Nespirah, Unshackled's "after you cast a Fel
    /// spell" trigger effect; the Naga pool samples the active window
    /// minus the Colossal registry).
    AddRandomNagaCost1,
    /// M5-W1 — JAIL_384 Chainbreaker Hogger's Start of Game: duplicate
    /// all OTHER Legendary cards in the starting deck (a copy of each —
    /// the 1-copy-per-legendary invariant is overridden by Hogger's
    /// presence in the starting deck). Resolved by the StartOfGame phase
    /// in `GameBuilder::build`.
    HoggerStartOfGame,
    /// M5-W1 — JAIL_430 Azalina Soulsever's Start of Game: starting
    /// Health 40, deck trimmed to 20 cards plus 20 random copies from
    /// the enemy's starting deck. Resolved by the StartOfGame phase.
    AzalinaStartOfGame,
    /// M5-W1 — JAIL_509 Godfrey the Betrayer's Start of Game: arms the
    /// player's F-A11 override — overdrawn cards park in the SetAside
    /// zone and return to the hand (costing (1) less) when there is
    /// space, instead of burning.
    GodfreyStartOfGame,
    /// M5-W1 — JAIL_860 Chef Neth'rek's Start of Game: when the starting
    /// deck only holds ≤3-cost cards, arms the "Mana to 10 after five
    /// turns" flag (the timer runs at the owner's turn starts).
    NethrekStartOfGame,
    /// M5-W1 — JAIL_800 Mug'Zee's Start of Game: deck-composition
    /// hero-power override — no other minions in the deck gets Mug's
    /// Magic ("your first minion each turn costs (2) less"), no spells
    /// gets Zee's Might ("every fifth minion you play triggers its
    /// Battlecry twice"); both are passive hero powers.
    MugzeeStartOfGame,
    /// M5-W1 — JAIL_397 Commander Beatrix's Start of Game (simplified,
    /// §27): the deck-building-time 2-Cost pick is a random 2-Cost
    /// minion sampled at setup; ten copies join the starting deck.
    BeatrixStartOfGame,
    /// M5-W1 — JAIL_504 Aya, Lotus Kingpin's battlecry branch: pick an
    /// upgraded counterfeit — the chosen coin replaces The Coin this
    /// game (`Player::coin_replacement`, read by the add-to-hand choke
    /// point), any Coin already in hand transforms into it, and two
    /// copies join the hand. The three branches ride the choose-one
    /// machinery (battlecry slot = Jade Coin, choose-one slot = Grimy
    /// Coin, third branch = Kabal Coin).
    AyaUpgradeCoins {
        /// The upgraded counterfeit token's card id
        card_id: &'static str,
    },
    /// M5-W1 — JAIL_448 Karov the Broken's deathrattle: get three 1/1
    /// copies of random Legendary minions; they cost (1). The pool is
    /// the `is_legendary` filter over the minion cards (the D2 pool
    /// convention).
    KarovThreeLegendaryCopies,
    /// M5-W1 — JAIL_446 Blood Doctor Thal'ena's battlecry (simplified,
    /// §27): the second hero power is a swap — the hero power becomes
    /// Vampyr's Kiss (3 mana, "Give a minion +3 Attack"), which costs 3
    /// Corpses instead of Mana (the HeroPowerActivated cost site).
    ThalenaSecondHeroPower,
    /// M5-W1 — JAIL_122 Jailhouse Manastorm's battlecry: while this
    /// minion is on the board, each spell the owner casts this game
    /// summons a random minion of the same Cost. (Simplified to
    /// while-alive, §27: the game-long flag is cleared when the minion
    /// dies.)
    ManastormSetAfterSpell,
    /// M5-W1 — JAIL_407 Vanessa the Ringleader's trigger: after the
    /// owner plays a card, get a random Battlecry minion; it costs (2)
    /// less (the `AddRandomBattlecryMinion` pool — ALL_CARDS minions
    /// with a battlecry).
    VanessaGetBattlecryMinionCost2Less,
    /// M5-W1 — JAIL_721 Tras'tath, Soul Parasite's trigger: after the
    /// owner summons a Demon, gain its stats (a permanent enchantment
    /// on the Tras'tath; the just-summoned Demon is the event subject).
    TrastathGainSummonedDemonStats,
    /// M5-W1 — JAIL_906 Moragg's deathrattle: summon a random Demon
    /// from the owner's deck and grant it "Deathrattle: Summon Moragg"
    /// (the deck entity is consumed, the §21 deck-summon convention).
    MoraggDeathrattle,
    /// M5-W1 — the deathrattle granted by Moragg's chain: summon Moragg
    /// (JAIL_906).
    SummonMoragg,
    /// M5-W1 — a passive hero power's effect marker (JAIL_800hp1 Mug's
    /// Magic / JAIL_800hp2 Zee's Might): the power's actual behaviour
    /// lives in the cost pipeline and the CardPlayed counter — the
    /// effect resolves nothing.
    PassiveHeroPower,
    /// M5-W1 — JAIL_504t Jade Coin: gain 1 Mana Crystal this turn only
    /// and summon a 1/1 Jade Golem (the Golem's scaling is simplified
    /// to a fixed 1/1, §27).
    JadeCoin,
    /// M5-W1 — JAIL_504t2 Grimy Coin: gain 1 Mana Crystal this turn
    /// only and deal 2 damage to a random enemy minion.
    GrimyCoin,
    /// M5-W1 — JAIL_504t3 Kabal Coin (simplified, §27): gain 1 Mana
    /// Crystal this turn only and get a random 1-Cost spell (the
    /// Kazakus potion pool is generated multi-choice spells with no
    /// static defs — approximated by the 1-cost spell pool).
    KabalCoin,
    // ----------------------------------------------------------------
    // M5-W2 — the closing Escape from Violet Hold wave (exp_jail_w2.rs,
    // fidelity-debt §28)
    // ----------------------------------------------------------------
    /// M5-W2 — JAIL_500 Slice and Dice (legendary): replay all other
    /// cards played this turn, then end the turn. The replay runs the
    /// rewind machinery over the entries since the turn-start mark
    /// (`Player::rewind_turn_start_len`); "targeting enemies if
    /// possible" is registered as a simplification (§28).
    SliceAndDice,
    /// M5-W2 — JAIL_458 Tiny Pal (legendary weapon): `ammo == 0` is
    /// the battlecry — surface a `ChoiceKind::TinyPalAmmo` choice;
    /// `ammo` 1-4 are the weapon triggers (JAIL_458t1-t4): resolve the
    /// ammo effect and re-surface the choice.
    TinyPal {
        /// The ammunition slot: 0 = battlecry choice, 1-4 = ammo effects
        ammo: u8,
    },
    /// M5-W2 — JAIL_875 Staff of Trickery (legendary weapon trigger):
    /// after the hero attacks, discover a Druid card; the picked card
    /// costs (the hero's Attack) less.
    StaffOfTrickery,
    /// M5-W2 — JAIL_882 R4T-C4TCH3R's battlecry: copy every spell in
    /// the owner's deck (fresh copies at random deck positions).
    R4TCatcher,
    /// M5-W2 — JAIL_719 Irida Sinseeker's battlecry: send the deck to
    /// the Void — the deck card ids ride `Player::void_cards` and the
    /// deck entities are destroyed.
    IridaSinseeker,
    /// M5-W2 — JAIL_719 Irida Sinseeker's turn-start trigger: get two
    /// cards from the Void.
    IridaGetVoid,
    /// M5-W2 — JAIL_831 King of the Underbelly's battlecry: discover
    /// one of three random Beasts; it costs (3) less.
    KingOfTheUnderbelly,
    /// M5-W2 — JAIL_851 Inspector Murloc Holmes's battlecry: look at
    /// three random enemy hand cards and secretly pick one — the
    /// `ChoiceKind::MurlocHolmes` choice; if the enemy plays the pick
    /// on their next turn, the owner gets 3 Coins.
    MurlocHolmes,
    /// M5-W2 — JAIL_852 Togwaggle, Smuggler King's battlecry: shuffle
    /// both players' hands together — one pile, shuffled, the first
    /// half to the caster and the rest to the opponent (§28 pins the
    /// split convention).
    TogwaggleShuffleHands,
    /// M5-W2 — JAIL_850 Warden Maiev's trigger: after the owner plays
    /// a minion, give it +3/+3 and make it Dormant for 1 turn.
    MaievBuffDormant,
    /// M5-W2 — JAIL_887t2 Zuramat the Obliterator's turn-end trigger:
    /// play a card discarded by Zuramat's Prison (JAIL_887) — the
    /// discarded ids ride `Player::zuramat_discarded`; a minion is
    /// summoned, a weapon equipped, or a spell effect resolved.
    ZuramatPlaysDiscarded,
    /// M5-W2 — JAIL_125 Cold Snap: Freeze an enemy and get a random
    /// Frost spell.
    ColdSnap,
    /// M5-W2 — JAIL_432 Mind Sweeper's battlecry: if the owner played
    /// a copy of an opponent's card while holding this, deal 2 damage
    /// to all enemy minions.
    MindSweeper,
    /// M5-W2 — JAIL_434 Enthralled Shade's deathrattle: reduce the
    /// Cost of hand cards copied from the opponent by (1).
    EnthralledShade,
    /// M5-W2 — JAIL_321 Tricksy Improviser's battlecry: if the owner
    /// cast a spell this turn, cast a random Mage Secret.
    TricksyImproviser,
    /// M5-W2 — JAIL_326 Judgment: choose a friendly minion (the pick
    /// is random, §28) and set all minions' stats equal to that
    /// minion's.
    Judgment,
    /// M5-W2 — JAIL_327 Reinforcement Aura: at each of the next 3
    /// owner turn ends, summon a random minion costing (2) or less
    /// from the deck (`Player::reinforcement_aura_ticks`).
    ReinforcementAura,
    /// M5-W2 — JAIL_328 Scarlet Bruiser's deathrattle: if the deck
    /// has no Neutral cards, get a random Paladin card; it costs (2)
    /// less.
    ScarletBruiser,
    /// M5-W2 — JAIL_376 Ball and Chain's deathrattle: give the
    /// owner's damaged minions +1/+2.
    BallAndChain,
    /// M5-W2 — JAIL_377 Holy Bola!: draw a card; if it costs (2) or
    /// less, draw another.
    HolyBola,
    /// M5-W2 — JAIL_379 Spire Security's battlecry: reveal a random
    /// spell in the deck; if it costs (5) or more, deal 5 damage split
    /// among enemy minions.
    SpireSecurity,
    /// M5-W2 — JAIL_380 Smuggled Shovel's deathrattle: draw a spell
    /// that didn't start in the deck (`Player::starting_deck`).
    SmuggledShovel,
    /// M5-W2 — JAIL_386 Scramble for Gear: gain 2 Armor and shuffle
    /// five Found Gear! (JAIL_386t) spells into the deck.
    ScrambleForGear,
    /// M5-W2 — JAIL_387 Release the Beasts: give minions in hand
    /// +1/+1 (the Legendary extra +2/+1 is dropped — the CardDef
    /// carries no rarity, §28).
    ReleaseTheBeasts,
    /// M5-W2 — JAIL_395 Sewer Swimmer's battlecry: trigger a random
    /// friendly minion's Deathrattle.
    SewerSwimmer,
    /// M5-W2 — JAIL_398 IMPFERNAL!'s deathrattle: deal 3 damage to
    /// all other characters (the in-hand/in-deck trigger is dropped,
    /// §28).
    Imfernal,
    /// M5-W2 — JAIL_399 Imp Gang Stooge's deathrattle: put two
    /// Grandmother Imps (JAIL_399t1, 8/8 Taunt Lifesteal) on the
    /// bottom of the deck.
    ImpGangStooge,
    /// M5-W2 — JAIL_442 Disguised Doctor's deathrattle: shuffle four
    /// Blights (JAIL_443t) into the deck.
    DisguisedDoctor,
    /// M5-W2 — JAIL_444 Sawbones's battlecry: destroy the owner's
    /// other minions; draw a card and refresh a Mana Crystal for each
    /// one destroyed.
    Sawbones,
    /// M5-W2 — JAIL_445 Bone Flurry: deal 3 damage split among
    /// enemies; +3 more if a friendly minion died this turn. The
    /// ImmuneToSpellpower keyword exempts it from spell power (the
    /// `apply_spell_power` exemption list).
    BoneFlurry,
    /// M5-W2 — JAIL_441 Drink Blood: deal 3 damage to a minion with
    /// Lifesteal — the owner's hero heals for the damage dealt — and
    /// refresh the Hero Power.
    DrinkBlood,
    /// M5-W2 — JAIL_454 Emergency Surgery: summon four 3/1 Undead
    /// with Lifesteal (JAIL_454t Necronurse) that attack the chosen
    /// enemy minion.
    EmergencySurgery,
    /// M5-W2 — JAIL_455 Disguised Watchman's battlecry: deal 1 damage
    /// to all other friendly minions, twice.
    DisguisedWatchman,
    /// M5-W2 — JAIL_456 P1CK-P0K3T's battlecry: if the deck has 25 or
    /// more cards, draw a card.
    PickPocket,
    /// M5-W2 — JAIL_474 Jade Guardians: get two random 8-Cost minions;
    /// they cost (1) less for each card played for 2 Mana this game
    /// (`Player::cards_played_cost_2`).
    JadeGuardians,
    /// M5-W2 — JAIL_307 Crowd Control: deal 2 damage to all minions;
    /// +2 more if the deck has 25 or more cards.
    CrowdControl,
    /// M5-W2 — JAIL_035 Vigilant Sentry's battlecry: if the deck has
    /// no Neutral cards, summon two Vigilant Sentries.
    VigilantSentry,
    /// M5-W2 — JAIL_101 Violet Punisher's battlecry: choose an enemy
    /// minion (the pick is random, §28) and gain +1/+1 per keyword it
    /// has (the keywords themselves are not stolen, §28).
    VioletPunisher,
    /// M5-W2 — JAIL_123 Breakout Architect's battlecry: get a random
    /// spell costing (5) or more; the next spell the owner casts casts
    /// twice (the Discover is simplified to a random pick, §28).
    BreakoutArchitect,
    /// M5-W2 — JAIL_861 Noxious Bribe: get a random Choose One card
    /// (a plain pick, §28 — the combined-effects discover is
    /// simplified) and give the opponent a plain copy of it.
    NoxiousBribe,
    /// M5-W2 — JAIL_502 Alarm-o-Matic's start-of-turn effect: swap
    /// this minion with a random minion in the opponent's hand.
    AlarmOMatic,
    /// M5-W2 — JAIL_507 Spiteful Chef's battlecry: summon a random
    /// 2-Cost Taunt minion, or a 6-Cost one at 10+ Mana.
    SpitefulChef,
    /// M5-W2 — JAIL_510 Annihilation: destroy all minions, then
    /// summon any Demons among the bottom 3 cards of the deck.
    Annihilation,
    /// M5-W2 — JAIL_511 Spire of Solitude: summon a Demon with stats
    /// equal to the hand size; it attacks a random enemy minion.
    SpireOfSolitude,
    /// M5-W2 — JAIL_515 Shadow Rounds: deal 2 damage to an enemy
    /// minion; if it dies, cast this on another random enemy minion.
    ShadowRounds,
    /// M5-W2 — JAIL_516 Scarlet Recruiter's battlecry: summon two
    /// minions costing (2) or less from the deck and give them Rush.
    ScarletRecruiter,
    /// M5-W2 — JAIL_706 Thief's Tools: get two random 4-Cost spells;
    /// they cost (2) less.
    ThievesTools,
    /// M5-W2 — JAIL_732 Void Soul: summon a random 1-Cost Demon and
    /// improve future Void Souls (the summoned Demon's cost scales
    /// with `Player::void_soul_level`).
    VoidSoul,
    /// M5-W2 — JAIL_735 Code Violet: summon an 8-Cost minion; if the
    /// owner cast 3 other spells this turn, summon another.
    CodeViolet,
    /// M5-W2 — JAIL_801 Molten Gold: deal 4 damage; after 3 spells
    /// cast this turn, summon a Molten Gold Elemental (JAIL_801t)
    /// instead.
    MoltenGold,
    /// M5-W2 — JAIL_803 Frostshatter: Freeze an enemy and draw 2
    /// cards; after 3 spells cast this turn, summon a Frostshatter
    /// Elemental (JAIL_803t) instead (the elemental's battlecry rides
    /// this same variant — the source check selects the branch).
    Frostshatter,
    /// M5-W2 — JAIL_805 Stormfury: deal 2 damage to all enemy minions
    /// with Lifesteal; after 3 spells cast this turn, summon a
    /// Stormfury Elemental (JAIL_805t) instead (the elemental's
    /// battlecry rides this same variant).
    Stormfury,
    /// M5-W2 — JAIL_806 Hexmarshal's battlecry: get a random spell
    /// costing (5) or more; if the deck started with no spells, it
    /// costs (5) less.
    Hexmarshal,
    /// M5-W2 — JAIL_866 Lethal Recipe: draw 2 minions; at 10+ Mana,
    /// give them +3/+3.
    LethalRecipe,
    /// M5-W2 — JAIL_876 Dig for Freedom: give a friendly minion
    /// "Deathrattle: Summon two random 4-Cost minions."
    DigForFreedom,
    /// M5-W2 — JAIL_878 Guard Dog's deathrattle: summon a random
    /// 1-Cost Deathrattle minion.
    GuardDog,
    /// M5-W2 — JAIL_879 Beast Tripwire: summon a random 5-Cost Beast
    /// and shuffle two Tripped Beast Tripwires (JAIL_879t) into the
    /// deck.
    BeastTripwire,
    /// M5-W2 — JAIL_881 Arcane Tripwire: deal 5 damage split among
    /// all enemies and shuffle two Tripped Arcane Tripwires (JAIL_881t)
    /// into the deck. ImmuneToSpellpower exempts it (the
    /// `apply_spell_power` exemption list).
    ArcaneTripwire,
    /// M5-W2 — JAIL_974 Captured Archmage's deathrattle: if 4 other
    /// Captured Archmages died this game, cast Fireball at a random
    /// enemy.
    CapturedArchmage,
    /// M5-W2 — JAIL_986 Frantic Forger's battlecry: get a random
    /// playable spell; it is Temporary.
    FranticForger,
    /// M5-W2 — JAIL_987 Low Security Wing: get a random Shaman
    /// minion, locked in hand until the owner plays another card (the
    /// `LockedUntilCardPlayed` component).
    LowSecurityWing,
    /// M5-W2 — JAIL_997 Demonic Confinement: make a minion go Dormant
    /// for 2 turns; if it is a friendly Demon, give it +3/+3 instead.
    DemonicConfinement,
    /// M5-W2 — JAIL_436 Widow's Bite: hero +1 Attack this turn, gain
    /// 1 Armor, add Widow's Feast (JAIL_436t) to hand.
    WidowsBite,
    /// M5-W2 — JAIL_436t Widow's Feast: hero +2 Attack this turn,
    /// gain 2 Armor, add Widow's Banquet (JAIL_436t2) to hand.
    WidowsFeast,
    /// M5-W2 — JAIL_436t2 Widow's Banquet: hero +4 Attack this turn
    /// and gain 4 Armor.
    WidowsBanquet,
    /// M5-W2 — JAIL_225 Nab: deal 3 damage to a minion; if it dies,
    /// shuffle a copy of it into the deck with its Cost set to (2)
    /// (the fresh copy's cost is set directly, §28).
    Nab,
    /// M5-W2 — JAIL_891 Void Blast: deal 3 damage to a minion; if it
    /// dies, get a Void Soul (JAIL_732).
    VoidBlast,
    /// M5-W2 — JAIL_892 Cosmic Manifestations: deal 2 damage (4 when
    /// Outcast) and shuffle a random Demon Hunter spell into the deck.
    CosmicManifestations,
    /// M5-W2 — JAIL_909 Defias Wannabe's Combo: gain +1/+1 for each
    /// other card played this turn.
    DefiasWannabe,
    /// M5-W2 — JAIL_912 Soothsayer's deathrattle: restore 6 Health to
    /// the owner's hero and summon a random 6-Cost minion.
    Soothsayer,
    /// M5-W2 — JAIL_941 Holy Embrace: restore 4 Health and get a Dark
    /// Embrace (JAIL_941t).
    HolyEmbrace,
    /// M5-W2 — JAIL_205 Rat Burglar's turn-end trigger: steal all
    /// cards that entered the opponent's hand during the owner's turn
    /// (simplified to one random enemy hand card, §28).
    RatBurglar,
    /// M5-W2 — JAIL_206 Dark Bribe: draw 3 cards, then choose one to
    /// give to the opponent (the `ChooseHandCardKind::GiveToOpponent`
    /// hand choice).
    DarkBribe,
    /// M5-W2 — JAIL_303 Ancient Augur's battlecry: look at 3 random
    /// enemy hand cards and secretly choose one — the
    /// `ChoiceKind::PickEnemyHandCard` choice.
    AncientAugurPick,
    /// M5-W2 — JAIL_303 Ancient Augur's deathrattle: discard the
    /// secretly chosen enemy hand card.
    AncientAugurDeathrattle,
    /// M5-W2 — JAIL_459 Arachnathid's battlecry (the Poisonous aura
    /// approximation, §28): give other friendly minions Poisonous; the
    /// FriendlyMinionSummoned trigger grants it to new minions too.
    ArachnathidVenom,
    /// M5-W2 — JAIL_329 Truth Seeker's weapon trigger: after the hero
    /// attacks, give the owner's Paladin minions +2/+2.
    TruthSeeker,
    /// M5-W2 — JAIL_460 Concealing Confection's deathrattle: get a
    /// random weapon.
    ConcealingConfection,
    /// M5-W2 — JAIL_461 Disguised Executioner's battlecry: destroy a
    /// random adjacent minion.
    DisguisedExecutioner,
    /// M5-W2 — JAIL_462 Getaway Hogdriver's battlecry: draw 2 cards;
    /// if both are minions, gain Charge.
    GetawayHogdriver,
    /// M5-W2 — JAIL_030 Escape Artist's trigger: after this attacks
    /// and survives, draw a card and escape the game — the minion is
    /// moved to the set-aside zone (the official return-to-hand
    /// behaviour is approximated, §28).
    EscapeArtist,
    /// M5-W2 — JAIL_451 Blood Clone: discover a 5-Cost minion and
    /// spend 5 Corpses to summon a copy of it (the
    /// `ChoiceKind::BloodClone` choice).
    BloodClone,
    /// M5-W2 — JAIL_802 Gallagio Goon's CardPlayed trigger: when the
    /// played card is a Battlecry minion, give it +1/+1 (the subject).
    GallagioGoon,
    /// M5-W2 — JAIL_880 Black Market Overseer's CardPlayed trigger: when
    /// the played card is a Deathrattle minion, give it Rush (the
    /// subject).
    BlackMarketOverseer,
    /// M5-W2 — JAIL_883 Activated Golem's TurnEnd trigger: grant the
    /// source Reborn (§28: TurnEnd triggers are owner-scoped, so the
    /// "each turn" pin is approximated).
    ActivatedGolem,
    /// M5-W2 — JAIL_734 Hellraiser's battlecry: add a random deck card
    /// to the hand; when the deck is empty, the source gains +4/+4
    /// (§28: the discover is a random deck card).
    Hellraiser,
    /// M5-W2 — JAIL_876 Dig for Freedom's granted Deathrattle: "Summon
    /// two random minions of the given Cost."
    SummonTwoRandomMinionsOfCost {
        /// The minion Cost to summon.
        cost: i32,
    },
    /// MEND W1 — the Druid class-set wave (src/cards/exp_cata_w5.rs,
    /// fidelity-debt §29). MEND_041 Wizened Wildspeaker — "Battlecry: If
    /// you didn't play a minion last turn, refresh 3 Mana Crystals." The
    /// last-turn flag reads the `last_turn_minion_play_ids` snapshot.
    RefreshManaIfNoMinionPlayedLastTurn {
        /// Mana Crystals to refresh.
        amount: i32,
    },
    /// MEND W1 — MEND_042 Lifebloom: "Restore 8 Health to all friendly
    /// characters. Summon two random 8-Cost minions."
    RestoreAllFriendlyAndSummonTwoRandomCostMinions {
        /// Health restored to every friendly character.
        heal: i32,
        /// Cost of the two random minions to summon.
        cost: i32,
    },
    /// MEND W1 — MEND_043 Heartroot Stones: "Draw a card and gain 3
    /// Armor. If you didn't play a minion last turn, do it again."
    DrawAndGainArmorRepeatIfNoMinionPlayedLastTurn {
        /// Cards drawn per iteration.
        draw: i32,
        /// Armor gained per iteration.
        armor: i32,
    },
    /// MEND W1 — MEND_044 Tranquil Clearing: "Give a minion +2 Health
    /// and Taunt. It falls asleep until the end of your next turn." The
    /// sleep rides the Dormant component (one turn — §29).
    BuffHealthTauntAndDormant {
        /// Health bonus.
        health: i32,
    },
    /// MEND W1 — MEND_045 Seeding Dragon deathrattle: "Get a random
    /// Dragon. It costs (2) less."
    AddRandomDragonCostReduced {
        /// Cost reduction on the added Dragon.
        reduction: i32,
    },
    /// MEND W1 — MEND_046 Bashana Runetotem: "Battlecry: Get three 2/2
    /// Treants. Carve 12 Mana worth of Nature spells into them." (§29:
    /// the carve is simplified to up to three random Nature spells —
    /// each costing no more than the remaining 12-Mana budget — added
    /// to hand.)
    GetThreeTreantsAndCarveNatureSpells,
    /// MEND W1 — MEND_100t Blooming Bulb: "Cast three random spells
    /// that cost (1). (Upgrades each turn!)" — the hand-turn counter
    /// (CATA_498 convention) raises the cost of the cast spells.
    CastRandomSpellsScaledByHandTurns {
        /// Base cost of the cast spells (before the upgrade ticks).
        base: i32,
        /// Number of random spells to cast.
        count: i32,
    },
    /// Replace this game's future Animal Companions with random Beasts
    /// that cost (1) more (MEND W2 — Tame Pet). The bump is added to the
    /// existing `companion_replacement` (repeated casts stack, official
    /// upgrade behaviour; §30 simplification).
    SetCompanionReplacement {
        /// Cost bump over the base Beast cost of 3.
        bump: i32,
    },
    /// Replace this game's future Animal Companions with random Beasts
    /// that cost (1) more, then draw a card (MEND W2 — Tame Pet).
    SetCompanionReplacementAndDraw {
        /// Cost bump over the base Beast cost of 3.
        bump: i32,
        /// Number of cards to draw.
        draw: i32,
    },
    /// Your cards that summon Animal Companions summon 1 more this game
    /// (MEND W2 — Talya Earthstrider). Increments `companion_bonus`;
    /// each summoned companion is independently subject to
    /// `companion_replacement`.
    SetCompanionBonus {
        /// Extra companions per Animal Companion summon.
        amount: i32,
    },
    /// Replace this game's future Animal Companions with random Beasts
    /// that cost (2) more, then summon a random one of them (MEND W2 —
    /// Roam Free; the Choose One resolution passes the chosen cost tier).
    ReplaceCompanionsAndSummonRandomBeast {
        /// Cost bump over the base Beast cost of 3 (2 for Roam Free).
        bump: i32,
        /// Cost tier of the chosen Beast to summon (5/6/7 for Roam Free's
        /// three options — §30 simplification: a random Beast of exactly
        /// this cost rather than one of the fixed trio).
        cost: i32,
    },
    /// Deal damage split among all enemies (random enemy per point);
    /// if any of them dies, deal that many more (MEND W2 — Wasteland
    /// Vanguard). The chain fires at most once (official ruling; §30).
    SplitDamageAmongAllEnemiesChainOnDeath {
        /// Total damage to split among all enemies (3 for Wasteland
        /// Vanguard), one point at a time against a random surviving
        /// enemy; repeated once if any died.
        amount: i32,
    },
    /// Give a friendly Beast +N/+N and a random Beast in your hand +N/+N
    /// (MEND W2 — Nurturing Nature). Buffs apply directly on the board /
    /// hand entities.
    BuffFriendlyBeastAndRandomHandBeast {
        /// Attack buff.
        attack: i32,
        /// Health buff.
        health: i32,
    },
    // ----------------------------------------------------------------
    // MEND W3 — the Cataclysm Mage class-set wave
    // (src/cards/exp_cata_w7.rs, fidelity-debt §31). The "Leyline"
    // package: the three Leyline cards (MEND_500 / MEND_502 / MEND_504)
    // carry the {0}/{1} scalar pairs the support cards upgrade — the
    // resolve arms read the owner's three Player flags
    // (`leyline_discount` / `leyline_extra_trigger` / `leyline_effect_bonus`)
    // at resolution time.
    // ----------------------------------------------------------------
    /// Deal {amount} damage to a random enemy minion, {times} times.
    /// Excess damage hits the enemy hero (MEND_500 Bursting Leyline —
    /// the {0}/{1} bases; the ImmuneToSpellpower exemption in
    /// `apply_spell_power` keeps the amount unbuffed). The resolution
    /// adds the owner's `leyline_effect_bonus` to {amount} and
    /// `leyline_extra_trigger` to {times}.
    DealDamageToRandomEnemyMinionExcessToHero {
        /// Damage per hit (the {0} base — 3).
        amount: i32,
        /// Number of hits (the {1} base — 1).
        times: i32,
    },
    /// Your Leylines cost (1) less this game (MEND_501 Ley Walker
    /// battlecry). Increments the owner's `leyline_discount`; the
    /// play-cost pipeline subtracts it for every `cards::leyline` id.
    SetLeylineDiscount {
        /// Discount per Leyline play.
        amount: i32,
    },
    /// Get a random Leyline (MEND_501 Ley Walker deathrattle — the
    /// `cards::leyline::LEYLINE_CARD_IDS` pool).
    AddRandomLeylineToHand,
    /// Summon a random {cost}-Cost minion, {times} times (MEND_502
    /// Crystallized Leyline — the {0}/{1} bases). The resolution adds
    /// the owner's `leyline_effect_bonus` to {cost} and
    /// `leyline_extra_trigger` to {times}.
    SummonRandomCostMinionTimes {
        /// Cost of the summoned minion (the {0} base — 5).
        cost: i32,
        /// Number of summons (the {1} base — 1).
        times: i32,
    },
    /// Your Leylines trigger an additional time this game (MEND_503
    /// Surge Needle battlecry). Increments the owner's
    /// `leyline_extra_trigger`.
    SetLeylineExtraTrigger {
        /// Extra triggers per Leyline play.
        amount: i32,
    },
    /// Draw {count} cards. They cost ({reduction}) less (MEND_504
    /// Leyline Nexus — the {0}/{1} bases). The resolution adds the
    /// owner's `leyline_effect_bonus` to {reduction} and
    /// `leyline_extra_trigger` to {count}; each drawn card's cost is
    /// reduced via the `draw_card_with_reduction` enchantment.
    DrawCardsCostsLess {
        /// Cost reduction on each drawn card (the {0} base — 1).
        reduction: i32,
        /// Number of cards drawn (the {1} base — 1).
        count: i32,
    },
    /// Get all 3 Leylines, then apply the chosen upgrade (MEND_505 The
    /// Arcanomicon — the three Choose One branches share this variant;
    /// `upgrade` picks the axis). The cards are added to the hand and
    /// the matching Player flag increments by 1.
    GetAllLeylinesAndUpgrade {
        /// The chosen upgrade axis.
        upgrade: LeylineUpgrade,
    },
    /// Increase the effects of your Leylines by 1 this game (MEND_506
    /// Mystic Runesaber battlecry). Increments the owner's
    /// `leyline_effect_bonus` — the {0} scalars: +1 damage / +1
    /// summoned-minion cost / +1 cost reduction.
    SetLeylineEffectBonus {
        /// Effect-magnitude bonus for the Leyline cards.
        amount: i32,
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
    SetHealth {
        health: i32,
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
    GainStatsAndGrantWindfury {
        attack: i32,
        health: i32,
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
    DiscardHand,
    NextSpellDiscount {
        amount: i32,
    },
    GrantAdjacentStatsAndDivineShield {
        attack: i32,
        health: i32,
    },
    DestroyAllOtherMinionsAndDiscardHand,
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
        target: EffectTarget,
    },
    PreventFatalDamageAndImmune,
    TakeControlUntilEndOfTurn,
    TakeControl,
    TakeControlAttackLE {
        max_attack: i32,
    },
    Corrupt,
    MinHealthUntilEndOfTurn,
    TransformToRandom {
        card_a: String,
        card_b: String,
    },
    AddRandomCardToHand {
        pool: RandomPool,
    },
    DiscoverDeckTop3,
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
    DrawCardByRace {
        count: u32,
        race: crate::core::component::Race,
    },
    Demonfire {
        damage: i32,
        attack_bonus: i32,
        health_bonus: i32,
    },
    GainStatsAndTaunt {
        attack: i32,
        health: i32,
        target: EffectTarget,
    },
    DestroyAndGainStats {
        attack: i32,
        health: i32,
        target: EffectTarget,
    },
    DestroyRandomEnemySecret,
    DestroyAllEnemySecretsAndGainStats {
        attack: i32,
        health: i32,
    },
    DestroyAllEnemySecretsAndDraw {
        count: u32,
    },
    AttachAttackDraw {
        count: u32,
    },
    GainStatsPerHandCard {
        attack: i32,
        health_per_card: i32,
    },
    GainStatsPerFriendlyMinion {
        attack: i32,
        health_per_minion: i32,
    },
    DealDamageRandomly {
        amount: i32,
        count: i32,
        target: EffectTarget,
    },
    MortalStrike {
        damage: i32,
        boosted: i32,
        threshold: i32,
    },
    DrawPerDamagedFriendlyCharacter,
    GainStatsIfOwnSecret {
        attack: i32,
        health: i32,
    },
    AbsorbDivineShields {
        attack_per_shield: i32,
        health_per_shield: i32,
    },
    RemoveWeaponDurability {
        amount: i32,
    },
    GainAttackEqualToWeapon,
    EnemySpellsCostZero,
    GiveOpponentManaCrystal {
        count: i32,
    },
    SetPlayedMinionHealth {
        health: i32,
    },
    SilenceAllEnemyMinionsAndDraw {
        count: u32,
    },
    SwapAttackAndHealth {
        target: EffectTarget,
    },
    FreezeAdjacent,
    GrantAdjacentTaunt,
    GrantAdjacentSpellDamage {
        amount: i32,
    },
    FullHealAndTaunt {
        target: EffectTarget,
    },
    ChanceDraw {
        percent: u32,
    },
    GainStatsThisTurn {
        attack: i32,
        health: i32,
        target: EffectTarget,
    },
    GrantDivineShieldAllFriendly,
    GrantDivineShield {
        target: EffectTarget,
    },
    YseraAwakens {
        damage: i32,
    },
    GainStatsAndTauntAllFriendly {
        attack: i32,
        health: i32,
    },
    DrawAndDamageByCost,
    RestoreDamagedFriendly {
        amount: i32,
    },
    SwapWithHandMinion,
    ResurrectDiedMinion,
    CopyRandomEnemyHandCard {
        count: u32,
    },
    CopyRandomEnemyDeckCards {
        count: u32,
    },
    SummonRandomEnemyDeckMinion {
        fallback_card_id: String,
    },
    CopyCastSpellToOtherPlayerHand,
    FillHandWithMinion {
        card_id: String,
    },
    ForceEnemyMinionsAttackThis,
    SpendCorpsesSummonCopy {
        cost: u32,
    },
    DrawCardOutcast {
        normal: u32,
        outcast: u32,
    },
    OutcastDamage {
        amount: i32,
        outcast_amount: i32,
        target: EffectTarget,
    },
    RestoreRandomFriendly {
        amount: i32,
    },
    DestroyEnemyLocation,
    DamagePlayedMinionAndExcess {
        amount: i32,
    },
    DamageAndDrawIfHandEmpty {
        damage: i32,
        target: EffectTarget,
    },
    AoeDamageAndHealFriendly {
        damage: i32,
        heal: i32,
    },
    DamageAndDrawIfSurvives {
        damage: i32,
        target: EffectTarget,
    },
    GainArmorAndDraw {
        armor: i32,
        draw: u32,
    },
    DamageAndDrawIfKilled {
        damage: i32,
        target: EffectTarget,
    },
    DamageAndGainArmor {
        damage: i32,
        armor: i32,
        target: EffectTarget,
    },
    GainPoisonousToFriendlyUndead,
    TransformToMinion {
        card_id: String,
    },
    GrantDeathrattleToTarget {
        card_id: String,
    },
    DestroyAllMinionsAttackGE {
        attack: i32,
    },
    AoeDamageAndDraw {
        damage: i32,
        draw: u32,
    },
    GainStatsTauntAndDeathrattle {
        attack: i32,
        health: i32,
        card_id: String,
    },
    SummonRandomFishFromDeck,
    AddRandomSpellToOpponentDeckTop,
    SummonStatueTrio,
    CopyEnemyDeckCardOnSelfAttack,
    DamageAndSummon {
        damage: i32,
        target: EffectTarget,
        card_id: String,
    },
    RehgarBolt,
    DamageTwoDrawIfKilled {
        damage: i32,
    },
    FreezeAndDiscoverSpell,
    HenchThugBuff,
    SummonRecruitsAndEquipWeapon,
    BuffAndSummonRandomCost2,
    DamageAndSummonCopyIfKilled {
        damage: i32,
    },
    KeymasterCopy,
    FordragonBuff,
    DamageAndSummonVoidwalkers {
        damage: i32,
        target: EffectTarget,
    },
    DamageAndAddToHand {
        damage: i32,
        card_id: String,
    },
    AddRandomOtherClassSpells {
        count: u32,
        min_cost: i32,
    },
    SummonFelbatOnDraw,
    SpendCorpsesSummonRandomMinion {
        max: u32,
    },
    GainStatsAndDraw {
        attack: i32,
        health: i32,
        target: EffectTarget,
        draw: u32,
    },
    DamageUndamaged {
        damage: i32,
    },
    DamageMinionAndSelfHero {
        damage: i32,
    },
    GainHeroAttackAndDraw {
        attack: i32,
    },
    GainArmorAndSummonDeckMinion {
        armor: i32,
        max_cost: i32,
    },
    GainArmorAndDrawOnHeroAttack {
        armor: i32,
    },
    SummonAllCompanions,
    DamageFreezeAllAndSummon {
        damage: i32,
        card_id: String,
    },
    DestroyHighestAttackEnemy,
    SummonZombiesWithCorpseReborn {
        corpses: u32,
    },
    TransformSelfToCastSpell,
    BuffHandMinionsWithCorpses {
        corpses: u32,
    },
    RestoreHealthAndDraw {
        amount: i32,
        target: EffectTarget,
    },
    DrawIfUnspentMana,
    GainArmorAndSummonRandomCost {
        armor: i32,
        cost: i32,
    },
    GainStatsIfHealedThisTurn {
        attack: i32,
        health: i32,
    },
    BattleToTheDeath,
    NextDemonDiscount {
        amount: i32,
    },
    BuffHandMinions {
        attack: i32,
        health: i32,
    },
    SummonRandomEnemyHandMinion,
    DrawForBoth,
    NextComboDiscount {
        amount: i32,
    },
    AddRandomMageSpells {
        count: u32,
    },
    DamageEnemyHeroAndHealSelf {
        amount: i32,
    },
    LoseHealthPerOpponentHandCard,
    GrantRandomFriendlyDivineShieldTaunt,
    RemoveTopEnemyDeckCard,
    DiscoverSpellAndHealCost,
    DrawBeastDragonMurloc,
    AddRandomOtherClassCard,
    AddRandomShamanSpell,
    DamageSelfHero {
        damage: i32,
    },
    SummonTwoCopiesOfSelf,
    SpendCorpsesDamageRandom {
        max: u32,
        damage: i32,
    },
    SpendCorpsesSummonFootmen {
        max: u32,
    },
    OngoingEndTurnDamage {
        damage: i32,
    },
    DamageAllOtherMinions {
        damage: i32,
    },
    BuffTauntHandMinions {
        attack: i32,
        health: i32,
    },
    SummonRandomMinionCostEqHandSize,
    ResurrectHighestCostFallen,
    GrantDeathrattleSummonOwnCost,
    AddRandomPirateToHand,
    NextEnemyHeroPowerCostMore {
        amount: i32,
    },
    SummonRandomMinionOfCost {
        cost: i32,
    },
    SummonRandomDemonFromHandOrDeck,
    NextEnemySpellsCostMore {
        amount: i32,
    },
    BuffWeaponDurabilityIfBeast {
        amount: i32,
    },
    ReturnLastTurnSpells,
    DestroyMinionAndSelfDamage,
    DamageSelfMinion {
        damage: i32,
    },
    AddRandomOneCostCard,
    BuffThreeDifferentRaces {
        attack: i32,
        health: i32,
    },
    AddFiveRandomCards,
    DiscardTwoRandomCards,
    DamageAllMinionsIfHoldingDragon {
        damage: i32,
    },
    DamageAllEnemiesByAttack,
    ReturnRandomFriendlyAndReduceCost {
        amount: i32,
    },
    GrantAttackToRandomFriendly,
    SummonRandomLegendaryMinion,
    ResurrectWeaponKilled,
    DestroyRandomEnemyMinion,
    SummonOasisWaterElemental,
    SummonRandomCostAndFreeze {
        cost: i32,
    },
    DamageAndAddRandomSpell {
        damage: i32,
        target: EffectTarget,
    },
    FreezeAndSummonElementals,
    AddRandomTauntBuffed,
    AddRandomBattlecryMinion,
    DamageAndFreeze {
        damage: i32,
        target: EffectTarget,
    },
    DamageAllEnemyMinionsAndFreeze {
        damage: i32,
    },
    AddRandomOutcastCardNextCheaper,
    ImbueHeroPower,
    ImbuedHeroPower {
        class: ImbueClass,
    },
    UseHeroPower,
    DrawBeastAndImbue,
    RestoreAndDrawAndImbue {
        amount: i32,
    },
    SummonRandomTwoCostTauntAndImbue,
    ImbueAndReduceHandCost,
    ImbueAndTriggerHeroPower,
    ImbueAndGetWisp,
    ImbueAndDebuffEnemies {
        attack_reduction: i32,
    },
    DealDamageIfImbuedTwice {
        damage: i32,
    },
    DiscoverWildGodIfImbued4,
    ImbueEveryThirdSpell,
    SummonRandomDragonOfCost {
        cost: i32,
    },
    ApplyDarkGift {
        gift: DarkGiftKind,
    },
    DiscoverWithDarkGift {
        pool: RandomPool,
    },
    DiscoverDragonWithDarkGift,
    DiscoverUndeadWithCorpseGift {
        corpses: u32,
    },
    DiscoverEnemyDeckMinionCopy {
        with_gift: bool,
    },
    DiscoverDeckMinionWithDarkGift,
    ReduceHandMinionGiftCost,
    GainStatsAndGrantDivineShield {
        attack: i32,
        health: i32,
        target: EffectTarget,
    },
    GainStatsAndGrantLifesteal {
        attack: i32,
        health: i32,
        target: EffectTarget,
    },
    GrantPoisonousThisTurn,
    GrantHeroLifestealThisTurn,
    GrantWeaponDeathrattleAllEnemies {
        damage: i32,
    },
    DrawCardByType {
        count: u32,
        card_type: CardType,
    },
    SpendCorpsesDamageMinion {
        cost: u32,
        damage: i32,
    },
    DamageAllMinions {
        damage: i32,
    },
    AddRandomDruidSpell,
    AddRandomOtherClassChooseOneCard,
    AttackTwoRandomEnemyMinionsIfCostLE {
        cost: u8,
    },
    GainArmorSummonCostTaunt {
        armor: i32,
        cost: u8,
    },
    AddRandomCostMinionWithDarkGift {
        cost: u8,
    },
    BuffTopDeckMinions {
        attack: i32,
        health: i32,
        count: u8,
    },
    ShuffleAllMinionsIntoDecks,
    DrawDeckSpellAndAddRandomSpell,
    SetStatsByFriendlyTarget {
        enemy_attack: i32,
        enemy_health: i32,
        friendly_attack: i32,
        friendly_health: i32,
    },
    GainAttackEqualSpellCost,
    DamageLowestHealthEnemyTwice {
        amount: i32,
    },
    DrawAndGainStats {
        attack: i32,
        health: i32,
    },
    ShuffleCardIntoDeck {
        card_id: String,
        count: u8,
    },
    AmphibianSpiritBuff {
        attack: i32,
        health: i32,
    },
    DamageAndSummonWolfIfKilled {
        damage: i32,
    },
    AddRandomSpellCostsLess {
        reduction: u8,
    },
    SummonTreantCopyingSpell,
    SummonEggHatchingDragon,
    ResurrectRandomFallenDragon,
    EquipSwordIfHoldingDragon,
    DamageAllOtherFriendlyMinions {
        damage: i32,
    },
    DamageMinionWithMoonLifesteal {
        amount: i32,
    },
    SummonTwoRandomCostMinions {
        base_cost: u8,
        upgraded_cost: u8,
    },
    DamageIfHoldingSpell5Plus {
        amount: i32,
    },
    SummonCopyIfAttackGE {
        attack: i32,
    },
    RestoreHealthAndPendingSelfDamage {
        heal: i32,
        damage: i32,
        turns: u8,
    },
    DestroyCrystalGainCrystalsLater {
        gain: i32,
        turns: u8,
    },
    DrawMinionCostGE {
        cost: u8,
    },
    GainDeathrattleOfDiedThisTurn,
    AddRandomDeckMinionToHand,
    EatDeckMinionGainStats,
    DebuffRandomHandMinionBoth {
        attack_reduction: i32,
    },
    SpendAllManaCastRandomSpell,
    CopyLowestCostEnemyHandCard,
    OpponentDrawsTwoAndCopies,
    ReturnFriendlyMinionSummonSpider,
    ShuffleMatchingEnemyHandCardIntoDeck,
    DestroyFriendlyMinionGainArmor {
        armor: i32,
    },
    DrawSpellCostGE {
        cost: u8,
    },
    DrawDragonsReduced {
        count: u8,
        reduction: u8,
    },
    SummonCopyOfSelf,
    DestroyFriendlyWispDraw {
        count: u8,
    },
    DrawAndSummonLeeches {
        draw: u8,
    },
    DrawAndSummonDreadseed {
        draw: u8,
    },
    NextHeroPowerCostsZero,
    SetMurlocSummonBuff,
    SetDealExact2Bonus,
    RestoreHealthAndGetDruidSpells {
        amount: i32,
        count: u8,
    },
    GainManaCrystalBoth {
        count: i32,
    },
    TransformNeutralDeckToDruid,
    AddMoonfireAndStarfireWithSpellDamage,
    BuffAnotherRandomFriendlyDragon {
        attack: i32,
        health: i32,
    },
    ReduceRightmostHandCardCost {
        reduction: u8,
    },
    ResurrectDeathrattleMinionCostLE {
        cost: u8,
    },
    ResurrectDeathrattleMinionCostGE {
        cost: u8,
    },
    GainArmorPerWisp {
        base: i32,
    },
    DamageMinionScaledByFallen {
        base: i32,
    },
    GrantHeroDivineShield,
    RestoreBothHeroes {
        amount: i32,
    },
    AddSelfToDeckBottomCost {
        cost: u8,
    },
    SummonCopyOfRandomFriendlyDragon,
    GainHealthIfHeroPowerUsed {
        amount: i32,
    },
    AttackRandomEnemyMinionExcess,
    SplashHeroAttackToRandomEnemy,
    GainDeadMinionAttack,
    DrawIfMinionPlayedBefore,
    GrantRandomBonusEffect,
    YseraEmeraldAspect,
    ResurrectAllDifferentFriendlyCostGE {
        cost: u8,
    },
    CastHighestCostSpellFromHand,
    IncrementOmenAttack,
    OmenDeathrattle,
    SplitDamageAmongAllEnemiesIfFallen {
        amount: u8,
        threshold: u8,
    },
    NextSpellsCastTwice {
        count: u8,
    },
    SummonRandomDragonPerSelfDeath,
    GainArmorAndSelfAttack {
        armor: i32,
        attack: i32,
    },
    NextCardCostsZero,
    TransformHandMinionsToRandomDemons,
    DiscoverSpellKeepOrTop,
    DiscardRandomEnemyHandCard,
    FillHandWithEnemyDeckCopies {
        reduction: u8,
    },
    SummonBeetles {
        count: u8,
    },
    UrsocBattlecry,
    UrsocDeathrattle,
    GainStatsAllOtherFriendlyMinions {
        attack: i32,
        health: i32,
    },
    SummonRandomAnimalCompanion,
    AddAllDreamCards,
    CardsCostOneThisGame,
    GainStatsIfHeroPowerUsed {
        attack: i32,
        health: i32,
    },
    GiveMinionStatsRushIfHeroPowerUsed {
        attack: i32,
        health: i32,
    },
    DrawIfImbuedTwice {
        count: u32,
    },
    DealDamageToAllEnemyMinions {
        damage: i32,
    },
    DiscoverWithDarkGiftCostReduction {
        reduction: u8,
    },
    SummonBroodlingsIfHoldingGift,
    FelfireBlazeTrigger {
        damage: i32,
    },
    BuffFriendlyMinionsDiscardBonus {
        attack: i32,
        health: i32,
        bonus_attack: i32,
        bonus_health: i32,
    },
    AmirdrassilActivate,
    InfernoHeraldTrigger {
        reduction: u8,
    },
    BuffMinionReturnIfSpellsCast {
        attack: i32,
        health: i32,
        threshold: u32,
    },
    GainWeaponAttackIfHoldingGift {
        amount: i32,
    },
    DamageRandomEnemyMinionHoldingCostGE {
        base: i32,
        upgraded: i32,
        threshold: i32,
    },
    DiscoverComboBattlecryStealthWithDarkGift,
    DiscoverDemonWithDarkGiftCopy,
    DiscoverCostCardGainTempMana {
        cost: u8,
        mana: u8,
    },
    DamageAndDiscoverWarriorWithGift {
        damage: i32,
    },
    ReduceHandCostIfAllDistinct {
        reduction: u8,
    },
    DrawMinionSummonDivineShieldCopy,
    VolcorossBattlecry,
    DiscoverSpellReduceHandSpells {
        reduction: u8,
    },
    MagmaHoundSplash,
    DamageMinionOwnerDraws {
        damage: i32,
    },
    DeathrattleDamageAllEnemiesTurnScaled {
        base: i32,
        boosted: i32,
    },
    DealDamageSplitAmongAllEnemies {
        amount: i32,
    },
    CopyLowestCostBeastInHand,
    GainDivineShieldLifestealIfHoldingSpellGE {
        cost: i32,
    },
    GainHeroAttackArmorIfHoldingGift {
        attack: i32,
        armor: i32,
    },
    DamageAndDiscardSpellMore {
        base: i32,
        bonus: i32,
    },
    BuffAllHandMinions {
        attack: i32,
        health: i32,
    },
    GainRush {
        target: EffectTarget,
    },
    GainImmuneThisTurn {
        target: EffectTarget,
    },
    NextMurlocCostsLess {
        amount: i32,
    },
    GiveNextMurlocDivineShield,
    SetNextKindredTwice,
    DrawKindredAndActivator,
    DrawSpellGiveSpellDamage {
        amount: i32,
    },
    DrawMinionsOfEachCost {
        up_to: i32,
    },
    DrawDeathrattleMinionCostLE {
        max_cost: i32,
    },
    DestroyLowestAttackEnemy,
    TriggerFriendlyCinderDeathrattles,
    DestroyMinionAndGainItsStats {
        target: EffectTarget,
    },
    DealSelfAttackDamage {
        target: EffectTarget,
    },
    SummonRandomMinionCostTaunt {
        cost: i32,
    },
    DiscoverPool {
        pool: DiscoverPool,
    },
    AddRandomCardToHandCount {
        pool: RandomPool,
        count: u8,
    },
    AddCardToHandCount {
        card_id: String,
        count: u8,
    },
    AddTemporaryRandomMinionsCost {
        cost: i32,
        count: u8,
    },
    AddRandomFelSpellsCostHealth {
        count: u8,
    },
    AddRandomHolyAndShadowSpell,
    AddRandomHolySpellCost1,
    CopyRandomHandElementalOrDragon,
    ReduceRandomEnemyHandMinionCost {
        amount: i32,
    },
    ReduceRandomBeastHandCost {
        amount: i32,
    },
    ReduceNonStartingHandCost {
        amount: i32,
    },
    SummonTreantsAttackMinion,
    DealDamageSummonCinders {
        amount: i32,
    },
    DealDamageLowestHealthEnemyRepeated {
        amount: i32,
        times: i32,
    },
    DealDamageRandomEnemies {
        amount: i32,
        count: i32,
    },
    DrawMinionsDifferentTypesBuff {
        count: u8,
        attack: i32,
        health: i32,
    },
    DrawMinionBuffArmorIfAttackGE {
        min_attack: i32,
        buff_health: i32,
        armor: i32,
    },
    SetFlockPending,
    GiveBuffOtherMinionsAttackLE {
        attack: i32,
        health: i32,
        max_attack: i32,
    },
    DestroyMinionSummonRandomSameCost {
        target: EffectTarget,
    },
    SummonMinionsGrantRandomBonus {
        card_id: String,
        count: u8,
    },
    SummonMinionPair {
        a: String,
        b: String,
    },
    SummonRandomMinionCostOrEscalated {
        cost: i32,
        escalated_cost: i32,
    },
    DealDamageGainArmorIfKilled {
        amount: i32,
        armor: i32,
        target: EffectTarget,
    },
    DealDamageAllEnemyMinionsSetMinionsCostMore {
        damage: i32,
    },
    GiveBuffSameType {
        attack: i32,
        health: i32,
        target: EffectTarget,
    },
    GrantRandomBonusEffects {
        count: u8,
        target: EffectTarget,
    },
    GrantRandomBonusEffectAndDeathrattle,
    SetLakkariTicks {
        ticks: u8,
    },
    GiveBuffAndSummonDeathrattle {
        attack: i32,
        health: i32,
        summon_cost: i32,
        target: EffectTarget,
    },
    DealDamageImprovedByShuffles {
        amount: i32,
        target: EffectTarget,
    },
    DrawCardLinkDeathrattle,
    DiscardLinkedDrawnCard,
    GainArmorDealDamageEqual {
        armor: i32,
        target: EffectTarget,
    },
    DestroyDeckTop {
        count: u8,
    },
    ResurrectOneOfEachCostGiveReborn {
        max_cost: i32,
    },
    DealDamageSetNextBeastDiscount {
        amount: i32,
        discount: i32,
        target: EffectTarget,
    },
    BuffAllBeastsEverywhere {
        attack: i32,
        health: i32,
    },
    DealDamageSameType {
        amount: i32,
        target: EffectTarget,
    },
    SetNextTemporaryDiscount {
        amount: i32,
    },
    SetEnemyHeroCantBeHealed,
    DestroyFriendlyMinionAddBones {
        target: EffectTarget,
    },
    GainManaCrystalsMatchOpponent,
    GiveBuffDifferentTypeMinions {
        attack: i32,
        health: i32,
    },
    DealDamageIfQuestPlayed {
        amount: i32,
        target: EffectTarget,
    },
    SwapHeroPowerToDeal8Random,
    RecastRandomHolySpellThisTurn,
    CastRandomSpellFromDeckCostLE {
        max_cost: i32,
    },
    SpendArmorDealDamageAllMinions {
        max_spend: i32,
    },
    DestroyTopCardDiscoverSameRarity,
    GrantKeyword {
        keyword: KeywordKind,
        target: EffectTarget,
    },
    GrantDeathrattleSummon {
        card_id: String,
        count: u8,
        target: EffectTarget,
    },
    AddRandomWeaponAnotherClassComboAttack {
        combo_attack: i32,
    },
    Drain {
        amount: i32,
    },
    GainStatsEqualFireSpellCost,
    DealDamageAndSummon {
        amount: i32,
        card_id: String,
    },
    DiscoverDeckCard,
    DiscoverEnemyDeckTop,
    SummonRandomFelBeast,
    AddRandomBeastCostLess {
        amount: i32,
    },
    TriggerFriendlyDeadDeathrattles {
        count: i32,
    },
    EshoDeckCheckBuffEverywhere {
        attack: i32,
        health: i32,
    },
    SetStatsAllEnemyMinions {
        attack: i32,
        health: i32,
    },
    SummonDamagedCopiesRush,
    SummonTwoDeathrattleMinionsAndFight,
    LohMinionsCost5,
    EliseCraftLocation,
    NiriOfTheCrater,
    SetEventSubjectHealthToSource,
    DealDamageToLeftRightEnemyMinions {
        amount: i32,
    },
    GiveOtherFriendlyMinionsRush,
    DealDamageToTwoAndFreeze {
        amount: i32,
    },
    SetStatsAndFillBoardWithCopies {
        attack: i32,
        health: i32,
    },
    SetStatsAndGrantCharge {
        attack: i32,
        health: i32,
        target: EffectTarget,
    },
    SetStatsGrantLifestealForceAttack {
        attack: i32,
        health: i32,
        target: EffectTarget,
    },
    SetStatsAttachDamageAllDeathrattle {
        attack: i32,
        health: i32,
        target: EffectTarget,
    },
    SetStatsGrantStealthAndDraw {
        attack: i32,
        health: i32,
        draw: i32,
        target: EffectTarget,
    },
    SummonMinionAndBuffFriendlyMinions {
        card_id: String,
        attack: i32,
        health: i32,
    },
    SummonRandomDeckBeastGiveLifesteal,
    SummonRaptorsOutcast {
        count: i32,
    },
    ReduceAdjacentHandCardCost {
        amount: i32,
    },
    DracorexSplash,
    SetHatchingPending,
    DealDamageAndBuffFriendlyElementals {
        damage: i32,
        attack: i32,
        health: i32,
    },
    ShuffleLeftmostHandCardIntoDeck,
    DrawZeroAttackMinion,
    TransformRandomMinionIntoRandomMinion,
    SummonRandomDeathrattleMinionCostGEAndTrigger {
        min_cost: i32,
    },
    SpendCorpsesGainReborn {
        amount: i32,
    },
    SoulrestMarkAndBuff,
    GainStatsAndGrantRush {
        attack: i32,
        health: i32,
        target: EffectTarget,
    },
    BuffHandAndDeckMinions {
        attack: i32,
        health: i32,
    },
    SummonTwoRandomCostBeastsAttackRandomEnemies {
        cost: i32,
    },
    SummonRandomLegendaryMinionSetStats {
        attack: i32,
        health: i32,
    },
    SummonRandomCostMinionSetStats {
        cost: i32,
        attack: i32,
        health: i32,
    },
    AddRandomMaskCombo {
        reduction: i32,
    },
    GainStatsOfRandomLegendaryBeast,
    SummonRandomLegendaryBeast,
    SummonRandomTauntMinionCostGE {
        min_cost: i32,
    },
    SummonRandomTauntMinionsOfCosts {
        a: i32,
        b: i32,
        c: i32,
    },
    AddRandomOneCostMinion,
    AddRandomOneCostSpell,
    AddRandomMultiTribeMinion,
    // M3-W2a (Across the Timeways — the 120 non-legendary TIME cards)
    AddRandomMinionCostsLess {
        reduction: i32,
    },
    AddRandomSpellsFromClass {
        count: i32,
    },
    DrawRandomMinionGiveStats {
        attack: i32,
        health: i32,
    },
    BothPlayersDiscardRandomCard,
    SummonManaWorthRandomMinions {
        total: i32,
    },
    GetHolySpellsRestoreHealthEqualCosts,
    CastRandomNatureSpells {
        count: i32,
    },
    BothPlayersEquipRandomWeaponBuffOurs {
        attack: i32,
        health: i32,
    },
    AddRandomRewindCardToHand,
    CopyRightmostEnemyHandCardOrIncreaseCost,
    SummonTwoRandomLegendaryMinions,
    DiscoverEnemyHandCardCopy,
    SummonTauntAndIfHoldingDragonAgain {
        card_id: String,
    },
    RestoreAndGrantHeroDivineShield {
        amount: i32,
    },
    DiscoverPaladinMechPastGiveStats {
        attack: i32,
        health: i32,
    },
    DealDamageAllEnemiesIfControllingAura {
        amount: i32,
    },
    GiveHeroImmuneThisTurn,
    DrawBottomCards {
        count: i32,
    },
    BuffAllFriendlyMinionsShuffleShreds {
        attack: i32,
        health: i32,
    },
    DealDamageSplitAmongAllEnemiesShuffleShreds {
        amount: i32,
    },
    CastShredFromDeckGainStats {
        attack: i32,
        health: i32,
    },
    CastShredFromDeckSummonCopy,
    CopyRandomHandMinion,
    DrawCardsOfDifferentCosts {
        count: i32,
    },
    DrawMinionAndBuffHandMinionsHealth {
        health: i32,
    },
    SetStatsAndCantAttackHeroesThisTurn {
        attack: i32,
        health: i32,
    },
    GainStatsPerTurnTaken {
        attack: i32,
        health: i32,
    },
    TransformSelfToRandomMinionOfCost {
        cost: i32,
    },
    SwapStatsIfSurvivesDamage,
    GiveCoin,
    TransformSelfIfSurvivesDamageToRandomCost {
        cost: i32,
    },
    ResetBothHandsCosts,
    SummonRandomMinionOfCostDormant {
        cost: i32,
        turns: u32,
    },
    SummonRandomDragonCostGE {
        min_cost: i32,
    },
    ReverseDeckOrder,
    GainTauntAndDivineShieldIfHoldingDragon,
    AddRandomCostMinionMarkedTurnDiscount {
        cost: i32,
    },
    DealDamageFriendlyMinionToRandomEnemy {
        damage: i32,
        amount: i32,
    },
    GainStatsAndDrawIfNatureSpellCast {
        attack: i32,
        health: i32,
    },
    DamageAllMinionsAndAddCardToHand {
        amount: i32,
        card_id: String,
    },
    DamageAndDrawTwoIfSurvives {
        damage: i32,
        target: EffectTarget,
    },
    DamageMinionGiveHeroAttack {
        damage: i32,
        attack: i32,
    },
    DealDamageEnemyMinionEqualToSourceHealth,
    SetHandMinionStatsToHigher,
    RestoreHealthEqualToSourceHealth,
    DiscoverDeckAndEnemyHandCardCopy,
    SilenceAndDestroyRandomEnemyMinion,
    SummonShadowAttacksRandomEnemy {
        card_id: String,
    },
    SummonTwoDemonsAttackLowestHealthIfDeckNoMinions,
    GrantDivineShieldAndBuffHandMinionsHealth {
        health: i32,
    },
    DiscoverMinionReduceHandCostsIfDeckNoMinions {
        reduction: i32,
    },
    GainHeroAttackAndBuffHandMinionsIfDeckNoMinions {
        attack: i32,
    },
    PreciseShot {
        amount: i32,
        center_amount: i32,
    },
    DrawUntilHandSize {
        size: i32,
    },
    SummonRandomCostBeastAttackRandomEnemy {
        cost: i32,
    },
    SummonMinionsGrantTwoRandomBonus {
        card_id: String,
        count: i32,
    },
    AddRandomLegendaryMinionCostReduced {
        reduction: i32,
    },
    DealDamageEnemyMinionIfHeroHealthChanged {
        amount: i32,
    },
    FillHandWithRandomUndeadCostHealth,
    SummonHighestCostFallenUndead,
    SetChronologicalAura {
        ticks: i32,
    },
    DiscoverDeckCardOthersBottom,
    DamageAndGainArmorIfMinionPlayedWhileHeld {
        damage: i32,
        armor: i32,
    },
    GainStatsAndSummonCopyIfHeroHealthLE {
        attack: i32,
        health: i32,
        threshold: i32,
    },
    GetPupilAndDiscoverSpellCostGE {
        pupil_card_id: String,
        min_cost: i32,
    },
    ReplaceHandAndDeckWithRandomChooseOne,
    SummonTwoRandomCostMinionsWithAttack {
        cost: i32,
        bonus: i32,
    },
    DestroyMinionAndSummonRandomCost {
        cost: i32,
    },
    NextTurnEnemyCardsCostMore {
        amount: i32,
    },
    AddRandomBeastsToBottomDeckWithStats {
        count: i32,
        attack: i32,
        health: i32,
    },
    DamageAndDrawMinionIfHoldingCostGE {
        damage: i32,
        cost: i32,
    },
    DrawTwoReduceRandomCost {
        reduction: i32,
    },
    DealDamagePrimaryAndSplash {
        primary: i32,
        splash: i32,
    },
    DiscoverArcaneSpellsReduced {
        reduction: i32,
    },
    DealDamageAndDrawExcess {
        amount: i32,
    },
    SummonPairScrambleStats {
        first_cost: i32,
        second_cost: i32,
    },
    LookAtSecretsGiveRandom,
    SummonRandomDeckMinionAndTigerForOpponent {
        card_id: String,
    },
    GainStatsPerDamagedMinion {
        attack: i32,
        health: i32,
    },
    FillEnemyBoardWithRandomCost1Minions,
    GainArmorAndSummonTwoBeastsForOpponent {
        armor: i32,
        card_id: String,
    },
    ImprisonEnemyMinion,
    AwakenImprisonedMinion,
    GuessEnemyHandGainHealth {
        health: i32,
    },
    TransformHandSelfToRandomEnemyHandMinion,
    ResurrectDiedMinionFull,
    SummonRandomMinionFromDeck,
    SylvanasDealToAllEnemiesRepeated {
        damage: i32,
    },
    ChronogorDrawsHighestLowest,
    DiscardHandAndAddInfiniteBanana,
    MurozondPrepareInfiniteAttack,
    AddCopiesOfLastPlayedCards {
        count: i32,
    },
    SummonLocationForPlayer {
        card_id: String,
    },
    SummonCopyOfFriendlyMinion,
    FillHandWithRandomTemporarySpells,
    TakeControlEnemyMinionHealthLE,
    DiscoverDemonGE5AndSetNextDemonCostOne,
    SpendCorpsesRestoreHeroHealth {
        max: i32,
    },
    DrawOrResurrectBwonsamdiAndGrantBoon {
        keyword: KeywordKind,
    },
    SummonRandomCostMinion {
        cost: i32,
    },
    SetDeckBottomCostsOne {
        count: i32,
    },
    ReplaceHandAndSwapBackAtTurnEnd,
    RestoreHandSnapshot,
    SummonChestForOpponent,
    FillOpponentHandWithCoins,
    DestroyAllMinionsOpponentPlayedLastTurn,
    SummonBloodFighterFromHandBuffAndAttack,
    GetThreeRandomSpellsFromPastTracked,
    DestroyHeldKingLlaneAndHalveEnemyHealth,
    SilenceAndDestroyAllOtherMinions,
    // M3-W3 — the Across the Timeways closing wave (The End of Time
    // miniset, 38 END_ cards; the mirror stays in lockstep with the
    // CardEffect declarations above, enforced by
    // `card_effect_de_mirror_order_matches`).
    DealDamageAndImbue {
        amount: i32,
        target: EffectTarget,
    },
    DrawUndeadAndImbueTwice,
    EquipDaggerOrBuffWeapon,
    BygoneEchoesSummon,
    ChronikarHeroAttackBuff,
    ChronikarRebuff,
    PressTheAdvantage,
    RefreshManaCrystals {
        amount: i32,
    },
    SummonTwoTreantsScaling,
    SetAllOtherMinionsAttack {
        attack: i32,
    },
    SetAllOtherMinionsHealth {
        health: i32,
    },
    ArmAccelerationAura,
    SetWeaponAttackInfinityThisTurn,
    DamageAndBuffFriendlyIfKilled {
        amount: i32,
        attack: i32,
        health: i32,
    },
    AddRandomDeathrattleMinionCostsLess,
    DiscardHighestCostCard,
    DrawUntilHandFull,
    EmptyOpponentHand,
    SetRandomHandCardCostInfinity,
    RestoreInfinityHandCardCost,
    GainStatsIfHeroDamagedThisTurn {
        attack: i32,
        health: i32,
    },
    DamageMinionDrawIfSurvivesSummonIfDies {
        amount: i32,
    },
    BuffHandMinionsAndWeapons {
        attack: i32,
    },
    FreezeMinionAndNeighborsDestroyDamaged,
    InfiniteDamageToHighestHealthEnemyMinion,
    DamageMinionEternalFirebolt {
        amount: i32,
    },
    DestroyAllMinionsWith4OrLessAttack,
    AddRandomShadowSpell,
    OverloadForAndGainImmuneWindfury {
        overload: i32,
    },
    DestroyRandomEnemyMinionLocationWeapon,
    DestroyTopFiveEnemyDeckIfOwnEmpty,
    FillBoardRandomDragonsHealHeroSkipNextTurn,
    GetRandomOtherClassMinionCostsLess {
        reduction: i32,
    },
    BuffFirstUndeadPlayedEachTurn {
        attack: i32,
    },
    GainStatsAndCopyToColossalMain {
        attack: i32,
        health: i32,
    },
    RemoveKeywordFromColossalMain {
        keyword: KeywordKind,
    },
    AddRandomCostMinionCostsHealth {
        cost: i32,
    },
    ColossalArmDestroyRight {
        attack: i32,
        health: i32,
    },
    AddRandomFireSpellCostsLess {
        reduction: i32,
    },
    TriggerFriendlyDeathrattles,
    AddRandomMinionsCostEqualAttack {
        count: i32,
    },
    GiveRandomFriendlyMinionAttack {
        attack: i32,
    },
    SummonMinionsAndGrantDeathrattleAll {
        card_id: String,
        count: u32,
    },
    GainStatsElusiveAndSummonCopy {
        attack: i32,
        health: i32,
    },
    SummonMinionsAndGrantFriendlyAttackDivineShield {
        card_id: String,
        count: u32,
        attack: i32,
    },
    DealDamageAndDamageAllEnemies {
        amount: i32,
        aoe: i32,
    },
    DrawMinionsAndBuffHandMinions {
        count: u32,
        attack: i32,
        health: i32,
    },
    AddRandomShatterCardToHand,
    ReplaceHero {
        card_id: String,
    },
    ChooseCataclysms,
    ChooseHandCard,
    RefreshManaIfHoldingDragon {
        amount: i32,
    },
    GainTempManaOrPermanentIfSpent {
        threshold: i32,
    },
    GetOrSummonTauntWhelpsIfSpent {
        threshold: i32,
    },
    SummonGolemsSpendAllMana,
    ShuffleRandomMinionsCostGE8DoubleStats,
    GainStatsPerFriendlyMinionTargeted,
    FillHandWithRandomDragons {
        threshold: i32,
    },
    SetAttackEqualToSource,
    NextMurlocCostsHealth,
    TransformRandomEnemyMinionToSelf,
    GiveOpponentSabotage,
    ReturnEnemyMinionCantPlayNextTurn,
    SetHealingBonus {
        amount: i32,
    },
    SetNextHealDealsDamage,
    FullHealMinionAndDraw,
    DamageMinionHealEnemyHeroIfKilled {
        amount: i32,
    },
    LifestealSelfDamage {
        amount: i32,
    },
    GainHealthIfFullHealth {
        amount: i32,
    },
    SetRemainingHealthAndFullHealDamage {
        health: i32,
        damage: i32,
    },
    BuffSpellDamageHandAndDeck,
    AddDragonBreathScaledByAttack,
    DealDamageEqualDragonBreath,
    SummonFiveHungryDrakesSpendCorpsesRush,
    RefreshManaEqualSelfAttack,
    AddNefariansCreation {
        reduction: i32,
    },
    GrantDeathrattleSummonRandomCostMinion {
        cost: i32,
    },
    TriggerRandomFriendlyEndTurnEffect,
    GrantDivineShieldOrGainStats {
        attack: i32,
        health: i32,
    },
    AddRandomHolySpellCostReduced {
        reduction: i32,
    },
    SummonDragonWithSelfStats,
    SetEndTurnEffectsTwice {
        turns: u32,
    },
    DevourTwoEnemyHandCardsAndDormant,
    ReturnDevouredCards,
    SummonCopyIfSpellDamageDealtThisTurn,
    DealDamageAndDamageRandomEnemyMinion {
        amount: i32,
        secondary: i32,
    },
    GainAttackFirstSpellDamageThisTurn {
        attack: i32,
    },
    DamageAllMinionsRepeatDescending {
        amount: i32,
    },
    SummonCopyOfDiscardedMinion,
    TakeControlUntilEndOfTurnCantAttack,
    DealDamageToTwoScaledByHandTurns {
        base: i32,
    },
    DamageAllMinionsDrawPerDeath {
        amount: i32,
    },
    ReopenLocation,
    SetNextTurnSummon {
        card_id: String,
    },
    DealDamageLeftRightOutcastAgain {
        amount: i32,
    },
    DealDamageEqualSelfAttack,
    SetDragonsHaveRush,
    SetEnemyMinionHealthTo1IfHoldingDragon,
    AddRandomDragonCostLE3,
    DealDamageOrDamageAllEnemiesIfCopyPlayed {
        amount: i32,
    },
    ReplayOneCostCardsPlayedThisGame,
    CastAbsorbedSpell,
    GrantMegaWindfuryCantAttackHeroes,
    TransformFriendlyMinionsCost1MoreSummonOriginals,
    SummonRandomThreeTwoOneCostMinions,
    DrawAndReduceCostRepeated {
        reduction: i32,
    },
    DamageAllMinionsScaledByBoard {
        base: i32,
    },
    DamageAllMinionsAndGainHeroAttack {
        amount: i32,
        attack: i32,
    },
    DealDamageSplitAmongEnemiesIfFireSpell {
        amount: i32,
    },
    DamageDamagedMinionReturnIfExcess {
        amount: i32,
    },
    SetGeddonDiscoverDraw,
    GrantDeathrattleSummonRandomHandMinion,
    SummonRandomMinionFromHand,
    UpgradeHeroPowerCost1,
    AddRandomPaladinAura,
    StealHealthThreeTimes {
        amount: i32,
    },
    DestroyDeckCardsCostLE2Both,
    UnlockOverloadedCrystals,
    CastRandomSpellSameCostOtherClass,
    ReturnHoardCostLess,
    DamageMinionReduceHandCostByExcess {
        amount: i32,
    },
    AddTwoRandomSpellsSameCost {
        cost: i32,
    },
    Split100StatsAmongDeckMinionsIfCostGE100 {
        threshold: i32,
    },
    DestroyHighestHealthEnemyMinion,
    ShuffleRandomLegendaryDragonsCost1,
    DestroyLegendaryMinion,
    GrantRandomFriendlyMinionAttack {
        attack: u8,
    },
    DiscoverOneCostSpell,
    DiscoverAnySpell,
    SummonTwoRandomOneCostMinions,
    ReopenLocationIfFelSpell,
    SummonRandomMinionsOfCost {
        cost: u8,
        count: u8,
    },
    AddRandomNagaCost1,
    HoggerStartOfGame,
    AzalinaStartOfGame,
    GodfreyStartOfGame,
    NethrekStartOfGame,
    MugzeeStartOfGame,
    BeatrixStartOfGame,
    AyaUpgradeCoins {
        card_id: String,
    },
    KarovThreeLegendaryCopies,
    ThalenaSecondHeroPower,
    ManastormSetAfterSpell,
    VanessaGetBattlecryMinionCost2Less,
    TrastathGainSummonedDemonStats,
    MoraggDeathrattle,
    SummonMoragg,
    PassiveHeroPower,
    JadeCoin,
    GrimyCoin,
    KabalCoin,
    // M5-W2 — the closing Escape from Violet Hold wave
    SliceAndDice,
    TinyPal {
        ammo: u8,
    },
    StaffOfTrickery,
    R4TCatcher,
    IridaSinseeker,
    IridaGetVoid,
    KingOfTheUnderbelly,
    MurlocHolmes,
    TogwaggleShuffleHands,
    MaievBuffDormant,
    ZuramatPlaysDiscarded,
    ColdSnap,
    MindSweeper,
    EnthralledShade,
    TricksyImproviser,
    Judgment,
    ReinforcementAura,
    ScarletBruiser,
    BallAndChain,
    HolyBola,
    SpireSecurity,
    SmuggledShovel,
    ScrambleForGear,
    ReleaseTheBeasts,
    SewerSwimmer,
    Imfernal,
    ImpGangStooge,
    DisguisedDoctor,
    Sawbones,
    BoneFlurry,
    DrinkBlood,
    EmergencySurgery,
    DisguisedWatchman,
    PickPocket,
    JadeGuardians,
    CrowdControl,
    VigilantSentry,
    VioletPunisher,
    BreakoutArchitect,
    NoxiousBribe,
    AlarmOMatic,
    SpitefulChef,
    Annihilation,
    SpireOfSolitude,
    ShadowRounds,
    ScarletRecruiter,
    ThievesTools,
    VoidSoul,
    CodeViolet,
    MoltenGold,
    Frostshatter,
    Stormfury,
    Hexmarshal,
    LethalRecipe,
    DigForFreedom,
    GuardDog,
    BeastTripwire,
    ArcaneTripwire,
    CapturedArchmage,
    FranticForger,
    LowSecurityWing,
    DemonicConfinement,
    WidowsBite,
    WidowsFeast,
    WidowsBanquet,
    Nab,
    VoidBlast,
    CosmicManifestations,
    DefiasWannabe,
    Soothsayer,
    HolyEmbrace,
    RatBurglar,
    DarkBribe,
    AncientAugurPick,
    AncientAugurDeathrattle,
    ArachnathidVenom,
    TruthSeeker,
    ConcealingConfection,
    DisguisedExecutioner,
    GetawayHogdriver,
    EscapeArtist,
    BloodClone,
    GallagioGoon,
    BlackMarketOverseer,
    ActivatedGolem,
    Hellraiser,
    SummonTwoRandomMinionsOfCost {
        cost: i32,
    },
    RefreshManaIfNoMinionPlayedLastTurn {
        amount: i32,
    },
    RestoreAllFriendlyAndSummonTwoRandomCostMinions {
        heal: i32,
        cost: i32,
    },
    DrawAndGainArmorRepeatIfNoMinionPlayedLastTurn {
        draw: i32,
        armor: i32,
    },
    BuffHealthTauntAndDormant {
        health: i32,
    },
    AddRandomDragonCostReduced {
        reduction: i32,
    },
    GetThreeTreantsAndCarveNatureSpells,
    CastRandomSpellsScaledByHandTurns {
        base: i32,
        count: i32,
    },
    SetCompanionReplacement {
        bump: i32,
    },
    SetCompanionReplacementAndDraw {
        bump: i32,
        draw: i32,
    },
    SetCompanionBonus {
        amount: i32,
    },
    ReplaceCompanionsAndSummonRandomBeast {
        bump: i32,
        cost: i32,
    },
    SplitDamageAmongAllEnemiesChainOnDeath {
        amount: i32,
    },
    BuffFriendlyBeastAndRandomHandBeast {
        attack: i32,
        health: i32,
    },
    DealDamageToRandomEnemyMinionExcessToHero {
        amount: i32,
        times: i32,
    },
    SetLeylineDiscount {
        amount: i32,
    },
    AddRandomLeylineToHand,
    SummonRandomCostMinionTimes {
        cost: i32,
        times: i32,
    },
    SetLeylineExtraTrigger {
        amount: i32,
    },
    DrawCardsCostsLess {
        reduction: i32,
        count: i32,
    },
    GetAllLeylinesAndUpgrade {
        upgrade: LeylineUpgrade,
    },
    SetLeylineEffectBonus {
        amount: i32,
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
            CardEffectDe::SetHealth { health, target } => CardEffect::SetHealth { health, target },
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
            CardEffectDe::GainStatsAndGrantWindfury {
                attack,
                health,
                target,
            } => CardEffect::GainStatsAndGrantWindfury {
                attack,
                health,
                target,
            },
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
            CardEffectDe::DiscardHand => CardEffect::DiscardHand,
            CardEffectDe::NextSpellDiscount { amount } => CardEffect::NextSpellDiscount { amount },
            CardEffectDe::GrantAdjacentStatsAndDivineShield { attack, health } => {
                CardEffect::GrantAdjacentStatsAndDivineShield { attack, health }
            }
            CardEffectDe::DestroyAllOtherMinionsAndDiscardHand => {
                CardEffect::DestroyAllOtherMinionsAndDiscardHand
            }
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
            CardEffectDe::GrantAttackAndImmune { attack, target } => {
                CardEffect::GrantAttackAndImmune { attack, target }
            }
            CardEffectDe::PreventFatalDamageAndImmune => CardEffect::PreventFatalDamageAndImmune,
            CardEffectDe::TakeControlUntilEndOfTurn => CardEffect::TakeControlUntilEndOfTurn,
            CardEffectDe::TakeControl => CardEffect::TakeControl,
            CardEffectDe::TakeControlAttackLE { max_attack } => {
                CardEffect::TakeControlAttackLE { max_attack }
            }
            CardEffectDe::Corrupt => CardEffect::Corrupt,
            CardEffectDe::MinHealthUntilEndOfTurn => CardEffect::MinHealthUntilEndOfTurn,
            CardEffectDe::TransformToRandom { card_a, card_b } => CardEffect::TransformToRandom {
                card_a: intern(card_a)?,
                card_b: intern(card_b)?,
            },
            CardEffectDe::AddRandomCardToHand { pool } => CardEffect::AddRandomCardToHand { pool },
            CardEffectDe::DiscoverDeckTop3 => CardEffect::DiscoverDeckTop3,
            CardEffectDe::SummonRandomMinion { pool } => CardEffect::SummonRandomMinion { pool },
            CardEffectDe::AddCardToHand { card_id } => CardEffect::AddCardToHand {
                card_id: intern(card_id)?,
            },
            CardEffectDe::DealDamageAndSummonIfKilled { amount, pool } => {
                CardEffect::DealDamageAndSummonIfKilled { amount, pool }
            }
            CardEffectDe::DrawCardByRace { count, race } => {
                CardEffect::DrawCardByRace { count, race }
            }
            CardEffectDe::Demonfire {
                damage,
                attack_bonus,
                health_bonus,
            } => CardEffect::Demonfire {
                damage,
                attack_bonus,
                health_bonus,
            },
            CardEffectDe::GainStatsAndTaunt {
                attack,
                health,
                target,
            } => CardEffect::GainStatsAndTaunt {
                attack,
                health,
                target,
            },
            CardEffectDe::CopyRandomEnemyHandCard { count } => {
                CardEffect::CopyRandomEnemyHandCard { count }
            }
            CardEffectDe::CopyRandomEnemyDeckCards { count } => {
                CardEffect::CopyRandomEnemyDeckCards { count }
            }
            CardEffectDe::SummonRandomEnemyDeckMinion { fallback_card_id } => {
                CardEffect::SummonRandomEnemyDeckMinion {
                    fallback_card_id: intern(fallback_card_id)?,
                }
            }
            CardEffectDe::CopyCastSpellToOtherPlayerHand => {
                CardEffect::CopyCastSpellToOtherPlayerHand
            }
            CardEffectDe::FillHandWithMinion { card_id } => CardEffect::FillHandWithMinion {
                card_id: intern(card_id)?,
            },
            CardEffectDe::ForceEnemyMinionsAttackThis => CardEffect::ForceEnemyMinionsAttackThis,
            CardEffectDe::SpendCorpsesSummonCopy { cost } => {
                CardEffect::SpendCorpsesSummonCopy { cost }
            }
            CardEffectDe::DrawCardOutcast { normal, outcast } => {
                CardEffect::DrawCardOutcast { normal, outcast }
            }
            CardEffectDe::OutcastDamage {
                amount,
                outcast_amount,
                target,
            } => CardEffect::OutcastDamage {
                amount,
                outcast_amount,
                target,
            },
            CardEffectDe::RestoreRandomFriendly { amount } => {
                CardEffect::RestoreRandomFriendly { amount }
            }
            CardEffectDe::DestroyEnemyLocation => CardEffect::DestroyEnemyLocation,
            CardEffectDe::DamagePlayedMinionAndExcess { amount } => {
                CardEffect::DamagePlayedMinionAndExcess { amount }
            }
            CardEffectDe::DamageAndDrawIfHandEmpty { damage, target } => {
                CardEffect::DamageAndDrawIfHandEmpty { damage, target }
            }
            CardEffectDe::AoeDamageAndHealFriendly { damage, heal } => {
                CardEffect::AoeDamageAndHealFriendly { damage, heal }
            }
            CardEffectDe::DamageAndDrawIfSurvives { damage, target } => {
                CardEffect::DamageAndDrawIfSurvives { damage, target }
            }
            CardEffectDe::GainArmorAndDraw { armor, draw } => {
                CardEffect::GainArmorAndDraw { armor, draw }
            }
            CardEffectDe::DamageAndDrawIfKilled { damage, target } => {
                CardEffect::DamageAndDrawIfKilled { damage, target }
            }
            CardEffectDe::DamageAndGainArmor {
                damage,
                armor,
                target,
            } => CardEffect::DamageAndGainArmor {
                damage,
                armor,
                target,
            },
            CardEffectDe::GainPoisonousToFriendlyUndead => {
                CardEffect::GainPoisonousToFriendlyUndead
            }
            CardEffectDe::TransformToMinion { card_id } => CardEffect::TransformToMinion {
                card_id: intern(card_id)?,
            },
            CardEffectDe::GrantDeathrattleToTarget { card_id } => {
                CardEffect::GrantDeathrattleToTarget {
                    card_id: intern(card_id)?,
                }
            }
            CardEffectDe::DestroyAllMinionsAttackGE { attack } => {
                CardEffect::DestroyAllMinionsAttackGE { attack }
            }
            CardEffectDe::AoeDamageAndDraw { damage, draw } => {
                CardEffect::AoeDamageAndDraw { damage, draw }
            }
            CardEffectDe::GainStatsTauntAndDeathrattle {
                attack,
                health,
                card_id,
            } => CardEffect::GainStatsTauntAndDeathrattle {
                attack,
                health,
                card_id: intern(card_id)?,
            },
            CardEffectDe::SummonRandomFishFromDeck => CardEffect::SummonRandomFishFromDeck,
            CardEffectDe::AddRandomSpellToOpponentDeckTop => {
                CardEffect::AddRandomSpellToOpponentDeckTop
            }
            CardEffectDe::SummonStatueTrio => CardEffect::SummonStatueTrio,
            CardEffectDe::CopyEnemyDeckCardOnSelfAttack => {
                CardEffect::CopyEnemyDeckCardOnSelfAttack
            }
            CardEffectDe::DamageAndSummon {
                damage,
                target,
                card_id,
            } => CardEffect::DamageAndSummon {
                damage,
                target,
                card_id: intern(card_id)?,
            },
            CardEffectDe::RehgarBolt => CardEffect::RehgarBolt,
            CardEffectDe::DamageTwoDrawIfKilled { damage } => {
                CardEffect::DamageTwoDrawIfKilled { damage }
            }
            CardEffectDe::FreezeAndDiscoverSpell => CardEffect::FreezeAndDiscoverSpell,
            CardEffectDe::HenchThugBuff => CardEffect::HenchThugBuff,
            CardEffectDe::SummonRecruitsAndEquipWeapon => CardEffect::SummonRecruitsAndEquipWeapon,
            CardEffectDe::BuffAndSummonRandomCost2 => CardEffect::BuffAndSummonRandomCost2,
            CardEffectDe::DamageAndSummonCopyIfKilled { damage } => {
                CardEffect::DamageAndSummonCopyIfKilled { damage }
            }
            CardEffectDe::KeymasterCopy => CardEffect::KeymasterCopy,
            CardEffectDe::FordragonBuff => CardEffect::FordragonBuff,
            CardEffectDe::DamageAndSummonVoidwalkers { damage, target } => {
                CardEffect::DamageAndSummonVoidwalkers { damage, target }
            }
            CardEffectDe::DamageAndAddToHand { damage, card_id } => {
                CardEffect::DamageAndAddToHand {
                    damage,
                    card_id: intern(card_id)?,
                }
            }
            CardEffectDe::AddRandomOtherClassSpells { count, min_cost } => {
                CardEffect::AddRandomOtherClassSpells { count, min_cost }
            }
            CardEffectDe::SummonFelbatOnDraw => CardEffect::SummonFelbatOnDraw,
            CardEffectDe::SpendCorpsesSummonRandomMinion { max } => {
                CardEffect::SpendCorpsesSummonRandomMinion { max }
            }
            CardEffectDe::GainStatsAndDraw {
                attack,
                health,
                target,
                draw,
            } => CardEffect::GainStatsAndDraw {
                attack,
                health,
                target,
                draw,
            },
            CardEffectDe::DamageUndamaged { damage } => CardEffect::DamageUndamaged { damage },
            CardEffectDe::DamageMinionAndSelfHero { damage } => {
                CardEffect::DamageMinionAndSelfHero { damage }
            }
            CardEffectDe::GainHeroAttackAndDraw { attack } => {
                CardEffect::GainHeroAttackAndDraw { attack }
            }
            CardEffectDe::GainArmorAndSummonDeckMinion { armor, max_cost } => {
                CardEffect::GainArmorAndSummonDeckMinion { armor, max_cost }
            }
            CardEffectDe::GainArmorAndDrawOnHeroAttack { armor } => {
                CardEffect::GainArmorAndDrawOnHeroAttack { armor }
            }
            CardEffectDe::SummonAllCompanions => CardEffect::SummonAllCompanions,
            CardEffectDe::DamageFreezeAllAndSummon { damage, card_id } => {
                CardEffect::DamageFreezeAllAndSummon {
                    damage,
                    card_id: intern(card_id)?,
                }
            }
            CardEffectDe::DestroyHighestAttackEnemy => CardEffect::DestroyHighestAttackEnemy,
            CardEffectDe::SummonZombiesWithCorpseReborn { corpses } => {
                CardEffect::SummonZombiesWithCorpseReborn { corpses }
            }
            CardEffectDe::TransformSelfToCastSpell => CardEffect::TransformSelfToCastSpell,
            CardEffectDe::BuffHandMinionsWithCorpses { corpses } => {
                CardEffect::BuffHandMinionsWithCorpses { corpses }
            }
            CardEffectDe::RestoreHealthAndDraw { amount, target } => {
                CardEffect::RestoreHealthAndDraw { amount, target }
            }
            CardEffectDe::DrawIfUnspentMana => CardEffect::DrawIfUnspentMana,
            CardEffectDe::GainArmorAndSummonRandomCost { armor, cost } => {
                CardEffect::GainArmorAndSummonRandomCost { armor, cost }
            }
            CardEffectDe::GainStatsIfHealedThisTurn { attack, health } => {
                CardEffect::GainStatsIfHealedThisTurn { attack, health }
            }
            CardEffectDe::BattleToTheDeath => CardEffect::BattleToTheDeath,
            CardEffectDe::NextDemonDiscount { amount } => CardEffect::NextDemonDiscount { amount },
            CardEffectDe::BuffHandMinions { attack, health } => {
                CardEffect::BuffHandMinions { attack, health }
            }
            CardEffectDe::SummonRandomEnemyHandMinion => CardEffect::SummonRandomEnemyHandMinion,
            CardEffectDe::DrawForBoth => CardEffect::DrawForBoth,
            CardEffectDe::NextComboDiscount { amount } => CardEffect::NextComboDiscount { amount },
            CardEffectDe::AddRandomMageSpells { count } => {
                CardEffect::AddRandomMageSpells { count }
            }
            CardEffectDe::DamageEnemyHeroAndHealSelf { amount } => {
                CardEffect::DamageEnemyHeroAndHealSelf { amount }
            }
            CardEffectDe::LoseHealthPerOpponentHandCard => {
                CardEffect::LoseHealthPerOpponentHandCard
            }
            CardEffectDe::GrantRandomFriendlyDivineShieldTaunt => {
                CardEffect::GrantRandomFriendlyDivineShieldTaunt
            }
            CardEffectDe::RemoveTopEnemyDeckCard => CardEffect::RemoveTopEnemyDeckCard,
            CardEffectDe::DiscoverSpellAndHealCost => CardEffect::DiscoverSpellAndHealCost,
            CardEffectDe::DrawBeastDragonMurloc => CardEffect::DrawBeastDragonMurloc,
            CardEffectDe::AddRandomOtherClassCard => CardEffect::AddRandomOtherClassCard,
            CardEffectDe::AddRandomShamanSpell => CardEffect::AddRandomShamanSpell,
            CardEffectDe::DamageSelfHero { damage } => CardEffect::DamageSelfHero { damage },
            CardEffectDe::SummonTwoCopiesOfSelf => CardEffect::SummonTwoCopiesOfSelf,
            CardEffectDe::SpendCorpsesDamageRandom { max, damage } => {
                CardEffect::SpendCorpsesDamageRandom { max, damage }
            }
            CardEffectDe::SpendCorpsesSummonFootmen { max } => {
                CardEffect::SpendCorpsesSummonFootmen { max }
            }
            CardEffectDe::OngoingEndTurnDamage { damage } => {
                CardEffect::OngoingEndTurnDamage { damage }
            }
            CardEffectDe::DamageAllOtherMinions { damage } => {
                CardEffect::DamageAllOtherMinions { damage }
            }
            CardEffectDe::BuffTauntHandMinions { attack, health } => {
                CardEffect::BuffTauntHandMinions { attack, health }
            }
            CardEffectDe::SummonRandomMinionCostEqHandSize => {
                CardEffect::SummonRandomMinionCostEqHandSize
            }
            CardEffectDe::ResurrectHighestCostFallen => CardEffect::ResurrectHighestCostFallen,
            CardEffectDe::GrantDeathrattleSummonOwnCost => {
                CardEffect::GrantDeathrattleSummonOwnCost
            }
            CardEffectDe::AddRandomPirateToHand => CardEffect::AddRandomPirateToHand,
            CardEffectDe::NextEnemyHeroPowerCostMore { amount } => {
                CardEffect::NextEnemyHeroPowerCostMore { amount }
            }
            CardEffectDe::SummonRandomMinionOfCost { cost } => {
                CardEffect::SummonRandomMinionOfCost { cost }
            }
            CardEffectDe::SummonRandomDemonFromHandOrDeck => {
                CardEffect::SummonRandomDemonFromHandOrDeck
            }
            CardEffectDe::NextEnemySpellsCostMore { amount } => {
                CardEffect::NextEnemySpellsCostMore { amount }
            }
            CardEffectDe::BuffWeaponDurabilityIfBeast { amount } => {
                CardEffect::BuffWeaponDurabilityIfBeast { amount }
            }
            CardEffectDe::ReturnLastTurnSpells => CardEffect::ReturnLastTurnSpells,
            CardEffectDe::DestroyMinionAndSelfDamage => CardEffect::DestroyMinionAndSelfDamage,
            CardEffectDe::DamageSelfMinion { damage } => CardEffect::DamageSelfMinion { damage },
            CardEffectDe::AddRandomOneCostCard => CardEffect::AddRandomOneCostCard,
            CardEffectDe::BuffThreeDifferentRaces { attack, health } => {
                CardEffect::BuffThreeDifferentRaces { attack, health }
            }
            CardEffectDe::AddFiveRandomCards => CardEffect::AddFiveRandomCards,
            CardEffectDe::DiscardTwoRandomCards => CardEffect::DiscardTwoRandomCards,
            CardEffectDe::DamageAllMinionsIfHoldingDragon { damage } => {
                CardEffect::DamageAllMinionsIfHoldingDragon { damage }
            }
            CardEffectDe::DamageAllEnemiesByAttack => CardEffect::DamageAllEnemiesByAttack,
            CardEffectDe::ReturnRandomFriendlyAndReduceCost { amount } => {
                CardEffect::ReturnRandomFriendlyAndReduceCost { amount }
            }
            CardEffectDe::GrantAttackToRandomFriendly => CardEffect::GrantAttackToRandomFriendly,
            CardEffectDe::SummonRandomLegendaryMinion => CardEffect::SummonRandomLegendaryMinion,
            CardEffectDe::ResurrectWeaponKilled => CardEffect::ResurrectWeaponKilled,
            CardEffectDe::DestroyRandomEnemyMinion => CardEffect::DestroyRandomEnemyMinion,
            CardEffectDe::SummonOasisWaterElemental => CardEffect::SummonOasisWaterElemental,
            CardEffectDe::SummonRandomCostAndFreeze { cost } => {
                CardEffect::SummonRandomCostAndFreeze { cost }
            }
            CardEffectDe::DamageAndAddRandomSpell { damage, target } => {
                CardEffect::DamageAndAddRandomSpell { damage, target }
            }
            CardEffectDe::FreezeAndSummonElementals => CardEffect::FreezeAndSummonElementals,
            CardEffectDe::AddRandomTauntBuffed => CardEffect::AddRandomTauntBuffed,
            CardEffectDe::AddRandomBattlecryMinion => CardEffect::AddRandomBattlecryMinion,
            CardEffectDe::DamageAndFreeze { damage, target } => {
                CardEffect::DamageAndFreeze { damage, target }
            }
            CardEffectDe::DamageAllEnemyMinionsAndFreeze { damage } => {
                CardEffect::DamageAllEnemyMinionsAndFreeze { damage }
            }
            CardEffectDe::AddRandomOutcastCardNextCheaper => {
                CardEffect::AddRandomOutcastCardNextCheaper
            }
            CardEffectDe::DestroyAndGainStats {
                attack,
                health,
                target,
            } => CardEffect::DestroyAndGainStats {
                attack,
                health,
                target,
            },
            CardEffectDe::DestroyRandomEnemySecret => CardEffect::DestroyRandomEnemySecret,
            CardEffectDe::DestroyAllEnemySecretsAndGainStats { attack, health } => {
                CardEffect::DestroyAllEnemySecretsAndGainStats { attack, health }
            }
            CardEffectDe::DestroyAllEnemySecretsAndDraw { count } => {
                CardEffect::DestroyAllEnemySecretsAndDraw { count }
            }
            CardEffectDe::AttachAttackDraw { count } => CardEffect::AttachAttackDraw { count },
            CardEffectDe::GainStatsPerHandCard {
                attack,
                health_per_card,
            } => CardEffect::GainStatsPerHandCard {
                attack,
                health_per_card,
            },
            CardEffectDe::GainStatsPerFriendlyMinion {
                attack,
                health_per_minion,
            } => CardEffect::GainStatsPerFriendlyMinion {
                attack,
                health_per_minion,
            },
            CardEffectDe::DealDamageRandomly {
                amount,
                count,
                target,
            } => CardEffect::DealDamageRandomly {
                amount,
                count,
                target,
            },
            CardEffectDe::MortalStrike {
                damage,
                boosted,
                threshold,
            } => CardEffect::MortalStrike {
                damage,
                boosted,
                threshold,
            },
            CardEffectDe::DrawPerDamagedFriendlyCharacter => {
                CardEffect::DrawPerDamagedFriendlyCharacter
            }
            CardEffectDe::GainStatsIfOwnSecret { attack, health } => {
                CardEffect::GainStatsIfOwnSecret { attack, health }
            }
            CardEffectDe::AbsorbDivineShields {
                attack_per_shield,
                health_per_shield,
            } => CardEffect::AbsorbDivineShields {
                attack_per_shield,
                health_per_shield,
            },
            CardEffectDe::RemoveWeaponDurability { amount } => {
                CardEffect::RemoveWeaponDurability { amount }
            }
            CardEffectDe::GainAttackEqualToWeapon => CardEffect::GainAttackEqualToWeapon,
            CardEffectDe::EnemySpellsCostZero => CardEffect::EnemySpellsCostZero,
            CardEffectDe::GiveOpponentManaCrystal { count } => {
                CardEffect::GiveOpponentManaCrystal { count }
            }
            CardEffectDe::SetPlayedMinionHealth { health } => {
                CardEffect::SetPlayedMinionHealth { health }
            }
            CardEffectDe::SilenceAllEnemyMinionsAndDraw { count } => {
                CardEffect::SilenceAllEnemyMinionsAndDraw { count }
            }
            CardEffectDe::SwapAttackAndHealth { target } => {
                CardEffect::SwapAttackAndHealth { target }
            }
            CardEffectDe::FreezeAdjacent => CardEffect::FreezeAdjacent,
            CardEffectDe::GrantAdjacentTaunt => CardEffect::GrantAdjacentTaunt,
            CardEffectDe::GrantAdjacentSpellDamage { amount } => {
                CardEffect::GrantAdjacentSpellDamage { amount }
            }
            CardEffectDe::FullHealAndTaunt { target } => CardEffect::FullHealAndTaunt { target },
            CardEffectDe::ChanceDraw { percent } => CardEffect::ChanceDraw { percent },
            CardEffectDe::GainStatsThisTurn {
                attack,
                health,
                target,
            } => CardEffect::GainStatsThisTurn {
                attack,
                health,
                target,
            },
            CardEffectDe::GrantDivineShieldAllFriendly => CardEffect::GrantDivineShieldAllFriendly,
            CardEffectDe::GrantDivineShield { target } => CardEffect::GrantDivineShield { target },
            CardEffectDe::YseraAwakens { damage } => CardEffect::YseraAwakens { damage },
            CardEffectDe::GainStatsAndTauntAllFriendly { attack, health } => {
                CardEffect::GainStatsAndTauntAllFriendly { attack, health }
            }
            CardEffectDe::DrawAndDamageByCost => CardEffect::DrawAndDamageByCost,
            CardEffectDe::RestoreDamagedFriendly { amount } => {
                CardEffect::RestoreDamagedFriendly { amount }
            }
            CardEffectDe::SwapWithHandMinion => CardEffect::SwapWithHandMinion,
            CardEffectDe::ResurrectDiedMinion => CardEffect::ResurrectDiedMinion,
            CardEffectDe::ImbueHeroPower => CardEffect::ImbueHeroPower,
            CardEffectDe::ImbuedHeroPower { class } => CardEffect::ImbuedHeroPower { class },
            CardEffectDe::UseHeroPower => CardEffect::UseHeroPower,
            CardEffectDe::DrawBeastAndImbue => CardEffect::DrawBeastAndImbue,
            CardEffectDe::RestoreAndDrawAndImbue { amount } => {
                CardEffect::RestoreAndDrawAndImbue { amount }
            }
            CardEffectDe::SummonRandomTwoCostTauntAndImbue => {
                CardEffect::SummonRandomTwoCostTauntAndImbue
            }
            CardEffectDe::ImbueAndReduceHandCost => CardEffect::ImbueAndReduceHandCost,
            CardEffectDe::ImbueAndTriggerHeroPower => CardEffect::ImbueAndTriggerHeroPower,
            CardEffectDe::ImbueAndGetWisp => CardEffect::ImbueAndGetWisp,
            CardEffectDe::ImbueAndDebuffEnemies { attack_reduction } => {
                CardEffect::ImbueAndDebuffEnemies { attack_reduction }
            }
            CardEffectDe::DealDamageIfImbuedTwice { damage } => {
                CardEffect::DealDamageIfImbuedTwice { damage }
            }
            CardEffectDe::DiscoverWildGodIfImbued4 => CardEffect::DiscoverWildGodIfImbued4,
            CardEffectDe::ImbueEveryThirdSpell => CardEffect::ImbueEveryThirdSpell,
            CardEffectDe::SummonRandomDragonOfCost { cost } => {
                CardEffect::SummonRandomDragonOfCost { cost }
            }
            CardEffectDe::ApplyDarkGift { gift } => CardEffect::ApplyDarkGift { gift },
            CardEffectDe::DiscoverWithDarkGift { pool } => {
                CardEffect::DiscoverWithDarkGift { pool }
            }
            CardEffectDe::DiscoverDragonWithDarkGift => CardEffect::DiscoverDragonWithDarkGift,
            CardEffectDe::DiscoverUndeadWithCorpseGift { corpses } => {
                CardEffect::DiscoverUndeadWithCorpseGift { corpses }
            }
            CardEffectDe::DiscoverEnemyDeckMinionCopy { with_gift } => {
                CardEffect::DiscoverEnemyDeckMinionCopy { with_gift }
            }
            CardEffectDe::DiscoverDeckMinionWithDarkGift => {
                CardEffect::DiscoverDeckMinionWithDarkGift
            }
            CardEffectDe::ReduceHandMinionGiftCost => CardEffect::ReduceHandMinionGiftCost,
            CardEffectDe::GainStatsAndGrantDivineShield {
                attack,
                health,
                target,
            } => CardEffect::GainStatsAndGrantDivineShield {
                attack,
                health,
                target,
            },
            CardEffectDe::GainStatsAndGrantLifesteal {
                attack,
                health,
                target,
            } => CardEffect::GainStatsAndGrantLifesteal {
                attack,
                health,
                target,
            },
            CardEffectDe::GrantPoisonousThisTurn => CardEffect::GrantPoisonousThisTurn,
            CardEffectDe::GrantHeroLifestealThisTurn => CardEffect::GrantHeroLifestealThisTurn,
            CardEffectDe::GrantWeaponDeathrattleAllEnemies { damage } => {
                CardEffect::GrantWeaponDeathrattleAllEnemies { damage }
            }
            CardEffectDe::DrawCardByType { count, card_type } => {
                CardEffect::DrawCardByType { count, card_type }
            }
            CardEffectDe::SpendCorpsesDamageMinion { cost, damage } => {
                CardEffect::SpendCorpsesDamageMinion { cost, damage }
            }
            CardEffectDe::DamageAllMinions { damage } => CardEffect::DamageAllMinions { damage },
            CardEffectDe::AddRandomDruidSpell => CardEffect::AddRandomDruidSpell,
            CardEffectDe::AddRandomOtherClassChooseOneCard => {
                CardEffect::AddRandomOtherClassChooseOneCard
            }
            CardEffectDe::AttackTwoRandomEnemyMinionsIfCostLE { cost } => {
                CardEffect::AttackTwoRandomEnemyMinionsIfCostLE { cost }
            }
            CardEffectDe::GainArmorSummonCostTaunt { armor, cost } => {
                CardEffect::GainArmorSummonCostTaunt { armor, cost }
            }
            CardEffectDe::AddRandomCostMinionWithDarkGift { cost } => {
                CardEffect::AddRandomCostMinionWithDarkGift { cost }
            }
            CardEffectDe::BuffTopDeckMinions {
                attack,
                health,
                count,
            } => CardEffect::BuffTopDeckMinions {
                attack,
                health,
                count,
            },
            CardEffectDe::ShuffleAllMinionsIntoDecks => CardEffect::ShuffleAllMinionsIntoDecks,
            CardEffectDe::DrawDeckSpellAndAddRandomSpell => {
                CardEffect::DrawDeckSpellAndAddRandomSpell
            }
            CardEffectDe::SetStatsByFriendlyTarget {
                enemy_attack,
                enemy_health,
                friendly_attack,
                friendly_health,
            } => CardEffect::SetStatsByFriendlyTarget {
                enemy_attack,
                enemy_health,
                friendly_attack,
                friendly_health,
            },
            CardEffectDe::GainAttackEqualSpellCost => CardEffect::GainAttackEqualSpellCost,
            CardEffectDe::DamageLowestHealthEnemyTwice { amount } => {
                CardEffect::DamageLowestHealthEnemyTwice { amount }
            }
            CardEffectDe::DrawAndGainStats { attack, health } => {
                CardEffect::DrawAndGainStats { attack, health }
            }
            CardEffectDe::ShuffleCardIntoDeck { card_id, count } => {
                CardEffect::ShuffleCardIntoDeck {
                    card_id: intern(card_id)?,
                    count,
                }
            }
            CardEffectDe::AmphibianSpiritBuff { attack, health } => {
                CardEffect::AmphibianSpiritBuff { attack, health }
            }
            CardEffectDe::DamageAndSummonWolfIfKilled { damage } => {
                CardEffect::DamageAndSummonWolfIfKilled { damage }
            }
            CardEffectDe::AddRandomSpellCostsLess { reduction } => {
                CardEffect::AddRandomSpellCostsLess { reduction }
            }
            CardEffectDe::SummonTreantCopyingSpell => CardEffect::SummonTreantCopyingSpell,
            CardEffectDe::SummonEggHatchingDragon => CardEffect::SummonEggHatchingDragon,
            CardEffectDe::ResurrectRandomFallenDragon => CardEffect::ResurrectRandomFallenDragon,
            CardEffectDe::EquipSwordIfHoldingDragon => CardEffect::EquipSwordIfHoldingDragon,
            CardEffectDe::DamageAllOtherFriendlyMinions { damage } => {
                CardEffect::DamageAllOtherFriendlyMinions { damage }
            }
            CardEffectDe::DamageMinionWithMoonLifesteal { amount } => {
                CardEffect::DamageMinionWithMoonLifesteal { amount }
            }
            CardEffectDe::SummonTwoRandomCostMinions {
                base_cost,
                upgraded_cost,
            } => CardEffect::SummonTwoRandomCostMinions {
                base_cost,
                upgraded_cost,
            },
            CardEffectDe::DamageIfHoldingSpell5Plus { amount } => {
                CardEffect::DamageIfHoldingSpell5Plus { amount }
            }
            CardEffectDe::SummonCopyIfAttackGE { attack } => {
                CardEffect::SummonCopyIfAttackGE { attack }
            }
            CardEffectDe::RestoreHealthAndPendingSelfDamage {
                heal,
                damage,
                turns,
            } => CardEffect::RestoreHealthAndPendingSelfDamage {
                heal,
                damage,
                turns,
            },
            CardEffectDe::DestroyCrystalGainCrystalsLater { gain, turns } => {
                CardEffect::DestroyCrystalGainCrystalsLater { gain, turns }
            }
            CardEffectDe::DrawMinionCostGE { cost } => CardEffect::DrawMinionCostGE { cost },
            CardEffectDe::GainDeathrattleOfDiedThisTurn => {
                CardEffect::GainDeathrattleOfDiedThisTurn
            }
            CardEffectDe::AddRandomDeckMinionToHand => CardEffect::AddRandomDeckMinionToHand,
            CardEffectDe::EatDeckMinionGainStats => CardEffect::EatDeckMinionGainStats,
            CardEffectDe::DebuffRandomHandMinionBoth { attack_reduction } => {
                CardEffect::DebuffRandomHandMinionBoth { attack_reduction }
            }
            CardEffectDe::SpendAllManaCastRandomSpell => CardEffect::SpendAllManaCastRandomSpell,
            CardEffectDe::CopyLowestCostEnemyHandCard => CardEffect::CopyLowestCostEnemyHandCard,
            CardEffectDe::OpponentDrawsTwoAndCopies => CardEffect::OpponentDrawsTwoAndCopies,
            CardEffectDe::ReturnFriendlyMinionSummonSpider => {
                CardEffect::ReturnFriendlyMinionSummonSpider
            }
            CardEffectDe::ShuffleMatchingEnemyHandCardIntoDeck => {
                CardEffect::ShuffleMatchingEnemyHandCardIntoDeck
            }
            CardEffectDe::DestroyFriendlyMinionGainArmor { armor } => {
                CardEffect::DestroyFriendlyMinionGainArmor { armor }
            }
            CardEffectDe::DrawSpellCostGE { cost } => CardEffect::DrawSpellCostGE { cost },
            CardEffectDe::DrawDragonsReduced { count, reduction } => {
                CardEffect::DrawDragonsReduced { count, reduction }
            }
            CardEffectDe::SummonCopyOfSelf => CardEffect::SummonCopyOfSelf,
            CardEffectDe::DestroyFriendlyWispDraw { count } => {
                CardEffect::DestroyFriendlyWispDraw { count }
            }
            CardEffectDe::DrawAndSummonLeeches { draw } => {
                CardEffect::DrawAndSummonLeeches { draw }
            }
            CardEffectDe::DrawAndSummonDreadseed { draw } => {
                CardEffect::DrawAndSummonDreadseed { draw }
            }
            CardEffectDe::NextHeroPowerCostsZero => CardEffect::NextHeroPowerCostsZero,
            CardEffectDe::SetMurlocSummonBuff => CardEffect::SetMurlocSummonBuff,
            CardEffectDe::SetDealExact2Bonus => CardEffect::SetDealExact2Bonus,
            CardEffectDe::RestoreHealthAndGetDruidSpells { amount, count } => {
                CardEffect::RestoreHealthAndGetDruidSpells { amount, count }
            }
            CardEffectDe::GainManaCrystalBoth { count } => {
                CardEffect::GainManaCrystalBoth { count }
            }
            CardEffectDe::TransformNeutralDeckToDruid => CardEffect::TransformNeutralDeckToDruid,
            CardEffectDe::AddMoonfireAndStarfireWithSpellDamage => {
                CardEffect::AddMoonfireAndStarfireWithSpellDamage
            }
            CardEffectDe::BuffAnotherRandomFriendlyDragon { attack, health } => {
                CardEffect::BuffAnotherRandomFriendlyDragon { attack, health }
            }
            CardEffectDe::ReduceRightmostHandCardCost { reduction } => {
                CardEffect::ReduceRightmostHandCardCost { reduction }
            }
            CardEffectDe::ResurrectDeathrattleMinionCostLE { cost } => {
                CardEffect::ResurrectDeathrattleMinionCostLE { cost }
            }
            CardEffectDe::ResurrectDeathrattleMinionCostGE { cost } => {
                CardEffect::ResurrectDeathrattleMinionCostGE { cost }
            }
            CardEffectDe::GainArmorPerWisp { base } => CardEffect::GainArmorPerWisp { base },
            CardEffectDe::DamageMinionScaledByFallen { base } => {
                CardEffect::DamageMinionScaledByFallen { base }
            }
            CardEffectDe::GrantHeroDivineShield => CardEffect::GrantHeroDivineShield,
            CardEffectDe::RestoreBothHeroes { amount } => CardEffect::RestoreBothHeroes { amount },
            CardEffectDe::AddSelfToDeckBottomCost { cost } => {
                CardEffect::AddSelfToDeckBottomCost { cost }
            }
            CardEffectDe::SummonCopyOfRandomFriendlyDragon => {
                CardEffect::SummonCopyOfRandomFriendlyDragon
            }
            CardEffectDe::GainHealthIfHeroPowerUsed { amount } => {
                CardEffect::GainHealthIfHeroPowerUsed { amount }
            }
            CardEffectDe::AttackRandomEnemyMinionExcess => {
                CardEffect::AttackRandomEnemyMinionExcess
            }
            CardEffectDe::SplashHeroAttackToRandomEnemy => {
                CardEffect::SplashHeroAttackToRandomEnemy
            }
            CardEffectDe::GainDeadMinionAttack => CardEffect::GainDeadMinionAttack,
            CardEffectDe::DrawIfMinionPlayedBefore => CardEffect::DrawIfMinionPlayedBefore,
            CardEffectDe::GrantRandomBonusEffect => CardEffect::GrantRandomBonusEffect,
            CardEffectDe::YseraEmeraldAspect => CardEffect::YseraEmeraldAspect,
            CardEffectDe::ResurrectAllDifferentFriendlyCostGE { cost } => {
                CardEffect::ResurrectAllDifferentFriendlyCostGE { cost }
            }
            CardEffectDe::CastHighestCostSpellFromHand => CardEffect::CastHighestCostSpellFromHand,
            CardEffectDe::IncrementOmenAttack => CardEffect::IncrementOmenAttack,
            CardEffectDe::OmenDeathrattle => CardEffect::OmenDeathrattle,
            CardEffectDe::SplitDamageAmongAllEnemiesIfFallen { amount, threshold } => {
                CardEffect::SplitDamageAmongAllEnemiesIfFallen { amount, threshold }
            }
            CardEffectDe::NextSpellsCastTwice { count } => {
                CardEffect::NextSpellsCastTwice { count }
            }
            CardEffectDe::SummonRandomDragonPerSelfDeath => {
                CardEffect::SummonRandomDragonPerSelfDeath
            }
            CardEffectDe::GainArmorAndSelfAttack { armor, attack } => {
                CardEffect::GainArmorAndSelfAttack { armor, attack }
            }
            CardEffectDe::NextCardCostsZero => CardEffect::NextCardCostsZero,
            CardEffectDe::TransformHandMinionsToRandomDemons => {
                CardEffect::TransformHandMinionsToRandomDemons
            }
            CardEffectDe::DiscoverSpellKeepOrTop => CardEffect::DiscoverSpellKeepOrTop,
            CardEffectDe::DiscardRandomEnemyHandCard => CardEffect::DiscardRandomEnemyHandCard,
            CardEffectDe::FillHandWithEnemyDeckCopies { reduction } => {
                CardEffect::FillHandWithEnemyDeckCopies { reduction }
            }
            CardEffectDe::SummonBeetles { count } => CardEffect::SummonBeetles { count },
            CardEffectDe::UrsocBattlecry => CardEffect::UrsocBattlecry,
            CardEffectDe::UrsocDeathrattle => CardEffect::UrsocDeathrattle,
            CardEffectDe::GainStatsAllOtherFriendlyMinions { attack, health } => {
                CardEffect::GainStatsAllOtherFriendlyMinions { attack, health }
            }
            CardEffectDe::SummonRandomAnimalCompanion => CardEffect::SummonRandomAnimalCompanion,
            CardEffectDe::AddAllDreamCards => CardEffect::AddAllDreamCards,
            CardEffectDe::CardsCostOneThisGame => CardEffect::CardsCostOneThisGame,
            CardEffectDe::GainStatsIfHeroPowerUsed { attack, health } => {
                CardEffect::GainStatsIfHeroPowerUsed { attack, health }
            }
            CardEffectDe::GiveMinionStatsRushIfHeroPowerUsed { attack, health } => {
                CardEffect::GiveMinionStatsRushIfHeroPowerUsed { attack, health }
            }
            CardEffectDe::DrawIfImbuedTwice { count } => CardEffect::DrawIfImbuedTwice { count },
            CardEffectDe::DealDamageToAllEnemyMinions { damage } => {
                CardEffect::DealDamageToAllEnemyMinions { damage }
            }
            CardEffectDe::DiscoverWithDarkGiftCostReduction { reduction } => {
                CardEffect::DiscoverWithDarkGiftCostReduction { reduction }
            }
            CardEffectDe::SummonBroodlingsIfHoldingGift => {
                CardEffect::SummonBroodlingsIfHoldingGift
            }
            CardEffectDe::FelfireBlazeTrigger { damage } => {
                CardEffect::FelfireBlazeTrigger { damage }
            }
            CardEffectDe::BuffFriendlyMinionsDiscardBonus {
                attack,
                health,
                bonus_attack,
                bonus_health,
            } => CardEffect::BuffFriendlyMinionsDiscardBonus {
                attack,
                health,
                bonus_attack,
                bonus_health,
            },
            CardEffectDe::AmirdrassilActivate => CardEffect::AmirdrassilActivate,
            CardEffectDe::InfernoHeraldTrigger { reduction } => {
                CardEffect::InfernoHeraldTrigger { reduction }
            }
            CardEffectDe::BuffMinionReturnIfSpellsCast {
                attack,
                health,
                threshold,
            } => CardEffect::BuffMinionReturnIfSpellsCast {
                attack,
                health,
                threshold,
            },
            CardEffectDe::GainWeaponAttackIfHoldingGift { amount } => {
                CardEffect::GainWeaponAttackIfHoldingGift { amount }
            }
            CardEffectDe::DamageRandomEnemyMinionHoldingCostGE {
                base,
                upgraded,
                threshold,
            } => CardEffect::DamageRandomEnemyMinionHoldingCostGE {
                base,
                upgraded,
                threshold,
            },
            CardEffectDe::DiscoverComboBattlecryStealthWithDarkGift => {
                CardEffect::DiscoverComboBattlecryStealthWithDarkGift
            }
            CardEffectDe::DiscoverDemonWithDarkGiftCopy => {
                CardEffect::DiscoverDemonWithDarkGiftCopy
            }
            CardEffectDe::DiscoverCostCardGainTempMana { cost, mana } => {
                CardEffect::DiscoverCostCardGainTempMana { cost, mana }
            }
            CardEffectDe::DamageAndDiscoverWarriorWithGift { damage } => {
                CardEffect::DamageAndDiscoverWarriorWithGift { damage }
            }
            CardEffectDe::ReduceHandCostIfAllDistinct { reduction } => {
                CardEffect::ReduceHandCostIfAllDistinct { reduction }
            }
            CardEffectDe::DrawMinionSummonDivineShieldCopy => {
                CardEffect::DrawMinionSummonDivineShieldCopy
            }
            CardEffectDe::VolcorossBattlecry => CardEffect::VolcorossBattlecry,
            CardEffectDe::DiscoverSpellReduceHandSpells { reduction } => {
                CardEffect::DiscoverSpellReduceHandSpells { reduction }
            }
            CardEffectDe::MagmaHoundSplash => CardEffect::MagmaHoundSplash,
            CardEffectDe::DamageMinionOwnerDraws { damage } => {
                CardEffect::DamageMinionOwnerDraws { damage }
            }
            CardEffectDe::DeathrattleDamageAllEnemiesTurnScaled { base, boosted } => {
                CardEffect::DeathrattleDamageAllEnemiesTurnScaled { base, boosted }
            }
            CardEffectDe::DealDamageSplitAmongAllEnemies { amount } => {
                CardEffect::DealDamageSplitAmongAllEnemies { amount }
            }
            CardEffectDe::CopyLowestCostBeastInHand => CardEffect::CopyLowestCostBeastInHand,
            CardEffectDe::GainDivineShieldLifestealIfHoldingSpellGE { cost } => {
                CardEffect::GainDivineShieldLifestealIfHoldingSpellGE { cost }
            }
            CardEffectDe::GainHeroAttackArmorIfHoldingGift { attack, armor } => {
                CardEffect::GainHeroAttackArmorIfHoldingGift { attack, armor }
            }
            CardEffectDe::DamageAndDiscardSpellMore { base, bonus } => {
                CardEffect::DamageAndDiscardSpellMore { base, bonus }
            }
            CardEffectDe::BuffAllHandMinions { attack, health } => {
                CardEffect::BuffAllHandMinions { attack, health }
            }
            CardEffectDe::GainRush { target } => CardEffect::GainRush { target },
            CardEffectDe::GainImmuneThisTurn { target } => {
                CardEffect::GainImmuneThisTurn { target }
            }
            CardEffectDe::NextMurlocCostsLess { amount } => {
                CardEffect::NextMurlocCostsLess { amount }
            }
            CardEffectDe::GiveNextMurlocDivineShield => CardEffect::GiveNextMurlocDivineShield,
            CardEffectDe::SetNextKindredTwice => CardEffect::SetNextKindredTwice,
            CardEffectDe::DrawKindredAndActivator => CardEffect::DrawKindredAndActivator,
            CardEffectDe::DrawSpellGiveSpellDamage { amount } => {
                CardEffect::DrawSpellGiveSpellDamage { amount }
            }
            CardEffectDe::DrawMinionsOfEachCost { up_to } => {
                CardEffect::DrawMinionsOfEachCost { up_to }
            }
            CardEffectDe::DrawDeathrattleMinionCostLE { max_cost } => {
                CardEffect::DrawDeathrattleMinionCostLE { max_cost }
            }
            CardEffectDe::DestroyLowestAttackEnemy => CardEffect::DestroyLowestAttackEnemy,
            CardEffectDe::TriggerFriendlyCinderDeathrattles => {
                CardEffect::TriggerFriendlyCinderDeathrattles
            }
            CardEffectDe::DestroyMinionAndGainItsStats { target } => {
                CardEffect::DestroyMinionAndGainItsStats { target }
            }
            CardEffectDe::DealSelfAttackDamage { target } => {
                CardEffect::DealSelfAttackDamage { target }
            }
            CardEffectDe::SummonRandomMinionCostTaunt { cost } => {
                CardEffect::SummonRandomMinionCostTaunt { cost }
            }
            CardEffectDe::DiscoverPool { pool } => CardEffect::DiscoverPool { pool },
            CardEffectDe::AddRandomCardToHandCount { pool, count } => {
                CardEffect::AddRandomCardToHandCount { pool, count }
            }
            CardEffectDe::AddCardToHandCount { card_id, count } => CardEffect::AddCardToHandCount {
                card_id: intern(card_id)?,
                count,
            },
            CardEffectDe::AddTemporaryRandomMinionsCost { cost, count } => {
                CardEffect::AddTemporaryRandomMinionsCost { cost, count }
            }
            CardEffectDe::AddRandomFelSpellsCostHealth { count } => {
                CardEffect::AddRandomFelSpellsCostHealth { count }
            }
            CardEffectDe::AddRandomHolyAndShadowSpell => CardEffect::AddRandomHolyAndShadowSpell,
            CardEffectDe::AddRandomHolySpellCost1 => CardEffect::AddRandomHolySpellCost1,
            CardEffectDe::CopyRandomHandElementalOrDragon => {
                CardEffect::CopyRandomHandElementalOrDragon
            }
            CardEffectDe::ReduceRandomEnemyHandMinionCost { amount } => {
                CardEffect::ReduceRandomEnemyHandMinionCost { amount }
            }
            CardEffectDe::ReduceRandomBeastHandCost { amount } => {
                CardEffect::ReduceRandomBeastHandCost { amount }
            }
            CardEffectDe::ReduceNonStartingHandCost { amount } => {
                CardEffect::ReduceNonStartingHandCost { amount }
            }
            CardEffectDe::SummonTreantsAttackMinion => CardEffect::SummonTreantsAttackMinion,
            CardEffectDe::DealDamageSummonCinders { amount } => {
                CardEffect::DealDamageSummonCinders { amount }
            }
            CardEffectDe::DealDamageLowestHealthEnemyRepeated { amount, times } => {
                CardEffect::DealDamageLowestHealthEnemyRepeated { amount, times }
            }
            CardEffectDe::DealDamageRandomEnemies { amount, count } => {
                CardEffect::DealDamageRandomEnemies { amount, count }
            }
            CardEffectDe::DrawMinionsDifferentTypesBuff {
                count,
                attack,
                health,
            } => CardEffect::DrawMinionsDifferentTypesBuff {
                count,
                attack,
                health,
            },
            CardEffectDe::DrawMinionBuffArmorIfAttackGE {
                min_attack,
                buff_health,
                armor,
            } => CardEffect::DrawMinionBuffArmorIfAttackGE {
                min_attack,
                buff_health,
                armor,
            },
            CardEffectDe::SetFlockPending => CardEffect::SetFlockPending,
            CardEffectDe::GiveBuffOtherMinionsAttackLE {
                attack,
                health,
                max_attack,
            } => CardEffect::GiveBuffOtherMinionsAttackLE {
                attack,
                health,
                max_attack,
            },
            CardEffectDe::DestroyMinionSummonRandomSameCost { target } => {
                CardEffect::DestroyMinionSummonRandomSameCost { target }
            }
            CardEffectDe::SummonMinionsGrantRandomBonus { card_id, count } => {
                CardEffect::SummonMinionsGrantRandomBonus {
                    card_id: intern(card_id)?,
                    count,
                }
            }
            CardEffectDe::SummonMinionPair { a, b } => CardEffect::SummonMinionPair {
                a: intern(a)?,
                b: intern(b)?,
            },
            CardEffectDe::SummonRandomMinionCostOrEscalated {
                cost,
                escalated_cost,
            } => CardEffect::SummonRandomMinionCostOrEscalated {
                cost,
                escalated_cost,
            },
            CardEffectDe::DealDamageGainArmorIfKilled {
                amount,
                armor,
                target,
            } => CardEffect::DealDamageGainArmorIfKilled {
                amount,
                armor,
                target,
            },
            CardEffectDe::DealDamageAllEnemyMinionsSetMinionsCostMore { damage } => {
                CardEffect::DealDamageAllEnemyMinionsSetMinionsCostMore { damage }
            }
            CardEffectDe::GiveBuffSameType {
                attack,
                health,
                target,
            } => CardEffect::GiveBuffSameType {
                attack,
                health,
                target,
            },
            CardEffectDe::GrantRandomBonusEffects { count, target } => {
                CardEffect::GrantRandomBonusEffects { count, target }
            }
            CardEffectDe::GrantRandomBonusEffectAndDeathrattle => {
                CardEffect::GrantRandomBonusEffectAndDeathrattle
            }
            CardEffectDe::SetLakkariTicks { ticks } => CardEffect::SetLakkariTicks { ticks },
            CardEffectDe::GiveBuffAndSummonDeathrattle {
                attack,
                health,
                summon_cost,
                target,
            } => CardEffect::GiveBuffAndSummonDeathrattle {
                attack,
                health,
                summon_cost,
                target,
            },
            CardEffectDe::DealDamageImprovedByShuffles { amount, target } => {
                CardEffect::DealDamageImprovedByShuffles { amount, target }
            }
            CardEffectDe::DrawCardLinkDeathrattle => CardEffect::DrawCardLinkDeathrattle,
            CardEffectDe::DiscardLinkedDrawnCard => CardEffect::DiscardLinkedDrawnCard,
            CardEffectDe::GainArmorDealDamageEqual { armor, target } => {
                CardEffect::GainArmorDealDamageEqual { armor, target }
            }
            CardEffectDe::DestroyDeckTop { count } => CardEffect::DestroyDeckTop { count },
            CardEffectDe::ResurrectOneOfEachCostGiveReborn { max_cost } => {
                CardEffect::ResurrectOneOfEachCostGiveReborn { max_cost }
            }
            CardEffectDe::DealDamageSetNextBeastDiscount {
                amount,
                discount,
                target,
            } => CardEffect::DealDamageSetNextBeastDiscount {
                amount,
                discount,
                target,
            },
            CardEffectDe::BuffAllBeastsEverywhere { attack, health } => {
                CardEffect::BuffAllBeastsEverywhere { attack, health }
            }
            CardEffectDe::DealDamageSameType { amount, target } => {
                CardEffect::DealDamageSameType { amount, target }
            }
            CardEffectDe::SetNextTemporaryDiscount { amount } => {
                CardEffect::SetNextTemporaryDiscount { amount }
            }
            CardEffectDe::SetEnemyHeroCantBeHealed => CardEffect::SetEnemyHeroCantBeHealed,
            CardEffectDe::DestroyFriendlyMinionAddBones { target } => {
                CardEffect::DestroyFriendlyMinionAddBones { target }
            }
            CardEffectDe::GainManaCrystalsMatchOpponent => {
                CardEffect::GainManaCrystalsMatchOpponent
            }
            CardEffectDe::GiveBuffDifferentTypeMinions { attack, health } => {
                CardEffect::GiveBuffDifferentTypeMinions { attack, health }
            }
            CardEffectDe::DealDamageIfQuestPlayed { amount, target } => {
                CardEffect::DealDamageIfQuestPlayed { amount, target }
            }
            CardEffectDe::SwapHeroPowerToDeal8Random => CardEffect::SwapHeroPowerToDeal8Random,
            CardEffectDe::RecastRandomHolySpellThisTurn => {
                CardEffect::RecastRandomHolySpellThisTurn
            }
            CardEffectDe::CastRandomSpellFromDeckCostLE { max_cost } => {
                CardEffect::CastRandomSpellFromDeckCostLE { max_cost }
            }
            CardEffectDe::SpendArmorDealDamageAllMinions { max_spend } => {
                CardEffect::SpendArmorDealDamageAllMinions { max_spend }
            }
            CardEffectDe::DestroyTopCardDiscoverSameRarity => {
                CardEffect::DestroyTopCardDiscoverSameRarity
            }
            CardEffectDe::GrantKeyword { keyword, target } => {
                CardEffect::GrantKeyword { keyword, target }
            }
            CardEffectDe::GrantDeathrattleSummon {
                card_id,
                count,
                target,
            } => CardEffect::GrantDeathrattleSummon {
                card_id: intern(card_id)?,
                count,
                target,
            },
            CardEffectDe::AddRandomWeaponAnotherClassComboAttack { combo_attack } => {
                CardEffect::AddRandomWeaponAnotherClassComboAttack { combo_attack }
            }
            CardEffectDe::Drain { amount } => CardEffect::Drain { amount },
            CardEffectDe::GainStatsEqualFireSpellCost => CardEffect::GainStatsEqualFireSpellCost,
            CardEffectDe::DealDamageAndSummon { amount, card_id } => {
                CardEffect::DealDamageAndSummon {
                    amount,
                    card_id: intern(card_id)?,
                }
            }
            CardEffectDe::DiscoverDeckCard => CardEffect::DiscoverDeckCard,
            CardEffectDe::DiscoverEnemyDeckTop => CardEffect::DiscoverEnemyDeckTop,
            CardEffectDe::SummonRandomFelBeast => CardEffect::SummonRandomFelBeast,
            CardEffectDe::AddRandomBeastCostLess { amount } => {
                CardEffect::AddRandomBeastCostLess { amount }
            }
            CardEffectDe::TriggerFriendlyDeadDeathrattles { count } => {
                CardEffect::TriggerFriendlyDeadDeathrattles { count }
            }
            CardEffectDe::EshoDeckCheckBuffEverywhere { attack, health } => {
                CardEffect::EshoDeckCheckBuffEverywhere { attack, health }
            }
            CardEffectDe::SetStatsAllEnemyMinions { attack, health } => {
                CardEffect::SetStatsAllEnemyMinions { attack, health }
            }
            CardEffectDe::SummonDamagedCopiesRush => CardEffect::SummonDamagedCopiesRush,
            CardEffectDe::SummonTwoDeathrattleMinionsAndFight => {
                CardEffect::SummonTwoDeathrattleMinionsAndFight
            }
            CardEffectDe::LohMinionsCost5 => CardEffect::LohMinionsCost5,
            CardEffectDe::EliseCraftLocation => CardEffect::EliseCraftLocation,
            CardEffectDe::NiriOfTheCrater => CardEffect::NiriOfTheCrater,
            CardEffectDe::SetEventSubjectHealthToSource => {
                CardEffect::SetEventSubjectHealthToSource
            }
            CardEffectDe::DealDamageToLeftRightEnemyMinions { amount } => {
                CardEffect::DealDamageToLeftRightEnemyMinions { amount }
            }
            CardEffectDe::GiveOtherFriendlyMinionsRush => CardEffect::GiveOtherFriendlyMinionsRush,
            CardEffectDe::DealDamageToTwoAndFreeze { amount } => {
                CardEffect::DealDamageToTwoAndFreeze { amount }
            }
            CardEffectDe::SetStatsAndFillBoardWithCopies { attack, health } => {
                CardEffect::SetStatsAndFillBoardWithCopies { attack, health }
            }
            CardEffectDe::SetStatsAndGrantCharge {
                attack,
                health,
                target,
            } => CardEffect::SetStatsAndGrantCharge {
                attack,
                health,
                target,
            },
            CardEffectDe::SetStatsGrantLifestealForceAttack {
                attack,
                health,
                target,
            } => CardEffect::SetStatsGrantLifestealForceAttack {
                attack,
                health,
                target,
            },
            CardEffectDe::SetStatsAttachDamageAllDeathrattle {
                attack,
                health,
                target,
            } => CardEffect::SetStatsAttachDamageAllDeathrattle {
                attack,
                health,
                target,
            },
            CardEffectDe::SetStatsGrantStealthAndDraw {
                attack,
                health,
                draw,
                target,
            } => CardEffect::SetStatsGrantStealthAndDraw {
                attack,
                health,
                draw,
                target,
            },
            CardEffectDe::SummonMinionAndBuffFriendlyMinions {
                card_id,
                attack,
                health,
            } => CardEffect::SummonMinionAndBuffFriendlyMinions {
                card_id: intern(card_id)?,
                attack,
                health,
            },
            CardEffectDe::SummonRandomDeckBeastGiveLifesteal => {
                CardEffect::SummonRandomDeckBeastGiveLifesteal
            }
            CardEffectDe::SummonRaptorsOutcast { count } => {
                CardEffect::SummonRaptorsOutcast { count }
            }
            CardEffectDe::ReduceAdjacentHandCardCost { amount } => {
                CardEffect::ReduceAdjacentHandCardCost { amount }
            }
            CardEffectDe::DracorexSplash => CardEffect::DracorexSplash,
            CardEffectDe::SetHatchingPending => CardEffect::SetHatchingPending,
            CardEffectDe::DealDamageAndBuffFriendlyElementals {
                damage,
                attack,
                health,
            } => CardEffect::DealDamageAndBuffFriendlyElementals {
                damage,
                attack,
                health,
            },
            CardEffectDe::ShuffleLeftmostHandCardIntoDeck => {
                CardEffect::ShuffleLeftmostHandCardIntoDeck
            }
            CardEffectDe::DrawZeroAttackMinion => CardEffect::DrawZeroAttackMinion,
            CardEffectDe::TransformRandomMinionIntoRandomMinion => {
                CardEffect::TransformRandomMinionIntoRandomMinion
            }
            CardEffectDe::SummonRandomDeathrattleMinionCostGEAndTrigger { min_cost } => {
                CardEffect::SummonRandomDeathrattleMinionCostGEAndTrigger { min_cost }
            }
            CardEffectDe::SpendCorpsesGainReborn { amount } => {
                CardEffect::SpendCorpsesGainReborn { amount }
            }
            CardEffectDe::SoulrestMarkAndBuff => CardEffect::SoulrestMarkAndBuff,
            CardEffectDe::GainStatsAndGrantRush {
                attack,
                health,
                target,
            } => CardEffect::GainStatsAndGrantRush {
                attack,
                health,
                target,
            },
            CardEffectDe::BuffHandAndDeckMinions { attack, health } => {
                CardEffect::BuffHandAndDeckMinions { attack, health }
            }
            CardEffectDe::SummonTwoRandomCostBeastsAttackRandomEnemies { cost } => {
                CardEffect::SummonTwoRandomCostBeastsAttackRandomEnemies { cost }
            }
            CardEffectDe::SummonRandomLegendaryMinionSetStats { attack, health } => {
                CardEffect::SummonRandomLegendaryMinionSetStats { attack, health }
            }
            CardEffectDe::SummonRandomCostMinionSetStats {
                cost,
                attack,
                health,
            } => CardEffect::SummonRandomCostMinionSetStats {
                cost,
                attack,
                health,
            },
            CardEffectDe::AddRandomMaskCombo { reduction } => {
                CardEffect::AddRandomMaskCombo { reduction }
            }
            CardEffectDe::GainStatsOfRandomLegendaryBeast => {
                CardEffect::GainStatsOfRandomLegendaryBeast
            }
            CardEffectDe::SummonRandomLegendaryBeast => CardEffect::SummonRandomLegendaryBeast,
            CardEffectDe::SummonRandomTauntMinionCostGE { min_cost } => {
                CardEffect::SummonRandomTauntMinionCostGE { min_cost }
            }
            CardEffectDe::SummonRandomTauntMinionsOfCosts { a, b, c } => {
                CardEffect::SummonRandomTauntMinionsOfCosts { a, b, c }
            }
            CardEffectDe::AddRandomOneCostMinion => CardEffect::AddRandomOneCostMinion,
            CardEffectDe::AddRandomOneCostSpell => CardEffect::AddRandomOneCostSpell,
            CardEffectDe::AddRandomMultiTribeMinion => CardEffect::AddRandomMultiTribeMinion,
            // M3-W2a (Across the Timeways — the 120 non-legendary TIME cards)
            CardEffectDe::AddRandomMinionCostsLess { reduction } => {
                CardEffect::AddRandomMinionCostsLess { reduction }
            }
            CardEffectDe::AddRandomSpellsFromClass { count } => {
                CardEffect::AddRandomSpellsFromClass { count }
            }
            CardEffectDe::DrawRandomMinionGiveStats { attack, health } => {
                CardEffect::DrawRandomMinionGiveStats { attack, health }
            }
            CardEffectDe::BothPlayersDiscardRandomCard => CardEffect::BothPlayersDiscardRandomCard,
            CardEffectDe::SummonManaWorthRandomMinions { total } => {
                CardEffect::SummonManaWorthRandomMinions { total }
            }
            CardEffectDe::GetHolySpellsRestoreHealthEqualCosts => {
                CardEffect::GetHolySpellsRestoreHealthEqualCosts
            }
            CardEffectDe::CastRandomNatureSpells { count } => {
                CardEffect::CastRandomNatureSpells { count }
            }
            CardEffectDe::BothPlayersEquipRandomWeaponBuffOurs { attack, health } => {
                CardEffect::BothPlayersEquipRandomWeaponBuffOurs { attack, health }
            }
            CardEffectDe::AddRandomRewindCardToHand => CardEffect::AddRandomRewindCardToHand,
            CardEffectDe::CopyRightmostEnemyHandCardOrIncreaseCost => {
                CardEffect::CopyRightmostEnemyHandCardOrIncreaseCost
            }
            CardEffectDe::SummonTwoRandomLegendaryMinions => {
                CardEffect::SummonTwoRandomLegendaryMinions
            }
            CardEffectDe::DiscoverEnemyHandCardCopy => CardEffect::DiscoverEnemyHandCardCopy,
            CardEffectDe::SummonTauntAndIfHoldingDragonAgain { card_id } => {
                CardEffect::SummonTauntAndIfHoldingDragonAgain {
                    card_id: intern(card_id)?,
                }
            }
            CardEffectDe::RestoreAndGrantHeroDivineShield { amount } => {
                CardEffect::RestoreAndGrantHeroDivineShield { amount }
            }
            CardEffectDe::DiscoverPaladinMechPastGiveStats { attack, health } => {
                CardEffect::DiscoverPaladinMechPastGiveStats { attack, health }
            }
            CardEffectDe::DealDamageAllEnemiesIfControllingAura { amount } => {
                CardEffect::DealDamageAllEnemiesIfControllingAura { amount }
            }
            CardEffectDe::GiveHeroImmuneThisTurn => CardEffect::GiveHeroImmuneThisTurn,
            CardEffectDe::DrawBottomCards { count } => CardEffect::DrawBottomCards { count },
            CardEffectDe::BuffAllFriendlyMinionsShuffleShreds { attack, health } => {
                CardEffect::BuffAllFriendlyMinionsShuffleShreds { attack, health }
            }
            CardEffectDe::DealDamageSplitAmongAllEnemiesShuffleShreds { amount } => {
                CardEffect::DealDamageSplitAmongAllEnemiesShuffleShreds { amount }
            }
            CardEffectDe::CastShredFromDeckGainStats { attack, health } => {
                CardEffect::CastShredFromDeckGainStats { attack, health }
            }
            CardEffectDe::CastShredFromDeckSummonCopy => CardEffect::CastShredFromDeckSummonCopy,
            CardEffectDe::CopyRandomHandMinion => CardEffect::CopyRandomHandMinion,
            CardEffectDe::DrawCardsOfDifferentCosts { count } => {
                CardEffect::DrawCardsOfDifferentCosts { count }
            }
            CardEffectDe::DrawMinionAndBuffHandMinionsHealth { health } => {
                CardEffect::DrawMinionAndBuffHandMinionsHealth { health }
            }
            CardEffectDe::SetStatsAndCantAttackHeroesThisTurn { attack, health } => {
                CardEffect::SetStatsAndCantAttackHeroesThisTurn { attack, health }
            }
            CardEffectDe::GainStatsPerTurnTaken { attack, health } => {
                CardEffect::GainStatsPerTurnTaken { attack, health }
            }
            CardEffectDe::TransformSelfToRandomMinionOfCost { cost } => {
                CardEffect::TransformSelfToRandomMinionOfCost { cost }
            }
            CardEffectDe::SwapStatsIfSurvivesDamage => CardEffect::SwapStatsIfSurvivesDamage,
            CardEffectDe::GiveCoin => CardEffect::GiveCoin,
            CardEffectDe::TransformSelfIfSurvivesDamageToRandomCost { cost } => {
                CardEffect::TransformSelfIfSurvivesDamageToRandomCost { cost }
            }
            CardEffectDe::ResetBothHandsCosts => CardEffect::ResetBothHandsCosts,
            CardEffectDe::SummonRandomMinionOfCostDormant { cost, turns } => {
                CardEffect::SummonRandomMinionOfCostDormant { cost, turns }
            }
            CardEffectDe::SummonRandomDragonCostGE { min_cost } => {
                CardEffect::SummonRandomDragonCostGE { min_cost }
            }
            CardEffectDe::ReverseDeckOrder => CardEffect::ReverseDeckOrder,
            CardEffectDe::GainTauntAndDivineShieldIfHoldingDragon => {
                CardEffect::GainTauntAndDivineShieldIfHoldingDragon
            }
            CardEffectDe::AddRandomCostMinionMarkedTurnDiscount { cost } => {
                CardEffect::AddRandomCostMinionMarkedTurnDiscount { cost }
            }
            CardEffectDe::DealDamageFriendlyMinionToRandomEnemy { damage, amount } => {
                CardEffect::DealDamageFriendlyMinionToRandomEnemy { damage, amount }
            }
            CardEffectDe::GainStatsAndDrawIfNatureSpellCast { attack, health } => {
                CardEffect::GainStatsAndDrawIfNatureSpellCast { attack, health }
            }
            CardEffectDe::DamageAllMinionsAndAddCardToHand { amount, card_id } => {
                CardEffect::DamageAllMinionsAndAddCardToHand {
                    amount,
                    card_id: intern(card_id)?,
                }
            }
            CardEffectDe::DamageAndDrawTwoIfSurvives { damage, target } => {
                CardEffect::DamageAndDrawTwoIfSurvives { damage, target }
            }
            CardEffectDe::DamageMinionGiveHeroAttack { damage, attack } => {
                CardEffect::DamageMinionGiveHeroAttack { damage, attack }
            }
            CardEffectDe::DealDamageEnemyMinionEqualToSourceHealth => {
                CardEffect::DealDamageEnemyMinionEqualToSourceHealth
            }
            CardEffectDe::SetHandMinionStatsToHigher => CardEffect::SetHandMinionStatsToHigher,
            CardEffectDe::RestoreHealthEqualToSourceHealth => {
                CardEffect::RestoreHealthEqualToSourceHealth
            }
            CardEffectDe::DiscoverDeckAndEnemyHandCardCopy => {
                CardEffect::DiscoverDeckAndEnemyHandCardCopy
            }
            CardEffectDe::SilenceAndDestroyRandomEnemyMinion => {
                CardEffect::SilenceAndDestroyRandomEnemyMinion
            }
            CardEffectDe::SummonShadowAttacksRandomEnemy { card_id } => {
                CardEffect::SummonShadowAttacksRandomEnemy {
                    card_id: intern(card_id)?,
                }
            }
            CardEffectDe::SummonTwoDemonsAttackLowestHealthIfDeckNoMinions => {
                CardEffect::SummonTwoDemonsAttackLowestHealthIfDeckNoMinions
            }
            CardEffectDe::GrantDivineShieldAndBuffHandMinionsHealth { health } => {
                CardEffect::GrantDivineShieldAndBuffHandMinionsHealth { health }
            }
            CardEffectDe::DiscoverMinionReduceHandCostsIfDeckNoMinions { reduction } => {
                CardEffect::DiscoverMinionReduceHandCostsIfDeckNoMinions { reduction }
            }
            CardEffectDe::GainHeroAttackAndBuffHandMinionsIfDeckNoMinions { attack } => {
                CardEffect::GainHeroAttackAndBuffHandMinionsIfDeckNoMinions { attack }
            }
            CardEffectDe::PreciseShot {
                amount,
                center_amount,
            } => CardEffect::PreciseShot {
                amount,
                center_amount,
            },
            CardEffectDe::DrawUntilHandSize { size } => CardEffect::DrawUntilHandSize { size },
            CardEffectDe::SummonRandomCostBeastAttackRandomEnemy { cost } => {
                CardEffect::SummonRandomCostBeastAttackRandomEnemy { cost }
            }
            CardEffectDe::SummonMinionsGrantTwoRandomBonus { card_id, count } => {
                CardEffect::SummonMinionsGrantTwoRandomBonus {
                    card_id: intern(card_id)?,
                    count,
                }
            }
            CardEffectDe::AddRandomLegendaryMinionCostReduced { reduction } => {
                CardEffect::AddRandomLegendaryMinionCostReduced { reduction }
            }
            CardEffectDe::DealDamageEnemyMinionIfHeroHealthChanged { amount } => {
                CardEffect::DealDamageEnemyMinionIfHeroHealthChanged { amount }
            }
            CardEffectDe::FillHandWithRandomUndeadCostHealth => {
                CardEffect::FillHandWithRandomUndeadCostHealth
            }
            CardEffectDe::SummonHighestCostFallenUndead => {
                CardEffect::SummonHighestCostFallenUndead
            }
            CardEffectDe::SetChronologicalAura { ticks } => {
                CardEffect::SetChronologicalAura { ticks }
            }
            CardEffectDe::DiscoverDeckCardOthersBottom => CardEffect::DiscoverDeckCardOthersBottom,
            CardEffectDe::DamageAndGainArmorIfMinionPlayedWhileHeld { damage, armor } => {
                CardEffect::DamageAndGainArmorIfMinionPlayedWhileHeld { damage, armor }
            }
            CardEffectDe::GainStatsAndSummonCopyIfHeroHealthLE {
                attack,
                health,
                threshold,
            } => CardEffect::GainStatsAndSummonCopyIfHeroHealthLE {
                attack,
                health,
                threshold,
            },
            CardEffectDe::GetPupilAndDiscoverSpellCostGE {
                pupil_card_id,
                min_cost,
            } => CardEffect::GetPupilAndDiscoverSpellCostGE {
                pupil_card_id: intern(pupil_card_id)?,
                min_cost,
            },
            CardEffectDe::ReplaceHandAndDeckWithRandomChooseOne => {
                CardEffect::ReplaceHandAndDeckWithRandomChooseOne
            }
            CardEffectDe::SummonTwoRandomCostMinionsWithAttack { cost, bonus } => {
                CardEffect::SummonTwoRandomCostMinionsWithAttack { cost, bonus }
            }
            CardEffectDe::DestroyMinionAndSummonRandomCost { cost } => {
                CardEffect::DestroyMinionAndSummonRandomCost { cost }
            }
            CardEffectDe::NextTurnEnemyCardsCostMore { amount } => {
                CardEffect::NextTurnEnemyCardsCostMore { amount }
            }
            CardEffectDe::AddRandomBeastsToBottomDeckWithStats {
                count,
                attack,
                health,
            } => CardEffect::AddRandomBeastsToBottomDeckWithStats {
                count,
                attack,
                health,
            },
            CardEffectDe::DamageAndDrawMinionIfHoldingCostGE { damage, cost } => {
                CardEffect::DamageAndDrawMinionIfHoldingCostGE { damage, cost }
            }
            CardEffectDe::DrawTwoReduceRandomCost { reduction } => {
                CardEffect::DrawTwoReduceRandomCost { reduction }
            }
            CardEffectDe::DealDamagePrimaryAndSplash { primary, splash } => {
                CardEffect::DealDamagePrimaryAndSplash { primary, splash }
            }
            CardEffectDe::DiscoverArcaneSpellsReduced { reduction } => {
                CardEffect::DiscoverArcaneSpellsReduced { reduction }
            }
            CardEffectDe::DealDamageAndDrawExcess { amount } => {
                CardEffect::DealDamageAndDrawExcess { amount }
            }
            CardEffectDe::SummonPairScrambleStats {
                first_cost,
                second_cost,
            } => CardEffect::SummonPairScrambleStats {
                first_cost,
                second_cost,
            },
            CardEffectDe::LookAtSecretsGiveRandom => CardEffect::LookAtSecretsGiveRandom,
            CardEffectDe::SummonRandomDeckMinionAndTigerForOpponent { card_id } => {
                CardEffect::SummonRandomDeckMinionAndTigerForOpponent {
                    card_id: intern(card_id)?,
                }
            }
            CardEffectDe::GainStatsPerDamagedMinion { attack, health } => {
                CardEffect::GainStatsPerDamagedMinion { attack, health }
            }
            CardEffectDe::FillEnemyBoardWithRandomCost1Minions => {
                CardEffect::FillEnemyBoardWithRandomCost1Minions
            }
            CardEffectDe::GainArmorAndSummonTwoBeastsForOpponent { armor, card_id } => {
                CardEffect::GainArmorAndSummonTwoBeastsForOpponent {
                    armor,
                    card_id: intern(card_id)?,
                }
            }
            CardEffectDe::ImprisonEnemyMinion => CardEffect::ImprisonEnemyMinion,
            CardEffectDe::AwakenImprisonedMinion => CardEffect::AwakenImprisonedMinion,
            CardEffectDe::GuessEnemyHandGainHealth { health } => {
                CardEffect::GuessEnemyHandGainHealth { health }
            }
            CardEffectDe::TransformHandSelfToRandomEnemyHandMinion => {
                CardEffect::TransformHandSelfToRandomEnemyHandMinion
            }
            CardEffectDe::ResurrectDiedMinionFull => CardEffect::ResurrectDiedMinionFull,
            CardEffectDe::SummonRandomMinionFromDeck => CardEffect::SummonRandomMinionFromDeck,
            CardEffectDe::SylvanasDealToAllEnemiesRepeated { damage } => {
                CardEffect::SylvanasDealToAllEnemiesRepeated { damage }
            }
            CardEffectDe::ChronogorDrawsHighestLowest => CardEffect::ChronogorDrawsHighestLowest,
            CardEffectDe::DiscardHandAndAddInfiniteBanana => {
                CardEffect::DiscardHandAndAddInfiniteBanana
            }
            CardEffectDe::MurozondPrepareInfiniteAttack => {
                CardEffect::MurozondPrepareInfiniteAttack
            }
            CardEffectDe::AddCopiesOfLastPlayedCards { count } => {
                CardEffect::AddCopiesOfLastPlayedCards { count }
            }
            CardEffectDe::SummonLocationForPlayer { card_id } => {
                CardEffect::SummonLocationForPlayer {
                    card_id: intern(card_id)?,
                }
            }
            CardEffectDe::SummonCopyOfFriendlyMinion => CardEffect::SummonCopyOfFriendlyMinion,
            CardEffectDe::FillHandWithRandomTemporarySpells => {
                CardEffect::FillHandWithRandomTemporarySpells
            }
            CardEffectDe::TakeControlEnemyMinionHealthLE => {
                CardEffect::TakeControlEnemyMinionHealthLE
            }
            CardEffectDe::DiscoverDemonGE5AndSetNextDemonCostOne => {
                CardEffect::DiscoverDemonGE5AndSetNextDemonCostOne
            }
            CardEffectDe::SpendCorpsesRestoreHeroHealth { max } => {
                CardEffect::SpendCorpsesRestoreHeroHealth { max }
            }
            CardEffectDe::DrawOrResurrectBwonsamdiAndGrantBoon { keyword } => {
                CardEffect::DrawOrResurrectBwonsamdiAndGrantBoon { keyword }
            }
            CardEffectDe::SummonRandomCostMinion { cost } => {
                CardEffect::SummonRandomCostMinion { cost }
            }
            CardEffectDe::SetDeckBottomCostsOne { count } => {
                CardEffect::SetDeckBottomCostsOne { count }
            }
            CardEffectDe::ReplaceHandAndSwapBackAtTurnEnd => {
                CardEffect::ReplaceHandAndSwapBackAtTurnEnd
            }
            CardEffectDe::RestoreHandSnapshot => CardEffect::RestoreHandSnapshot,
            CardEffectDe::SummonChestForOpponent => CardEffect::SummonChestForOpponent,
            CardEffectDe::FillOpponentHandWithCoins => CardEffect::FillOpponentHandWithCoins,
            CardEffectDe::DestroyAllMinionsOpponentPlayedLastTurn => {
                CardEffect::DestroyAllMinionsOpponentPlayedLastTurn
            }
            CardEffectDe::SummonBloodFighterFromHandBuffAndAttack => {
                CardEffect::SummonBloodFighterFromHandBuffAndAttack
            }
            CardEffectDe::GetThreeRandomSpellsFromPastTracked => {
                CardEffect::GetThreeRandomSpellsFromPastTracked
            }
            CardEffectDe::DestroyHeldKingLlaneAndHalveEnemyHealth => {
                CardEffect::DestroyHeldKingLlaneAndHalveEnemyHealth
            }
            CardEffectDe::SilenceAndDestroyAllOtherMinions => {
                CardEffect::SilenceAndDestroyAllOtherMinions
            }
            // M3-W3 — the Across the Timeways closing wave (The End of
            // Time miniset).
            CardEffectDe::DealDamageAndImbue { amount, target } => {
                CardEffect::DealDamageAndImbue { amount, target }
            }
            CardEffectDe::DrawUndeadAndImbueTwice => CardEffect::DrawUndeadAndImbueTwice,
            CardEffectDe::EquipDaggerOrBuffWeapon => CardEffect::EquipDaggerOrBuffWeapon,
            CardEffectDe::BygoneEchoesSummon => CardEffect::BygoneEchoesSummon,
            CardEffectDe::ChronikarHeroAttackBuff => CardEffect::ChronikarHeroAttackBuff,
            CardEffectDe::ChronikarRebuff => CardEffect::ChronikarRebuff,
            CardEffectDe::PressTheAdvantage => CardEffect::PressTheAdvantage,
            CardEffectDe::RefreshManaCrystals { amount } => {
                CardEffect::RefreshManaCrystals { amount }
            }
            CardEffectDe::SummonTwoTreantsScaling => CardEffect::SummonTwoTreantsScaling,
            CardEffectDe::SetAllOtherMinionsAttack { attack } => {
                CardEffect::SetAllOtherMinionsAttack { attack }
            }
            CardEffectDe::SetAllOtherMinionsHealth { health } => {
                CardEffect::SetAllOtherMinionsHealth { health }
            }
            CardEffectDe::ArmAccelerationAura => CardEffect::ArmAccelerationAura,
            CardEffectDe::SetWeaponAttackInfinityThisTurn => {
                CardEffect::SetWeaponAttackInfinityThisTurn
            }
            CardEffectDe::DamageAndBuffFriendlyIfKilled {
                amount,
                attack,
                health,
            } => CardEffect::DamageAndBuffFriendlyIfKilled {
                amount,
                attack,
                health,
            },
            CardEffectDe::AddRandomDeathrattleMinionCostsLess => {
                CardEffect::AddRandomDeathrattleMinionCostsLess
            }
            CardEffectDe::DiscardHighestCostCard => CardEffect::DiscardHighestCostCard,
            CardEffectDe::DrawUntilHandFull => CardEffect::DrawUntilHandFull,
            CardEffectDe::EmptyOpponentHand => CardEffect::EmptyOpponentHand,
            CardEffectDe::SetRandomHandCardCostInfinity => {
                CardEffect::SetRandomHandCardCostInfinity
            }
            CardEffectDe::RestoreInfinityHandCardCost => CardEffect::RestoreInfinityHandCardCost,
            CardEffectDe::GainStatsIfHeroDamagedThisTurn { attack, health } => {
                CardEffect::GainStatsIfHeroDamagedThisTurn { attack, health }
            }
            CardEffectDe::DamageMinionDrawIfSurvivesSummonIfDies { amount } => {
                CardEffect::DamageMinionDrawIfSurvivesSummonIfDies { amount }
            }
            CardEffectDe::BuffHandMinionsAndWeapons { attack } => {
                CardEffect::BuffHandMinionsAndWeapons { attack }
            }
            CardEffectDe::FreezeMinionAndNeighborsDestroyDamaged => {
                CardEffect::FreezeMinionAndNeighborsDestroyDamaged
            }
            CardEffectDe::InfiniteDamageToHighestHealthEnemyMinion => {
                CardEffect::InfiniteDamageToHighestHealthEnemyMinion
            }
            CardEffectDe::DamageMinionEternalFirebolt { amount } => {
                CardEffect::DamageMinionEternalFirebolt { amount }
            }
            CardEffectDe::DestroyAllMinionsWith4OrLessAttack => {
                CardEffect::DestroyAllMinionsWith4OrLessAttack
            }
            CardEffectDe::AddRandomShadowSpell => CardEffect::AddRandomShadowSpell,
            CardEffectDe::OverloadForAndGainImmuneWindfury { overload } => {
                CardEffect::OverloadForAndGainImmuneWindfury { overload }
            }
            CardEffectDe::DestroyRandomEnemyMinionLocationWeapon => {
                CardEffect::DestroyRandomEnemyMinionLocationWeapon
            }
            CardEffectDe::DestroyTopFiveEnemyDeckIfOwnEmpty => {
                CardEffect::DestroyTopFiveEnemyDeckIfOwnEmpty
            }
            CardEffectDe::FillBoardRandomDragonsHealHeroSkipNextTurn => {
                CardEffect::FillBoardRandomDragonsHealHeroSkipNextTurn
            }
            CardEffectDe::GetRandomOtherClassMinionCostsLess { reduction } => {
                CardEffect::GetRandomOtherClassMinionCostsLess { reduction }
            }
            CardEffectDe::BuffFirstUndeadPlayedEachTurn { attack } => {
                CardEffect::BuffFirstUndeadPlayedEachTurn { attack }
            }
            CardEffectDe::GainStatsAndCopyToColossalMain { attack, health } => {
                CardEffect::GainStatsAndCopyToColossalMain { attack, health }
            }
            CardEffectDe::RemoveKeywordFromColossalMain { keyword } => {
                CardEffect::RemoveKeywordFromColossalMain { keyword }
            }
            CardEffectDe::AddRandomCostMinionCostsHealth { cost } => {
                CardEffect::AddRandomCostMinionCostsHealth { cost }
            }
            CardEffectDe::ColossalArmDestroyRight { attack, health } => {
                CardEffect::ColossalArmDestroyRight { attack, health }
            }
            CardEffectDe::AddRandomFireSpellCostsLess { reduction } => {
                CardEffect::AddRandomFireSpellCostsLess { reduction }
            }
            CardEffectDe::TriggerFriendlyDeathrattles => CardEffect::TriggerFriendlyDeathrattles,
            CardEffectDe::AddRandomMinionsCostEqualAttack { count } => {
                CardEffect::AddRandomMinionsCostEqualAttack { count }
            }
            CardEffectDe::GiveRandomFriendlyMinionAttack { attack } => {
                CardEffect::GiveRandomFriendlyMinionAttack { attack }
            }
            CardEffectDe::SummonMinionsAndGrantDeathrattleAll { card_id, count } => {
                CardEffect::SummonMinionsAndGrantDeathrattleAll {
                    card_id: intern(card_id)?,
                    count,
                }
            }
            CardEffectDe::GainStatsElusiveAndSummonCopy { attack, health } => {
                CardEffect::GainStatsElusiveAndSummonCopy { attack, health }
            }
            CardEffectDe::SummonMinionsAndGrantFriendlyAttackDivineShield {
                card_id,
                count,
                attack,
            } => CardEffect::SummonMinionsAndGrantFriendlyAttackDivineShield {
                card_id: intern(card_id)?,
                count,
                attack,
            },
            CardEffectDe::DealDamageAndDamageAllEnemies { amount, aoe } => {
                CardEffect::DealDamageAndDamageAllEnemies { amount, aoe }
            }
            CardEffectDe::DrawMinionsAndBuffHandMinions {
                count,
                attack,
                health,
            } => CardEffect::DrawMinionsAndBuffHandMinions {
                count,
                attack,
                health,
            },
            CardEffectDe::AddRandomShatterCardToHand => CardEffect::AddRandomShatterCardToHand,
            CardEffectDe::ReplaceHero { card_id } => CardEffect::ReplaceHero {
                card_id: intern(card_id)?,
            },
            CardEffectDe::ChooseCataclysms => CardEffect::ChooseCataclysms,
            CardEffectDe::ChooseHandCard => CardEffect::ChooseHandCard,
            CardEffectDe::RefreshManaIfHoldingDragon { amount } => {
                CardEffect::RefreshManaIfHoldingDragon { amount }
            }
            CardEffectDe::GainTempManaOrPermanentIfSpent { threshold } => {
                CardEffect::GainTempManaOrPermanentIfSpent { threshold }
            }
            CardEffectDe::GetOrSummonTauntWhelpsIfSpent { threshold } => {
                CardEffect::GetOrSummonTauntWhelpsIfSpent { threshold }
            }
            CardEffectDe::SummonGolemsSpendAllMana => CardEffect::SummonGolemsSpendAllMana,
            CardEffectDe::ShuffleRandomMinionsCostGE8DoubleStats => {
                CardEffect::ShuffleRandomMinionsCostGE8DoubleStats
            }
            CardEffectDe::GainStatsPerFriendlyMinionTargeted => {
                CardEffect::GainStatsPerFriendlyMinionTargeted
            }
            CardEffectDe::FillHandWithRandomDragons { threshold } => {
                CardEffect::FillHandWithRandomDragons { threshold }
            }
            CardEffectDe::SetAttackEqualToSource => CardEffect::SetAttackEqualToSource,
            CardEffectDe::NextMurlocCostsHealth => CardEffect::NextMurlocCostsHealth,
            CardEffectDe::TransformRandomEnemyMinionToSelf => {
                CardEffect::TransformRandomEnemyMinionToSelf
            }
            CardEffectDe::GiveOpponentSabotage => CardEffect::GiveOpponentSabotage,
            CardEffectDe::ReturnEnemyMinionCantPlayNextTurn => {
                CardEffect::ReturnEnemyMinionCantPlayNextTurn
            }
            CardEffectDe::SetHealingBonus { amount } => CardEffect::SetHealingBonus { amount },
            CardEffectDe::SetNextHealDealsDamage => CardEffect::SetNextHealDealsDamage,
            CardEffectDe::FullHealMinionAndDraw => CardEffect::FullHealMinionAndDraw,
            CardEffectDe::DamageMinionHealEnemyHeroIfKilled { amount } => {
                CardEffect::DamageMinionHealEnemyHeroIfKilled { amount }
            }
            CardEffectDe::LifestealSelfDamage { amount } => {
                CardEffect::LifestealSelfDamage { amount }
            }
            CardEffectDe::GainHealthIfFullHealth { amount } => {
                CardEffect::GainHealthIfFullHealth { amount }
            }
            CardEffectDe::SetRemainingHealthAndFullHealDamage { health, damage } => {
                CardEffect::SetRemainingHealthAndFullHealDamage { health, damage }
            }
            CardEffectDe::BuffSpellDamageHandAndDeck => CardEffect::BuffSpellDamageHandAndDeck,
            CardEffectDe::AddDragonBreathScaledByAttack => {
                CardEffect::AddDragonBreathScaledByAttack
            }
            CardEffectDe::DealDamageEqualDragonBreath => CardEffect::DealDamageEqualDragonBreath,
            CardEffectDe::SummonFiveHungryDrakesSpendCorpsesRush => {
                CardEffect::SummonFiveHungryDrakesSpendCorpsesRush
            }
            CardEffectDe::RefreshManaEqualSelfAttack => CardEffect::RefreshManaEqualSelfAttack,
            CardEffectDe::AddNefariansCreation { reduction } => {
                CardEffect::AddNefariansCreation { reduction }
            }
            CardEffectDe::GrantDeathrattleSummonRandomCostMinion { cost } => {
                CardEffect::GrantDeathrattleSummonRandomCostMinion { cost }
            }
            CardEffectDe::TriggerRandomFriendlyEndTurnEffect => {
                CardEffect::TriggerRandomFriendlyEndTurnEffect
            }
            CardEffectDe::GrantDivineShieldOrGainStats { attack, health } => {
                CardEffect::GrantDivineShieldOrGainStats { attack, health }
            }
            CardEffectDe::AddRandomHolySpellCostReduced { reduction } => {
                CardEffect::AddRandomHolySpellCostReduced { reduction }
            }
            CardEffectDe::SummonDragonWithSelfStats => CardEffect::SummonDragonWithSelfStats,
            CardEffectDe::SetEndTurnEffectsTwice { turns } => {
                CardEffect::SetEndTurnEffectsTwice { turns }
            }
            CardEffectDe::DevourTwoEnemyHandCardsAndDormant => {
                CardEffect::DevourTwoEnemyHandCardsAndDormant
            }
            CardEffectDe::ReturnDevouredCards => CardEffect::ReturnDevouredCards,
            CardEffectDe::SummonCopyIfSpellDamageDealtThisTurn => {
                CardEffect::SummonCopyIfSpellDamageDealtThisTurn
            }
            CardEffectDe::DealDamageAndDamageRandomEnemyMinion { amount, secondary } => {
                CardEffect::DealDamageAndDamageRandomEnemyMinion { amount, secondary }
            }
            CardEffectDe::GainAttackFirstSpellDamageThisTurn { attack } => {
                CardEffect::GainAttackFirstSpellDamageThisTurn { attack }
            }
            CardEffectDe::DamageAllMinionsRepeatDescending { amount } => {
                CardEffect::DamageAllMinionsRepeatDescending { amount }
            }
            CardEffectDe::SummonCopyOfDiscardedMinion => CardEffect::SummonCopyOfDiscardedMinion,
            CardEffectDe::TakeControlUntilEndOfTurnCantAttack => {
                CardEffect::TakeControlUntilEndOfTurnCantAttack
            }
            CardEffectDe::DealDamageToTwoScaledByHandTurns { base } => {
                CardEffect::DealDamageToTwoScaledByHandTurns { base }
            }
            CardEffectDe::DamageAllMinionsDrawPerDeath { amount } => {
                CardEffect::DamageAllMinionsDrawPerDeath { amount }
            }
            CardEffectDe::ReopenLocation => CardEffect::ReopenLocation,
            CardEffectDe::SetNextTurnSummon { card_id } => CardEffect::SetNextTurnSummon {
                card_id: intern(card_id)?,
            },
            CardEffectDe::DealDamageLeftRightOutcastAgain { amount } => {
                CardEffect::DealDamageLeftRightOutcastAgain { amount }
            }
            CardEffectDe::DealDamageEqualSelfAttack => CardEffect::DealDamageEqualSelfAttack,
            CardEffectDe::SetDragonsHaveRush => CardEffect::SetDragonsHaveRush,
            CardEffectDe::SetEnemyMinionHealthTo1IfHoldingDragon => {
                CardEffect::SetEnemyMinionHealthTo1IfHoldingDragon
            }
            CardEffectDe::AddRandomDragonCostLE3 => CardEffect::AddRandomDragonCostLE3,
            CardEffectDe::DealDamageOrDamageAllEnemiesIfCopyPlayed { amount } => {
                CardEffect::DealDamageOrDamageAllEnemiesIfCopyPlayed { amount }
            }
            CardEffectDe::ReplayOneCostCardsPlayedThisGame => {
                CardEffect::ReplayOneCostCardsPlayedThisGame
            }
            CardEffectDe::CastAbsorbedSpell => CardEffect::CastAbsorbedSpell,
            CardEffectDe::GrantMegaWindfuryCantAttackHeroes => {
                CardEffect::GrantMegaWindfuryCantAttackHeroes
            }
            CardEffectDe::TransformFriendlyMinionsCost1MoreSummonOriginals => {
                CardEffect::TransformFriendlyMinionsCost1MoreSummonOriginals
            }
            CardEffectDe::SummonRandomThreeTwoOneCostMinions => {
                CardEffect::SummonRandomThreeTwoOneCostMinions
            }
            CardEffectDe::DrawAndReduceCostRepeated { reduction } => {
                CardEffect::DrawAndReduceCostRepeated { reduction }
            }
            CardEffectDe::DamageAllMinionsScaledByBoard { base } => {
                CardEffect::DamageAllMinionsScaledByBoard { base }
            }
            CardEffectDe::DamageAllMinionsAndGainHeroAttack { amount, attack } => {
                CardEffect::DamageAllMinionsAndGainHeroAttack { amount, attack }
            }
            CardEffectDe::DealDamageSplitAmongEnemiesIfFireSpell { amount } => {
                CardEffect::DealDamageSplitAmongEnemiesIfFireSpell { amount }
            }
            CardEffectDe::DamageDamagedMinionReturnIfExcess { amount } => {
                CardEffect::DamageDamagedMinionReturnIfExcess { amount }
            }
            CardEffectDe::SetGeddonDiscoverDraw => CardEffect::SetGeddonDiscoverDraw,
            CardEffectDe::GrantDeathrattleSummonRandomHandMinion => {
                CardEffect::GrantDeathrattleSummonRandomHandMinion
            }
            CardEffectDe::SummonRandomMinionFromHand => CardEffect::SummonRandomMinionFromHand,
            CardEffectDe::UpgradeHeroPowerCost1 => CardEffect::UpgradeHeroPowerCost1,
            CardEffectDe::AddRandomPaladinAura => CardEffect::AddRandomPaladinAura,
            CardEffectDe::StealHealthThreeTimes { amount } => {
                CardEffect::StealHealthThreeTimes { amount }
            }
            CardEffectDe::DestroyDeckCardsCostLE2Both => CardEffect::DestroyDeckCardsCostLE2Both,
            CardEffectDe::UnlockOverloadedCrystals => CardEffect::UnlockOverloadedCrystals,
            CardEffectDe::CastRandomSpellSameCostOtherClass => {
                CardEffect::CastRandomSpellSameCostOtherClass
            }
            CardEffectDe::ReturnHoardCostLess => CardEffect::ReturnHoardCostLess,
            CardEffectDe::DamageMinionReduceHandCostByExcess { amount } => {
                CardEffect::DamageMinionReduceHandCostByExcess { amount }
            }
            CardEffectDe::AddTwoRandomSpellsSameCost { cost } => {
                CardEffect::AddTwoRandomSpellsSameCost { cost }
            }
            CardEffectDe::Split100StatsAmongDeckMinionsIfCostGE100 { threshold } => {
                CardEffect::Split100StatsAmongDeckMinionsIfCostGE100 { threshold }
            }
            CardEffectDe::DestroyHighestHealthEnemyMinion => {
                CardEffect::DestroyHighestHealthEnemyMinion
            }
            CardEffectDe::ShuffleRandomLegendaryDragonsCost1 => {
                CardEffect::ShuffleRandomLegendaryDragonsCost1
            }
            CardEffectDe::DestroyLegendaryMinion => CardEffect::DestroyLegendaryMinion,
            CardEffectDe::GrantRandomFriendlyMinionAttack { attack } => {
                CardEffect::GrantRandomFriendlyMinionAttack { attack }
            }
            CardEffectDe::DiscoverOneCostSpell => CardEffect::DiscoverOneCostSpell,
            CardEffectDe::DiscoverAnySpell => CardEffect::DiscoverAnySpell,
            CardEffectDe::SummonTwoRandomOneCostMinions => {
                CardEffect::SummonTwoRandomOneCostMinions
            }
            CardEffectDe::ReopenLocationIfFelSpell => CardEffect::ReopenLocationIfFelSpell,
            CardEffectDe::SummonRandomMinionsOfCost { cost, count } => {
                CardEffect::SummonRandomMinionsOfCost { cost, count }
            }
            CardEffectDe::AddRandomNagaCost1 => CardEffect::AddRandomNagaCost1,
            CardEffectDe::HoggerStartOfGame => CardEffect::HoggerStartOfGame,
            CardEffectDe::AzalinaStartOfGame => CardEffect::AzalinaStartOfGame,
            CardEffectDe::GodfreyStartOfGame => CardEffect::GodfreyStartOfGame,
            CardEffectDe::NethrekStartOfGame => CardEffect::NethrekStartOfGame,
            CardEffectDe::MugzeeStartOfGame => CardEffect::MugzeeStartOfGame,
            CardEffectDe::BeatrixStartOfGame => CardEffect::BeatrixStartOfGame,
            CardEffectDe::AyaUpgradeCoins { card_id } => CardEffect::AyaUpgradeCoins {
                card_id: intern(card_id)?,
            },
            CardEffectDe::KarovThreeLegendaryCopies => CardEffect::KarovThreeLegendaryCopies,
            CardEffectDe::ThalenaSecondHeroPower => CardEffect::ThalenaSecondHeroPower,
            CardEffectDe::ManastormSetAfterSpell => CardEffect::ManastormSetAfterSpell,
            CardEffectDe::VanessaGetBattlecryMinionCost2Less => {
                CardEffect::VanessaGetBattlecryMinionCost2Less
            }
            CardEffectDe::TrastathGainSummonedDemonStats => {
                CardEffect::TrastathGainSummonedDemonStats
            }
            CardEffectDe::MoraggDeathrattle => CardEffect::MoraggDeathrattle,
            CardEffectDe::SummonMoragg => CardEffect::SummonMoragg,
            CardEffectDe::PassiveHeroPower => CardEffect::PassiveHeroPower,
            CardEffectDe::JadeCoin => CardEffect::JadeCoin,
            CardEffectDe::GrimyCoin => CardEffect::GrimyCoin,
            CardEffectDe::KabalCoin => CardEffect::KabalCoin,
            // M5-W2 — the closing Escape from Violet Hold wave
            CardEffectDe::SliceAndDice => CardEffect::SliceAndDice,
            CardEffectDe::TinyPal { ammo } => CardEffect::TinyPal { ammo },
            CardEffectDe::StaffOfTrickery => CardEffect::StaffOfTrickery,
            CardEffectDe::R4TCatcher => CardEffect::R4TCatcher,
            CardEffectDe::IridaSinseeker => CardEffect::IridaSinseeker,
            CardEffectDe::IridaGetVoid => CardEffect::IridaGetVoid,
            CardEffectDe::KingOfTheUnderbelly => CardEffect::KingOfTheUnderbelly,
            CardEffectDe::MurlocHolmes => CardEffect::MurlocHolmes,
            CardEffectDe::TogwaggleShuffleHands => CardEffect::TogwaggleShuffleHands,
            CardEffectDe::MaievBuffDormant => CardEffect::MaievBuffDormant,
            CardEffectDe::ZuramatPlaysDiscarded => CardEffect::ZuramatPlaysDiscarded,
            CardEffectDe::ColdSnap => CardEffect::ColdSnap,
            CardEffectDe::MindSweeper => CardEffect::MindSweeper,
            CardEffectDe::EnthralledShade => CardEffect::EnthralledShade,
            CardEffectDe::TricksyImproviser => CardEffect::TricksyImproviser,
            CardEffectDe::Judgment => CardEffect::Judgment,
            CardEffectDe::ReinforcementAura => CardEffect::ReinforcementAura,
            CardEffectDe::ScarletBruiser => CardEffect::ScarletBruiser,
            CardEffectDe::BallAndChain => CardEffect::BallAndChain,
            CardEffectDe::HolyBola => CardEffect::HolyBola,
            CardEffectDe::SpireSecurity => CardEffect::SpireSecurity,
            CardEffectDe::SmuggledShovel => CardEffect::SmuggledShovel,
            CardEffectDe::ScrambleForGear => CardEffect::ScrambleForGear,
            CardEffectDe::ReleaseTheBeasts => CardEffect::ReleaseTheBeasts,
            CardEffectDe::SewerSwimmer => CardEffect::SewerSwimmer,
            CardEffectDe::Imfernal => CardEffect::Imfernal,
            CardEffectDe::ImpGangStooge => CardEffect::ImpGangStooge,
            CardEffectDe::DisguisedDoctor => CardEffect::DisguisedDoctor,
            CardEffectDe::Sawbones => CardEffect::Sawbones,
            CardEffectDe::BoneFlurry => CardEffect::BoneFlurry,
            CardEffectDe::DrinkBlood => CardEffect::DrinkBlood,
            CardEffectDe::EmergencySurgery => CardEffect::EmergencySurgery,
            CardEffectDe::DisguisedWatchman => CardEffect::DisguisedWatchman,
            CardEffectDe::PickPocket => CardEffect::PickPocket,
            CardEffectDe::JadeGuardians => CardEffect::JadeGuardians,
            CardEffectDe::CrowdControl => CardEffect::CrowdControl,
            CardEffectDe::VigilantSentry => CardEffect::VigilantSentry,
            CardEffectDe::VioletPunisher => CardEffect::VioletPunisher,
            CardEffectDe::BreakoutArchitect => CardEffect::BreakoutArchitect,
            CardEffectDe::NoxiousBribe => CardEffect::NoxiousBribe,
            CardEffectDe::AlarmOMatic => CardEffect::AlarmOMatic,
            CardEffectDe::SpitefulChef => CardEffect::SpitefulChef,
            CardEffectDe::Annihilation => CardEffect::Annihilation,
            CardEffectDe::SpireOfSolitude => CardEffect::SpireOfSolitude,
            CardEffectDe::ShadowRounds => CardEffect::ShadowRounds,
            CardEffectDe::ScarletRecruiter => CardEffect::ScarletRecruiter,
            CardEffectDe::ThievesTools => CardEffect::ThievesTools,
            CardEffectDe::VoidSoul => CardEffect::VoidSoul,
            CardEffectDe::CodeViolet => CardEffect::CodeViolet,
            CardEffectDe::MoltenGold => CardEffect::MoltenGold,
            CardEffectDe::Frostshatter => CardEffect::Frostshatter,
            CardEffectDe::Stormfury => CardEffect::Stormfury,
            CardEffectDe::Hexmarshal => CardEffect::Hexmarshal,
            CardEffectDe::LethalRecipe => CardEffect::LethalRecipe,
            CardEffectDe::DigForFreedom => CardEffect::DigForFreedom,
            CardEffectDe::GuardDog => CardEffect::GuardDog,
            CardEffectDe::BeastTripwire => CardEffect::BeastTripwire,
            CardEffectDe::ArcaneTripwire => CardEffect::ArcaneTripwire,
            CardEffectDe::CapturedArchmage => CardEffect::CapturedArchmage,
            CardEffectDe::FranticForger => CardEffect::FranticForger,
            CardEffectDe::LowSecurityWing => CardEffect::LowSecurityWing,
            CardEffectDe::DemonicConfinement => CardEffect::DemonicConfinement,
            CardEffectDe::WidowsBite => CardEffect::WidowsBite,
            CardEffectDe::WidowsFeast => CardEffect::WidowsFeast,
            CardEffectDe::WidowsBanquet => CardEffect::WidowsBanquet,
            CardEffectDe::Nab => CardEffect::Nab,
            CardEffectDe::VoidBlast => CardEffect::VoidBlast,
            CardEffectDe::CosmicManifestations => CardEffect::CosmicManifestations,
            CardEffectDe::DefiasWannabe => CardEffect::DefiasWannabe,
            CardEffectDe::Soothsayer => CardEffect::Soothsayer,
            CardEffectDe::HolyEmbrace => CardEffect::HolyEmbrace,
            CardEffectDe::RatBurglar => CardEffect::RatBurglar,
            CardEffectDe::DarkBribe => CardEffect::DarkBribe,
            CardEffectDe::AncientAugurPick => CardEffect::AncientAugurPick,
            CardEffectDe::AncientAugurDeathrattle => CardEffect::AncientAugurDeathrattle,
            CardEffectDe::ArachnathidVenom => CardEffect::ArachnathidVenom,
            CardEffectDe::TruthSeeker => CardEffect::TruthSeeker,
            CardEffectDe::ConcealingConfection => CardEffect::ConcealingConfection,
            CardEffectDe::DisguisedExecutioner => CardEffect::DisguisedExecutioner,
            CardEffectDe::GetawayHogdriver => CardEffect::GetawayHogdriver,
            CardEffectDe::EscapeArtist => CardEffect::EscapeArtist,
            CardEffectDe::BloodClone => CardEffect::BloodClone,
            CardEffectDe::GallagioGoon => CardEffect::GallagioGoon,
            CardEffectDe::BlackMarketOverseer => CardEffect::BlackMarketOverseer,
            CardEffectDe::ActivatedGolem => CardEffect::ActivatedGolem,
            CardEffectDe::Hellraiser => CardEffect::Hellraiser,
            CardEffectDe::SummonTwoRandomMinionsOfCost { cost } => {
                CardEffect::SummonTwoRandomMinionsOfCost { cost }
            }
            // MEND W1 — the Druid class-set wave (src/cards/exp_cata_w5.rs)
            CardEffectDe::RefreshManaIfNoMinionPlayedLastTurn { amount } => {
                CardEffect::RefreshManaIfNoMinionPlayedLastTurn { amount }
            }
            CardEffectDe::RestoreAllFriendlyAndSummonTwoRandomCostMinions { heal, cost } => {
                CardEffect::RestoreAllFriendlyAndSummonTwoRandomCostMinions { heal, cost }
            }
            CardEffectDe::DrawAndGainArmorRepeatIfNoMinionPlayedLastTurn { draw, armor } => {
                CardEffect::DrawAndGainArmorRepeatIfNoMinionPlayedLastTurn { draw, armor }
            }
            CardEffectDe::BuffHealthTauntAndDormant { health } => {
                CardEffect::BuffHealthTauntAndDormant { health }
            }
            CardEffectDe::AddRandomDragonCostReduced { reduction } => {
                CardEffect::AddRandomDragonCostReduced { reduction }
            }
            CardEffectDe::GetThreeTreantsAndCarveNatureSpells => {
                CardEffect::GetThreeTreantsAndCarveNatureSpells
            }
            CardEffectDe::CastRandomSpellsScaledByHandTurns { base, count } => {
                CardEffect::CastRandomSpellsScaledByHandTurns { base, count }
            }
            CardEffectDe::SetCompanionReplacement { bump } => {
                CardEffect::SetCompanionReplacement { bump }
            }
            CardEffectDe::SetCompanionReplacementAndDraw { bump, draw } => {
                CardEffect::SetCompanionReplacementAndDraw { bump, draw }
            }
            CardEffectDe::SetCompanionBonus { amount } => CardEffect::SetCompanionBonus { amount },
            CardEffectDe::ReplaceCompanionsAndSummonRandomBeast { bump, cost } => {
                CardEffect::ReplaceCompanionsAndSummonRandomBeast { bump, cost }
            }
            CardEffectDe::SplitDamageAmongAllEnemiesChainOnDeath { amount } => {
                CardEffect::SplitDamageAmongAllEnemiesChainOnDeath { amount }
            }
            CardEffectDe::BuffFriendlyBeastAndRandomHandBeast { attack, health } => {
                CardEffect::BuffFriendlyBeastAndRandomHandBeast { attack, health }
            }
            CardEffectDe::DealDamageToRandomEnemyMinionExcessToHero { amount, times } => {
                CardEffect::DealDamageToRandomEnemyMinionExcessToHero { amount, times }
            }
            CardEffectDe::SetLeylineDiscount { amount } => {
                CardEffect::SetLeylineDiscount { amount }
            }
            CardEffectDe::AddRandomLeylineToHand => CardEffect::AddRandomLeylineToHand,
            CardEffectDe::SummonRandomCostMinionTimes { cost, times } => {
                CardEffect::SummonRandomCostMinionTimes { cost, times }
            }
            CardEffectDe::SetLeylineExtraTrigger { amount } => {
                CardEffect::SetLeylineExtraTrigger { amount }
            }
            CardEffectDe::DrawCardsCostsLess { reduction, count } => {
                CardEffect::DrawCardsCostsLess { reduction, count }
            }
            CardEffectDe::GetAllLeylinesAndUpgrade { upgrade } => {
                CardEffect::GetAllLeylinesAndUpgrade { upgrade }
            }
            CardEffectDe::SetLeylineEffectBonus { amount } => {
                CardEffect::SetLeylineEffectBonus { amount }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::ImbueClass;

    /// Structural guard (2025–2026 expansions M3-W2a): the bincode encoding of
    /// `CardEffect` is positional — `CardEffect` derives `Serialize` while
    /// `Deserialize` decodes through the `CardEffectDe` mirror by variant
    /// index, so the mirror's variant ORDER must match `CardEffect`'s exactly,
    /// or every variant at a drifted index silently deserializes as a
    /// DIFFERENT effect (a latent pre-existing drift of the 33-variant
    /// DestroyAndGainStats..BuffAnotherRandomFriendlyDragon run was repaired
    /// by reordering the mirror on 2026-08-09 — the M3-W2a roundtrip test
    /// caught it). This test compares the two declaration sequences directly
    /// so any future insertion in one enum without the other fails loudly.
    #[test]
    fn card_effect_de_mirror_order_matches() {
        let source = include_str!("effect.rs");
        let enum_names = |text: &str| -> Vec<String> {
            text.lines()
                .filter_map(|line| {
                    let name = line.split_whitespace().next()?;
                    if !name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                        return None;
                    }
                    let name = name.trim_end_matches(['{', ',']);
                    if name.is_empty()
                        || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    {
                        return None;
                    }
                    Some(name.to_string())
                })
                .collect::<Vec<_>>()
        };
        let ce = source
            .split("pub enum CardEffect {")
            .nth(1)
            .expect("CardEffect")
            .split("/// Deserialization mirror")
            .next()
            .expect("mirror head");
        let de = source
            .split("enum CardEffectDe {")
            .nth(1)
            .expect("CardEffectDe")
            .split("impl<'de> serde::Deserialize<'de> for CardEffect")
            .next()
            .expect("impl head");
        let ce_names = enum_names(ce);
        let de_names = enum_names(de);
        assert_eq!(
            ce_names.len(),
            de_names.len(),
            "CardEffect ({}) and CardEffectDe ({}) must declare the same number of variants",
            ce_names.len(),
            de_names.len()
        );
        for (i, (a, b)) in ce_names.iter().zip(de_names.iter()).enumerate() {
            assert_eq!(
                a, b,
                "bincode mirror drift at variant index {i}: CardEffect has {a} but \
                 CardEffectDe has {b} — a variant was inserted into only one enum; \
                 keep the CardEffectDe mirror in lockstep"
            );
        }
    }

    /// Every new 2025–2026 expansions M1-W1 variant survives the bincode
    /// roundtrip (CardEffectDe → CardEffect via the interned card ids).
    #[test]
    fn imbue_effects_serialize_roundtrip() {
        for effect in [
            CardEffect::ImbueHeroPower,
            CardEffect::ImbuedHeroPower {
                class: ImbueClass::Druid,
            },
            CardEffect::ImbuedHeroPower {
                class: ImbueClass::Shaman,
            },
            CardEffect::UseHeroPower,
            CardEffect::DrawBeastAndImbue,
            CardEffect::RestoreAndDrawAndImbue { amount: 4 },
            CardEffect::SummonRandomTwoCostTauntAndImbue,
            CardEffect::ImbueAndReduceHandCost,
            CardEffect::ImbueAndTriggerHeroPower,
            CardEffect::ImbueAndGetWisp,
            CardEffect::ImbueAndDebuffEnemies {
                attack_reduction: 2,
            },
            CardEffect::DealDamageIfImbuedTwice { damage: 4 },
            CardEffect::DiscoverWildGodIfImbued4,
            CardEffect::ImbueEveryThirdSpell,
            CardEffect::SummonRandomDragonOfCost { cost: 1 },
        ] {
            let bytes = bincode::serialize(&effect).expect("serialize");
            let back: CardEffect = bincode::deserialize(&bytes).expect("deserialize");
            assert_eq!(back, effect);
        }
    }

    /// Every new 2025–2026 expansions M1-W3 variant survives the bincode
    /// roundtrip (CardEffectDe → CardEffect).
    #[test]
    fn choose_one_effects_serialize_roundtrip() {
        use crate::core::component::CardType;
        for effect in [
            CardEffect::GainStatsAndGrantDivineShield {
                attack: 3,
                health: 0,
                target: EffectTarget::Self_,
            },
            CardEffect::GainStatsAndGrantLifesteal {
                attack: 0,
                health: 3,
                target: EffectTarget::Self_,
            },
            CardEffect::GrantPoisonousThisTurn,
            CardEffect::GrantHeroLifestealThisTurn,
            CardEffect::GrantWeaponDeathrattleAllEnemies { damage: 2 },
            CardEffect::DrawCardByType {
                count: 1,
                card_type: CardType::Spell,
            },
            CardEffect::DrawCardByType {
                count: 1,
                card_type: CardType::Minion,
            },
            CardEffect::SpendCorpsesDamageMinion { cost: 2, damage: 4 },
            CardEffect::DamageAllMinions { damage: 2 },
            CardEffect::AddRandomDruidSpell,
            CardEffect::AddRandomOtherClassChooseOneCard,
        ] {
            let bytes = bincode::serialize(&effect).expect("serialize");
            let back: CardEffect = bincode::deserialize(&bytes).expect("deserialize");
            assert_eq!(back, effect);
        }
    }

    /// Every new 2025–2026 expansions M1-W2 variant survives the bincode
    /// roundtrip (CardEffectDe → CardEffect).
    #[test]
    fn dark_gift_effects_serialize_roundtrip() {
        use crate::core::component::DarkGiftKind;
        for effect in [
            CardEffect::ApplyDarkGift {
                gift: DarkGiftKind::Charge,
            },
            CardEffect::DiscoverWithDarkGift {
                pool: RandomPool::DeathrattleMinion,
            },
            CardEffect::DiscoverWithDarkGift {
                pool: RandomPool::UndeadMinion,
            },
            CardEffect::DiscoverWithDarkGift {
                pool: RandomPool::DemonCost5Plus,
            },
            CardEffect::DiscoverDragonWithDarkGift,
            CardEffect::DiscoverUndeadWithCorpseGift { corpses: 2 },
            CardEffect::DiscoverEnemyDeckMinionCopy { with_gift: true },
            CardEffect::DiscoverEnemyDeckMinionCopy { with_gift: false },
            CardEffect::DiscoverDeckMinionWithDarkGift,
            CardEffect::ReduceHandMinionGiftCost,
        ] {
            let bytes = bincode::serialize(&effect).expect("serialize");
            let back: CardEffect = bincode::deserialize(&bytes).expect("deserialize");
            assert_eq!(back, effect);
        }
    }

    /// Every new 2025–2026 expansions M1-W4a variant survives the bincode
    /// roundtrip (CardEffectDe → CardEffect via the interned card ids).
    #[test]
    fn w4a_effects_serialize_roundtrip() {
        for effect in [
            CardEffect::AttackTwoRandomEnemyMinionsIfCostLE { cost: 3 },
            CardEffect::GainArmorSummonCostTaunt { armor: 5, cost: 5 },
            CardEffect::AddRandomCostMinionWithDarkGift { cost: 3 },
            CardEffect::BuffTopDeckMinions {
                attack: 4,
                health: 4,
                count: 3,
            },
            CardEffect::ShuffleAllMinionsIntoDecks,
            CardEffect::DrawDeckSpellAndAddRandomSpell,
            CardEffect::SetStatsByFriendlyTarget {
                enemy_attack: 1,
                enemy_health: 1,
                friendly_attack: 3,
                friendly_health: 3,
            },
            CardEffect::GainAttackEqualSpellCost,
            CardEffect::DamageLowestHealthEnemyTwice { amount: 5 },
            CardEffect::DrawAndGainStats {
                attack: 2,
                health: 2,
            },
            CardEffect::ShuffleCardIntoDeck {
                card_id: "EDR_260t",
                count: 2,
            },
            CardEffect::AmphibianSpiritBuff {
                attack: 2,
                health: 2,
            },
            CardEffect::DamageAndSummonWolfIfKilled { damage: 3 },
            CardEffect::AddRandomSpellCostsLess { reduction: 2 },
            CardEffect::SummonTreantCopyingSpell,
            CardEffect::SummonEggHatchingDragon,
            CardEffect::ResurrectRandomFallenDragon,
            CardEffect::EquipSwordIfHoldingDragon,
            CardEffect::DamageAllOtherFriendlyMinions { damage: 3 },
            CardEffect::DamageMinionWithMoonLifesteal { amount: 6 },
            CardEffect::SummonTwoRandomCostMinions {
                base_cost: 3,
                upgraded_cost: 6,
            },
            CardEffect::DamageIfHoldingSpell5Plus { amount: 3 },
            CardEffect::SummonCopyIfAttackGE { attack: 4 },
            CardEffect::RestoreHealthAndPendingSelfDamage {
                heal: 12,
                damage: 3,
                turns: 2,
            },
            CardEffect::DestroyCrystalGainCrystalsLater { gain: 2, turns: 2 },
            CardEffect::DrawMinionCostGE { cost: 7 },
            CardEffect::GainDeathrattleOfDiedThisTurn,
            CardEffect::AddRandomDeckMinionToHand,
            CardEffect::EatDeckMinionGainStats,
            CardEffect::DebuffRandomHandMinionBoth {
                attack_reduction: 2,
            },
            CardEffect::SpendAllManaCastRandomSpell,
            CardEffect::CopyLowestCostEnemyHandCard,
            CardEffect::OpponentDrawsTwoAndCopies,
            CardEffect::ReturnFriendlyMinionSummonSpider,
            CardEffect::ShuffleMatchingEnemyHandCardIntoDeck,
            CardEffect::DestroyFriendlyMinionGainArmor { armor: 8 },
            CardEffect::DrawSpellCostGE { cost: 5 },
            CardEffect::DrawDragonsReduced {
                count: 2,
                reduction: 1,
            },
            CardEffect::SummonCopyOfSelf,
            CardEffect::DestroyFriendlyWispDraw { count: 3 },
            CardEffect::DrawAndSummonLeeches { draw: 2 },
            CardEffect::DrawAndSummonDreadseed { draw: 1 },
            CardEffect::NextHeroPowerCostsZero,
            CardEffect::RestoreHealthAndGetDruidSpells {
                amount: 6,
                count: 3,
            },
            CardEffect::GainManaCrystalBoth { count: 1 },
            CardEffect::TransformNeutralDeckToDruid,
            CardEffect::AddMoonfireAndStarfireWithSpellDamage,
            CardEffect::BuffAnotherRandomFriendlyDragon {
                attack: 1,
                health: 1,
            },
            CardEffect::ReduceRightmostHandCardCost { reduction: 2 },
            CardEffect::ResurrectDeathrattleMinionCostLE { cost: 4 },
            CardEffect::ResurrectDeathrattleMinionCostGE { cost: 5 },
            CardEffect::GainArmorPerWisp { base: 1 },
            CardEffect::DamageMinionScaledByFallen { base: 1 },
            CardEffect::GrantHeroDivineShield,
            CardEffect::RestoreBothHeroes { amount: 3 },
            CardEffect::AddSelfToDeckBottomCost { cost: 1 },
            CardEffect::SummonCopyOfRandomFriendlyDragon,
            CardEffect::GainHealthIfHeroPowerUsed { amount: 2 },
            CardEffect::AttackRandomEnemyMinionExcess,
            CardEffect::SplashHeroAttackToRandomEnemy,
            CardEffect::GainDeadMinionAttack,
            CardEffect::DrawIfMinionPlayedBefore,
            CardEffect::GrantRandomBonusEffect,
            CardEffect::YseraEmeraldAspect,
            CardEffect::ResurrectAllDifferentFriendlyCostGE { cost: 8 },
            CardEffect::CastHighestCostSpellFromHand,
            CardEffect::IncrementOmenAttack,
            CardEffect::OmenDeathrattle,
            CardEffect::SplitDamageAmongAllEnemiesIfFallen {
                amount: 20,
                threshold: 20,
            },
            CardEffect::NextSpellsCastTwice { count: 3 },
            CardEffect::SummonRandomDragonPerSelfDeath,
            CardEffect::GainArmorAndSelfAttack {
                armor: 1,
                attack: 1,
            },
            CardEffect::NextCardCostsZero,
            CardEffect::TransformHandMinionsToRandomDemons,
            CardEffect::DiscoverSpellKeepOrTop,
            CardEffect::DiscardRandomEnemyHandCard,
            CardEffect::FillHandWithEnemyDeckCopies { reduction: 3 },
            CardEffect::SummonBeetles { count: 3 },
            CardEffect::UrsocBattlecry,
            CardEffect::UrsocDeathrattle,
            CardEffect::GainStatsAllOtherFriendlyMinions {
                attack: 1,
                health: 3,
            },
            CardEffect::SummonRandomAnimalCompanion,
            CardEffect::AddAllDreamCards,
            CardEffect::CardsCostOneThisGame,
            CardEffect::SetMurlocSummonBuff,
            CardEffect::SetDealExact2Bonus,
            CardEffect::GainRush {
                target: EffectTarget::Self_,
            },
            CardEffect::GainImmuneThisTurn {
                target: EffectTarget::Self_,
            },
            CardEffect::NextMurlocCostsLess { amount: 1 },
            CardEffect::GiveNextMurlocDivineShield,
            CardEffect::SetNextKindredTwice,
            CardEffect::DrawKindredAndActivator,
            CardEffect::DrawSpellGiveSpellDamage { amount: 2 },
            CardEffect::DrawMinionsOfEachCost { up_to: 4 },
            CardEffect::DrawDeathrattleMinionCostLE { max_cost: 3 },
            CardEffect::DestroyLowestAttackEnemy,
            CardEffect::TriggerFriendlyCinderDeathrattles,
            CardEffect::DestroyMinionAndGainItsStats {
                target: EffectTarget::AnyMinion,
            },
            CardEffect::DealSelfAttackDamage {
                target: EffectTarget::AnyEnemyMinion,
            },
            CardEffect::SummonRandomMinionCostTaunt { cost: 4 },
            CardEffect::DiscoverPool {
                pool: DiscoverPool::FrostRune,
            },
            CardEffect::AddRandomCardToHandCount {
                pool: RandomPool::FelSpell,
                count: 2,
            },
            CardEffect::AddCardToHandCount {
                card_id: "TLC_630t",
                count: 2,
            },
            CardEffect::AddTemporaryRandomMinionsCost { cost: 2, count: 2 },
            CardEffect::AddRandomFelSpellsCostHealth { count: 2 },
            CardEffect::AddRandomHolyAndShadowSpell,
            CardEffect::AddRandomHolySpellCost1,
            CardEffect::CopyRandomHandElementalOrDragon,
            CardEffect::ReduceRandomEnemyHandMinionCost { amount: 2 },
            CardEffect::ReduceRandomBeastHandCost { amount: 1 },
            CardEffect::ReduceNonStartingHandCost { amount: 1 },
            CardEffect::SummonTreantsAttackMinion,
            CardEffect::DealDamageSummonCinders { amount: 3 },
            CardEffect::DealDamageLowestHealthEnemyRepeated {
                amount: 2,
                times: 3,
            },
            CardEffect::DealDamageRandomEnemies {
                amount: 6,
                count: 3,
            },
            CardEffect::DrawMinionsDifferentTypesBuff {
                count: 2,
                attack: 1,
                health: 1,
            },
            CardEffect::DrawMinionBuffArmorIfAttackGE {
                min_attack: 5,
                buff_health: 5,
                armor: 5,
            },
            CardEffect::SetFlockPending,
            CardEffect::GiveBuffOtherMinionsAttackLE {
                attack: 1,
                health: 1,
                max_attack: 2,
            },
            CardEffect::DestroyMinionSummonRandomSameCost {
                target: EffectTarget::AnyMinion,
            },
            CardEffect::SummonMinionsGrantRandomBonus {
                card_id: "TLC_240t",
                count: 3,
            },
            CardEffect::SummonMinionPair {
                a: "TLC_468t1",
                b: "TLC_468t2",
            },
            CardEffect::SummonRandomMinionCostOrEscalated {
                cost: 2,
                escalated_cost: 4,
            },
            CardEffect::DealDamageGainArmorIfKilled {
                amount: 2,
                armor: 5,
                target: EffectTarget::AnyEnemyMinion,
            },
            CardEffect::DealDamageAllEnemyMinionsSetMinionsCostMore { damage: 2 },
            CardEffect::GiveBuffSameType {
                attack: 1,
                health: 2,
                target: EffectTarget::FriendlyMinion,
            },
            CardEffect::GrantRandomBonusEffects {
                count: 3,
                target: EffectTarget::AnyMinion,
            },
            CardEffect::GrantRandomBonusEffectAndDeathrattle,
            CardEffect::SetLakkariTicks { ticks: 3 },
            CardEffect::GiveBuffAndSummonDeathrattle {
                attack: 4,
                health: 4,
                summon_cost: 4,
                target: EffectTarget::FriendlyMinion,
            },
            CardEffect::DealDamageImprovedByShuffles {
                amount: 1,
                target: EffectTarget::AnyMinion,
            },
            CardEffect::DrawCardLinkDeathrattle,
            CardEffect::DiscardLinkedDrawnCard,
            CardEffect::GainArmorDealDamageEqual {
                armor: 3,
                target: EffectTarget::AnyEnemyMinion,
            },
            CardEffect::DestroyDeckTop { count: 3 },
            CardEffect::ResurrectOneOfEachCostGiveReborn { max_cost: 3 },
            CardEffect::DealDamageSetNextBeastDiscount {
                amount: 3,
                discount: 2,
                target: EffectTarget::AnyMinion,
            },
            CardEffect::BuffAllBeastsEverywhere {
                attack: 2,
                health: 2,
            },
            CardEffect::DealDamageSameType {
                amount: 3,
                target: EffectTarget::AnyMinion,
            },
            CardEffect::SetNextTemporaryDiscount { amount: 2 },
            CardEffect::SetEnemyHeroCantBeHealed,
            CardEffect::DestroyFriendlyMinionAddBones {
                target: EffectTarget::FriendlyMinion,
            },
            CardEffect::GainManaCrystalsMatchOpponent,
            CardEffect::GiveBuffDifferentTypeMinions {
                attack: 1,
                health: 1,
            },
            CardEffect::DealDamageIfQuestPlayed {
                amount: 3,
                target: EffectTarget::AnyEnemyMinion,
            },
            CardEffect::SwapHeroPowerToDeal8Random,
            CardEffect::RecastRandomHolySpellThisTurn,
            CardEffect::CastRandomSpellFromDeckCostLE { max_cost: 2 },
            CardEffect::SpendArmorDealDamageAllMinions { max_spend: 5 },
            CardEffect::DestroyTopCardDiscoverSameRarity,
            CardEffect::GrantKeyword {
                keyword: KeywordKind::Taunt,
                target: EffectTarget::Self_,
            },
            CardEffect::GrantDeathrattleSummon {
                card_id: "TLC_245t",
                count: 2,
                target: EffectTarget::Self_,
            },
            CardEffect::AddRandomWeaponAnotherClassComboAttack { combo_attack: 2 },
            CardEffect::Drain { amount: 1 },
            CardEffect::GainStatsEqualFireSpellCost,
            CardEffect::DealDamageAndSummon {
                amount: 2,
                card_id: "TLC_903t",
            },
            CardEffect::DiscoverDeckCard,
            CardEffect::DiscoverEnemyDeckTop,
            CardEffect::SummonRandomFelBeast,
            CardEffect::AddRandomBeastCostLess { amount: 3 },
            CardEffect::TriggerFriendlyDeadDeathrattles { count: 5 },
            CardEffect::EshoDeckCheckBuffEverywhere {
                attack: 2,
                health: 2,
            },
            CardEffect::SetStatsAllEnemyMinions {
                attack: 1,
                health: 1,
            },
            CardEffect::SummonDamagedCopiesRush,
            CardEffect::SummonTwoDeathrattleMinionsAndFight,
            CardEffect::LohMinionsCost5,
            CardEffect::EliseCraftLocation,
            CardEffect::NiriOfTheCrater,
            CardEffect::SetEventSubjectHealthToSource,
            CardEffect::DealDamageToLeftRightEnemyMinions { amount: 6 },
            CardEffect::GiveOtherFriendlyMinionsRush,
            CardEffect::DealDamageToTwoAndFreeze { amount: 2 },
            CardEffect::SetStatsAndFillBoardWithCopies {
                attack: 1,
                health: 1,
            },
            CardEffect::SetStatsAndGrantCharge {
                attack: 8,
                health: 8,
                target: EffectTarget::AnyMinion,
            },
            CardEffect::SetStatsGrantLifestealForceAttack {
                attack: 8,
                health: 10,
                target: EffectTarget::AnyMinion,
            },
            CardEffect::SetStatsAttachDamageAllDeathrattle {
                attack: 1,
                health: 1,
                target: EffectTarget::AnyMinion,
            },
            CardEffect::SetStatsGrantStealthAndDraw {
                attack: 5,
                health: 4,
                draw: 2,
                target: EffectTarget::AnyMinion,
            },
            CardEffect::SummonMinionAndBuffFriendlyMinions {
                card_id: "DINO_130t",
                attack: 1,
                health: 1,
            },
            CardEffect::SummonRandomDeckBeastGiveLifesteal,
            CardEffect::SummonRaptorsOutcast { count: 3 },
            CardEffect::ReduceAdjacentHandCardCost { amount: 1 },
            CardEffect::DracorexSplash,
            CardEffect::SetHatchingPending,
            CardEffect::DealDamageAndBuffFriendlyElementals {
                damage: 4,
                attack: 1,
                health: 1,
            },
            CardEffect::ShuffleLeftmostHandCardIntoDeck,
            CardEffect::DrawZeroAttackMinion,
            CardEffect::TransformRandomMinionIntoRandomMinion,
            CardEffect::SummonRandomDeathrattleMinionCostGEAndTrigger { min_cost: 5 },
            CardEffect::SpendCorpsesGainReborn { amount: 3 },
            CardEffect::SoulrestMarkAndBuff,
            CardEffect::GainStatsAndGrantRush {
                attack: 2,
                health: 2,
                target: EffectTarget::FriendlyRace(crate::core::component::Race::Beast),
            },
            CardEffect::BuffHandAndDeckMinions {
                attack: 3,
                health: 3,
            },
            CardEffect::SummonTwoRandomCostBeastsAttackRandomEnemies { cost: 3 },
            CardEffect::SummonRandomLegendaryMinionSetStats {
                attack: 10,
                health: 10,
            },
            CardEffect::SummonRandomCostMinionSetStats {
                cost: 3,
                attack: 2,
                health: 3,
            },
            CardEffect::AddRandomMaskCombo { reduction: 2 },
            CardEffect::GainStatsOfRandomLegendaryBeast,
            CardEffect::SummonRandomLegendaryBeast,
            CardEffect::SummonRandomTauntMinionCostGE { min_cost: 5 },
            CardEffect::SummonRandomTauntMinionsOfCosts { a: 6, b: 4, c: 2 },
            CardEffect::AddRandomOneCostMinion,
            CardEffect::AddRandomOneCostSpell,
            CardEffect::AddRandomMultiTribeMinion,
        ] {
            let bytes = bincode::serialize(&effect).expect("serialize");
            let back: CardEffect = bincode::deserialize(&bytes).expect("deserialize");
            assert_eq!(back, effect, "roundtrip failed for {effect:?}");
        }
    }

    /// Every new 2025–2026 expansions M3-W2a variant (Across the Timeways,
    /// src/cards/exp_tmw_w2a.rs) survives the bincode roundtrip
    /// (CardEffectDe → CardEffect via the interned card ids).
    #[test]
    fn w2a_effects_serialize_roundtrip() {
        for effect in [
            CardEffect::AddRandomMinionCostsLess { reduction: 3 },
            CardEffect::AddRandomSpellsFromClass { count: 2 },
            CardEffect::DrawRandomMinionGiveStats {
                attack: 2,
                health: 2,
            },
            CardEffect::BothPlayersDiscardRandomCard,
            CardEffect::RestoreAndGrantHeroDivineShield { amount: 3 },
            CardEffect::ShuffleCardIntoDeck {
                card_id: "TIME_025t",
                count: 2,
            },
            CardEffect::CastShredFromDeckGainStats {
                attack: 3,
                health: 3,
            },
            CardEffect::CastShredFromDeckSummonCopy,
            CardEffect::CastRandomNatureSpells { count: 2 },
            CardEffect::BothPlayersEquipRandomWeaponBuffOurs {
                attack: 1,
                health: 1,
            },
            CardEffect::AddRandomRewindCardToHand,
            CardEffect::CopyRightmostEnemyHandCardOrIncreaseCost,
            CardEffect::DrawMinionAndBuffHandMinionsHealth { health: 2 },
            CardEffect::SummonTwoRandomLegendaryMinions,
            CardEffect::GuessEnemyHandGainHealth { health: 4 },
            CardEffect::SetStatsAndCantAttackHeroesThisTurn {
                attack: 8,
                health: 8,
            },
            CardEffect::GainStatsPerTurnTaken {
                attack: 0,
                health: 1,
            },
            CardEffect::TransformSelfToRandomMinionOfCost { cost: 5 },
            CardEffect::SwapStatsIfSurvivesDamage,
            CardEffect::TransformSelfIfSurvivesDamageToRandomCost { cost: 7 },
            CardEffect::SummonRandomMinionOfCostDormant { cost: 2, turns: 2 },
            CardEffect::ResetBothHandsCosts,
            CardEffect::ReverseDeckOrder,
            CardEffect::GainTauntAndDivineShieldIfHoldingDragon,
            CardEffect::GainStatsAndDrawIfNatureSpellCast {
                attack: 1,
                health: 1,
            },
            CardEffect::DealDamageEnemyMinionEqualToSourceHealth,
            CardEffect::SetHandMinionStatsToHigher,
            CardEffect::RestoreHealthEqualToSourceHealth,
            CardEffect::SummonShadowAttacksRandomEnemy {
                card_id: "TIME_434t",
            },
            CardEffect::ImprisonEnemyMinion,
            CardEffect::AwakenImprisonedMinion,
            CardEffect::DrawUntilHandSize { size: 3 },
            CardEffect::AddRandomLegendaryMinionCostReduced { reduction: 1 },
            CardEffect::DealDamageEnemyMinionIfHeroHealthChanged { amount: 6 },
            CardEffect::GainStatsAndSummonCopyIfHeroHealthLE {
                attack: 5,
                health: 5,
                threshold: 10,
            },
            CardEffect::GetPupilAndDiscoverSpellCostGE {
                pupil_card_id: "TIME_704t",
                min_cost: 7,
            },
            CardEffect::SummonManaWorthRandomMinions { total: 12 },
            CardEffect::DiscoverPaladinMechPastGiveStats {
                attack: 5,
                health: 5,
            },
            CardEffect::GetHolySpellsRestoreHealthEqualCosts,
            CardEffect::DrawBottomCards { count: 2 },
            CardEffect::BuffAllFriendlyMinionsShuffleShreds {
                attack: 1,
                health: 1,
            },
            CardEffect::DealDamageSplitAmongAllEnemiesShuffleShreds { amount: 6 },
            CardEffect::CopyRandomHandMinion,
            CardEffect::DrawCardsOfDifferentCosts { count: 3 },
            CardEffect::DiscoverEnemyHandCardCopy,
            CardEffect::DealDamageFriendlyMinionToRandomEnemy {
                damage: 2,
                amount: 4,
            },
            CardEffect::DamageAllMinionsAndAddCardToHand {
                amount: 1,
                card_id: "TIME_218",
            },
            CardEffect::DamageAndDrawTwoIfSurvives {
                damage: 5,
                target: EffectTarget::AnyMinion,
            },
            CardEffect::DamageMinionGiveHeroAttack {
                damage: 1,
                attack: 1,
            },
            CardEffect::DiscoverDeckAndEnemyHandCardCopy,
            CardEffect::SilenceAndDestroyRandomEnemyMinion,
            CardEffect::SummonTwoDemonsAttackLowestHealthIfDeckNoMinions,
            CardEffect::GrantDivineShieldAndBuffHandMinionsHealth { health: 2 },
            CardEffect::DiscoverMinionReduceHandCostsIfDeckNoMinions { reduction: 2 },
            CardEffect::GainHeroAttackAndBuffHandMinionsIfDeckNoMinions { attack: 4 },
            CardEffect::PreciseShot {
                amount: 3,
                center_amount: 5,
            },
            CardEffect::SummonRandomCostBeastAttackRandomEnemy { cost: 3 },
            CardEffect::SummonMinionsGrantTwoRandomBonus {
                card_id: "TIME_610t2",
                count: 4,
            },
            CardEffect::DealDamageToTwoAndFreeze { amount: 3 },
            CardEffect::FillHandWithRandomUndeadCostHealth,
            CardEffect::SummonHighestCostFallenUndead,
            CardEffect::ResurrectDiedMinionFull,
            CardEffect::SetChronologicalAura { ticks: 3 },
            CardEffect::DiscoverDeckCardOthersBottom,
            CardEffect::DamageAndGainArmorIfMinionPlayedWhileHeld {
                damage: 3,
                armor: 5,
            },
            CardEffect::ReplaceHandAndDeckWithRandomChooseOne,
            CardEffect::SummonTwoRandomCostMinionsWithAttack { cost: 1, bonus: 0 },
            CardEffect::DestroyMinionAndSummonRandomCost { cost: 0 },
            CardEffect::DrawTwoReduceRandomCost { reduction: 2 },
            CardEffect::DealDamagePrimaryAndSplash {
                primary: 3,
                splash: 2,
            },
            CardEffect::DiscoverArcaneSpellsReduced { reduction: 2 },
            CardEffect::SummonPairScrambleStats {
                first_cost: 10,
                second_cost: 1,
            },
            CardEffect::SummonRandomDeckMinionAndTigerForOpponent {
                card_id: "TIME_870t",
            },
            CardEffect::GainArmorAndSummonTwoBeastsForOpponent {
                armor: 10,
                card_id: "TIME_873t",
            },
            CardEffect::AddRandomCostMinionMarkedTurnDiscount { cost: 8 },
            CardEffect::AddRandomBeastsToBottomDeckWithStats {
                count: 2,
                attack: 5,
                health: 5,
            },
            CardEffect::DamageAndDrawMinionIfHoldingCostGE { damage: 3, cost: 5 },
            CardEffect::NextTurnEnemyCardsCostMore { amount: 1 },
            CardEffect::LookAtSecretsGiveRandom,
            CardEffect::DealDamageAllEnemiesIfControllingAura { amount: 3 },
            CardEffect::GiveHeroImmuneThisTurn,
            CardEffect::GainStatsPerDamagedMinion {
                attack: 2,
                health: 2,
            },
            CardEffect::FillEnemyBoardWithRandomCost1Minions,
            CardEffect::TransformHandSelfToRandomEnemyHandMinion,
            CardEffect::DiscoverPool {
                pool: DiscoverPool::Spell,
            },
            CardEffect::DamageSelfHero { damage: 3 },
            CardEffect::SummonRandomMinionFromDeck,
            CardEffect::SylvanasDealToAllEnemiesRepeated { damage: 2 },
            CardEffect::ChronogorDrawsHighestLowest,
            CardEffect::DiscardHandAndAddInfiniteBanana,
            CardEffect::MurozondPrepareInfiniteAttack,
            CardEffect::AddCopiesOfLastPlayedCards { count: 3 },
            CardEffect::SummonLocationForPlayer {
                card_id: "TIME_211t1",
            },
            CardEffect::SummonCopyOfFriendlyMinion,
            CardEffect::FillHandWithRandomTemporarySpells,
            CardEffect::TakeControlEnemyMinionHealthLE,
            CardEffect::DiscoverDemonGE5AndSetNextDemonCostOne,
            CardEffect::SpendCorpsesRestoreHeroHealth { max: 20 },
            CardEffect::DrawOrResurrectBwonsamdiAndGrantBoon {
                keyword: KeywordKind::Lifesteal,
            },
            CardEffect::SummonRandomCostMinion { cost: 4 },
            CardEffect::SetDeckBottomCostsOne { count: 5 },
            CardEffect::ReplaceHandAndSwapBackAtTurnEnd,
            CardEffect::RestoreHandSnapshot,
            CardEffect::SummonChestForOpponent,
            CardEffect::FillOpponentHandWithCoins,
            CardEffect::DestroyAllMinionsOpponentPlayedLastTurn,
            CardEffect::SummonBloodFighterFromHandBuffAndAttack,
            CardEffect::GetThreeRandomSpellsFromPastTracked,
            CardEffect::DestroyHeldKingLlaneAndHalveEnemyHealth,
            CardEffect::SilenceAndDestroyAllOtherMinions,
            // M3-W3 — the Across the Timeways closing wave (The End of
            // Time miniset; every new variant gets a roundtrip entry).
            CardEffect::DealDamageAndImbue {
                amount: 2,
                target: EffectTarget::AnyCharacter,
            },
            CardEffect::DrawUndeadAndImbueTwice,
            CardEffect::EquipDaggerOrBuffWeapon,
            CardEffect::BygoneEchoesSummon,
            CardEffect::ChronikarHeroAttackBuff,
            CardEffect::ChronikarRebuff,
            CardEffect::PressTheAdvantage,
            CardEffect::RefreshManaCrystals { amount: 2 },
            CardEffect::SummonTwoTreantsScaling,
            CardEffect::SetAllOtherMinionsAttack { attack: 1 },
            CardEffect::SetAllOtherMinionsHealth { health: 1 },
            CardEffect::ArmAccelerationAura,
            CardEffect::SetWeaponAttackInfinityThisTurn,
            CardEffect::DamageAndBuffFriendlyIfKilled {
                amount: 3,
                attack: 3,
                health: 3,
            },
            CardEffect::AddRandomDeathrattleMinionCostsLess,
            CardEffect::DiscardHighestCostCard,
            CardEffect::DrawUntilHandFull,
            CardEffect::EmptyOpponentHand,
            CardEffect::SetRandomHandCardCostInfinity,
            CardEffect::RestoreInfinityHandCardCost,
            CardEffect::GainStatsIfHeroDamagedThisTurn {
                attack: 3,
                health: 3,
            },
            CardEffect::DamageMinionDrawIfSurvivesSummonIfDies { amount: 1 },
            CardEffect::BuffHandMinionsAndWeapons { attack: 2 },
            CardEffect::FreezeMinionAndNeighborsDestroyDamaged,
            CardEffect::InfiniteDamageToHighestHealthEnemyMinion,
            CardEffect::DamageMinionEternalFirebolt { amount: 3 },
            CardEffect::DestroyAllMinionsWith4OrLessAttack,
            CardEffect::AddRandomShadowSpell,
            CardEffect::OverloadForAndGainImmuneWindfury { overload: 2 },
            CardEffect::DestroyRandomEnemyMinionLocationWeapon,
            CardEffect::DestroyTopFiveEnemyDeckIfOwnEmpty,
            CardEffect::FillBoardRandomDragonsHealHeroSkipNextTurn,
            CardEffect::GetRandomOtherClassMinionCostsLess { reduction: 1 },
            CardEffect::BuffFirstUndeadPlayedEachTurn { attack: 1 },
            // 2025–2026 expansions M4-W1 (Colossal) variants.
            CardEffect::GainStatsAndCopyToColossalMain {
                attack: 1,
                health: 1,
            },
            CardEffect::RemoveKeywordFromColossalMain {
                keyword: KeywordKind::Taunt,
            },
            CardEffect::AddRandomCostMinionCostsHealth { cost: 0 },
            CardEffect::ColossalArmDestroyRight {
                attack: 2,
                health: 2,
            },
            CardEffect::AddRandomFireSpellCostsLess { reduction: 3 },
            CardEffect::TriggerFriendlyDeathrattles,
            CardEffect::AddRandomMinionsCostEqualAttack { count: 2 },
            CardEffect::GiveRandomFriendlyMinionAttack { attack: 2 },
            // 2025–2026 expansions M4-W3 (Shatter) variants.
            CardEffect::SummonMinionsAndGrantDeathrattleAll {
                card_id: "CATA_134t3",
                count: 2,
            },
            CardEffect::GainStatsElusiveAndSummonCopy {
                attack: 2,
                health: 3,
            },
            CardEffect::SummonMinionsAndGrantFriendlyAttackDivineShield {
                card_id: "CATA_479t3",
                count: 2,
                attack: 1,
            },
            CardEffect::DealDamageAndDamageAllEnemies { amount: 4, aoe: 2 },
            CardEffect::DrawMinionsAndBuffHandMinions {
                count: 3,
                attack: 2,
                health: 2,
            },
            CardEffect::AddRandomShatterCardToHand,
            // 2025–2026 expansions M4-W4 (Deathwing + remaining cards) variants.
            CardEffect::ReplaceHero {
                card_id: "CATA_190h",
            },
            CardEffect::ChooseCataclysms,
            CardEffect::ChooseHandCard,
            CardEffect::RefreshManaIfHoldingDragon { amount: 2 },
            CardEffect::GainTempManaOrPermanentIfSpent { threshold: 2 },
            CardEffect::GetOrSummonTauntWhelpsIfSpent { threshold: 2 },
            CardEffect::SummonGolemsSpendAllMana,
            CardEffect::ShuffleRandomMinionsCostGE8DoubleStats,
            CardEffect::GainStatsPerFriendlyMinionTargeted,
            CardEffect::FillHandWithRandomDragons { threshold: 2 },
            CardEffect::SetAttackEqualToSource,
            CardEffect::NextMurlocCostsHealth,
            CardEffect::TransformRandomEnemyMinionToSelf,
            CardEffect::GiveOpponentSabotage,
            CardEffect::ReturnEnemyMinionCantPlayNextTurn,
            CardEffect::SetHealingBonus { amount: 2 },
            CardEffect::SetNextHealDealsDamage,
            CardEffect::FullHealMinionAndDraw,
            CardEffect::DamageMinionHealEnemyHeroIfKilled { amount: 2 },
            CardEffect::LifestealSelfDamage { amount: 2 },
            CardEffect::GainHealthIfFullHealth { amount: 2 },
            CardEffect::SetRemainingHealthAndFullHealDamage {
                health: 4,
                damage: 3,
            },
            CardEffect::BuffSpellDamageHandAndDeck,
            CardEffect::AddDragonBreathScaledByAttack,
            CardEffect::DealDamageEqualDragonBreath,
            CardEffect::SummonFiveHungryDrakesSpendCorpsesRush,
            CardEffect::RefreshManaEqualSelfAttack,
            CardEffect::AddNefariansCreation { reduction: 1 },
            CardEffect::GrantDeathrattleSummonRandomCostMinion { cost: 3 },
            CardEffect::TriggerRandomFriendlyEndTurnEffect,
            CardEffect::GrantDivineShieldOrGainStats {
                attack: 2,
                health: 2,
            },
            CardEffect::AddRandomHolySpellCostReduced { reduction: 2 },
            CardEffect::SummonDragonWithSelfStats,
            CardEffect::SetEndTurnEffectsTwice { turns: 2 },
            CardEffect::DevourTwoEnemyHandCardsAndDormant,
            CardEffect::ReturnDevouredCards,
            CardEffect::SummonCopyIfSpellDamageDealtThisTurn,
            CardEffect::DealDamageAndDamageRandomEnemyMinion {
                amount: 3,
                secondary: 1,
            },
            CardEffect::GainAttackFirstSpellDamageThisTurn { attack: 2 },
            CardEffect::DamageAllMinionsRepeatDescending { amount: 3 },
            CardEffect::SummonCopyOfDiscardedMinion,
            CardEffect::TakeControlUntilEndOfTurnCantAttack,
            CardEffect::DealDamageToTwoScaledByHandTurns { base: 2 },
            CardEffect::DamageAllMinionsDrawPerDeath { amount: 2 },
            CardEffect::ReopenLocation,
            CardEffect::SetNextTurnSummon {
                card_id: "CATA_528t",
            },
            CardEffect::DealDamageLeftRightOutcastAgain { amount: 2 },
            CardEffect::DealDamageEqualSelfAttack,
            CardEffect::SetDragonsHaveRush,
            CardEffect::SetEnemyMinionHealthTo1IfHoldingDragon,
            CardEffect::AddRandomDragonCostLE3,
            CardEffect::DealDamageOrDamageAllEnemiesIfCopyPlayed { amount: 3 },
            CardEffect::ReplayOneCostCardsPlayedThisGame,
            CardEffect::CastAbsorbedSpell,
            CardEffect::GrantMegaWindfuryCantAttackHeroes,
            CardEffect::TransformFriendlyMinionsCost1MoreSummonOriginals,
            CardEffect::SummonRandomThreeTwoOneCostMinions,
            CardEffect::DrawAndReduceCostRepeated { reduction: 1 },
            CardEffect::DamageAllMinionsScaledByBoard { base: 2 },
            CardEffect::DamageAllMinionsAndGainHeroAttack {
                amount: 2,
                attack: 3,
            },
            CardEffect::DealDamageSplitAmongEnemiesIfFireSpell { amount: 6 },
            CardEffect::DamageDamagedMinionReturnIfExcess { amount: 4 },
            CardEffect::SetGeddonDiscoverDraw,
            CardEffect::GrantDeathrattleSummonRandomHandMinion,
            CardEffect::SummonRandomMinionFromHand,
            CardEffect::UpgradeHeroPowerCost1,
            CardEffect::AddRandomPaladinAura,
            CardEffect::StealHealthThreeTimes { amount: 2 },
            CardEffect::DestroyDeckCardsCostLE2Both,
            CardEffect::UnlockOverloadedCrystals,
            CardEffect::CastRandomSpellSameCostOtherClass,
            CardEffect::ReturnHoardCostLess,
            CardEffect::DamageMinionReduceHandCostByExcess { amount: 3 },
            CardEffect::AddTwoRandomSpellsSameCost { cost: 3 },
            CardEffect::Split100StatsAmongDeckMinionsIfCostGE100 { threshold: 100 },
            CardEffect::DestroyHighestHealthEnemyMinion,
            CardEffect::ShuffleRandomLegendaryDragonsCost1,
            CardEffect::DestroyLegendaryMinion,
            CardEffect::GrantRandomFriendlyMinionAttack { attack: 2 },
            CardEffect::DiscoverOneCostSpell,
            CardEffect::DiscoverAnySpell,
            CardEffect::SummonTwoRandomOneCostMinions,
            CardEffect::ReopenLocationIfFelSpell,
            CardEffect::SummonRandomMinionsOfCost { cost: 4, count: 2 },
            CardEffect::AddRandomNagaCost1,
            CardEffect::HoggerStartOfGame,
            CardEffect::AzalinaStartOfGame,
            CardEffect::GodfreyStartOfGame,
            CardEffect::NethrekStartOfGame,
            CardEffect::MugzeeStartOfGame,
            CardEffect::BeatrixStartOfGame,
            CardEffect::AyaUpgradeCoins {
                card_id: "JAIL_504t",
            },
            CardEffect::KarovThreeLegendaryCopies,
            CardEffect::ThalenaSecondHeroPower,
            CardEffect::ManastormSetAfterSpell,
            CardEffect::VanessaGetBattlecryMinionCost2Less,
            CardEffect::TrastathGainSummonedDemonStats,
            CardEffect::MoraggDeathrattle,
            CardEffect::SummonMoragg,
            CardEffect::PassiveHeroPower,
            CardEffect::JadeCoin,
            CardEffect::GrimyCoin,
            CardEffect::KabalCoin,
        ] {
            let bytes = bincode::serialize(&effect).expect("serialize");
            let back: CardEffect = bincode::deserialize(&bytes).expect("deserialize");
            assert_eq!(back, effect, "roundtrip failed for {effect:?}");
        }
    }

    /// Every new M5-W2 (the closing Escape from Violet Hold wave,
    /// src/cards/exp_jail_w2.rs) variant survives the bincode roundtrip
    /// (CardEffectDe → CardEffect via the interned card ids).
    #[test]
    fn w2_effects_serialize_roundtrip() {
        for effect in [
            CardEffect::SliceAndDice,
            CardEffect::TinyPal { ammo: 0 },
            CardEffect::TinyPal { ammo: 4 },
            CardEffect::StaffOfTrickery,
            CardEffect::R4TCatcher,
            CardEffect::IridaSinseeker,
            CardEffect::IridaGetVoid,
            CardEffect::KingOfTheUnderbelly,
            CardEffect::MurlocHolmes,
            CardEffect::TogwaggleShuffleHands,
            CardEffect::MaievBuffDormant,
            CardEffect::ZuramatPlaysDiscarded,
            CardEffect::ColdSnap,
            CardEffect::MindSweeper,
            CardEffect::EnthralledShade,
            CardEffect::TricksyImproviser,
            CardEffect::Judgment,
            CardEffect::ReinforcementAura,
            CardEffect::ScarletBruiser,
            CardEffect::BallAndChain,
            CardEffect::HolyBola,
            CardEffect::SpireSecurity,
            CardEffect::SmuggledShovel,
            CardEffect::ScrambleForGear,
            CardEffect::ReleaseTheBeasts,
            CardEffect::SewerSwimmer,
            CardEffect::Imfernal,
            CardEffect::ImpGangStooge,
            CardEffect::DisguisedDoctor,
            CardEffect::Sawbones,
            CardEffect::BoneFlurry,
            CardEffect::DrinkBlood,
            CardEffect::EmergencySurgery,
            CardEffect::DisguisedWatchman,
            CardEffect::PickPocket,
            CardEffect::JadeGuardians,
            CardEffect::CrowdControl,
            CardEffect::VigilantSentry,
            CardEffect::VioletPunisher,
            CardEffect::BreakoutArchitect,
            CardEffect::NoxiousBribe,
            CardEffect::AlarmOMatic,
            CardEffect::SpitefulChef,
            CardEffect::Annihilation,
            CardEffect::SpireOfSolitude,
            CardEffect::ShadowRounds,
            CardEffect::ScarletRecruiter,
            CardEffect::ThievesTools,
            CardEffect::VoidSoul,
            CardEffect::CodeViolet,
            CardEffect::MoltenGold,
            CardEffect::Frostshatter,
            CardEffect::Stormfury,
            CardEffect::Hexmarshal,
            CardEffect::LethalRecipe,
            CardEffect::DigForFreedom,
            CardEffect::GuardDog,
            CardEffect::BeastTripwire,
            CardEffect::ArcaneTripwire,
            CardEffect::CapturedArchmage,
            CardEffect::FranticForger,
            CardEffect::LowSecurityWing,
            CardEffect::DemonicConfinement,
            CardEffect::WidowsBite,
            CardEffect::WidowsFeast,
            CardEffect::WidowsBanquet,
            CardEffect::Nab,
            CardEffect::VoidBlast,
            CardEffect::CosmicManifestations,
            CardEffect::DefiasWannabe,
            CardEffect::Soothsayer,
            CardEffect::HolyEmbrace,
            CardEffect::RatBurglar,
            CardEffect::DarkBribe,
            CardEffect::AncientAugurPick,
            CardEffect::AncientAugurDeathrattle,
            CardEffect::ArachnathidVenom,
            CardEffect::TruthSeeker,
            CardEffect::ConcealingConfection,
            CardEffect::DisguisedExecutioner,
            CardEffect::GetawayHogdriver,
            CardEffect::EscapeArtist,
            CardEffect::BloodClone,
            CardEffect::GallagioGoon,
            CardEffect::BlackMarketOverseer,
            CardEffect::ActivatedGolem,
            CardEffect::Hellraiser,
            CardEffect::SummonTwoRandomMinionsOfCost { cost: 4 },
        ] {
            let bytes = bincode::serialize(&effect).expect("serialize");
            let back: CardEffect = bincode::deserialize(&bytes).expect("deserialize");
            assert_eq!(back, effect, "roundtrip failed for {effect:?}");
        }
    }

    /// Every new MEND W1 (the Druid class-set wave,
    /// src/cards/exp_cata_w5.rs) variant survives the bincode roundtrip
    /// (CardEffectDe → CardEffect via the interned card ids).
    #[test]
    fn mend_w1_effects_serialize_roundtrip() {
        for effect in [
            CardEffect::RefreshManaIfNoMinionPlayedLastTurn { amount: 3 },
            CardEffect::RestoreAllFriendlyAndSummonTwoRandomCostMinions { heal: 8, cost: 8 },
            CardEffect::DrawAndGainArmorRepeatIfNoMinionPlayedLastTurn { draw: 1, armor: 3 },
            CardEffect::BuffHealthTauntAndDormant { health: 2 },
            CardEffect::AddRandomDragonCostReduced { reduction: 2 },
            CardEffect::GetThreeTreantsAndCarveNatureSpells,
            CardEffect::CastRandomSpellsScaledByHandTurns { base: 1, count: 3 },
        ] {
            let bytes = bincode::serialize(&effect).expect("serialize");
            let back: CardEffect = bincode::deserialize(&bytes).expect("deserialize");
            assert_eq!(back, effect, "roundtrip failed for {effect:?}");
        }
    }

    /// Every new MEND W2 (the Hunter class-set wave,
    /// src/cards/exp_cata_w6.rs) variant survives the bincode roundtrip
    /// (CardEffectDe → CardEffect via the interned card ids).
    #[test]
    fn mend_w2_effects_serialize_roundtrip() {
        for effect in [
            CardEffect::SetCompanionReplacement { bump: 1 },
            CardEffect::SetCompanionReplacementAndDraw { bump: 1, draw: 1 },
            CardEffect::SetCompanionBonus { amount: 1 },
            CardEffect::ReplaceCompanionsAndSummonRandomBeast { bump: 2, cost: 7 },
            CardEffect::SplitDamageAmongAllEnemiesChainOnDeath { amount: 3 },
            CardEffect::BuffFriendlyBeastAndRandomHandBeast {
                attack: 2,
                health: 2,
            },
        ] {
            let bytes = bincode::serialize(&effect).expect("serialize");
            let back: CardEffect = bincode::deserialize(&bytes).expect("deserialize");
            assert_eq!(back, effect, "roundtrip failed for {effect:?}");
        }
    }

    /// Every new MEND W3 (the Mage class-set wave,
    /// src/cards/exp_cata_w7.rs) variant survives the bincode roundtrip
    /// (CardEffectDe → CardEffect via the interned card ids).
    #[test]
    fn mend_w3_effects_serialize_roundtrip() {
        for effect in [
            CardEffect::DealDamageToRandomEnemyMinionExcessToHero {
                amount: 3,
                times: 1,
            },
            CardEffect::SetLeylineDiscount { amount: 1 },
            CardEffect::AddRandomLeylineToHand,
            CardEffect::SummonRandomCostMinionTimes { cost: 5, times: 1 },
            CardEffect::SetLeylineExtraTrigger { amount: 1 },
            CardEffect::DrawCardsCostsLess {
                reduction: 1,
                count: 1,
            },
            CardEffect::GetAllLeylinesAndUpgrade {
                upgrade: LeylineUpgrade::Discount,
            },
            CardEffect::SetLeylineEffectBonus { amount: 1 },
        ] {
            let bytes = bincode::serialize(&effect).expect("serialize");
            let back: CardEffect = bincode::deserialize(&bytes).expect("deserialize");
            assert_eq!(back, effect, "roundtrip failed for {effect:?}");
        }
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
    /// A random card of another class (Pilfer — the other eight classes'
    /// class cards; neutrals are not class cards)
    OtherClass,
    /// A random Dream card (Ysera)
    Dream,
    /// A random Animal Companion (Huffer/Leokk/Misha)
    Companion,
    /// A random Dragon (Core Set W4a)
    Dragon,
    /// A random Mechanical (Core Set W4a)
    Mechanical,
    /// A random spell (Core Set W4a — generic spell discovery simplification)
    Spell,
    /// A random Priest class minion or spell (Blessing of the Moon —
    /// 2025–2026 expansions M1-W1; the real card lets the player choose,
    /// simplified to random)
    PriestCard,
    /// A random minion with a Deathrattle (Avant-Gardening — 2025–2026
    /// expansions M1-W2 dark gifts; "Deathrattle" = a Deathrattle effect or
    /// a death trigger, matching the engine's Deathrattle component model)
    DeathrattleMinion,
    /// A random Undead minion (Rite of Atrocity — 2025–2026 expansions
    /// M1-W2 dark gifts)
    UndeadMinion,
    /// A random Demon costing (5) or more (Jumpscare! — 2025–2026
    /// expansions M1-W2 dark gifts)
    DemonCost5Plus,
    /// A random Choose One card of another class (Symbiosis — 2025–2026
    /// expansions M1-W3; sampled from the fixed
    /// `OTHER_CLASS_CHOOSE_ONE_POOL` table, the real Discover simplified to
    /// random, see §14.2)
    OtherClassChooseOne,
    /// A random Murloc (Gnawing Greenfin — 2025–2026 expansions M1-W4a)
    Murloc,
    /// A random Elemental (Inferno Herald — 2025–2026 expansions M1-W5)
    Elemental,
    /// A random Warrior minion (Shadowflame Suffusion — 2025–2026
    /// expansions M1-W5; the class filter is the MageSpell precedent)
    WarriorMinion,
    /// A random Fel spell (Whispering Stone — 2025–2026 expansions M2-W4a)
    FelSpell,
    /// A random Holy spell (Twilight Mender — 2025–2026 expansions M2-W4a)
    HolySpell,
    /// A random 1-Cost Holy spell that gives +2 or -2 Health (Glade
    /// Ecologist — 2025–2026 expansions M2-W4a)
    HolySpellCost1,
    /// A random Fel Beast (Deathrot Maw — 2025–2026 expansions M2-W4a)
    FelBeast,
    /// A random weapon from another class (Neferset Weaponsmith — 2025–2026
    /// expansions M2-W4a; the class filter is the Pilfer OtherClass
    /// precedent, restricted to weapons)
    WeaponAnotherClass,
    /// A random minion (2025–2026 expansions M3-W2a — the generic
    /// any-minion pool, TIME_033/602/711/859 style filters)
    AnyMinion,
    /// A random 5-Cost minion (2025–2026 expansions M3-W2a — TIME_040
    /// Fading Memory)
    Cost5Minion,
    /// A random Rewind card (2025–2026 expansions M3-W2a — TIME_035 Time
    /// Machine's deathrattle; the fixed REWIND_CARD_IDS table, D2)
    RewindCard,
    /// A random spell from the player's class (2025–2026 expansions
    /// M3-W2a — TIME_002 Aeon Wizard; the class is approximated by the
    /// union of the class card groups, §20)
    ClassSpell,
    /// A random Nature spell (2025–2026 expansions M3-W2a — TIME_033
    /// Druid of Regrowth; the school filter is the FelSpell precedent)
    NatureSpell,
    /// A random weapon (2025–2026 expansions M3-W2a — TIME_034 Stadium
    /// Announcer's "both players equip a random weapon")
    RandomWeapon,
    /// A random 1-Cost minion (2025–2026 expansions M3-W3 — END_013
    /// Brutish Endmaw's "Discover a 1-Cost minion")
    OneCostMinion,
    /// A random minion from another class (2025–2026 expansions M3-W3 —
    /// END_000p Blessing of the Bronze; the class filter is the
    /// `OtherClass` precedent, restricted to minions)
    OtherClassMinion,
    /// A random Dragon costing (3) or less (2025–2026 expansions M4-W4 —
    /// CATA_556 Carrier Whelp "Get a random Dragon that costs (3) or
    /// less"; the active-window cost filter, §26)
    DragonCost3OrLess,
    /// A random Legendary Dragon (2025–2026 expansions M4-W4 — CATA_190t13
    /// Enthrall "Shuffle five random Legendary Dragons into your deck";
    /// the LEGENDARY_CLASSIC pool restricted to the Dragon race, §26)
    LegendaryDragon,
    /// A random minion costing (8) or more (2025–2026 expansions M4-W4 —
    /// CATA_136 Azshara's Triumph "Shuffle 5 random minions into your deck
    /// that cost (8) or more"; the Cost5Minion filter shape, §26)
    MinionCost8OrMore,
    /// A random Paladin spell (2025–2026 expansions M4-W4 — CATA_621
    /// Gelbin's Triumph "Get a random Paladin Aura"; the class-spell pool
    /// restricted to Paladin, §26)
    PaladinSpell,
}
