use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use crate::fs_engine::crawl_directory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub hidden: bool,
    pub score: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerStatus {
    pub is_running: bool,
    pub indexed_count: usize,
    pub current_path: Option<String>,
    pub elapsed_ms: u64,
}

pub struct Indexer {
    db_path: PathBuf,
    status: Arc<Mutex<IndexerStatus>>,
}

impl Indexer {
    pub fn new(data_dir: &str) -> SqliteResult<Self> {
        let db_path = PathBuf::from(data_dir).join("hardbore_index.db");
        let indexer = Self {
            db_path,
            status: Arc::new(Mutex::new(IndexerStatus {
                is_running: false,
                indexed_count: 0,
                current_path: None,
                elapsed_ms: 0,
            })),
        };

        indexer.init_db()?;
        indexer.restore_status();

        Ok(indexer)
    }

    fn get_connection(&self) -> SqliteResult<Connection> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = -64000;
            PRAGMA temp_store = MEMORY;
            PRAGMA mmap_size = 268435456;
            ",
        )?;
        Ok(conn)
    }

    fn init_db(&self) -> SqliteResult<()> {
        let conn = self.get_connection()?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                is_dir INTEGER NOT NULL,
                hidden INTEGER NOT NULL DEFAULT 0,
                parent_path TEXT,
                extension TEXT,
                size INTEGER,
                modified INTEGER
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
                name,
                path,
                content='files',
                content_rowid='id',
                tokenize='trigram'
            );

            CREATE TRIGGER IF NOT EXISTS files_ai AFTER INSERT ON files BEGIN
                INSERT INTO files_fts(rowid, name, path) VALUES (new.id, new.name, new.path);
            END;

            CREATE TRIGGER IF NOT EXISTS files_ad AFTER DELETE ON files BEGIN
                INSERT INTO files_fts(files_fts, rowid, name, path) VALUES('delete', old.id, old.name, old.path);
            END;

            CREATE TRIGGER IF NOT EXISTS files_au AFTER UPDATE ON files BEGIN
                INSERT INTO files_fts(files_fts, rowid, name, path) VALUES('delete', old.id, old.name, old.path);
                INSERT INTO files_fts(rowid, name, path) VALUES (new.id, new.name, new.path);
            END;

            CREATE INDEX IF NOT EXISTS idx_files_parent ON files(parent_path);
            CREATE INDEX IF NOT EXISTS idx_files_is_dir ON files(is_dir);
            CREATE INDEX IF NOT EXISTS idx_files_extension ON files(extension);
            ",
        )?;

        let _ = conn.execute_batch(
            "
            ALTER TABLE files ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0;
            ",
        );

        Ok(())
    }

    pub fn index_directory(&self, root: &str, max_depth: Option<usize>) {
        let root = root.to_string();
        let db_path = self.db_path.clone();
        let status = self.status.clone();

        thread::spawn(move || {
            let start = Instant::now();

            {
                let mut s = status.lock().unwrap();
                s.is_running = true;
                s.current_path = Some(root.clone());
                s.indexed_count = 0;
            }

            let entries = crawl_directory(&root, max_depth);

            if let Ok(conn) = Connection::open(&db_path) {
                let _ = conn.execute_batch(
                    "PRAGMA synchronous = OFF;
                     PRAGMA journal_mode = MEMORY;
                     PRAGMA temp_store = MEMORY;"
                );

                let mut stmt = conn.prepare(
                    "INSERT OR REPLACE INTO files (path, name, is_dir, hidden, parent_path, extension, size, modified)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
                ).ok();

                if let Some(ref mut prepared_stmt) = stmt {
                    const BATCH_SIZE: usize = 10000;
                    let _ = conn.execute_batch("BEGIN TRANSACTION;");

                    for (i, entry) in entries.iter().enumerate() {
                        let parent = PathBuf::from(&entry.path)
                            .parent()
                            .map(|p| p.to_string_lossy().to_string());

                        let _ = prepared_stmt.execute((
                            &entry.path,
                            &entry.name,
                            entry.is_dir as i32,
                            entry.hidden as i32,
                            &parent,
                            &entry.extension,
                            entry.size as i64,
                            entry.modified,
                        ));

                        if (i + 1) % BATCH_SIZE == 0 {
                            let _ = conn.execute_batch("COMMIT; BEGIN TRANSACTION;");
                            
                            let mut s = status.lock().unwrap();
                            s.indexed_count = i + 1;
                            s.elapsed_ms = start.elapsed().as_millis() as u64;
                        }
                    }

                    let _ = conn.execute_batch("COMMIT;");
                }

                let _ = conn.execute_batch(
                    "PRAGMA synchronous = NORMAL;
                     PRAGMA journal_mode = WAL;"
                );
            }

            {
                let mut s = status.lock().unwrap();
                s.is_running = false;
                s.indexed_count = entries.len();
                s.elapsed_ms = start.elapsed().as_millis() as u64;
                s.current_path = None;
            }
        });
    }

    pub fn search_fts(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let conn = match self.get_connection() {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let escaped_query = query
            .chars()
            .map(|c| match c {
                '"' | '*' | '+' | '-' | '(' | ')' | ':' | '^' => ' ',
                _ => c,
            })
            .collect::<String>();

        let fts_query = format!("\"{}\"*", escaped_query);

        let mut stmt = match conn.prepare(
            "SELECT f.name, f.path, f.is_dir, f.hidden
             FROM files_fts fts
             JOIN files f ON fts.rowid = f.id
             WHERE files_fts MATCH ?1
             ORDER BY bm25(files_fts) 
             LIMIT ?2",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };

        let results: Vec<SearchResult> = stmt
            .query_map([&fts_query, &limit.to_string()], |row| {
                Ok(SearchResult {
                    name: row.get(0)?,
                    path: row.get(1)?,
                    is_dir: row.get::<_, i32>(2)? != 0,
                    hidden: row.get::<_, i32>(3)? != 0,
                    score: 0,
                })
            })
            .ok()
            .map(|iter| iter.filter_map(|r| r.ok()).collect())
            .unwrap_or_default();

        results
    }

    pub fn search_fuzzy(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let conn = match self.get_connection() {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let mut stmt = match conn.prepare(
            "SELECT name, path, is_dir, hidden FROM files 
             WHERE name LIKE ?1 OR path LIKE ?1
             LIMIT 5000",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };

        let pattern = format!("%{}%", query);
        let candidates: Vec<(String, String, bool, bool)> = stmt
            .query_map([&pattern], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i32>(2)? != 0,
                    row.get::<_, i32>(3)? != 0,
                ))
            })
            .ok()
            .map(|iter| iter.filter_map(|r| r.ok()).collect())
            .unwrap_or_default();

        let matcher = SkimMatcherV2::default();
        let mut results: Vec<SearchResult> = candidates
            .into_iter()
            .filter_map(|(name, path, is_dir, hidden)| {
                let score = matcher.fuzzy_match(&name, query)
                    .or_else(|| matcher.fuzzy_match(&path, query))?;
                Some(SearchResult {
                    name,
                    path,
                    is_dir,
                    hidden,
                    score,
                })
            })
            .collect();

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(limit);
        results
    }

    pub fn get_status(&self) -> IndexerStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn clear_index(&self) -> SqliteResult<()> {
        let conn = self.get_connection()?;
        conn.execute_batch(
            "
            DELETE FROM files;
            DELETE FROM files_fts;
            VACUUM;
            ",
        )?;
        Ok(())
    }

    pub fn get_indexed_count(&self) -> usize {
        let conn = match self.get_connection() {
            Ok(c) => c,
            Err(_) => return 0,
        };

        conn.query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
            .unwrap_or(0)
    }

    fn restore_status(&self) {
        let count = self.get_indexed_count();
        if count > 0 {
            let mut status = self.status.lock().unwrap();
            status.indexed_count = count;
        }
    }
}
