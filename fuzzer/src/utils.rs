use crate::logger;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;

pub fn initialize_logger(matches: &clap::ArgMatches) {
    let verbose = matches.get_flag("verbose");
    let output_file = matches.get_one::<String>("output").map(|s| s.as_str());
    let output_format = matches.get_one::<String>("output-format").unwrap();
    let formatter = get_formatter(&output_format);

    logger::Logger::init(verbose, output_file, false, formatter);
}

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

fn get_formatter(output_format: &str) -> Arc<dyn logger::OutputFormatter + Send + Sync> {
    match output_format {
        "json" => Arc::new(logger::JsonFormatter),
        "csv" => Arc::new(logger::CsvFormatter),
        _ => Arc::new(logger::JsonFormatter),
    }
}
