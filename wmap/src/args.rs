use clap::{command, Arg, ArgAction};

pub fn parse_args() -> clap::ArgMatches {
    command!()
        .about("HTTP request fuzzing CLI tool for web framework detection")
        .arg(Arg::new("url").short('u').long("url").value_name("URL or URL;FRAMEWORK").help("Target URL").required(true))
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Input file with list of URLs")
                .conflicts_with("url")
                .required(true),
        )
        .arg(
            Arg::new("methods")
                .short('m')
                .long("methods")
                .value_name("METHOD_1,METHOD_2,...")
                .help("List of HTTP methods to mutate")
                .value_delimiter(',')
                .value_parser(["GET", "HEAD", "POST", "PUT", "DELETE", "CONNECT", "OPTIONS", "TRACE", "PATCH"])
                .default_values(["GET"]),
        )
        .arg(Arg::new("output").short('o').long("output").value_name("FILE").help("Write output to a file"))
        .arg(
            Arg::new("output-format")
                .long("output-format")
                .short('f')
                .value_name("FORMAT")
                .help("Specifiy the output format")
                .value_parser(["json", "csv"])
                .default_value("json"),
        )
        .arg(Arg::new("verbose").short('v').long("verbose").help("Display additional information").action(ArgAction::SetTrue))
        .get_matches()
}
