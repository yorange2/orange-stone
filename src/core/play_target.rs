//! Play-time targeting: which `CardEffect`s make the player pick a target.
//!
//! When a card is played from hand, [`crate::rl::env`] asks this table whether
//! the card's effect wants a player-chosen target and over what domain; a
//! `Some(_)` answer expands the play into one legal action per candidate
//! target, `None` yields a single untargeted play.
//!
//! **The match is deliberately exhaustive — no `_` arm.** A new `CardEffect`
//! variant fails to compile until it declares its targeting here. That is the
//! whole point of this module: the previous whitelist lived inline in
//! `play_targets()` with a `_ => Vec::new()` fallback, so every composite
//! effect minted by a card wave silently defaulted to "no target" and then
//! resolved against a random candidate (or fizzled) at execution time. See
//! `docs/play-target-gap-audit.md` for the 64 cards that drift produced.
//!
//! Declaring `Some(_)` here is only half of a targeted effect: the resolution
//! arm in `engine::trigger` must also honour the `explicit_target` it is
//! handed. The two sides are checked together by the card's F5 scenario.

use crate::core::effect::{CardEffect, EffectTarget};

impl CardEffect {
    /// The target the player chooses when this effect is played from hand,
    /// or `None` when the card is played without a targeting step.
    ///
    /// Mirrors the effect's own target domain: an effect that stores an
    /// [`EffectTarget`] reports it, while the fixed-domain effects name theirs
    /// inline.
    #[must_use]
    pub fn play_target(&self) -> Option<EffectTarget> {
        match self {
            // ── Effects that make the player pick ──────────────────────────
            CardEffect::DealDamage { target, .. } => Some(*target),
            CardEffect::DestroyMinion { target } => Some(*target),
            CardEffect::SilenceMinion { target } => Some(*target),
            CardEffect::SetAttack { target, .. } => Some(*target),
            CardEffect::SetHealth { target, .. } => Some(*target),
            CardEffect::RestoreHealth { target, .. } => Some(*target),
            CardEffect::FreezeCharacter { target } => Some(*target),
            CardEffect::ReturnToHand { target } => Some(*target),
            CardEffect::IncreaseCost { target, .. } => Some(*target),
            CardEffect::GainStats { target, .. } => Some(*target),
            CardEffect::GainArmor { target, .. } => Some(*target),
            CardEffect::FullHeal { target } => Some(*target),
            CardEffect::GrantWindfury { target } => Some(*target),
            CardEffect::DoubleAttack { target } => Some(*target),
            CardEffect::DoubleHealth { target } => Some(*target),
            CardEffect::SetAttackToHealth { target } => Some(*target),
            CardEffect::TempDebuff { target, .. } => Some(*target),
            CardEffect::GainStatsAndTaunt { target, .. } => Some(*target),
            CardEffect::DestroyAndGainStats { target, .. } => Some(*target),
            CardEffect::SwapAttackAndHealth { target } => Some(*target),
            CardEffect::GrantDivineShield { target } => Some(*target),
            CardEffect::OutcastDamage { target, .. } => Some(*target),
            CardEffect::DamageAndDrawIfSurvives { target, .. } => Some(*target),
            CardEffect::DamageAndDrawIfKilled { target, .. } => Some(*target),
            CardEffect::DamageAndGainArmor { target, .. } => Some(*target),
            CardEffect::TransformToMinion { .. } => Some(EffectTarget::AnyMinion),
            CardEffect::GrantDeathrattleToTarget { .. } => Some(EffectTarget::AnyMinion),
            CardEffect::GainStatsTauntAndDeathrattle { .. } => Some(EffectTarget::AnyMinion),
            CardEffect::DamageAndSummon { target, .. } => Some(*target),
            CardEffect::DamageAndSummonVoidwalkers { target, .. } => Some(*target),
            CardEffect::DamageAndAddToHand { .. } => Some(EffectTarget::AnyMinion),
            CardEffect::DamageAndSummonCopyIfKilled { .. } => Some(EffectTarget::AnyMinion),
            CardEffect::BuffAndSummonRandomCost2 => Some(EffectTarget::AnyMinion),
            CardEffect::FreezeAndDiscoverSpell => Some(EffectTarget::AnyCharacter),
            CardEffect::GainStatsAndDraw { target, .. } => Some(*target),
            CardEffect::DamageUndamaged { .. } => Some(EffectTarget::AnyMinion),
            CardEffect::DamageMinionAndSelfHero { .. } => Some(EffectTarget::AnyMinion),
            CardEffect::RestoreHealthAndDraw { target, .. } => Some(*target),
            CardEffect::BattleToTheDeath => Some(EffectTarget::AnyEnemyMinion),
            CardEffect::DestroyMinionAndSelfDamage => Some(EffectTarget::AnyMinion),
            CardEffect::DamageAndAddRandomSpell { target, .. } => Some(*target),
            CardEffect::FreezeAndSummonElementals => Some(EffectTarget::AnyEnemy),
            CardEffect::DamageAndFreeze { target, .. } => Some(*target),
            CardEffect::DamageAndGainAttack { target, .. } => Some(*target),

            CardEffect::DealHeroAttackDamage { target } => Some(*target),
            CardEffect::GrantAttackAndImmune { target, .. } => Some(*target),
            CardEffect::GainStatsThisTurn { target, .. } => Some(*target),
            CardEffect::DestroyAndHeal { target, .. } => Some(*target),
            CardEffect::GrantCharge { target, .. } => Some(*target),
            // Demonfire's resolution collects both sides' minions.
            CardEffect::Demonfire { .. } => Some(EffectTarget::AnyMinion),
            // Mortal Strike resolves over AnyEnemy — narrower than the official
            // "any character"; widening domains is tracked separately.
            CardEffect::MortalStrike { .. } => Some(EffectTarget::AnyEnemy),

            CardEffect::DealDamageGainArmorIfKilled { target, .. } => Some(*target),
            CardEffect::DealDamageIfQuestPlayed { target, .. } => Some(*target),
            CardEffect::DealDamageSameType { target, .. } => Some(*target),
            CardEffect::DealDamageSetNextBeastDiscount { target, .. } => Some(*target),
            CardEffect::DestroyFriendlyMinionAddBones { target, .. } => Some(*target),
            CardEffect::GainArmorDealDamageEqual { target, .. } => Some(*target),
            CardEffect::GainStatsAndGrantRush { target, .. } => Some(*target),
            CardEffect::GiveBuffSameType { target, .. } => Some(*target),
            CardEffect::AmphibianSpiritBuff { .. } => Some(EffectTarget::AnyMinion),
            CardEffect::BuffMinionReturnIfSpellsCast { .. } => Some(EffectTarget::AnyMinion),
            // Shadowflame Suffusion's bare "Deal 2 damage" officially reaches any
            // character; the resolution's domain is enemy-only (tracked as
            // narrowness, not widened here).
            CardEffect::DamageAndDiscoverWarriorWithGift { .. } => Some(EffectTarget::AnyEnemy),
            CardEffect::DamageAndSummonWolfIfKilled { .. } => Some(EffectTarget::AnyMinion),
            CardEffect::DamageMinionOwnerDraws { .. } => Some(EffectTarget::AnyMinion),
            CardEffect::DamageMinionWithMoonLifesteal { .. } => Some(EffectTarget::AnyMinion),
            CardEffect::DealDamageEqualSelfAttack => Some(EffectTarget::AnyCharacter),
            CardEffect::DealDamageIfImbuedTwice { .. } => Some(EffectTarget::AnyMinion),
            CardEffect::DestroyFriendlyMinionGainArmor { .. } => Some(EffectTarget::FriendlyMinion),
            CardEffect::GiveMinionStatsRushIfHeroPowerUsed { .. } => {
                Some(EffectTarget::FriendlyMinion)
            }
            // Air Support's resolution scans the owner's play zone (hero included);
            // the offer is the minion subset of it.
            CardEffect::GrantMegaWindfuryCantAttackHeroes => Some(EffectTarget::FriendlyMinion),
            CardEffect::ReturnFriendlyMinionSummonSpider => Some(EffectTarget::FriendlyMinion),
            // Gruesome Nightmare's official domain also covers the hand — the
            // engine keeps the battlefield-side approximation (ledger §26).
            CardEffect::SetAttackEqualToSource => Some(EffectTarget::FriendlyMinion),
            CardEffect::SetStatsByFriendlyTarget { .. } => Some(EffectTarget::AnyMinion),
            CardEffect::StealHealthThreeTimes { .. } => Some(EffectTarget::AnyEnemyMinion),
            CardEffect::SummonTreantsAttackMinion => Some(EffectTarget::AnyMinion),

            // Icicle — the engine's domain is enemy minions.
            CardEffect::FreezeOrDamage { .. } => Some(EffectTarget::AnyEnemyMinion),
            CardEffect::DealDamageAndDraw { target, .. } => Some(*target),
            // Cone of Cold freezes the chosen minion and its neighbours;
            // the engine's domain is enemy minions (official: any minion).
            CardEffect::FreezeAdjacent => Some(EffectTarget::AnyEnemyMinion),
            // Faceless Manipulator copies a friendly minion here (official:
            // any minion).
            CardEffect::CopyMinionStats => Some(EffectTarget::FriendlyMinion),
            CardEffect::DrawAndDamageByCost => Some(EffectTarget::AnyEnemy),
            // Cabal Shadow Priest — an enemy minion within the card's own
            // Attack limit.
            CardEffect::TakeControlAttackLE { max_attack } => {
                Some(EffectTarget::EnemyMinionAttackLE(*max_attack))
            }
            CardEffect::DestroyAndGainHealth => Some(EffectTarget::AnyEnemyMinion),
            // Shadow Madness — "an enemy minion with 3 or less Attack".
            CardEffect::TakeControlUntilEndOfTurn => Some(EffectTarget::EnemyMinionAttackLE(3)),
            // Mind Control.
            CardEffect::TakeControl => Some(EffectTarget::AnyEnemyMinion),
            CardEffect::ReturnFriendlyToHandAndReduceCost { .. } => {
                Some(EffectTarget::FriendlyMinion)
            }
            CardEffect::AdjacentDamage => Some(EffectTarget::AnyEnemyMinion),
            // Master of Disguise cannot cloak itself — the resolution
            // excludes the source, so the offer must too.
            CardEffect::GrantStealth => Some(EffectTarget::OtherFriendlyMinion),
            // Rockbiter Weapon buffs the hero only in this engine (official:
            // any friendly character).
            CardEffect::GainHeroAttack { .. } => Some(EffectTarget::FriendlyHero),
            // Shadowflame sacrifices a friendly minion.
            CardEffect::DestroyAndAOE { .. } => Some(EffectTarget::FriendlyMinion),
            CardEffect::Corrupt => Some(EffectTarget::AnyEnemyMinion),
            CardEffect::VioletPunisher => Some(EffectTarget::AnyEnemyMinion),
            CardEffect::DealDamageSummonCinders { .. } => Some(EffectTarget::AnyEnemy),
            CardEffect::SetStatsAndCantAttackHeroesThisTurn { .. } => {
                Some(EffectTarget::FriendlyMinion)
            }
            CardEffect::DealDamageEnemyMinionEqualToSourceHealth => {
                Some(EffectTarget::AnyEnemyMinion)
            }
            CardEffect::RestoreHealthEqualToSourceHealth => Some(EffectTarget::AnyCharacter),
            CardEffect::ImprisonEnemyMinion => Some(EffectTarget::AnyEnemyMinion),
            CardEffect::DealDamageEnemyMinionIfHeroHealthChanged { .. } => {
                Some(EffectTarget::AnyEnemyMinion)
            }
            CardEffect::DealDamageAndDrawExcess { .. } => Some(EffectTarget::AnyEnemyMinion),

            // Sewer Swimmer triggers a Deathrattle, so only minions that have
            // one are legal targets.
            CardEffect::SewerSwimmer => Some(EffectTarget::FriendlyMinionWithDeathrattle),
            // Eternus steals an enemy minion no healthier than itself.
            CardEffect::TakeControlEnemyMinionHealthLE => {
                Some(EffectTarget::EnemyMinionHealthLESource)
            }

            // ── Everything else: played without a targeting step ───────────
            // (AoE, summons, draws, Discover, self-buffs, triggered-only
            // effects — plus, for now, the audit's outstanding cards.)
            CardEffect::AbsorbDivineShields { .. }
            | CardEffect::ActivatedGolem
            | CardEffect::AddAllDreamCards
            | CardEffect::AddCardToHand { .. }
            | CardEffect::AddCardToHandCount { .. }
            | CardEffect::AddCopiesOfLastPlayedCards { .. }
            | CardEffect::AddDragonBreathScaledByAttack
            | CardEffect::AddFiveRandomCards
            | CardEffect::AddMoonfireAndStarfireWithSpellDamage
            | CardEffect::AddNefariansCreation { .. }
            | CardEffect::AddRandomBattlecryMinion
            | CardEffect::AddRandomBeastCostLess { .. }
            | CardEffect::AddRandomBeastsToBottomDeckWithStats { .. }
            | CardEffect::AddRandomCardToHand { .. }
            | CardEffect::AddRandomCardToHandCount { .. }
            | CardEffect::AddRandomCostMinionCostsHealth { .. }
            | CardEffect::AddRandomCostMinionMarkedTurnDiscount { .. }
            | CardEffect::AddRandomCostMinionWithDarkGift { .. }
            | CardEffect::AddRandomDeathrattleMinionCostsLess
            | CardEffect::AddRandomDeckMinionToHand
            | CardEffect::AddRandomDragonCostLE3
            | CardEffect::AddRandomDragonCostReduced { .. }
            | CardEffect::AddRandomDruidSpell
            | CardEffect::AddRandomFelSpellsCostHealth { .. }
            | CardEffect::AddRandomFireSpellCostsLess { .. }
            | CardEffect::AddRandomHolyAndShadowSpell
            | CardEffect::AddRandomHolySpellCost1
            | CardEffect::AddRandomHolySpellCostReduced { .. }
            | CardEffect::AddRandomLegendaryMinionCostReduced { .. }
            | CardEffect::AddRandomLeylineToHand
            | CardEffect::AddRandomMageSpells { .. }
            | CardEffect::AddRandomMaskCombo { .. }
            | CardEffect::AddRandomMinionCostsLess { .. }
            | CardEffect::AddRandomMinionsCostEqualAttack { .. }
            | CardEffect::AddRandomMultiTribeMinion
            | CardEffect::AddRandomNagaCost1
            | CardEffect::AddRandomOneCostCard
            | CardEffect::AddRandomOneCostMinion
            | CardEffect::AddRandomOneCostSpell
            | CardEffect::AddRandomOtherClassCard
            | CardEffect::AddRandomOtherClassChooseOneCard
            | CardEffect::AddRandomOtherClassSpells { .. }
            | CardEffect::AddRandomOutcastCardNextCheaper
            | CardEffect::AddRandomPaladinAura
            | CardEffect::AddRandomPirateToHand
            | CardEffect::AddRandomRewindCardToHand
            | CardEffect::AddRandomShadowSpell
            | CardEffect::AddRandomShamanSpell
            | CardEffect::AddRandomShatterCardToHand
            | CardEffect::AddRandomSpellCostsLess { .. }
            | CardEffect::AddRandomSpellToOpponentDeckTop
            | CardEffect::AddRandomSpellsFromClass { .. }
            | CardEffect::AddRandomTauntBuffed
            | CardEffect::AddRandomWeaponAnotherClassComboAttack { .. }
            | CardEffect::AddSelfToDeckBottomCost { .. }
            | CardEffect::AddTemporaryRandomMinionsCost { .. }
            | CardEffect::AddTwoRandomSpellsSameCost { .. }
            | CardEffect::AlarmOMatic
            | CardEffect::AmirdrassilActivate
            | CardEffect::AncientAugurDeathrattle
            | CardEffect::AncientAugurPick
            | CardEffect::Annihilation
            | CardEffect::AoeDamageAndDraw { .. }
            | CardEffect::AoeDamageAndHealFriendly { .. }
            | CardEffect::ApplyDarkGift { .. }
            | CardEffect::ArachnathidVenom
            | CardEffect::AratorDoubleSilverHandRecruits
            | CardEffect::ArcaneTripwire
            | CardEffect::ArmAccelerationAura
            | CardEffect::AttachAttackDraw { .. }
            | CardEffect::AttackRandomEnemyMinionExcess
            | CardEffect::AttackTwoRandomEnemyMinionsIfCostLE { .. }
            | CardEffect::AwakenImprisonedMinion
            | CardEffect::AyaUpgradeCoins { .. }
            | CardEffect::AzalinaStartOfGame
            | CardEffect::BallAndChain
            | CardEffect::BeastTripwire
            | CardEffect::BeatrixStartOfGame
            | CardEffect::BlackMarketOverseer
            | CardEffect::BloodClone
            | CardEffect::BoneFlurry
            | CardEffect::BothPlayersDiscardRandomCard
            | CardEffect::BothPlayersEquipRandomWeaponBuffOurs { .. }
            | CardEffect::BreakoutArchitect
            | CardEffect::BuffAllBeastsEverywhere { .. }
            | CardEffect::BuffAllFriendlyMinionsShuffleShreds { .. }
            | CardEffect::BuffAllHandMinions { .. }
            | CardEffect::BuffAnotherRandomFriendlyDragon { .. }
            | CardEffect::BuffFirstUndeadPlayedEachTurn { .. }
            | CardEffect::BuffFriendlyBeastAndRandomHandBeast { .. }
            | CardEffect::BuffFriendlyMinionsDiscardBonus { .. }
            | CardEffect::BuffHandAndDeckMinions { .. }
            | CardEffect::BuffHandMinions { .. }
            | CardEffect::BuffHandMinionsAndWeapons { .. }
            | CardEffect::BuffHandMinionsWithCorpses { .. }
            | CardEffect::BuffHealthTauntAndDormant { .. }
            | CardEffect::BuffSpellDamageHandAndDeck
            | CardEffect::BuffTauntHandMinions { .. }
            | CardEffect::BuffThreeDifferentRaces { .. }
            | CardEffect::BuffTopDeckMinions { .. }
            | CardEffect::BuffWeapon { .. }
            | CardEffect::BuffWeaponDurabilityIfBeast { .. }
            | CardEffect::BygoneEchoesSummon
            | CardEffect::CapturedArchmage
            | CardEffect::CardsCostOneThisGame
            | CardEffect::CastAbsorbedSpell
            | CardEffect::CastHighestCostSpellFromHand
            | CardEffect::CastRandomNatureSpells { .. }
            | CardEffect::CastRandomSpellFromDeckCostLE { .. }
            | CardEffect::CastRandomSpellSameCostOtherClass
            | CardEffect::CastRandomSpellsScaledByHandTurns { .. }
            | CardEffect::CastShredFromDeckGainStats { .. }
            | CardEffect::CastShredFromDeckSummonCopy
            | CardEffect::ChanceDraw { .. }
            | CardEffect::CharityCopiesDiedThisTurn { .. }
            | CardEffect::ChooseCataclysms
            | CardEffect::ChooseHandCard
            | CardEffect::ChronikarHeroAttackBuff
            | CardEffect::ChronikarRebuff
            | CardEffect::ChronogorDrawsHighestLowest
            | CardEffect::CodeViolet
            | CardEffect::ColdSnap
            | CardEffect::ColossalArmDestroyRight { .. }
            | CardEffect::ConcealingConfection
            | CardEffect::CopyCastSpellToOtherPlayerHand
            | CardEffect::CopyEnemyDeckCardOnSelfAttack
            | CardEffect::CopyLowestCostBeastInHand
            | CardEffect::CopyLowestCostEnemyHandCard
            | CardEffect::CopyRandomEnemyDeckCards { .. }
            | CardEffect::CopyRandomEnemyHandCard { .. }
            | CardEffect::CopyRandomHandElementalOrDragon
            | CardEffect::CopyRandomHandMinion
            | CardEffect::CopyRightmostEnemyHandCardOrIncreaseCost
            | CardEffect::CosmicManifestations
            | CardEffect::CrowdControl
            | CardEffect::DamageAllEnemiesByAttack
            | CardEffect::DamageAllEnemyMinionsAndFreeze { .. }
            | CardEffect::DamageAllMinions { .. }
            | CardEffect::DamageAllMinionsAndAddCardToHand { .. }
            | CardEffect::DamageAllMinionsAndGainHeroAttack { .. }
            | CardEffect::DamageAllMinionsDrawPerDeath { .. }
            | CardEffect::DamageAllMinionsIfHoldingDragon { .. }
            | CardEffect::DamageAllMinionsRepeatDescending { .. }
            | CardEffect::DamageAllMinionsScaledByBoard { .. }
            | CardEffect::DamageAllOtherFriendlyMinions { .. }
            | CardEffect::DamageAllOtherMinions { .. }
            | CardEffect::DamageAndBuffFriendlyIfKilled { .. }
            | CardEffect::DamageAndDiscardSpellMore { .. }
            | CardEffect::DamageAndDrawIfHandEmpty { .. }
            | CardEffect::DamageAndDrawMinionIfHoldingCostGE { .. }
            | CardEffect::DamageAndDrawTwoIfSurvives { .. }
            | CardEffect::DamageAndGainArmorIfMinionPlayedWhileHeld { .. }
            | CardEffect::DamageDamagedMinionReturnIfExcess { .. }
            | CardEffect::DamageEnemyHeroAndHealSelf { .. }
            | CardEffect::DamageFreezeAllAndSummon { .. }
            | CardEffect::DamageIfHoldingSpell5Plus { .. }
            | CardEffect::DamageLowestHealthEnemyTwice { .. }
            | CardEffect::DamageMinionDrawIfSurvivesSummonIfDies { .. }
            | CardEffect::DamageMinionEternalFirebolt { .. }
            | CardEffect::DamageMinionGiveHeroAttack { .. }
            | CardEffect::DamageMinionHealEnemyHeroIfKilled { .. }
            | CardEffect::DamageMinionReduceHandCostByExcess { .. }
            | CardEffect::DamageMinionScaledByFallen { .. }
            | CardEffect::DamagePlayedMinion { .. }
            | CardEffect::DamagePlayedMinionAndExcess { .. }
            | CardEffect::DamageRandomEnemyMinionHoldingCostGE { .. }
            | CardEffect::DamageSelfHero { .. }
            | CardEffect::DamageSelfMinion { .. }
            | CardEffect::DamageTwoDrawIfKilled { .. }
            | CardEffect::DarkBribe
            | CardEffect::DealArmorDamage { .. }
            | CardEffect::DealDamageAllEnemiesIfControllingAura { .. }
            | CardEffect::DealDamageAllEnemyMinionsSetMinionsCostMore { .. }
            | CardEffect::DealDamageAndBuffFriendlyElementals { .. }
            | CardEffect::DealDamageAndDamageAllEnemies { .. }
            | CardEffect::DealDamageAndDamageRandomEnemyMinion { .. }
            | CardEffect::DealDamageAndImbue { .. }
            | CardEffect::DealDamageAndReturnToHand { .. }
            | CardEffect::DealDamageAndSummon { .. }
            | CardEffect::DealDamageAndSummonIfKilled { .. }
            | CardEffect::DealDamageEqualDragonBreath
            | CardEffect::DealDamageFriendlyMinionToRandomEnemy { .. }
            | CardEffect::DealDamageImprovedByShuffles { .. }
            | CardEffect::DealDamageLeftRightOutcastAgain { .. }
            | CardEffect::DealDamageLowestHealthEnemyRepeated { .. }
            | CardEffect::DealDamageOrDamageAllEnemiesIfCopyPlayed { .. }
            | CardEffect::DealDamagePrimaryAndSplash { .. }
            | CardEffect::DealDamageRandomEnemies { .. }
            | CardEffect::DealDamageRandomly { .. }
            | CardEffect::DealDamageSplitAmongAllEnemies { .. }
            | CardEffect::DealDamageSplitAmongAllEnemiesShuffleShreds { .. }
            | CardEffect::DealDamageSplitAmongEnemiesIfFireSpell { .. }
            | CardEffect::DealDamageToAllEnemyMinions { .. }
            | CardEffect::DealDamageToLeftRightEnemyMinions { .. }
            | CardEffect::DealDamageToRandomEnemyMinionExcessToHero { .. }
            | CardEffect::DealDamageToTwo { .. }
            | CardEffect::DealDamageToTwoAndFreeze { .. }
            | CardEffect::DealDamageToTwoScaledByHandTurns { .. }
            | CardEffect::DealSelfAttackDamage { .. }
            | CardEffect::DeathrattleDamageAllEnemiesTurnScaled { .. }
            | CardEffect::DebuffRandomHandMinionBoth { .. }
            | CardEffect::DefiasWannabe
            | CardEffect::DemonicConfinement
            | CardEffect::DestroyAdjacent { .. }
            | CardEffect::DestroyAllEnemySecretsAndDraw { .. }
            | CardEffect::DestroyAllEnemySecretsAndGainStats { .. }
            | CardEffect::DestroyAllExceptOne
            | CardEffect::DestroyAllMinionsAttackGE { .. }
            | CardEffect::DestroyAllMinionsOpponentPlayedLastTurn
            | CardEffect::DestroyAllMinionsWith4OrLessAttack
            | CardEffect::DestroyAllOtherMinionsAndDiscardHand
            | CardEffect::DestroyCrystalGainCrystalsLater { .. }
            | CardEffect::DestroyDeckCardsCostLE2Both
            | CardEffect::DestroyDeckTop { .. }
            | CardEffect::DestroyEnemyLocation
            | CardEffect::DestroyFriendlyWispDraw { .. }
            | CardEffect::DestroyHeldKingLlaneAndHalveEnemyHealth
            | CardEffect::DestroyHighestAttackEnemy
            | CardEffect::DestroyHighestHealthEnemyMinion
            | CardEffect::DestroyLegendaryMinion
            | CardEffect::DestroyLowestAttackEnemy
            | CardEffect::DestroyManaCrystal
            | CardEffect::DestroyMinionAndGainItsStats { .. }
            | CardEffect::DestroyMinionAndSummonRandomCost { .. }
            | CardEffect::DestroyMinionSummonRandomSameCost { .. }
            | CardEffect::DestroyRandomEnemyMinion
            | CardEffect::DestroyRandomEnemyMinionLocationWeapon
            | CardEffect::DestroyRandomEnemySecret
            | CardEffect::DestroyTopCardDiscoverSameRarity
            | CardEffect::DestroyTopFiveEnemyDeckIfOwnEmpty
            | CardEffect::DestroyWeapon
            | CardEffect::DestroyWeaponAndDealAttackToEnemies
            | CardEffect::DestroyWeaponAndDraw
            | CardEffect::DevourTwoEnemyHandCardsAndDormant
            | CardEffect::DigForFreedom
            | CardEffect::DiscardHand
            | CardEffect::DiscardHandAndAddInfiniteBanana
            | CardEffect::DiscardHighestCostCard
            | CardEffect::DiscardLinkedDrawnCard
            | CardEffect::DiscardRandomCard
            | CardEffect::DiscardRandomEnemyHandCard
            | CardEffect::DiscardTwoRandomCards
            | CardEffect::DiscoverAnySpell
            | CardEffect::DiscoverArcaneSpellsReduced { .. }
            | CardEffect::DiscoverComboBattlecryStealthWithDarkGift
            | CardEffect::DiscoverCostCardGainTempMana { .. }
            | CardEffect::DiscoverDeckAndEnemyHandCardCopy
            | CardEffect::DiscoverDeckCard
            | CardEffect::DiscoverDeckCardOthersBottom
            | CardEffect::DiscoverDeckMinionWithDarkGift
            | CardEffect::DiscoverDeckTop3
            | CardEffect::DiscoverDemonGE5AndSetNextDemonCostOne
            | CardEffect::DiscoverDemonWithDarkGiftCopy
            | CardEffect::DiscoverDragonWithDarkGift
            | CardEffect::DiscoverEnemyDeckMinionCopy { .. }
            | CardEffect::DiscoverEnemyDeckTop
            | CardEffect::DiscoverEnemyHandCardCopy
            | CardEffect::DiscoverMinionReduceHandCostsIfDeckNoMinions { .. }
            | CardEffect::DiscoverOneCostSpell
            | CardEffect::DiscoverPaladinMechPastGiveStats { .. }
            | CardEffect::DiscoverPool { .. }
            | CardEffect::DiscoverSpellAndHealCost
            | CardEffect::DiscoverSpellKeepOrTop
            | CardEffect::DiscoverSpellReduceHandSpells { .. }
            | CardEffect::DiscoverUndeadWithCorpseGift { .. }
            | CardEffect::DiscoverWildGodIfImbued4
            | CardEffect::DiscoverWithDarkGift { .. }
            | CardEffect::DiscoverWithDarkGiftCostReduction { .. }
            | CardEffect::DisguisedDoctor
            | CardEffect::DisguisedExecutioner
            | CardEffect::DisguisedWatchman
            | CardEffect::DracorexSplash
            | CardEffect::Drain { .. }
            | CardEffect::DrawAndGainArmorRepeatIfNoMinionPlayedLastTurn { .. }
            | CardEffect::DrawAndGainStats { .. }
            | CardEffect::DrawAndReduceCostRepeated { .. }
            | CardEffect::DrawAndSummonDreadseed { .. }
            | CardEffect::DrawAndSummonLeeches { .. }
            | CardEffect::DrawBeastAndImbue
            | CardEffect::DrawBeastDragonMurloc
            | CardEffect::DrawBottomCards { .. }
            | CardEffect::DrawCard { .. }
            | CardEffect::DrawCardAndReduceCost { .. }
            | CardEffect::DrawCardByRace { .. }
            | CardEffect::DrawCardByType { .. }
            | CardEffect::DrawCardLinkDeathrattle
            | CardEffect::DrawCardOutcast { .. }
            | CardEffect::DrawCardsCostsLess { .. }
            | CardEffect::DrawCardsOfDifferentCosts { .. }
            | CardEffect::DrawDeathrattleMinionCostLE { .. }
            | CardEffect::DrawDeckSpellAndAddRandomSpell
            | CardEffect::DrawDragonsReduced { .. }
            | CardEffect::DrawForBoth
            | CardEffect::DrawIfImbuedTwice { .. }
            | CardEffect::DrawIfMinionPlayedBefore
            | CardEffect::DrawIfUnspentMana
            | CardEffect::DrawKindredAndActivator
            | CardEffect::DrawMinionAndBuffHandMinionsHealth { .. }
            | CardEffect::DrawMinionBuffArmorIfAttackGE { .. }
            | CardEffect::DrawMinionCostGE { .. }
            | CardEffect::DrawMinionSummonDivineShieldCopy
            | CardEffect::DrawMinionsAndBuffHandMinions { .. }
            | CardEffect::DrawMinionsDifferentTypesBuff { .. }
            | CardEffect::DrawMinionsOfEachCost { .. }
            | CardEffect::DrawOrResurrectBwonsamdiAndGrantBoon { .. }
            | CardEffect::DrawPerDamagedFriendlyCharacter
            | CardEffect::DrawRandomMinionGiveStats { .. }
            | CardEffect::DrawSpellCostGE { .. }
            | CardEffect::DrawSpellGiveSpellDamage { .. }
            | CardEffect::DrawTwoReduceRandomCost { .. }
            | CardEffect::DrawUndeadAndImbueTwice
            | CardEffect::DrawUntilHandFull
            | CardEffect::DrawUntilHandSize { .. }
            | CardEffect::DrawZeroAttackMinion
            | CardEffect::DrinkBlood
            | CardEffect::EatDeckMinionGainStats
            | CardEffect::EliseCraftLocation
            | CardEffect::EmergencySurgery
            | CardEffect::EmptyOpponentHand
            | CardEffect::EnemySpellsCostZero
            | CardEffect::EnthralledShade
            | CardEffect::EquipDaggerOrBuffWeapon
            | CardEffect::EquipSwordIfHoldingDragon
            | CardEffect::EquipWeapon { .. }
            | CardEffect::EscapeArtist
            | CardEffect::EshoDeckCheckBuffEverywhere { .. }
            | CardEffect::FelfireBlazeTrigger { .. }
            | CardEffect::FillBoardRandomDragonsHealHeroSkipNextTurn
            | CardEffect::FillEnemyBoardWithRandomCost1Minions
            | CardEffect::FillHandWithEnemyDeckCopies { .. }
            | CardEffect::FillHandWithMinion { .. }
            | CardEffect::FillHandWithRandomDragons { .. }
            | CardEffect::FillHandWithRandomTemporarySpells
            | CardEffect::FillHandWithRandomUndeadCostHealth
            | CardEffect::FillOpponentHandWithCoins
            | CardEffect::ForceEnemyMinionsAttackThis
            | CardEffect::FordragonBuff
            | CardEffect::FranticForger
            | CardEffect::FreezeMinionAndNeighborsDestroyDamaged
            | CardEffect::Frostshatter
            | CardEffect::FullHealAndTaunt { .. }
            | CardEffect::FullHealMinionAndDraw
            | CardEffect::GainArmorAndDraw { .. }
            | CardEffect::GainArmorAndDrawOnHeroAttack { .. }
            | CardEffect::GainArmorAndSelfAttack { .. }
            | CardEffect::GainArmorAndSummonDeckMinion { .. }
            | CardEffect::GainArmorAndSummonRandomCost { .. }
            | CardEffect::GainArmorAndSummonTwoBeastsForOpponent { .. }
            | CardEffect::GainArmorPerWisp { .. }
            | CardEffect::GainArmorSummonCostTaunt { .. }
            | CardEffect::GainAttackEqualSpellCost
            | CardEffect::GainAttackEqualToWeapon
            | CardEffect::GainAttackFirstSpellDamageThisTurn { .. }
            | CardEffect::GainDeadMinionAttack
            | CardEffect::GainDeathrattleOfDiedThisTurn
            | CardEffect::GainDivineShieldLifestealIfHoldingSpellGE { .. }
            | CardEffect::GainHealthIfFullHealth { .. }
            | CardEffect::GainHealthIfHeroPowerUsed { .. }
            | CardEffect::GainHeroAttackAndBuffHandMinionsIfDeckNoMinions { .. }
            | CardEffect::GainHeroAttackAndDraw { .. }
            | CardEffect::GainHeroAttackArmorIfHoldingGift { .. }
            | CardEffect::GainImmuneThisTurn { .. }
            | CardEffect::GainManaCrystal { .. }
            | CardEffect::GainManaCrystalBoth { .. }
            | CardEffect::GainManaCrystalsMatchOpponent
            | CardEffect::GainManaThisTurn { .. }
            | CardEffect::GainPoisonousToFriendlyUndead
            | CardEffect::GainRush { .. }
            | CardEffect::GainStatsAllOtherFriendlyMinions { .. }
            | CardEffect::GainStatsAndCopyToColossalMain { .. }
            | CardEffect::GainStatsAndDrawIfNatureSpellCast { .. }
            | CardEffect::GainStatsAndGrantDivineShield { .. }
            | CardEffect::GainStatsAndGrantLifesteal { .. }
            | CardEffect::GainStatsAndGrantWindfury { .. }
            | CardEffect::GainStatsAndSummonCopyIfHeroHealthLE { .. }
            | CardEffect::GainStatsAndTauntAllFriendly { .. }
            | CardEffect::GainStatsElusiveAndSummonCopy { .. }
            | CardEffect::GainStatsEqualFireSpellCost
            | CardEffect::GainStatsIfHealedThisTurn { .. }
            | CardEffect::GainStatsIfHeroDamagedThisTurn { .. }
            | CardEffect::GainStatsIfHeroPowerUsed { .. }
            | CardEffect::GainStatsIfOwnSecret { .. }
            | CardEffect::GainStatsOfRandomLegendaryBeast
            | CardEffect::GainStatsPerDamagedMinion { .. }
            | CardEffect::GainStatsPerFriendlyMinion { .. }
            | CardEffect::GainStatsPerFriendlyMinionTargeted
            | CardEffect::GainStatsPerHandCard { .. }
            | CardEffect::GainStatsPerTurnTaken { .. }
            | CardEffect::GainTauntAndDivineShieldIfHoldingDragon
            | CardEffect::GainTempManaOrPermanentIfSpent { .. }
            | CardEffect::GainWeaponAttackIfHoldingGift { .. }
            | CardEffect::GallagioGoon
            | CardEffect::GetAllLeylinesAndUpgrade { .. }
            | CardEffect::GetHolySpellsRestoreHealthEqualCosts
            | CardEffect::GetOrSummonTauntWhelpsIfSpent { .. }
            | CardEffect::GetPupilAndDiscoverSpellCostGE { .. }
            | CardEffect::GetRandomOtherClassMinionCostsLess { .. }
            | CardEffect::GetThreeRandomSpellsFromPastTracked
            | CardEffect::GetThreeTreantsAndCarveNatureSpells
            | CardEffect::GetawayHogdriver
            | CardEffect::GiveBuffAndSummonDeathrattle { .. }
            | CardEffect::GiveBuffDifferentTypeMinions { .. }
            | CardEffect::GiveBuffOtherMinionsAttackLE { .. }
            | CardEffect::GiveCardToOpponent { .. }
            | CardEffect::GiveCardsToOpponent { .. }
            | CardEffect::GiveCoin
            | CardEffect::GiveHeroImmuneThisTurn
            | CardEffect::GiveNextMurlocDivineShield
            | CardEffect::GiveOpponentManaCrystal { .. }
            | CardEffect::GiveOpponentSabotage
            | CardEffect::GiveOtherFriendlyMinionsRush
            | CardEffect::GiveRandomFriendlyMinionAttack { .. }
            | CardEffect::GodfreyStartOfGame
            | CardEffect::GrantAdjacentSpellDamage { .. }
            | CardEffect::GrantAdjacentStatsAndDivineShield { .. }
            | CardEffect::GrantAdjacentTaunt
            | CardEffect::GrantAttackToRandomFriendly
            | CardEffect::GrantDeathrattleAll { .. }
            | CardEffect::GrantDeathrattleSummon { .. }
            | CardEffect::GrantDeathrattleSummonOwnCost
            | CardEffect::GrantDeathrattleSummonRandomCostMinion { .. }
            | CardEffect::GrantDeathrattleSummonRandomHandMinion
            | CardEffect::GrantDivineShieldAllFriendly
            | CardEffect::GrantDivineShieldAndBuffHandMinionsHealth { .. }
            | CardEffect::GrantDivineShieldOrGainStats { .. }
            | CardEffect::GrantHeroDivineShield
            | CardEffect::GrantHeroLifestealThisTurn
            | CardEffect::GrantKeyword { .. }
            | CardEffect::GrantPoisonousThisTurn
            | CardEffect::GrantRandomBonusEffect
            | CardEffect::GrantRandomBonusEffectAndDeathrattle
            | CardEffect::GrantRandomBonusEffects { .. }
            | CardEffect::GrantRandomFriendlyDivineShieldTaunt
            | CardEffect::GrantRandomFriendlyMinionAttack { .. }
            | CardEffect::GrantWeaponDeathrattleAllEnemies { .. }
            | CardEffect::GrimyCoin
            | CardEffect::GuardDog
            | CardEffect::GuessEnemyHandGainHealth { .. }
            | CardEffect::Hellraiser
            | CardEffect::HenchThugBuff
            | CardEffect::Hexmarshal
            | CardEffect::HoggerStartOfGame
            | CardEffect::HolyBola
            | CardEffect::HolyEmbrace
            | CardEffect::ImbueAndDebuffEnemies { .. }
            | CardEffect::ImbueAndGetWisp
            | CardEffect::ImbueAndReduceHandCost
            | CardEffect::ImbueAndTriggerHeroPower
            | CardEffect::ImbueEveryThirdSpell
            | CardEffect::ImbueHeroPower
            | CardEffect::ImbuedHeroPower { .. }
            | CardEffect::Imfernal
            | CardEffect::ImpGangStooge
            | CardEffect::IncrementOmenAttack
            | CardEffect::InfernoHeraldTrigger { .. }
            | CardEffect::InfiniteDamageToHighestHealthEnemyMinion
            | CardEffect::IridaGetVoid
            | CardEffect::IridaSinseeker
            | CardEffect::JadeCoin
            | CardEffect::JadeGuardians
            | CardEffect::Judgment
            | CardEffect::KabalCoin
            | CardEffect::KarovThreeLegendaryCopies
            | CardEffect::KeymasterCopy
            | CardEffect::KingOfTheUnderbelly
            | CardEffect::LethalRecipe
            | CardEffect::LifestealSelfDamage { .. }
            | CardEffect::LohMinionsCost5
            | CardEffect::LookAtSecretsGiveRandom
            | CardEffect::LoseHealthPerOpponentHandCard
            | CardEffect::LowSecurityWing
            | CardEffect::MagmaHoundSplash
            | CardEffect::MaievBuffDormant
            | CardEffect::ManastormSetAfterSpell
            | CardEffect::MinHealthUntilEndOfTurn
            | CardEffect::MindSweeper
            | CardEffect::MoltenGold
            | CardEffect::MoraggDeathrattle
            | CardEffect::MugzeeStartOfGame
            | CardEffect::MurlocHolmes
            | CardEffect::MurozondPrepareInfiniteAttack
            | CardEffect::Nab
            | CardEffect::NethrekStartOfGame
            | CardEffect::NextCardCostsZero
            | CardEffect::NextComboDiscount { .. }
            | CardEffect::NextDemonDiscount { .. }
            | CardEffect::NextEnemyHeroPowerCostMore { .. }
            | CardEffect::NextEnemySpellsCostMore { .. }
            | CardEffect::NextHeroPowerCostsZero
            | CardEffect::NextMurlocCostsHealth
            | CardEffect::NextMurlocCostsLess { .. }
            | CardEffect::NextSecretCostsZero
            | CardEffect::NextSpellDiscount { .. }
            | CardEffect::NextSpellsCastTwice { .. }
            | CardEffect::NextTurnEnemyCardsCostMore { .. }
            | CardEffect::NiriOfTheCrater
            | CardEffect::NoxiousBribe
            | CardEffect::OmenDeathrattle
            | CardEffect::OngoingEndTurnDamage { .. }
            | CardEffect::OpponentDrawsTwoAndCopies
            | CardEffect::OverloadForAndGainImmuneWindfury { .. }
            | CardEffect::PassiveHeroPower
            | CardEffect::PickPocket
            | CardEffect::PreciseShot { .. }
            | CardEffect::PressTheAdvantage
            | CardEffect::PreventFatalDamageAndImmune
            | CardEffect::R4TCatcher
            | CardEffect::RatBurglar
            | CardEffect::RecastRandomHolySpellThisTurn
            | CardEffect::RedirectAttackToRandomCharacter
            | CardEffect::ReduceAdjacentHandCardCost { .. }
            | CardEffect::ReduceHandCostIfAllDistinct { .. }
            | CardEffect::ReduceHandMinionGiftCost
            | CardEffect::ReduceNonStartingHandCost { .. }
            | CardEffect::ReduceRandomBeastHandCost { .. }
            | CardEffect::ReduceRandomEnemyHandMinionCost { .. }
            | CardEffect::ReduceRightmostHandCardCost { .. }
            | CardEffect::ReflectDamage
            | CardEffect::RefreshManaCrystals { .. }
            | CardEffect::RefreshManaEqualSelfAttack
            | CardEffect::RefreshManaIfHoldingDragon { .. }
            | CardEffect::RefreshManaIfNoMinionPlayedLastTurn { .. }
            | CardEffect::RehgarBolt
            | CardEffect::ReinforcementAura
            | CardEffect::ReleaseTheBeasts
            | CardEffect::RemoveKeywordFromColossalMain { .. }
            | CardEffect::RemoveTopEnemyDeckCard
            | CardEffect::RemoveWeaponDurability { .. }
            | CardEffect::ReopenLocation
            | CardEffect::ReopenLocationIfFelSpell
            | CardEffect::ReplaceCompanionsAndSummonRandomBeast { .. }
            | CardEffect::ReplaceHandAndDeckWithRandomChooseOne
            | CardEffect::ReplaceHandAndSwapBackAtTurnEnd
            | CardEffect::ReplaceHero { .. }
            | CardEffect::ReplayOneCostCardsPlayedThisGame
            | CardEffect::ResetBothHandsCosts
            | CardEffect::ResilientSaviorHealthBonus { .. }
            | CardEffect::RestoreAllFriendlyAndSummonTwoRandomCostMinions { .. }
            | CardEffect::RestoreAndDrawAndImbue { .. }
            | CardEffect::RestoreAndGrantHeroDivineShield { .. }
            | CardEffect::RestoreBothHeroes { .. }
            | CardEffect::RestoreDamagedFriendly { .. }
            | CardEffect::RestoreHandSnapshot
            | CardEffect::RestoreHealthAndGetDruidSpells { .. }
            | CardEffect::RestoreHealthAndPendingSelfDamage { .. }
            | CardEffect::RestoreInfinityHandCardCost
            | CardEffect::RestoreRandomFriendly { .. }
            | CardEffect::ResurrectAllDifferentFriendlyCostGE { .. }
            | CardEffect::ResurrectDeathrattleMinionCostGE { .. }
            | CardEffect::ResurrectDeathrattleMinionCostLE { .. }
            | CardEffect::ResurrectDiedMinion
            | CardEffect::ResurrectDiedMinionFull
            | CardEffect::ResurrectHighestCostFallen
            | CardEffect::ResurrectMinion
            | CardEffect::ResurrectOneOfEachCostGiveReborn { .. }
            | CardEffect::ResurrectRandomFallenDragon
            | CardEffect::ResurrectWeaponKilled
            | CardEffect::ReturnAllToHand
            | CardEffect::ReturnDevouredCards
            | CardEffect::ReturnEnemyMinionCantPlayNextTurn
            | CardEffect::ReturnHoardCostLess
            | CardEffect::ReturnLastTurnSpells
            | CardEffect::ReturnRandomFriendlyAndReduceCost { .. }
            | CardEffect::ReturnToHandAndIncreaseCost { .. }
            | CardEffect::ReverseDeckOrder
            | CardEffect::Sawbones
            | CardEffect::ScarletBruiser
            | CardEffect::ScarletRecruiter
            | CardEffect::ScrambleForGear
            | CardEffect::SetAllOtherMinionsAttack { .. }
            | CardEffect::SetAllOtherMinionsHealth { .. }
            | CardEffect::SetChronologicalAura { .. }
            | CardEffect::SetCompanionBonus { .. }
            | CardEffect::SetCompanionReplacement { .. }
            | CardEffect::SetCompanionReplacementAndDraw { .. }
            | CardEffect::SetDealExact2Bonus
            | CardEffect::SetDeckBottomCostsOne { .. }
            | CardEffect::SetDragonsHaveRush
            | CardEffect::SetEndTurnEffectsTwice { .. }
            | CardEffect::SetEnemyHeroCantBeHealed
            | CardEffect::SetEnemyMinionHealthTo1IfHoldingDragon
            | CardEffect::SetEventSubjectHealthToSource
            | CardEffect::SetFlockPending
            | CardEffect::SetGeddonDiscoverDraw
            | CardEffect::SetHandMinionStatsToHigher
            | CardEffect::SetHatchingPending
            | CardEffect::SetHealingBonus { .. }
            | CardEffect::SetLakkariTicks { .. }
            | CardEffect::SetLeylineDiscount { .. }
            | CardEffect::SetLeylineEffectBonus { .. }
            | CardEffect::SetLeylineExtraTrigger { .. }
            | CardEffect::SetMurlocSummonBuff
            | CardEffect::SetNextHealDealsDamage
            | CardEffect::SetNextKindredTwice
            | CardEffect::SetNextTemporaryDiscount { .. }
            | CardEffect::SetNextTurnSummon { .. }
            | CardEffect::SetPlayedMinionHealth { .. }
            | CardEffect::SetRandomHandCardCostInfinity
            | CardEffect::SetRemainingHealthAndFullHealDamage { .. }
            | CardEffect::SetSilverHandRecruitStats { .. }
            | CardEffect::SetStatsAllEnemyMinions { .. }
            | CardEffect::SetStatsAndFillBoardWithCopies { .. }
            | CardEffect::SetStatsAndGrantCharge { .. }
            | CardEffect::SetStatsAttachDamageAllDeathrattle { .. }
            | CardEffect::SetStatsGrantLifestealForceAttack { .. }
            | CardEffect::SetStatsGrantStealthAndDraw { .. }
            | CardEffect::SetWeaponAttackInfinityThisTurn
            | CardEffect::ShadowRounds
            | CardEffect::ShuffleAllMinionsIntoDecks
            | CardEffect::ShuffleCardIntoDeck { .. }
            | CardEffect::ShuffleLeftmostHandCardIntoDeck
            | CardEffect::ShuffleMatchingEnemyHandCardIntoDeck
            | CardEffect::ShuffleRandomLegendaryDragonsCost1
            | CardEffect::ShuffleRandomMinionsCostGE8DoubleStats
            | CardEffect::SilenceAllEnemyMinionsAndDraw { .. }
            | CardEffect::SilenceAndDestroyAllOtherMinions
            | CardEffect::SilenceAndDestroyRandomEnemyMinion
            | CardEffect::SliceAndDice
            | CardEffect::SmuggledShovel
            | CardEffect::Soothsayer
            | CardEffect::SoulrestMarkAndBuff
            | CardEffect::SpendAllManaCastRandomSpell
            | CardEffect::SpendArmorDealDamageAllMinions { .. }
            | CardEffect::SpendCorpsesDamageMinion { .. }
            | CardEffect::SpendCorpsesDamageRandom { .. }
            | CardEffect::SpendCorpsesGainReborn { .. }
            | CardEffect::SpendCorpsesRestoreHeroHealth { .. }
            | CardEffect::SpendCorpsesSummonCopy { .. }
            | CardEffect::SpendCorpsesSummonFootmen { .. }
            | CardEffect::SpendCorpsesSummonRandomMinion { .. }
            | CardEffect::SpireOfSolitude
            | CardEffect::SpireSecurity
            | CardEffect::SpitefulChef
            | CardEffect::SplashHeroAttackToRandomEnemy
            | CardEffect::Split100StatsAmongDeckMinionsIfCostGE100 { .. }
            | CardEffect::SplitDamageAmongAllEnemiesChainOnDeath { .. }
            | CardEffect::SplitDamageAmongAllEnemiesIfFallen { .. }
            | CardEffect::StaffOfTrickery
            | CardEffect::Stormfury
            | CardEffect::SummonAllCompanions
            | CardEffect::SummonAndRedirectAttack { .. }
            | CardEffect::SummonBeetles { .. }
            | CardEffect::SummonBloodFighterFromHandBuffAndAttack
            | CardEffect::SummonBroodlingsIfHoldingGift
            | CardEffect::SummonChestForOpponent
            | CardEffect::SummonCopyIfAttackGE { .. }
            | CardEffect::SummonCopyIfSpellDamageDealtThisTurn
            | CardEffect::SummonCopyOfDiscardedMinion
            | CardEffect::SummonCopyOfFriendlyMinion
            | CardEffect::SummonCopyOfRandomFriendlyDragon
            | CardEffect::SummonCopyOfSelf
            | CardEffect::SummonDamagedCopiesRush
            | CardEffect::SummonDragonWithSelfStats
            | CardEffect::SummonEggHatchingDragon
            | CardEffect::SummonFelbatOnDraw
            | CardEffect::SummonFiveHungryDrakesSpendCorpsesRush
            | CardEffect::SummonGolemsSpendAllMana
            | CardEffect::SummonHighestCostFallenUndead
            | CardEffect::SummonLocationForPlayer { .. }
            | CardEffect::SummonManaWorthRandomMinions { .. }
            | CardEffect::SummonMinion { .. }
            | CardEffect::SummonMinionAndBuffFriendlyMinions { .. }
            | CardEffect::SummonMinionPair { .. }
            | CardEffect::SummonMinionsAndGrantDeathrattleAll { .. }
            | CardEffect::SummonMinionsAndGrantFriendlyAttackDivineShield { .. }
            | CardEffect::SummonMinionsGrantRandomBonus { .. }
            | CardEffect::SummonMinionsGrantTwoRandomBonus { .. }
            | CardEffect::SummonMoragg
            | CardEffect::SummonMultipleMinions { .. }
            | CardEffect::SummonOasisWaterElemental
            | CardEffect::SummonPairScrambleStats { .. }
            | CardEffect::SummonRandomAnimalCompanion
            | CardEffect::SummonRandomCostAndFreeze { .. }
            | CardEffect::SummonRandomCostBeastAttackRandomEnemy { .. }
            | CardEffect::SummonRandomCostMinion { .. }
            | CardEffect::SummonRandomCostMinionSetStats { .. }
            | CardEffect::SummonRandomCostMinionTimes { .. }
            | CardEffect::SummonRandomDeathrattleMinionCostGEAndTrigger { .. }
            | CardEffect::SummonRandomDeckBeastGiveLifesteal
            | CardEffect::SummonRandomDeckMinionAndTigerForOpponent { .. }
            | CardEffect::SummonRandomDemonFromHandOrDeck
            | CardEffect::SummonRandomDragonCostGE { .. }
            | CardEffect::SummonRandomDragonOfCost { .. }
            | CardEffect::SummonRandomDragonPerSelfDeath
            | CardEffect::SummonRandomEnemyDeckMinion { .. }
            | CardEffect::SummonRandomEnemyHandMinion
            | CardEffect::SummonRandomFelBeast
            | CardEffect::SummonRandomFishFromDeck
            | CardEffect::SummonRandomLegendaryBeast
            | CardEffect::SummonRandomLegendaryMinion
            | CardEffect::SummonRandomLegendaryMinionSetStats { .. }
            | CardEffect::SummonRandomMinion { .. }
            | CardEffect::SummonRandomMinionCostEqHandSize
            | CardEffect::SummonRandomMinionCostOrEscalated { .. }
            | CardEffect::SummonRandomMinionCostTaunt { .. }
            | CardEffect::SummonRandomMinionFromDeck
            | CardEffect::SummonRandomMinionFromHand
            | CardEffect::SummonRandomMinionOfCost { .. }
            | CardEffect::SummonRandomMinionOfCostDormant { .. }
            | CardEffect::SummonRandomMinionsOfCost { .. }
            | CardEffect::SummonRandomTauntMinionCostGE { .. }
            | CardEffect::SummonRandomTauntMinionsOfCosts { .. }
            | CardEffect::SummonRandomThreeTwoOneCostMinions
            | CardEffect::SummonRandomTwoCostTauntAndImbue
            | CardEffect::SummonRaptorsOutcast { .. }
            | CardEffect::SummonRecruitsAndEquipWeapon
            | CardEffect::SummonShadowAttacksRandomEnemy { .. }
            | CardEffect::SummonSilverHandRecruitsWithDivineShield { .. }
            | CardEffect::SummonSpellbender
            | CardEffect::SummonStatueTrio
            | CardEffect::SummonTauntAndIfHoldingDragonAgain { .. }
            | CardEffect::SummonTreantCopyingSpell
            | CardEffect::SummonTwoCopiesOfSelf
            | CardEffect::SummonTwoDeathrattleMinionsAndFight
            | CardEffect::SummonTwoDemonsAttackLowestHealthIfDeckNoMinions
            | CardEffect::SummonTwoRandomCostBeastsAttackRandomEnemies { .. }
            | CardEffect::SummonTwoRandomCostMinions { .. }
            | CardEffect::SummonTwoRandomCostMinionsWithAttack { .. }
            | CardEffect::SummonTwoRandomLegendaryMinions
            | CardEffect::SummonTwoRandomMinionsOfCost { .. }
            | CardEffect::SummonTwoRandomOneCostMinions
            | CardEffect::SummonTwoTreantsScaling
            | CardEffect::SummonZombiesWithCorpseReborn { .. }
            | CardEffect::SwapHeroPowerToDeal8Random
            | CardEffect::SwapStatsIfSurvivesDamage
            | CardEffect::SwapWithHandMinion
            | CardEffect::SylvanasDealToAllEnemiesRepeated { .. }
            | CardEffect::TakeControlUntilEndOfTurnCantAttack
            | CardEffect::TeamworkSummonAndGetRecruits
            | CardEffect::ThalenaSecondHeroPower
            | CardEffect::ThievesTools
            | CardEffect::TinyPal { .. }
            | CardEffect::TogwaggleShuffleHands
            | CardEffect::TransformFriendlyMinionsCost1MoreSummonOriginals
            | CardEffect::TransformHandMinionsToRandomDemons
            | CardEffect::TransformHandSelfToRandomEnemyHandMinion
            | CardEffect::TransformNeutralDeckToDruid
            | CardEffect::TransformRandomEnemyMinionToSelf
            | CardEffect::TransformRandomMinionIntoRandomMinion
            | CardEffect::TransformSelfIfSurvivesDamageToRandomCost { .. }
            | CardEffect::TransformSelfToCastSpell
            | CardEffect::TransformSelfToRandomMinionOfCost { .. }
            | CardEffect::TransformToRandom { .. }
            | CardEffect::TrastathGainSummonedDemonStats
            | CardEffect::TricksyImproviser
            | CardEffect::TriggerFriendlyCinderDeathrattles
            | CardEffect::TriggerFriendlyDeadDeathrattles { .. }
            | CardEffect::TriggerFriendlyDeathrattles
            | CardEffect::TriggerRandomFriendlyEndTurnEffect
            | CardEffect::TruthSeeker
            | CardEffect::UnlockOverloadedCrystals
            | CardEffect::UpgradeHeroPowerCost1
            | CardEffect::UrsocBattlecry
            | CardEffect::UrsocDeathrattle
            | CardEffect::UseHeroPower
            | CardEffect::VanessaGetBattlecryMinionCost2Less
            | CardEffect::VigilantSentry
            | CardEffect::VoidBlast
            | CardEffect::VoidSoul
            | CardEffect::VolcorossBattlecry
            | CardEffect::WidowsBanquet
            | CardEffect::WidowsBite
            | CardEffect::WidowsFeast
            | CardEffect::YseraAwakens { .. }
            | CardEffect::YseraEmeraldAspect
            | CardEffect::ZuramatPlaysDiscarded => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::card_by_id;

    /// The effect a card plays with: battlecry and spell_effect share one slot
    /// at runtime (`cards::mod`), so the test mirrors that union.
    fn play_effect(id: &str) -> CardEffect {
        let def = card_by_id(id).unwrap_or_else(|| panic!("{id} is not in ALL_CARDS"));
        def.battlecry
            .or(def.spell_effect)
            .unwrap_or_else(|| panic!("{id} has no play-time effect"))
    }

    /// Guards the declarations against silent deletion: these are the shapes
    /// the enumeration in `rl::env` relies on.
    #[test]
    fn representative_cards_declare_their_target_domain() {
        // Fireball — "Deal 6 damage." (The engine's domain is AnyEnemy;
        // the official card can also hit friendly characters, which is a
        // separate fidelity question from this module's job.)
        assert_eq!(
            play_effect("MAGE_001").play_target(),
            Some(EffectTarget::AnyEnemy)
        );
        // Assassinate — destroy an enemy minion.
        assert_eq!(
            play_effect("ROGUE_006").play_target(),
            Some(EffectTarget::AnyEnemyMinion)
        );
        // Bloodfen Raptor — a vanilla minion has no play-time effect at all.
        assert!(card_by_id("CLASSIC_001").unwrap().battlecry.is_none());
    }

    /// The outstanding debt from `docs/play-target-gap-audit.md`, as an
    /// executable ledger: every card here is one the official text targets but
    /// the engine still plays untargeted (resolving at random, or fizzling).
    ///
    /// **Fixing a card means deleting its id from this list in the same
    /// commit** — the assertion fails otherwise, which is the point: the
    /// ledger cannot drift away from the code the way the old inline
    /// whitelist did.
    #[test]
    fn audit_gap_cards_are_still_untargeted() {
        const OUTSTANDING: &[&str] = &["EDR_813"];
        let fixed: Vec<&str> = OUTSTANDING
            .iter()
            .copied()
            .filter(|id| play_effect(id).play_target().is_some())
            .collect();
        assert!(
            fixed.is_empty(),
            "these cards now declare a play target — remove them from \
             OUTSTANDING and from docs/play-target-gap-audit.md: {fixed:?}"
        );
    }
}
