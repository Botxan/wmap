mod args;
mod fuzz;
mod http_client;
mod logger;
mod utils;

use fuzz::Fuzzer;
use http_client::spacing_type::SpacingType;
use logger::RequestResult;
use std::collections::BTreeMap;

const DEFAULT_HTTP_VERSION: &str = "HTTP/1.1";

fn main() {
    let matches = args::parse_args();

    logger::initialize_logger(&matches);
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
        process_mutated_spacings(fuzzer, method, url, request_target, headers, framework, results);
    }
}

fn process_mutated_methods(fuzzer: &mut Fuzzer, method: &str, url: &str, request_target: &str, headers: &BTreeMap<String, String>, framework: Option<&str>, results: &mut Vec<RequestResult>) {
    let (descriptions, mutations) = fuzzer.fuzz_http_method(method);

    for (description, mutation) in descriptions.iter().zip(mutations.iter()) {
        let request = http_client::craft_request(&mutation, request_target, DEFAULT_HTTP_VERSION, headers, None);
        let (response, response_time) = http_client::send_request(url, &request);
        results.push(RequestResult {
            request_index: fuzzer.request_index,
            mutation_description: description.clone(),
            request,
            response,
            response_time,
            framework: framework.map(|f| f.to_string()),
        });
        fuzzer.request_index += 1;
    }
}

fn process_mutated_request_targets(fuzzer: &mut Fuzzer, method: &str, url: &str, request_target: &str, headers: &BTreeMap<String, String>, framework: Option<&str>, results: &mut Vec<RequestResult>) {
    let (descriptions, mutations) = fuzzer.fuzz_request_target(request_target);

    for (description, mutation) in descriptions.iter().zip(mutations.iter()) {
        let request = http_client::craft_request(method, &mutation, DEFAULT_HTTP_VERSION, headers, None);
        let (response, response_time) = http_client::send_request(url, &request);
        results.push(RequestResult {
            request_index: fuzzer.request_index,
            mutation_description: description.clone(),
            request,
            response,
            response_time,
            framework: framework.map(|f| f.to_string()),
        });
        fuzzer.request_index += 1;
    }
}

fn process_mutated_http_versions(fuzzer: &mut Fuzzer, method: &str, url: &str, request_target: &str, headers: &BTreeMap<String, String>, framework: Option<&str>, results: &mut Vec<RequestResult>) {
    let (descriptions, mutations) = fuzzer.fuzz_http_version(DEFAULT_HTTP_VERSION);

    for (description, mutation) in descriptions.iter().zip(mutations.iter()) {
        let request = http_client::craft_request(method, request_target, mutation, headers, None);
        let (response, response_time) = http_client::send_request(url, &request);
        results.push(RequestResult {
            request_index: fuzzer.request_index,
            mutation_description: description.clone(),
            request,
            response,
            response_time,
            framework: framework.map(|f| f.to_string()),
        });
        fuzzer.request_index += 1;
    }
}

fn process_mutated_headers(fuzzer: &mut Fuzzer, method: &str, url: &str, request_target: &str, framework: Option<&str>, results: &mut Vec<RequestResult>) {
    let (descriptions, mutations) = fuzzer.fuzz_headers(url);

    for (description, mutation) in descriptions.iter().zip(mutations.iter()) {
        let request = http_client::craft_request(method, request_target, DEFAULT_HTTP_VERSION, &mutation, None);
        let (response, response_time) = http_client::send_request(url, &request);
        results.push(RequestResult {
            request_index: fuzzer.request_index,
            mutation_description: description.clone(),
            request,
            response,
            response_time,
            framework: framework.map(|f| f.to_string()),
        });
        fuzzer.request_index += 1;
    }
}

fn process_mutated_spacings(fuzzer: &mut Fuzzer, method: &str, url: &str, request_target: &str, headers: &BTreeMap<String, String>, framework: Option<&str>, results: &mut Vec<RequestResult>) {
    let (descriptions, spacing_types) = (
        vec![
            String::from("[spacing] All spaces"),
            String::from("[spacing] All tabs"),
            String::from("[spacing] Double spaces"),
            String::from("[spacing] Multiple spaces"),
            String::from("[spacing] Null terminated"),
            String::from("[spacing] Multiple line breaks"),
            String::from("[spacing] Leading and trailing tabs"),
            String::from("[spacing] Leading and trailing whitespaces"),
            String::from("[spacing] BEL and SOH control chars instead of \r\n"),
        ],
        vec![
            SpacingType::AllSpaces,
            SpacingType::AllTabs,
            SpacingType::DoubleSpaces,
            SpacingType::MultipleSpaces,
            SpacingType::NullTerminated,
            SpacingType::MultipleLineBreaks,
            SpacingType::LeadingTrailingTabs,
            SpacingType::LeadingTrailingWhitespaces,
            SpacingType::ControlChars,
        ],
    );

    for (description, spacing_type) in descriptions.iter().zip(spacing_types.iter()) {
        let request = http_client::craft_request(method, request_target, DEFAULT_HTTP_VERSION, headers, Some(&spacing_type));
        let (response, response_time) = http_client::send_request(url, &request);
        results.push(RequestResult {
            request_index: fuzzer.request_index,
            mutation_description: description.clone(),
            request,
            response,
            response_time,
            framework: framework.map(|f| f.to_string()),
        });
        fuzzer.request_index += 1;
    }
}
