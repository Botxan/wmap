use crate::logger;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn read_urls_from_file(file_path: &str) -> Vec<String> {
    let file = File::open(file_path).expect("Failed to open input file");
    let reader = BufReader::new(file);
    reader.lines().map(|line| line.expect("Failed to read line")).collect()
}

pub fn extract_url_and_framework(url: &str) -> (String, Option<String>) {
    if let Some(pos) = url.rfind(';') {
        let (url_part, framework) = url.split_at(pos);
        (url_part.to_string(), Some(framework[1..].to_string())) // Skip the ';'
    } else {
        (url.to_string(), None)
    }
}
