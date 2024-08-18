use clap::ArgMatches;
use lazy_static::lazy_static;
use serde::Serialize;
use std::fmt::Write as FmtWrite;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::sync::Mutex;

pub struct Logger {
    verbose: bool,
    output_file: Option<Mutex<io::BufWriter<std::fs::File>>>,
    include_framework: bool,
}

#[derive(Serialize)]
pub struct JSONEntry {
    request_index: u32,
    request: String,
    response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    framework: Option<String>,
}

impl Logger {
    pub fn init(verbose: bool, output_file: Option<&str>, include_framework: bool) {
        let mut logger = GLOBAL_LOGGER.lock().unwrap();
        logger.verbose = verbose;
        logger.include_framework = include_framework;

        if let Some(file_path) = output_file {
            let file = OpenOptions::new().create(true).append(true).open(file_path).expect("Failed to open log file");
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

    pub fn print_json(&self, entry: &JSONEntry) {
        let json_entry = serde_json::to_string(entry).expect("Failed to serialize log entry");
        if let Some(ref output_file) = self.output_file {
            let mut writer = output_file.lock().unwrap();
            writeln!(writer, "{}", json_entry).expect("Failed to write to log file");
        } else {
            println!("{}", json_entry);
        }
    }

    pub fn create_json_entry(&self, request_index: u32, request: &str, response: &str, framework: Option<&str>) -> JSONEntry {
        JSONEntry {
            request_index,
            request: request.to_string(),
            response: response.to_string(),
            framework: if self.include_framework { framework.map(|s| s.to_string()) } else { None },
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
                // Single/Multiple value arg
                if let Ok(Some(values)) = matches.try_get_many::<String>(id_str) {
                    let values: Vec<String> = values.map(|v| v.to_string()).collect();
                    if values.len() == 1 {
                        writeln!(output, "  {}: {:?}", id_str, values[0]).unwrap();
                    } else {
                        writeln!(output, "  {}: {:?}", id_str, values).unwrap();
                    }
                }
            } else if matches.get_flag(id_str) {
                // Flag
                writeln!(output, "  {}: true", id_str).unwrap();
            } else {
                writeln!(output, "  {}: false", id_str).unwrap();
            }
        }

        // Output handling
        if let Some(ref output_file) = self.output_file {
            let mut writer = output_file.lock().unwrap();
            writeln!(writer, "{}", output).expect("Failed to write to log file");
        } else {
            print!("{}", output);
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Logger {
            verbose: false,
            output_file: None,
            include_framework: false,
        }
    }
}

lazy_static! {
    pub static ref GLOBAL_LOGGER: Mutex<Logger> = Mutex::new(Logger::default());
}

#[macro_export]
macro_rules! log_json {
    ($count:expr, $request:expr, $response:expr $(, $framework:expr)?) => {
        let framework: Option<String> = None;
        $(
            let framework = Some($framework.to_string());
        )?

        let entry = $crate::logger::GLOBAL_LOGGER.lock().unwrap().create_json_entry(
            $count,
            $request,
            $response,
            framework.as_deref()
        );
        $crate::logger::GLOBAL_LOGGER.lock().unwrap().print_json(&entry);
    };
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
macro_rules! log_print_args {
    ($matches:expr) => {
        $crate::logger::GLOBAL_LOGGER.lock().unwrap().print_args($matches);
    };
}
