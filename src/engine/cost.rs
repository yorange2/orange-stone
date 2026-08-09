//! Cost manager — the single composition point for card play costs (roadmap G5).
//!
//! The modifier stack lives in `World::effective_cost`: base + enchantment
//! deltas, then set-to-value / floor modifiers, then hand-only aura
//! reductions, floored at 0. This module adds the player-level modifiers that
//! the World cannot see (Kirin Tor Mage's one-time free secret) and is the
//! ONLY place a play cost is composed — validation, mana deduction, and bots
//! all read from here.

use crate::core::component::Cost;
use crate::core::entity::Entity;
use crate::core::player::PlayerId;
use crate::core::state::GameState;

/// The cost of playing `card` for `player`: the entity cost stack plus
/// player-level modifiers (e.g. Kirin Tor Mage's one-time free secret).
#[must_use]
pub fn play_cost(state: &GameState, card: Entity, player: PlayerId) -> Cost {
    let mut cost = state.world().effective_cost(card).unwrap_or_default();
    // Kirin Tor Mage: the next secret costs 0 (one-time, consumed on play)
    let is_secret = state
        .world()
        .card_id(card)
        .and_then(|cid| crate::cards::def::card_by_id(cid.0))
        .is_some_and(|def| def.secret.is_some());
    if is_secret && state.player(player).next_secret_free {
        cost = Cost(0);
    }
    // Millhouse Manastorm: the opponent's spells cost 0 this turn
    if state.world().card_type(card) == Some(crate::core::component::CardType::Spell)
        && state.player(player).spells_cost_zero
    {
        cost = Cost(0);
    }
    // TIME_716 Slow Motion (M3-W2a): "The opponent's cards cost (1) more
    // next turn" — the tax rides the OPPONENT's player field (the caster
    // set it during their own turn; it applies to the caster's enemy
    // during the enemy's next turn, cleared at the caster's next turn
    // start).
    let tax = state
        .player(player.opponent())
        .next_turn_enemy_cards_cost_more;
    if tax > 0 {
        cost = Cost(cost.0 + tax);
    }
    // Preparation (W11): the next spell cast this turn costs `amount` less
    // (one-time — the flag is consumed by the first spell played)
    if state.world().card_type(card) == Some(crate::core::component::CardType::Spell) {
        let discount = state.player(player).next_spell_discount;
        if discount > 0 {
            cost = Cost((cost.0 - discount).max(0));
        }
    }
    // Raging Felscreamer (Core Set W4a): the next Demon costs less
    // (one-time, consumed on play — cleared after use)
    if state
        .world()
        .card_id(card)
        .and_then(|cid| crate::cards::def::card_by_id(cid.0))
        .is_some_and(|def| def.race == Some(crate::core::component::Race::Demon))
        && state.player(player).next_demon_discount > 0
    {
        cost = Cost((cost.0 - state.player(player).next_demon_discount).max(0));
    }
    // Foxy Fraud (Core Set W4a): the next Combo card costs less this turn
    if state
        .world()
        .card_id(card)
        .and_then(|cid| crate::cards::def::card_by_id(cid.0))
        .is_some_and(|def| def.combo_effect.is_some())
        && state.player(player).next_combo_discount > 0
    {
        cost = Cost((cost.0 - state.player(player).next_combo_discount).max(0));
    }
    // Illidari Studies (Core Set W6): the next Outcast card costs less
    // (one-time, consumed on play — cleared after use)
    if state
        .world()
        .card_id(card)
        .and_then(|cid| crate::cards::def::card_by_id(cid.0))
        .is_some_and(crate::cards::def::has_outcast)
        && state.player(player).next_outcast_discount > 0
    {
        cost = Cost((cost.0 - state.player(player).next_outcast_discount).max(0));
    }
    // Cult Neophyte (Core Set W4b): the opponent's spells cost more
    if state.world().card_type(card) == Some(crate::core::component::CardType::Spell) {
        let more = state.player(player).enemy_spell_cost_more;
        if more > 0 {
            cost = Cost(cost.0 + more);
        }
    }
    // Dread Corsair (Core Set W3b): costs (1) less per Attack of the
    // owner's weapon
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "CORE_NEW1_022")
    {
        let weapon_atk = state
            .player(player)
            .weapon
            .and_then(|w| state.world().effective_attack(w))
            .map_or(0, |a| a.0);
        cost = Cost((cost.0 - weapon_atk).max(0));
    }
    // Glowroot Lure (2025–2026 expansions M1-W4a): costs (1) less for each
    // time the owner used their Hero Power this game
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "EDR_477")
    {
        let uses = state.player(player).hero_power_uses;
        cost = Cost((cost.0 - uses as i32).max(0));
    }
    // Everburning Phoenix (2025–2026 expansions M1-W5): costs (1) less for
    // each card the owner played this turn. The counter is bumped at the
    // CardPlayed path AFTER this discount computes, so the current card is
    // excluded — matching the official discount.
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "FIR_919")
    {
        let played = state.player(player).cards_played_this_turn;
        cost = Cost((cost.0 - played as i32).max(0));
    }
    // Kindred (M2-W3): TLC_366/600/816 cost less while the activation
    // condition holds. Computed BEFORE the CardPlayed path pushes the
    // card's own type, so the check counts earlier same-type cards only
    // (>= 1 — the current card never discounts itself).
    cost =
        Cost((cost.0 - crate::cards::kindred::kindred_cost_discount(state, card, player)).max(0));
    // Hot Spring Glider (M2-W3): "your next Murloc costs (1) less"
    // (one-time, consumed by the next Murloc play — cleared on play at the
    // CardPlayed path)
    if state
        .world()
        .card_id(card)
        .and_then(|cid| crate::cards::def::card_by_id(cid.0))
        .is_some_and(|def| def.race == Some(crate::core::component::Race::Murloc))
        && state.player(player).next_murloc_discount > 0
    {
        cost = Cost((cost.0 - state.player(player).next_murloc_discount).max(0));
    }
    // M3-W2a — Across the Timeways cost-pipeline entries (the
    // id-keyed discount pattern, like Everburning Phoenix):
    // - TIME_022 Perennial Serpent: costs (4) less if ANY minion on either
    //   board is Dormant.
    // - TIME_047 Devious Coyote: costs (1) less for each time the enemy
    //   hero actually lost Health this turn (the counter is bumped in the
    //   damage pipeline; cleared at turn start).
    // - TIME_715 For Glory!: costs (1) less for each minion the opponent
    //   controls.
    // - TIME_102 Circadiamancer's marked hand card: costs (1) less per own
    //   turn start (TurnCostReducer counter).
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "TIME_022")
    {
        let any_dormant = [
            crate::core::player::PlayerId::Player1,
            crate::core::player::PlayerId::Player2,
        ]
        .into_iter()
        .any(|pid| {
            state
                .world()
                .zones()
                .iter(crate::core::zone::Zone::Play, pid)
                .any(|e| state.world().dormant(e).is_some())
        });
        if any_dormant {
            cost = Cost((cost.0 - 4).max(0));
        }
    }
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "TIME_047")
    {
        let dmg = state.player(player).enemy_hero_damaged_this_turn as i32;
        cost = Cost((cost.0 - dmg).max(0));
    }
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "TIME_715")
    {
        let enemy = player.opponent();
        let minions = state
            .world()
            .zones()
            .iter(crate::core::zone::Zone::Play, enemy)
            .filter(|&e| {
                state.world().card_type(e) == Some(crate::core::component::CardType::Minion)
            })
            .count() as i32;
        cost = Cost((cost.0 - minions).max(0));
    }
    if let Some(reducer) = state.world().turn_cost_reducer(card) {
        cost = Cost((cost.0 - reducer.0 as i32).max(0));
    }
    // Sea Giant (W11): costs (1) less for each minion on the battlefield
    // (both sides — the board-count rule composes here like Dread Corsair)
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "NEUTRAL_026")
    {
        let board_count = [player, player.opponent()]
            .iter()
            .map(|&p| {
                state
                    .world()
                    .zones()
                    .iter(crate::core::zone::Zone::Play, p)
                    .filter(|&e| {
                        state.world().card_type(e) == Some(crate::core::component::CardType::Minion)
                    })
                    .count()
            })
            .sum::<usize>();
        cost = Cost((cost.0 - board_count as i32).max(0));
    }
    // Dread Corsair: costs (1) less per Attack of your weapon
    let weapon_attack = state
        .player(player)
        .weapon
        .and_then(|w| state.world().attack(w))
        .map_or(0, |a| a.0);
    if weapon_attack > 0
        && state
            .world()
            .card_id(card)
            .is_some_and(|c| c.0 == "NEUTRAL_C13")
    {
        cost = Cost((cost.0 - weapon_attack).max(0));
    }
    // Pint-Sized Summoner: while its FirstMinionDiscount aura is on the board
    // (silencing the summoner removes the aura), the FIRST minion played this
    // turn costs `amount` less.
    if state.world().card_type(card) == Some(crate::core::component::CardType::Minion)
        && state.player(player).minions_played_this_turn == 0
    {
        let discount: i32 = state
            .world()
            .zones()
            .iter(crate::core::zone::Zone::Play, player)
            .filter_map(|e| state.world().aura(e))
            .filter_map(|a| match a.effect {
                crate::core::component::AuraEffect::FirstMinionDiscount { amount } => Some(amount),
                _ => None,
            })
            .sum();
        cost = Cost((cost.0 - discount).max(0));
    }
    // Naralex, Herald of the Flights (2025–2026 expansions M1-W4b): while he
    // is on the board, the first Dragon the owner plays each turn costs (1)
    // — the per-turn play counter is Player::dragons_played_this_turn
    // (reset at the owner's turn start; incremented at the CardPlayed path).
    if state
        .world()
        .has_race(card, crate::core::component::Race::Dragon)
        && state.player(player).dragons_played_this_turn == 0
        && state
            .world()
            .zones()
            .iter(crate::core::zone::Zone::Play, player)
            .any(|e| state.world().card_id(e).is_some_and(|c| c.0 == "EDR_844"))
    {
        cost = Cost(1);
    }
    // Agamaggan (2025–2026 expansions M1-W4b): the next card the owner plays
    // costs (0) — registered simplification (§14.4, the official effect sets
    // the cost to the opponent's Health). Applied after the Naralex set, so
    // Agamaggan's flag wins.
    if state.player(player).next_card_costs_zero {
        cost = Cost(0);
    }
    // Aviana, Elune's Chosen (2025–2026 expansions M1-W4b): all the owner's
    // cards cost (1) this game (registered simplification §14.4 — the real
    // lunar-cycle timing is approximated as immediate). "(1)" is a SET — the
    // card costs exactly 1 (official text: "Your cards cost (1)"), so a
    // 9-Cost minion is discounted to 1 and a (0) card is raised to 1.
    // Applied LAST so the one-time/set-to-value effects above keep their
    // lower costs only when they land at or below 1.
    if state.player(player).cards_cost_1 {
        cost = Cost(1);
    }
    // M2-W4a arms (Un'Goro main-set wave — all set/discount semantics read
    // the same composed cost; applied before Aviana so her set-to-1 wins).
    // Storage Scuffle: "Costs (0) if you've Discovered this turn".
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "TLC_365")
        && state.player(player).discovered_this_turn
    {
        cost = Cost(0);
    }
    // Gladesong Siren: "Costs (1) if you've cast a Holy and Shadow spell
    // this turn" (SET — the card costs exactly 1).
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "TLC_819")
        && state.player(player).shadow_cast_this_turn
        && !state.player(player).holy_cast_ids.is_empty()
    {
        cost = Cost(1);
    }
    // Underbrush Tracker: "Costs (1) less for each time you've shuffled
    // cards into your deck" (the per-shuffle counter — bumped at every
    // shuffle-into-deck resolution).
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "TLC_520")
    {
        let shuffles = state.player(player).shuffled_count as i32;
        cost = Cost((cost.0 - shuffles).max(0));
    }
    // Spelunker (M2-W4a): "Your next Temporary card costs (2) less" — the
    // one-time flag is consumed by the next play of a card carrying the
    // Temporary marker (cleared on play at the CardPlayed path).
    if state.world().temporary(card).is_some() {
        let discount = state.player(player).next_temporary_discount;
        if discount > 0 {
            cost = Cost((cost.0 - discount).max(0));
        }
    }
    // Cower in Fear (M2-W4a): "The next Beast you play this turn costs
    // (2) less" — the one-time this-turn flag.
    if state
        .world()
        .card_id(card)
        .and_then(|cid| crate::cards::def::card_by_id(cid.0))
        .is_some_and(|def| def.race == Some(crate::core::component::Race::Beast))
        && state.player(player).next_beast_discount > 0
    {
        cost = Cost((cost.0 - state.player(player).next_beast_discount).max(0));
    }
    // Wave of Tar (M2-W4a): "Enemy minions cost (2) more next turn" — the
    // flag sits on the caster's player record (cleared at their next turn
    // start); the enemy's minions pay the tax while it is set.
    if state.world().card_type(card) == Some(crate::core::component::CardType::Minion)
        && state.player(player.opponent()).minions_cost_more
    {
        cost = Cost(cost.0 + 2);
    }
    // Loh, the Living Legend (M2-W4b): "Your minions cost (5) this game" —
    // a SET: the card costs exactly 5 regardless of its printed cost
    // (a 2-Cost minion is raised to 5, a 9-Cost minion discounted to 5).
    // Applied BEFORE the Reanimated Pterrordax arm so Pterrordax's own
    // set-to-0 wins over the flag; Aviana's set-to-1 stays last and wins
    // over both.
    if state.world().card_type(card) == Some(crate::core::component::CardType::Minion)
        && state.player(player).minions_cost_5
    {
        cost = Cost(5);
    }
    // Reanimated Pterrordax (M2-W4a): "Costs Corpses instead of Mana" —
    // the 5 Corpses are spent at the CardPlayed path; the mana cost is 0.
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "TLC_436")
    {
        cost = Cost(0);
    }
    // M3-W2b — Across the Timeways legendary wave
    // (src/cards/exp_tmw_w2b.rs).
    // Azure Queen Sindragosa (TIME_852): "If you control another Dragon,
    // your Arcane spells cost (2) less" — a board-presence aura read like
    // Naralex: the discount applies while the owner controls a Dragon on
    // the battlefield that is not TIME_852 herself ("another" — the second
    // copy of herself does not count, §21).
    if state.world().card_type(card) == Some(crate::core::component::CardType::Spell)
        && state
            .world()
            .card_id(card)
            .and_then(|c| crate::cards::quest::spell_school(c.0))
            .is_some_and(|s| s == crate::cards::quest::SpellSchool::Arcane)
        && state
            .world()
            .zones()
            .iter(crate::core::zone::Zone::Play, player)
            .any(|e| {
                state.world().card_id(e).is_some_and(|c| {
                    c.0 != "TIME_852"
                        && crate::cards::def::card_by_id(c.0)
                            .is_some_and(|d| d.race == Some(crate::core::component::Race::Dragon))
                })
            })
    {
        cost = Cost((cost.0 - 2).max(0));
    }
    // Medivh the Hallowed (TIME_890): "Costs (0) if you control Karazhan" —
    // the Karazhan check reads the owner's Location slot (TIME_890t2).
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "TIME_890")
        && state.player(player).location.is_some_and(|l| {
            state
                .world()
                .card_id(l)
                .is_some_and(|c| c.0 == "TIME_890t2")
        })
    {
        cost = Cost(0);
    }
    // The Eternal Hold (TIME_446) fallback: "If your deck has no minions,
    // your next Demon costs (1)" — the one-time flag is consumed by the
    // next Demon play at the CardPlayed path.
    if state
        .world()
        .card_id(card)
        .and_then(|cid| crate::cards::def::card_by_id(cid.0))
        .is_some_and(|def| def.race == Some(crate::core::component::Race::Demon))
        && state.player(player).next_demon_cost_one
    {
        cost = Cost(1);
    }
    // M3-W3 — The End of Time miniset cost arms
    // (src/cards/exp_tmw_w3.rs, §22).
    // Remnant of Rage (END_004): "Costs (1) less for each minion that
    // died this turn" — the per-turn death lists (pushed at MinionDied,
    // cleared at the owner's turn end). The owner's list holds the
    // friendly deaths; the opponent's list holds the enemy deaths that
    // happened on the same turn, so both are summed (§22).
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "END_004")
    {
        let died = state.player(player).died_this_turn.len() as i32
            + state.player(player.opponent()).died_this_turn.len() as i32;
        cost = Cost((cost.0 - died).max(0));
    }
    // Haywire Hornswog (END_030): "Costs (1) less for each Mana Crystal
    // you've Overloaded this game" — the game-long overload counter
    // (bumped at the CardPlayed overload site and by Winged Aberration's
    // END_032 combo arm).
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "END_030")
    {
        cost = Cost((cost.0 - state.player(player).overload_total).max(0));
    }
    // Prescient Slitherdrake (END_033): "Costs (3) less if you're holding
    // another Dragon" — a HAND card of the Dragon race other than itself
    // (the holding-dragon check, TIME_852's filter shape).
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "END_033")
        && state
            .world()
            .zones()
            .iter(crate::core::zone::Zone::Hand, player)
            .any(|e| {
                e != card
                    && state
                        .world()
                        .has_race(e, crate::core::component::Race::Dragon)
            })
    {
        cost = Cost((cost.0 - 3).max(0));
    }
    // M4-W4 — the Cataclysm closing wave cost arms
    // (src/cards/exp_cata_w4.rs, §26).
    // Deathwing, Worldbreaker (CATA_190h): "Costs (1) less for each
    // Ultraxion Herald" — the flag accumulates Ultraxion's herald-count
    // reductions (cards/herald.rs keyed CATA_497).
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "CATA_190h")
    {
        cost = Cost((cost.0 - state.player(player).deathwing_cost_reduction).max(0));
    }
    // Medivh's Triumph (CATA_308): "Costs (1) if you control a Legendary
    // card" — the engine's legendary pool (LEGENDARY_CLASSIC, the active
    // window convention) on the friendly board.
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "CATA_308")
        && state
            .world()
            .zones()
            .iter(crate::core::zone::Zone::Play, player)
            .any(|e| {
                state.world().card_id(e).is_some_and(|c| {
                    crate::cards::sets::LEGENDARY_CLASSIC
                        .iter()
                        .any(|d| d.id == c.0)
                })
            })
    {
        cost = Cost(1);
    }
    // Spellweaver's Brilliance (CATA_452): "Costs (1) less for each damage
    // you dealt with spells this turn" — the per-turn spell-damage counter
    // (bumped by the damage pipeline when a friendly spell deals damage).
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "CATA_452")
    {
        cost = Cost((cost.0 - state.player(player).spell_damage_dealt_this_turn).max(0));
    }
    // Ravenous Felfisher (CATA_529): "Costs (1) less for each Fel spell
    // you've cast this game" — the game-long Fel cast counter (bumped at
    // the spell-cast school site).
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "CATA_529")
    {
        cost = Cost((cost.0 - state.player(player).fel_spells_cast_this_game as i32).max(0));
    }
    // Muradin's Last Stand (CATA_568): "Costs (1) less for each time a
    // friendly character attacked this game" — the game-long friendly
    // attack counter (bumped by the attack resolution).
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "CATA_568")
    {
        cost = Cost((cost.0 - state.player(player).friendly_attacks_this_game as i32).max(0));
    }
    // Gronn Giant (CATA_616): "This minion's Cost is reduced by the Cost
    // of the last card you played" — recorded at the CardPlayed path.
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "CATA_616")
    {
        cost = Cost((cost.0 - state.player(player).last_played_card_cost).max(0));
    }
    // M5-W1 — Mug's Magic (JAIL_800hp1, the Mug'Zee hero power): "Your
    // first minion each turn costs (2) less." The flag is set at the
    // StartOfGame hero-power swap and never expires (the replacement is
    // permanent, like any hero-power swap); the per-turn gate reuses the
    // `minions_played_this_turn` counter (reset at the owner's turn
    // start, incremented at the CardPlayed path) — the same shape as
    // Pint-Sized Summoner's FirstMinionDiscount above.
    if state.world().card_type(card) == Some(crate::core::component::CardType::Minion)
        && state.player(player).mugzee_mug_magic
        && state.player(player).minions_played_this_turn == 0
    {
        cost = Cost((cost.0 - 2).max(0));
    }
    // M5-W2 — the closing Violet Hold wave, id-keyed overrides:
    // - JAIL_204 Tricksy Rouge: "Costs (2) if you control no minions."
    // - JAIL_433 Unshackle Soul: "Costs (1) if you played a copy of an
    //   opponent's card while holding this" (the game-scoped flag, §28).
    // - JAIL_503 Bribe: "Costs (1) less for each Coin in your hand."
    // - JAIL_514 Nifty Lockpick: "Costs (1) less for each card in your
    //   hand."
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "JAIL_204")
    {
        let has_minion = state
            .world()
            .zones()
            .iter(crate::core::zone::Zone::Play, player)
            .any(|e| state.world().card_type(e) == Some(crate::core::component::CardType::Minion));
        if !has_minion {
            cost = Cost(2);
        }
    }
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "JAIL_433")
        && state.player(player).copied_from_opponent_played
    {
        cost = Cost((cost.0 - 1).max(0));
    }
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "JAIL_503")
    {
        let coins = state
            .world()
            .zones()
            .iter(crate::core::zone::Zone::Hand, player)
            .filter(|&e| state.world().card_id(e).is_some_and(|c| c.0 == "GAME_005"))
            .count() as i32;
        cost = Cost((cost.0 - coins).max(0));
    }
    if state
        .world()
        .card_id(card)
        .is_some_and(|c| c.0 == "JAIL_514")
    {
        let hand_size = state
            .world()
            .zones()
            .len(crate::core::zone::Zone::Hand, player) as i32;
        cost = Cost((cost.0 - hand_size).max(0));
    }
    cost
}
