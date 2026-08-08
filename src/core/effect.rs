//! Card effect definitions — compile-time constant CardEffect and EffectTarget.
//!
//! Phase 2 card effects: damage, card draw, summon, buff.
//! Effects are stored as `Copy` enum constants in `CardDef` and the `Battlecry`/`Deathrattle` components.
use serde::{Deserialize, Serialize};

use crate::core::component::{DarkGiftKind, ImbueClass};

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
}
