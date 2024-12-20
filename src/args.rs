use std::env::Args;

#[derive(Debug)]
pub enum Command {
    Create{container_id: String, bundle_path: String},
    Delete{container_id: String},
    Kill{container_id: String, signal: String},
    Start{container_id: String},
    State{container_id: String},
}

pub fn parse_args(args: Args) -> Result<Command, ()> {
    let args: Vec<String> = args.collect();
    match args.len() {
	3 => {
	    match args[1].as_str() {
		"start" => Ok(Command::Start { container_id: args[2].clone() }),
		"delete" => Ok(Command::Delete { container_id: args[2].clone() }),
		"state" => Ok(Command::State { container_id: args[2].clone() }),
		_ => Err(())
	    }
	}
	4 => {
	    match args[1].as_str() {
		"create" => Ok(Command::Create { container_id: args[2].clone(), bundle_path: args[3].clone() }),
		"kill" => Ok(Command::Kill { container_id: args[2].clone(), signal: args[3].clone() }),
		_ => Err(()),
  	    }
	},
	_ => Err(()),
    }
}
