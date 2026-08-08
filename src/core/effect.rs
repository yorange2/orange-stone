//! Card effect definitions — compile-time constant CardEffect and EffectTarget.
//!
//! Phase 2 card effects: damage, card draw, summon, buff.
//! Effects are stored as `Copy` enum constants in `CardDef` and the `Battlecry`/`Deathrattle` components.
use serde::{Deserialize, Serialize};

use crate::core::component::{CardType, DarkGiftKind, ImbueClass};

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
    /// Summon a copy of the player's own minion (Bloodthistle Illusionist)
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::ImbueClass;

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
}
