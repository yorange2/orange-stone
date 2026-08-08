#!/usr/bin/env python3
"""Backfill the per-wave card inventories into the expansion sub-roadmaps
(roadmap M0.5).

Reads cards/data/<SET>.json and classifies each card into its sub-roadmap
wave by text/mechanic signature (Imbue / Dark Gift / Choose One / Quest /
Kindred / Rewind / Fabled / Colossal / Herald / Shatter / Deathwing /
elite-legendary), then replaces every `- [ ] （数据回填）` / `- [ ] (backfill
from data)` placeholder in the matching zh+en sub-roadmap with the real
`- [ ] <ID> <Name>` list, sorted by ID.

Notes:
- miniset cards (isMiniSet) always land in the closing wave; the Cataclysm
  Class Sets (29) land in the closing wave as a follow-up group.
- Cards with overlapping signatures (e.g. Lady Azshara = Fabled + Choose One,
  CATA_190h = Deathwing hero + Herald text) land in the wave of their primary
  mechanic; the secondary mechanic is noted inline.
- "Smoldering" is absent from the dump (see M0.5 decision) — the EDR miniset
  wave is listed without a mechanic-level signature.

Usage:
    python3 tools/backfill_roadmap_checklists.py
"""

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DATA_DIR = REPO_ROOT / "cards" / "data"
DOCS_DIR = REPO_ROOT / "docs"

PLACEHOLDER_ZH = "- [ ] （数据回填）"
PLACEHOLDER_EN = "- [ ] (backfill from data)"


def text_of(card) -> str:
    return (card.get("text") or "").lower()


def has(card, *needles) -> bool:
    t = text_of(card)
    return any(n in t for n in needles)


def categorize(card) -> set[str]:
    m = card.get("mechanics") or []
    out = set()
    if has(card, "imbue"):
        out.add("imbue")
    if has(card, "dark gift"):
        out.add("dark_gift")
    if has(card, "choose one") or "CHOOSE_ONE" in m:
        out.add("choose_one")
    if has(card, "quest") and card.get("type") == "SPELL":
        out.add("quest")
    if has(card, "kindred"):
        out.add("kindred")
    if has(card, "rewind"):
        out.add("rewind")
    if has(card, "fabled"):
        out.add("fabled")
    if has(card, "colossal"):
        out.add("colossal")
    if has(card, "herald"):
        out.add("herald")
    if has(card, "shatter"):
        out.add("shatter")
    if "deathwing" in card.get("name", "").lower() or has(card, "deathwing"):
        out.add("deathwing")
    return out


def lines(cards) -> list[str]:
    return [f"- [ ] {c['id']} {c['name']}" for c in sorted(cards, key=lambda c: c["id"])]


def note(ids: list[str], msg: str) -> list[str]:
    return [f"> note: {msg}: {', '.join(ids)}"] if ids else []


def edr(cards) -> list[list[str]]:
    main = [c for c in cards if not c.get("isMiniSet")]
    mini = [c for c in cards if c.get("isMiniSet")]
    w1 = [c for c in main if "imbue" in categorize(c)]
    w2 = [c for c in main if "dark_gift" in categorize(c)]
    w3 = [c for c in main if "choose_one" in categorize(c)]
    w1w2w3 = {c["id"] for c in w1 + w2 + w3}
    w4 = [c for c in main if c["id"] not in w1w2w3]
    wild = [c for c in w4 if c.get("elite")]
    rest = [c for c in w4 if not c.get("elite")]
    mini_note = note([c["id"] for c in mini if "imbue" in categorize(c) or "dark_gift" in categorize(c)],
                     "miniset cards also reuse Imbue/Dark Gift")
    return [lines(w1), lines(w2), lines(w3),
            lines(wild) + ["> Wild Gods — one per class (elite)"] + lines(rest),
            lines(mini) + mini_note]


def ungoro(cards) -> list[list[str]]:
    main = [c for c in cards if not c.get("isMiniSet")]
    mini = [c for c in cards if c.get("isMiniSet")]
    w1 = [c for c in main if "quest" in categorize(c)]
    w3 = [c for c in main if "kindred" in categorize(c)]
    w13 = {c["id"] for c in w1 + w3}
    rest = [c for c in main if c["id"] not in w13]
    return [lines(w1), lines(w3), lines(rest) + lines(mini)]


def timeways(cards) -> list[list[str]]:
    main = [c for c in cards if not c.get("isMiniSet")]
    mini = [c for c in cards if c.get("isMiniSet")]
    w1 = [c for c in main if "rewind" in categorize(c)]
    w2 = [c for c in main if "fabled" in categorize(c)]
    w12 = {c["id"] for c in w1 + w2}
    rest = [c for c in main if c["id"] not in w12]
    azshara = [c["id"] for c in w2 if "choose_one" in categorize(c)]
    return [lines(w1) + lines(w2) + note(azshara, "Lady Azshara is also a Choose One card"),
            lines(rest) + lines(mini)]


def cataclysm(cards) -> list[list[str]]:
    main = [c for c in cards if not c.get("isMiniSet")]
    class_sets = [c for c in cards if c.get("isMiniSet")]
    w1 = [c for c in main if "colossal" in categorize(c)]
    w2 = [c for c in main if "herald" in categorize(c) and "deathwing" not in categorize(c)]
    w3 = [c for c in main if "shatter" in categorize(c)]
    w4 = [c for c in main if "deathwing" in categorize(c)]
    w123 = {c["id"] for c in w1 + w2 + w3 + w4}
    rest = [c for c in main if c["id"] not in w123]
    return [lines(w1), lines(w2), lines(w3),
            lines(w4) + lines(rest)
            + ["> Class Sets (follow-up wave, 29 cards — the miniset slot):"]
            + lines(class_sets)]


def violet_hold(cards) -> list[list[str]]:
    elite = [c for c in cards if c.get("elite")]
    rest = [c for c in cards if not c.get("elite")]
    return [lines(elite) + ["> Rulebreaker wave membership verified per-card text during W1/W2 implementation"],
            lines(rest)]


GENERATORS = {
    "EMERALD_DREAM": ("expansion-emerald-dream-roadmap", edr),
    "THE_LOST_CITY": ("expansion-ungoro-roadmap", ungoro),
    "TIME_TRAVEL": ("expansion-timeways-roadmap", timeways),
    "CATACLYSM": ("expansion-cataclysm-roadmap", cataclysm),
    "ESCAPEFROM_VIOLET_HOLD": ("expansion-violet-hold-roadmap", violet_hold),
}


def fill(path: Path, lists: list[list[str]]) -> bool:
    text = path.read_text()
    it = iter(lists)
    for placeholder in (PLACEHOLDER_ZH, PLACEHOLDER_EN):
        if placeholder in text:
            break
    else:
        raise SystemExit(f"no placeholder found in {path}")
    changed = False
    out = []
    for line in text.splitlines():
        if line == placeholder:
            try:
                block = next(it)
            except StopIteration as exc:
                raise SystemExit(f"too many placeholders in {path}") from exc
            out.extend(block)
            changed = True
        else:
            out.append(line)
    if changed:
        path.write_text("\n".join(out) + "\n")
    return changed


def main() -> int:
    for set_code, (stem, generator) in GENERATORS.items():
        cards = json.loads((DATA_DIR / f"{set_code}.json").read_text())
        lists = generator(cards)
        total = sum(len(b) for b in lists)
        print(f"{set_code}: {total} checklist lines across {len(lists)} waves")
        for suffix in ("", "-zh"):
            path = DOCS_DIR / f"{stem}{suffix}.md"
            if fill(path, lists):
                print(f"  updated {path.name}")
            else:
                print(f"  unchanged {path.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
