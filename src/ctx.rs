use std::path::PathBuf;

const BASE_DIR: &'static str = "/var/run/generic_brand_container_runtime";

/// Container runtime settings
pub struct Ctx {
    root_dir: PathBuf,
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from(BASE_DIR),
        }
    }
}
