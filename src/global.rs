use once_cell::sync::OnceCell;

static SESSION_TIME: OnceCell<chrono::Duration> = OnceCell::new();

#[inline(always)]
pub fn get_session_time() -> &'static chrono::Duration {
    unsafe { SESSION_TIME.get_unchecked() }
}

pub fn init() {
    // Init SESSION_TIME
    let time = std::env::var("SESSION_TIME").expect("SESSION_TIME MISSING from .env");
    let time = time
        .parse::<i64>()
        .expect("SESSION_TIME cannot be parsed as an integer");
    if time < 0 {
        panic!("SESSION_TIME is a negative number!");
    }
    let time = chrono::Duration::minutes(time);
    SESSION_TIME.set(time).expect("failed to set SESSION_TIME");
}
