use super::config::Config;
use super::ctx::Ctx;
use super::error::ContainerErr;
use super::state::State;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct Container {
    state: State,
    config: Config,
}

impl Container {
    pub fn new(container_id: String, bundle_path: PathBuf, config: Config) -> Self {
        Self {
            state: State::new(container_id, bundle_path, config.oci_version.clone()),
            config,
        }
    }

    pub fn state(&self) -> &State {
	&self.state
    }

    /// Writes container state to <ctx.state_dir>/<container_id>/state.json
    pub fn write_state(&self, ctx: &Ctx) -> Result<(), ContainerErr> {
        let raw_state =
            serde_json::to_string(&self.state).map_err(|e| ContainerErr::State(e.to_string()))?;
        let container_dir = ctx.state_dir.join(self.state.id());
        let container_state_path = container_dir.join("state.json");

        if let Err(_) = fs::metadata(container_dir) {
            fs::create_dir(ctx.state_dir.join(self.state.id())).map_err(|e| ContainerErr::IO(e))?;
        }

        let mut f = OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(container_state_path)
            .map_err(|e| ContainerErr::IO(e))?;

        f.write_all(raw_state.as_bytes())
            .map_err(|e| ContainerErr::IO(e))?;
        Ok(())
    }

    /// Checks if the container state already exists on the filesystem
    pub fn exists(&self, ctx: &Ctx) -> bool {
        fs::metadata(ctx.state_path_for(&self.state.id())).is_ok()
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
