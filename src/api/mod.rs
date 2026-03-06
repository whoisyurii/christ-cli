pub mod bolls;
pub mod types;

use crate::data::kjv;
use types::{Chapter, SearchResult, Verse};

pub struct Resolver {
    bolls: bolls::BollsProvider,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            bolls: bolls::BollsProvider::new(),
        }
    }

    pub async fn get_verse(
        &self,
        book: &str,
        chapter: u32,
        verse: u32,
        translation: &str,
    ) -> Result<Verse, String> {
        // Try bundled KJV first for offline support
        if translation.eq_ignore_ascii_case("KJV") {
            if let Some(v) = kjv::get_verse(book, chapter, verse) {
                return Ok(v);
            }
        }

        // Try Bolls API
        match self.bolls.get_verse(book, chapter, verse, translation).await {
            Ok(v) => Ok(v),
            Err(e) => Err(format!("Failed to fetch verse: {}", e)),
        }
    }

    pub async fn get_chapter(
        &self,
        book: &str,
        chapter: u32,
        translation: &str,
    ) -> Result<Chapter, String> {
        // Try bundled KJV first
        if translation.eq_ignore_ascii_case("KJV") {
            if let Some(c) = kjv::get_chapter(book, chapter) {
                return Ok(c);
            }
        }

        // Try Bolls API
        match self.bolls.get_chapter(book, chapter, translation).await {
            Ok(c) => Ok(c),
            Err(e) => Err(format!("Failed to fetch chapter: {}", e)),
        }
    }

    pub async fn get_verse_range(
        &self,
        book: &str,
        chapter: u32,
        verse_start: u32,
        verse_end: u32,
        translation: &str,
    ) -> Result<Vec<Verse>, String> {
        // Try bundled KJV first
        if translation.eq_ignore_ascii_case("KJV") {
            let verses = kjv::get_verse_range(book, chapter, verse_start, verse_end);
            if !verses.is_empty() {
                return Ok(verses);
            }
        }

        // Try Bolls API
        match self
            .bolls
            .get_verse_range(book, chapter, verse_start, verse_end, translation)
            .await
        {
            Ok(v) => Ok(v),
            Err(e) => Err(format!("Failed to fetch verses: {}", e)),
        }
    }

    pub async fn search(
        &self,
        query: &str,
        translation: &str,
    ) -> Result<Vec<SearchResult>, String> {
        // For KJV, use bundled data (instant, clean text, no Strong's numbers)
        if translation.eq_ignore_ascii_case("KJV") {
            return Ok(kjv::search(query));
        }

        // For other translations, use Bolls API
        match self.bolls.search(query, translation).await {
            Ok(results) => Ok(results),
            Err(e) => Err(format!("Search failed: {}", e)),
        }
    }

    pub async fn get_book_names(&self, translation: &str) -> Result<Vec<String>, String> {
        self.bolls.get_book_names(translation).await
    }

    pub async fn get_random_verse(&self, translation: &str) -> Result<Verse, String> {
        if translation.eq_ignore_ascii_case("KJV") {
            return Ok(kjv::random_verse());
        }

        match self.bolls.get_random_verse(translation).await {
            Ok(v) => Ok(v),
            Err(_) => Ok(kjv::random_verse()),
        }
    }
}
