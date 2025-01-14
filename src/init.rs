//! Code for the initial process which runs inside a container.

use crate::config::Namespace;
use crate::container::Container;
use crate::ctx::Ctx;
use crate::error::ContainerErr;
use crate::ioprio::set_iopriority;
use crate::mount::setup_mounts;
use crate::namespaces::join_namspaces;
use crate::process::{clear_env, populate_env};
use crate::rlimit::set_rlimits;
use crate::rootfs::setup_rootfs;
use libc::{__errno_location, c_int, c_void, write, EINTR};
use log::debug;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

/// Init arguments
pub struct InitArgs {
    pub bundle_path: PathBuf,
    pub fifo_path: PathBuf,
    pub rdy_pipe_write_fd: c_int,
    pub container: Container,
    pub ctx: Ctx,
    pub join_ns: Vec<Namespace>,
}

/// First thing that runs in a new container process.
pub fn init(mut args: InitArgs) -> Result<(), ContainerErr> {
    let pid = std::process::id();
    args.container.state_mut().set_pid(pid);

    join_namspaces(&args.join_ns)?;

    clear_env();
    populate_env(args.container.config());

    set_rlimits(args.container.config())?;

    set_iopriority(args.container.config())?;

    setup_rootfs(args.container.config(), args.bundle_path)?;

    setup_mounts(args.container.config())?;

    // Write exit code to pipe for parent process
    notify_container_ready(args.rdy_pipe_write_fd);

    // Wait for FIFO to be opened. Then we can exec, at this moment we don't care what's
    // sent. Opening the fifo is the signal.
    wait_for_exec(&args.fifo_path);

    exec(args.container)?;

    debug!("container successfully created");

    Ok(())
}

fn notify_container_ready(fd: c_int) {
    let ret: c_int = 0;
    if fd > 0 {
        unsafe {
            debug!("writing to ready pipe");

            while write(fd, &raw const ret as *const c_void, size_of_val(&ret)) == -1
                && *__errno_location() == EINTR
            {
                debug!("retrying rdy notif");
            }
        }
    }
}

/// Won't return on success.
fn exec(container: Container) -> Result<(), ContainerErr> {
    Ok(())
}

fn wait_for_exec<P: AsRef<Path>>(fifo: P) {
    debug!("opening fifo");
    let _ = OpenOptions::new().read(true).open(fifo).unwrap();
}
