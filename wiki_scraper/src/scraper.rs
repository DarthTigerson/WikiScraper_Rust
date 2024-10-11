use reqwest::Client;
use scraper::{Html, Selector};
use anyhow::Result;
use flate2::{write::GzEncoder, Compression};
use std::fs::File;
use std::io::Write;
use crate::database::add_new_urls;

pub async fn scrape_raw(url: &str) -> Result<String> {
    let client = Client::new();
    let full_url = format!("https://en.wikipedia.org{}", url);
    let res = client.get(&full_url).send().await?;
    if !res.status().is_success() {
        return Err(anyhow::anyhow!("Failed to load page: {}", res.status()));
    }
    let text = res.text().await?;
    let cleaned_data = clean_data(&text)?;
    Ok(cleaned_data)
}

pub fn clean_data(raw_html: &str) -> Result<String> {
    let document = Html::parse_document(raw_html);
    let content_selector = Selector::parse("div.mw-content-container").unwrap();

    let content = document
        .select(&content_selector)
        .next()
        .ok_or_else(|| anyhow::anyhow!("Could not find the main content container."))?;

    Ok(content.inner_html())
}

pub fn extract_links(html_content: &str) -> Result<usize> {
    let document = Html::parse_document(html_content);
    let link_selector = Selector::parse("a[href^=\"/wiki/\"]").unwrap();

    let mut new_urls = Vec::new();

    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            if !href.starts_with("/wiki/File:")
                && !href.starts_with("/wiki/Wikipedia:")
                && !href.starts_with("/wiki/Special:")
            {
                let title = element.text().collect::<Vec<_>>().join(" ");
                let clean_title = clean_title(&title);
                new_urls.push((href.to_string(), clean_title));
            }
        }
    }

    let added_count = add_new_urls(new_urls)?;
    Ok(added_count)
}

fn clean_title(text: &str) -> String {
    let cleaned_text = text.trim();
    if cleaned_text.chars().any(|c| c.is_alphabetic()) {
        cleaned_text.to_string()
    } else {
        "No Valid Title".to_string()
    }
}

pub fn save_to_gzip(content: &str, file_path: &str) -> Result<()> {
    let file = File::create(file_path)?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(content.as_bytes())?;
    encoder.finish()?;
    Ok(())
}