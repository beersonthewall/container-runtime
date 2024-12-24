use crate::error::ContainerErr;
use crate::config::Config;

/// Entrypoint for container initialization.
fn init(container_id: String, bundle_path: String) -> Result<(), ContainerErr> {
    log!("start initializing {}", container_id);

    let config = Config::load(bundle_path)?;

    log!("done initializing {}", container_id);
}
