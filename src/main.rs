mod args;

use args::Command;
use container_runtime_lib::cmd::create;
use container_runtime_lib::error::ContainerErr;
use std::env::args;

fn main() -> Result<(), ContainerErr> {
    match args::parse_args(args())? {
        Command::Create {
            container_id,
            bundle_path,
        } => create(container_id, bundle_path)?,
        _ => todo!("implement"),
    }
    Ok(())
}
