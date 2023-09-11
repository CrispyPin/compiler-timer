use std::{
	env,
	fs::{self, File},
	io::Write,
	process::{exit, Command},
	time::{SystemTime, UNIX_EPOCH},
};

use chrono::NaiveDateTime;

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
	let time_taken = start_time.elapsed().unwrap().as_millis() as i64;
	println!("Build took {}", printable_time(time_taken));

	log(&start_time);

	println!("{:?}", exit_status);
	if let Some(status) = exit_status.ok().and_then(|s| s.code()) {
		exit(status);
	}
}

fn log(start: &SystemTime) -> Option<()> {
	let start_time = start.duration_since(UNIX_EPOCH).ok()?.as_millis();
	let duration = start.elapsed().ok()?.as_millis();
	let mut history = fs::read_to_string("compiler_history.txt").unwrap_or_default();
	history.push_str(&format!("{}:{}\n", start_time, duration));

	let today = NaiveDateTime::from_timestamp_millis(start_time as i64)?;
	let mut wasted = 0;
	for line in history.lines() {
		let (time, duration) = line.split_once(':')?;
		let time: i64 = time.parse().ok()?;
		let duration: i64 = duration.parse().ok()?;
		let date = NaiveDateTime::from_timestamp_millis(time)?;
		if date.date() == today.date() {
			wasted += duration;
		}
	}
	println!("Total wasted today: {}", printable_time(wasted));

	let mut f = File::create("compiler_history.txt").unwrap();
	f.write_all(history.as_bytes()).unwrap();
	Some(())
}

fn printable_time(ms: i64) -> String {
	let secs = ms / 1000 % 60;
	let mins = ms / (60 * 1000) % 60;
	let hours = ms / (60 * 60 * 1000);
	if hours > 0 {
		format!("{}h {}m {}s", hours, mins, secs)
	} else if mins > 0 {
		format!("{}m {}s", mins, secs)
	} else {
		format!("{}.{:03}s", secs, ms % 1000)
	}
}
