use std::{
	env,
	fs::{self, File},
	io::Write,
	process::{exit, Command},
	sync::mpsc::channel,
	thread,
	time::{Duration, SystemTime, UNIX_EPOCH},
};

use chrono::NaiveDateTime;

fn main() {
	let (send_ctrlc, recv_ctrlc) = channel();
	ctrlc::set_handler(move || {
		send_ctrlc.send(()).unwrap();
	})
	.expect("Error setting Ctrl-C handler");

	let start_time = SystemTime::now();

	let args: Vec<String> = env::args().collect();

	println!();
	if args.len() < 2 {
		print_day();
		return;
	}
	println!("{args:?}");
	println!("Starting build");
	let cmd = args[1].clone();
	let compiler_thread = thread::spawn(move || Command::new(cmd).args(&args[2..]).status());

	let mut exit_status = None;
	loop {
		if let Ok(()) = recv_ctrlc.try_recv() {
			println!("\nCompilation killed");
			break;
		}
		if compiler_thread.is_finished() {
			exit_status = Some(compiler_thread.join());
			break;
		}
		thread::sleep(Duration::from_millis(10));
	}

	{
		let time_taken = start_time.elapsed().unwrap().as_millis() as i64;
		println!("\nBuild took {}", printable_time(time_taken));

		log_single(start_time);
		print_day();
		println!();

		if let Some(exit_status) = exit_status {
			if let Ok(Ok(status)) = exit_status {
				println!("exit status: {:?}", status.code());
				exit(status.code().unwrap_or_default());
			} else {
				print!("invalid exit status: {exit_status:?}");
			}
		} else {
			exit(1);
		}
	}
}

fn log_single(start: SystemTime) -> Option<()> {
	let start_time = start.duration_since(UNIX_EPOCH).ok()?.as_millis();
	let duration = start.elapsed().ok()?.as_millis();
	let mut history = fs::read_to_string("compiler_history.txt").unwrap_or_default();
	history.push_str(&format!("{start_time}:{duration}\n"));
	let mut f = File::create("compiler_history.txt").unwrap();
	f.write_all(history.as_bytes()).unwrap();
	Some(())
}

fn print_day() -> Option<()> {
	let Ok(history) = fs::read_to_string("compiler_history.txt") else {
		println!("No history in this directory.");
		return None;
	};

	let mut wasted_total = 0;
	let mut by_day = Vec::new();
	let mut last_date = None;
	let mut first_date = String::new();
	for line in history.lines() {
		let (time, duration) = line.split_once(':')?;
		let time: i64 = time.parse().ok()?;
		let duration: i64 = duration.parse().ok()?;
		let date = NaiveDateTime::from_timestamp_millis(time)?;

		wasted_total += duration;

		if Some(date.date()) != last_date {
			last_date = Some(date.date());
			by_day.push(0);
		}
		*by_day.last_mut().unwrap() += duration;

		if first_date.is_empty() {
			first_date = format!("{date:?}")[..10].to_owned();
		}
	}
	println!(
		"Total wasted today: {}",
		printable_time(*by_day.last().unwrap())
	);
	if by_day.len() >= 5 {
		println!(
			"5 day average: {}",
			printable_time(by_day[by_day.len() - 5..].iter().sum::<i64>() / 5)
		);
	}
	println!(
		"Overall average: {}",
		printable_time(wasted_total / by_day.len() as i64)
	);
	println!("Since {first_date}: {}", printable_time(wasted_total));
	Some(())
}

fn printable_time(ms: i64) -> String {
	let secs = ms / 1000 % 60;
	let mins = ms / (60 * 1000) % 60;
	let hours = ms / (60 * 60 * 1000);
	if hours > 0 {
		format!("{hours}h {mins}m {secs}s")
	} else if mins > 0 {
		format!("{mins}m {secs}s")
	} else {
		format!("{secs}.{:03}s", ms % 1000)
	}
}
