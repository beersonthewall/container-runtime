use libc::{__errno_location, c_int, c_void, write, EINTR};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::time::UNIX_EPOCH;
use std::{path::PathBuf, time::SystemTime};

/// Init arguments
#[repr(C)]
pub struct InitArgs {
    pub fifo_path: PathBuf,
    pub rdy_pipe_write_fd: c_int,
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

    // Write exit code to pipe for parent process
    let ret: c_int = 0;
    let ret_ptr: *const c_int = &ret;
    if args.rdy_pipe_write_fd > 0 {
        unsafe {
            log_file.write_all(b"writing to ready pipe..\n").unwrap();
            log_file.flush().unwrap();

            while write(
                args.rdy_pipe_write_fd,
                ret_ptr as *const c_void,
                size_of_val(&ret),
            ) == -1
                && *__errno_location() == EINTR
            {
                log_file.write_all(b"retrying rdy notif\n").unwrap();
                log_file.flush().unwrap();
            }
        }
    }

    // Wait for FIFO to be opened. Then we can exec, at this moment we don't care what's
    // sent. Opening the fifo is the signal.
    log_file.write_all(b"opening fifo").unwrap();
    log_file.flush().unwrap();
    let mut f = if let Ok(f) = OpenOptions::new().read(true).open(&args.fifo_path) {
        f
    } else {
        return 1;
    };

    let mut buf = Vec::new();
    f.read(&mut buf).unwrap();

    // TODO: exec, for now write logs to a file.
    log_file
        .write_all(b"container successfully created\n")
        .unwrap();

    ret
}
