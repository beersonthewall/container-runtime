use crate::error::ContainerErr;
use serde::{self, Deserialize};
use log::debug;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

/// A container's config.json
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
pub struct Config {
    pub oci_version: String,
    root: Root,
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
    pub fn load(bundle_path: &Path) -> Result<Self, ContainerErr> {
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
        let config: Self = serde_json::from_str(&buf).map_err(|e| ContainerErr::Bundle(e.to_string()))?;
	if !config.valid_spec() {
	    return Err(ContainerErr::Bundle(String::new()))
	}

	debug!("config.json lodaded");
        Ok(config)
    }

    pub fn linux_namespaces(&self) -> Option<&[Namespace]> {
	if let Some(linux) = &self.linux {
	    Some(&linux.namespaces)
	} else {
	    None
	}
    }

    fn valid_spec(&self) -> bool {
	let cwd = Path::new(&self.process.cwd);
	cwd.is_absolute()
    }
}

/// Root configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#root
#[derive(Deserialize, Debug)]
#[repr(C)]
struct Root {
    path: String,
    readonly: bool,
}

/// Mount configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#mounts
#[derive(Deserialize)]
#[repr(C)]
struct Mount {
    destination: String,
    source: Option<String>,
    options: Option<Vec<String>>,
}

/// Process configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#mounts
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
struct Process {
    terminal: bool,
    console_size: Option<ConsoleSize>,
    cwd: String,
    env: Option<Vec<String>>,
    args: Option<Vec<String>>,
    command_line: Option<String>,
    user: User,

    // POSIX process fields
    rlimits: Option<Vec<RLimit>>,

    // Linux process fields
    apparmor_profile: Option<String>,
    //capabilities: todo
    //no_new_privileges: bool,
    oom_score_adj: Option<isize>,
    scheduler: Option<LinuxScheduler>,
    selinux_label: Option<String>,
    io_priority: Option<LinuxIOPriority>,

    #[serde(rename = "execCPUAffinity")]
    exec_cpu_affinity: Option<ExecCPUAffinity>,
}

/// POSIX process resource limit
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#posix-process
#[derive(Deserialize)]
#[repr(C)]
struct RLimit {
    #[serde(rename = "type")]
    typ: String,
    soft: u64,
    hard: u64,
}

/// Console Size configuration
#[derive(Deserialize)]
#[repr(C)]
struct ConsoleSize {
    height: usize,
    width: usize,
}

/// A Process' user configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#user
#[derive(Deserialize)]
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
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
struct Linux {
    namespaces: Vec<Namespace>,
    uid_mapings: Option<Vec<UidMapping>>,
    time_offsets: Option<TimeOffsets>,
    devices: Option<Vec<Device>>,
    cgroups_path: Option<String>,
    resources: Option<Resources>,
}

/// Linux process configuration for the scheduler
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#linux-process
#[derive(Deserialize)]
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
#[derive(Deserialize)]
#[repr(C)]
struct ExecCPUAffinity {
    initial: Option<String>,
    #[serde(rename = "final")]
    fnl: Option<String>,
}

/// Linux process IO priority configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#linux-process
#[derive(Deserialize)]
#[repr(C)]
struct LinuxIOPriority {
    class: String,
    priority: isize,
}

/// Linux Namespace configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#namespaces
#[derive(Deserialize)]
#[repr(C)]
pub struct Namespace {
    // TODO: make this an enum?
    #[serde(rename = "type")]
    pub typ: String,
    pub path: Option<String>,
}

/// User namespace mappings
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#user-namespace-mappings
#[derive(Deserialize)]
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
#[derive(Deserialize)]
#[repr(C)]
struct TimeOffsets {
    secs: i64,
    nanosecs: u32,
}

/// Linux device configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#devices
#[derive(Deserialize)]
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
#[derive(Deserialize)]
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
#[derive(Deserialize)]
#[repr(C)]
struct Hook {
    path: String,
    args: Option<Vec<String>>,
    env: Option<Vec<String>>,
    timeout: Option<usize>,
}

/// Cgroup resource configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#cgroup-ownership
#[derive(Deserialize)]
#[repr(C)]
struct Resources {
    memory: Option<Memory>,
    devices: Option<Vec<AllowedDevice>>,
    cpu: Option<Cpu>,
    block_io: Option<BlockIO>,
    hugepage_limits: Option<Vec<HugePageLimits>>,
    network: Option<Network>,
    pids: Option<Pids>,
    rdma: Option<Rdma>,
    /// cgroup v2 parameters
    /// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#unified
    unified: Option<HashMap<String, String>>,
}

/// cgroup subsystem memory
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#memory
#[derive(Deserialize)]
#[repr(C)]
struct Memory {
    limit: Option<i64>,
    reservatiion: Option<i64>,
    swap: Option<i64>,
    kernel: Option<i64>,
    #[serde(rename = "kernelTCP")]
    kernel_tcp: Option<i64>,
    swappiness: Option<u64>,
    #[serde(rename = "disableOOMKiller")]
    disable_oom_killer: bool,
    use_hierarchy: bool,
    check_before_update: bool,
}

/// cgroup allowed devices
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#allowed-device-list
#[derive(Deserialize)]
#[repr(C)]
struct AllowedDevice {
    allow: bool,
    #[serde(rename = "type")]
    typ: DeviceType,
    major: i64,
    minor: i64,
    access: String,
}

#[derive(Deserialize)]
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
#[derive(Deserialize)]
#[repr(C)]
struct Cpu {
    shares: Option<i64>,
    quota: Option<i64>,
    burst: Option<u64>,
    period: Option<u64>,
    realtime_runtime: Option<i64>,
    realtime_period: Option<u64>,
    cpus: Option<String>,
    mems: Option<String>,
    idle: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
struct BlockIO {
    weight: Option<u16>,
    leaf_weight: Option<u16>,
    weight_device: Option<Vec<WeightDevice>>,

    throttle_read_bps_device: Option<Vec<DevThrottle>>,
    throttle_write_bps_device: Option<Vec<DevThrottle>>,

    throttle_read_iops_device: Option<Vec<DevThrottle>>,
    throttle_write_iops_device: Option<Vec<DevThrottle>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
struct WeightDevice {
    major: i64,
    minor: i64,
    weight: Option<u16>,
    leaf_weight: Option<u16>
}

#[derive(Deserialize)]
#[repr(C)]
struct DevThrottle {
    major: i64,
    minor: i64,
    rate: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
struct HugePageLimits {
    page_size: String,
    limit: u64,
}

/// cgroup subsystem network
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#network
#[derive(Deserialize)]
#[repr(C)]
struct Network {
    class_id: Option<u32>,
    priorities: Option<Vec<Prio>>,
}

#[derive(Deserialize)]
#[repr(C)]
struct Prio {
    name: String,
    priority: u32,
}

/// cgroup subsystem pids
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#pids
#[derive(Deserialize)]
#[repr(C)]
struct Pids {
    limit: i64,
}

/// cgroup subsystem rdma
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#rdma
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
struct Rdma {
    hca_handles: Option<u32>,
    hca_objects: Option<u32>,
}

