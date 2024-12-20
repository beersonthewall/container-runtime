/// A container's config.json
/// https://github.com/opencontainers/runtime-spec/blob/main/config.md
pub struct Config {
    oci_version: String,

    // Root
    // https://github.com/opencontainers/runtime-spec/blob/main/config.md#root
    rootfs_path: String,
    rootfs_readonly: bool,

    // Process
    // https://github.com/opencontainers/runtime-spec/blob/main/config.md#process
    terminal: bool,
    cwd: String,
    env: Vec<String>,
    args: Vec<String>,
    command_line: String,

    // User
    // https://github.com/opencontainers/runtime-spec/blob/main/config.md#user
    //user: todo,

    // Hostname
    // https://github.com/opencontainers/runtime-spec/blob/main/config.md#hostname
    hostname: String,
}

