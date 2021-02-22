use anyhow::Result;
use clap::Clap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

mod ecal_rs;

type Publisher<T> = ecal::prost::Publisher<T>;
type Subscriber<T> = ecal::prost::Subscriber<T>;

#[derive(Clap)]
struct Opts {
    #[clap(long)]
    pong: bool,
}

fn pong_main() -> Result<()> {
    let exit_requested = Arc::new(AtomicBool::new(false));

    let er = exit_requested.clone();
    ctrlc::set_handler(move || {
        er.store(true, Ordering::Relaxed);
    })?;

    let tick_len = Duration::from_millis(500);

    let publisher = Publisher::<ecal_rs::Pong>::new("/kpns/demo/pong")?;
    let subscriber = Subscriber::<ecal_rs::Ping>::new("/kpns/demo/ping")?;

    let mut pong = ecal_rs::Pong { sync: 1 };

    while !exit_requested.load(Ordering::Relaxed) && ecal::ok() {
        let start = Instant::now();

        if let Some(ping) = subscriber.try_recv(tick_len) {
            log::info!("Ping {}", ping.sync);
            pong.sync = ping.sync + 1;
            log::info!("Pong {}", pong.sync);
            publisher.send(&pong)?;
        }

        // No real need to use this instead of std::thread.
        let elapsed = start.elapsed();
        ecal::sleep(tick_len - elapsed.min(tick_len));
    }

    Ok(())
}

fn ping_main() -> Result<()> {
    let exit_requested = Arc::new(AtomicBool::new(false));

    let er = exit_requested.clone();
    ctrlc::set_handler(move || {
        er.store(true, Ordering::Relaxed);
    })?;

    let tick_len = Duration::from_millis(500);

    let publisher = Publisher::<ecal_rs::Ping>::new("/kpns/demo/ping")?;
    let subscriber = Subscriber::<ecal_rs::Pong>::new("/kpns/demo/pong")?;

    let mut ping = ecal_rs::Ping { sync: 1 };

    while !exit_requested.load(Ordering::Relaxed) && ecal::ok() {
        let start = Instant::now();

        log::info!("Ping {}", ping.sync);
        publisher.send(&ping)?;

        if let Some(pong) = subscriber.try_recv(tick_len) {
            log::info!("Pong {}", pong.sync);
            ping.sync = pong.sync;
        }

        // No real need to use this instead of std::thread.
        let elapsed = start.elapsed();
        ecal::sleep(tick_len - elapsed.min(tick_len));
    }

    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    let opts = Opts::parse();

    // Using the RAII based automated finalization approach
    let _cal = ecal::Cal::new("kcal_ping")?;

    if opts.pong {
        pong_main()
    } else {
        ping_main()
    }
}
