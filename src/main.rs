mod args;

use std::env::args;
use args::Command;
use container_runtime_lib::cmd::create;

fn main() {
    match args::parse_args(args()).unwrap() {
	Command::Create{ container_id, bundle_path } => create(container_id, bundle_path),
	_ => todo!("implement")
    }
}
