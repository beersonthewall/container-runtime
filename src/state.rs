use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct State {
    pid: usize,
    container_id: String,
    status: Status,
    bundle: PathBuf,
    annotations: HashMap<String, String>
}

impl State {
    pub fn new(container_id: String, bundle: PathBuf) -> Self {
	Self {
	    pid: 0,
	    container_id,
	    status: Status::Creating,
	    bundle,
	    annotations: HashMap::new(),
	}
    }
}

#[derive(Serialize, Deserialize)]
enum Status {
    Creating,
    Created,
    Running,
    Stopped,
}
