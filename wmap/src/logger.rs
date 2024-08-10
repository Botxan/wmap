use clap::ArgMatches;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct Logger {
    verbose: bool,
    output_file: Option<File>,
}

impl Logger {
    pub fn new(verbose: bool, output: Option<&str>) -> Logger {
        let output_file = output.map(|path| File::create(Path::new(path)).expect("Failed to create output file"));
        Logger { verbose, output_file }
    }

    pub fn print(&mut self, message: &str) {
        self.print_to_target(format_args!("{}", message));
    }

    pub fn print_verbose(&mut self, message: &str) {
        if self.verbose {
            self.print(message);
        }
    }

    pub fn print_args(&mut self, matches: &ArgMatches) {
        if self.verbose {
            self.print_formatted_args(matches);
        }
    }

    pub fn print_format(&mut self, format: std::fmt::Arguments) {
        self.print_to_target(format);
    }

    fn print_to_target(&mut self, format: std::fmt::Arguments) {
        if let Some(ref mut file) = self.output_file {
            writeln!(file, "{}", format).expect("Failed to write to output file");
        } else {
            println!("{}", format);
        }
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    fn print_formatted_args(&mut self, matches: &clap::ArgMatches) {
        self.print("Command line arguments:");

        if let Some(url) = matches.get_one::<String>("url") {
            self.print_format(format_args!("> URL: {}", url));
        } else {
            self.print_format(format_args!("[x] No URL provided"));
        }

        if let Some(methods) = matches.get_many::<String>("methods") {
            self.print_format(format_args!("> HTTP Methods:"));
            for method in methods {
                self.print_format(format_args!("  - {}", method));
            }
        }

        if let Some(encoding) = matches.get_one::<String>("encoding") {
            self.print_format(format_args!("> Encoding: {}", encoding));
        }

        if let Some(output) = matches.get_one::<String>("output") {
            match std::fs::canonicalize(output) {
                Ok(abs_path) => self.print_format(format_args!(
                    "> Output file: {} (absolute path: {})",
                    output,
                    abs_path.display()
                )),
                Err(_) => self.print_format(format_args!("> Output file: {}", output)),
            }
        } else {
            self.print_format(format_args!("[x] No output file specified."));
        }

        if matches.get_flag("verbose") {
            self.print_format(format_args!("> Verbose mode enabled."));
        } else {
            self.print_format(format_args!("> Verbose mode disabled."));
        }
    }
}

#[macro_export]
macro_rules! log_print {
    ($logger:expr, $($arg:tt)*) => {
        $logger.print_format(format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_print_verbose {
    ($logger:expr, $($arg:tt)*) => {
        if $logger.is_verbose() {
            $logger.print_format(format_args!($($arg)*));
        }
    }
}
