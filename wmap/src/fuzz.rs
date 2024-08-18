use crate::http_client;
use std::{cmp, collections::BTreeMap};
use url::Url;

pub struct Fuzzer {
    pub methods: Vec<String>,
    pub encoding: String,
    pub request_index: u32,
}

impl Fuzzer {
    pub fn new(methods: Vec<String>, encoding: String, request_index: u32) -> Self {
        Self { methods, encoding, request_index }
    }

    pub fn fuzz_http_method(&self, method: &str) -> Vec<String> {
        let mut mutated_methods = Vec::new();
        let method_len = method.len();

        // 1. Suppress chars
        for i in 0..method_len {
            let suppressed = self.suppress_char(method, i).unwrap();
            mutated_methods.push(suppressed);
        }

        // 2. Interchange chars
        if method_len > 1 {
            for i in 0..method_len {
                for j in i + 1..method_len {
                    let swapped = self.swap_chars(method, i, j);
                    mutated_methods.push(swapped);
                }
            }
        }

        // 3. Add chars
        // Common chars
        let common_chars = vec!['A', 'B'];
        let added = self.add_chars(method, common_chars);
        mutated_methods.extend(added);

        // Unexpected chars
        let unexpected_chars = vec!['$', '@', '!', '1'];
        let added = self.add_chars(method, unexpected_chars);
        mutated_methods.extend(added);

        // Large number of chars
        let large_strings = vec!["#".repeat(1024), "%".repeat(4096)];
        let added = self.add_chars(method, large_strings);
        mutated_methods.extend(added);

        // 4. Modify existing chars
        let replacements = vec![('O', "0"), ('E', "3"), ('A', "@"), ('P', "Ρ"), ('T', "✝")];
        let replaced = self.replace_chars(method, &replacements);
        mutated_methods.extend(replaced);

        // 5. Case variations
        // Lowercase
        mutated_methods.push(method.to_lowercase());

        // Toggle between upper and lower case letters
        let case_alternated = self.alternate_case(method);
        mutated_methods.push(case_alternated);

        // 6. Whitespace and formatting
        let with_spaces = self.add_whitespaces_between_chars(method);
        mutated_methods.push(with_spaces);

        // 7. Concatenated methods
        let appended_methods = vec!["POST", "GET"];
        let concatenated = self.concatenate_with_methods(method, &appended_methods);
        mutated_methods.extend(concatenated);

        mutated_methods
    }

    pub fn fuzz_request_target(&self, request_target: &str) -> Vec<String> {
        let mut mutated_request_targets = Vec::new();
        let path_len = request_target.len();
        let max_len = cmp::min(path_len, 5);

        // 1. Basic structure manipulation
        // Suppress chars
        for i in 0..max_len {
            let suppressed = self.suppress_char(request_target, i).unwrap();
            mutated_request_targets.push(suppressed);
        }

        // Interchange chars
        if path_len > 1 {
            for i in 0..max_len {
                for j in i + 1..max_len {
                    let swapped = self.swap_chars(request_target, i, j);
                    mutated_request_targets.push(swapped);
                }
            }
        }

        // Add chars
        let chars = vec!['/', 'z', '#'];
        let added = self.add_chars(request_target, chars);
        mutated_request_targets.extend(added);

        // Path manipulation
        // Alter path separator
        mutated_request_targets.push(request_target.replace("/", "\\"));

        // Truncate path
        if request_target.contains('/') {
            let truncated = request_target.rsplitn(2, '/').last().unwrap_or("");
            mutated_request_targets.push(format!("/{}", truncated));
        }

        // Path traversal sequences
        mutated_request_targets.push(request_target.replace("/", "/../"));
        mutated_request_targets.push(request_target.replace("/", "/../../../../../../../../../../../../../../../../"));

        // Duplicate  segments
        mutated_request_targets.push(request_target.repeat(2));

        // Overlong segments
        if let Some(first_segment) = request_target.split('/').nth(1) {
            let overlong_segment = "too-long-".repeat(50);
            let mutated_target = request_target.replacen(first_segment, &overlong_segment, 1);
            mutated_request_targets.push(mutated_target);
        }

        // Slash padding
        mutated_request_targets.push(request_target.replace("/", "////"));

        // 3. Query string mutations
        let (path, query) = request_target.split_once('?').unwrap_or((request_target, ""));
        if !query.is_empty() {
            // Duplicate query parameter
            mutated_request_targets.push(format!("{}?{}&{}", path, query, query));

            // Add unexpected query parameter
            mutated_request_targets.push(format!("{}?{}&unexpected=1", path, query));

            // Empty query parameter
            mutated_request_targets.push(format!("{}?{}&param=", path, query));

            // Large query parameter values
            mutated_request_targets.push(format!("{}?{}={}", path, query, "a".repeat(1024)));

            // Special characters in query parameters
            mutated_request_targets.push(format!("{}?{}=va!ue", path, query));
            mutated_request_targets.push(format!("{}?{}=<script>alert(1)</script>", path, query));

            // Malformed query string
            mutated_request_targets.push(format!("{}?{}?otherparam=othervalue", path, query));
            mutated_request_targets.push(format!("{}?{}&", path, query));
        }

        // 4. Fragment mutations
        if let Some((path, _fragment)) = request_target.split_once('#') {
            // Alter or add fragment
            mutated_request_targets.push(format!("{}#fragme@nt", path));
            mutated_request_targets.push(format!("{}#", path));
            mutated_request_targets.push(format!("{}#{}", path, "longfragment".repeat(1024)));
        }

        // 5. Encoding and escaping mutations
        // Slash encoding
        mutated_request_targets.push(request_target.replace("/", "%2F"));

        if !query.is_empty() {
            // Double URL encoding
            mutated_request_targets.push(format!("{}?%25{}", path, query));

            // Invalid or incomplete URL encoding
            mutated_request_targets.push(format!("{}?%G4{}", path, query));
        }

        mutated_request_targets
    }

    pub fn fuzz_http_version(&self) -> Vec<String> {
        let mut mutated_versions = Vec::new();

        // 1. Valid but uncommon versions
        let uncommon_versions = vec!["HTTP/0.9", "HTTP/2.0", "HTTP/3.0"];
        for v in uncommon_versions {
            mutated_versions.push(v.to_string());
        }

        // 2. Malformed versions
        let malformed_versions = vec![
            "HTTP/1.",    // Incomplete version
            "HTTP/1.2.3", // Extra dot
            "HTT/1.1",    // Typo in protocol
            "HTTP/1.1 ",  // Trailing space
            "HTTP/",      // Empty version number
            "",           // Empty version
        ];
        for v in malformed_versions {
            mutated_versions.push(v.to_string());
        }

        // 3. Unexpected characters
        let unexpected_char_versions = vec!["HTTP/1.1#", "HTTP/1.1!", "HTTP/1.1@"];
        for v in unexpected_char_versions {
            mutated_versions.push(v.to_string());
        }

        // 5. Overlong version
        let overlong_version = format!("HTTP/1.1{}", "A".repeat(1024));
        mutated_versions.push(overlong_version);

        // 6. Encoding mutations
        let encoded_versions = vec![
            "HTTP%2F1.1",       // Slash encoded
            "HTTP%2F1%2E1",     // Slash and dot encoded
            "%48%54%54%50/1.1", // Full version encoded
        ];
        for v in encoded_versions {
            mutated_versions.push(v.to_string());
        }

        mutated_versions
    }

    pub fn fuzz_headers(&self, base_url: &str) -> Vec<BTreeMap<String, String>> {
        let normalized_url = http_client::normalize_url(base_url);
        let parsed_url = Url::parse(&normalized_url).expect("Invalid URL format");
        let domain = parsed_url.host_str().unwrap_or("");

        let base_headers = http_client::get_default_headers(domain);

        let mut mutated_headers = Vec::new();

        // 0. Default headers
        mutated_headers.push(base_headers.clone());

        // 1. User-Agent variations
        let user_agents = vec![
            "",                                                                               // Empty
            "curl/7.68.0",                                                                    // Command line tool
            "Mozilla/4.0 (compatible; MSIE 6.0; Windows NT 5.1)",                             // Old browser
            "Googlebot/2.1 (+http://www.google.com/bot.html)",                                // Web crawler
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0", // Another modern browser
            "UnknownAgent/1.0",                                                               // Non-standard agent
        ];

        for user_agent in user_agents {
            let mut headers = base_headers.clone();
            headers.insert("User-Agent".to_string(), user_agent.to_string());
            mutated_headers.push(headers);
        }

        // 2. Referer manipulations
        let referer_non_existent_page = format!("{}/non-existent-page", &normalized_url);
        let referer_https = format!("https://{}", domain);

        let referers = vec![
            "",                         // Empty
            "http://malicious.com",     // Malicious referer
            &referer_non_existent_page, // Non-existent page
            &referer_https,             // HTTPS version of the site
        ];

        for referer in referers {
            let mut headers = base_headers.clone();
            headers.insert("Referer".to_string(), referer.to_string());
            mutated_headers.push(headers);
        }

        // 3. Content-Type manipulations
        let content_types = vec![
            "application/json",                                  // JSON content
            "multipart/form-data",                               // Form data with files
            "text/plain",                                        // Plain text
            "application/xml",                                   // XML content
            "application/x-www-form-urlencoded; charset=UTF-16", // Different charset
        ];

        for content_type in content_types {
            let mut headers = base_headers.clone();
            headers.insert("Content-Type".to_string(), content_type.to_string());
            mutated_headers.push(headers);
        }

        // 4. Host manipulations
        let host_custom_port = format!("{}:8080", domain);
        let host_subdomain = format!("sub.{}", domain);

        let hosts = vec![
            "localhost",       // Localhost
            "127.0.0.1",       // Localhost IP
            &host_custom_port, // Custom port
            domain,            // Base domain
            &host_subdomain,   // Subdomain
        ];

        for host in hosts {
            let mut headers = base_headers.clone();
            headers.insert("Host".to_string(), host.to_string());
            mutated_headers.push(headers);
        }

        // 5. X-Forwarded-For manipulations
        let x_forwarded_for_values = vec![
            "",                              // Empty
            "192.168.1.1",                   // Private IP address
            "10.0.0.1",                      // Another private IP address
            "203.0.113.195, 198.51.100.101", // Multiple IP addresses
            "127.0.0.1",                     // Localhost IP
        ];

        for x_forwarded_for in x_forwarded_for_values {
            let mut headers = base_headers.clone();
            headers.insert("X-Forwarded-For".to_string(), x_forwarded_for.to_string());
            mutated_headers.push(headers);
        }

        // 6. Cookie manipulations
        let cookies = vec![
            "",                                       // Empty
            "PHPSESSID=abcdef123456",                 // Valid session ID
            "PHPSESSID=; path=/; HttpOnly",           // Empty session ID
            "malicious=1; PHPSESSID=123456789abcdef", // Additional malicious cookie
        ];

        for cookie in cookies {
            let mut headers = base_headers.clone();
            headers.insert("Cookie".to_string(), cookie.to_string());
            mutated_headers.push(headers);
        }

        // 7. Authorization manipulations
        let authorizations = vec![
            "",                           // Empty
            "Basic dXNlcjpwYXNzd29yZA==", // Basic auth with user:password
            "Bearer somejwttoken",        // Bearer token
            "Negotiate YII=",             // Negotiate (Kerberos) token
        ];

        for authorization in authorizations {
            let mut headers = base_headers.clone();
            headers.insert("Authorization".to_string(), authorization.to_string());
            mutated_headers.push(headers);
        }

        mutated_headers
    }

    fn suppress_char(&self, input: &str, i: usize) -> Option<String> {
        (input.len() > 1).then(|| input.chars().enumerate().filter_map(|(j, c)| if j != i { Some(c) } else { None }).collect::<String>())
    }

    fn swap_chars(&self, method: &str, i: usize, j: usize) -> String {
        let mut chars: Vec<char> = method.chars().collect();
        chars.swap(i, j);
        chars.into_iter().collect()
    }

    fn add_chars<T>(&self, input: &str, elements: Vec<T>) -> Vec<String>
    where
        T: Into<String>,
    {
        let mut results = Vec::new();

        for elem in elements {
            let elem_str = elem.into();
            results.push(format!("{}{}", input, elem_str));
            results.push(format!("{}{}", elem_str, input));
        }

        results
    }

    fn replace_chars(&self, input: &str, replacements: &[(char, &str)]) -> Vec<String> {
        let mut results = Vec::new();

        for (original, replacement) in replacements {
            let mutated = input.replace(*original, replacement);
            if mutated != input {
                results.push(mutated);
            }
        }

        results
    }

    fn alternate_case(&self, input: &str) -> String {
        input
            .chars()
            .enumerate()
            .map(|(i, c)| if i % 2 == 0 { c.to_ascii_lowercase() } else { c.to_ascii_uppercase() })
            .collect()
    }

    fn add_whitespaces_between_chars(&self, input: &str) -> String {
        input.chars().collect::<String>().replace("", " ").trim().to_string()
    }

    fn concatenate_with_methods(&self, input: &str, methods_to_append: &[&str]) -> Vec<String> {
        let mut results = Vec::new();

        for method in methods_to_append {
            let concatenated = format!("{}{}", input, method);
            results.push(concatenated);
        }

        results
    }
}
