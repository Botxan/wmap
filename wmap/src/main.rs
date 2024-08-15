mod args;
mod fuzz;
mod http_client;
mod logger;

use fuzz::Fuzzer;
use logger::Logger;
use std::collections::BTreeMap;

const DEFAULT_HTTP_VERSION: &str = "HTTP/1.1";

fn main() {
    let matches = args::parse_args();
    let mut logger = initialize_logger(&matches);
    logger.print_args(&matches); // Print arguments only if verbose

    let url = matches.get_one::<String>("url").expect("URL is required");
    let encoding = matches.get_one::<String>("encoding").expect("Encoding is required").clone();
    let methods: Vec<String> = matches.get_many::<String>("methods").expect("Methods are required").map(|s| s.to_string()).collect();
    let (_host, request_target) = http_client::parse_url(url);
    let headers: BTreeMap<String, String> = http_client::get_default_headers(url);

    let mut fuzzer = Fuzzer::new(methods, encoding, 0);
    process_requests_per_method(&mut fuzzer, &url, &request_target, &headers, &mut logger);

    println!("\nTotal number of requests to be generated: {}", fuzzer.request_count);
}

fn initialize_logger(matches: &clap::ArgMatches) -> Logger {
    let verbose = matches.get_flag("verbose");
    let output_file = matches.get_one::<String>("output").map(String::as_str);
    Logger::new(verbose, output_file)
}

fn process_requests_per_method(fuzzer: &mut Fuzzer, url: &str, request_target: &str, headers: &BTreeMap<String, String>, logger: &mut Logger) {
    let methods = fuzzer.methods.clone();

    for method in &methods {
        log_print!(logger, "\n----- Request line method mutations -----");
        process_mutated_methods(fuzzer, method, url, request_target, headers, logger);

        log_print!(logger, "\n----- Request line request target mutations -----");
        process_mutated_request_targets(fuzzer, method, request_target, url, headers, logger);

        log_print!(logger, "\n----- Request line HTTP version mutations -----");
        process_mutated_http_versions(fuzzer, method, request_target, url, headers, logger);

        log_print!(logger, "\n----- Header mutations -----");
        process_mutated_headers(fuzzer, method, request_target, url, logger);
    }
}

fn process_mutated_methods(fuzzer: &mut Fuzzer, method: &str, url: &str, request_target: &str, headers: &BTreeMap<String, String>, logger: &mut Logger) {
    let mutated_methods = fuzzer.fuzz_http_method(method);

    for mutated_method in mutated_methods {
        log_print!(logger, "[Request nº{}] - {} -> {}", fuzzer.request_count, method, mutated_method);
        fuzzer.request_count += 1;
        let response = http_client::send_request(url, &mutated_method, request_target, DEFAULT_HTTP_VERSION, headers);
        log_print!(logger, "Response\n: {}", response);
    }
}

fn process_mutated_request_targets(fuzzer: &mut Fuzzer, method: &str, request_target: &str, url: &str, headers: &BTreeMap<String, String>, logger: &mut Logger) {
    let mutated_request_targets = fuzzer.fuzz_request_target(request_target);

    for mutated_request_target in &mutated_request_targets {
        log_print!(logger, "[Request nº{}] - {} -> {}", fuzzer.request_count, request_target, mutated_request_target);
        fuzzer.request_count += 1;
        let response = http_client::send_request(url, method, mutated_request_target, DEFAULT_HTTP_VERSION, headers);
        log_print!(logger, "Response\n: {}", response);
    }
}

fn process_mutated_http_versions(fuzzer: &mut Fuzzer, method: &str, request_target: &str, url: &str, headers: &BTreeMap<String, String>, logger: &mut Logger) {
    let mutated_http_versions = fuzzer.fuzz_http_version();

    for mutated_http_version in &mutated_http_versions {
        log_print!(logger, "[Request nº{}] - {} -> {}", fuzzer.request_count, DEFAULT_HTTP_VERSION, mutated_http_version);
        fuzzer.request_count += 1;
        let response = http_client::send_request(url, method, request_target, mutated_http_version, headers);
        log_print!(logger, "Response\n: {}", response);
    }
}

fn process_mutated_headers(fuzzer: &mut Fuzzer, method: &str, request_target: &str, url: &str, logger: &mut Logger) {
    let mutated_headers_list = fuzzer.fuzz_headers(url);
    for mutated_headers in mutated_headers_list {
        log_print!(logger, "[Request nº{}] - Headers:", fuzzer.request_count);

        for (mutated_header_key, mutated_header_value) in &mutated_headers {
            log_print!(logger, "\t{}: {}", mutated_header_key, mutated_header_value);
        }

        fuzzer.request_count += 1;
        let response = http_client::send_request(url, method, request_target, DEFAULT_HTTP_VERSION, &mutated_headers);
        log_print!(logger, "Response\n: {}", response);
    }
}
