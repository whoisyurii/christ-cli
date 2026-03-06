use crate::api::types::{Chapter, SearchResult, Verse};
use crate::data::books::BOOKS;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

fn cache_dir() -> Option<PathBuf> {
    let dirs = ProjectDirs::from("", "", "christ-cli")?;
    Some(dirs.data_dir().join("translations"))
}

fn chapter_path(translation: &str, book_id: u32, chapter: u32) -> Option<PathBuf> {
    let dir = cache_dir()?;
    Some(
        dir.join(translation.to_uppercase())
            .join(format!("{}_{}.json", book_id, chapter)),
    )
}

pub fn save_chapter(translation: &str, book_id: u32, chapter: &Chapter) {
    if let Some(path) = chapter_path(translation, book_id, chapter.chapter) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string(chapter) {
            let _ = fs::write(path, json);
        }
    }
}

pub fn load_chapter(translation: &str, book_id: u32, chapter: u32) -> Option<Chapter> {
    let path = chapter_path(translation, book_id, chapter)?;
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn is_fully_cached(translation: &str) -> bool {
    if translation.eq_ignore_ascii_case("KJV") {
        return true; // Bundled
    }
    let Some(dir) = cache_dir() else {
        return false;
    };
    dir.join(translation.to_uppercase())
        .join(".complete")
        .exists()
}

fn mark_complete(translation: &str) {
    if let Some(dir) = cache_dir() {
        let trans_dir = dir.join(translation.to_uppercase());
        let _ = fs::create_dir_all(&trans_dir);
        let _ = fs::write(trans_dir.join(".complete"), "");
    }
}

/// Remove the .complete marker (cache was found to be incomplete).
pub fn remove_complete_marker(translation: &str) {
    if let Some(dir) = cache_dir() {
        let _ = fs::remove_file(
            dir.join(translation.to_uppercase()).join(".complete"),
        );
    }
}

pub fn save_book_names(translation: &str, names: &[String]) {
    if let Some(dir) = cache_dir() {
        let trans_dir = dir.join(translation.to_uppercase());
        let _ = fs::create_dir_all(&trans_dir);
        if let Ok(json) = serde_json::to_string(names) {
            let _ = fs::write(trans_dir.join("books.json"), json);
        }
    }
}

pub fn load_book_names(translation: &str) -> Option<Vec<String>> {
    let dir = cache_dir()?;
    let data = fs::read_to_string(
        dir.join(translation.to_uppercase()).join("books.json"),
    )
    .ok()?;
    serde_json::from_str(&data).ok()
}

/// Returns true if any cached chapter files exist for this translation.
pub fn has_cached_data(translation: &str) -> bool {
    if translation.eq_ignore_ascii_case("KJV") {
        return true;
    }
    let Some(dir) = cache_dir() else {
        return false;
    };
    let trans_dir = dir.join(translation.to_uppercase());
    trans_dir
        .read_dir()
        .ok()
        .map(|mut entries| entries.any(|e| {
            e.ok()
                .map(|e| e.file_name().to_string_lossy().ends_with(".json"))
                .unwrap_or(false)
        }))
        .unwrap_or(false)
}

/// Search all cached chapters of a translation on disk.
pub fn search(translation: &str, query: &str) -> Vec<SearchResult> {
    let Some(dir) = cache_dir() else {
        return vec![];
    };
    let trans_dir = dir.join(translation.to_uppercase());
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for book in BOOKS {
        for ch_num in 1..=book.chapters {
            let path = trans_dir.join(format!("{}_{}.json", book.bolls_id, ch_num));
            let Ok(data) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(chapter) = serde_json::from_str::<Chapter>(&data) else {
                continue;
            };
            for verse in &chapter.verses {
                if verse.text.to_lowercase().contains(&query_lower) {
                    results.push(SearchResult {
                        book: book.name.to_string(),
                        chapter: ch_num,
                        verse: verse.verse,
                        text: verse.text.clone(),
                        translation: translation.to_uppercase(),
                    });
                    if results.len() >= 50 {
                        return results;
                    }
                }
            }
        }
    }
    results
}

/// Total number of chapters in the Bible (1189).
pub fn total_chapters() -> usize {
    BOOKS.iter().map(|b| b.chapters as usize).sum()
}

pub struct DownloadHandle {
    pub translation: String,
    pub completed: Arc<AtomicUsize>,
    pub total: usize,
    pub cancel: Arc<AtomicBool>,
    pub done: Arc<AtomicBool>,
}

/// Spawn a background task to download and cache all chapters of a translation.
pub fn spawn_download(translation: &str) -> DownloadHandle {
    let total = total_chapters();
    let completed = Arc::new(AtomicUsize::new(0));
    let cancel = Arc::new(AtomicBool::new(false));
    let done = Arc::new(AtomicBool::new(false));

    let trans = translation.to_uppercase();
    let completed_c = completed.clone();
    let cancel_c = cancel.clone();
    let done_c = done.clone();

    tokio::spawn(async move {
        download_translation(&trans, &completed_c, &cancel_c).await;
        done_c.store(true, Ordering::Relaxed);
    });

    DownloadHandle {
        translation: translation.to_uppercase(),
        completed,
        total,
        cancel,
        done,
    }
}

async fn download_translation(
    translation: &str,
    progress: &Arc<AtomicUsize>,
    cancel: &Arc<AtomicBool>,
) {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap();

    // Fetch and cache localized book names first
    if load_book_names(translation).is_none() {
        let url = format!("https://bolls.life/get-books/{}/", translation);
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(books_json) = resp.json::<Vec<serde_json::Value>>().await {
                let mut names = vec![String::new(); 66];
                for b in &books_json {
                    if let (Some(id), Some(name)) = (
                        b.get("bookid").and_then(|v| v.as_u64()),
                        b.get("name").and_then(|v| v.as_str()),
                    ) {
                        if id >= 1 && id <= 66 {
                            names[(id - 1) as usize] = name.to_string();
                        }
                    }
                }
                if names.iter().any(|n| !n.is_empty()) {
                    save_book_names(translation, &names);
                }
            }
        }
    }

    if cancel.load(Ordering::Relaxed) {
        return;
    }

    let sem = Arc::new(tokio::sync::Semaphore::new(5));
    let failures = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    for book in BOOKS {
        for ch_num in 1..=book.chapters {
            let client = client.clone();
            let sem = sem.clone();
            let progress = progress.clone();
            let cancel = cancel.clone();
            let failures = failures.clone();
            let trans = translation.to_string();
            let book_name = book.name.to_string();
            let book_id = book.bolls_id;

            handles.push(tokio::spawn(async move {
                if cancel.load(Ordering::Relaxed) {
                    return;
                }

                // Skip already cached
                if load_chapter(&trans, book_id, ch_num).is_some() {
                    progress.fetch_add(1, Ordering::Relaxed);
                    return;
                }

                let _permit = match sem.acquire().await {
                    Ok(p) => p,
                    Err(_) => return,
                };

                if cancel.load(Ordering::Relaxed) {
                    return;
                }

                let url = format!(
                    "https://bolls.life/get-chapter/{}/{}/{}/",
                    trans, book_id, ch_num
                );

                let mut saved = false;
                if let Ok(resp) = client.get(&url).send().await {
                    if let Ok(verses_raw) = resp.json::<Vec<serde_json::Value>>().await {
                        let verses: Vec<Verse> = verses_raw
                            .iter()
                            .filter_map(|v| {
                                let verse_num = v.get("verse")?.as_u64()? as u32;
                                let text = v.get("text")?.as_str()?;
                                Some(Verse {
                                    book: book_name.clone(),
                                    chapter: ch_num,
                                    verse: verse_num,
                                    text: crate::api::bolls::clean_html(text),
                                    translation: trans.clone(),
                                })
                            })
                            .collect();

                        if !verses.is_empty() {
                            let chapter = Chapter {
                                book: book_name,
                                chapter: ch_num,
                                verses,
                                translation: trans.clone(),
                            };
                            save_chapter(&trans, book_id, &chapter);
                            saved = true;
                        }
                    }
                }

                if !saved {
                    failures.fetch_add(1, Ordering::Relaxed);
                }
                progress.fetch_add(1, Ordering::Relaxed);
            }));
        }
    }

    for handle in handles {
        let _ = handle.await;
    }

    // Only mark complete if all chapters were successfully downloaded
    if !cancel.load(Ordering::Relaxed) && failures.load(Ordering::Relaxed) == 0 {
        mark_complete(translation);
    }
}
