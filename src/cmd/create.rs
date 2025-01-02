//! Create cmd

use crate::cgroup::create_cgroup;
use crate::config::Config;
use crate::container::Container;
use crate::ctx::{setup_ctx, Ctx};
use crate::error::ContainerErr;
use crate::init::{init, InitArgs};
use crate::namespaces::{clone_namespace_flags, namespaces_to_join};
use crate::process::clone3;
use libc::{__errno_location, c_int, mkfifo, pipe2, read, EINTR, O_CLOEXEC};
use log::debug;
use std::ffi::{c_void, CString};
use std::fs::OpenOptions;
use std::io::Write;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};

/// Creates a new container from the OCI bundle located at bundle_path
pub fn create(container_id: String, bundle_path: String) -> Result<(), ContainerErr> {
    let bundle_path = PathBuf::from(bundle_path);
    let config = Config::load(&bundle_path)?;
    let ctx = setup_ctx()?;

    let c = Container::new(container_id.clone(), bundle_path, config);
    if c.exists(&ctx) {
        return Err(ContainerErr::State(format!(
            "Container: {} already exists.",
            &container_id
        )));
    }

    c.write_state(&ctx)?;

    // Create container ready pipe. This is used for the container process to notify us
    // when it's ready to execute.
    let (rdy_pipe_read_fd, rdy_pipe_write_fd) = pipe()?;

    // Create FIFO used by container process to block until we send a signal to exec
    // the entrypoint process.
    let fifo_path = ctx.state_dir.join(&container_id).join("exec_fifo");
    fifo(&fifo_path)?;

    init_container_proc(fifo_path, rdy_pipe_read_fd, rdy_pipe_write_fd, c, ctx)?;

    Ok(())
}

/// Creates a FIFO
fn fifo<P: AsRef<Path>>(path: P) -> Result<(), ContainerErr> {
    debug!("creating fifo");
    let path = if let Some(path) = path.as_ref().to_str() {
        path
    } else {
        debug!("fifo path: {:?}", path.as_ref());
        return Err(ContainerErr::Fifo(String::from(
            "Fifo path not valid unicode",
        )));
    };

    debug!("path: {}", path);
    let path =
        CString::new(path).map_err(|_| ContainerErr::Fifo(String::from("Invalid FIFO path")))?;
    let err = unsafe { mkfifo(path.as_c_str().as_ptr(), 0o622) };
    if err < 0 {
        debug!("{:?}", err);
        unsafe { debug!("errno {:?}", *__errno_location()) };
        return Err(ContainerErr::Fifo(String::from("Failed to create fifo.")));
    }

    debug!("done creating fifo");
    Ok(())
}

/// Creates a pipe, on success returning a tuple (readfd, writefd)
fn pipe() -> Result<(c_int, c_int), ContainerErr> {
    let mut fd: [c_int; 2] = [0, 0];
    let flags = O_CLOEXEC;
    let err = unsafe { pipe2(fd.as_mut_ptr(), flags) };
    if err < 0 {
        return Err(ContainerErr::Pipe(format!(
            "Failed to create pipe err code: {}",
            err
        )));
    }

    Ok((fd[0], fd[1]))
}

/// Clones container child process
fn init_container_proc(
    fifo_path: PathBuf,
    rdy_pipe_read_fd: c_int,
    rdy_pipe_write_fd: c_int,
    container: Container,
    ctx: Ctx,
) -> Result<(), ContainerErr> {
    let mut flags = 0;
    if let Some(ns) = &container.config().linux_namespaces() {
        flags |= clone_namespace_flags(ns);
    }

    let join_ns = if let Some(ns) = &container.config().linux_namespaces() {
        namespaces_to_join(ns)
    } else {
        Vec::new()
    };

    // Create the cgroup in the parent process. We're going to use CLONE_INTO_CGROUP flag
    // for clone3 to join the group. If we create the process and only then create/join the
    // cgroup the child is automatically a part of the parent process' cgroup and we'd need
    // to handle migrating the child process to the new cgroup. Which is annoying :/
    let cgroup_path = ctx.cgroups_root().join(container.state().id());
    create_cgroup(&cgroup_path, container.config())?;

    let mut init_args = InitArgs {
        fifo_path: fifo_path.clone(),
        rdy_pipe_write_fd,
        container,
        ctx,
        join_ns,
    };

    debug!("cloning child process");
    log::logger().flush();
    let cgroup_file = OpenOptions::new()
        .read(true)
        .open(&cgroup_path)
        .map_err(|e| ContainerErr::IO(e))?;
    let pid = clone3(flags, cgroup_file.as_raw_fd())?;
    debug!("PID: {}", pid);
    if pid == 0 {
        // child process'
        let mut f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open("/tmp/output")
            .unwrap();
        f.write_all(b"child is alive...").unwrap();
        f.flush().unwrap();
        let err = init(init_args);
        if err != 0 {
            f.write_all(b"ded").unwrap();
            f.flush().unwrap();
            return Err(ContainerErr::Child(format!(
                "child process crashed exit code {}",
                err
            )));
        } else {
            f.write_all(b"child is done successfully...").unwrap();
            f.flush().unwrap();
            return Ok(());
        }
    } else {
        // parent
        // Read child process ready status
        let mut ret: c_int = 0;
        debug!("waiting for container ready status... {}", pid);
        unsafe {
            while read(
                rdy_pipe_read_fd,
                &raw mut ret as *mut c_void,
                size_of_val(&ret),
            ) == -1
                && *libc::__errno_location() == EINTR
            {}
        }

        if ret > 0 {
            return Err(ContainerErr::Init("Error initializing container process"));
        }

        return Ok(());
    }
}
