use crate::data::books::{self, BookInfo};

/// A parsed Bible reference.
#[derive(Debug, Clone)]
pub struct BibleReference {
    pub book: &'static BookInfo,
    pub chapter: u32,
    pub verse_start: Option<u32>,
    pub verse_end: Option<u32>,
}

impl BibleReference {
    pub fn display(&self) -> String {
        match (self.verse_start, self.verse_end) {
            (Some(start), Some(end)) if start != end => {
                format!("{} {}:{}-{}", self.book.name, self.chapter, start, end)
            }
            (Some(start), _) => {
                format!("{} {}:{}", self.book.name, self.chapter, start)
            }
            _ => {
                format!("{} {}", self.book.name, self.chapter)
            }
        }
    }
}

/// Parse a Bible reference string into a structured reference.
///
/// Supports formats:
/// - "John 3:16"        -> book=John, chapter=3, verse=16
/// - "John 3:16-18"     -> book=John, chapter=3, verses=16-18
/// - "Genesis 1"        -> book=Genesis, chapter=1 (whole chapter)
/// - "1 Cor 13"         -> book=1 Corinthians, chapter=13
/// - "Ps 23:1-6"        -> book=Psalms, chapter=23, verses=1-6
/// - "jn3:16"           -> book=John, chapter=3, verse=16 (no space)
/// - "João 3.16"        -> localized book names, "." or "," separators
/// - "1. Mose 3,16"     -> German-style numbered books
pub fn parse(input: &str) -> Result<BibleReference, String> {
    let input = input.trim();

    if input.is_empty() {
        return Err("Empty reference".to_string());
    }

    // Book names may themselves contain digits ("요한1서" = 1 John), so
    // every digit run is a candidate split point between book name and
    // chapter/verse. Try them left to right; first fully valid parse wins.
    let candidates = split_candidates(input)?;
    let mut first_err: Option<String> = None;
    for (book_str, rest) in &candidates {
        match parse_candidate(book_str, rest) {
            Ok(r) => return Ok(r),
            Err(e) => first_err.get_or_insert(e),
        };
    }
    Err(first_err.unwrap_or_else(|| "No book name found".to_string()))
}

fn parse_candidate(book_str: &str, rest: &str) -> Result<BibleReference, String> {
    let book = books::normalize_book(book_str)
        .ok_or_else(|| format!("Unknown book: '{}'", book_str))?;

    if rest.is_empty() {
        // Just a book name with no chapter — default to chapter 1
        return Ok(BibleReference {
            book,
            chapter: 1,
            verse_start: None,
            verse_end: None,
        });
    }

    let (chapter, verse_start, verse_end) = parse_chapter_verse(rest)?;

    if chapter == 0 || chapter > book.chapters {
        return Err(format!(
            "{} has {} chapters, but chapter {} was requested",
            book.name, book.chapters, chapter
        ));
    }

    Ok(BibleReference {
        book,
        chapter,
        verse_start,
        verse_end,
    })
}

/// Split input into (book_name, chapter_verse_rest) candidates, one per
/// digit run (plus the whole input as a bare book name).
fn split_candidates(input: &str) -> Result<Vec<(String, String)>, String> {
    // Byte offsets alongside chars so slicing stays correct for
    // multi-byte names like "João" or "Об'явлення".
    let chars: Vec<(usize, char)> = input.char_indices().collect();
    let len = chars.len();
    let mut i = 0;

    // Skip a leading numbered-book prefix: "1", "2", "3" followed by a
    // letter, space, dot, or hyphen ("1 Cor", "1cor", "1. Mose", "1-а Царів").
    if i < len && chars[i].1.is_ascii_digit() && chars[i].1 != '0' {
        let mut j = i + 1;
        if j < len && (chars[j].1 == '.' || chars[j].1 == '-') {
            j += 1;
        }
        while j < len && chars[j].1 == ' ' {
            j += 1;
        }
        if j < len && chars[j].1.is_alphabetic() {
            i = j;
        } else {
            // Bare digits ("3:16", "1.16") — no book name in the input.
            return Err("No book name found".to_string());
        }
    }

    let mut candidates: Vec<(String, String)> = Vec::new();
    while candidates.len() < 4 {
        // Advance to the next digit run.
        while i < len && !chars[i].1.is_ascii_digit() {
            i += 1;
        }
        if i >= len {
            break;
        }
        let split_at = chars[i].0;
        let book_str = input[..split_at].trim();
        if !book_str.is_empty() {
            candidates.push((book_str.to_string(), input[split_at..].trim().to_string()));
        }
        // Step past this digit run and keep scanning.
        while i < len && chars[i].1.is_ascii_digit() {
            i += 1;
        }
    }

    // The whole input as a book name with no chapter ("Jude", "요한1서").
    candidates.push((input.to_string(), String::new()));

    Ok(candidates)
}

/// Parse "3:16", "3.16", "3,16", "3:16-18", "3" into
/// (chapter, verse_start, verse_end). The chapter/verse separator may be
/// ":", "." or "," — many languages write "João 3.16" or "Johannes 3,16".
fn parse_chapter_verse(input: &str) -> Result<(u32, Option<u32>, Option<u32>), String> {
    let input = input.trim();

    if let Some((chapter_str, verse_part)) = input.split_once([':', '.', ',']) {
        let chapter: u32 = chapter_str
            .trim()
            .parse()
            .map_err(|_| format!("Invalid chapter number: '{}'", chapter_str))?;

        if let Some((start_str, end_str)) = verse_part.split_once(['-', '\u{2013}']) {
            let start: u32 = start_str
                .trim()
                .parse()
                .map_err(|_| format!("Invalid verse number: '{}'", start_str))?;
            let end: u32 = end_str
                .trim()
                .parse()
                .map_err(|_| format!("Invalid verse number: '{}'", end_str))?;
            Ok((chapter, Some(start), Some(end)))
        } else {
            let verse: u32 = verse_part
                .trim()
                .parse()
                .map_err(|_| format!("Invalid verse number: '{}'", verse_part))?;
            Ok((chapter, Some(verse), Some(verse)))
        }
    } else {
        // Just a chapter number
        let chapter: u32 = input
            .parse()
            .map_err(|_| format!("Invalid chapter number: '{}'", input))?;
        Ok((chapter, None, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_verse() {
        let r = parse("John 3:16").unwrap();
        assert_eq!(r.book.name, "John");
        assert_eq!(r.chapter, 3);
        assert_eq!(r.verse_start, Some(16));
        assert_eq!(r.verse_end, Some(16));
    }

    #[test]
    fn test_verse_range() {
        let r = parse("Psalm 23:1-6").unwrap();
        assert_eq!(r.book.name, "Psalms");
        assert_eq!(r.chapter, 23);
        assert_eq!(r.verse_start, Some(1));
        assert_eq!(r.verse_end, Some(6));
    }

    #[test]
    fn test_whole_chapter() {
        let r = parse("Genesis 1").unwrap();
        assert_eq!(r.book.name, "Genesis");
        assert_eq!(r.chapter, 1);
        assert_eq!(r.verse_start, None);
    }

    #[test]
    fn test_numbered_book() {
        let r = parse("1 Cor 13").unwrap();
        assert_eq!(r.book.name, "1 Corinthians");
        assert_eq!(r.chapter, 13);
    }

    #[test]
    fn test_abbreviation() {
        let r = parse("Jn 3:16").unwrap();
        assert_eq!(r.book.name, "John");
        assert_eq!(r.chapter, 3);
        assert_eq!(r.verse_start, Some(16));
    }

    #[test]
    fn test_abbreviated_numbered_book() {
        let r = parse("1jn 5:3").unwrap();
        assert_eq!(r.book.name, "1 John");
        assert_eq!(r.chapter, 5);
        assert_eq!(r.verse_start, Some(3));
    }

    #[test]
    fn test_display() {
        let r = parse("John 3:16").unwrap();
        assert_eq!(r.display(), "John 3:16");

        let r = parse("Genesis 1").unwrap();
        assert_eq!(r.display(), "Genesis 1");

        let r = parse("Psalm 23:1-6").unwrap();
        assert_eq!(r.display(), "Psalms 23:1-6");
    }

    #[test]
    fn test_invalid_chapter() {
        assert!(parse("Genesis 51").is_err());
    }

    #[test]
    fn test_invalid_book() {
        assert!(parse("Notabook 1:1").is_err());
    }

    #[test]
    fn test_dot_separator() {
        let r = parse("John 3.16").unwrap();
        assert_eq!(r.book.name, "John");
        assert_eq!(r.chapter, 3);
        assert_eq!(r.verse_start, Some(16));
    }

    #[test]
    fn test_comma_separator() {
        let r = parse("John 3,16").unwrap();
        assert_eq!(r.book.name, "John");
        assert_eq!(r.chapter, 3);
        assert_eq!(r.verse_start, Some(16));
    }

    #[test]
    fn test_dot_separator_range() {
        let r = parse("Ps 23.1-6").unwrap();
        assert_eq!(r.book.name, "Psalms");
        assert_eq!(r.chapter, 23);
        assert_eq!(r.verse_start, Some(1));
        assert_eq!(r.verse_end, Some(6));
    }

    #[test]
    fn test_german_style_numbered_book() {
        // "1." prefix before the book name (e.g. "1. Mose", "1. Korinther")
        let r = parse("1. John 5:3").unwrap();
        assert_eq!(r.book.name, "1 John");
        assert_eq!(r.chapter, 5);
        assert_eq!(r.verse_start, Some(3));
    }

    #[test]
    fn test_bare_numbers_are_not_a_book() {
        assert!(parse("1.16").is_err());
        assert!(parse("3:16").is_err());
    }

    #[test]
    fn test_multibyte_book_name_splits_cleanly() {
        // Book/location splitting must be UTF-8 safe even for unknown names.
        let err = parse("Жоао 3:16").unwrap_err();
        assert!(err.contains("Жоао"), "book name preserved intact: {}", err);
    }

    #[test]
    fn test_portuguese_reference() {
        // The exact example from the issue: "João 3.16"
        let r = parse("João 3.16").unwrap();
        assert_eq!(r.book.name, "John");
        assert_eq!(r.chapter, 3);
        assert_eq!(r.verse_start, Some(16));
    }

    #[test]
    fn test_german_reference() {
        let r = parse("1. Mose 3,16").unwrap();
        assert_eq!(r.book.name, "Genesis");
        assert_eq!(r.chapter, 3);
        assert_eq!(r.verse_start, Some(16));
    }

    #[test]
    fn test_cyrillic_reference() {
        let r = parse("Буття 1").unwrap();
        assert_eq!(r.book.name, "Genesis");
        let r = parse("Псалми 23").unwrap();
        assert_eq!(r.book.name, "Psalms");
    }

    #[test]
    fn test_cjk_reference() {
        let r = parse("約翰福音 3:16").unwrap();
        assert_eq!(r.book.name, "John");
        let r = parse("요한복음 3.16").unwrap();
        assert_eq!(r.book.name, "John");
        assert_eq!(r.verse_start, Some(16));
    }

    #[test]
    fn test_book_names_containing_digits() {
        // Korean epistles are written with an embedded digit ("요한1서" =
        // 1 John): the digit must not be mistaken for the chapter.
        let r = parse("요한1서 5:3").unwrap();
        assert_eq!(r.book.name, "1 John");
        assert_eq!(r.chapter, 5);
        assert_eq!(r.verse_start, Some(3));

        // Bare book name with an embedded digit, no chapter.
        let r = parse("요한1서").unwrap();
        assert_eq!(r.book.name, "1 John");
        assert_eq!(r.chapter, 1);
    }
}
