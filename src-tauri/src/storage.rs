use crate::{
    book_import,
    chapters::{self, Chapter},
    geometry::Placement,
    reading_settings::ReadingSettings,
};
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    pub id: String,
    pub title: String,
    pub encoding: String,
    pub utf16_length: usize,
    pub anchor: usize,
    pub added_at: u64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub books: Vec<Book>,
    pub current_book_id: Option<String>,
    pub settings: ReadingSettings,
}

pub struct Store {
    connection: Connection,
    directory: PathBuf,
    pending_progress: Option<(String, usize)>,
    saved_progress: Option<(String, usize)>,
    last_saved: Instant,
    saved_placement: Option<Placement>,
    pending_placement: Option<Placement>,
}
fn db_error(error: rusqlite::Error) -> String {
    format!("本地数据库操作失败：{error}")
}
fn book_row(row: &Row<'_>) -> rusqlite::Result<Book> {
    Ok(Book {
        id: row.get(0)?,
        title: row.get(1)?,
        encoding: row.get(2)?,
        utf16_length: row.get::<_, i64>(3)? as usize,
        anchor: row.get::<_, i64>(4)? as usize,
        added_at: row.get::<_, i64>(5)? as u64,
    })
}

impl Store {
    pub fn open(directory: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
        let mut connection =
            Connection::open(directory.join("floatleaf.sqlite3")).map_err(db_error)?;
        connection
            .busy_timeout(Duration::from_secs(3))
            .map_err(db_error)?;
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON; PRAGMA journal_mode = DELETE;
            CREATE TABLE IF NOT EXISTS books (
                id TEXT PRIMARY KEY, title TEXT NOT NULL, encoding TEXT NOT NULL,
                content TEXT NOT NULL, utf16_length INTEGER NOT NULL,
                anchor INTEGER NOT NULL DEFAULT 0, added_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS app_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                current_book_id TEXT REFERENCES books(id) ON DELETE SET NULL,
                settings TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS window_layout (id INTEGER PRIMARY KEY CHECK (id = 1), monitor TEXT, x REAL NOT NULL, y REAL NOT NULL);
            CREATE TABLE IF NOT EXISTS chapters (
                book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
                position INTEGER NOT NULL, title TEXT NOT NULL,
                anchor INTEGER NOT NULL, level INTEGER NOT NULL,
                PRIMARY KEY (book_id, position)
            );",
            )
            .map_err(db_error)?;
        let version: u32 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(db_error)?;
        if version < 2 {
            let transaction = connection.transaction().map_err(db_error)?;
            {
                let mut books = transaction
                    .prepare("SELECT id, content FROM books")
                    .map_err(db_error)?;
                let rows = books
                    .query_map([], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    })
                    .map_err(db_error)?;
                for book in rows {
                    let (id, text) = book.map_err(db_error)?;
                    write_chapters(&transaction, &id, &chapters::detect(&text))?;
                }
            }
            transaction
                .pragma_update(None, "user_version", 2)
                .map_err(db_error)?;
            transaction.commit().map_err(db_error)?;
        }
        let settings =
            serde_json::to_string(&ReadingSettings::default()).map_err(|e| e.to_string())?;
        connection
            .execute(
                "INSERT OR IGNORE INTO app_state (id, settings) VALUES (1, ?1)",
                [settings],
            )
            .map_err(db_error)?;
        let saved_placement = connection
            .query_row(
                "SELECT monitor, x, y FROM window_layout WHERE id = 1",
                [],
                |r| {
                    Ok(Placement {
                        monitor: r.get(0)?,
                        x: r.get(1)?,
                        y: r.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(db_error)?;
        Ok(Self {
            connection,
            directory,
            pending_progress: None,
            saved_progress: None,
            last_saved: Instant::now(),
            saved_placement,
            pending_placement: None,
        })
    }
    pub fn placement(&self) -> Option<Placement> {
        self.saved_placement.clone()
    }
    pub fn update_placement(&mut self, placement: Option<Placement>) {
        self.pending_placement = placement.filter(|p| Some(p) != self.saved_placement.as_ref());
    }
    pub fn directory(&self) -> &Path {
        &self.directory
    }
    pub fn library(&self) -> Result<Library, String> {
        let mut statement = self.connection.prepare("SELECT id, title, encoding, utf16_length, anchor, added_at FROM books ORDER BY added_at, rowid").map_err(db_error)?;
        let mut books = statement
            .query_map([], book_row)
            .map_err(db_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(db_error)?;
        if let Some((id, anchor)) = &self.pending_progress
            && let Some(book) = books.iter_mut().find(|book| &book.id == id)
        {
            book.anchor = *anchor;
        }
        let (current_book_id, settings): (Option<String>, String) = self
            .connection
            .query_row(
                "SELECT current_book_id, settings FROM app_state WHERE id = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(db_error)?;
        Ok(Library {
            books,
            current_book_id,
            settings: serde_json::from_str(&settings).map_err(|e| e.to_string())?,
        })
    }
    pub fn book(&self, id: &str) -> Result<Book, String> {
        self.connection.query_row("SELECT id, title, encoding, utf16_length, anchor, added_at FROM books WHERE id = ?1", [id], book_row).map_err(db_error)
    }
    pub fn import(&mut self, path: &Path, encoding: &str) -> Result<Book, String> {
        self.flush()?;
        let imported = book_import::read(path, encoding)?;
        let id = format!("{:x}", Sha256::digest(imported.text.as_bytes()));
        let added_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let transaction = self.connection.transaction().map_err(db_error)?;
        let inserted = transaction.execute("INSERT INTO books (id, title, encoding, content, utf16_length, added_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(id) DO NOTHING",
            params![id, imported.title, imported.encoding, imported.text, imported.text.encode_utf16().count() as i64, added_at as i64]).map_err(db_error)?;
        if inserted > 0 {
            write_chapters(&transaction, &id, &imported.chapters)?;
        }
        transaction
            .execute(
                "UPDATE app_state SET current_book_id = ?1 WHERE id = 1",
                [&id],
            )
            .map_err(db_error)?;
        transaction.commit().map_err(db_error)?;
        self.book(&id)
    }
    pub fn chapters(&self, id: &str) -> Result<Vec<Chapter>, String> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT title, anchor, level FROM chapters WHERE book_id = ?1 ORDER BY position",
            )
            .map_err(db_error)?;
        statement
            .query_map([id], |row| {
                Ok(Chapter {
                    title: row.get(0)?,
                    anchor: row.get::<_, i64>(1)? as usize,
                    level: row.get::<_, i64>(2)? as usize,
                })
            })
            .map_err(db_error)?
            .collect::<rusqlite::Result<_>>()
            .map_err(db_error)
    }
    pub fn content(&self, id: &str) -> Result<String, String> {
        self.connection
            .query_row("SELECT content FROM books WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .map_err(db_error)
    }
    pub fn select(&mut self, id: &str) -> Result<String, String> {
        self.flush()?;
        let text = self.content(id)?;
        self.connection
            .execute(
                "UPDATE app_state SET current_book_id = ?1 WHERE id = 1",
                [id],
            )
            .map_err(db_error)?;
        Ok(text)
    }
    pub fn set_settings(&mut self, settings: ReadingSettings) -> Result<(), String> {
        let json = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
        self.connection
            .execute("UPDATE app_state SET settings = ?1 WHERE id = 1", [json])
            .map_err(db_error)?;
        Ok(())
    }
    /// Coalesce rapid page turns; ordinary exit and book switching flush immediately.
    pub fn progress(&mut self, id: &str, anchor: usize) {
        let next = (id.to_owned(), anchor);
        self.pending_progress = (self.saved_progress.as_ref() != Some(&next)).then_some(next);
    }
    pub fn flush(&mut self) -> Result<(), String> {
        if let Some(p) = &self.pending_placement {
            self.connection.execute("INSERT INTO window_layout VALUES (1, ?1, ?2, ?3) ON CONFLICT(id) DO UPDATE SET monitor=excluded.monitor, x=excluded.x, y=excluded.y", params![p.monitor, p.x, p.y]).map_err(db_error)?;
            self.saved_placement = self.pending_placement.take();
            self.last_saved = Instant::now();
        }
        if let Some((id, anchor)) = &self.pending_progress {
            self.connection
                .execute(
                    "UPDATE books SET anchor = ?1 WHERE id = ?2",
                    params![*anchor as i64, id],
                )
                .map_err(db_error)?;
            self.saved_progress = self.pending_progress.take();
            self.last_saved = Instant::now();
        }
        Ok(())
    }
    pub fn flush_due(&mut self) -> Result<(), String> {
        if self.last_saved.elapsed() >= Duration::from_secs(1) {
            self.flush()?;
        }
        Ok(())
    }
    pub fn remove(&mut self, id: &str) -> Result<(), String> {
        self.flush()?;
        // Body and progress belong to this row; foreign key clears current book.
        self.connection
            .execute("DELETE FROM books WHERE id = ?1", [id])
            .map_err(db_error)?;
        Ok(())
    }
}

fn write_chapters(connection: &Connection, id: &str, chapters: &[Chapter]) -> Result<(), String> {
    let mut insert = connection.prepare("INSERT INTO chapters (book_id, position, title, anchor, level) VALUES (?1, ?2, ?3, ?4, ?5)").map_err(db_error)?;
    for (index, chapter) in chapters.iter().enumerate() {
        insert
            .execute(params![
                id,
                index as i64,
                chapter.title,
                chapter.anchor as i64,
                chapter.level as i64
            ])
            .map_err(db_error)?;
    }
    Ok(())
}

/// JS and Rust exchange UTF-16 code-unit offsets, never UTF-8 byte offsets.
pub fn utf16_boundary(text: &str, offset: usize) -> bool {
    let mut units = 0;
    for c in text.chars() {
        if units == offset {
            return true;
        }
        units += c.len_utf16();
        if units > offset {
            return false;
        }
    }
    units == offset
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sqlite_keeps_text_and_per_book_progress_after_source_removal_and_restart() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("book.txt");
        fs::write(&path, "第一本 😀 e\u{301}\n后续内容").unwrap();
        let mut store = Store::open(temp.path().join("data")).unwrap();
        let first = store.import(&path, "auto").unwrap();
        let text = store.content(&first.id).unwrap();
        store.progress(&first.id, 6);
        store.flush().unwrap();
        assert_eq!(store.import(&path, "auto").unwrap().anchor, 6);
        fs::write(&path, "第二本内容").unwrap();
        let second = store.import(&path, "auto").unwrap();
        store.progress(&second.id, 2);
        store.flush().unwrap();
        fs::remove_file(&path).unwrap();
        drop(store);
        let store = Store::open(temp.path().join("data")).unwrap();
        let library = store.library().unwrap();
        assert_eq!(store.content(&first.id).unwrap(), text);
        assert_eq!(library.books.len(), 2);
        assert_eq!(store.book(&first.id).unwrap().anchor, 6);
        assert_eq!(store.book(&second.id).unwrap().anchor, 2);
        assert_eq!(library.current_book_id, Some(second.id));
        assert!(!temp.path().join("data/books").exists()); // Novels are embedded in SQLite.
    }
    #[test]
    fn settings_and_removal_are_persistent_without_touching_source() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("book.txt");
        fs::write(&source, "原始内容").unwrap();
        let mut store = Store::open(temp.path().join("data")).unwrap();
        let book = store.import(&source, "auto").unwrap();
        let settings = ReadingSettings {
            font_size: 36.0,
            lines: 12,
            interval_seconds: 17,
            capture_protection: true,
            ..ReadingSettings::default()
        };
        store.set_settings(settings.clone()).unwrap();
        store.remove(&book.id).unwrap();
        drop(store);
        let store = Store::open(temp.path().join("data")).unwrap();
        let library = store.library().unwrap();
        assert_eq!(library.settings, settings);
        assert!(library.books.is_empty());
        assert!(library.current_book_id.is_none());
        assert!(source.exists());
    }
    #[test]
    fn rapid_progress_is_coalesced_and_flush_persists_latest_position() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("book.txt");
        fs::write(&source, "0123456789").unwrap();
        let mut store = Store::open(temp.path().join("data")).unwrap();
        let book = store.import(&source, "auto").unwrap();
        for anchor in 1..8 {
            store.progress(&book.id, anchor);
        }
        assert_eq!(store.book(&book.id).unwrap().anchor, 0);
        assert_eq!(store.library().unwrap().books[0].anchor, 7);
        store.flush().unwrap();
        assert_eq!(store.book(&book.id).unwrap().anchor, 7);
        store.progress(&book.id, 7);
        assert!(store.pending_progress.is_none());
        store.progress(&book.id, 8);
        store.progress(&book.id, 7);
        store.flush().unwrap();
        assert_eq!(store.book(&book.id).unwrap().anchor, 7);
    }
    #[test]
    fn sqlite_transaction_rolls_back_partial_book_insertion() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = Store::open(temp.path().into()).unwrap();
        {
            let transaction = store.connection.transaction().unwrap();
            transaction
                .execute(
                    "INSERT INTO books VALUES ('partial','title','UTF-8','text',4,0,0)",
                    [],
                )
                .unwrap();
            // Dropping an uncommitted transaction rolls back; no orphan body/index pair.
        }
        assert!(store.library().unwrap().books.is_empty());
    }
    #[test]
    fn placement_and_new_settings_defaults_survive_restart() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = Store::open(temp.path().into()).unwrap();
        let placement = Placement {
            monitor: Some("external".into()),
            x: 128.0,
            y: 64.5,
        };
        store.update_placement(Some(placement.clone()));
        store.flush().unwrap();
        // Existing P1 databases have no shortcuts field in the settings JSON.
        store
            .connection
            .execute("UPDATE app_state SET settings = '{\"fontSize\":24.0}'", [])
            .unwrap();
        drop(store);
        let store = Store::open(temp.path().into()).unwrap();
        assert_eq!(store.placement(), Some(placement));
        assert_eq!(
            store.library().unwrap().settings.shortcuts,
            Default::default()
        );
        assert_eq!(store.library().unwrap().settings.font_size, 24.0);
        assert!(!store.library().unwrap().settings.capture_protection);
    }
    #[test]
    fn surrogate_boundaries_and_end_are_validated() {
        let text = "a😀中e\u{301}";
        for valid in [0, 1, 3, 4, 5, 6] {
            assert!(utf16_boundary(text, valid));
        }
        assert!(!utf16_boundary(text, 2));
        assert!(!utf16_boundary(text, 7));
    }
    #[test]
    fn existing_txt_library_gets_chapters_without_resetting_progress_or_settings() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("legacy.txt");
        fs::write(&source, "第一章 出发\n正文😀\n第二章 归来\n结束").unwrap();
        let mut store = Store::open(temp.path().join("data")).unwrap();
        let book = store.import(&source, "auto").unwrap();
        store.progress(&book.id, 7);
        store.flush().unwrap();
        store
            .set_settings(ReadingSettings {
                interval_seconds: 17,
                ..Default::default()
            })
            .unwrap();
        store
            .connection
            .execute_batch("DROP TABLE chapters; PRAGMA user_version = 1;")
            .unwrap();
        drop(store);
        fs::remove_file(&source).unwrap();
        for _ in 0..2 {
            let store = Store::open(temp.path().join("data")).unwrap();
            assert_eq!(store.chapters(&book.id).unwrap().len(), 2);
            assert_eq!(store.book(&book.id).unwrap().anchor, 7);
            assert_eq!(
                store.library().unwrap().current_book_id,
                Some(book.id.clone())
            );
            assert_eq!(store.library().unwrap().settings.interval_seconds, 17);
        }
    }
    #[test]
    fn epub_chapters_survive_duplicate_import_source_removal_and_restart() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("fixture.epub");
        crate::epub_import::tests::fixture(&source, false);
        let mut store = Store::open(temp.path().join("data")).unwrap();
        let book = store.import(&source, "auto").unwrap();
        let chapters = store.chapters(&book.id).unwrap();
        assert_eq!(book.encoding, "EPUB");
        store.progress(&book.id, chapters[1].anchor);
        store.flush().unwrap();
        assert_eq!(
            store.import(&source, "auto").unwrap().anchor,
            chapters[1].anchor
        );
        assert_eq!(store.chapters(&book.id).unwrap(), chapters);
        drop(store);
        fs::remove_file(source).unwrap();
        let mut store = Store::open(temp.path().join("data")).unwrap();
        assert_eq!(store.chapters(&book.id).unwrap(), chapters);
        assert_eq!(store.book(&book.id).unwrap().anchor, chapters[1].anchor);
        store.remove(&book.id).unwrap();
        assert!(store.chapters(&book.id).unwrap().is_empty());
    }
}
