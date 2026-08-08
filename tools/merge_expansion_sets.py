#!/usr/bin/env python3
"""Merge the 2025-2026 expansion sets into cards/cards.json (roadmap M0.3).

Appends the five expansions from cards/data/<SET>.json (the M0.1 raw per-set
dumps) to the generator's single data source, slimmed to the extended schema
(id/name/cost/type/attack/health/durability/mechanics + text/race/cardClass/
classes/set/collectible — the fields build.rs and the effect waves consume).

Set order follows release order; within a set, cards are sorted by ID. The
merge refuses to run on ID collisions (existing or intra-set). Idempotent:
already-merged IDs are skipped, so re-running after a data refresh updates
fields but never duplicates entries.

Usage:
    python3 tools/merge_expansion_sets.py
"""

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CARDS_JSON = REPO_ROOT / "cards" / "cards.json"
DATA_DIR = REPO_ROOT / "cards" / "data"

# (set code, json file) in release order (2025-2026-expansions-roadmap).
EXPANSION_ORDER = [
    ("EMERALD_DREAM", "EMERALD_DREAM.json"),
    ("THE_LOST_CITY", "THE_LOST_CITY.json"),
    ("TIME_TRAVEL", "TIME_TRAVEL.json"),
    ("CATACLYSM", "CATACLYSM.json"),
    ("ESCAPEFROM_VIOLET_HOLD", "ESCAPEFROM_VIOLET_HOLD.json"),
]

# Fields kept per card, in the same order the existing entries use.
BASE_FIELDS = ("id", "name", "cost", "attack", "health", "durability", "type", "mechanics")
EXTRA_FIELDS = ("text", "race", "cardClass", "classes", "set", "collectible")


def slim(card: dict) -> dict:
    entry = {}
    for field in BASE_FIELDS:
        if field in card:
            entry[field] = card[field]
    for field in EXTRA_FIELDS:
        if field in card:
            entry[field] = card[field]
    return entry


def main() -> int:
    entries = json.loads(CARDS_JSON.read_text())
    existing = {e["id"] for e in entries}

    added = 0
    for set_code, filename in EXPANSION_ORDER:
        cards = json.loads((DATA_DIR / filename).read_text())
        cards.sort(key=lambda c: c["id"])
        for card in cards:
            if card["id"] in existing:
                continue  # idempotent re-run — keep existing entry
            entry = slim(card)
            entry.setdefault("set", set_code)
            if entry["id"] in existing:
                raise SystemExit(f"duplicate ID after merge: {entry['id']}")
            entries.append(entry)
            existing.add(entry["id"])
            added += 1

    CARDS_JSON.write_text(
        json.dumps(entries, ensure_ascii=False, separators=(",", ":")) + "\n"
    )
    print(f"merged {added} expansion cards -> {CARDS_JSON.relative_to(REPO_ROOT)} "
          f"(total {len(entries)})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
