use std::fs::File;
use std::os::fd::{RawFd, AsRawFd};
use std::path::PathBuf;
use libc::{
    c_int,
    setns,
    waitpid,
    malloc,
};
use super::config::Config;
use super::state::State;
use super::error::ContainerErr;
use super::ctx::Ctx;

pub struct Container {
    state: State,
    config: Config,
}

impl Container {

    pub fn new(container_id: String, bundle_path: PathBuf, config: Config) -> Self {
	Self {
	    state: State::new(container_id, bundle_path, config.oci_version.clone()),
	    config,
	}
    }

    pub fn create(&mut self, ctx: &Ctx) -> Result<(), ContainerErr> {
	const STACK_SZ: libc::size_t = 1024 * 1024;
	let stack = unsafe { malloc(STACK_SZ) };
	if stack.is_null() {
	    panic!("stack malloc failed");
	}

	let flags = get_proc_flags(&self.config);
	let config_ptr: *mut Config = &mut self.config;

	// Clone child process
	let child_pid = unsafe {
	    libc::clone(child, stack.offset(STACK_SZ as isize), flags | libc::SIGCHLD, config_ptr as *mut libc::c_void)
	};
	if child_pid == -1 {
	    let errno = unsafe { *libc::__errno_location() };
	    panic!("clone failed errno: {}", errno);
	}

	let mut child_status: c_int = 0;
	unsafe { waitpid(child_pid, &mut child_status, 0); }
	if libc::WIFEXITED(child_status) {
	    Ok(())
	} else {
	    return Err(ContainerErr::Child(format!("{}", libc::WEXITSTATUS(child_status))));
	}
    }
}

unsafe fn enter_namespace(fd: RawFd, ns: c_int) {
    if setns(fd, ns) != 0 {
	panic!("failed to set namespace");
    }
}

extern "C" fn child(config: *mut libc::c_void) -> libc::c_int {
    let config: &mut Config = unsafe {
	let ptr = config as *mut Config;
	ptr.as_mut().unwrap()
    };

    for ns in config.linux_namespaces().unwrap() {
	if ns.path.is_none() {
	    continue;
	}

	let fd = File::open(ns.path.clone().unwrap()).unwrap().as_raw_fd();
	unsafe {
	    match ns.typ.as_ref() {
		"pid" => enter_namespace(fd, libc::CLONE_NEWPID),
		"network" => enter_namespace(fd, libc::CLONE_NEWNET),
		"mount" => enter_namespace(fd, libc::CLONE_NEWNS),
		"ipc" => enter_namespace(fd, libc::CLONE_NEWIPC),
		"uts" => enter_namespace(fd, libc::CLONE_NEWUTS),
		"user" => enter_namespace(fd, libc::CLONE_NEWUSER),
		"cgroup" => enter_namespace(fd, libc::CLONE_NEWCGROUP),
		"time" => enter_namespace(fd, libc::CLONE_NEWTIME),
		_ => {},
	    }
	}
    }

    return 0;
}

fn get_proc_flags(config: &Config) -> c_int {
    let mut flags: c_int = 0;
    for ns in config.linux_namespaces().unwrap() {
	// If we've got a path the child will use setns()
	// to join. We don't want to create a new namespace in that case.
	if ns.path.is_some() {
	    continue;
	}

	match ns.typ.as_str() {
	    "pid" => flags |= libc::CLONE_NEWPID,
	    "network" => flags |= libc::CLONE_NEWNET,
	    "mount" => flags |= libc::CLONE_NEWNS,
	    "ipc" => flags |= libc::CLONE_NEWIPC,
	    "uts" => flags |= libc::CLONE_NEWUTS,
	    "user" => flags |= libc::CLONE_NEWUSER,
	    "cgroup" => flags |= libc::CLONE_NEWCGROUP,
	    "time" => flags |= libc::CLONE_NEWTIME,
	    _ => {},
	}
    }
    flags
}

