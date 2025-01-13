//! Create cmd

use crate::cgroup::{create_cgroup, detect_cgroup_version};
use crate::config::Config;
use crate::container::Container;
use crate::ctx::{setup_ctx, Ctx};
use crate::error::ContainerErr;
use crate::init::{init, InitArgs};
use crate::namespaces::{clone_namespace_flags, namespaces_to_join};
use crate::process::clone3;
use crate::state::Status;
use libc::{__errno_location, c_int, mkfifo, read, EINTR};
use log::debug;
use std::ffi::{c_void, CString};
use std::fs::OpenOptions;
use std::io::{ErrorKind, Read};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::pipe::{PipeReader, PipeWriter};
use std::process::exit;

/// Creates a new container from the OCI bundle located at bundle_path
pub fn create(container_id: String, bundle_path: String) -> Result<(), ContainerErr> {
    let bundle_path = PathBuf::from(bundle_path);
    let config = Config::load(&bundle_path)?;
    let ctx = setup_ctx()?;

    let mut c = Container::new(container_id.clone(), bundle_path.clone(), config);
    if c.exists(&ctx) {
        return Err(ContainerErr::State(format!(
            "Container: {} already exists.",
            &container_id
        )));
    }

    c.write_state(&ctx)?;

    // Create container ready pipe. This is used for the container process to notify us
    // when it's ready to execute.
    let (rdy_pipe_reader, rdy_pipe_writer) = std::pipe::pipe().map_err(ContainerErr::IO)?;

    // Create FIFO used by container process to block until we send a signal to exec
    // the entrypoint process.
    let fifo_path = ctx.state_dir.join(&container_id).join("exec_fifo");
    fifo(&fifo_path)?;

    init_container_proc(
        fifo_path,
        rdy_pipe_reader,
        rdy_pipe_writer,
        c.clone(),
        ctx.clone(),
        bundle_path,
    )?;

    c.update_status(Status::Created);
    c.write_state(&ctx)?;

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

/// Clones container child process
fn init_container_proc(
    fifo_path: PathBuf,
    rdy_pipe_reader: PipeReader,
    rdy_pipe_writer: PipeWriter,
    container: Container,
    ctx: Ctx,
    bundle_path: PathBuf,
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

    if let Err(e) = detect_cgroup_version(ctx.cgroups_root()) {
        debug!("detect_cgroup_version {:?}", e);
        exit(1);
    }
    let cgroup_path = ctx.cgroups_root().join(container.state().id());
    create_cgroup(&cgroup_path, container.config())?;

    let init_args = InitArgs {
        bundle_path,
        fifo_path: fifo_path.clone(),
        rdy_pipe_write_fd: rdy_pipe_writer.as_raw_fd(),
        container,
        ctx,
        join_ns,
    };

    debug!("cloning child process");
    log::logger().flush();
    let cgroup_file = OpenOptions::new()
        .read(true)
        .open(&cgroup_path)
        .map_err(ContainerErr::IO)?;
    let pid = clone3(flags, cgroup_file.as_raw_fd())?;
    debug!("PID: {}", pid);
    if pid == 0 {
        // child process
        let err = init(init_args);
        if err != 0 {
            Err(ContainerErr::Child(format!(
                "child process crashed exit code {}",
                err
            )))
        } else {
            Ok(())
        }
    } else {
        // parent
        // Read child process ready status
        let mut ret: c_int = 0;
        debug!("waiting for container ready status... {}", pid);

        unsafe {
            while read(
                rdy_pipe_reader.as_raw_fd(),
                &raw mut ret as *mut c_void,
                size_of_val(&ret),
            ) == -1
                && *libc::__errno_location() == EINTR
            {}
        }

        if ret > 0 {
            return Err(ContainerErr::Init("Error initializing container process"));
        }

        Ok(())
    }
}

/// Reads from a pipe and retries interrupted reads until sucessful or encounters
/// another error.
fn read_pipe_retry_temp_fail<P: AsRef<Path>>(pipe: P) -> Result<Vec<u8>, std::io::Error> {
    let mut f = OpenOptions::new().read(true).open(pipe)?;
    let mut buffer = Vec::new();

    while let Err(e) = f.read(&mut buffer) {
        if e.kind() != ErrorKind::Interrupted {
            return Err(e);
        }
    }

    Ok(buffer)
}
