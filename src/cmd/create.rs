//! Create cmd

use std::path::{
    Path,
    PathBuf
};
use std::fs::OpenOptions;
use std::io::Write;
use std::ffi::{c_void, CString};

use log::debug;
use libc::{
    read,
    malloc,
    c_int,
    size_t,
    pipe2,
    mkfifo,
    clone,
    O_CLOEXEC,
    EINTR,
    __errno_location,
};

use crate::config::Config;
use crate::container::Container;
use crate::ctx::setup_ctx;
use crate::error::ContainerErr;
use crate::init::{
    init,
    InitArgs
};

/// Creates a new container from the OCI bundle located at bundle_path
pub fn create(container_id: String, bundle_path: String) -> Result<(), ContainerErr> {
    let bundle_path = PathBuf::from(bundle_path);
    let config = Config::load(&bundle_path)?;
    let ctx = setup_ctx()?;

    let c = Container::new(container_id.clone(), bundle_path, config);
    if c.exists(&ctx) {
	return Err(ContainerErr::State(format!("Container: {} already exists.", &container_id)));
    }

    c.write_state(&ctx)?;

    // Create container ready pipe. This is used for the container process to notify us
    // when it's ready to execute.
    let (rdy_pipe_read_fd, rdy_pipe_write_fd) = pipe()?;

    // Create FIFO used by container process to block until we send a signal to exec
    // the entrypoint process.
    let fifo_path = ctx.state_dir.join(&container_id).join("exec_fifo");
    fifo(&fifo_path)?;

    start_container(fifo_path, rdy_pipe_read_fd, rdy_pipe_write_fd)?;

    Ok(())
}

/// Creates a FIFO
fn fifo<P: AsRef<Path>>(path: P) -> Result<(), ContainerErr>{
    debug!("creating fifo");
    let path = if let Some(path) = path.as_ref().to_str() {
	path
    } else {
	debug!("fifo path: {:?}", path.as_ref());
	return Err(ContainerErr::Fifo("Fifo path not valid unicode"));
    };

    debug!("path: {}", path);
    let path = CString::new(path).map_err(|_| ContainerErr::Fifo("Invalid FIFO path"))?;
    let err = unsafe { mkfifo(path.as_c_str().as_ptr(), 0o622) };
    if err < 0 {
	debug!("{:?}", err);
	unsafe { debug!("errno {:?}", *__errno_location()) };
	return Err(ContainerErr::Fifo("Failed to create fifo."));
    }

    debug!("done creating fifo");
    Ok(())
}

/// Creates a pipe, on success returning a tuple (readfd, writefd)
fn pipe() -> Result<(c_int, c_int), ContainerErr> {
    let mut fd: [c_int;2] = [0, 0];
    let flags = O_CLOEXEC;
    let err = unsafe { pipe2(fd.as_mut_ptr(), flags) };
    if err < 0 {
	return Err(ContainerErr::Pipe(format!("Failed to create pipe err code: {}", err)));
    }

    Ok((fd[0], fd[1]))
}

/// Clones container child process
fn start_container(fifo_path: PathBuf, rdy_pipe_read_fd: c_int, rdy_pipe_write_fd: c_int) -> Result<(), ContainerErr> {
    let mut init_args = InitArgs {
	fifo_path: fifo_path.clone(),
	rdy_pipe_write_fd,
    };
    let args_ptr: *mut InitArgs = &mut init_args;

    debug!("allocating container stack");
    const STACK_SIZE: size_t = 1024* 1024;
    let stack = unsafe { malloc(STACK_SIZE) };
    let stack_ptr = unsafe { stack.offset(STACK_SIZE as isize) };

    debug!("cloning child process");
    let child_pid = unsafe { clone(init, stack_ptr, libc::SIGCHLD, args_ptr as *mut c_void) };

    if child_pid == -1 {
	debug!("clone failed, exiting.");
	std::process::exit(1);
    }

    // Read child process ready status
    let mut ret: c_int = 0;
    let ret_ptr: *mut c_int = &mut ret;
    debug!("waiting for container ready status...");
    unsafe {
	while read(rdy_pipe_read_fd, ret_ptr as *mut c_void, size_of_val(&ret)) == -1 &&
	    *libc::__errno_location() == EINTR {}
    }

    if ret > 0 {
	return Err(ContainerErr::Init("Error initializing container process"));
    }

    debug!("opening FIFO");
    let mut f = OpenOptions::new().write(true).append(true).open(&fifo_path).map_err(|_| ContainerErr::Fifo("failed to open fifo"))?;
    f.write_all(b"exec").map_err(|_| ContainerErr::Fifo("failed to open fifo"))?;
    debug!("done with fifo");

    Ok(())
}
