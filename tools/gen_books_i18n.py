#!/usr/bin/env python3
"""Regenerate the localized book tables in src/data/books_i18n.rs.

Reads tools/book_tables.json (66 entries per language, indexed against
data/books.rs BOOKS order) and rewrites everything below the generated-code
marker in src/data/books_i18n.rs.

The Rust lookup runs in namespaced passes (strict names > abbrevs > folded
names > unambiguous prefix), so conflicts are policed per pass:
- a NAME whose strict or folded canonical form maps to a DIFFERENT book in
  another language (or in English) is a hard generation failure — fix the
  data;
- an ABBREV that collides with English for a different book is dropped
  (English matching runs first, the entry would be dead weight);
- an ABBREV that collides with another language's abbrev for a different
  book is dropped from BOTH (e.g. Russian "1 Цар" is 1 Samuel but Ukrainian
  "1 цар" is 1 Kings — neither should win silently; the unambiguous-prefix
  rule in the Rust lookup then rejects the input instead of guessing);
- an abbrev colliding with a folded NAME is allowed — pass precedence
  resolves it (Portuguese "jo" -> João via abbrevs, "Jó" -> Job via strict
  names);
- Ukrainian/Russian ordinal forms ("1-а Царів", "2-е Петра") are
  synthesized for numbered books;
- duplicates within an entry are deduped by strict canonical form.
"""

import json
import re
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
JSON_PATH = ROOT / "tools" / "book_tables.json"
RS_PATH = ROOT / "src" / "data" / "books_i18n.rs"
MARKER = "// Generated tables below"

LANG_LABELS = {
    "pt": "Português",
    "es": "Español",
    "fr": "Français",
    "de": "Deutsch",
    "it": "Italiano",
    "nl": "Nederlands",
    "uk": "Українська",
    "ru": "Русский",
    "zh": "中文",
    "ko": "한국어",
    "ja": "日本語",
}

ENGLISH = ["Genesis", "Exodus", "Leviticus", "Numbers", "Deuteronomy", "Joshua", "Judges", "Ruth", "1 Samuel", "2 Samuel", "1 Kings", "2 Kings", "1 Chronicles", "2 Chronicles", "Ezra", "Nehemiah", "Esther", "Job", "Psalms", "Proverbs", "Ecclesiastes", "Song of Solomon", "Isaiah", "Jeremiah", "Lamentations", "Ezekiel", "Daniel", "Hosea", "Joel", "Amos", "Obadiah", "Jonah", "Micah", "Nahum", "Habakkuk", "Zephaniah", "Haggai", "Zechariah", "Malachi", "Matthew", "Mark", "Luke", "John", "Acts", "Romans", "1 Corinthians", "2 Corinthians", "Galatians", "Ephesians", "Philippians", "Colossians", "1 Thessalonians", "2 Thessalonians", "1 Timothy", "2 Timothy", "Titus", "Philemon", "Hebrews", "James", "1 Peter", "2 Peter", "1 John", "2 John", "3 John", "Jude", "Revelation"]

EN_ABBREVS = {0: ["gen", "ge", "gn"], 1: ["exo", "ex", "exod"], 2: ["lev", "le", "lv"], 3: ["num", "nu", "nm", "nb"], 4: ["deu", "de", "dt", "deut"], 5: ["jos", "josh", "jsh"], 6: ["jdg", "judg", "jg", "jdgs"], 7: ["rut", "ru", "rth"], 8: ["1sa", "1sam", "1sm"], 9: ["2sa", "2sam", "2sm"], 10: ["1ki", "1kgs", "1kin"], 11: ["2ki", "2kgs", "2kin"], 12: ["1ch", "1chr", "1chron"], 13: ["2ch", "2chr", "2chron"], 14: ["ezr", "ez"], 15: ["neh", "ne"], 16: ["est", "esth"], 17: ["job", "jb"], 18: ["psa", "ps", "psalm", "psm", "pss"], 19: ["pro", "pr", "prov", "prv"], 20: ["ecc", "ec", "eccl", "eccles"], 21: ["sng", "sos", "song", "sol"], 22: ["isa", "is"], 23: ["jer", "je", "jr"], 24: ["lam", "la"], 25: ["ezk", "eze", "ezek"], 26: ["dan", "da", "dn"], 27: ["hos", "ho"], 28: ["joe", "jl", "joel"], 29: ["amo", "am"], 30: ["oba", "ob", "obad"], 31: ["jon", "jnh"], 32: ["mic", "mc"], 33: ["nah", "na"], 34: ["hab", "hb"], 35: ["zep", "zp", "zeph"], 36: ["hag", "hg"], 37: ["zec", "zc", "zech"], 38: ["mal", "ml"], 39: ["mat", "mt", "matt"], 40: ["mrk", "mk", "mar"], 41: ["luk", "lk", "lu"], 42: ["jhn", "jn", "joh"], 43: ["act", "ac"], 44: ["rom", "ro", "rm"], 45: ["1co", "1cor"], 46: ["2co", "2cor"], 47: ["gal", "ga"], 48: ["eph", "ep"], 49: ["php", "phil", "pp"], 50: ["col", "co"], 51: ["1th", "1thess", "1thes"], 52: ["2th", "2thess", "2thes"], 53: ["1ti", "1tim"], 54: ["2ti", "2tim"], 55: ["tit", "ti"], 56: ["phm", "philem"], 57: ["heb", "he"], 58: ["jas", "jm", "jam"], 59: ["1pe", "1pet", "1pt"], 60: ["2pe", "2pet", "2pt"], 61: ["1jn", "1jo", "1john"], 62: ["2jn", "2jo", "2john"], 63: ["3jn", "3jo", "3john"], 64: ["jud", "jde"], 65: ["rev", "re", "rv"]}

FOLD = {"ß": "ss", "ё": "е"}

# Ordinal prefixes synthesized for numbered books ("1 Царів" -> "1-а Царів").
ORDINAL_SUFFIXES = {"uk": ["а", "е", "ше"], "ru": ["я", "е"]}


def canon(s: str) -> str:
    """Mirror of books::canon in Rust (diacritics folded)."""
    out = []
    for c in s.strip().lower():
        if c in ". -'’ʼ":
            continue
        if unicodedata.combining(c):
            continue
        if c in FOLD:
            out.append(FOLD[c])
            continue
        d = unicodedata.normalize("NFD", c)
        base = d[0]
        if base.isascii() and base.isalpha() and len(d) > 1:
            out.append(base)
        else:
            out.append(c)
    return "".join(out)


def canon_strict(s: str) -> str:
    """Mirror of books::canon_strict in Rust (diacritics preserved)."""
    return "".join(c for c in s.strip().lower() if c not in ". -'’ʼ")


def main() -> None:
    data = json.loads(JSON_PATH.read_text(encoding="utf-8"))
    for t in data:
        assert len(t["entries"]) == 66, (t["lang"], len(t["entries"]))
        for i, e in enumerate(t["entries"]):
            assert e["index"] == i, (t["lang"], i, e["index"])
            assert e["names"], (t["lang"], i)

    # Synthesize Cyrillic ordinal variants for numbered books.
    for t in data:
        for suffix_list, e in ((ORDINAL_SUFFIXES.get(t["lang"]), e) for e in t["entries"]):
            if not suffix_list:
                continue
            extra = []
            for n in e["names"]:
                if len(n) > 2 and n[0] in "1234" and n[1] == " ":
                    extra.extend(f"{n[0]}-{sfx} {n[2:]}" for sfx in suffix_list)
            e["names"] = e["names"] + extra

    english_claims = {}
    for i, n in enumerate(ENGLISH):
        english_claims.setdefault(canon(n), i)
    for i, abbrevs in EN_ABBREVS.items():
        for a in abbrevs:
            english_claims.setdefault(canon(a), i)

    # Hard-fail on name conflicts (strict or folded): those would silently
    # mis-resolve inside a single lookup pass.
    for key_fn, label in ((canon_strict, "strict"), (canon, "folded")):
        name_claims = dict(english_claims) if key_fn is canon else {}
        if key_fn is canon_strict:
            name_claims = dict(english_claims)  # English keys are ASCII: same either way
        for t in data:
            for e in t["entries"]:
                for n in e["names"]:
                    key = key_fn(n)
                    prev = name_claims.setdefault(key, e["index"])
                    if prev != e["index"]:
                        raise SystemExit(
                            f"{label} name conflict: '{n}' ({t['lang']} #{e['index']}) "
                            f"already claimed by book {prev}"
                        )

    # Abbrev claims across languages: a folded abbrev pointing at two
    # different books is ambiguous and dropped from both sides. Abbrevs
    # that merely restate one of their own entry's names (pt Jó's "jó")
    # are redundant — the name passes already cover them — and must not
    # poison the ambiguity analysis for other entries (João's "jo").
    abbrev_claims = {}
    for t in data:
        for e in t["entries"]:
            own_names = {canon(n) for n in e["names"]}
            for a in e["abbrevs"]:
                c = canon(a)
                if c not in own_names:
                    abbrev_claims.setdefault(c, set()).add(e["index"])

    dropped = []

    def keep(lang, idx, value, is_name):
        c = canon(value)
        if not c:
            return False
        en_idx = english_claims.get(c)
        if en_idx is not None and en_idx != idx:
            dropped.append((lang, idx, value, f"collides with English for book {en_idx}"))
            return False
        if not is_name and len(abbrev_claims.get(c, ())) > 1:
            dropped.append((lang, idx, value, f"ambiguous across languages {sorted(abbrev_claims[c])}"))
            return False
        return True

    lines = []
    lines.append(f"{MARKER} — regenerate with tools/gen_books_i18n.py.")
    lines.append("#[rustfmt::skip]")
    lines.append("pub static LANGUAGES: &[LanguageTable] = &[")
    for t in data:
        label = LANG_LABELS[t["lang"]]
        lines.append(f'    LanguageTable {{ lang: "{label}", books: &[')
        for e in t["entries"]:
            seen = set()
            names, abbrevs = [], []
            for n in e["names"]:
                c = canon(n)
                if c in seen or not keep(t["lang"], e["index"], n, True):
                    continue
                seen.add(c)
                names.append(n)
            for a in e["abbrevs"]:
                c = canon(a)
                if c in seen or not keep(t["lang"], e["index"], a, False):
                    continue
                seen.add(c)
                abbrevs.append(a)
            if not names:
                raise SystemExit(f"{t['lang']} #{e['index']}: every name dropped")

            def lit(values):
                return ", ".join('"' + v.replace("\\", "\\\\").replace('"', '\\"') + '"' for v in values)

            lines.append(
                f"        LocalizedBook {{ idx: {e['index']}, "
                f"names: &[{lit(names)}], abbrevs: &[{lit(abbrevs)}] }},"
            )
        lines.append("    ] },")
    lines.append("];")
    generated = "\n".join(lines) + "\n"

    src = RS_PATH.read_text(encoding="utf-8")
    head = re.split(re.escape(MARKER), src)[0].rstrip() + "\n\n"
    RS_PATH.write_text(head + generated, encoding="utf-8")

    print(f"wrote {RS_PATH} ({len(data)} languages)")
    for d in dropped:
        print("dropped:", d)


if __name__ == "__main__":
    main()
