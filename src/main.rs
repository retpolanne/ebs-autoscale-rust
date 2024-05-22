use core::time;
use figment::{Figment, providers::{Format, Toml, Json, Env, Serialized}};
use log::{info, trace};
use std::error::Error;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;
use signal_hook::flag;
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::iterator::SignalsInfo;
use signal_hook::iterator::exfiltrator::WithOrigin;
use ebs_autoscale_rust::config::Config;

fn main() -> Result<(), Box<dyn Error>> {
    let figment = Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file("ebs-autoscale.toml"))
        .merge(Env::prefixed("EBS_AUTOSCALE_"))
        .join(Json::file("ebs-autoscale.json"));

    let config : Config = figment.extract()?;

    env_logger::init_from_env(
        env_logger::Env::default()
            .filter_or("LOG_LEVEL", "info")
    );
    info!("Started ebs autoscaler - config {:?}", config);

    let term_now = Arc::new(AtomicBool::new(false));
    for sig in TERM_SIGNALS {
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now))?;
        flag::register(*sig, Arc::clone(&term_now))?;
    }
    let mut signals = SignalsInfo::<WithOrigin>::new(TERM_SIGNALS)?;

    thread::spawn(move || {
        while !term_now.load(Ordering::Relaxed) {
           trace!("Checking if autoscaling is needed");
           thread::sleep(
               time::Duration::from_secs(config.detection_interval.into())
           );
        }
        trace!("Stopped checking...");
    });

    for info in &mut signals {
        match info.signal {
            _sigint => {
                info!("Gracefully shutting down ebs autoscaler");
                break;
            }
        }
    }

    if config.ensure_ebs_deleted_on_term {
        info!("Deleting ebs volumes before termination...");
        thread::sleep(time::Duration::from_millis(10000));
    }
    info!("Finished ebs autoscaler gracefully");

    Ok(())
}
