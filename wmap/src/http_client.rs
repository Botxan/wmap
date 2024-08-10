use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::time::Duration;
use url::Url;

pub fn send_request(target_url: &str, method: &str, path: &str, version: &str, headers: &BTreeMap<String, String>) -> String {
    let (host, _path) = parse_url(target_url);
    let port = 80;

    // Craft the request line
    let request_line = format!("{} {} {}\r\n", method, path, version);

    // Craft the headers
    let mut headers_str = String::new();
    for (key, value) in headers {
        headers_str.push_str(&format!("{}: {}\r\n", key, value));
    }

    // Combine request line, headers and body
    let request = format!("{}{}\r\n", request_line, headers_str);

    // Open a TCP stream to the server
    let mut stream = TcpStream::connect(format!("{}:{}", host, port)).expect("Failed to connect to server");
    stream.set_nodelay(true).expect("Failed to set nodelay");
    stream.set_read_timeout(Some(Duration::new(5, 0))).expect("Failed to set read timeout");

    // Send the crafted request
    stream.write_all(request.as_bytes()).expect("Failed to write to stream");

    // Read the response
    let mut response = String::new();
    let mut buffer = [0; 512];
    let mut headers_str = String::new();

    while stream.read(&mut buffer).expect("Error reading response") > 0 {
        response.push_str(&String::from_utf8_lossy(&buffer));
        if response.contains("\r\n\r\n") {
            break;
        }
    }

    // Parse headers and get Content-Length header if exists
    let headers_end = match response.find("\r\n\r\n") {
        Some(index) => index + 4,
        None => {
            eprintln!("Error: HTTP headers not found in the response");
            return String::new();
        }
    };
    headers_str = response[..headers_end].to_string();
    let content_length = headers_str
        .lines()
        .find(|&line| line.starts_with("Content-Length:"))
        .and_then(|line| line.split(": ").nth(1))
        .and_then(|len| len.trim().parse::<usize>().ok())
        .unwrap_or(0);

    // Read the body
    let mut body = String::new();
    let mut total_bytes_read = response[headers_end..].len();
    body.push_str(&response[headers_end..]);

    while total_bytes_read < content_length {
        let bytes_read = stream.read(&mut buffer).expect("Error reading response");
        total_bytes_read += bytes_read;
        body.push_str(&String::from_utf8_lossy(&buffer[..bytes_read]));
    }

    // Close connection
    stream.shutdown(Shutdown::Both).expect("Shutdown failed");

    response
}

pub fn parse_url(url: &str) -> (String, String) {
    let mut cleaned_url = url.to_string();

    // Remove the schema (http:// or https://) if present
    if cleaned_url.starts_with("http://") {
        cleaned_url = cleaned_url[7..].to_string();
    } else if cleaned_url.starts_with("https://") {
        cleaned_url = cleaned_url[8..].to_string();
    }

    // Split the remaining URL into host and path
    let mut parts = cleaned_url.splitn(2, '/');
    let host = parts.next().unwrap_or("").to_string();
    let path = format!("/{}", parts.next().unwrap_or(""));

    (host, path)
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
