use crate::error::ContainerErr;
use serde::{self, Deserialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

/// A container's config.json
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    oci_version: String,
    root: Root,
    mounts: Option<Vec<Mount>>,
    process: Process,

    // Hostname
    // https://github.com/opencontainers/runtime-spec/blob/main/config.md#hostname
    hostname: Option<String>,

    // Domainname
    // https://github.com/opencontainers/runtime-spec/blob/main/config.md#domainname
    domainname: Option<String>,

    // Linux platform specific configuration
    // https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#linux-container-configuration
    linux: Option<Linux>,
}

impl Config {
    /// Reads the 
    pub fn load(bundle_path: &Path) -> Result<Self, ContainerErr> {
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
        Ok(config)
    }
}

/// Root configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#root
#[derive(Deserialize, Debug)]
struct Root {
    path: String,
    readonly: bool,
}

/// Mount configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#mounts
#[derive(Deserialize)]
struct Mount {
    destination: String,
    source: Option<String>,
    options: Option<Vec<String>>,
}

/// Process configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#mounts
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Process {
    terminal: bool,
    console_size: Option<ConsoleSize>,
    cwd: String,
    env: Option<Vec<String>>,
    args: Option<Vec<String>>,
    command_line: Option<String>,
    user: User,
}

/// Console Size configuration
#[derive(Deserialize)]
struct ConsoleSize {
    height: usize,
    width: usize,
}

/// A Process' user configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#user
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    uid: isize,
    gid: isize,
    umask: Option<isize>,
    additional_gids: Option<Vec<isize>>,
}

/// Linux platform specific configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md#platform-specific-configuration
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Linux {
    namespaces: Vec<Namespace>,
    uid_mapings: Option<Vec<UidMapping>>,
    time_offsets: Option<TimeOffsets>,
    devices: Option<Vec<Device>>,
    cgroups_path: Option<String>,
}

/// Linux Namespace configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#namespaces
#[derive(Deserialize)]
struct Namespace {
    // TODO: make this an enum?
    #[serde(rename = "type")]
    typ: String,
    path: Option<String>,
}

/// User namespace mappings
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#user-namespace-mappings
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
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
struct TimeOffsets {
    secs: i64,
    nanosecs: u32,
}

/// Linux device configuration
/// https://github.com/opencontainers/runtime-spec/blob/main/config-linux.md#devices
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
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
