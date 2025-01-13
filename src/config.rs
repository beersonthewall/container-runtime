use crate::error::ContainerErr;
use log::debug;
use serde::{self, Deserialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

/// A container's config.json
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
pub struct Config {
    pub oci_version: String,
    pub root: Root,
    mounts: Option<Vec<Mount>>,
    process: Process,

    // Hostname
    // https://github.com/opencontainers/runtime-spec/blob/main/config.md#hostname
    hostname: Option<String>,

    // Domainname
    // https://github.com/opencontainers/runtime-spec/blob/main/config.md#domainname
    domainname: Option<String>,

    linux: Option<Linux>,

    hooks: Option<Hooks>,
}

impl Config {
    /// Reads config.json from the bundle_path, and parses the json
    pub fn load<P: AsRef<Path>>(bundle_path: P) -> Result<Self, ContainerErr> {
        debug!("loading config.json");
        // Get path to config.json
        let mut pb = PathBuf::new();
        pb.push(bundle_path);
        pb.push("config.json");

        let mut f = File::open(pb).map_err(|e| ContainerErr::Bundle(e.to_string()))?;
        let mut buf = String::new();
        let _ = f
            .read_to_string(&mut buf)
            .map_err(|e| ContainerErr::Bundle(e.to_string()))?;
        let config: Self =
            serde_json::from_str(&buf).map_err(|e| ContainerErr::Bundle(e.to_string()))?;
        if !config.valid_spec() {
            return Err(ContainerErr::Bundle(String::new()));
        }

        debug!("config.json loaded");
        Ok(config)
    }

    pub fn linux_namespaces(&self) -> Option<&[Namespace]> {
        if let Some(linux) = &self.linux {
            Some(&linux.namespaces)
        } else {
            None
        }
    }

    pub fn cgroup_memory(&self) -> Option<&Memory> {
        if let Some(linux) = &self.linux {
            if let Some(resources) = &linux.resources {
                if let Some(memory) = &resources.memory {
                    return Some(&memory);
                }
            }
        }
        None
    }

    pub fn cgroup_cpu(&self) -> Option<&Cpu> {
        if let Some(linux) = &self.linux {
            if let Some(resources) = &linux.resources {
                if let Some(cpu) = &resources.cpu {
                    return Some(&cpu);
                }
            }
        }
        None
    }

    pub fn blockio(&self) -> Option<&BlockIO> {
        if let Some(linux) = &self.linux {
            if let Some(resources) = &linux.resources {
                if let Some(block_io) = &resources.block_io {
                    return Some(&block_io);
                }
            }
        }
        None
    }

    pub fn hugepage_limits(&self) -> Option<&[HugePageLimits]> {
        if let Some(linux) = &self.linux {
            if let Some(resources) = &linux.resources {
                if let Some(hpl) = &resources.hugepage_limits {
                    return Some(&hpl);
                }
            }
        }
        None
    }

    pub fn rdma(&self) -> Option<std::collections::hash_map::Iter<'_, String, Rdma>> {
        if let Some(linux) = &self.linux {
            if let Some(resources) = &linux.resources {
                if let Some(rdma) = &resources.rdma {
                    return Some(rdma.iter());
                }
            }
        }
        None
    }

    pub fn pids(&self) -> Option<&Pids> {
        if let Some(linux) = &self.linux {
            if let Some(resources) = &linux.resources {
                if let Some(p) = &resources.pids {
                    return Some(&p);
                }
            }
        }
        None
    }

    pub fn mounts(&self) -> Option<&[Mount]> {
        if let Some(mounts) = &self.mounts {
            return Some(mounts);
        }
        None
    }

    pub fn cgroups_path(&self) -> Option<&str> {
        if let Some(linux) = &self.linux {
            if let Some(path) = &linux.cgroups_path {
                return Some(&path);
            }
        }
        None
    }

    pub fn process(&self) -> &Process {
        &self.process
    }

    fn valid_spec(&self) -> bool {
        let cwd = Path::new(&self.process.cwd);
        cwd.is_absolute()
    }
}

/// Root configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#root
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
pub struct Root {
    pub path: String,
    pub readonly: bool,
}

/// Mount configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#mounts
#[serde(rename_all = "camelCase")]
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
pub struct Mount {
    pub destination: String,
    pub source: Option<String>,
    pub options: Option<Vec<String>>,
    #[serde(rename="type")]
    pub typ: Option<String>,
    pub uid_mappings: Option<Vec<String>>,
    pub gid_mappings: Option<Vec<String>>,
}

/// Process configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#mounts
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
pub struct Process {
    pub terminal: bool,
    console_size: Option<ConsoleSize>,
    pub cwd: String,
    pub env: Option<Vec<String>>,
    pub args: Option<Vec<String>>,
    pub command_line: Option<String>,
    user: User,

    // POSIX process fields
    pub rlimits: Option<Vec<RLimit>>,

    // Linux process fields
    pub apparmor_profile: Option<String>,
    //capabilities: todo
    //no_new_privileges: bool,
    pub oom_score_adj: Option<isize>,
    scheduler: Option<LinuxScheduler>,
    pub selinux_label: Option<String>,
    pub io_priority: Option<LinuxIOPriority>,

    #[serde(rename = "execCPUAffinity")]
    exec_cpu_affinity: Option<ExecCPUAffinity>,
}

/// POSIX process resource limit
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#posix-process
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
pub struct RLimit {
    #[serde(rename = "type")]
    pub typ: String,
    pub soft: u64,
    pub hard: u64,
}

/// Console Size configuration
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
struct ConsoleSize {
    height: usize,
    width: usize,
}

/// A Process' user configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#user
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
struct User {
    uid: isize,
    gid: isize,
    umask: Option<isize>,
    additional_gids: Option<Vec<isize>>,
}

// Linux platform structs

// Linux platform specific configuration
// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#linux-container-configuration
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
struct Linux {
    namespaces: Vec<Namespace>,
    uid_mapings: Option<Vec<UidMapping>>,
    time_offsets: Option<HashMap<String, TimeOffsets>>,
    devices: Option<Vec<Device>>,
    cgroups_path: Option<String>,
    resources: Option<Resources>,
}

/// Linux process configuration for the scheduler
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#linux-process
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
struct LinuxScheduler {
    policy: String,
    nice: i32,
    prority: i32,
    flags: Option<Vec<String>>,
    runtime: Option<u64>,
    deadline: Option<u64>,
    period: Option<u64>,
}

/// Linux process exec CPU affinity
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#linux-process
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
struct ExecCPUAffinity {
    initial: Option<String>,
    #[serde(rename = "final")]
    fnl: Option<String>,
}

/// Linux process IO priority configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#linux-process
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
pub struct LinuxIOPriority {
    pub class: String,
    pub priority: i32,
}

/// Linux Namespace configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#namespaces
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
pub struct Namespace {
    // TODO: make this an enum?
    #[serde(rename = "type")]
    pub typ: String,
    pub path: Option<String>,
}

/// User namespace mappings
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#user-namespace-mappings
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
struct UidMapping {
    #[serde(rename = "containerID")]
    container_id: u32,

    #[serde(rename = "hostID")]
    host_id: u32,

    size: u32,
}

/// Offset for Time Namespace
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#offset-for-time-namespace
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
struct TimeOffsets {
    secs: i64,
    nanosecs: u32,
}

/// Linux device configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#devices
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
struct Device {
    #[serde(rename = "type")]
    typ: String,
    path: String,
    major: Option<i64>,
    minor: Option<i64>,
    file_mode: Option<u32>,
    uid: Option<u32>,
    gid: Option<u32>,
}

// Hooks structs

/// POSIX platform hooks
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#posix-platform-hooks
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
struct Hooks {
    prestart: Option<Vec<Hook>>,
    create_runtime: Option<Vec<Hook>>,
    create_container: Option<Vec<Hook>>,
    start_container: Option<Vec<Hook>>,
    poststart: Option<Vec<Hook>>,
    poststop: Option<Vec<Hook>>,
}

/// A single Hook configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#posix-platform-hooks
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
struct Hook {
    path: String,
    args: Option<Vec<String>>,
    env: Option<Vec<String>>,
    timeout: Option<usize>,
}

/// Cgroup resource configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#cgroup-ownership
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
struct Resources {
    memory: Option<Memory>,
    devices: Option<Vec<AllowedDevice>>,
    cpu: Option<Cpu>,
    block_io: Option<BlockIO>,
    hugepage_limits: Option<Vec<HugePageLimits>>,
    network: Option<Network>,
    pids: Option<Pids>,
    rdma: Option<HashMap<String, Rdma>>,
    /// cgroup v2 parameters
    /// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#unified
    unified: Option<HashMap<String, String>>,
}

/// cgroup subsystem memory
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#memory
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
pub struct Memory {
    pub limit: Option<i64>,
    pub reservation: Option<i64>,
    pub swap: Option<i64>,
    pub kernel: Option<i64>,
    #[serde(rename = "kernelTCP")]
    pub kernel_tcp: Option<i64>,
    pub swappiness: Option<u64>,
    #[serde(rename = "disableOOMKiller")]
    pub disable_oom_killer: Option<bool>,
    pub use_hierarchy: Option<bool>,
    pub check_before_update: Option<bool>,
}

/// cgroup allowed devices
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#allowed-device-list
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
struct AllowedDevice {
    allow: bool,
    #[serde(rename = "type")]
    typ: Option<DeviceType>,
    major: Option<i64>,
    minor: Option<i64>,
    access: Option<String>,
}

#[derive(Clone, Deserialize, Debug)]
enum DeviceType {
    #[serde(rename = "a")]
    All,
    #[serde(rename = "c")]
    Char,
    #[serde(rename = "b")]
    Block,
}

/// cgroup subsystems cpu and cpusets
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#cpu
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
pub struct Cpu {
    pub shares: Option<i64>,
    pub quota: Option<i64>,
    pub burst: Option<u64>,
    pub period: Option<u64>,
    pub realtime_runtime: Option<i64>,
    pub realtime_period: Option<u64>,
    pub cpus: Option<String>,
    pub mems: Option<String>,
    pub idle: Option<i64>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
pub struct BlockIO {
    pub weight: Option<u16>,
    pub leaf_weight: Option<u16>,
    pub weight_device: Option<Vec<WeightDevice>>,

    pub throttle_read_bps_device: Option<Vec<DevThrottle>>,
    pub throttle_write_bps_device: Option<Vec<DevThrottle>>,

    pub throttle_read_iops_device: Option<Vec<DevThrottle>>,
    pub throttle_write_iops_device: Option<Vec<DevThrottle>>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
pub struct WeightDevice {
    pub major: i64,
    pub minor: i64,
    pub weight: Option<u16>,
    pub leaf_weight: Option<u16>,
}

#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
pub struct DevThrottle {
    pub major: i64,
    pub minor: i64,
    pub rate: u64,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
pub struct HugePageLimits {
    pub page_size: String,
    pub limit: u64,
}

/// cgroup subsystem network
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#network
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
struct Network {
    class_id: Option<u32>,
    priorities: Option<Vec<Prio>>,
}

#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
struct Prio {
    name: String,
    priority: u32,
}

/// cgroup subsystem pids
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#pids
#[derive(Clone, Deserialize, Debug)]
#[repr(C)]
pub struct Pids {
    pub limit: i64,
}

/// cgroup subsystem rdma
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#rdma
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
pub struct Rdma {
    pub hca_handles: Option<u32>,
    pub hca_objects: Option<u32>,
}
