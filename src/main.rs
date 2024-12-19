mod args;

use std::env::args;

fn main() {
    let _ = args::parse_args(args()).unwrap();
}
