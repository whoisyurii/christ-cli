/// All 66 books of the Bible with their abbreviations and Bolls API book numbers.
#[derive(Debug, Clone)]
pub struct BookInfo {
    pub name: &'static str,
    pub abbrevs: &'static [&'static str],
    pub bolls_id: u32,
    pub chapters: u32,
}

pub static BOOKS: &[BookInfo] = &[
    BookInfo { name: "Genesis", abbrevs: &["gen", "ge", "gn"], bolls_id: 1, chapters: 50 },
    BookInfo { name: "Exodus", abbrevs: &["exo", "ex", "exod"], bolls_id: 2, chapters: 40 },
    BookInfo { name: "Leviticus", abbrevs: &["lev", "le", "lv"], bolls_id: 3, chapters: 27 },
    BookInfo { name: "Numbers", abbrevs: &["num", "nu", "nm", "nb"], bolls_id: 4, chapters: 36 },
    BookInfo { name: "Deuteronomy", abbrevs: &["deu", "de", "dt", "deut"], bolls_id: 5, chapters: 34 },
    BookInfo { name: "Joshua", abbrevs: &["jos", "josh", "jsh"], bolls_id: 6, chapters: 24 },
    BookInfo { name: "Judges", abbrevs: &["jdg", "judg", "jg", "jdgs"], bolls_id: 7, chapters: 21 },
    BookInfo { name: "Ruth", abbrevs: &["rut", "ru", "rth"], bolls_id: 8, chapters: 4 },
    BookInfo { name: "1 Samuel", abbrevs: &["1sa", "1sam", "1sm"], bolls_id: 9, chapters: 31 },
    BookInfo { name: "2 Samuel", abbrevs: &["2sa", "2sam", "2sm"], bolls_id: 10, chapters: 24 },
    BookInfo { name: "1 Kings", abbrevs: &["1ki", "1kgs", "1kin"], bolls_id: 11, chapters: 22 },
    BookInfo { name: "2 Kings", abbrevs: &["2ki", "2kgs", "2kin"], bolls_id: 12, chapters: 25 },
    BookInfo { name: "1 Chronicles", abbrevs: &["1ch", "1chr", "1chron"], bolls_id: 13, chapters: 29 },
    BookInfo { name: "2 Chronicles", abbrevs: &["2ch", "2chr", "2chron"], bolls_id: 14, chapters: 36 },
    BookInfo { name: "Ezra", abbrevs: &["ezr", "ez"], bolls_id: 15, chapters: 10 },
    BookInfo { name: "Nehemiah", abbrevs: &["neh", "ne"], bolls_id: 16, chapters: 13 },
    BookInfo { name: "Esther", abbrevs: &["est", "esth"], bolls_id: 17, chapters: 10 },
    BookInfo { name: "Job", abbrevs: &["job", "jb"], bolls_id: 18, chapters: 42 },
    BookInfo { name: "Psalms", abbrevs: &["psa", "ps", "psalm", "psm", "pss"], bolls_id: 19, chapters: 150 },
    BookInfo { name: "Proverbs", abbrevs: &["pro", "pr", "prov", "prv"], bolls_id: 20, chapters: 31 },
    BookInfo { name: "Ecclesiastes", abbrevs: &["ecc", "ec", "eccl", "eccles"], bolls_id: 21, chapters: 12 },
    BookInfo { name: "Song of Solomon", abbrevs: &["sng", "sos", "song", "sol"], bolls_id: 22, chapters: 8 },
    BookInfo { name: "Isaiah", abbrevs: &["isa", "is"], bolls_id: 23, chapters: 66 },
    BookInfo { name: "Jeremiah", abbrevs: &["jer", "je", "jr"], bolls_id: 24, chapters: 52 },
    BookInfo { name: "Lamentations", abbrevs: &["lam", "la"], bolls_id: 25, chapters: 5 },
    BookInfo { name: "Ezekiel", abbrevs: &["ezk", "eze", "ezek"], bolls_id: 26, chapters: 48 },
    BookInfo { name: "Daniel", abbrevs: &["dan", "da", "dn"], bolls_id: 27, chapters: 12 },
    BookInfo { name: "Hosea", abbrevs: &["hos", "ho"], bolls_id: 28, chapters: 14 },
    BookInfo { name: "Joel", abbrevs: &["joe", "jl", "joel"], bolls_id: 29, chapters: 3 },
    BookInfo { name: "Amos", abbrevs: &["amo", "am"], bolls_id: 30, chapters: 9 },
    BookInfo { name: "Obadiah", abbrevs: &["oba", "ob", "obad"], bolls_id: 31, chapters: 1 },
    BookInfo { name: "Jonah", abbrevs: &["jon", "jnh"], bolls_id: 32, chapters: 4 },
    BookInfo { name: "Micah", abbrevs: &["mic", "mc"], bolls_id: 33, chapters: 7 },
    BookInfo { name: "Nahum", abbrevs: &["nah", "na"], bolls_id: 34, chapters: 3 },
    BookInfo { name: "Habakkuk", abbrevs: &["hab", "hb"], bolls_id: 35, chapters: 3 },
    BookInfo { name: "Zephaniah", abbrevs: &["zep", "zp", "zeph"], bolls_id: 36, chapters: 3 },
    BookInfo { name: "Haggai", abbrevs: &["hag", "hg"], bolls_id: 37, chapters: 2 },
    BookInfo { name: "Zechariah", abbrevs: &["zec", "zc", "zech"], bolls_id: 38, chapters: 14 },
    BookInfo { name: "Malachi", abbrevs: &["mal", "ml"], bolls_id: 39, chapters: 4 },
    BookInfo { name: "Matthew", abbrevs: &["mat", "mt", "matt"], bolls_id: 40, chapters: 28 },
    BookInfo { name: "Mark", abbrevs: &["mrk", "mk", "mar"], bolls_id: 41, chapters: 16 },
    BookInfo { name: "Luke", abbrevs: &["luk", "lk", "lu"], bolls_id: 42, chapters: 24 },
    BookInfo { name: "John", abbrevs: &["jhn", "jn", "joh"], bolls_id: 43, chapters: 21 },
    BookInfo { name: "Acts", abbrevs: &["act", "ac"], bolls_id: 44, chapters: 28 },
    BookInfo { name: "Romans", abbrevs: &["rom", "ro", "rm"], bolls_id: 45, chapters: 16 },
    BookInfo { name: "1 Corinthians", abbrevs: &["1co", "1cor"], bolls_id: 46, chapters: 16 },
    BookInfo { name: "2 Corinthians", abbrevs: &["2co", "2cor"], bolls_id: 47, chapters: 13 },
    BookInfo { name: "Galatians", abbrevs: &["gal", "ga"], bolls_id: 48, chapters: 6 },
    BookInfo { name: "Ephesians", abbrevs: &["eph", "ep"], bolls_id: 49, chapters: 6 },
    BookInfo { name: "Philippians", abbrevs: &["php", "phil", "pp"], bolls_id: 50, chapters: 4 },
    BookInfo { name: "Colossians", abbrevs: &["col", "co"], bolls_id: 51, chapters: 4 },
    BookInfo { name: "1 Thessalonians", abbrevs: &["1th", "1thess", "1thes"], bolls_id: 52, chapters: 5 },
    BookInfo { name: "2 Thessalonians", abbrevs: &["2th", "2thess", "2thes"], bolls_id: 53, chapters: 3 },
    BookInfo { name: "1 Timothy", abbrevs: &["1ti", "1tim"], bolls_id: 54, chapters: 6 },
    BookInfo { name: "2 Timothy", abbrevs: &["2ti", "2tim"], bolls_id: 55, chapters: 4 },
    BookInfo { name: "Titus", abbrevs: &["tit", "ti"], bolls_id: 56, chapters: 3 },
    BookInfo { name: "Philemon", abbrevs: &["phm", "philem"], bolls_id: 57, chapters: 1 },
    BookInfo { name: "Hebrews", abbrevs: &["heb", "he"], bolls_id: 58, chapters: 13 },
    BookInfo { name: "James", abbrevs: &["jas", "jm", "jam"], bolls_id: 59, chapters: 5 },
    BookInfo { name: "1 Peter", abbrevs: &["1pe", "1pet", "1pt"], bolls_id: 60, chapters: 5 },
    BookInfo { name: "2 Peter", abbrevs: &["2pe", "2pet", "2pt"], bolls_id: 61, chapters: 3 },
    BookInfo { name: "1 John", abbrevs: &["1jn", "1jo", "1john"], bolls_id: 62, chapters: 5 },
    BookInfo { name: "2 John", abbrevs: &["2jn", "2jo", "2john"], bolls_id: 63, chapters: 1 },
    BookInfo { name: "3 John", abbrevs: &["3jn", "3jo", "3john"], bolls_id: 64, chapters: 1 },
    BookInfo { name: "Jude", abbrevs: &["jud", "jde"], bolls_id: 65, chapters: 1 },
    BookInfo { name: "Revelation", abbrevs: &["rev", "re", "rv"], bolls_id: 66, chapters: 22 },
];

/// Canonicalize a book name for matching: lowercase, fold Latin diacritics,
/// and drop dots, spaces, hyphens, and apostrophes. "João" and "joao" (or
/// "1. Mose" and "1 mose") canonicalize identically.
pub fn canon(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.trim().chars() {
        for lc in c.to_lowercase() {
            match lc {
                '.' | ' ' | '-' | '\'' | '\u{2019}' | '\u{02bc}' => {}
                // Combining diacritics (NFD-normalized input, e.g. text
                // copied from macOS): drop the mark, keep the base letter.
                '\u{0300}'..='\u{036f}' => {}
                'á' | 'à' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => out.push('a'),
                'ç' | 'ć' | 'č' => out.push('c'),
                'é' | 'è' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' | 'ě' => out.push('e'),
                'í' | 'ì' | 'î' | 'ï' | 'ī' | 'į' => out.push('i'),
                'ñ' | 'ń' | 'ň' => out.push('n'),
                'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' => out.push('o'),
                'ú' | 'ù' | 'û' | 'ü' | 'ū' | 'ů' => out.push('u'),
                'ý' | 'ÿ' => out.push('y'),
                'š' | 'ś' => out.push('s'),
                'ž' | 'ź' | 'ż' => out.push('z'),
                'ł' => out.push('l'),
                'ß' => out.push_str("ss"),
                'ё' => out.push('е'),
                other => out.push(other),
            }
        }
    }
    out
}

/// Like `canon`, but diacritics are preserved ("Jó" -> "jó"). Used so an
/// accented name (Portuguese Jó = Job) stays distinct from a same-letters
/// abbreviation (Jo = João/John).
pub fn canon_strict(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.trim().chars() {
        for lc in c.to_lowercase() {
            match lc {
                '.' | ' ' | '-' | '\'' | '\u{2019}' | '\u{02bc}' => {}
                other => out.push(other),
            }
        }
    }
    out
}

/// Normalize a book name input to the canonical book name.
/// Handles full names, abbreviations, case insensitivity, numbered books,
/// and localized names ("João" -> John, "1. Mose" -> Genesis).
pub fn normalize_book(input: &str) -> Option<&'static BookInfo> {
    let folded = canon(input);
    if folded.is_empty() {
        return None;
    }

    // Try exact full name match first
    for book in BOOKS {
        if canon(book.name) == folded {
            return Some(book);
        }
    }

    // Try abbreviation match
    for book in BOOKS {
        for abbrev in book.abbrevs {
            if *abbrev == folded {
                return Some(book);
            }
        }
    }

    // Try prefix match on full name (e.g., "gene" -> "Genesis")
    if folded.chars().count() >= 3 {
        for book in BOOKS {
            if canon(book.name).starts_with(&folded) {
                return Some(book);
            }
        }
    }

    // Fall back to localized book names (Portuguese, Spanish, German, ...)
    super::books_i18n::lookup(&canon_strict(input), &folded).map(|idx| &BOOKS[idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_name() {
        assert_eq!(normalize_book("Genesis").unwrap().name, "Genesis");
        assert_eq!(normalize_book("revelation").unwrap().name, "Revelation");
    }

    #[test]
    fn test_abbreviation() {
        assert_eq!(normalize_book("gen").unwrap().name, "Genesis");
        assert_eq!(normalize_book("jn").unwrap().name, "John");
        assert_eq!(normalize_book("rev").unwrap().name, "Revelation");
        assert_eq!(normalize_book("ps").unwrap().name, "Psalms");
    }

    #[test]
    fn test_numbered_books() {
        assert_eq!(normalize_book("1cor").unwrap().name, "1 Corinthians");
        assert_eq!(normalize_book("2sam").unwrap().name, "2 Samuel");
        assert_eq!(normalize_book("1jn").unwrap().name, "1 John");
    }

    #[test]
    fn test_prefix_match() {
        assert_eq!(normalize_book("gene").unwrap().name, "Genesis");
        assert_eq!(normalize_book("matt").unwrap().name, "Matthew");
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(normalize_book("JOHN").unwrap().name, "John");
        assert_eq!(normalize_book("GEN").unwrap().name, "Genesis");
    }

    #[test]
    fn test_invalid() {
        assert!(normalize_book("notabook").is_none());
        assert!(normalize_book("xyz").is_none());
    }

    #[test]
    fn test_canon_folds_diacritics_and_punctuation() {
        assert_eq!(canon("João"), "joao");
        assert_eq!(canon("Gênesis"), "genesis");
        assert_eq!(canon("1. Mose"), "1mose");
        assert_eq!(canon("1 Corinthians"), "1corinthians");
        assert_eq!(canon("Об'явлення"), "обявлення");
        assert_eq!(canon("Cantar de los Cantares"), "cantardeloscantares");
    }

    #[test]
    fn test_localized_names() {
        // Portuguese (the issue's motivating example)
        assert_eq!(normalize_book("João").unwrap().name, "John");
        assert_eq!(normalize_book("joao").unwrap().name, "John");
        assert_eq!(normalize_book("Jó").unwrap().name, "Job");
        assert_eq!(normalize_book("Gênesis").unwrap().name, "Genesis");
        assert_eq!(normalize_book("Apocalipse").unwrap().name, "Revelation");
        // Spanish
        assert_eq!(normalize_book("Santiago").unwrap().name, "James");
        assert_eq!(normalize_book("Cantares").unwrap().name, "Song of Solomon");
        // German
        assert_eq!(normalize_book("1. Mose").unwrap().name, "Genesis");
        assert_eq!(normalize_book("Offenbarung").unwrap().name, "Revelation");
        // French
        assert_eq!(normalize_book("Genèse").unwrap().name, "Genesis");
        // Ukrainian
        assert_eq!(normalize_book("Буття").unwrap().name, "Genesis");
        assert_eq!(normalize_book("Об'явлення").unwrap().name, "Revelation");
        // Russian (Synodal numbering: 4 Царств is 2 Kings)
        assert_eq!(normalize_book("Бытие").unwrap().name, "Genesis");
        assert_eq!(normalize_book("4 Царств").unwrap().name, "2 Kings");
        // CJK
        assert_eq!(normalize_book("約翰福音").unwrap().name, "John");
        assert_eq!(normalize_book("约翰福音").unwrap().name, "John");
        assert_eq!(normalize_book("요한복음").unwrap().name, "John");
        assert_eq!(normalize_book("ヨハネによる福音書").unwrap().name, "John");
    }

    #[test]
    fn test_english_still_wins_over_localized() {
        // "jo" could prefix-match localized names, but stays unmatched-or-
        // exact: English abbrevs must never be shadowed.
        assert_eq!(normalize_book("jn").unwrap().name, "John");
        assert_eq!(normalize_book("ez").unwrap().name, "Ezra");
        assert_eq!(normalize_book("mc").unwrap().name, "Micah");
    }

    #[test]
    fn test_portuguese_jo_is_john_but_accented_jo_is_job() {
        // "Jo" is the standard Portuguese abbreviation for João (John);
        // only the accented "Jó" is the book of Job.
        assert_eq!(normalize_book("Jo").unwrap().name, "John");
        assert_eq!(normalize_book("jo").unwrap().name, "John");
        assert_eq!(normalize_book("Jó").unwrap().name, "Job");
        assert_eq!(normalize_book("jó").unwrap().name, "Job");
    }

    #[test]
    fn test_ambiguous_cross_language_prefix_is_rejected() {
        // "1 Цар" is 1 Samuel to a Russian reader (1 Царств) but prefixes
        // Ukrainian 1 Kings (1 Царів): refusing beats silently guessing.
        assert!(normalize_book("1 Цар").is_none());
        // One more letter disambiguates.
        assert_eq!(normalize_book("1 Царс").unwrap().name, "1 Samuel");
        assert_eq!(normalize_book("1 Царі").unwrap().name, "1 Kings");
    }

    #[test]
    fn test_cyrillic_ordinal_forms() {
        assert_eq!(normalize_book("1-а Царів").unwrap().name, "1 Kings");
        assert_eq!(normalize_book("1-ше Івана").unwrap().name, "1 John");
        assert_eq!(normalize_book("1-я Царств").unwrap().name, "1 Samuel");
        assert_eq!(normalize_book("2-е Петра").unwrap().name, "2 Peter");
    }

    #[test]
    fn test_nfd_normalized_input() {
        // "João" with the tilde as a combining mark (macOS file paths,
        // some PDFs) must fold the same as the precomposed form.
        assert_eq!(normalize_book("Joa\u{0303}o").unwrap().name, "John");
        assert_eq!(normalize_book("Ge\u{0302}nesis").unwrap().name, "Genesis");
    }
}
