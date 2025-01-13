use crate::{ctx::setup_ctx, error::ContainerErr};
use log::debug;
use std::fs;

pub fn delete(container_id: String) -> Result<(), ContainerErr> {
    let ctx = setup_ctx()?;

    // Cleanup state directory
    let container_state_dir = ctx.state_dir(&container_id);
    if fs::metadata(&container_state_dir).is_ok() {
        debug!("deleting state directory");
        fs::remove_dir_all(&container_state_dir).map_err(ContainerErr::IO)?;
    }

    // Cleanup cgroup
    let cgroup_path = ctx.cgroups_root().join(&container_id);
    if fs::metadata(&cgroup_path).is_ok() {
        debug!("cleaning up cgroup",);
        fs::remove_dir(&cgroup_path).map_err(ContainerErr::IO)?;
    }

    Ok(())
}
