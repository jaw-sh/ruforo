use std::sync::Mutex;
use std::sync::Arc;
use chrono::NaiveDateTime;
use chrono::Utc;

pub struct BigChungus {
	val: Mutex<i32>,
	start_time: Mutex<NaiveDateTime>,
}

impl BigChungus {
	pub fn new() -> Self {
		BigChungus {
			val: Mutex::new(32),
			start_time: Mutex::new(Utc::now().naive_utc()),
		}
	}
}

pub fn test() {
	let cache = Arc::new(BigChungus::new());
	let a = cache.clone();
	*a.val.lock().unwrap() = 10;
}
