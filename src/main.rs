mod watchdog;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;


fn main() {

    let watchdog = Arc::new(Mutex::new(watchdog::Watchdog::new()));


    let handle = thread::spawn(move || {
        watchdog.lock().unwrap().start();
    });

    thread::sleep(Duration::new(50, 0));

}