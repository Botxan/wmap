mod args;
mod logger;

use logger::Logger;

fn main() {
    let matches = args::parse_args();

    let verbose = matches.get_flag("verbose");
    let logger = Logger::new(verbose);

    // print args only if verbose
    logger.print_args(&matches);
}
