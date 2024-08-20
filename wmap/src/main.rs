mod args;
mod fuzz;
mod http_client;
mod logger;
mod utils;

use fuzz::Fuzzer;
use logger::RequestResult;
use std::collections::BTreeMap;

const DEFAULT_HTTP_VERSION: &str = "HTTP/1.1";

fn main() {
    let matches = args::parse_args();

    utils::initialize_logger(&matches);
    log_args!(&matches);

    let urls_and_frameworks: Vec<String> = if let Some(input_file) = matches.get_one::<String>("input") {
        utils::read_urls_from_file(input_file)
    } else {
        vec![matches.get_one::<String>("url").expect("URL is required").clone()]
    };

    let methods: Vec<String> = matches.get_many::<String>("methods").expect("Methods are required").map(|s| s.to_string()).collect();

    let mut results = Vec::new();
    let mut request_index = 0;

    for url_and_framework in &urls_and_frameworks {
        let (url, framework) = utils::extract_url_and_framework(url_and_framework);
        let (_, _, request_target) = http_client::parse_url(&url);
        let headers: BTreeMap<String, String> = http_client::get_default_headers(&url);

        let mut fuzzer = Fuzzer::new(methods.clone(), request_index);
        process_requests_per_method(&mut fuzzer, &url, &request_target, &headers, framework.as_deref(), &mut results);
        request_index = fuzzer.request_index;
    }

    log_formatted_results!(results);
}

fn process_requests_per_method(fuzzer: &mut Fuzzer, url: &str, request_target: &str, headers: &BTreeMap<String, String>, framework: Option<&str>, results: &mut Vec<RequestResult>) {
    let methods = fuzzer.methods.clone();

    for method in &methods {
        process_mutated_methods(fuzzer, method, url, request_target, headers, framework, results);
        process_mutated_request_targets(fuzzer, method, url, request_target, headers, framework, results);
        process_mutated_http_versions(fuzzer, method, url, request_target, headers, framework, results);
        process_mutated_headers(fuzzer, method, url, request_target, framework, results);
    }
}

fn process_mutated_methods(fuzzer: &mut Fuzzer, method: &str, url: &str, request_target: &str, headers: &BTreeMap<String, String>, framework: Option<&str>, results: &mut Vec<RequestResult>) {
    let mutated_methods = fuzzer.fuzz_http_method(method);

    for mutated_method in mutated_methods {
        let request = http_client::craft_request(&mutated_method, request_target, DEFAULT_HTTP_VERSION, headers);
        let (response, response_time) = http_client::send_request(url, &request);
        results.push(RequestResult {
            request_index: fuzzer.request_index,
            request,
            response,
            response_time,
            framework: framework.map(|s| s.to_string()),
        });
        fuzzer.request_index += 1;
    }
}

fn process_mutated_request_targets(fuzzer: &mut Fuzzer, method: &str, url: &str, request_target: &str, headers: &BTreeMap<String, String>, framework: Option<&str>, results: &mut Vec<RequestResult>) {
    let mutated_request_targets = fuzzer.fuzz_request_target(request_target);

    for mutated_request_target in &mutated_request_targets {
        let request = http_client::craft_request(method, &mutated_request_target, DEFAULT_HTTP_VERSION, headers);
        let (response, response_time) = http_client::send_request(url, &request);
        results.push(RequestResult {
            request_index: fuzzer.request_index,
            request,
            response,
            response_time,
            framework: framework.map(|s| s.to_string()),
        });
        fuzzer.request_index += 1;
    }
}

fn process_mutated_http_versions(fuzzer: &mut Fuzzer, method: &str, url: &str, request_target: &str, headers: &BTreeMap<String, String>, framework: Option<&str>, results: &mut Vec<RequestResult>) {
    let mutated_http_versions = fuzzer.fuzz_http_version();

    for mutated_http_version in &mutated_http_versions {
        let request = http_client::craft_request(method, request_target, mutated_http_version, headers);
        let (response, response_time) = http_client::send_request(url, &request);
        results.push(RequestResult {
            request_index: fuzzer.request_index,
            request,
            response,
            response_time,
            framework: framework.map(|s| s.to_string()),
        });
        fuzzer.request_index += 1;
    }
}

fn process_mutated_headers(fuzzer: &mut Fuzzer, method: &str, url: &str, request_target: &str, framework: Option<&str>, results: &mut Vec<RequestResult>) {
    let mutated_headers_list = fuzzer.fuzz_headers(url);
    for mutated_headers in mutated_headers_list {
        let request = http_client::craft_request(method, request_target, DEFAULT_HTTP_VERSION, &mutated_headers);
        let (response, response_time) = http_client::send_request(url, &request);
        results.push(RequestResult {
            request_index: fuzzer.request_index,
            request,
            response,
            response_time,
            framework: framework.map(|s| s.to_string()),
        });
        fuzzer.request_index += 1;
    }
}
