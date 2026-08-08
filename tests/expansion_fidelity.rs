//! Per-set wiring fidelity (expansion sub-roadmap W0): every card in the raw
//! dump resolves through the generated registry with a matching name, and
//! every generated card with the set's ID prefixes exists in the dump —
//! bidirectional coverage in both directions.

use orange_stone::cards::generated;

fn check_dump_fidelity(dump_file: &str, prefixes: &[&str], expected: usize) {
    let dump: serde_json::Value =
        serde_json::from_str(dump_file).expect("parse per-set dump (JSON)");
    let cards = dump.as_array().expect("dump is an array");
    assert_eq!(cards.len(), expected, "dump card count");
    let dump_ids: std::collections::HashSet<&str> = cards
        .iter()
        .map(|c| c["id"].as_str().expect("dump id"))
        .collect();
    for entry in cards {
        let id = entry["id"].as_str().expect("id");
        let name = entry["name"].as_str().expect("name");
        let card = generated::find_by_id(id)
            .unwrap_or_else(|| panic!("dump card {id} missing from generated registry"));
        assert_eq!(card.name, name, "name mismatch for {id}");
    }
    let generated_count = generated::GENERATED_IDS
        .iter()
        .filter(|id| prefixes.iter().any(|p| id.starts_with(p)))
        .count();
    assert_eq!(generated_count, expected, "generated {prefixes:?} count");
    for id in generated::GENERATED_IDS {
        if prefixes.iter().any(|p| id.starts_with(p)) {
            assert!(
                dump_ids.contains(*id),
                "generated {id} missing from the dump"
            );
        }
    }
}

/// Emerald Dream sub-roadmap W0 (M1-W0): EDR_ + FIR_ (miniset) prefixes.
#[test]
fn emerald_dream_dump_fidelity() {
    check_dump_fidelity(
        include_str!("../cards/data/EMERALD_DREAM.json"),
        &["EDR_", "FIR_"],
        183,
    );
}

/// The Lost City of Un'Goro sub-roadmap W0 (M2-W0): TLC_ + DINO_ (miniset)
/// prefixes.
#[test]
fn the_lost_city_dump_fidelity() {
    check_dump_fidelity(
        include_str!("../cards/data/THE_LOST_CITY.json"),
        &["TLC_", "DINO_"],
        183,
    );
}

/// Across the Timeways sub-roadmap W0 (M3-W0): TIME_ + END_ (miniset)
/// prefixes.
#[test]
fn across_the_timeways_dump_fidelity() {
    check_dump_fidelity(
        include_str!("../cards/data/TIME_TRAVEL.json"),
        &["TIME_", "END_"],
        183,
    );
}
