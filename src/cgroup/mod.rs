//! Functions for manipulating cgroups
//! https://www.kernel.org/doc/Documentation/cgroup-v2.txt

mod util;

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use libc::{c_char, statfs};
use util::{read_flat_keyed_file, read_nested_keyed_file, write_nested_keyed_file};

use crate::config::{BlockIO, Config, Cpu, DevThrottle, HugePageLimits, Memory, Pids, Rdma};
use crate::error::ContainerErr;

#[derive(Debug, Eq, PartialEq)]
pub enum CgroupVersion {
    V1,
    V2,
    Hybrid,
}

/// Attempts to detect which cgroup version is being used
pub fn detect_cgroup_version<P: AsRef<Path>>(
    mount_point: P,
) -> Result<CgroupVersion, ContainerErr> {
    let mount_point = mount_point.as_ref().as_os_str().as_bytes().to_vec();
    let mut statfs = unsafe { std::mem::zeroed::<statfs>() };
    let err = unsafe { libc::statfs(mount_point.as_ptr() as *const c_char, &mut statfs) };
    if err < 0 {
        return Err(ContainerErr::Cgroup(String::from(
            "Cgroup mount at /sys/fs/cgroup not found.",
        )));
    }

    match statfs.f_type {
        libc::CGROUP2_SUPER_MAGIC => Ok(CgroupVersion::V2),
        libc::CGROUP_SUPER_MAGIC => Err(ContainerErr::Cgroup(String::from(
            "Cgroup v1 or hybrid not supported",
        ))),
        _ => Err(ContainerErr::Cgroup(String::from(
            "/sys/fs/cgroup mount has an unsupported f_type",
        ))),
    }
}

/// Creates a cgroup at the provided path.
/// Assumes this directory does not exist and will Err if it does.
pub fn create_cgroup<P: AsRef<Path>>(cgroup_path: P, config: &Config) -> Result<(), ContainerErr> {
    std::fs::create_dir(&cgroup_path).map_err(|e| ContainerErr::IO(e))?;

    // create the necessary files
    let filenames = ["cgroup.procs"];
    for f in filenames {
        let mut pb = PathBuf::new();
        pb.push(&cgroup_path);
        pb.push(f);
        let _ = File::create(pb).map_err(|e| ContainerErr::IO(e))?;
    }

    if let Some(memory) = config.cgroup_memory() {
        set_cgroup_memory(&cgroup_path, memory)?;
    }

    if let Some(cpu) = config.cgroup_cpu() {
        set_cgroup_cpu(&cgroup_path, cpu)?;
    }

    if let Some(blockio) = config.blockio() {
        set_cgroup_blockio(&cgroup_path, blockio)?;
    }

    if let Some(hpl) = config.hugepage_limits() {
        set_cgroup_hugepage(&cgroup_path, hpl)?;
    }

    if let Some(rdma) = config.rdma() {
        set_cgroup_rdma(&cgroup_path, rdma)?;
    }

    if let Some(pids) = config.pids() {
        set_cgroup_pids(&cgroup_path, pids)?;
    }
    Ok(())
}

/// Resolves the cgroup path from cgroups_path set in the config defaulting
/// to /sys/fs/cgroup/container_runtime/<container_id>
pub fn resolve_cgroup_path<P: AsRef<Path>>(
    config_cgroups_path: Option<P>,
    cgroups_root: P,
    container_id: &str,
) -> PathBuf {
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

/// Write values from cgroup memory config into the appropriate files
fn set_cgroup_memory<P: AsRef<Path>>(cgroup: P, memory: &Memory) -> Result<(), ContainerErr> {
    let mut current = String::new();
    //File::read_to_string("memory.current", &current).map_err(|e| ContainerErr::IO(e))?;

    if let Some(val) = memory.limit {
        write_to_cgroup_file(val.to_string().as_bytes(), &cgroup, "memory.limit")?;
    }

    // FIXME: is this memory.low for cgroups v2? Which is the version I'm coding against
    // accidentally read v1 docs for filenames.... oops
    if let Some(val) = memory.reservation {
        write_to_cgroup_file(
            val.to_string().as_bytes(),
            &cgroup,
            "memory.soft_limit_in_bytes",
        )?;
    }

    if let Some(val) = memory.swap {
        write_to_cgroup_file(val.to_string().as_bytes(), &cgroup, "memory.swap.max")?;
    }

    if let Some(val) = memory.swappiness {
        write_to_cgroup_file(val.to_string().as_bytes(), &cgroup, "memory.swappiness")?;
    }

    if let Some(val) = memory.disable_oom_killer {
        let toggle = if val { b"1" } else { b"0" };
        write_to_cgroup_file(toggle, &cgroup, "memory.oom_control")?;
    }

    if let Some(val) = memory.use_hierarchy {
        let toggle = if val { b"1" } else { b"0" };
        write_to_cgroup_file(toggle, &cgroup, "memory.use_hierarchy")?;
    }

    Ok(())
}

fn set_cgroup_cpu<P: AsRef<Path>>(cgroup: P, cpu: &Cpu) -> Result<(), ContainerErr> {
    if let Some(val) = cpu.burst {
        write_to_cgroup_file(val.to_string().as_bytes(), &cgroup, "cpu.max.burst")?;
    }
    Ok(())
}

/// Writes information for the IO controller
/// https://docs.kernel.org/admin-guide/cgroup-v2.html#io
fn set_cgroup_blockio<P: AsRef<Path>>(cgroup: P, blockio: &BlockIO) -> Result<(), ContainerErr> {
    if let Some(weight) = blockio.weight {
        let io_weight_path = cgroup.as_ref().join("io.weight");
        let mut data = read_flat_keyed_file(&io_weight_path)?;

        if let Some(weight_devices) = &blockio.weight_device {
            for device in weight_devices {
                if let Some(device_weight) = device.weight {
                    let key = format!("{}:{}", device.major, device.minor);
                    data.insert(key, device_weight.to_string());
                }
            }
        }

        data.insert(String::from("default"), weight.to_string());
        util::write_flat_keyed_file(&io_weight_path, data)?;
    }

    let io_max_path = cgroup.as_ref().join("io.max");
    let mut io_max = read_nested_keyed_file(&io_max_path)?;

    if let Some(throttle_read_bps_device) = &blockio.throttle_read_bps_device {
        update_device(throttle_read_bps_device, "rbps", &mut io_max);
    }

    if let Some(throttle_write_bps_device) = &blockio.throttle_write_bps_device {
        update_device(throttle_write_bps_device, "wbps", &mut io_max);
    }

    if let Some(throttle_read_iops_device) = &blockio.throttle_read_iops_device {
        update_device(throttle_read_iops_device, "riops", &mut io_max);
    }

    if let Some(throttle_write_iops_device) = &blockio.throttle_write_iops_device {
        update_device(throttle_write_iops_device, "wiops", &mut io_max);
    }

    write_nested_keyed_file(&io_max_path, io_max)?;

    Ok(())
}

fn update_device(
    dev_list: &[DevThrottle],
    subkey: &str,
    file_map: &mut HashMap<String, HashMap<String, String>>,
) {
    for dev in dev_list {
        if let Some(dev_entry) = file_map.get_mut(&format!("{}:{}", dev.major, dev.minor)) {
            dev_entry.insert(String::from("rbps"), dev.rate.to_string());
        } else {
            let mut dev_entry = HashMap::new();
            dev_entry.insert(String::from(subkey), dev.rate.to_string());
            file_map.insert(format!("{}:{}", dev.major, dev.minor), dev_entry);
        }
    }
}

/// Writes information for the hugetlb controller
/// https://docs.kernel.org/admin-guide/cgroup-v2.html#hugetlb
fn set_cgroup_hugepage<P: AsRef<Path>>(
    cgroup: P,
    limits: &[HugePageLimits],
) -> Result<(), ContainerErr> {
    for hp in limits {
        let hp_path = cgroup
            .as_ref()
            .join(format!("hugepage.{}.max", hp.page_size));
        let mut f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(hp_path)
            .map_err(|e| ContainerErr::IO(e))?;
        f.write_all(hp.limit.to_string().as_bytes())
            .map_err(|e| ContainerErr::IO(e))?;
    }
    Ok(())
}

/// https://docs.kernel.org/admin-guide/cgroup-v2.html#rdma
fn set_cgroup_rdma<P: AsRef<Path>>(
    cgroup: P,
    rdma: std::collections::hash_map::Iter<String, Rdma>,
) -> Result<(), ContainerErr> {
    let mut rdma_data = read_nested_keyed_file(cgroup.as_ref().join("rdma.max"))?;
    for (key, rdma_cfg) in rdma {
        let sub_map = if let Some(sub_map) = rdma_data.get_mut(key) {
            sub_map
        } else {
            let sub_map = HashMap::new();
            rdma_data.insert(key.clone(), sub_map);
            rdma_data.get_mut(key).unwrap()
        };

        if let Some(h) = rdma_cfg.hca_handles {
            sub_map.insert(String::from("hca_handle"), h.to_string());
        }
        if let Some(o) = rdma_cfg.hca_objects {
            sub_map.insert(String::from("hca_object"), o.to_string());
        }
    }
    Ok(())
}

/// Writes max pids
/// https://docs.kernel.org/admin-guide/cgroup-v2.html#pid
fn set_cgroup_pids<P: AsRef<Path>>(cgroup: P, pids: &Pids) -> Result<(), ContainerErr> {
    let mut f = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(cgroup.as_ref().join("pids.max"))
        .map_err(|e| ContainerErr::IO(e))?;

    f.write_all(pids.limit.to_string().as_bytes())
        .map_err(|e| ContainerErr::IO(e))?;
    Ok(())
}

fn write_to_cgroup_file<P: AsRef<Path>, F: AsRef<Path>>(
    bytes: &[u8],
    cgroup: P,
    filepath: F,
) -> Result<(), ContainerErr> {
    let mut f =
        File::create(Path::new(cgroup.as_ref()).join(filepath)).map_err(|e| ContainerErr::IO(e))?;
    f.write(bytes).map_err(|e| ContainerErr::IO(e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_cgroup_path() {
        // Aboslute paths should be treated as relative to '/sys/fs/cgroup'
        let result = resolve_cgroup_path(
            Some("/myruntime/mycontainer"),
            "/sys/fs/cgroup",
            "test-container",
        );
        assert_eq!(
            PathBuf::from("/sys/fs/cgroup/myruntime/mycontainer"),
            result
        );

        // Relative paths will also be treated that way, but the runtime may chose to
        // put it elsewhere.
        let result = resolve_cgroup_path(
            Some("myruntime/mycontainer"),
            "/sys/fs/cgroup",
            "test-container",
        );
        assert_eq!(
            PathBuf::from("/sys/fs/cgroup/myruntime/mycontainer"),
            result
        );

        // If it's not provided we get to pick. We chose to use the container id as cgroup name.
        let result = resolve_cgroup_path(None, "/sys/fs/cgroup", "test-container");
        assert_eq!(PathBuf::from("/sys/fs/cgroup/test-container"), result);
    }

    #[test]
    fn test_create_cgroup() {
        use std::fs::metadata;
        use std::time::{SystemTime, UNIX_EPOCH};

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let dir = format!("foo_{}", time);
        let mut procs_file = PathBuf::from(&dir);
        procs_file.push("cgroup.procs");

        let config = Config::load("test_configs/").expect("to load full_config_example.json");

        let result = create_cgroup(&dir, &config);
        assert!(result.is_ok(), "{:?}", result);
        let metadata = metadata(&procs_file);
        if let Err(e) = metadata {
            println!("{:?}", &procs_file);
            assert!(false, "error checking cgroup.procs: {:?}", e);
        }

        // try to cleanup
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
