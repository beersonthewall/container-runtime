use std::path::PathBuf;

const BASE_DIR: &'static str = "/var/run/generic_brand_container_runtime";

/// Container runtime settings
pub struct Ctx {
    root_dir: PathBuf,
    cgroups_root: String,
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from(BASE_DIR),
            cgroups_root: String::from("/sys/fs/cgroup"),
        }
    }
}

impl Ctx {
    pub fn cgroups_root(&self) -> &str {
        &self.cgroups_root
    }
}
