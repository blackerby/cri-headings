use anyhow::{bail, Result};
use chrono::Datelike;
use clap::Parser;
use futures::future::join_all;
use reqwest::{self, Response, StatusCode};
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufWriter, Write},
};
use tokio::task::JoinHandle;

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
    let years = &args.years;
    let api_key = &args.api_key;
    let offset = &args.offset;
    let page_size = &args.page_size;
    let urls = years
        .iter()
        .map(|year| build_url(year, offset, page_size, api_key))
        .collect::<Vec<(String, String, String)>>();
    let mut tasks: Vec<JoinHandle<Result<()>>> = Vec::new();

    for (api_key, year, url) in urls {
        let url = url.clone();
        tasks.push(tokio::spawn(async move {
            println!("Getting {} headings", year);
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

            let output_filename = format!("CRI-{}_headings.txt", year);
            let output_file = File::create(output_filename)?;
            let mut buf = BufWriter::new(output_file);

            for granule in granules {
                writeln!(buf, "{}", granule.title)?;
            }

            let mut next_page = page.next_page;
            while let Some(base_url) = next_page {
                let next_url = format!("{}&api_key={}", base_url, api_key);
                let page = reqwest::get(next_url).await?.json::<Page>().await?;
                let granules = page.granules;
                for granule in granules {
                    writeln!(buf, "{}", granule.title)?;
                }
                next_page = page.next_page;
            }
            println!("Wrote {} headings", year);
            Ok(())
        }))
    }
    join_all(tasks).await;

    Ok(())
}

fn build_url(
    year: &String,
    offset: &String,
    page_size: &String,
    api_key: &String,
) -> (String, String, String) {
    (
        api_key.to_string(),
        year.to_string(),
        format!(
            "{}{}/granules?offset={}&pageSize={}&api_key={}",
            BASE_URL, year, offset, page_size, api_key
        ),
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
