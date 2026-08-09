//! Card definition module — basic card data.
//!
//! Contains the CardDef struct, the vanilla! macro, and all Classic card definitions.
//! All card constants are re-exported through the `def` module,
//! accessible to external code via `crate::cards::def::*`.

pub mod choose_hand_card;
pub mod classic_druid;
pub mod classic_hunter;
pub mod classic_legendary;
pub mod classic_mage;
pub mod classic_neutral;
pub mod classic_paladin;
pub mod classic_priest;
pub mod classic_rogue;
pub mod classic_shaman;
pub mod classic_warlock;
pub mod classic_warrior;
pub mod colossal;
pub mod core_w1;
pub mod core_w2;
pub mod core_w3a;
pub mod core_w3b;
pub mod core_w3c;
pub mod core_w3d;
pub mod core_w4a;
pub mod core_w4b;
pub mod core_w5;
pub mod core_w6;
pub mod core_w7;
pub mod core_w8;
pub mod def;
pub mod exp_cata_w1;
pub mod exp_cata_w2;
pub mod exp_cata_w3;
pub mod exp_cata_w4;
pub mod exp_edr_w1;
pub mod exp_edr_w2;
pub mod exp_edr_w3;
pub mod exp_edr_w4a;
pub mod exp_edr_w4b;
pub mod exp_edr_w5;
pub mod exp_jail_w1;
pub mod exp_jail_w2;
pub mod exp_tlc_w2;
pub mod exp_tlc_w3;
pub mod exp_tlc_w4a;
pub mod exp_tlc_w4b;
pub mod exp_tlc_w4c;
pub mod exp_tmw_w2a;
pub mod exp_tmw_w2b;
pub mod exp_tmw_w3;
pub mod generated;
pub mod herald;
pub mod kindred;
pub mod pool;
pub mod prepare;
pub mod quest;
pub mod rewind;
pub mod sets;
pub mod start_of_game;

use crate::core::component::{
    Attack, AttacksUsed, Aura, CardId, Cost, Deathrattle, Dormant, Durability, Enrage, Health,
    Lifesteal, Overload, Poison, Reborn, Rush, Stealth, Tradeable, Trigger, TriggerEvent,
    TriggerTiming,
};
use crate::core::effect::{CardEffect, EffectTarget, KeywordKind};
use crate::core::entity::Entity;
use crate::core::player::PlayerId;
use crate::core::state::GameState;
use crate::core::world::World;
use def::CardDef;

/// M4-W4 — CATA_493 Duke of Below: "Has +2/+2 for each card you've
/// discarded this game." The self-scoped GainStats aura bakes at
/// summon/play with the CURRENT discarded count; the discard chokepoint
/// (resolve_discard_entity) re-bakes it on every later discard (§26).
pub(crate) fn bake_duke_of_below(state: &mut GameState, entity: Entity, owner: PlayerId) {
    let n = state.player(owner).discarded_this_game;
    let bonus = (2 * n) as i32;
    state.world_mut().set_aura(
        entity,
        crate::core::component::Aura {
            effect: crate::core::component::AuraEffect::GainStats {
                attack: bonus,
                health: bonus,
            },
            target: crate::core::component::AuraTarget::Self_,
        },
    );
}

/// Looks up a card by its ID (first match in `ALL_CARDS`, which deduplicates
/// IDs; then the handwritten expansion cards `HANDWRITTEN_EXPANSION_CARDS`
/// — the M1+ effect waves' handwritten implementations override the
/// generated baselines; finally the 2025–2026 expansion baselines
/// `EXPANSION_CARDS`).
///
/// Used by the RL environment and Python bindings to build explicit decks
/// (roadmap M1-G2); `None` for unknown IDs.
#[must_use]
pub fn card_by_id(id: &str) -> Option<&'static CardDef> {
    sets::ALL_CARDS
        .iter()
        .find(|c| c.id == id)
        .or_else(|| {
            sets::HANDWRITTEN_EXPANSION_CARDS
                .iter()
                .find(|c| c.id == id)
        })
        .or_else(|| sets::EXPANSION_CARDS.iter().find(|c| c.id == id))
}

#[cfg(test)]
mod lookup_tests {
    use super::card_by_id;
    use crate::cards::def::BLOODFEN_RAPTOR;

    #[test]
    fn card_by_id_resolves_known_and_unknown() {
        let card = card_by_id("CLASSIC_001").expect("Bloodfen Raptor is in ALL_CARDS");
        assert_eq!(card.id, BLOODFEN_RAPTOR.id);
        assert_eq!(card.name, "Bloodfen Raptor");
        assert!(card_by_id("NOT_A_REAL_CARD_999").is_none());
        assert!(card_by_id("").is_none());
    }
}

/// Applies special keyword components (Poison, Stealth, Overload, etc.) to an entity.
///
/// These keywords do not add `CardDef` fields (to avoid large struct changes);
/// instead they are mapped here centrally by card ID. Called when summoning minions
/// (`trigger::resolve_summon`) and building cards (`GameBuilder::spawn_minion`).
///
/// Pool-open triggers registered through this hook (no `CardDef` field to live
/// in): the sets.rs closure test treats these IDs as pool-open as well. The
/// registration itself is per-ID in `apply_card_keywords` (Cho's AnySpellCast
/// copy, Shaku's Attacked copy) — this list is the test's second source of
/// truth only.
#[allow(dead_code)] // consumed by the sets.rs registry test (cfg(test))
pub(crate) const POOL_OPEN_KEYWORD_IDS: &[&str] = &[
    "LEGENDARY_024", // Lorewalker Cho — AnySpellCast copy trigger
    "CORE_EX1_100",  // Lorewalker Cho (Core Set W3a) — same trigger
    "CORE_CFM_781",  // Shaku, the Collector — Attacked copy trigger (registered below)
];

/// M3-W2a — minions that enter play Dormant, keyed by card ID (the
/// Across the Timeways Dormant primitive): TIME_046 Cyborg Patriarch
/// (Dormant 3), TIME_063 Timelord Nozdormu (Dormant 5). The returned
/// countdown starts at the owner's next turn start.
pub(crate) fn dormant_at_summon(card_id: &str) -> Option<u32> {
    match card_id {
        "TIME_046" => Some(3), // Cyborg Patriarch
        "TIME_063" => Some(5), // Timelord Nozdormu
        _ => None,
    }
}

pub(crate) fn apply_card_keywords(world: &mut World, entity: Entity, card_def: &CardDef) {
    // Core Set W1 keywords — RUSH / LIFESTEAL / REBORN as components (Core
    // Set W1 primitives; the `CardDef` struct stays untouched per the
    // "avoid large struct changes" convention, matching Poison/Overload).
    if matches!(
        card_def.id,
        // Rush (5)
        "CORE_BT_156"   // Imprisoned Vilefiend
        | "CORE_DRG_079" // Evasive Wyrm
        | "CORE_RLK_657" // Underking
        | "CORE_TRL_900" // Halazzi, the Lynx
        | "CORE_WC_701" // Felrattler
        | "CORE_BAR_801t" // Swift Hyena (W3b token)
        | "CORE_ULD_178" // Siamat (W4b — simplified to Rush+Taunt)
        | "CORE_BOT_451t" // Spark (W6 token)
        | "CORE_TSC_650t4" // Otter (W6 token — Flipper Friends)
        | "EDR_227" // Umbraclaw (M1-W1 — the Emerald Dream imbue wave)
        | "EDR_263t" // Greatwolf (M1-W3 — Grace of the Greatwolf token)
        | "EDR_486" // Scorching Observer (M1-W4a)
        | "EDR_492t" // Duckling (M1-W4a — Mother Duck token)
        | "EDR_262t" // Wolf (M1-W4a — Spirit Bond token)
        | "EDR_421" // Omen (M1-W4b — the Wild Gods wave)
        | "EDR_480" // Goldrinn (M1-W4b — the Wild Gods wave)
        | "FIR_951" // Volcoross (M1-W5 — the Embers of the World Tree wave)
        | "FIR_953" // Magma Hound (M1-W5 — the Embers of the World Tree wave)
        | "TLC_229t14" // Ashalon, Ridge Guardian (M2-W2 — quest reward token)
        | "TLC_830t" // Shokk, Jungle Tyrant (M2-W2 — quest reward token)
        | "TLC_243" // Whirling Stormdrake (M2-W3 — Kindred wave)
        | "TLC_366" // Pterrorwing Ravager (M2-W3 — Kindred wave)
        | "TLC_429t" // Juvenile Steamfin (M2-W3 — Steamfin Thief token)
        | "TLC_903" // Silithid Queen (M2-W3 — Kindred wave)
        | "TLC_240" // Tyrannogill (M2-W4a — the main-set wave)
        | "TLC_436" // Reanimated Pterrordax (M2-W4a)
        | "TLC_520" // Underbrush Tracker (M2-W4a)
        | "TLC_630" // Gorishi Wasp (M2-W4a)
        | "DINO_136t" // Ravenous Raptor (M2-W4c — Horn of Feasting token)
        // M3-W2a — Across the Timeways
        | "TIME_022" // Perennial Serpent
        | "TIME_029" // Ruinous Velocidrake
        | "TIME_050" // Sentient Hourglass
        | "TIME_051" // Soldier of the Infinite
        | "TIME_063" // Timelord Nozdormu
        | "TIME_605" // Epoch Stalker
        | "TIME_872" // Undefeated Champion
        // M3-W2b — Across the Timeways legendary wave (the Blood Fighter
        // tokens TIME_850t/TIME_850t1 are the transformed forms — their
        // Taunt / Elusive ride the CardDef fields, no Rush)
        | "TIME_209" // Muradin, High King
        | "TIME_850" // Lo'Gosh, Blood Fighter
        // M3-W3 — The End of Time miniset
        | "END_032" // Winged Aberration
        // M4-W1 — the Cataclysm Colossal wave
        | "CATA_153" // Al'Akir, Lord of Storms
        // M4-W2 — the Cataclysm Herald wave
        | "CATA_525" // Armored Bloodletter
        | "CATA_561t" // Breezling (Ritual of Power token)
        // M4-W4 — the Cataclysm closing wave
        | "CATA_469" // Chromatic Broodmother
        | "CATA_493" // Duke of Below
        // M5-W1 — Escape from Violet Hold (only Tras'tath carries Rush;
        // Warptooth and The Living Plague ride the CardDef charge field
        // and Moragg has no Rush)
        | "JAIL_721" // Tras'tath, Soul Parasite (also Prepare)
        // M5-W2 — the Violet Hold closing wave
        | "JAIL_447" // Reckless Detective
        | "JAIL_459" // Arachnathid
    ) {
        world.set_rush(entity, Rush);
    }
    if matches!(
        card_def.id,
        // Lifesteal (8)
        "CORE_BAR_311"   // Devouring Plague (spell)
        | "CORE_BT_801"  // Eye Beam (spell)
        | "CORE_BT_921"  // Aldrachi Warblades (weapon)
        | "CORE_GIL_558" // Swamp Leech
        | "CORE_ICC_055" // Drain Soul (spell)
        | "CORE_ICC_214" // Obsidian Statue
        | "CORE_SW_442"  // Void Shard (spell)
        | "CORE_TTN_866" // Mythical Terror
        | "EDR_449"     // Lunarwing Messenger (M1-W1)
        | "EDR_860" // Resplendent Dreamweaver (M1-W1)
        | "EDR_255" // Renewing Flames (M1-W4a — spell)
        | "EDR_272" // Evergreen Stag (M1-W4a)
        | "EDR_486" // Scorching Observer (M1-W4a)
        | "FIR_777" // Spirit of the Kaldorei (M1-W5 — the Embers wave)
        | "TLC_436" // Reanimated Pterrordax (M2-W4a — the main-set wave)
        | "TLC_605" // Tar Tyrant (M2-W4a)
        | "TLC_819" // Gladesong Siren (M2-W4a)
        | "TLC_821" // Wilted Shadow (M2-W4a)
        | "TIME_028" // Fatebreaker (M3-W2a)
        | "TIME_056" // Whelp of the Bronze (M3-W2a)
        | "TIME_427" // Cleansing Lightspawn (M3-W2a)
        // M3-W3 — The End of Time miniset
        | "END_025" // Eternal Firebolt (spell)
        // M4-W1 — the Cataclysm Colossal wave (Chromatus and its Red
        // Head — the other three heads ride the CardDef fields)
        | "CATA_432" // Chromatus
        | "CATA_432t2" // Red Head of Chromatus
        // M4-W2 — the Cataclysm Herald wave
        | "CATA_780" // Obsessive Technician
        // M4-W4 — the Cataclysm closing wave
        | "CATA_304" // Injured Attendant
        // M5-W2 — the Violet Hold closing wave
        | "JAIL_441" // Drink Blood (spell)
        | "JAIL_454t" // Necronurse (token)
        | "JAIL_805" // Stormfury (spell)
        | "JAIL_805t" // Stormfury Elemental (token)
    ) {
        world.set_lifesteal(entity, Lifesteal);
    }
    if matches!(
        card_def.id,
        // Reborn (5): Malignant Horror, Murmy, Sol'etos Death's Touch
        // (TLC_817t4, M2-W2 — the Un'Goro quest reward token), Undercover
        // Cultist and Reluctant Wrangler (M2-W4a — the main-set wave)
        "CORE_RLK_745" | "CORE_ULD_723" | "TLC_817t4" | "TLC_101" | "TLC_443"
        // M3-W2a — Across the Timeways
        | "TIME_045" // Whelp of the Infinite
    ) {
        world.set_reborn(entity, Reborn);
    }
    if card_def.id == "CORE_TTN_866" {
        // Mythical Terror — dual tribe Demon + Beast (the CardDef carries
        // the primary Demon; the Beast half lands here, Core Set W1)
        world.add_race(entity, crate::core::component::Race::Beast);
    }
    // M1-W4b dual tribes — the Wild Gods wave (the CardDef carries the
    // primary tribe from the JSON "race" field; the "races" second entry
    // lands here, the Mythical Terror precedent)
    if matches!(
        card_def.id,
        "EDR_421" // Omen — Demon + Beast
        | "EDR_493" // Alara'shi — Demon + Beast
    ) {
        world.add_race(entity, crate::core::component::Race::Demon);
    }
    if card_def.id == "EDR_818" {
        // Nythendra — Undead + Dragon
        world.add_race(entity, crate::core::component::Race::Dragon);
    }
    // M2-W3 dual tribes — the Kindred wave (the CardDef carries the primary
    // race — the one the Kindred text keys on, matching the registry in
    // cards/kindred.rs — the second tribe lands here, the Mythical Terror
    // precedent; a played minion counts for BOTH tribes' synergies and for
    // the Kindred push's first (primary) race)
    if card_def.id == "TLC_102" {
        // Torga — Beast + Undead
        world.add_race(entity, crate::core::component::Race::Undead);
    }
    if card_def.id == "TLC_223" {
        // Volcanic Thrasher — Elemental + Beast
        world.add_race(entity, crate::core::component::Race::Beast);
    }
    if card_def.id == "TLC_243" {
        // Whirling Stormdrake — Elemental + Dragon
        world.add_race(entity, crate::core::component::Race::Dragon);
    }
    // M4-W1 dual tribes — the Cataclysm Colossal wave (the same
    // convention: the CardDef carries the first dump-listed race, the
    // second tribe lands here)
    if matches!(
        card_def.id,
        // Arisen Onyxia and its Wings — Dragon + Undead
        "CATA_155" | "CATA_155t" | "CATA_155t1"
    ) {
        world.add_race(entity, crate::core::component::Race::Undead);
    }
    if matches!(
        card_def.id,
        // Magmaw and its Bodies — Beast + Mechanical
        "CATA_550"
            | "CATA_550t"
            | "CATA_550t2"
            | "CATA_550t3"
            | "CATA_550t4"
            | "CATA_550t5"
            | "CATA_550t6"
    ) {
        world.add_race(entity, crate::core::component::Race::Mechanical);
    }
    if card_def.id == "TLC_432" {
        // Dread Raptor — Undead + Beast
        world.add_race(entity, crate::core::component::Race::Beast);
    }
    if card_def.id == "TLC_463" {
        // Razidir — Demon + Beast
        world.add_race(entity, crate::core::component::Race::Beast);
    }
    if card_def.id == "TLC_482" {
        // Slagclaw — Elemental + Dragon
        world.add_race(entity, crate::core::component::Race::Dragon);
    }
    // M2-W4c dual tribes — the Festival of the Devilsaur wave (the same
    // convention: the CardDef carries the first dump-listed race, the
    // second tribe lands here — the Kindred cards' race matches the
    // registry in cards/kindred.rs)
    if card_def.id == "DINO_132" {
        // Asphyxiodon — Demon + Beast
        world.add_race(entity, crate::core::component::Race::Beast);
    }
    if card_def.id == "DINO_138" {
        // Diabolus Rex — Demon + Beast
        world.add_race(entity, crate::core::component::Race::Beast);
    }
    if card_def.id == "DINO_401" {
        // The Great Dracorex — Beast + Dragon
        world.add_race(entity, crate::core::component::Race::Dragon);
    }
    if card_def.id == "DINO_404" {
        // Firegill — Elemental + Murloc
        world.add_race(entity, crate::core::component::Race::Murloc);
    }
    if card_def.id == "DINO_407" {
        // Mirrex, the Crystalline — Elemental + Beast
        world.add_race(entity, crate::core::component::Race::Beast);
    }
    if card_def.id == "DINO_409" {
        // Techysaurus — Mechanical + Beast
        world.add_race(entity, crate::core::component::Race::Beast);
    }
    if card_def.id == "DINO_413" {
        // Chillspine Stegodon — Elemental + Beast
        world.add_race(entity, crate::core::component::Race::Beast);
    }
    if card_def.id == "DINO_416" {
        // Hollow Direhorn — Undead + Beast
        world.add_race(entity, crate::core::component::Race::Beast);
    }
    // M1-W4b per-card triggers — the Wild Gods wave:
    // - Omen (EDR_421): every attack improves his deathrattle (the
    //   per-player counter, §14.4);
    // - Tortolla (EDR_471): taking damage gains the owner 1 Armor and gives
    //   this minion +1 Attack.
    if card_def.id == "EDR_421" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::Attacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::IncrementOmenAttack,
            },
        );
    }
    if card_def.id == "EDR_471" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::ThisMinionDamaged,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GainArmorAndSelfAttack {
                    armor: 1,
                    attack: 1,
                },
            },
        );
    }
    // M1-W5 per-card trigger — Magma Hound (FIR_953, the Embers of the
    // World Tree wave): after this attacks a minion and survives, deal its
    // Attack damage split among all enemies. The trigger is pinned to the
    // attacker (AttackedMinion), and the handler re-checks the survive
    // condition at trigger time (§14.5 — the "and survives" reads effective
    // health, the attacker-dead-from-the-trade splash case).
    if card_def.id == "FIR_953" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::AttackedMinion,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::MagmaHoundSplash,
            },
        );
    }
    // Tradeable (Core Set W2) — the 6 Tradeable cards: shuffle-for-1-and-draw
    if matches!(
        card_def.id,
        "CORE_EX1_002"   // The Black Knight
        | "CORE_EX1_005" // Big Game Hunter
        | "CORE_REV_023" // Demolition Renovator
        | "CORE_SW_066"  // Royal Librarian
        | "CORE_SW_072"  // Rustrot Viper
        | "CORE_SW_429" // Best in Shell
        | "TLC_255" // Crystal Tender (M2-W4a — the main-set wave)
        | "CATA_203" // Garona's Last Stand (M4-W4 — the Cataclysm closing wave)
    ) {
        world.set_tradeable(entity, Tradeable);
    }
    // Core Set W3a triggers — Acolyte of Pain (draw on damage), Murloc
    // Tidecaller (attack on friendly Murloc summon), Frothing Berserker
    // (attack on friendly minion damage), Archmage Antonidas (Fireball on
    // friendly spell cast — declared via spell_trigger, no hook needed).
    if card_def.id == "CORE_EX1_007" {
        // Acolyte of Pain — whenever this minion takes damage, draw a card
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::ThisMinionDamaged,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::DrawCard { count: 1 },
            },
        );
    }
    if card_def.id == "CORE_EX1_509" {
        // Murloc Tidecaller — whenever you summon a Murloc, gain +1 Attack
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                race: Some(crate::core::component::Race::Murloc),
                max_attack: None,
                effect: CardEffect::GainStats {
                    attack: 1,
                    health: 0,
                    target: EffectTarget::Self_,
                },
            },
        );
    }
    if card_def.id == "CORE_CFM_344" {
        // Finja, the Flying Star — when this minion attacks, summon a random
        // Murloc from the deck (the subject check lives in the effect)
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::Attacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::SummonRandomFishFromDeck,
            },
        );
    }
    if card_def.id == "CORE_CFM_781" {
        // Shaku, the Collector — when this minion attacks, copy a random
        // enemy deck card (pool-open; registered in POOL_OPEN_CARDS)
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::Attacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::CopyEnemyDeckCardOnSelfAttack,
            },
        );
    }
    if card_def.id == "CORE_CATA_004" {
        // Rehgar Earthfury — this or an adjacent minion attacks: get a
        // Lightning Bolt (the adjacency check lives in the effect)
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::Attacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::RehgarBolt,
            },
        );
    }
    // M2-W4b per-card triggers — the Un'Goro legendary wave:
    // - Archaios (TLC_811): "after a friendly minion attacks, set its
    //   Health to this minion's Health" — the new FriendlyMinionAttacked
    //   event is friendly-scoped with the attacker as the subject (the
    //   pinned Attacked class cannot carry an "another minion" trigger);
    // - Niri of the Crater (TLC_836): "after you play a 1-Cost minion,
    //   double its stats" — CardPlayed fires after the card fully
    //   resolved, subject = the played card). Niri's spell trigger
    //   ("cast a 1-Cost spell twice") rides the SAME CardPlayed event —
    //   it fires for spell casts too — and the combined NiriOfTheCrater
    //   effect branches on the subject's card type (the Trigger component
    //   is a single slot; a second per-ID registration would clobber).
    if card_def.id == "TLC_811" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionAttacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::SetEventSubjectHealthToSource,
            },
        );
    }
    if card_def.id == "TLC_836" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::CardPlayed,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::NiriOfTheCrater,
            },
        );
    }
    // M5-W1 — Escape from Violet Hold trigger attachments (the TLC_836
    // pattern — the trigger rides the card, so it only fires for its
    // own owner):
    // - Vanessa the Ringleader (JAIL_407, Prepare): "After you play a
    //   card, get a random Battlecry minion. It costs (2) less."
    // - Tras'tath, Soul Parasite (JAIL_721, Prepare): "After you summon
    //   a Demon, gain its stats" — the race filter rides the trigger
    //   (the FriendlyMinionSummoned event's subject is the just-summoned
    //   Demon).
    if card_def.id == "JAIL_407" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::CardPlayed,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::VanessaGetBattlecryMinionCost2Less,
            },
        );
    }
    if card_def.id == "JAIL_721" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                race: Some(crate::core::component::Race::Demon),
                max_attack: None,
                effect: CardEffect::TrastathGainSummonedDemonStats,
            },
        );
    }
    // M2-W4c per-card triggers — the Festival of the Devilsaur wave:
    // - The Great Dracorex (DINO_401): "after this attacks an enemy
    //   minion, it damages ALL other enemy minions" — the new
    //   AttackedEnemyMinion event (friendly scope, the DEFENDER as the
    //   subject so the splash can exclude the attacked minion, not pinned
    //   — the trigger rides the Dracorex);
    // - Hollow Direhorn (DINO_416): "after a friendly minion dies, spend
    //   3 Corpses to gain Reborn" — a plain FriendlyMinionDied trigger.
    if card_def.id == "DINO_401" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::AttackedEnemyMinion,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::DracorexSplash,
            },
        );
    }
    if card_def.id == "DINO_416" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionDied,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::SpendCorpsesGainReborn { amount: 3 },
            },
        );
    }
    if card_def.id == "CORE_GIL_534" {
        // Hench-Clan Thug — after your hero attacks, +1/+1 (the subject is
        // the hero; the effect checks it)
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::HeroAttacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::HenchThugBuff,
            },
        );
    }
    if card_def.id == "CORE_SCH_717" {
        // Keymaster Alabaster — whenever the OPPONENT draws a card, add a
        // 1-cost copy to hand
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::CardDrawn,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::KeymasterCopy,
            },
        );
    }
    if card_def.id == "CORE_SW_047" {
        // Highlord Fordragon — a friendly minion losing Divine Shield buffs
        // a minion in hand
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::DivineShieldLost,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::FordragonBuff,
            },
        );
    }
    if card_def.id == "CORE_TTN_843" {
        // Eredar Deceptor — whenever the OWNER draws a card, summon a 1/1
        // Demon with Rush (the effect ignores foreign draws)
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::CardDrawn,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::SummonFelbatOnDraw,
            },
        );
    }
    if card_def.id == "CORE_BT_351" {
        // Battlefiend — after your hero attacks, +1 Attack
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::HeroAttacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GainStats {
                    attack: 1,
                    health: 0,
                    target: EffectTarget::Self_,
                },
            },
        );
    }
    if card_def.id == "CORE_BT_510" {
        // Wrathspike Brute — after this is attacked, 1 damage to all enemies
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::ThisMinionAttacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::DealDamage {
                    amount: 1,
                    target: EffectTarget::AllEnemies,
                },
            },
        );
    }
    if card_def.id == "CORE_NX2_028" {
        // Hookfist-3000 — after your hero attacks, 4 armor and draw
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::HeroAttacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GainArmorAndDrawOnHeroAttack { armor: 4 },
            },
        );
    }
    if card_def.id == "CORE_RLK_121" {
        // Acolyte of Death — after a friendly Undead dies, draw
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionDied,
                timing: TriggerTiming::Whenever,
                race: Some(crate::core::component::Race::Undead),
                max_attack: None,
                effect: CardEffect::DrawCard { count: 1 },
            },
        );
    }
    if card_def.id == "CORE_RLK_567" {
        // Shadow of Demise — each cast spell transforms this into a copy
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::AnySpellCast,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::TransformSelfToCastSpell,
            },
        );
    }
    if card_def.id == "CORE_EX1_604" {
        // Frothing Berserker — whenever a friendly minion takes damage, +1 Attack
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionDamaged,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GainStats {
                    attack: 1,
                    health: 0,
                    target: EffectTarget::Self_,
                },
            },
        );
    }
    // Shaman cards with Overload — the amount locks mana on the owner's next
    // turn (roadmap F1): Lightning Bolt 1, Lightning Storm 2, Feral Spirit 2,
    // Dust Devil 2, Forked Lightning 2, Lava Burst 2, Stormforged Axe 1,
    // Doomhammer 2, Earth Elemental 3. Keyed by actual card IDs (F-A8 fix:
    // the map was mis-wired by the 8 duplicate-ID pairs, giving Forked
    // Lightning 1 instead of 2 and phantom Overload to Windfury / Windspeaker /
    // Ancestral Spirit — those cards are now on unique CS2_*/CS1_* IDs).
    let overload_amount = match card_def.id {
        "SHAMAN_002" => Some(1), // Lightning Bolt
        "SHAMAN_006" => Some(2), // Lightning Storm
        "SHAMAN_009" => Some(2), // Feral Spirit
        "SHAMAN_011" => Some(2), // Doomhammer
        "SHAMAN_015" => Some(2), // Dust Devil
        "SHAMAN_016" => Some(2), // Forked Lightning
        "SHAMAN_017" => Some(2), // Lava Burst
        "SHAMAN_018" => Some(1), // Stormforged Axe
        "SHAMAN_019" => Some(3), // Earth Elemental
        // Core Set W6 overload amounts
        "CORE_AT_052" => Some(1),  // Totem Golem
        "CORE_BOT_451" => Some(1), // Voltaic Burst
        "CORE_EX1_238" => Some(1), // Lightning Bolt
        "CORE_EX1_250" => Some(3), // Earth Elemental
        "CORE_EX1_259" => Some(2), // Lightning Storm
        // M2-W4a — the Un'Goro main-set wave
        "TLC_227" => Some(1), // Lava Flow
        // M3-W2a — Across the Timeways
        "TIME_014" => Some(3), // Instant Multiverse
        // M3-W3 — The End of Time miniset
        "END_028" => Some(2), // For All Time
        // M4-W4 — the Cataclysm closing wave
        "CATA_569" => Some(1), // Ceremonial Clash
        "CATA_724" => Some(3), // Stormbinder
        // M5-W2 — the Violet Hold closing wave
        "JAIL_452" => Some(2), // Disguised Detective
        _ => None,
    };
    if let Some(amount) = overload_amount {
        world.set_overload(entity, Overload(amount));
    }
    // M3-W2a — minions that enter play Dormant (the Dormant primitive, see
    // `Dormant` in core::component). Consulted by the minion play path and
    // by effect summons (resolve_summon_doubled); the countdown starts at
    // the owner's next turn start.
    if let Some(turns) = dormant_at_summon(card_def.id) {
        world.set_dormant(entity, Dormant { turns });
    }
    // M3-W2a — cards that cost Health instead of Mana while in hand
    // (TIME_612 Blood Draw — the marker is read by the CardPlayed
    // pay-health branch, the M2-W4a CostHealth machinery).
    if card_def.id == "TIME_612" {
        world.set_cost_health(entity, crate::core::component::CostHealth);
    }
    // M4-W4 — CATA_208 Selfless Protector: "Takes one extra damage
    // from all sources" (the BonusDamageTaken marker, consulted by the
    // damage pipeline).
    if card_def.id == "CATA_208" {
        world.set_bonus_damage_taken(entity, crate::core::component::BonusDamageTaken);
    }
    // M3-W2a — minions that permanently take double damage (TIME_060
    // Quantum Destabilizer — the marker is exempt from the wrap-up clear).
    if card_def.id == "TIME_060" {
        world.set_double_damage_taken(entity, crate::core::component::DoubleDamageTaken);
    }
    // M3-W2a — "survives damage" triggers (the only registrants of the
    // `TriggerEvent::SurvivedDamage` event, fired after the damage pipeline
    // for a minion that still lives):
    // - TIME_050 Sentient Hourglass — swap this minion's stats;
    // - TIME_055 Unknown Voyager — transform into a random 7-Cost minion.
    if card_def.id == "TIME_050" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::SurvivedDamage,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::SwapStatsIfSurvivesDamage,
            },
        );
    }
    if card_def.id == "TIME_055" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::SurvivedDamage,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::TransformSelfIfSurvivesDamageToRandomCost { cost: 7 },
            },
        );
    }
    if card_def.id == "SHAMAN_021" {
        // Unbound Elemental — gain +1/+1 whenever you play a card with Overload
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyOverloadPlayed,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GainStats {
                    attack: 1,
                    health: 1,
                    target: EffectTarget::Self_,
                },
            },
        );
    }
    if card_def.id == "NEUTRAL_004" {
        // Acolyte of Pain — whenever this minion takes damage, draw a card
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::ThisMinionDamaged,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::DrawCard { count: 1 },
            },
        );
    }
    if card_def.id == "WARRIOR_007" {
        // Frothing Berserker — whenever a friendly minion takes damage, gain +1 attack
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionDamaged,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GainStats {
                    attack: 1,
                    health: 0,
                    target: EffectTarget::Self_,
                },
            },
        );
    }
    if card_def.id == "WARRIOR_013" {
        // Armorsmith — whenever a friendly minion takes damage, gain 1 armor
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionDamaged,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GainArmor {
                    amount: 1,
                    target: EffectTarget::FriendlyHero,
                },
            },
        );
    }
    if card_def.id == "ROGUE_022" {
        // Patient Assassin — Stealth + Poison
        world.set_poison(entity, Poison);
        world.set_stealth(entity, Stealth);
    }
    if card_def.id == "TLC_519t" {
        // Venomous Spitter (M2-W3 — Ambush Predators token): Stealth +
        // Poisonous
        world.set_poison(entity, Poison);
        world.set_stealth(entity, Stealth);
    }
    if matches!(card_def.id, "TLC_468" | "TLC_468t1") {
        // Blob of Tar (M2-W4a — Poisonous, Taunt; the Taunt half rides the
        // CardDef) and its Lanky Blob token — Poisonous
        world.set_poison(entity, Poison);
    }
    if card_def.id == "TIME_045" {
        // Whelp of the Infinite (M3-W2a) — Poisonous
        world.set_poison(entity, Poison);
    }
    // M3-W3 — The End of Time miniset per-card triggers. `spell_trigger`
    // hardcodes FriendlySpellCast, so these ride the registration path.
    if card_def.id == "END_008" {
        // Enduring Roach — after you use your Hero Power, refresh 2 Mana
        // Crystals (the HeroPowerUsed event fires after the hero power's
        // effect resolved; the trigger is pinned to the Roach, friendly
        // scope — any hero power use fires it)
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::HeroPowerUsed,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::RefreshManaCrystals { amount: 2 },
            },
        );
    }
    if card_def.id == "END_016" {
        // Chronoclaws (weapon) — after your hero attacks, discard your
        // highest Cost card (the trigger rides the weapon entity, pinned
        // to the hero's equipped weapon like the Defiled Spear precedent)
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::HeroAttacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::DiscardHighestCostCard,
            },
        );
    }
    if card_def.id == "END_026" {
        // Fragment of Nothing — after you cast a spell on a minion, draw a
        // card (the FriendlySpellCastOnMinion event carries the target
        // minion as the subject; the trigger is pinned to the Fragment, so
        // ANY friendly minion target fires it)
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlySpellCastOnMinion,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::DrawCard { count: 1 },
            },
        );
    }
    if card_def.id == "WARRIOR_008" {
        // Warsong Commander — whenever you summon a minion with 3 or less
        // Attack, give it Charge. A trigger rather than an aura: the Charge is
        // granted once, to that minion, and outlives the commander.
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: Some(3),
                effect: CardEffect::GrantCharge {
                    target: EffectTarget::EventSubject,
                    attack_bonus: 0,
                },
            },
        );
    }
    // Enrage — a conditional state, not a buff. The bonus applies while the
    // minion is damaged, does not stack across damage instances, and is gone
    // the moment the minion is healed to full; `World::effective_attack` and
    // `World::max_attacks` resolve it on read.
    let enrage: Option<Enrage> = match card_def.id {
        // Tauren Warrior — Taunt. Enrage: +3 Attack
        "NEUTRAL_C11" => Some(Enrage {
            attack: 3,
            ..Enrage::default()
        }),
        // Amani Berserker — Enrage: +3 Attack (2/3 → 5/3 while damaged)
        "CLASSIC_018" => Some(Enrage {
            attack: 3,
            ..Enrage::default()
        }),
        // Raging Worgen — Enrage: Windfury and +1 Attack. Both halves are part
        // of the Enrage, so the Windfury goes away with the damage too.
        "NEUTRAL_008" => Some(Enrage {
            attack: 1,
            windfury: true,
            ..Enrage::default()
        }),
        // Grommash Hellscream — Charge. Enrage: +6 Attack (4/9 → 10/9)
        "WARRIOR_010" => Some(Enrage {
            attack: 6,
            ..Enrage::default()
        }),
        // Core Set W7: Grommash Hellscream (8费 4/9 — Charge. Enrage: +6)
        // and Bloodhoof Brave (4费 2/6 — Taunt. Enrage: +3)
        "CORE_EX1_414" => Some(Enrage {
            attack: 6,
            ..Enrage::default()
        }),
        "CORE_OG_218" => Some(Enrage {
            attack: 3,
            ..Enrage::default()
        }),
        // Angry Chicken — Enrage: +5 Attack (1/1 → 6/1)
        "NEUTRAL_R02" => Some(Enrage {
            attack: 5,
            ..Enrage::default()
        }),
        // Spiteful Smith — Enrage: your weapon has +2 Attack
        "NEUTRAL_C15" => Some(Enrage {
            weapon_attack: 2,
            ..Enrage::default()
        }),
        // Undercover Cultist (M2-W4a) — Taunt, Reborn. Enrage: +3 Attack
        "TLC_101" => Some(Enrage {
            attack: 3,
            ..Enrage::default()
        }),
        _ => None,
    };
    if let Some(enrage) = enrage {
        world.set_enrage(entity, enrage);
    }
    // Gurubashi Berserker is deliberately NOT an Enrage minion: its real text
    // is "Whenever this minion takes damage, gain +3 Attack", a permanent buff
    // that stacks per damage instance and survives a full heal.
    if card_def.id == "NEUTRAL_B19" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::ThisMinionDamaged,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GainStats {
                    attack: 3,
                    health: 0,
                    target: EffectTarget::Self_,
                },
            },
        );
    }
    if card_def.id == "NEUTRAL_R16" {
        // Emperor Cobra — Poison
        world.set_poison(entity, Poison);
    }
    if card_def.id == "EDR_110" {
        // Sporegnasher (M1-W4a) — Poisonous
        world.set_poison(entity, Poison);
    }
    if matches!(card_def.id, "LEGENDARY_024" | "CORE_EX1_100") {
        // Lorewalker Cho — whenever a player casts a spell, put a copy into
        // the other player's hand. Pool-open (reads the cast spell — it can
        // move a card across a pool boundary); registered here by ID because
        // CardDef has no generic trigger field. The registry check below
        // keeps the closure invariant testable: this ID list is the second
        // source of truth the sets.rs closure test folds in.
        debug_assert!(
            crate::cards::sets::POOL_OPEN_CARDS.contains(&card_def.id),
            "pool-open trigger ({}) requires a POOL_OPEN_CARDS row",
            card_def.id
        );
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::AnySpellCast,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::CopyCastSpellToOtherPlayerHand,
            },
        );
    }
    // Race-conditioned triggers (fidelity-debt W1): the Trigger carries the
    // race requirement — the event's subject must match for the trigger to fire.
    let race_trigger: Option<(TriggerEvent, CardEffect)> = match card_def.id {
        // Murloc Tidecaller — whenever you summon a Murloc, gain +1 Attack
        "NEUTRAL_R05" => Some((
            TriggerEvent::FriendlyMinionSummoned,
            CardEffect::GainStats {
                attack: 1,
                health: 0,
                target: EffectTarget::Self_,
            },
        )),
        // Starving Buzzard — whenever you summon a Beast, draw a card
        "HUNTER_014" => Some((
            TriggerEvent::FriendlyMinionSummoned,
            CardEffect::DrawCard { count: 1 },
        )),
        // Scavenging Hyena — whenever a friendly Beast dies, gain +2/+1
        "HUNTER_013" => Some((
            TriggerEvent::FriendlyMinionDied,
            CardEffect::GainStats {
                attack: 2,
                health: 1,
                target: EffectTarget::Self_,
            },
        )),
        _ => None,
    };
    if let Some((event, effect)) = race_trigger {
        use crate::core::component::Race;
        let race = match card_def.id {
            "NEUTRAL_R05" => Race::Murloc,
            "HUNTER_014" | "HUNTER_013" => Race::Beast,
            _ => unreachable!(),
        };
        world.set_trigger(
            entity,
            Trigger {
                event,
                timing: TriggerTiming::Whenever,
                effect,
                race: Some(race),
                max_attack: None,
            },
        );
    }
    // W2 trigger classes (fidelity-debt): heal / card-played / secret-played /
    // any-minion-died — registered per card ID via the unified Trigger component.
    let w2_trigger: Option<(TriggerEvent, CardEffect)> = match card_def.id {
        // Lightwarden — whenever a character is healed, gain +2 Attack
        "NEUTRAL_R04" => Some((
            TriggerEvent::CharacterHealed,
            CardEffect::GainStats {
                attack: 2,
                health: 0,
                target: EffectTarget::Self_,
            },
        )),
        // Northshire Cleric — whenever a MINION is healed, draw a card.
        // Either player's minion counts; healing a hero does not.
        "PRIEST_004" => Some((
            TriggerEvent::MinionHealed,
            CardEffect::DrawCard { count: 1 },
        )),
        // Questing Adventurer — whenever you play a card, gain +1/+1
        "NEUTRAL_R17" => Some((
            TriggerEvent::CardPlayed,
            CardEffect::GainStats {
                attack: 1,
                health: 1,
                target: EffectTarget::Self_,
            },
        )),
        // Secretkeeper — whenever a Secret is played, gain +1/+1
        "NEUTRAL_R06" => Some((
            TriggerEvent::SecretPlayed,
            CardEffect::GainStats {
                attack: 1,
                health: 1,
                target: EffectTarget::Self_,
            },
        )),
        // Flesheating Ghoul — whenever a minion dies, gain +1 Attack
        "NEUTRAL_C12" => Some((
            TriggerEvent::MinionDied,
            CardEffect::GainStats {
                attack: 1,
                health: 0,
                target: EffectTarget::Self_,
            },
        )),
        // The Black Blood (M4-W1 — the Cataclysm Colossal wave): "After
        // you restore Health to a character, attack a random enemy
        // minion." The trigger rides the global CharacterHealed class —
        // it fires on ANY heal on either side (the official "you
        // restore" scope is approximated, §23); the excess-damage attack
        // model is the Briarspawn Drake convention.
        "CATA_300" => Some((
            TriggerEvent::CharacterHealed,
            CardEffect::AttackRandomEnemyMinionExcess,
        )),
        // M4-W4 — the Cataclysm closing wave.
        // CATA_130 Wickerstone Elemental — "After you spend all your
        // Mana Crystals, gain +1/+1" (the LastManaCrystalSpent event,
        // fired by the CardPlayed handler).
        "CATA_130" => Some((
            TriggerEvent::LastManaCrystalSpent,
            CardEffect::GainStats {
                attack: 1,
                health: 1,
                target: EffectTarget::Self_,
            },
        )),
        // CATA_469 Chromatic Broodmother — "Whenever this attacks,
        // refresh your Mana Crystals equal to its Attack" (Rush rides the
        // keyword table; the attack value is read at fire time).
        "CATA_469" => Some((
            TriggerEvent::Attacked,
            CardEffect::RefreshManaEqualSelfAttack,
        )),
        // CATA_487 Raincaller — "After you deal damage with a spell,
        // gain +2 Attack" (the once-per-turn flag rides the player).
        "CATA_487" => Some((
            TriggerEvent::FriendlySpellDealtDamage,
            CardEffect::GainAttackFirstSpellDamageThisTurn { attack: 2 },
        )),
        // CATA_494 Maloriak — "After you discard a minion, summon a
        // copy of it" (the minion gate lives in the effect arm).
        "CATA_494" => Some((
            TriggerEvent::FriendlyDiscarded,
            CardEffect::SummonCopyOfDiscardedMinion,
        )),
        // CATA_586 Conflagration — "After this survives damage, summon
        // a copy of this" (SurvivedDamage — the M3-W2a survivor event).
        "CATA_586" => Some((TriggerEvent::SurvivedDamage, CardEffect::SummonCopyOfSelf)),
        // CATA_786 Madder Bomber — "After you cast a spell, cast a
        // random spell of the same Cost from another class" (the
        // same-cost re-cast arm, §26).
        "CATA_786" => Some((
            TriggerEvent::FriendlySpellCast,
            CardEffect::CastRandomSpellSameCostOtherClass,
        )),
        // CATA_527 Nespirah — "After you play a Fel spell, reopen a
        // friendly Location" (the Fel gate lives in the effect arm).
        "CATA_527" => Some((
            TriggerEvent::FriendlySpellCast,
            CardEffect::ReopenLocationIfFelSpell,
        )),
        // CATA_527t2 Nespirah, Unshackled — "After you cast a Fel spell,
        // get a random non-Colossal Naga. It costs (1)."
        "CATA_527t2" => Some((
            TriggerEvent::FriendlySpellCast,
            CardEffect::AddRandomNagaCost1,
        )),
        _ => None,
    };
    if let Some((event, effect)) = w2_trigger {
        world.set_trigger(
            entity,
            Trigger {
                event,
                timing: TriggerTiming::Whenever,
                effect,
                race: None,
                max_attack: None,
            },
        );
    }
    // W9 weapon-attack triggers (fidelity-debt): the trigger rides the weapon
    // entity — trigger_applies pins `Attacked`/`AttackedMinion` to the
    // attacker OR the attacking hero's equipped weapon.
    let weapon_trigger: Option<(TriggerEvent, CardEffect)> = match card_def.id {
        // Truesilver Champion — whenever your hero attacks, restore 2 Health
        "PALADIN_006" => Some((
            TriggerEvent::Attacked,
            CardEffect::RestoreHealth {
                amount: 2,
                target: EffectTarget::FriendlyHero,
            },
        )),
        // Gorehowl — attacking a minion costs 1 Attack instead of 1 Durability
        "WARRIOR_009" => Some((
            TriggerEvent::AttackedMinion,
            CardEffect::BuffWeapon {
                attack: -1,
                durability: 0,
            },
        )),
        // Eaglehorn Bow — +1 Durability whenever a friendly Secret is revealed
        "HUNTER_007" => Some((
            TriggerEvent::FriendlySecretRevealed,
            CardEffect::BuffWeapon {
                attack: 0,
                durability: 1,
            },
        )),
        // The Everbloom (TLC_239t, M2-W2 — the Un'Goro quest reward weapon):
        // whenever your hero attacks, give your other minions +2/+2.
        "TLC_239t" => Some((
            TriggerEvent::Attacked,
            CardEffect::GainStats {
                attack: 2,
                health: 2,
                target: EffectTarget::AllFriendlyMinions,
            },
        )),
        // Axe of the Forefathers (M2-W4a) — after your hero attacks, deal
        // 1 damage to all minions
        "TLC_478" => Some((
            TriggerEvent::Attacked,
            CardEffect::DealDamage {
                amount: 1,
                target: EffectTarget::AllMinions,
            },
        )),
        // Insect Claw (M2-W4a) — after your hero attacks, summon a 2/1
        // Grub with Rush (TLC_903t, the W3 token)
        "TLC_833" => Some((
            TriggerEvent::Attacked,
            CardEffect::SummonMinion {
                card_id: "TLC_903t",
            },
        )),
        // M4-W4 — CATA_467 Command Claw: "Whenever your hero attacks,
        // give a random friendly minion +2 Attack."
        "CATA_467" => Some((
            TriggerEvent::Attacked,
            CardEffect::GrantRandomFriendlyMinionAttack { attack: 2 },
        )),
        // M5-W2 — the Violet Hold closing wave (PR #162): the weapon
        // triggers of Truth Seeker, Corpse Cannon, Tiny Pal's ammunition,
        // Stardust Scythe and Staff of Trickery.
        "JAIL_329" => Some((TriggerEvent::Attacked, CardEffect::TruthSeeker)),
        "JAIL_450" => Some((
            TriggerEvent::Attacked,
            CardEffect::SummonMinion {
                card_id: "JAIL_440t",
            },
        )),
        "JAIL_458t1" => Some((TriggerEvent::Attacked, CardEffect::TinyPal { ammo: 1 })),
        "JAIL_458t2" => Some((TriggerEvent::Attacked, CardEffect::TinyPal { ammo: 2 })),
        "JAIL_458t3" => Some((TriggerEvent::Attacked, CardEffect::TinyPal { ammo: 3 })),
        "JAIL_458t4" => Some((TriggerEvent::Attacked, CardEffect::TinyPal { ammo: 4 })),
        "JAIL_730" => Some((
            TriggerEvent::Attacked,
            CardEffect::AddCardToHand {
                card_id: "JAIL_732",
            },
        )),
        "JAIL_875" => Some((TriggerEvent::Attacked, CardEffect::StaffOfTrickery)),
        _ => None,
    };
    if let Some((event, effect)) = weapon_trigger {
        world.set_trigger(
            entity,
            Trigger {
                event,
                timing: TriggerTiming::Whenever,
                effect,
                race: None,
                max_attack: None,
            },
        );
    }
    // M5-W2 — the Violet Hold closing wave (PR #162): the per-card
    // trigger attachments. The CardPlayed riders fire on ANY card play;
    // the GallagioGoon / BlackMarketOverseer / MaievBuffDormant arms
    // decide the subject condition inside the effect.
    match card_def.id {
        // Rioter — after a friendly minion survives damage, give it +1
        // Attack (fires at damage application — the dying-minion pin is
        // §28)
        "JAIL_029" => world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionDamaged,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GainStats {
                    attack: 1,
                    health: 0,
                    target: EffectTarget::EventSubject,
                },
            },
        ),
        // Escape Artist — after this attacks (and survives, §28), draw a
        // card and move this to the Set-Aside zone
        "JAIL_030" => world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::Attacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::EscapeArtist,
            },
        ),
        // Rat Burglar — at the end of your turn, steal a card from the
        // top of the enemy deck (§28: the draw-card machinery moves a
        // random deck card)
        "JAIL_205" => world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::TurnEnd,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::RatBurglar,
            },
        ),
        // Tower of Ghouls — after this takes damage, summon two Frail
        // Ghouls
        "JAIL_440" => world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::ThisMinionDamaged,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::SummonMultipleMinions {
                    card_id: "JAIL_440t",
                    count: 2,
                },
            },
        ),
        // Arachnathid — whenever you summon a minion, give it Poisonous
        "JAIL_459" => world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GrantKeyword {
                    keyword: KeywordKind::Poisonous,
                    target: EffectTarget::EventSubject,
                },
            },
        ),
        // Black Market Auctioneer — after you cast a spell, draw a card
        "JAIL_718" => world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlySpellCast,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::DrawCard { count: 1 },
            },
        ),
        // Gallagio Goon — after you play a Battlecry minion, give it +1/+1
        "JAIL_802" => world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::CardPlayed,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GallagioGoon,
            },
        ),
        // Warden Maiev — after you play a minion, give it +3/+3 and make
        // it Dormant for 1 turn
        "JAIL_850" => world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::CardPlayed,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::MaievBuffDormant,
            },
        ),
        // Spider Rider — after your hero attacks, draw a card
        "JAIL_872" => world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::HeroAttacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::DrawCard { count: 1 },
            },
        ),
        // Black Market Overseer — after you play a Deathrattle minion,
        // give it Rush
        "JAIL_880" => world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::CardPlayed,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::BlackMarketOverseer,
            },
        ),
        // Activated Golem — at the end of each turn, gain Reborn
        "JAIL_883" => world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::TurnEnd,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::ActivatedGolem,
            },
        ),
        // Zuramat the Obliterator — at the end of your turn, play a card
        // discarded by Zuramat's Prison
        "JAIL_887t2" => world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::TurnEnd,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::ZuramatPlaysDiscarded,
            },
        ),
        // Irida Sinseeker — at the start of your turns, get two cards
        // from the Void (§28: the retrieval stops if Irida leaves play)
        "JAIL_719" => world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::TurnStart,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::IridaGetVoid,
            },
        ),
        _ => {}
    }
    // Mana Addict (W6) — whenever you cast a spell, gain +2 Attack THIS TURN
    // (the temporary buff expires at the end of the turn)
    if card_def.id == "NEUTRAL_R10" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlySpellCast,
                timing: TriggerTiming::Whenever,
                effect: CardEffect::GainStatsThisTurn {
                    attack: 2,
                    health: 0,
                    target: EffectTarget::Self_,
                },
                race: None,
                max_attack: None,
            },
        );
    }
    // M1-W4a (2025–2026 expansions) — weapon triggers (the trigger rides the
    // weapon entity, like the W9 Truesilver/Gorehowl pattern):
    // Ursine Maul — after your hero attacks, draw a card
    // Shepherd's Crook — after your hero attacks, summon a 3/3 Sheep
    // Defiled Spear — after your hero attacks an enemy minion, splash
    //   (dealt as direct damage; the "another" exclusion is implemented —
    //   the splashed target excludes the minion that was just attacked)
    if card_def.id == "EDR_253" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::Attacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::DrawCard { count: 1 },
            },
        );
    }
    if card_def.id == "EDR_416" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::Attacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::SummonMinion {
                    card_id: "EDR_416t",
                },
            },
        );
    }
    if card_def.id == "EDR_842" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::HeroAttackedMinion,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::SplashHeroAttackToRandomEnemy,
            },
        );
    }
    // Scavenging Flytrap — after ANY minion dies, gain its Attack (the
    // graveyard wipe means the base Attack is what is gained, §14.3)
    if card_def.id == "EDR_484" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::MinionDied,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GainDeadMinionAttack,
            },
        );
    }
    // Twisted Webweaver — whenever you play a minion you've already played
    // this game, draw a card (the played-minion log lives on the Player)
    if card_def.id == "EDR_540" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::CardPlayed,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::DrawIfMinionPlayedBefore,
            },
        );
    }
    // Dreambound Raptor — after you play a minion, give it a random Bonus
    // Effect (the official pool approximated by a fixed keyword pool, §14.3)
    if card_def.id == "EDR_849" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::CardPlayed,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GrantRandomBonusEffect,
            },
        );
    }
    // M2-W4a (2025–2026 expansions — the Un'Goro main-set wave) per-card
    // triggers. Windswept Pageturner rides the race-conditioned
    // FriendlyMinionSummoned trigger (the Murloc Tidecaller pattern): the
    // summoned minion must be an Elemental — the pageturner's own play is
    // excluded by the event semantics, matching "After you summon an
    // Elemental" (a minion's summon trigger never fires on its own play).
    if card_def.id == "TLC_220" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                race: Some(crate::core::component::Race::Elemental),
                max_attack: None,
                effect: CardEffect::DealDamage {
                    amount: 3,
                    target: EffectTarget::AnyEnemy,
                },
            },
        );
    }
    // Steadfast Security (City Defenses' token) — whenever this takes
    // damage, gain +1 Attack
    if card_def.id == "TLC_622t" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::ThisMinionDamaged,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::GainStats {
                    attack: 1,
                    health: 0,
                    target: EffectTarget::Self_,
                },
            },
        );
    }
    // M4-W1 — Plume of Vulcanos (the Cataclysm Colossal wave):
    // "Whenever this takes damage, get a random Fire spell. It costs (3)
    // less."
    if matches!(card_def.id, "CATA_488t" | "CATA_488t2") {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::ThisMinionDamaged,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::AddRandomFireSpellCostsLess { reduction: 3 },
            },
        );
    }
    // Gorishi Wasp — whenever this takes damage, get a 1-Cost Gorishi
    // Stinger (the same token Infestation creates)
    if card_def.id == "TLC_630" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::ThisMinionDamaged,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::AddCardToHandCount {
                    card_id: "TLC_630t",
                    count: 1,
                },
            },
        );
    }
    // Gorishi Tunneler — after this attacks, deal 2 damage to the enemy
    // hero (the Attacked trigger rides the attacker, like the Everbloom)
    if card_def.id == "TLC_840" {
        world.set_trigger(
            entity,
            Trigger {
                event: TriggerEvent::Attacked,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: CardEffect::DealDamage {
                    amount: 2,
                    target: EffectTarget::EnemyHero,
                },
            },
        );
    }
}

/// Human-readable branch labels for a Choose One card's two options
/// (2025–2026 expansions M1-W3, P3 — real choice resolution). The EDR W3
/// cards get their real labels distilled from the card text; Classic/Core
/// and anything else fall back to the generic labels. Used by the
/// choose-one surface sites in `engine/rules.rs` (spell / minion / weapon).
pub(crate) fn choose_one_option_names(def: &CardDef) -> [&'static str; 2] {
    match def.id {
        "EDR_209" => [
            "Give your other minions +1/+3",
            "Summon a 5/5 Ancient with Taunt",
        ],
        "EDR_233" => [
            "Summon three 2/3 Wolves with Taunt",
            "Summon two 4/3 Falcons with Windfury",
        ],
        "EDR_257" => ["+3 Attack and Divine Shield", "+3 Health and Lifesteal"],
        "EDR_263" => [
            "Deal 4 damage to the enemy hero",
            "Summon two 3/2 Wolves with Rush",
        ],
        "EDR_463" => [
            "Destroy a minion with 3 or less Attack",
            "Summon a random 2-Cost minion",
        ],
        "EDR_490" => [
            "Summon two 3/6 Demons with Taunt that can't attack",
            "Destroy an enemy minion",
        ],
        "EDR_525" => [
            "Gain Poisonous this turn",
            "Gain 'Deathrattle: Deal 2 damage to all enemies'",
        ],
        "EDR_570" => [
            "Deal 1 damage to all minions",
            "Give a damaged minion +2/+2",
        ],
        "EDR_813" => [
            "Summon two 1/1 Ants",
            "Spend 2 Corpses: Deal 4 damage to a minion",
        ],
        "EDR_820" => [
            "Summon two Dormant Dreadseeds",
            "Deal 2 damage to all minions",
        ],
        "EDR_843" => ["Draw a spell", "Draw a minion"],
        "EDR_872" => ["Discover a Mage spell", "Discover a Druid spell"],
        // M3-W2b — the Across the Timeways legendary wave
        "TIME_211" => ["Empower Zin-Azshari", "The Well of Eternity"],
        "TIME_619" => ["Boon of Power (Taunt)", "Boon of Longevity (Lifesteal)"],
        // M3-W3 — The End of Time miniset
        "END_010" => ["Set Attack to 1", "Set Health to 1"],
        // M5-W1 — Aya, Lotus Kingpin's Start-of-Game pick (JAIL_504):
        // the upgraded counterfeits.
        "JAIL_504" => ["Jade Coin", "Grimy Coin"],
        // M5-W2 — Secret Ingredient (the Violet Hold closing wave)
        "JAIL_201" => ["A Little of This", "A Dash of That"],
        _ => ["First option", "Second option"],
    }
}

/// The THIRD branch of a three-option Choose One card (2025–2026
/// expansions M2-W4a — the Un'Goro beasts). Option 0 resolves the
/// battlecry slot, option 1 the choose-one slot, option 2 this table —
/// the ChoiceResolved arm in `engine/rules.rs` falls through to it. Cards
/// without a third branch return None.
pub(crate) fn choose_one_three_branch(def: &CardDef) -> Option<CardEffect> {
    match def.id {
        // Ancient Stegodon — Battlecry: Choose to gain Taunt, Poisonous,
        // or +1/+1
        "TLC_242" => Some(CardEffect::GainStats {
            attack: 1,
            health: 1,
            target: EffectTarget::Self_,
        }),
        // Ancient Raptor — Battlecry: Choose to gain +3 Attack, Divine
        // Shield, or "Deathrattle: Summon two 1/1 Plants."
        "TLC_245" => Some(CardEffect::GrantDeathrattleSummon {
            card_id: "TLC_245t",
            count: 2,
            target: EffectTarget::Self_,
        }),
        // Ancient Pterrordax — Battlecry: Choose to gain Stealth until
        // your next turn, Elusive, or Windfury
        "TLC_246" => Some(CardEffect::GrantKeyword {
            keyword: crate::core::effect::KeywordKind::Windfury,
            target: EffectTarget::Self_,
        }),
        // M3-W2b — TIME_619 Talanji of the Graves' third boon (Boon of
        // Speed). The battlecry slot and choose-one slot carry the Taunt and
        // Lifesteal boons; the draw-or-resurrect Bwonsamdi half runs first
        // inside EVERY branch effect, so it resolves exactly once regardless
        // of the chosen option.
        "TIME_619" => Some(CardEffect::DrawOrResurrectBwonsamdiAndGrantBoon {
            keyword: crate::core::effect::KeywordKind::Rush,
        }),
        // M5-W1 — Aya, Lotus Kingpin's third counterfeit (JAIL_504): the
        // Kabal Coin (the battlecry slot = Jade Coin, the choose-one
        // slot = Grimy Coin — see the card's registration).
        "JAIL_504" => Some(CardEffect::AyaUpgradeCoins {
            card_id: "JAIL_504t3",
        }),
        _ => None,
    }
}

/// The label of a three-option Choose One card's third branch (M2-W4a —
/// the Un'Goro beasts; the `choose_one_option_names` pair covers the
/// first two). The choose-one surface sites in `engine/rules.rs` append
/// it when present.
pub(crate) fn choose_one_three_option_names(def: &CardDef) -> Option<&'static str> {
    match def.id {
        "TLC_242" => Some("Gain +1/+1"),
        "TLC_245" => Some("Deathrattle: Summon two 1/1 Plants"),
        "TLC_246" => Some("Gain Windfury"),
        "TIME_619" => Some("Boon of Speed (Rush)"),
        "JAIL_504" => Some("Kabal Coin"),
        _ => None,
    }
}

/// Clears all effect components of a minion (resets the entity before transform).
///
/// Keeps base components (Health/Attack/zone, etc.); the caller re-applies the target card's attributes.
pub(crate) fn clear_minion_effects(world: &mut World, entity: Entity) {
    world.remove_battlecry(entity);
    world.remove_deathrattle(entity);
    world.remove_taunt(entity);
    world.remove_aura(entity);
    world.remove_secret(entity);
    world.remove_divine_shield(entity);
    world.remove_windfury(entity);
    world.remove_charge(entity);
    world.remove_spell_damage(entity);
    world.remove_cant_attack(entity);
    world.remove_trigger(entity);
    world.remove_choose_one_effect(entity);
    // Enchantments and damage (roadmap G4): transform resets to the new base
    world.remove_enchantments(entity);
    world.remove_damage(entity);
    world.remove_combo_effect(entity);
    world.remove_attack_equals_health(entity);
    world.remove_poison(entity);
    world.remove_enrage(entity);
    world.remove_stealth(entity);
    world.remove_immune(entity);
    world.remove_freeze(entity);
    world.remove_overload(entity);
}

/// Creates a card entity from a `CardDef` under the given player (zone not set).
///
/// Shared by `GameBuilder` and the effect system (e.g., King Mukla's Banana, random pools),
/// ensuring card entities in hand/deck carry the full set of components (Battlecry/Deathrattle/keywords, etc.).
pub(crate) fn spawn_card_from_def(world: &mut World, player: PlayerId, card: &CardDef) -> Entity {
    let e = world.spawn();
    world.set_card_id(e, CardId(card.id));
    world.set_health(e, Health(card.health));
    world.set_attack(e, Attack(card.attack));
    world.set_cost(e, Cost(card.cost));
    world.set_card_type(e, card.card_type);
    world.set_player(e, player);
    world.set_attacks_used(e, AttacksUsed(0));
    // Set weapon durability (if it is a weapon card); locations carry their
    // durability charges the same way (Core Set W8)
    if (card.card_type == crate::core::component::CardType::Weapon
        || card.card_type == crate::core::component::CardType::Location)
        && card.durability > 0
    {
        world.set_durability(e, Durability(card.durability));
    }
    // Set Divine Shield / Windfury / Charge / Spell Damage
    if card.divine_shield {
        world.set_divine_shield(e, crate::core::component::DivineShield);
    }
    if card.windfury {
        world.set_windfury(e, crate::core::component::Windfury);
    }
    if card.charge {
        world.set_charge(e, crate::core::component::Charge);
    }
    if card.spell_damage != 0 {
        world.set_spell_damage(e, crate::core::component::SpellDamage(card.spell_damage));
    }
    // Set aura (if any)
    if let Some((aura_effect, aura_target)) = card.aura {
        world.set_aura(
            e,
            Aura {
                effect: aura_effect,
                target: aura_target,
            },
        );
    }
    // Set Battlecry / Deathrattle (existing fields)
    if let Some(bc) = card.battlecry {
        world.set_battlecry(e, crate::core::component::Battlecry(bc));
    }
    if let Some(dr) = card.deathrattle {
        world.set_deathrattle(e, Deathrattle(dr));
    }
    // Set Stealth (roadmap M5: card-level stealth — the mechanic existed
    // (F2) but CardDef had no field, so stealth cards were defined vanilla)
    if card.stealth {
        world.set_stealth(e, crate::core::component::Stealth);
    }
    // Set Elusive (roadmap M5: cannot be targeted by spells/hero powers)
    if card.elusive {
        world.set_elusive(e, crate::core::component::Elusive);
    }
    // Set Taunt
    if card.taunt {
        world.set_taunt(e, crate::core::component::Taunt);
    }
    // Set Race / tribe (fidelity-debt W1)
    if let Some(race) = card.race {
        world.set_race(e, race);
    }
    // Set cannot-attack
    if card.cant_attack {
        world.set_cant_attack(e, crate::core::component::CantAttack);
    }
    // Register triggers (roadmap G2): CardDef trigger fields map to the unified
    // Trigger component, fired in play order with current-player precedence.
    if let Some(ete) = card.end_turn_effect {
        world.set_trigger(
            e,
            Trigger {
                event: TriggerEvent::TurnEnd,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: ete,
            },
        );
    }
    if let Some(ste) = card.start_turn_effect {
        world.set_trigger(
            e,
            Trigger {
                event: TriggerEvent::TurnStart,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: ste,
            },
        );
    }
    // Spell card effects are stored in the battlecry component (resolved by the engine when played)
    if let Some(se) = card.spell_effect {
        world.set_battlecry(e, crate::core::component::Battlecry(se));
    }
    if let Some(st) = card.spell_trigger {
        world.set_trigger(
            e,
            Trigger {
                event: TriggerEvent::FriendlySpellCast,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: st,
            },
        );
    }
    if let Some(dt) = card.death_trigger {
        world.set_trigger(
            e,
            Trigger {
                event: TriggerEvent::FriendlyMinionDied,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: dt,
            },
        );
    }
    if let Some(st) = card.summon_trigger {
        world.set_trigger(
            e,
            Trigger {
                event: TriggerEvent::FriendlyMinionSummoned,
                timing: TriggerTiming::Whenever,
                race: None,
                max_attack: None,
                effect: st,
            },
        );
    }
    // Choose One effect
    if let Some(ce) = card.choose_one_effect {
        world.set_choose_one_effect(e, crate::core::component::ChooseOneEffect(ce));
    }
    // Combo effect
    if let Some(cb) = card.combo_effect {
        world.set_combo_effect(e, crate::core::component::ComboEffect(cb));
    }
    // Attack equals Health
    if card.attack_equals_health {
        world.set_attack_equals_health(e, crate::core::component::AttackEqualsHealth);
    }
    // Special keywords (Poison/Stealth, etc.)
    apply_card_keywords(world, e, card);
    e
}

#[cfg(test)]
mod generated_tests {
    use crate::cards::generated;
    use crate::cards::sets;
    use crate::cards::sets::ALL_CARDS;

    /// Cards whose official (post-rebalance) static attributes intentionally
    /// differ from the handwritten 2014-classic pool — the rebalance ledger
    /// (roadmap E4). Each entry lists the fields that differ.
    fn known_rebalanced(name: &str, field: &str) -> bool {
        let fields: &[&str] = match name {
            "Abusive Sergeant" => &["attack"],
            "Acolyte of Pain" => &["health"],
            "Al'Akir the Windlord" => &["health"],
            "Ancient of Lore" => &["attack", "health"],
            "Arcane Golem" => &["health"],
            "Argent Protector" => &["attack"],
            "Assassin's Blade" => &["attack", "cost", "durability"],
            "Assassinate" => &["cost"],
            "Azure Drake" => &["health"],
            "Baron Geddon" => &["health"],
            "Barrens Stablehand" => &["attack", "cost", "health"],
            "Blizzard" => &["cost"],
            "Brightwing" => &["cost"],
            "Cairne Bloodhoof" => &["attack", "taunt"],
            "Cenarius" => &["cost"],
            "Cone of Cold" => &["cost"],
            "Consecration" => &["cost"],
            "Cruel Taskmaster" => &["health"],
            "Defender of Argus" => &["attack", "taunt"],
            "Defias Ringleader" => &["attack"],
            "Druid of the Claw" => &["cost", "health"],
            "Earth Elemental" => &["health"],
            "Fan of Knives" => &["cost"],
            "Force of Nature" => &["cost"],
            "Gadgetzan Auctioneer" => &["cost"],
            "Gnoll" => &["cost"],
            "Guardian of Kings" => &["health", "taunt"],
            "Hammer of Wrath" => &["cost"],
            "Hellfire" => &["cost"],
            "Holy Nova" => &["cost"],
            "Holy Wrath" => &["cost"],
            "Ironbeak Owl" => &["cost"],
            "King Mukla" => &["health"],
            "Lay on Hands" => &["cost"],
            "Lightspawn" => &["cost", "health"],
            "Lord Jaraxxus" => &["attack", "card_type", "cost", "health"],
            "Malygos" => &["spell_damage"],
            "Mind Control" => &["cost"],
            "Mirror Image" => &["card_type", "cost", "health", "taunt"],
            "Misdirection" => &["cost"],
            "Misha" => &["attack"],
            "Natalie Seline" => &["attack", "cost", "health"],
            "Patient Assassin" => &["health"],
            "Prophet Velen" => &["spell_damage"],
            "Savannah Highmane" => &["attack"],
            "Shadow Madness" => &["cost"],
            "Shadow Word: Death" => &["cost"],
            "Shield Block" => &["cost"],
            "Siphon Soul" => &["cost"],
            "Slam" => &["cost"],
            "Soul of the Forest" => &["cost"],
            "Southsea Deckhand" => &["charge"],
            "Sprint" => &["cost"],
            "Stormwind Champion" => &["attack", "health"],
            "Swipe" => &["cost"],
            "Temple Enforcer" => &["attack", "cost"],
            "The Black Knight" => &["cost", "health"],
            "Timber Wolf" => &["health"],
            "Tirion Fordring" => &["attack", "health"],
            "Thoughtsteal" => &["cost"],
            "Treant" => &["charge"],
            "Unbound Elemental" => &["attack"],
            "Void Terror" => &["health"],
            "Xavius" => &["attack", "cost", "health"],
            "Baine Bloodhoof" => &["attack", "cost"],
            "Panther" => &["cost"],
            "Big Game Hunter" => &["cost"],
            "Spellbender" => &["attack", "card_type", "cost", "health"],
            // Core Set W8 — special types: the generated data maps LOCATION
            // and ENCHANTMENT to Minion, so the type (and the location's
            // durability-in-health slot) diverge by design.
            "Sanguine Depths" => &["card_type", "health", "durability"],
            // "Windfury" also collides by name with the Classic spell CS2_039
            // (2 mana, give Windfury) — both fields diverge for the token.
            "Windfury" => &["card_type", "cost"],
            "Thornspeakers' Spirit" => &["card_type"],
            "Deathly Poison" => &["card_type"],
            "Emerald Drake" => &["attack", "health"],
            "Laughing Sister" => &["cost"],
            "Huffer" => &["charge", "health"],
            "Leokk" => &["attack", "health"],
            // 2025–2026 expansions M0.3: Core Set reprints of expansion
            // originals are rebalances by design — Babbling Bookcase is the
            // 2/4 Core version of the original 3/3 EDR_001.
            "Babbling Bookcase" => &["attack", "health"],
            _ => &[],
        };
        fields.contains(&field)
    }

    /// The official database (roadmap E4) covers the classic-era constructed
    /// sets. The handwritten pool uses custom IDs (CLASSIC_001, NEUTRAL_...)
    /// while the official data uses Blizzard IDs (CS2_172, EX1_...), so the
    /// verification matches by NAME: every handwritten card with an official
    /// counterpart must agree on the statically representable fields, except
    /// for the documented rebalances (known_rebalanced).
    ///
    /// Core Set W0 (decision D2): a `CORE_<P>_<n>` card's counterpart is its
    /// ID without the `CORE_` prefix (`CORE_EX1_100` ↔ generated `EX1_100`),
    /// matched by ID first — name matching would collide with the Classic
    /// reprint of the same card. Classic cards keep matching by name.
    #[test]
    fn generated_cards_match_handwritten() {
        assert!(
            !generated::GENERATED_IDS.is_empty(),
            "generated registry must be non-empty"
        );
        let mut compared = 0;
        let mut rebalanced = 0;
        for card in ALL_CARDS {
            let generated = card
                .id
                .strip_prefix("CORE_")
                .and_then(generated::find_by_id)
                .or_else(|| generated::find_by_name(card.name));
            let Some(generated) = generated else {
                continue; // no official counterpart (custom tokens)
            };
            compared += 1;
            for (field, generated_value, handwritten_value) in [
                ("card_type", generated.card_type == card.card_type, true),
                ("cost", generated.cost == card.cost, true),
                ("attack", generated.attack == card.attack, true),
                ("health", generated.health == card.health, true),
                ("durability", generated.durability == card.durability, true),
                ("taunt", generated.taunt == card.taunt, true),
                (
                    "divine_shield",
                    generated.divine_shield == card.divine_shield,
                    true,
                ),
                ("windfury", generated.windfury == card.windfury, true),
                ("charge", generated.charge == card.charge, true),
                (
                    "spell_damage",
                    generated.spell_damage == card.spell_damage,
                    true,
                ),
            ] {
                let _ = handwritten_value;
                if !generated_value {
                    assert!(
                        known_rebalanced(card.name, field),
                        "{field} mismatch: {} (rebalance not documented)",
                        card.name
                    );
                    rebalanced += 1;
                }
            }
        }
        assert!(
            compared > 100,
            "name-based verification should cover a meaningful share of the handwritten pool (got {compared})"
        );
        // Sanity: the ledger documents a meaningful number of rebalances but
        // the vast majority of the pool matches exactly.
        assert!(
            compared - rebalanced > compared / 2,
            "too many rebalances documented"
        );
    }

    /// Whether a Core Set reprint of an expansion original is a documented
    /// rebalance (2025–2026 expansions M0.3): the Core version may differ
    /// from the original by design, e.g. Babbling Bookcase 2/4 vs 3/3.
    fn core_reprint_rebalanced(id: &str, field: &str) -> bool {
        match id {
            "CORE_EDR_001" => ["attack", "health"].contains(&field),
            _ => false,
        }
    }

    /// Core Set reprint fidelity (decision D2, 2026-08-08): every generated
    /// `CORE_<P>_<n>` card whose classic-era original `<P>_<n>` exists in the
    /// generated database must agree with it on the statically representable
    /// fields — the Core version is a reprint, not a rebalance (verified
    /// 2026-08-08: all 90 classic reprint pairs in `cards.json` match
    /// exactly). Reprints of 2025–2026 expansion originals are Core
    /// rebalances by design and land in `core_reprint_rebalanced`.
    #[test]
    fn core_reprints_match_originals() {
        let mut pairs = 0;
        let mut rebalanced = 0;
        for id in generated::GENERATED_IDS {
            let Some(original_id) = id.strip_prefix("CORE_") else {
                continue; // not a Core Set card
            };
            let Some(core_card) = generated::find_by_id(id) else {
                panic!("generated registry lists {id} but lookup fails");
            };
            let Some(original) = generated::find_by_id(original_id) else {
                continue; // no classic-era original (e.g. CORE_AV_* from newer sets)
            };
            pairs += 1;
            for (field, compare) in [
                ("card_type", core_card.card_type == original.card_type),
                ("cost", core_card.cost == original.cost),
                ("attack", core_card.attack == original.attack),
                ("health", core_card.health == original.health),
                ("durability", core_card.durability == original.durability),
                ("taunt", core_card.taunt == original.taunt),
                (
                    "divine_shield",
                    core_card.divine_shield == original.divine_shield,
                ),
                ("windfury", core_card.windfury == original.windfury),
                ("charge", core_card.charge == original.charge),
                (
                    "spell_damage",
                    core_card.spell_damage == original.spell_damage,
                ),
            ] {
                if compare {
                    continue;
                }
                assert!(
                    core_reprint_rebalanced(id, field),
                    "{field} mismatch: {id} vs {original_id} (rebalance not documented)"
                );
                rebalanced += 1;
            }
        }
        assert!(
            pairs > 50,
            "reprint-pair coverage should cover the Classic reprints (got {pairs})"
        );
        assert!(
            rebalanced < 10,
            "too many undocumented Core rebalances (got {rebalanced})"
        );
    }

    /// The generated database must be internally consistent: every ID in the
    /// registry resolves to a const.
    #[test]
    fn generated_lookup_round_trips() {
        for id in generated::GENERATED_IDS {
            let card = generated::find_by_id(id).expect("generated const for id");
            assert_eq!(card.id, *id);
        }
    }

    /// 2025–2026 expansions M0.2 — set membership registry and race backfill:
    /// classic-era set codes map to `CardSet::Classic`, `CORE` to `CardSet::Core`,
    /// unknown/custom IDs default to `Classic` (the custom-ID handwritten pool),
    /// and tribed minions carry their tribe on the generated const.
    #[test]
    fn generated_set_and_race_metadata() {
        use crate::cards::def::CardSet;
        use crate::core::component::Race;

        assert_eq!(generated::card_set("EX1_001"), CardSet::Classic); // Lightwarden (EXPERT1)
        assert_eq!(generated::card_set("CS2_172"), CardSet::Classic); // Bloodfen Raptor (LEGACY)
        assert_eq!(generated::card_set("CORE_AT_037"), CardSet::Core); // Living Roots (CORE)
        assert_eq!(generated::card_set("CORE_RLK_062"), CardSet::Core); // Nerubian Swarmguard (CORE)
        assert_eq!(generated::card_set("CLASSIC_001"), CardSet::Classic); // custom-ID fallback

        assert_eq!(
            generated::find_by_id("BT_142").map(|c| c.race),
            Some(Some(Race::Demon)) // Shadowhoof Slayer
        );
        assert_eq!(
            generated::find_by_id("CORE_AT_123").map(|c| c.race),
            Some(Some(Race::Dragon)) // Chillmaw
        );
        assert_eq!(
            generated::find_by_id("CORE_RLK_062").map(|c| c.race),
            Some(Some(Race::Undead)) // Nerubian Swarmguard
        );
        assert_eq!(
            generated::find_by_id("EX1_001").map(|c| c.race),
            Some(Some(Race::Draenei)) // Lightwarden
        );
        assert_eq!(
            generated::find_by_id("CORE_AT_037").map(|c| c.race),
            Some(None) // Living Roots — untribed spell
        );
    }

    /// 2025–2026 expansions M0.3 — per-set registration: the generated group
    /// consts enumerate the dumped card counts, `card_set` maps every
    /// expansion ID to its set, and `EXPANSION_CARDS` is engine-available via
    /// `card_by_id` while staying out of the sampling pools.
    #[test]
    fn expansion_sets_registered() {
        use crate::cards::def::CardSet;
        use crate::cards::sets;

        assert_eq!(generated::EMERALD_DREAM_CARDS.len(), 183);
        assert_eq!(generated::THE_LOST_CITY_CARDS.len(), 183);
        assert_eq!(generated::TIME_TRAVEL_CARDS.len(), 183);
        assert_eq!(generated::CATACLYSM_CARDS.len(), 164);
        assert_eq!(generated::ESCAPEFROM_VIOLET_HOLD_CARDS.len(), 135);
        assert_eq!(sets::EXPANSION_CARDS.len(), 848);

        assert_eq!(generated::card_set("EDR_000"), CardSet::EmeraldDream);
        assert_eq!(generated::card_set("DINO_130"), CardSet::TheLostCity);
        assert_eq!(generated::card_set("END_000"), CardSet::TimeTravel);
        assert_eq!(generated::card_set("CATA_111"), CardSet::Cataclysm);
        assert_eq!(
            generated::card_set("JAIL_007"),
            CardSet::EscapeFromVioletHold
        );

        // Engine-wide availability via card_by_id …
        assert!(crate::cards::card_by_id("EDR_000").is_some());
        assert!(crate::cards::card_by_id("JAIL_007").is_some());
        // … without entering ALL_CARDS (the sampling pools keep the current
        // training window until the D3 cut-over).
        assert!(!sets::ALL_CARDS.iter().any(|c| c.id == "EDR_000"));
        assert!(!sets::ALL_CARDS.iter().any(|c| c.id == "JAIL_007"));
    }

    /// 2025–2026 expansions M0.3, updated for the D3 cut-over (2026-08-09):
    /// `is_standard` marks Core + the five expansions; every pool-sampled
    /// card stays inside the active training window (Standard) — no Classic
    /// card is sampled anymore, and the pools can reach expansion cards.
    #[test]
    fn sampling_pools_keep_current_window() {
        use crate::cards::pool;
        use crate::core::effect::RandomPool;

        let edr = crate::cards::card_by_id("EDR_000").expect("expansion card");
        let core = generated::find_by_id("CORE_AT_037").expect("core card");
        let classic = generated::find_by_id("CS2_172").expect("classic card");
        assert!(pool::is_standard(edr));
        assert!(pool::is_standard(&core));
        assert!(!pool::is_standard(&classic));

        for pool_kind in [
            RandomPool::Beast,
            RandomPool::Demon,
            RandomPool::Dragon,
            RandomPool::Mechanical,
            RandomPool::Spell,
            RandomPool::Legendary,
            RandomPool::MageSpell,
            RandomPool::ShadowSpell,
            RandomPool::OtherClass,
        ] {
            let cards = pool::pool_cards(pool_kind);
            assert!(
                !cards.is_empty(),
                "{pool_kind:?} must not be empty after the D3 cut-over"
            );
            assert!(
                cards.iter().all(|c| pool::is_standard(c)),
                "{pool_kind:?} leaked a non-Standard card: {:?}",
                cards.iter().find(|c| !pool::is_standard(c)).map(|c| c.id)
            );
        }
        // The cut-over widened the pools: the Spell pool reaches expansions.
        let spell_pool = pool::pool_cards(RandomPool::Spell);
        assert!(
            spell_pool.iter().any(|c| pool::is_expansion(c)),
            "the Spell pool must reach expansion cards after the D3 cut-over"
        );
    }

    /// 2025–2026 expansions M0.4 — ID uniqueness: the generated registry has
    /// no duplicate IDs (two IDs mapping to one const name would silently
    /// misbehave), and the expansion IDs are disjoint from the handwritten
    /// pool (the `card_by_id` union must be unambiguous).
    #[test]
    fn expansion_ids_unique() {
        let mut seen = std::collections::HashSet::new();
        for id in generated::GENERATED_IDS {
            assert!(seen.insert(*id), "duplicate generated ID: {id}");
        }
        for card in sets::EXPANSION_CARDS {
            assert!(
                !sets::ALL_CARDS.iter().any(|c| c.id == card.id),
                "ID clash between expansion and handwritten pool: {}",
                card.id
            );
        }
    }

    /// 2025–2026 expansions M0.4 — differential gate: every expansion card is
    /// either hand-written (a member of the handwritten pool) or exactly
    /// matches its generated baseline. M1+ effect waves that hand-write an
    /// expansion card must keep this invariant — the handwritten const must
    /// agree with the generated baseline on every statically representable
    /// field, or the divergence must be documented in
    /// `expansion_differential_rebalanced`.
    fn expansion_differential_rebalanced(id: &str, field: &str) -> bool {
        // Documented divergences land here (known_rebalanced style).
        //
        // The two EDR Locations (M1-W4a) — EDR_454 Clutch of Corruption and
        // EDR_520 Forbidden Shrine — diverge from their generated baselines
        // because the generator predates the Location CardType (added in the
        // Core Set W8 work): cards.json marks them `type: LOCATION` with the
        // durability in the "health" field, which the generated code maps to
        // a vanilla Minion (0/2 resp. 0/3, durability 0). The handwritten
        // cards use the faithful location representation (CardType::Location,
        // health 0, durability from the official data) — the same convention
        // as the Core Set W8 locations.
        if id == "EDR_454" || id == "EDR_520" {
            return matches!(field, "card_type" | "health" | "durability");
        }
        // FIR_907 Amirdrassil (M1-W5) — the same Location divergence as the
        // EDR_454/EDR_520 pair above (the miniset wave predates the Location
        // CardType in the generator; the handwritten card is the faithful
        // Location 5-mana / 3-durability representation).
        if id == "FIR_907" {
            return matches!(field, "card_type" | "health" | "durability");
        }
        // TLC_449 Bloodpetal Biome (M2-W4a) — the same Location divergence
        // (the generator predates the Location CardType: the generated
        // baseline is a vanilla Minion; the handwritten card is the faithful
        // Location 1-mana / 2-durability representation).
        if id == "TLC_449" {
            return matches!(field, "card_type" | "health" | "durability");
        }
        // M3-W2a — the three Across the Timeways Locations (TIME_044 Past
        // Gnomeregan, TIME_436 Past Conflux, TIME_810 Past Silvermoon): the
        // same generator-predates-Location divergence as the EDR_454 pair
        // above. The generated baselines are vanilla Minions; the handwritten
        // cards are the faithful Location representations (health 0,
        // durability 3, activation in the battlecry slot).
        if id == "TIME_044" || id == "TIME_436" || id == "TIME_810" {
            return matches!(field, "card_type" | "health" | "durability");
        }
        // M3-W2b — TIME_446 The Eternal Hold: the same generator-predates-
        // Location divergence (the generated baseline is a vanilla Minion
        // 6-cost 0/3; the handwritten card is the faithful Location 6-mana
        // / 3-durability representation, activation in the battlecry slot).
        if id == "TIME_446" {
            return matches!(field, "card_type" | "health" | "durability");
        }
        // M3-W3 — END_022 Time-Twisted Seer: "Has Spell Damage +2 while
        // damaged". The handwritten card carries the full `spell_damage: 2`
        // (so the stats agree with the baseline and the enrage-style check
        // in `world::total_spell_damage` skips the bonus while undamaged);
        // the `spell_damage` field is rebalanced so the tripwire still
        // compares it — the gate normally excludes spell_damage entirely.
        if id == "END_022" {
            return matches!(field, "spell_damage");
        }
        // M4-W2 — CATA_492 Shrine of Twilight: the same
        // generator-predates-Location divergence as the EDR_454/TLC_449
        // pair above (the generated baseline is a vanilla Minion; the
        // handwritten card is the faithful Location 4-mana / 2-durability
        // representation, activation in the battlecry slot).
        if id == "CATA_492" {
            return matches!(field, "card_type" | "health" | "durability");
        }
        // M4-W4 — the four Cataclysm Locations (CATA_301 Ruby Sanctum,
        // CATA_477 Chamber of Aspects, CATA_527 Nespirah, Enthralled,
        // CATA_584 Erupting Volcano): the same generator-predates-Location
        // divergence as the CATA_492 pair above (the generated baselines
        // are vanilla Minions 0/3 resp. 0/2, 0/5, 0/3; the handwritten
        // cards are the faithful Location representations — health 0,
        // durability from the official data, activation in the battlecry
        // slot).
        if id == "CATA_301" || id == "CATA_477" || id == "CATA_527" || id == "CATA_584" {
            return matches!(field, "card_type" | "health" | "durability");
        }
        // M5-W2 — the four Violet Hold Locations (JAIL_511 Spire of
        // Solitude, JAIL_877 Underbelly Network, JAIL_887 Zuramat's
        // Prison, JAIL_987 Low Security Wing): the same
        // generator-predates-Location divergence as the CATA_492 pair
        // above (the generated baselines are vanilla Minions; the
        // handwritten cards are the faithful Location representations —
        // health 0, durability from the official data, activation in the
        // battlecry slot).
        if id == "JAIL_511" || id == "JAIL_877" || id == "JAIL_887" || id == "JAIL_987" {
            return matches!(field, "card_type" | "health" | "durability");
        }
        // M5-W2 — JAIL_890 Captive Nathrezim: the official data dump
        // lists only the AURA mechanic (the Taunt appears in the text
        // alone), so the generated baseline drops the keyword; the
        // handwritten card follows the text and carries Taunt.
        if id == "JAIL_890" {
            return field == "taunt";
        }
        false
    }

    /// The gate itself: enumerate the generated expansion baselines, compare
    /// any handwritten counterpart (a member of the handwritten pool or of
    /// `HANDWRITTEN_EXPANSION_CARDS` — the M1+ effect waves) field by field.
    #[test]
    fn expansion_differential_gate() {
        let mut hand_written = 0;
        for card in sets::EXPANSION_CARDS {
            let Some(handwritten) = sets::ALL_CARDS
                .iter()
                .chain(sets::HANDWRITTEN_EXPANSION_CARDS.iter())
                .find(|c| c.id == card.id)
            else {
                continue; // generated baseline in effect — nothing to compare
            };
            hand_written += 1;
            for (field, equal) in [
                ("card_type", handwritten.card_type == card.card_type),
                ("cost", handwritten.cost == card.cost),
                ("attack", handwritten.attack == card.attack),
                ("health", handwritten.health == card.health),
                ("durability", handwritten.durability == card.durability),
                ("taunt", handwritten.taunt == card.taunt),
                (
                    "divine_shield",
                    handwritten.divine_shield == card.divine_shield,
                ),
                ("windfury", handwritten.windfury == card.windfury),
                ("charge", handwritten.charge == card.charge),
                (
                    "spell_damage",
                    handwritten.spell_damage == card.spell_damage,
                ),
            ] {
                if equal {
                    continue;
                }
                assert!(
                    expansion_differential_rebalanced(card.id, field),
                    "{field} mismatch: {} (handwritten vs generated baseline, not documented)",
                    card.id
                );
            }
        }
        // M1-W1 tripwire: EVERY handwritten expansion card with a generated
        // baseline must have been compared above (a handwritten card missing
        // from the generated list would silently sit outside the gate — and
        // one whose fields diverged without a documented rebalance would
        // have tripped the field asserts). Handwritten-only tokens (the
        // M1-W1 EDR_847pt2 / EDR_851t / EDR_445pt3) have no baseline and are
        // excluded from the expectation.
        let with_baseline = sets::HANDWRITTEN_EXPANSION_CARDS
            .iter()
            .filter(|hw| sets::EXPANSION_CARDS.iter().any(|g| g.id == hw.id))
            .count();
        assert_eq!(
            hand_written, with_baseline,
            "every handwritten expansion card must be compared against its generated baseline"
        );
    }
}
