use clap::{command, Arg, ArgAction};

pub fn parse_args() -> clap::ArgMatches {
    command!()
        .about("HTTP request fuzzing CLI tool for web framework detection")
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .help("Target URL")
                .required(true),
        )
        .arg(
            Arg::new("methods")
                .short('m')
                .long("methods")
                .help("List of HTTP methods to mutate")
                .value_delimiter(',')
                .value_parser([
                    "GET", "HEAD", "POST", "PUT", "DELETE", "CONNECT", "OPTIONS", "TRACE", "PATCH",
                ])
                .default_values(["GET", "HEAD"]),
        )
        .arg(
            Arg::new("encoding")
                .short('e')
                .long("encoding")
                .help("Encoding to be used in mutations")
                .value_parser(["ASCII", "UTF-8"])
                .default_value("ASCII"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Display additional information")
                .action(ArgAction::SetTrue),
        )
        .get_matches()
}

pub fn print_args(matches: &clap::ArgMatches) {
    // url
    if let Some(url) = matches.get_one::<String>("url") {
        println!("URL: {}", url);
    } else {
        println!("No URL provided");
    }

    // methods
    if let Some(methods) = matches.get_many::<String>("methods") {
        println!("Methods:");
        for method in methods {
            println!("- {}", method);
        }
    } else {
        println!("No methods selected");
    }

    // encoding
    if let Some(encoding) = matches.get_one::<String>("encoding") {
        println!("Encoding: {}", encoding)
    } else {
        println!("No encoding specified");
    }

    // output
    if let Some(output) = matches.get_one::<String>("output") {
        println!("Output file: {}", output);
    } else {
        println!("No output file specified");
    }

    // verbose
    if matches.get_flag("verbose") {
        println!("Verbose mode enabled");
    } else {
        println!("Verbose mode disabled");
    }
}
