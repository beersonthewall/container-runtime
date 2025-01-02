use crate::ctx::setup_ctx;
use crate::error::ContainerErr;
use log::debug;
use std::fs::OpenOptions;

/// Starts the container process.
pub fn start(container_id: String) -> Result<(), ContainerErr> {
    let ctx = setup_ctx()?;
    let state_dir = ctx.state_dir(&container_id);
    let fifo_path = state_dir.join("exec_fifo");

    debug!("opening FIFO");
    let _ = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&fifo_path)
        .map_err(|e| ContainerErr::Fifo(format!("err: {:?}", e)))?;
    debug!("done with fifo");

    Ok(())
}
