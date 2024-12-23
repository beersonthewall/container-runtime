//! Functions for manipulating cgroups
//! https://www.kernel.org/doc/Documentation/cgroup-v2.txt

use crate::error::ContainerErr;
use std::path::{Path, PathBuf};
use std::fs::File;

/// Creates a cgroup at the provided path.
/// Assumes this directory does not exist and will Err if it does.
pub fn create_cgroup<P: AsRef<Path>>(cgroup_path: P) -> Result<(), ContainerErr> {
    std::fs::create_dir(&cgroup_path).map_err(|e| ContainerErr::IO(e))?;

    // create the necessary files
    let filenames = ["cgroup.procs"];
    for f in filenames {
	let mut pb = PathBuf::new();
	pb.push(&cgroup_path);
	pb.push(f);
	let _ = File::create(pb).map_err(|e| ContainerErr::IO(e))?;
    }

    // TODO: apply settings from config

    Ok(())
}

/// Resolves the cgroup path from cgroups_path set in the config defaulting
/// to /sys/fs/cgroup/container_runtime/<container_id>
pub fn resolve_cgroup_path<P: AsRef<Path>>(config_cgroups_path: Option<P>,
					   cgroups_root: P,
					   container_id: &str) -> PathBuf {
    let mut pb = PathBuf::new();
    match config_cgroups_path {
        Some(path) => {
            pb.push(cgroups_root);
	    // If the path is absolute we're required by oci spec to treat this as
	    // relative to the cgroup mount point. We need drop the '/' prefix to get PathBuf
	    // to behave. If you don't it drops anything already in the buffer
	    // when pushing an absolute path.
	    if path.as_ref().is_absolute() {
		pb.push(path.as_ref().strip_prefix("/").unwrap());
	    } else {
		// If the path is relative we _may_ interpret this as relative to a
		// runtime-determined location. I chose to put this as relative to
		// the cgroup mount point anyway.
		pb.push(path);
	    }
            pb
        }
        None => {
            pb.push(cgroups_root);
            pb.push(container_id);
            pb
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_cgroup_path() {
	// Aboslute paths should be treated as relative to '/sys/fs/cgroup'
	let result = resolve_cgroup_path(Some("/myruntime/mycontainer"), "/sys/fs/cgroup", "test-container");
	assert_eq!(PathBuf::from("/sys/fs/cgroup/myruntime/mycontainer"), result);

	// Relative paths will also be treated that way, but the runtime may chose to
	// put it elsewhere.
	let result = resolve_cgroup_path(Some("myruntime/mycontainer"), "/sys/fs/cgroup", "test-container");
	assert_eq!(PathBuf::from("/sys/fs/cgroup/myruntime/mycontainer"), result);

	// If it's not provided we get to pick. We chose to use the container id as cgroup name.
	let result = resolve_cgroup_path(None, "/sys/fs/cgroup", "test-container");
	assert_eq!(PathBuf::from("/sys/fs/cgroup/test-container"), result);
    }

    #[test]
    fn test_create_cgroup() {
	use std::time::{ SystemTime, UNIX_EPOCH };
	use std::fs::metadata;

	let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
	let dir = format!("foo_{}", time);
	let mut procs_file = PathBuf::from(&dir);
	procs_file.push("cgroup.procs");

	let result = create_cgroup(&dir);
	assert!(result.is_ok(), "{:?}", result);
	let metadata = metadata(&procs_file);
	if let Err(e) = metadata {
	    println!("{:?}", &procs_file);
	    assert!(false, "error checking cgroup.procs: {:?}", e);
	}

	// try to cleanup
	std::fs::remove_file(&procs_file).unwrap();
	std::fs::remove_dir(&dir).unwrap();
    }
}
