use std::{env, process::Command, time::SystemTime};

fn main() {
	let start_time = SystemTime::now();
	let args: Vec<String> = env::args().collect();

	println!("{:?}", args);
	if args.len() < 2 {
		println!("Not enough arguments.");
		return;
	}
	let cmd = &args[1];

	println!("starting build");
	let exit_status = Command::new(cmd).args(&args[2..]).status();
	println!("\n");
	let time_taken = start_time.elapsed().unwrap();
	println!("Took {:?}", time_taken);

	println!("{:?}", exit_status);
	if let Some(status) = exit_status.ok().and_then(|s| s.code()) {
		std::process::exit(status);
	}
}
