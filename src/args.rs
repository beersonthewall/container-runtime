use std::env::Args;

#[derive(Debug)]
pub enum Command {
    Create,
    Delete,
    Kill,
    Start,
    State,
}

pub fn parse_args(mut args: Args) -> Result<Command, ()> {
    assert!(args.len() > 1);
    if let Some(cmd) = args.nth(1) {
	match cmd.as_str() {
	    "create" => Ok(Command::Create),
	    "delete" => Ok(Command::Delete),
	    "kill" => Ok(Command::Kill),
	    "start" => Ok(Command::Start),
	    "state" => Ok(Command::State),
	    _ => Err(())
	}
    } else {
	Err(())
    }
}
