use std::process::Command;

#[macro_export]
macro_rules! log_warn {
    ($x:expr) => (
        { eprint!("\x1b[93mWARNING: "); eprint!($x); eprintln!("\x1b[0m")}
    );
    ($x:expr, $($more:expr),+) => (
        { eprint!("\x1b[93mWARNING: "); eprint!($x, $($more),*); eprintln!("\x1b[0m")}
    );
}
#[macro_export]
macro_rules! log {
    ($x:expr) => (
        { eprint!(""); eprintln!($x); }
    );
    ($x:expr, $($more:expr),+) => (
        { eprint!(""); eprintln!($x, $($more),*); }
    );
}

pub fn notify_user(msg: &str, icon: &str) {
    log_warn!("{msg}");

    let icon = "--icon=dialog-".to_string() + icon;

    let _cmd = Command::new("notify-send").arg(&icon).arg("Thistle Yocto Build").arg(msg).output();

    // no err handling - best effort (not available headless, etc...)
}
