use clap::{command, Arg, ArgAction};

pub fn parse_args() -> clap::ArgMatches {
    command!()
        .about("HTTP request fuzzing CLI tool for web framework detection")
        .arg(Arg::new("url").short('u').long("url").help("Target URL").required(true))
        .arg(Arg::new("input").short('i').long("input").help("Input file with list of URLs").conflicts_with("url"))
        .arg(
            Arg::new("methods")
                .short('m')
                .long("methods")
                .help("List of HTTP methods to mutate")
                .value_delimiter(',')
                .value_parser(["GET", "HEAD", "POST", "PUT", "DELETE", "CONNECT", "OPTIONS", "TRACE", "PATCH"])
                .default_values(["GET"]),
        )
        .arg(
            Arg::new("encoding")
                .short('e')
                .long("encoding")
                .help("Encoding to be used in mutations")
                .value_parser(["ASCII", "UTF-8"])
                .default_value("ASCII"),
        )
        .arg(Arg::new("output").short('o').long("output").help("Output file"))
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Display additional information")
                .action(ArgAction::SetTrue),
        )
        .get_matches()
}
