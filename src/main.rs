use clap::Parser;
use ring::{Address, Config, Pinger};
use std::format;

fn main() {
    let args = Args::parse();
    match Pinger::new(args.as_config()) {
        Ok(pinger) => {
            pinger.run().unwrap_or_else( |e|{
                exit(format!("Error on run ping: {}", e));
            });
        },
        Err(e) => {
            exit(format!("Error on init: {}", e));
        }
    }
}

fn exit(msg: String) {
    eprintln!("{}", msg);
    std::process::exit(1);
}

/// ping but with rust, rust + ping -> ring
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Count of ping times
    #[arg(short, default_value_t = 4)]
    count: u16,

    /// Ping packet size
    #[arg(short = 's', default_value_t = 64)]
    packet_size: usize,

    /// Ping ttl
    #[arg(short = 't', default_value_t = 64)]
    ttl: u32,

    /// Ping timeout seconds
    #[arg(short = 'w', default_value_t = 1)]
    timeout: u64,

    /// Ping interval duration milliseconds
    #[arg(short = 'i', default_value_t = 1000)]
    interval: u64,

    /// Ping destination, ip or domain
    #[arg(value_parser=Address::parse)]
    destination: Address,
}

impl Args {
    fn as_config(&self) -> Config {
        Config {
            count: self.count,
            packet_size: self.packet_size,
            ttl: self.ttl,
            timeout: self.timeout,
            interval: self.interval,
            id: rand::random::<u16>(),
            sequence: 1,
            destination: self.destination.clone(),
        }
    }
}
