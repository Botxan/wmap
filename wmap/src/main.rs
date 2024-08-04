mod args;
mod fuzz;
mod http_client;
mod logger;

use logger::Logger;
use std::collections::BTreeMap;

fn main() {
    let matches = args::parse_args();

    let verbose = matches.get_flag("verbose");
    let logger = Logger::new(verbose);

    // Print args only if verbose
    logger.print_args(&matches);

    let url = matches.get_one::<String>("url").unwrap();
    let (host, path) = http_client::parse_url(url);
    let methods: Vec<String> = matches
        .get_many::<String>("methods")
        .unwrap()
        .map(|s| s.to_string())
        .collect();

    let headers: BTreeMap<String, String> = get_headers(&host);

    // Generate and send request line mutations for each user-specified method
    for method in &methods {
        let response = http_client::send_request(&url, &method, &path, "HTTP/1.1", &headers);
        println!("Response: {}", response);
    }
}

fn get_headers(host: &str) -> BTreeMap<String, String> {
    vec![
        ("Accept".to_string(), "*/*".to_string()),
        ("Host".to_string(), host.to_string()),
        ("User-Agent".to_string(), "wmap/0.1.0".to_string()),
    ]
    .into_iter()
    .collect()
}
