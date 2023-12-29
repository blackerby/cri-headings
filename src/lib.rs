use anyhow::{bail, Result};
use chrono::Datelike;
use clap::Parser;
use indicatif::ProgressBar;
use reqwest::{self, Response, StatusCode};
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufWriter, Write},
};

const BASE_URL: &str = "https://api.govinfo.gov/packages/CRI-";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Page {
    count: u16,
    page_size: u16,
    next_page: Option<String>,
    granules: Vec<Granule>,
}

#[derive(Deserialize)]
struct Granule {
    title: String,
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Get Congressional Record Index headings from the GovInfo API
pub struct Args {
    /// CRI years to download. Default to current year.
    #[arg(default_values_t = [current_year()])]
    years: Vec<String>,

    /// API offset
    #[arg(default_value = "0")]
    offset: String,

    /// API page size
    #[arg(default_value = "1000")]
    page_size: String,

    /// GovInfo API Key
    #[arg(default_value = "DEMO_KEY", last(true))]
    api_key: String,
}

#[tokio::main]
pub async fn run(args: Args) -> Result<()> {
    for year in &args.years {
        let url = build_url(&year, &args);
        let response = reqwest::get(url).await?;

        if is_rate_limited(&response) {
            bail!("Status Code 429: Too many requests. Wait one hour.")
        }

        let remaining_requests = remaining_requests(&response)?;
        let page = response.json::<Page>().await?;

        if remaining_requests < requests_to_make(&page) {
            bail!("Not enough requests remaining to complete task. Wait one hour.")
        }

        let granules = page.granules;

        let output_filename = format!("CRI-{}_headings.txt", &year);
        let output_file = File::create(output_filename)?;
        let mut buf = BufWriter::new(output_file);
        let bar = ProgressBar::new(page.count as u64);

        for granule in granules {
            writeln!(buf, "{}", granule.title)?;
            bar.inc(1);
        }

        let mut next_page = page.next_page;
        while let Some(base_url) = next_page {
            let next_url = format!("{}&api_key={}", base_url, args.api_key);
            let page = reqwest::get(next_url).await?.json::<Page>().await?;
            let granules = page.granules;
            for granule in granules {
                writeln!(buf, "{}", granule.title)?;
                bar.inc(1);
            }
            next_page = page.next_page;
        }
        bar.finish();
    }
    Ok(())
}

fn build_url(year: &String, args: &Args) -> String {
    format!(
        "{}{}/granules?offset={}&pageSize={}&api_key={}",
        BASE_URL, year, args.offset, args.page_size, args.api_key
    )
}

fn current_year() -> String {
    format!("{}", chrono::Utc::now().year())
}

fn is_rate_limited(response: &Response) -> bool {
    response.status() == StatusCode::TOO_MANY_REQUESTS
}

fn remaining_requests(response: &Response) -> Result<u16> {
    Ok(response
        .headers()
        .get("x-ratelimit-remaining")
        .expect("No matching header found")
        .to_str()?
        .parse::<u16>()?)
}

fn requests_to_make(page: &Page) -> u16 {
    let quotient = page.count / page.page_size;
    match page.count % page.page_size {
        0 => quotient,
        _ => quotient + 1,
    }
}

#[cfg(test)]
mod test {
    use crate::{requests_to_make, Page};

    #[test]
    fn test_requests_to_make_1000() {
        let page = Page {
            count: 14853,
            page_size: 1000,
            next_page: Some(String::new()),
            granules: Vec::new(),
        };
        let expected = 15;
        let result = requests_to_make(&page);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_requests_to_make_100() {
        let page = Page {
            count: 14853,
            page_size: 100,
            next_page: Some(String::new()),
            granules: Vec::new(),
        };
        let expected = 149;
        let result = requests_to_make(&page);
        assert_eq!(expected, result);
    }

    #[test]
    fn test_requests_to_make_10() {
        let page = Page {
            count: 14853,
            page_size: 10,
            next_page: Some(String::new()),
            granules: Vec::new(),
        };
        let expected = 1486;
        let result = requests_to_make(&page);
        assert_eq!(expected, result);
    }
}
