use rusqlite::{params, Connection, Result};
use chrono::Utc;

pub fn create_tables() -> Result<()> {
    let conn = Connection::open("urls.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS urls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT UNIQUE,
            title TEXT,
            date_added TEXT,
            date_scraped TEXT
        )",
        [],
    )?;
    Ok(())
}

pub fn get_next_url_to_scrape() -> Result<Option<(String, String)>> {
    let mut conn = Connection::open("urls.db")?;
    let tx = conn.transaction()?;

    let mut result = None;

    {
        let mut stmt = tx.prepare(
            "SELECT url, title FROM urls WHERE date_scraped IS NULL LIMIT 1",
        )?;

        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            let url: String = row.get(0)?;
            let title: String = row.get(1)?;

            tx.execute(
                "UPDATE urls SET date_scraped = 'IN_PROGRESS' WHERE url = ?1",
                params![url],
            )?;

            result = Some((url, title));
        }
    }

    tx.commit()?;
    Ok(result)
}

pub fn update_scraped_date(url: &str) -> Result<()> {
    let conn = Connection::open("urls.db")?;
    let current_time = Utc::now().naive_utc().to_string();
    conn.execute(
        "UPDATE urls SET date_scraped = ?1 WHERE url = ?2",
        params![current_time, url],
    )?;
    Ok(())
}

pub fn update_scraped_date_failed(url: &str) -> Result<()> {
    let conn = Connection::open("urls.db")?;
    conn.execute(
        "UPDATE urls SET date_scraped = NULL WHERE url = ?1 AND date_scraped = 'IN_PROGRESS'",
        params![url],
    )?;
    Ok(())
}

pub fn add_new_urls(urls: Vec<(String, String)>) -> Result<usize> {
    let mut conn = Connection::open("urls.db")?;
    let tx = conn.transaction()?;
    let mut added_count = 0;
    let date_added = Utc::now().naive_utc().to_string();

    {
        let mut stmt = tx.prepare(
            "INSERT OR IGNORE INTO urls (url, title, date_added) VALUES (?1, ?2, ?3)",
        )?;

        for (url, title) in urls {
            let result = stmt.execute(params![url, title, date_added])?;
            if result > 0 {
                added_count += 1;
            }
        }
    }

    tx.commit()?;
    Ok(added_count)
}

pub fn seed_urls(urls: Vec<(String, String)>) -> Result<()> {
    add_new_urls(urls)?;
    Ok(())
}