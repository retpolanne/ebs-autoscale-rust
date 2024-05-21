use core::time;
use env_logger::Env;
use log::info;
use std::io::Error;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use signal_hook::flag;
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::iterator::SignalsInfo;
use signal_hook::iterator::exfiltrator::WithOrigin;

fn main() -> Result<(), Error> {
    env_logger::init_from_env(
        Env::default()
            .filter_or("LOG_LEVEL", "info")
    );
    let term_now = Arc::new(AtomicBool::new(false));
    for sig in TERM_SIGNALS {
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now))?;
        flag::register(*sig, Arc::clone(&term_now))?;
    }

    let mut signals = SignalsInfo::<WithOrigin>::new(TERM_SIGNALS)?;

    thread::spawn(move || {
       loop {
           info!("Working");
           thread::sleep(time::Duration::from_millis(100000));
       }
    });

    for info in &mut signals {
        match info.signal {
            _sigint => {
                info!("Gracefully shutting down ebs autoscaler");
                break;
            }
        }
    }

    thread::sleep(time::Duration::from_millis(10000));
    info!("Finished ebs autoscaler gracefully");

    Ok(())
}
