pub mod spacing_type;

use spacing_type::SpacingType;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::time::Instant;
use url::Url;

pub fn craft_request(method: &str, request_target: &str, http_version: &str, headers: &BTreeMap<String, String>, spacing_type: Option<&SpacingType>) -> String {
    let mut request_line = format!("{} {} {}\r\n", method, request_target, http_version);

    // Apply spacing mutation to the request line
    if let Some(mutation_type) = spacing_type {
        request_line = mutation_type.apply(&request_line);
    }

    // Add headers
    let mut request = request_line;
    for (key, value) in headers {
        request.push_str(&format!("{}: {}\r\n", key, value));
    }
    request.push_str("\r\n");

    request
}

pub fn send_request(target_url: &str, request: &str) -> (String, u128) {
    let (host, port, _path) = parse_url(target_url);

    // Open a TCP stream to the server
    let mut stream = TcpStream::connect(format!("{}:{}", host, port)).expect("Failed to connect to server");
    stream.set_nodelay(true).expect("Failed to set nodelay");

    let start_time = Instant::now();

    // Send the crafted request
    stream.write_all(request.as_bytes()).expect("Failed to write to stream");

    // Read the response
    let mut response = Vec::new();
    let mut buffer = [0; 4096];

    loop {
        let bytes_read = stream.read(&mut buffer).expect("Error reading response");

        if bytes_read == 0 {
            break;
        }

        response.extend_from_slice(&buffer[..bytes_read]);

        if response.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }

    let response_str = String::from_utf8_lossy(&response);

    // Split headers and body
    let headers_end = response_str.find("\r\n\r\n").map(|index| index + 4).unwrap_or(response_str.len());

    let headers_str = &response_str[..headers_end];
    let body_str = &response_str[headers_end..];

    // Check Content-Length if available
    let content_length = headers_str
        .lines()
        .find(|&line| line.starts_with("Content-Length:"))
        .and_then(|line| line.split(": ").nth(1))
        .and_then(|len| len.trim().parse::<usize>().ok())
        .unwrap_or(0);

    // If content length not 0, read the rest of the body
    let mut body = body_str.to_string();
    if content_length > body.len() {
        let mut buffer = vec![0; content_length - body.len()];
        let mut total_bytes_read = body.len();

        while total_bytes_read < content_length {
            let bytes_read = stream.read(&mut buffer).expect("Error reading response body");
            if bytes_read == 0 {
                break;
            }
            total_bytes_read += bytes_read;
            body.push_str(&String::from_utf8_lossy(&buffer[..bytes_read]));
        }
    }

    let duration = start_time.elapsed().as_millis();

    stream.shutdown(Shutdown::Both).expect("Shutdown failed");

    (response_str.to_string(), duration)
}

pub fn parse_url(url: &str) -> (String, u16, String) {
    let mut cleaned_url = url.to_string();
    let mut default_port = 80;

    // Remove the scheme (http:// or https://) if present
    if cleaned_url.starts_with("http://") {
        cleaned_url = cleaned_url[7..].to_string();
    } else if cleaned_url.starts_with("https://") {
        cleaned_url = cleaned_url[8..].to_string();
        default_port = 443;
    }

    // Split the remaining URL into host:port and path
    let mut parts = cleaned_url.splitn(2, '/');
    let host_and_port = parts.next().unwrap_or("").to_string();
    let path = format!("/{}", parts.next().unwrap_or(""));

    // Split the host_and_port into host and port
    let mut host_parts = host_and_port.splitn(2, ':');
    let host = host_parts.next().unwrap_or("").to_string();
    let port = host_parts.next().map_or(default_port, |p| p.parse::<u16>().unwrap_or(default_port));

    (host, port, path)
}

pub fn normalize_url(base_url: &str) -> String {
    let normalized_url = if base_url.starts_with("http://") || base_url.starts_with("https://") {
        base_url.to_string()
    } else {
        format!("http://{}", base_url)
    };

    normalized_url
}

pub fn get_default_headers(domain: &str) -> BTreeMap<String, String> {
    let normalized_url = normalize_url(domain);
    let parsed_url = Url::parse(&normalized_url).expect("Invalid URL format");
    let domain = parsed_url.host_str().unwrap_or("");

    vec![
        ("User-Agent", "wmap/0.1.0"),
        ("Referer", &normalized_url),
        ("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"),
        ("Accept-Language", "en-US,en;q=0.5"),
        ("Content-Type", "application/x-www-form-urlencoded"),
        ("Host", domain),
        ("X-Forwarded-For", "203.0.113.195"),
        ("Cookie", "PHPSESSID=123456789abcdef"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}
