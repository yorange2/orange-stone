#!/usr/bin/env python3
"""Backfill the extended schema fields into cards/cards.json (roadmap M0.2).

Merges per-card metadata from the hearthstonejson collectible dump into the
existing entries: text (effect prose), race, cardClass, classes, set and
collectible — the fields the 2025-2026 expansions need for hand-wiring
effects and per-set registration. Entries without a dump counterpart (tokens,
enchantments, puzzle/prologue/battlegrounds cards, hero powers) are kept as-is.

Usage:
    python3 tools/backfill_card_fields.py              # download from network
    python3 tools/backfill_card_fields.py --input F    # use an existing dump

Idempotent: re-running rewrites the same merged entries.
"""

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CARDS_JSON = REPO_ROOT / "cards" / "cards.json"
SOURCE_URL = "https://api.hearthstonejson.com/v1/latest/enUS/cards.collectible.json"

# New optional fields appended to existing entries (in this order).
EXTRA_FIELDS = ("text", "race", "cardClass", "classes", "set", "collectible")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", help="path to a local cards.collectible.json dump (skip download)")
    args = parser.parse_args()

    if args.input:
        raw = Path(args.input).read_bytes()
    else:
        import urllib.request

        print(f"downloading {SOURCE_URL} ...")
        with urllib.request.urlopen(SOURCE_URL) as resp:
            raw = resp.read()
    dump = json.loads(raw)
    by_id = {c["id"]: c for c in dump}
    print(f"dump: {len(dump)} cards")

    entries = json.loads(CARDS_JSON.read_text())
    merged = 0
    for entry in entries:
        src = by_id.get(entry["id"])
        if src is None:
            continue
        for field in EXTRA_FIELDS:
            if field in src:
                entry[field] = src[field]
        merged += 1

    CARDS_JSON.write_text(
        json.dumps(entries, ensure_ascii=False, separators=(",", ":")) + "\n"
    )
    print(f"backfilled {merged}/{len(entries)} entries -> {CARDS_JSON.relative_to(REPO_ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
