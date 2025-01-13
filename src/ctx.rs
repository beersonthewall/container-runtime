//! Settings/Context for the container runtime itself.

use crate::error::ContainerErr;
use log::debug;
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

pub const STATE_FILENAME: &str = "state.json";
const BASE_DIR: &str = "/run/generic_brand_container_runtime";

/// Container runtime settings
#[derive(Clone)]
pub struct Ctx {
    pub state_dir: PathBuf,
    cgroups_root: PathBuf,
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            state_dir: PathBuf::from(BASE_DIR),
            cgroups_root: PathBuf::from("/sys/fs/cgroup"),
        }
    }
}

impl Ctx {
    pub fn cgroups_root(&self) -> &Path {
        &self.cgroups_root
    }

    pub fn state_dir(&self, container_id: &str) -> PathBuf {
        self.state_dir.join(container_id)
    }

    pub fn state_path_for(&self, container_id: &str) -> PathBuf {
        self.state_dir.join(container_id).join(STATE_FILENAME)
    }
}

/// Sets up context (creates state dir if it doesn't exist)
pub fn setup_ctx() -> Result<Ctx, ContainerErr> {
    debug!("setting up context...");
    let ctx = Ctx::default();

    if let Err(e) = fs::metadata(&ctx.state_dir) {
        if e.kind() == ErrorKind::NotFound {
            debug!("state dir not found, creating...");
            fs::create_dir(&ctx.state_dir).map_err(ContainerErr::IO)?;
        } else {
            return Err(ContainerErr::IO(e));
        }
    }

    debug!("DONE: setting up context.");
    Ok(ctx)
}
