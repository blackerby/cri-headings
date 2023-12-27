use anyhow::{bail, Result};
use scraper::{Html, Selector};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

fn main() -> Result<()> {
    // parse arguments
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <file name>"),
        _ => {}
    }

    let file_name = &args[1];
    let file = File::open(file_name)?;
    let mut buf_reader = BufReader::new(file);
    let mut html = String::new();
    buf_reader.read_to_string(&mut html)?;
    let _document = Html::parse_document(&html);
    let _selector = Selector::parse("p.heading").unwrap();

    let file_stem = Path::new(file_name).file_name().unwrap();
    let output_filename = format!("{}_headings.txt", file_stem.to_str().unwrap());
    let output_file = File::create(output_filename)?;
    let mut buf_writer = BufWriter::new(output_file);

    for heading in _document.select(&_selector) {
        writeln!(&mut buf_writer, "{}", heading.inner_html())?;
    }
    Ok(())
}
