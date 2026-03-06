use crate::api::types::{Chapter, SearchResult, Verse};
use crate::data::books;
use serde::Deserialize;

const BASE_URL: &str = "https://bolls.life";

pub struct BollsProvider {
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct BollsVerse {
    #[serde(alias = "pk")]
    _pk: Option<i64>,
    verse: u32,
    text: String,
}

#[derive(Deserialize)]
struct BollsBook {
    bookid: u32,
    name: String,
}

#[derive(Deserialize)]
struct BollsSearchResult {
    #[serde(default)]
    book: u32,
    #[serde(default)]
    chapter: u32,
    #[serde(default)]
    verse: u32,
    #[serde(default)]
    text: String,
}

impl BollsProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    fn translation_code(translation: &str) -> &str {
        // Bolls uses the code as-is (already uppercase in our TRANSLATIONS list)
        // Leak is fine — these are a small fixed set of user-selected codes
        Box::leak(translation.to_uppercase().into_boxed_str())
    }

    pub async fn get_verse(
        &self,
        book_name: &str,
        chapter: u32,
        verse: u32,
        translation: &str,
    ) -> Result<Verse, String> {
        let book = books::normalize_book(book_name)
            .ok_or_else(|| format!("Unknown book: {}", book_name))?;
        let trans = Self::translation_code(translation);

        let url = format!(
            "{}/get-verse/{}/{}/{}/{}/",
            BASE_URL, trans, book.bolls_id, chapter, verse
        );

        let resp: BollsVerse = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        Ok(Verse {
            book: book.name.to_string(),
            chapter,
            verse,
            text: clean_html(&resp.text),
            translation: translation.to_uppercase(),
        })
    }

    pub async fn get_chapter(
        &self,
        book_name: &str,
        chapter: u32,
        translation: &str,
    ) -> Result<Chapter, String> {
        let book = books::normalize_book(book_name)
            .ok_or_else(|| format!("Unknown book: {}", book_name))?;
        let trans = Self::translation_code(translation);

        let url = format!(
            "{}/get-chapter/{}/{}/{}/",
            BASE_URL, trans, book.bolls_id, chapter
        );

        let resp: Vec<BollsVerse> = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        let verses: Vec<Verse> = resp
            .into_iter()
            .map(|v| Verse {
                book: book.name.to_string(),
                chapter,
                verse: v.verse,
                text: clean_html(&v.text),
                translation: translation.to_uppercase(),
            })
            .collect();

        Ok(Chapter {
            book: book.name.to_string(),
            chapter,
            verses,
            translation: translation.to_uppercase(),
        })
    }

    pub async fn get_verse_range(
        &self,
        book_name: &str,
        chapter: u32,
        verse_start: u32,
        verse_end: u32,
        translation: &str,
    ) -> Result<Vec<Verse>, String> {
        // Fetch the whole chapter and filter
        let ch = self.get_chapter(book_name, chapter, translation).await?;
        Ok(ch
            .verses
            .into_iter()
            .filter(|v| v.verse >= verse_start && v.verse <= verse_end)
            .collect())
    }

    pub async fn search(
        &self,
        query: &str,
        translation: &str,
    ) -> Result<Vec<SearchResult>, String> {
        let trans = Self::translation_code(translation);
        let url = format!("{}/search/{}/", BASE_URL, trans);

        let resp: Vec<BollsSearchResult> = self
            .client
            .get(&url)
            .query(&[("search", query)])
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        Ok(resp
            .into_iter()
            .map(|r| {
                let book_name = books::BOOKS
                    .iter()
                    .find(|b| b.bolls_id == r.book)
                    .map(|b| b.name.to_string())
                    .unwrap_or_else(|| format!("Book {}", r.book));

                SearchResult {
                    book: book_name,
                    chapter: r.chapter,
                    verse: r.verse,
                    text: clean_html(&r.text),
                    translation: translation.to_uppercase(),
                }
            })
            .take(50)
            .collect())
    }

    pub async fn get_random_verse(&self, translation: &str) -> Result<Verse, String> {
        let trans = Self::translation_code(translation);
        let url = format!("{}/get-random-verse/{}/", BASE_URL, trans);

        let resp: BollsVerse = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        Ok(Verse {
            book: "Unknown".to_string(),
            chapter: 0,
            verse: resp.verse,
            text: clean_html(&resp.text),
            translation: translation.to_uppercase(),
        })
    }

    /// Fetch localized book names for a translation.
    /// Returns a Vec of 66 names indexed by (bolls_id - 1).
    pub async fn get_book_names(&self, translation: &str) -> Result<Vec<String>, String> {
        let trans = Self::translation_code(translation);
        let url = format!("{}/get-books/{}/", BASE_URL, trans);

        let resp: Vec<BollsBook> = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        // Build a vec indexed by (bookid - 1), matching our BOOKS order
        let mut names = vec![String::new(); 66];
        for b in resp {
            if b.bookid >= 1 && b.bookid <= 66 {
                names[(b.bookid - 1) as usize] = b.name;
            }
        }
        Ok(names)
    }
}

/// Strip HTML tags and Strong's concordance numbers (<S>1234</S>) from Bolls API responses.
fn clean_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_tag = false;
    let mut in_strongs = false;
    let mut tag_name = String::new();

    for ch in text.chars() {
        if ch == '<' {
            in_tag = true;
            tag_name.clear();
        } else if ch == '>' {
            in_tag = false;
            let tag_upper = tag_name.to_uppercase();
            if tag_upper == "S" {
                in_strongs = true;
            } else if tag_upper == "/S" {
                in_strongs = false;
            }
            tag_name.clear();
        } else if in_tag {
            tag_name.push(ch);
        } else if !in_strongs {
            result.push(ch);
        }
    }

    result.trim().to_string()
}
