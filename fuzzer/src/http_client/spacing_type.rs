pub enum SpacingType {
    AllSpaces,
    AllTabs,
    DoubleSpaces,
    MultipleSpaces,
    NullTerminated,
    MultipleLineBreaks,
    LeadingTrailingTabs,
    LeadingTrailingWhitespaces,
    ControlChars,
}

impl SpacingType {
    pub fn apply(&self, input: &str) -> String {
        match self {
            SpacingType::AllTabs => input.replace(" ", "\t"),
            SpacingType::AllSpaces => input.replace("\t", " "),
            SpacingType::DoubleSpaces => input.replace(" ", "  "),
            SpacingType::MultipleSpaces => input.replace(" ", "    "),
            SpacingType::NullTerminated => input.replace("\r\n", "\0\r\n"),
            SpacingType::MultipleLineBreaks => input.replace("\r\n", "\r\n\r\n"),
            SpacingType::LeadingTrailingTabs => {
                let mut result = String::new();
                result.push_str("\t");
                result.push_str(input);
                result.push_str("\t");
                result
            }
            SpacingType::LeadingTrailingWhitespaces => {
                let mut result = String::new();
                result.push_str(" ");
                result.push_str(input);
                result.push_str(" ");
                result
            }
            SpacingType::ControlChars => input
                .chars()
                .map(|c| match c {
                    '\r' => '\u{0007}', // ASCII BEL
                    '\n' => '\u{0001}', // ASCII SOH
                    _ => c,
                })
                .collect(),
        }
    }
}
