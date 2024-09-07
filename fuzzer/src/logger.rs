use clap::ArgMatches;
use lazy_static::lazy_static;
use serde::Serialize;
use serde_json;
use std::fmt::Write as FmtWrite;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

pub struct Logger {
    verbose: bool,
    output_file: Option<Mutex<io::BufWriter<std::fs::File>>>,
    include_framework: bool,
    formatter: Arc<dyn OutputFormatter + Send + Sync>,
}

impl Logger {
    pub fn init(verbose: bool, output_file: Option<&str>, include_framework: bool, formatter: Arc<dyn OutputFormatter + Send + Sync>) {
        let mut logger = GLOBAL_LOGGER.lock().unwrap();
        logger.verbose = verbose;
        logger.include_framework = include_framework;
        logger.formatter = formatter;

        if let Some(file_path) = output_file {
            let file = OpenOptions::new().write(true).truncate(true).create(true).open(file_path).expect("Failed to open log file");
            logger.output_file = Some(Mutex::new(io::BufWriter::new(file)));
        }
    }

    pub fn print(&self, args: std::fmt::Arguments) {
        let formatted = format!("{}", args);
        if let Some(ref output_file) = self.output_file {
            let mut writer = output_file.lock().unwrap();
            writeln!(writer, "{}", formatted).expect("Failed to write to log file");
        } else {
            println!("{}", formatted);
        }
    }

    pub fn print_verbose(&self, args: std::fmt::Arguments) {
        if self.verbose {
            self.print(args);
        }
    }

    pub fn write_formatted_results(&self, results: &[RequestResult]) {
        let formatted_results = self.formatter.format(results);
        if let Some(ref output_file) = self.output_file {
            let mut writer = output_file.lock().unwrap();
            writeln!(writer, "{}", formatted_results).expect("Failed to write formatted results to log file");
        } else {
            println!("{}", formatted_results);
        }
    }

    pub fn print_args(&self, matches: &ArgMatches) {
        if !self.verbose {
            return;
        }

        let mut output = String::new();
        writeln!(output, "Command line arguments:").unwrap();

        for arg_id in matches.ids() {
            let id_str = arg_id.as_str();

            if matches.try_get_many::<String>(id_str).is_ok() {
                if let Ok(Some(values)) = matches.try_get_many::<String>(id_str) {
                    let values: Vec<String> = values.map(|v| v.to_string()).collect();
                    if values.len() == 1 {
                        writeln!(output, "  {}: {:?}", id_str, values[0]).unwrap();
                    } else {
                        writeln!(output, "  {}: {:?}", id_str, values).unwrap();
                    }
                }
            } else if matches.get_flag(id_str) {
                writeln!(output, "  {}: true", id_str).unwrap();
            } else {
                writeln!(output, "  {}: false", id_str).unwrap();
            }
        }

        if let Some(ref output_file) = self.output_file {
            let mut writer = output_file.lock().unwrap();
            writeln!(writer, "{}", output).expect("Failed to write to log file");
        } else {
            print!("{}", output);
        }
    }
}

pub fn initialize_logger(matches: &clap::ArgMatches) {
    let verbose = matches.get_flag("verbose");
    let output_file = matches.get_one::<String>("output").map(|s| s.as_str());
    let output_format = matches.get_one::<String>("output-format").unwrap();
    let formatter = get_formatter(&output_format);

    Logger::init(verbose, output_file, false, formatter);
}

impl Default for Logger {
    fn default() -> Self {
        Logger {
            verbose: false,
            output_file: None,
            include_framework: false,
            formatter: Arc::new(JsonFormatter),
        }
    }
}

lazy_static! {
    pub static ref GLOBAL_LOGGER: Mutex<Logger> = Mutex::new(Logger::default());
}

#[macro_export]
macro_rules! log_print {
    ($($arg:tt)*) => {
        $crate::logger::GLOBAL_LOGGER.lock().unwrap().print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_print_verbose {
    ($($arg:tt)*) => {
        $crate::logger::GLOBAL_LOGGER.lock().unwrap().print_verbose(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_args {
    ($matches:expr) => {
        $crate::logger::GLOBAL_LOGGER.lock().unwrap().print_args($matches);
    };
}

#[macro_export]
macro_rules! log_formatted_results {
    ($results:expr) => {{
        let logger = $crate::logger::GLOBAL_LOGGER.lock().unwrap();
        logger.write_formatted_results(&$results);
    }};
}

pub trait OutputFormatter: Send + Sync {
    fn format(&self, results: &[RequestResult]) -> String;
}

#[derive(Debug, Serialize)]
pub struct RequestResult {
    pub request_index: u32,
    pub mutation_description: String,
    pub request: String,
    pub response: String,
    pub response_time: u128,
    pub framework: Option<String>,
}

#[derive(Serialize)]
pub struct JSONEntry {
    pub request_index: u32,
    pub mutation_description: String,
    pub request: String,
    pub response: String,
    pub response_time: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
}

pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn format(&self, results: &[RequestResult]) -> String {
        serde_json::to_string(results).unwrap_or_else(|_| "[]".to_string())
    }
}

pub struct CsvFormatter;

impl OutputFormatter for CsvFormatter {
    fn format(&self, results: &[RequestResult]) -> String {
        let mut csv_output = String::new();

        // Write header
        writeln!(csv_output, "request_index,request,response,response_time,framework").unwrap();

        // Write each result
        for result in results {
            writeln!(
                csv_output,
                "{},{},{},{},{},{}",
                result.request_index,
                result.mutation_description,
                escape_csv_value(&result.request),
                escape_csv_value(&result.response),
                result.response_time,
                result.framework.as_deref().unwrap_or("")
            )
            .unwrap();
        }

        csv_output
    }
}

// Helper function to escape CSV values
fn escape_csv_value(value: &str) -> String {
    let mut escaped = String::new();
    let mut in_quotes = false;

    for c in value.chars() {
        match c {
            '"' => {
                escaped.push('"');
                escaped.push('"'); // CSV standard: double up quotes
            }
            ',' | '\n' | '\r' => {
                if !in_quotes {
                    escaped.push('"');
                    in_quotes = true;
                }
                escaped.push(c);
            }
            _ => {
                if in_quotes {
                    escaped.push('"');
                    in_quotes = false;
                }
                escaped.push(c);
            }
        }
    }

    if in_quotes {
        escaped.push('"');
    }

    escaped
}

fn get_formatter(output_format: &str) -> Arc<dyn OutputFormatter + Send + Sync> {
    match output_format {
        "json" => Arc::new(JsonFormatter),
        "csv" => Arc::new(CsvFormatter),
        _ => Arc::new(JsonFormatter),
    }
}
