use std::path::PathBuf;

use clap::Parser;
use libc;
use log::{warn, info};
use simplelog;

struct Buffer(Vec<usize>);

impl Buffer {
    fn new(size_in_gb: usize) -> Buffer {
        let size_in_bytes = size_in_gb * 1024 * 1024 * 1024;
        let mut vec = Vec::with_capacity(size_in_bytes / std::mem::size_of::<usize>());

        for i in 0..vec.len() {
            vec.push(i);
        }

        let mlock_succeeded = unsafe { libc::mlock(vec.as_ptr() as _, size_in_bytes) == 0 };
        if !mlock_succeeded {
            warn!("mlock(2) failed, cannot guarantee pages will remain in memory");
        }

        Buffer(vec)
    }

    fn scan(&mut self) {
        for (expected, actual) in self.0.iter().enumerate() {
            if *actual != expected {
                info!(
                    "expected 0b{:b} (i.e. {}) but got 0b{:b}",
                    expected, expected, actual
                );
            }
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { libc::munlockall() };
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The size of the buffer to watch, in GB
    #[arg(short, long, default_value_t = 8)]
    size: usize,

    /// The delay between iterations, in seconds
    #[arg(short, long, default_value_t = 1)]
    delay: u64,

    /// Log the output to a file.
    #[arg(short, long, value_name = "FILE")]
    log: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    // literally the example code
    {
        use simplelog::*;
        use std::fs::File;

        let mut loggers: Vec<Box<dyn SharedLogger>> = vec![TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )];

        if let Some(path) = args.log {
            loggers.push(WriteLogger::new(
                LevelFilter::Info,
                Config::default(),
                File::create(path).unwrap(),
            ));
        }

        CombinedLogger::init(loggers).expect("could not initialize logging");
    }

    println!("Hit control-c to stop.");

    info!(
        "running with a {} GB buffer, checking every {} seconds",
        args.size, args.delay
    );

    let mut buf = Buffer::new(args.size);

    let mut iteration_count = 0;
    loop {
        info!("iteration {}", iteration_count);
        buf.scan();
        iteration_count += 1;
        std::thread::sleep(std::time::Duration::from_secs(args.delay));
    }
}
