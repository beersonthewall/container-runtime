//! namespaces

use std::{fs::File, os::fd::AsRawFd};

use libc::{c_int, setns, CLONE_NEWCGROUP, CLONE_NEWIPC, CLONE_NEWNET, CLONE_NEWNS, CLONE_NEWPID, CLONE_NEWTIME, CLONE_NEWUSER, CLONE_NEWUTS};
use log::debug;

use crate::{config::Namespace, error::ContainerErr};

/// returns the clone flags for any namespaces that need to be created
pub fn clone_namespace_flags(namespaces: &[Namespace]) -> c_int {
    let mut flags = 0;
    for ns in namespaces {
	if ns.path.is_some() {
	    continue;
	}

	// If we're not told what namespace to join we want to
	// create a new namespace when the child process is cloned.
	match ns.typ.as_str() {
	    "pid" => flags |= CLONE_NEWPID,
	    "network" => flags |= CLONE_NEWNET,
	    "mount" => flags |= CLONE_NEWNS,
	    "ipc" => flags |= CLONE_NEWIPC,
	    "uts" => flags |= CLONE_NEWUTS,
	    "user" => flags |= CLONE_NEWUSER,
	    "cgroup" => flags |= CLONE_NEWCGROUP,
	    "time" => flags |= CLONE_NEWTIME,
	    _ => {},
	}
    }
    flags
}

/// Selects namespaces we need to join in the child process (i.e. namespaces with
/// the path provided).
pub fn namespaces_to_join(namespaces: &[Namespace]) -> Vec<Namespace> {
    let mut ns_to_join = Vec::new();
    for ns in namespaces {
	if ns.path.is_none() {
	    continue;
	}

	debug!("found namespace to join {:?}", ns);
	ns_to_join.push(ns.clone());
    }
    ns_to_join
}

/// setns for each provided namespace.
pub fn join_namspaces(namespaces: &[Namespace]) -> Result<(), ContainerErr> {
    for ns in namespaces {
	if let Some(path) = &ns.path {
	    debug!("joining namespace: {:?}", ns);

	    let f = File::open(path).map_err(|e| ContainerErr::IO(e))?;
	    let fd = f.as_raw_fd();
	    let nstype = if let Some(nstype) = ns_type(&ns.typ) {
		nstype
	    } else {
		return Err(ContainerErr::InvalidNamespace(format!("invalid nstype: {}", ns.typ)));
	    };

	    // re-map any errors with the more human read-able information we've got.
	    set_namespace(fd, nstype).map_err(|_| ContainerErr::JoinNamespace(format!("failed to join namespace: {:?}", ns)))?;
	}
	
    }
    Ok(())
}

/// setns wrapper
fn set_namespace(fd: c_int, nstype: c_int) -> Result<(), ContainerErr> {
    debug!("fd {}, nstype {}", fd, nstype);
    if unsafe { setns(fd, nstype) } == -1 {
	return Err(ContainerErr::JoinNamespace(format!("failed to join namespace: nstype {}", nstype)));
    }

    Ok(())
}

fn ns_type(nstype: &str) -> Option<c_int> {
    match nstype {
	"pid" => Some(CLONE_NEWPID),
	"network" => Some(CLONE_NEWNET),
	"mount" => Some(CLONE_NEWNS),
	"ipc" => Some(CLONE_NEWIPC),
	"uts" => Some(CLONE_NEWUTS),
	"user" => Some(CLONE_NEWUSER),
	"cgroup" => Some(CLONE_NEWCGROUP),
	"time" => Some(CLONE_NEWTIME),
	_ => None,
    }
}
