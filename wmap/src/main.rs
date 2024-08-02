mod args;

fn main() {
    let matches = args::parse_args();
    args::print_args(&matches);
}
