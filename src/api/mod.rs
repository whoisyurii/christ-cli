pub mod bolls;
pub mod types;

use crate::data::{books, kjv};
use crate::store::cache;
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

        // Try disk cache
        if let Some(book_info) = books::normalize_book(book) {
            if let Some(ch) = cache::load_chapter(translation, book_info.bolls_id, chapter) {
                if let Some(v) = ch.verses.into_iter().find(|v| v.verse == verse) {
                    return Ok(v);
                }
            }
        }

        // Cache miss on a "fully cached" translation — cache is incomplete, invalidate
        if cache::is_fully_cached(translation) {
            cache::remove_complete_marker(translation);
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

        // Try disk cache
        if let Some(book_info) = books::normalize_book(book) {
            if let Some(c) = cache::load_chapter(translation, book_info.bolls_id, chapter) {
                return Ok(c);
            }
        }

        // Cache miss on a "fully cached" translation — cache is incomplete, invalidate
        if cache::is_fully_cached(translation) {
            cache::remove_complete_marker(translation);
        }

        // Fetch from Bolls API and cache the result
        match self.bolls.get_chapter(book, chapter, translation).await {
            Ok(c) => {
                if let Some(book_info) = books::normalize_book(book) {
                    cache::save_chapter(translation, book_info.bolls_id, &c);
                }
                Ok(c)
            }
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

        // Try disk cache
        if let Some(book_info) = books::normalize_book(book) {
            if let Some(ch) = cache::load_chapter(translation, book_info.bolls_id, chapter) {
                let verses: Vec<Verse> = ch
                    .verses
                    .into_iter()
                    .filter(|v| v.verse >= verse_start && v.verse <= verse_end)
                    .collect();
                if !verses.is_empty() {
                    return Ok(verses);
                }
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
        // For KJV, use bundled data
        if translation.eq_ignore_ascii_case("KJV") {
            return Ok(kjv::search(query));
        }

        // Always try local cache first — instant for any cached chapters
        let cached = cache::search(translation, query);
        if !cached.is_empty() || cache::has_cached_data(translation) {
            return Ok(cached);
        }

        // No cached data at all — use Bolls API
        match self.bolls.search(query, translation).await {
            Ok(results) => Ok(results),
            Err(e) => Err(format!("Search failed: {}", e)),
        }
    }

    pub async fn get_book_names(&self, translation: &str) -> Result<Vec<String>, String> {
        // Try disk cache first
        if let Some(names) = cache::load_book_names(translation) {
            return Ok(names);
        }

        // Don't hit network if fully cached — fallback to English names
        if cache::is_fully_cached(translation) {
            return Ok(Vec::new());
        }

        // Fetch and cache
        match self.bolls.get_book_names(translation).await {
            Ok(names) => {
                cache::save_book_names(translation, &names);
                Ok(names)
            }
            Err(e) => Err(e),
        }
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
