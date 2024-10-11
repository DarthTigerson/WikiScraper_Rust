// src/main.rs

mod database;
mod scraper;
mod utils;

use database::{
    create_tables,
    get_next_url_to_scrape,
    update_scraped_date,
    update_scraped_date_failed,
    seed_urls,
};
use scraper::{extract_links, save_to_gzip, scrape_raw};
use utils::file_path_cleaner;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the database tables
    create_tables()?;

    // Seed the database with initial URLs if necessary
    seed_urls(vec![
        ("/wiki/Python_(programming_language)".to_string(), "Python (programming language)".to_string()),
    ])?;

    let output_dir = "data/";
    if !std::path::Path::new(output_dir).exists() {
        std::fs::create_dir_all(output_dir)?;
    }

    // Define the number of concurrent workers
    let num_workers = 5;
    let semaphore = Arc::new(Semaphore::new(num_workers));

    // Create a vector to hold the handles of the spawned tasks
    let mut handles = Vec::new();

    for _ in 0..num_workers {
        let semaphore = semaphore.clone();
        let output_dir = output_dir.to_string();

        let handle = task::spawn(async move {
            loop {
                // Acquire a permit before proceeding
                let _permit = semaphore.acquire().await;

                // Fetch the next URL to scrape
                match get_next_url_to_scrape()? {
                    Some((url, title)) => {
                        // Scrape the raw HTML from the URL
                        let site_data = match scrape_raw(&url).await {
                            Ok(data) => data,
                            Err(e) => {
                                eprintln!("Error scraping URL {}: {}", url, e);
                                // Update the database to reset status
                                update_scraped_date_failed(&url)?;
                                continue;
                            }
                        };

                        // Save the HTML data to a compressed .gz file
                        let file_name = file_path_cleaner(&title);
                        let file_path = format!("{}{}.html.gz", output_dir, file_name);

                        if let Err(e) = save_to_gzip(&site_data, &file_path) {
                            eprintln!("Error saving file {}: {}", file_path, e);
                        }

                        // Extract and add new links to the database
                        if let Ok(added_count) = extract_links(&site_data) {
                            println!("{} new URLs added to the database.", added_count);
                        }

                        // Update the database to mark the URL as scraped
                        if let Err(e) = update_scraped_date(&url) {
                            eprintln!("Error updating scraped date: {}", e);
                        }
                    }
                    None => {
                        // No more URLs to scrape
                        break;
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await??;
    }

    println!("All URLs have been scraped.");
    Ok(())
}