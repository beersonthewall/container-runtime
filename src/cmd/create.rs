use crate::config::Config;
use crate::error::ContainerErr;
use crate::container::Container;
use std::path::PathBuf;

/// Creates a new container from the OCI bundle located at bundle_path
pub fn create(container_id: String, bundle_path: String) -> Result<(), ContainerErr> {
    let bundle_path = PathBuf::from(bundle_path);
    let config = Config::load(&bundle_path)?;
    let _ = Container::create(container_id, config)?;
    Ok(())
}
