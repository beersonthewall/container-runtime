use crate::{
    config::{Config, RLimit},
    error::ContainerErr,
};
use libc::{
    __errno_location, __rlimit_resource_t, getrlimit, rlimit, setrlimit, RLIMIT_AS, RLIMIT_CORE,
    RLIMIT_CPU, RLIMIT_DATA, RLIMIT_FSIZE, RLIMIT_LOCKS, RLIMIT_MEMLOCK, RLIMIT_MSGQUEUE,
    RLIMIT_NICE, RLIMIT_NOFILE, RLIMIT_NPROC, RLIMIT_RSS, RLIMIT_RTPRIO, RLIMIT_RTTIME,
    RLIMIT_SIGPENDING, RLIMIT_STACK,
};
use log::debug;

/// Sets process rlimits. See [getrlimit](https://pubs.opengroup.org/onlinepubs/9699919799/functions/getrlimit.html) for details.
pub fn set_rlimits(config: &Config) -> Result<(), ContainerErr> {
    let process = config.process();

    if let Some(rlimits) = &process.rlimits {
        for rl in rlimits {
            match rl.typ.as_str() {
                "RLIMIT_AS" => set_rlimit(RLIMIT_AS, rl)?,
                "RLIMIT_CORE" => set_rlimit(RLIMIT_CORE, rl)?,
                "RLIMIT_CPU" => set_rlimit(RLIMIT_CPU, rl)?,
                "RLIMIT_DATA" => set_rlimit(RLIMIT_DATA, rl)?,
                "RLIMIT_FSIZE" => set_rlimit(RLIMIT_FSIZE, rl)?,
                "RLIMIT_LOCKS" => set_rlimit(RLIMIT_LOCKS, rl)?,
                "RLIMIT_MEMLOCK" => set_rlimit(RLIMIT_MEMLOCK, rl)?,
                "RLIMIT_MSGQUEUE" => set_rlimit(RLIMIT_MSGQUEUE, rl)?,
                "RLIMIT_NICE" => set_rlimit(RLIMIT_NICE, rl)?,
                "RLIMIT_NOFILE" => set_rlimit(RLIMIT_NOFILE, rl)?,
                "RLIMIT_NPROC" => set_rlimit(RLIMIT_NPROC, rl)?,
                "RLIMIT_RSS" => set_rlimit(RLIMIT_RSS, rl)?,
                "RLIMIT_RTPRIO" => set_rlimit(RLIMIT_RTPRIO, rl)?,
                "RLIMIT_RTTIME" => set_rlimit(RLIMIT_RTTIME, rl)?,
                "RLIMIT_SIGPENDING" => set_rlimit(RLIMIT_SIGPENDING, rl)?,
                "RLIMIT_STACK" => set_rlimit(RLIMIT_STACK, rl)?,
                _ => return Err(ContainerErr::Rlimit(format!("Invalid rlimit: {}", rl.typ))),
            }
        }
    }

    Ok(())
}

fn set_rlimit(resource: __rlimit_resource_t, rlimit: &RLimit) -> Result<(), ContainerErr> {
    debug!("set rlimit {:?}", rlimit);
    unsafe {
        let mut rlim = std::mem::zeroed::<rlimit>();
        // https://github.com/opencontainers/runtime-spec/blob/main/config.md#posix-process
        // > For each entry in rlimits, a getrlimit(3) on type MUST succeed.
        // So we do getrlimit before setting.
        let err = getrlimit(resource, &mut rlim);
        if err == -1 {
            return Err(ContainerErr::Rlimit(format!(
                "getrlimit: resource {}, errno: {}",
                resource,
                *__errno_location()
            )));
        }
        rlim.rlim_cur = rlimit.soft;
        rlim.rlim_max = rlimit.hard;

        let err = setrlimit(resource, &mut rlim);
        if err == -1 {
            return Err(ContainerErr::Rlimit(format!(
                "setrlimit: resource {}, errno: {}",
                resource,
                *__errno_location()
            )));
        }
    }
    Ok(())
}
