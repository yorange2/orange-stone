#!/usr/bin/env python3
"""Fetch the 2025-2026 expansion sets from hearthstonejson and slice them into
per-set data files under cards/data/ (roadmap M0.1, decision D1).

Source: https://api.hearthstonejson.com/v1/latest/enUS/cards.collectible.json
(community-maintained full card database, same lineage as the SabberStone
ecosystem — our fidelity baseline).

Usage:
    python3 tools/fetch_expansion_sets.py              # download from network
    python3 tools/fetch_expansion_sets.py --input F    # slice an existing dump

Output: cards/data/<SET_CODE>.json — raw per-set slices (full hearthstonejson
fields, sorted by ID) + cards/data/SOURCE.md with the source URL, fetch date
and dump sha256 for reproducibility.
"""

import argparse
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

# The five expansions of the 2025-2026 roadmap (set code -> display name).
# Miniset cards are included in the same `set` slice and marked isMiniSet=true.
EXPANSION_SETS = {
    "EMERALD_DREAM": "Into the Emerald Dream + Embers of the World Tree (mini)",
    "THE_LOST_CITY": "The Lost City of Un'Goro + Festival of the Devilsaur (mini)",
    "TIME_TRAVEL": "Across the Timeways + The End of Time (mini)",
    "CATACLYSM": "Cataclysm (+ Class Sets)",
    "ESCAPEFROM_VIOLET_HOLD": "Escape from Violet Hold",
}

SOURCE_URL = "https://api.hearthstonejson.com/v1/latest/enUS/cards.collectible.json"

REPO_ROOT = Path(__file__).resolve().parent.parent
DATA_DIR = REPO_ROOT / "cards" / "data"


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
    print(f"dump: {len(dump)} cards, sha256={hashlib.sha256(raw).hexdigest()}")

    DATA_DIR.mkdir(parents=True, exist_ok=True)
    ok = True
    for set_code, display in EXPANSION_SETS.items():
        cards = sorted(
            (c for c in dump if c.get("set") == set_code),
            key=lambda c: c["id"],
        )
        ids = [c["id"] for c in cards]
        dupes = len(ids) - len(set(ids))
        missing = []
        for c in cards:
            for field in ("id", "name", "cost", "type", "collectible"):
                if field not in c:
                    missing.append((c.get("id"), field))
        mini = sum(1 for c in cards if c.get("isMiniSet"))
        print(
            f"{set_code}: {len(cards)} cards ({mini} miniset), "
            f"{'DUPLICATE IDS: %d' % dupes if dupes else 'ids unique'}, "
            f"{'MISSING FIELDS: %s' % missing if missing else 'required fields ok'}"
        )
        if dupes or missing:
            ok = False
            continue
        out = DATA_DIR / f"{set_code}.json"
        out.write_text(json.dumps(cards, ensure_ascii=False, indent=1) + "\n")
        print(f"  wrote {out.relative_to(REPO_ROOT)}")

    source_md = DATA_DIR / "SOURCE.md"
    source_md.write_text(
        f"# Expansion set data (M0.1, decision D1: hearthstonejson dump)\n\n"
        f"- Source URL: `{SOURCE_URL}`\n"
        f"- Fetched: {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')}\n"
        f"- Dump sha256: `{hashlib.sha256(raw).hexdigest()}`\n"
        f"- Dump size: {len(dump)} cards\n\n"
        f"Re-fetch:\n\n```bash\npython3 tools/fetch_expansion_sets.py\n```\n"
        f"or re-slice an existing dump: `python3 tools/fetch_expansion_sets.py --input /path/cards.json`\n"
    )
    print(f"wrote {source_md.relative_to(REPO_ROOT)}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
