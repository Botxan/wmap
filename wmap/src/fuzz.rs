use std::collections::BTreeMap;

pub struct Fuzzer {
    pub methods: Vec<String>,
    pub encoding: String,
}

impl Fuzzer {
    pub fn new(methods: Vec<String>, encoding: String) -> Self {
        Self { methods, encoding }
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
        /*
        // 1. Basic structure manipulation

        // 1.1 Suppress chars
        for i in 0..path_len {
            let suppressed = self.suppress_char(request_target, i).unwrap();
            mutated_request_targets.push(suppressed);
        }

        // 1.2. Interchange chars
        if path_len > 1 {
            for i in 0..path_len {
                for j in i + 1..path_len {
                    let swapped = self.swap_chars(request_target, i, j);
                    mutated_request_targets.push(swapped);
                }
            }
        }

        // 1.3. Add chars
        let chars = vec!['/', 'z', '#'];
        let added = self.add_chars(request_target, chars);
        mutated_request_targets.extend(added);
        */

        // 2. Path manipulation
        // 2.1 Alter path separator
        mutated_request_targets.push(request_target.replace("/", "\\"));

        // 2.2 Truncate path
        if request_target.contains('/') {
            let truncated = request_target.rsplitn(2, '/').last().unwrap_or("");
            mutated_request_targets.push(format!("/{}", truncated));
        }

        // 2.3 Path traversal sequences
        mutated_request_targets.push(request_target.replace("/", "/../"));
        mutated_request_targets.push(request_target.replace("/", "/../../../../../../../../../../../../../../../../"));

        // 2.4 Duplicate  segments
        mutated_request_targets.push(request_target.repeat(2));

        // 2.5 Overlong segments
        if let Some(first_segment) = request_target.split('/').nth(1) {
            let overlong_segment = "too-long-".repeat(50);
            let mutated_target = request_target.replacen(first_segment, &overlong_segment, 1);
            mutated_request_targets.push(mutated_target);
        }

        // 2.6 Slash padding
        mutated_request_targets.push(request_target.replace("/", "////"));

        mutated_request_targets
    }

    fn suppress_char(&self, input: &str, i: usize) -> Option<String> {
        (input.len() > 1).then(|| {
            input
                .chars()
                .enumerate()
                .filter_map(|(j, c)| if j != i { Some(c) } else { None })
                .collect::<String>()
        })
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
            .map(|(i, c)| {
                if i % 2 == 0 {
                    c.to_ascii_lowercase()
                } else {
                    c.to_ascii_uppercase()
                }
            })
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
