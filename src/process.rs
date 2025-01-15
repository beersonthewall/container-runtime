//! Module for manipulating a container process.

use crate::{config::Config, error::ContainerErr, state::Pid};
use libc::{c_int, clone_args, syscall, SYS_clone3, __errno_location, CLONE_INTO_CGROUP, SIG_IGN};
use log::debug;
use std::{env::set_var, os::fd::RawFd};

/// Populates the environment of the current process from the config
pub fn populate_env(cfg: &Config) {
    if let Some(vars) = &cfg.process().env {
        for env_var in vars {
            let parts: Vec<_> = env_var.split("=").collect();
            if parts.len() == 2 {
                debug!("setting {} = {}", parts[0], parts[1]);
                set_var(parts[0], parts[1])
            }
        }
    }
}

/// Clears the current processes' environment.
/// All safety conditions from `std::env::remove_var` apply here.
/// See [remove_var docs](https://doc.rust-lang.org/stable/std/env/fn.remove_var.html) for details.
pub fn clear_env() {
    for pair in std::env::args() {
        let parts = pair.split("=").collect::<Vec<_>>();
        if parts.len() == 2 {
            let key = parts[0];
            debug!("delete env var: {} = {}", key, parts[1]);
            unsafe { std::env::remove_var(key) }
        }
    }
}

/// Wrapper for the clone3 syscall
pub fn clone3(flags: c_int, cgroup_fd: RawFd) -> Result<Pid, ContainerErr> {
    debug!("clone3");
    let mut args = unsafe { std::mem::zeroed::<clone_args>() };

    args.flags |= flags as u64;
    args.flags |= CLONE_INTO_CGROUP as u64;
    args.cgroup = cgroup_fd as u64;
    args.exit_signal = SIG_IGN as u64;

    let pid = unsafe {
        syscall(
            SYS_clone3,
            &raw mut args,
            size_of::<clone_args>(),
        )
    };
    if pid == -1 {
        return Err(ContainerErr::Clone(format!(
            "clone failed, errno: {}",
            unsafe { *__errno_location() }
        )));
    }

    Ok(pid as Pid)
}
