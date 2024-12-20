use crate::config::Config;
use crate::error::ContainerErr;
use std::path::PathBuf;

pub fn create(container_id: String, bundle_path: String) -> Result<(), ContainerErr> {
    let bundle_path = PathBuf::from(bundle_path);
    let config = Config::load(&bundle_path)?;
    Ok(())
}
