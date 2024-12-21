mod args;

use args::Command;
use container_runtime_lib::cmd::{
    create,
    state,
    start,
    kill,
    delete,
};
use container_runtime_lib::error::ContainerErr;
use std::env::args;

fn main() -> Result<(), ContainerErr> {
    match args::parse_args(args())? {
        Command::Create {
            container_id,
            bundle_path,
        } => create(container_id, bundle_path)?,
	Command::State { container_id } => state(container_id)?,
	Command::Start { container_id } => start(container_id)?,
	Command::Kill { container_id, signal } => kill(container_id, signal)?,
	Command::Delete { container_id } => delete(container_id)?,
    }
    Ok(())
}
