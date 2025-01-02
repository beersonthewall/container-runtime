use crate::{config::Config, error::ContainerErr};
use libc::{c_int, syscall, SYS_ioprio_set, __errno_location};
use log::debug;

/// syscall ioprio_set
pub fn set_iopriority(config: &Config) -> Result<(), ContainerErr> {
    // linux header enum, so libc doesn't have this
    // https://github.com/torvalds/linux/blob/059dd502b263d8a4e2a84809cf1068d6a3905e6f/include/uapi/linux/ioprio.h#L53
    const IOPRIO_WHO_PROCESS: c_int = 1;
    if let Some(prio) = &config.process().io_priority {
        debug!("{:?}", prio);
        let err = unsafe { syscall(SYS_ioprio_set, IOPRIO_WHO_PROCESS, 0, prio.priority) };
        if err == -1 {
            let errno = unsafe { *__errno_location() };
            return Err(ContainerErr::IoPriority(format!(
                "syscall: ioprio_set failed errno: {}",
                errno
            )));
        }
    }

    Ok(())
}
