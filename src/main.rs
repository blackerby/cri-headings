use anyhow::{bail, Result};
use reqwest;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::{BufWriter, Write};

const BASE_URL: &str = "https://api.govinfo.gov/packages/CRI-";
const API_KEY: &str = "psDwdYpzXkhTHjJEGkubYyHy7e5FRgF88E64TrFC";

#[tokio::main]
async fn main() -> Result<()> {
    // parse arguments
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <year> and <format>"),
        2 => bail!("Missing <format>"),
        _ => {}
    }

    let year = &args[1];
    let format = &args[2];

    let file_name = format!("CRI-{}.txt", year);
    let file = File::create(file_name)?;
    let mut buf = BufWriter::new(file);

    let url = format!("{}{}/{}?api_key={}", BASE_URL, year, format, API_KEY);
    println!("{}", url);

    let response = reqwest::get(url).await?;

    let text = response.text().await?;

    let document = Html::parse_document(&text);
    let heading_selector = parse_selector("p.heading")?;
    let headings = document
        .select(&heading_selector)
        .map(|heading| heading.inner_html());

    for heading in headings {
        writeln!(buf, "{}", heading)?;
    }

    Ok(())
}

fn parse_selector(selector_str: &str) -> Result<Selector, anyhow::Error> {
    match Selector::parse(selector_str) {
        Ok(selector) => Ok(selector),
        Err(_) => bail!("Malformed selector string"),
    }
}
