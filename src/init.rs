use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::time::UNIX_EPOCH;
use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use libc::{__errno_location, c_int, c_void, write, EINTR};

use crate::container::Container;
use crate::rlimit::set_rlimits;
use crate::process::{clear_env, populate_env};

/// Init arguments
pub struct InitArgs {
    pub fifo_path: PathBuf,
    pub rdy_pipe_write_fd: c_int,
    pub container: Container,
}

/// First thing that runs in a new container process.
pub extern "C" fn init(args: *mut c_void) -> c_int {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let path = format!("/tmp/container_child_{}", time);
    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .unwrap();

    let args = args as *mut InitArgs;
    let args = unsafe { args.as_mut().unwrap() };

    clear_env();
    populate_env(args.container.config());

    if let Err(e) = set_rlimits(args.container.config()) {
        log_file.write_all(format!("{:?}", e).as_bytes()).unwrap();
        log_file.flush().unwrap();
        std::process::exit(1);
    }

    // Write exit code to pipe for parent process
    notify_container_ready(args.rdy_pipe_write_fd, &mut log_file);

    // Wait for FIFO to be opened. Then we can exec, at this moment we don't care what's
    // sent. Opening the fifo is the signal.
    wait_for_exec(&args.fifo_path, &mut log_file);

    // TODO: exec, for now write logs to a file.
    log_file
        .write_all(b"container successfully created\n")
        .unwrap();

    0
}

fn notify_container_ready(fd: c_int, log_file: &mut File) {
    let ret: c_int = 0;
    let ret_ptr: *const c_int = &ret;
    if fd > 0 {
        unsafe {
            log_file.write_all(b"writing to ready pipe..\n").unwrap();
            log_file.flush().unwrap();

            while write(fd, ret_ptr as *const c_void, size_of_val(&ret)) == -1
                && *__errno_location() == EINTR
            {
                log_file.write_all(b"retrying rdy notif\n").unwrap();
                log_file.flush().unwrap();
            }
        }
    }
}

fn wait_for_exec<P: AsRef<Path>>(fifo: P, log_file: &mut File) {
    log_file.write_all(b"opening fifo").unwrap();
    log_file.flush().unwrap();
    let _ = OpenOptions::new().read(true).open(fifo).unwrap();
}
