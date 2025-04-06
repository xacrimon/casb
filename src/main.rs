mod cache;
mod pack;
mod repo;
mod upath;

use clap::Parser;
use log::{Level, debug, error, info};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let args = Args::parse();
    env_logger::builder()
        .filter(None, Level::Debug.to_level_filter())
        .format_timestamp_millis()
        .init();

    debug!("and we're alive!");

    for _ in 0..args.count {
        println!("Hello {}!", args.name);
    }
}
