use clap::ArgMatches;

pub struct Logger {
    verbose: bool,
}

impl Logger {
    pub fn new(verbose: bool) -> Logger {
        Logger { verbose }
    }

    pub fn print(&self, message: &str) {
        println!("{}", message);
    }

    pub fn print_verbose(&self, message: &str) {
        if self.verbose {
            println!("{}", message);
        }
    }

    pub fn print_args(&self, matches: &ArgMatches) {
        if self.verbose {
            self.print_formatted_args(matches);
        }
    }

    fn print_formatted_args(&self, matches: &clap::ArgMatches) {
        // URL
        if let Some(url) = matches.get_one::<String>("url") {
            println!("URL: {}", url);
        } else {
            println!("No URL provided");
        }

        // Methods
        if let Some(methods) = matches.get_many::<String>("methods") {
            println!("HTTP Methods:");
            for method in methods {
                println!("  - {}", method);
            }
        }

        // Encoding
        if let Some(encoding) = matches.get_one::<String>("encoding") {
            println!("Encoding: {}", encoding);
        }

        // Output file
        if let Some(output) = matches.get_one::<String>("output") {
            // Convert to absolute path if possible
            match std::fs::canonicalize(output) {
                Ok(abs_path) => println!(
                    "Output file: {} (absolute path: {})",
                    output,
                    abs_path.display()
                ),
                Err(_) => println!("Output file: {}", output), // If it fails, print it as introduced by the user
            }
        } else {
            println!("No output file specified.");
        }

        // Verbose
        if matches.get_flag("verbose") {
            println!("Verbose mode enabled.");
        } else {
            println!("Verbose mode disabled.");
        }
    }
}
