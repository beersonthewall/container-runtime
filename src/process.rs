//! Module for manipulating a container process' settings.

use crate::config::Config;
use log::debug;
use std::env::set_var;

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
