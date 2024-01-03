use std::sync::Arc;

use crate::api::Page;
use crate::constants::BASE_URL;
use crate::Args;
use anyhow::Result;
use reqwest::{self, blocking::Response as BlockingResponse, Response, StatusCode};
use time::OffsetDateTime;

pub fn build_url(year: &String, args: Arc<Args>) -> (String, String) {
    (
        year.to_string(),
        format!(
            "{}{}/granules?offsetMark=*&pageSize={}&api_key={}",
            BASE_URL, year, args.page_size, args.api_key
        ),
    )
}

pub fn current_year() -> String {
    format!("{}", OffsetDateTime::now_utc().year())
}

pub fn is_rate_limited(response: &Response) -> bool {
    response.status() == StatusCode::TOO_MANY_REQUESTS
}

pub fn is_rate_limited_blocking(response: &BlockingResponse) -> bool {
    response.status() == StatusCode::TOO_MANY_REQUESTS
}

pub fn remaining_requests(response: &Response) -> Result<u16> {
    Ok(response
        .headers()
        .get("x-ratelimit-remaining")
        .expect("No matching header found")
        .to_str()?
        .parse::<u16>()?)
}

pub fn remaining_requests_blocking(response: &BlockingResponse) -> Result<u16> {
    Ok(response
        .headers()
        .get("x-ratelimit-remaining")
        .expect("No matching header found")
        .to_str()?
        .parse::<u16>()?)
}

pub fn requests_to_make(page: &Page) -> u16 {
    let quotient = page.count / page.page_size;
    match page.count % page.page_size {
        0 => quotient,
        _ => quotient + 1,
    }
}

#[cfg(test)]
mod test {
    use crate::utils::{current_year, requests_to_make, Page};

    #[test]
    pub fn test_requests_to_make_1000() {
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
    pub fn test_requests_to_make_100() {
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
    pub fn test_requests_to_make_10() {
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

    #[test]
    pub fn test_current_year() {
        let expected = String::from("2024");
        let result = current_year();
        assert_eq!(expected, result)
    }
}
