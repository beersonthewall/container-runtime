use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub type Pid = u32;

/// Container state
/// https://github.com/opencontainers/runtime-spec/blob/main/schema/state-schema.json
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    oci_version: String,
    pid: Pid,
    #[serde(rename = "id")]
    container_id: String,
    status: Status,
    bundle: PathBuf,
    annotations: HashMap<String, String>,
}

impl State {
    pub fn new(container_id: String, bundle: PathBuf, oci_version: String) -> Self {
        Self {
            oci_version,
            pid: 0,
            container_id,
            status: Status::Creating,
            bundle,
            annotations: HashMap::new(),
        }
    }

    pub fn update_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn id(&self) -> &str {
        &self.container_id
    }

    pub fn set_pid(&mut self, pid: Pid) {
        self.pid = pid;
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "creating")]
    Creating,
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stoped")]
    Stopped,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let id = String::from("foobar");
        let bundle = PathBuf::from("/blag/");
        let version = String::from("1.0.1");
        let state = State::new(id, bundle, version);
        assert_eq!("{\"ociVersion\":\"1.0.1\",\"pid\":0,\"id\":\"foobar\",\"status\":\"creating\",\"bundle\":\"/blag/\",\"annotations\":{}}",
		   serde_json::to_string(&state).unwrap());
    }
}
