use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub count: u16,
    pub page_size: u16,
    pub next_page: Option<String>,
    pub granules: Vec<Granule>,
}

#[derive(Deserialize)]
pub struct Granule {
    pub title: String,
}

#[derive(Serialize)]
pub struct Heading {
    pub title: String,
    pub year: usize,
}
