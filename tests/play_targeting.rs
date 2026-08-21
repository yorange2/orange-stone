//! Play-time targeting: the cards whose official text makes the player pick.
//!
//! Each test does the same two things, which together are what "the card is
//! targetable" means:
//!   1. the legal-action list offers one play per candidate target, and
//!   2. applying the play with a chosen target hits **that** target — not a
//!      random one from the same domain.
//!
//! Step 2 needs a board where a random pick would very likely differ from the
//! chosen one (several candidates, only one of which shows the effect), so a
//! regression back to random resolution fails the assertion rather than
//! passing by luck.
//!
//! Background: `docs/play-target-gap-audit.md` — these cards used to be played
//! untargeted because `play_targets` had no arm for their effect variant.

use orange_stone::core::action::Action;
use orange_stone::core::component::CardType;
use orange_stone::core::entity::Entity;
use orange_stone::core::player::PlayerId;
use orange_stone::core::state::GameState;
use orange_stone::core::zone::Zone;
use orange_stone::engine::game::GameEngine;
use orange_stone::sim::game::GameBuilder;

const P1: PlayerId = PlayerId::Player1;
const P2: PlayerId = PlayerId::Player2;

/// The card sitting in P1's hand.
fn card_in_hand(state: &GameState) -> Entity {
    state
        .world()
        .zones()
        .iter(Zone::Hand, P1)
        .next()
        .expect("card in hand")
}

/// Targets the legal-action list offers for playing `card`.
fn offered_targets(state: &GameState, card: Entity) -> Vec<Entity> {
    orange_stone::rl::env::legal_actions(state)
        .into_iter()
        .filter_map(|a| match a {
            Action::PlayCard { card: c, target, .. } if c == card => target,
            _ => None,
        })
        .collect()
}

fn play_at(state: &mut GameState, card: Entity, target: Entity) {
    GameEngine::new()
        .apply(
            state,
            Action::PlayCard {
                card,
                target: Some(target),
                position: None,
            },
        )
        .expect("play with an explicit target");
}

/// P1's minions in play order (the hero also sits in `Zone::Play`).
fn friendly_minions(state: &GameState) -> Vec<Entity> {
    state
        .world()
        .zones()
        .iter(Zone::Play, P1)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect()
}

fn attack_of(state: &GameState, e: Entity) -> i32 {
    state.world().effective_attack(e).map_or(0, |a| a.0)
}

fn damage_of(state: &GameState, e: Entity) -> i32 {
    state.world().damage(e).map_or(0, |d| d.0)
}

/// Abusive Sergeant / Dark Iron Dwarf / CORE Abusive Sergeant —
/// "Give a minion +2 Attack this turn." (`GainStatsThisTurn`)
#[test]
fn abusive_sergeant_buffs_the_chosen_friendly_minion() {
    use orange_stone::cards::def::ABUSIVE_SERGEANT;
    let mut builder = GameBuilder::new();
    let a = builder.add_custom_minion_to_board(P1, 1, 5, 1);
    let b = builder.add_custom_minion_to_board(P1, 1, 5, 1);
    let c = builder.add_custom_minion_to_board(P1, 1, 5, 1);
    builder.add_minion_to_hand(P1, &ABUSIVE_SERGEANT);
    builder.set_mana(P1, 10, 10);
    let mut state = builder.build();
    let card = card_in_hand(&state);

    let targets = offered_targets(&state, card);
    assert!(
        targets.contains(&a) && targets.contains(&b) && targets.contains(&c),
        "every friendly minion should be offered: {targets:?}"
    );

    play_at(&mut state, card, b);
    assert_eq!(attack_of(&state, b), 3, "the chosen minion gains +2 Attack");
    assert_eq!(attack_of(&state, a), 1, "the others are untouched");
    assert_eq!(attack_of(&state, c), 1, "the others are untouched");
}

/// Charge — "Give a friendly minion +2 Attack and Charge." (`GrantCharge`)
#[test]
fn charge_buffs_the_chosen_friendly_minion() {
    use orange_stone::cards::def::CHARGE_SPELL;
    let mut builder = GameBuilder::new();
    let a = builder.add_custom_minion_to_board(P1, 2, 5, 2);
    let b = builder.add_custom_minion_to_board(P1, 2, 5, 2);
    builder.add_minion_to_hand(P1, &CHARGE_SPELL);
    builder.set_mana(P1, 10, 10);
    let mut state = builder.build();
    let card = card_in_hand(&state);
    assert_eq!(offered_targets(&state, card).len(), 2);

    play_at(&mut state, card, a);
    assert_eq!(attack_of(&state, a), 4, "chosen minion gains +2 Attack");
    assert!(state.world().charge(a).is_some(), "and Charge");
    assert_eq!(attack_of(&state, b), 2, "the other minion is untouched");
}

/// Siphon Soul — "Destroy a minion. Restore 3 Health to your hero."
/// (`DestroyAndHeal`; the engine's domain is enemy minions.)
#[test]
fn siphon_soul_destroys_the_chosen_enemy_minion() {
    use orange_stone::cards::def::SIPHON_SOUL;
    let mut builder = GameBuilder::new();
    let a = builder.add_custom_minion_to_board(P2, 1, 3, 3);
    let b = builder.add_custom_minion_to_board(P2, 1, 3, 3);
    let c = builder.add_custom_minion_to_board(P2, 1, 3, 3);
    builder.add_minion_to_hand(P1, &SIPHON_SOUL);
    builder.set_mana(P1, 10, 10);
    let mut state = builder.build();
    let card = card_in_hand(&state);
    assert_eq!(offered_targets(&state, card).len(), 3);

    play_at(&mut state, card, b);
    let alive: Vec<Entity> = state.world().zones().iter(Zone::Play, P2).collect();
    assert!(!alive.contains(&b), "the chosen minion is destroyed");
    assert!(alive.contains(&a) && alive.contains(&c), "the others survive");
}

/// Demonfire — "Deal 2 damage to a minion. If it's a friendly Demon, give it
/// +2/+2 instead." (`Demonfire`; both sides' minions are candidates.)
#[test]
fn demonfire_hits_the_chosen_minion() {
    use orange_stone::cards::def::DEMONFIRE;
    let mut builder = GameBuilder::new();
    let mine = builder.add_custom_minion_to_board(P1, 2, 5, 1);
    let a = builder.add_custom_minion_to_board(P2, 2, 5, 1);
    let b = builder.add_custom_minion_to_board(P2, 2, 5, 1);
    builder.add_minion_to_hand(P1, &DEMONFIRE);
    builder.set_mana(P1, 10, 10);
    let mut state = builder.build();
    let card = card_in_hand(&state);

    let targets = offered_targets(&state, card);
    assert!(
        targets.contains(&mine) && targets.contains(&a) && targets.contains(&b),
        "minions on both sides are candidates: {targets:?}"
    );

    play_at(&mut state, card, b);
    assert_eq!(damage_of(&state, b), 2, "the chosen minion takes the damage");
    assert_eq!(damage_of(&state, a), 0);
    assert_eq!(damage_of(&state, mine), 0);
}

/// Mortal Strike — "Deal 4 damage." (`MortalStrike`; the engine's domain is
/// enemy characters, hero included.)
#[test]
fn mortal_strike_hits_the_chosen_enemy() {
    use orange_stone::cards::def::MORTAL_STRIKE;
    let mut builder = GameBuilder::new();
    let a = builder.add_custom_minion_to_board(P2, 1, 5, 5);
    let b = builder.add_custom_minion_to_board(P2, 1, 5, 5);
    builder.add_minion_to_hand(P1, &MORTAL_STRIKE);
    builder.set_mana(P1, 10, 10);
    let mut state = builder.build();
    let card = card_in_hand(&state);
    let enemy_hero = state.player(P2).hero;

    let targets = offered_targets(&state, card);
    assert!(
        targets.contains(&enemy_hero),
        "the enemy hero is a candidate: {targets:?}"
    );

    play_at(&mut state, card, a);
    assert_eq!(damage_of(&state, a), 4, "the chosen minion takes 4");
    assert_eq!(damage_of(&state, b), 0);
    assert_eq!(damage_of(&state, enemy_hero), 0);
}

/// Savagery / Savage Striker — "Deal damage equal to your hero's Attack to
/// a minion." (`DealHeroAttackDamage`)
#[test]
fn savagery_hits_the_chosen_enemy_minion() {
    use orange_stone::cards::def::SAVAGERY;
    use orange_stone::core::component::Attack;
    let mut builder = GameBuilder::new();
    let a = builder.add_custom_minion_to_board(P2, 1, 4, 4);
    let b = builder.add_custom_minion_to_board(P2, 1, 4, 4);
    builder.add_minion_to_hand(P1, &SAVAGERY);
    builder.set_mana(P1, 10, 10);
    let hero = builder.state_mut().player(P1).hero;
    builder.state_mut().world_mut().set_attack(hero, Attack(3));
    let mut state = builder.build();
    let card = card_in_hand(&state);
    assert_eq!(offered_targets(&state, card).len(), 2);

    play_at(&mut state, card, a);
    assert_eq!(damage_of(&state, a), 3, "hero attack lands on the chosen one");
    assert_eq!(damage_of(&state, b), 0);
}

/// Bestial Wrath — "Give a friendly Beast +2 Attack and Immune this turn."
/// (`GrantAttackAndImmune`; only friendly Beasts are candidates.)
#[test]
fn bestial_wrath_only_offers_friendly_beasts() {
    use orange_stone::cards::def::{ARGENT_SQUIRE, BESTIAL_WRATH, BLOODFEN_RAPTOR};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(P1, &BLOODFEN_RAPTOR); // Beast 3/2
    builder.add_minion_to_board(P1, &ARGENT_SQUIRE); // not a Beast
    builder.add_minion_to_hand(P1, &BESTIAL_WRATH);
    builder.set_mana(P1, 10, 10);
    let mut state = builder.build();
    let card = card_in_hand(&state);

    let board = friendly_minions(&state);
    let (beast, other) = (board[0], board[1]);

    let targets = offered_targets(&state, card);
    assert_eq!(targets, vec![beast], "non-Beasts are not candidates");

    play_at(&mut state, card, beast);
    assert_eq!(attack_of(&state, beast), 5, "the Beast gains +2 Attack");
    assert_eq!(attack_of(&state, other), 1, "the non-Beast is untouched");
}


// ─────────────────────────────────────────────────────────────────────────
// W2: the rest of T1 — the expansion cards whose resolution already accepts
// an explicit target. Covered two ways: one sweep asserting every card now
// offers targets, plus a per-domain "hits the chosen one" test.
// ─────────────────────────────────────────────────────────────────────────

/// Every card fixed in wave W2, with the domain it declares.
const W2_CARDS: &[(&str, &str)] = &[
    ("CATA_161", "friendly"),
    ("CATA_552", "any-character"),
    ("CATA_552t", "any-character"),
    ("CATA_564", "friendly"),
    ("CATA_699", "enemy-minion"),
    ("EDR_860", "any-minion"),
    ("EDR_252", "any-minion"),
    ("EDR_261", "any-minion"),
    ("EDR_262", "any-minion"),
    ("EDR_460", "any-minion"),
    ("EDR_523", "friendly"),
    ("EDR_531", "friendly"),
    ("FIR_908", "friendly"),
    ("FIR_918", "any-minion"),
    ("FIR_939", "enemy"),
    ("FIR_954", "any-minion"),
    ("JAIL_998", "friendly"),
    ("TLC_230", "any-minion"),
    ("TLC_252", "friendly"),
    ("TLC_441", "friendly"),
    ("TLC_606", "enemy-minion"),
    ("TLC_620", "enemy-minion"),
    ("TLC_823", "any-minion"),
    ("TLC_901", "any-minion"),
    ("TLC_987", "enemy-minion"),
    ("DINO_419", "friendly-beast"),
];

/// A board with minions on both sides — including a friendly Beast, so the
/// race-scoped cards have a candidate too.
///
/// Deliberately **no Divine Shield minions**: the engine currently routes
/// "destroy" through lethal damage, so a shield eats a destroy (ledger entry
/// — Assassinate on an Argent Squire leaves it alive). That would mask the
/// signal these tests are after, which is *which* entity the effect hit.
fn mixed_board(card: &'static orange_stone::cards::def::CardDef) -> (GameState, Entity) {
    use orange_stone::cards::def::{BLOODFEN_RAPTOR, CHILLWIND_YETI};
    let mut builder = GameBuilder::new();
    builder.add_minion_to_board(P1, &BLOODFEN_RAPTOR); // friendly Beast
    builder.add_minion_to_board(P1, &CHILLWIND_YETI);
    builder.add_minion_to_board(P2, &BLOODFEN_RAPTOR);
    builder.add_minion_to_board(P2, &CHILLWIND_YETI);
    builder.add_minion_to_hand(P1, card);
    builder.set_mana(P1, 10, 10);
    let state = builder.build();
    let hand = card_in_hand(&state);
    (state, hand)
}

/// The sweep: every W2 card offers at least one target, and each offered
/// target is a legal play (i.e. the enumeration and `rules::validate` agree).
#[test]
fn every_w2_card_offers_targets() {
    use orange_stone::cards::card_by_id;
    let mut missing = Vec::new();
    for (id, _domain) in W2_CARDS {
        let def = card_by_id(id).unwrap_or_else(|| panic!("{id} is in ALL_CARDS"));
        let (state, card) = mixed_board(def);
        let targets = offered_targets(&state, card);
        if targets.is_empty() {
            missing.push(*id);
        }
    }
    assert!(
        missing.is_empty(),
        "these cards still play untargeted: {missing:?}"
    );
}

/// Conflagrate — "Deal 5 damage to a minion." (`any-minion` domain: the
/// chosen minion takes it, on either side.)
#[test]
fn conflagrate_hits_the_chosen_minion() {
    use orange_stone::cards::card_by_id;
    let def = card_by_id("FIR_954").unwrap();
    let (mut state, card) = mixed_board(def);
    let enemy: Vec<Entity> = state
        .world()
        .zones()
        .iter(Zone::Play, P2)
        .filter(|&e| state.world().card_type(e) == Some(CardType::Minion))
        .collect();
    let chosen = enemy[1];

    assert!(offered_targets(&state, card).contains(&chosen));
    play_at(&mut state, card, chosen);
    assert!(
        state.world().damage(chosen).is_some_and(|d| d.0 > 0)
            || !state.world().zones().iter(Zone::Play, P2).any(|e| e == chosen),
        "the chosen minion took the damage (or died from it)"
    );
    assert_eq!(damage_of(&state, enemy[0]), 0, "the other enemy is untouched");
}

/// Siphoning Growth — "Destroy a friendly minion to gain 8 Armor."
/// (`friendly` domain: destroying the *chosen* one matters a lot here.)
#[test]
fn siphoning_growth_destroys_the_chosen_friendly_minion() {
    use orange_stone::cards::card_by_id;
    let def = card_by_id("EDR_531").unwrap();
    let (mut state, card) = mixed_board(def);
    let mine = friendly_minions(&state);
    let (keep, sacrifice) = (mine[0], mine[1]);

    let targets = offered_targets(&state, card);
    assert!(targets.contains(&keep) && targets.contains(&sacrifice));
    assert!(
        !targets.iter().any(|t| state.world().player(*t) == Some(P2)),
        "enemy minions are not candidates"
    );

    play_at(&mut state, card, sacrifice);
    let alive: Vec<Entity> = state.world().zones().iter(Zone::Play, P1).collect();
    assert!(!alive.contains(&sacrifice), "the chosen minion is sacrificed");
    assert!(alive.contains(&keep), "the other one stays");
}

/// Shadowflame Suffusion — the card that started the audit. "Deal 2 damage."
/// over the engine's enemy domain: the chosen enemy takes it.
#[test]
fn shadowflame_suffusion_hits_the_chosen_enemy() {
    use orange_stone::cards::card_by_id;
    let def = card_by_id("FIR_939").unwrap();
    let (mut state, card) = mixed_board(def);
    let enemy_hero = state.player(P2).hero;

    let targets = offered_targets(&state, card);
    assert!(
        targets.contains(&enemy_hero),
        "the enemy hero is a candidate: {targets:?}"
    );
    assert!(
        !targets.iter().any(|t| state.world().player(*t) == Some(P1)),
        "friendly characters are not candidates (engine domain is enemy-only)"
    );

    play_at(&mut state, card, enemy_hero);
    assert_eq!(damage_of(&state, enemy_hero), 2, "the chosen hero takes 2");
}

/// Herbivore Assistant — "Give a friendly Beast +2/+2 and Rush."
/// (race-scoped: only the friendly Beast is offered.)
#[test]
fn herbivore_assistant_only_offers_friendly_beasts() {
    use orange_stone::cards::card_by_id;
    let def = card_by_id("DINO_419").unwrap();
    let (mut state, card) = mixed_board(def);
    let mine = friendly_minions(&state);
    let beast = mine[0]; // Bloodfen Raptor

    let targets = offered_targets(&state, card);
    assert_eq!(targets, vec![beast], "only the friendly Beast is a candidate");

    let before = attack_of(&state, beast);
    play_at(&mut state, card, beast);
    assert!(attack_of(&state, beast) > before, "the chosen Beast is buffed");
}

