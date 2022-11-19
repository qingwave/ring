use std::net;
use std::net::ToSocketAddrs;
use anyhow::bail;
use crate::error::RingError;

#[derive(Debug, Clone)]
pub struct Config {
    pub count: u16,
    pub packet_size: usize,
    pub ttl: u32,
    pub timeout: u64,
    pub interval: u64,
    pub id: u16,
    pub sequence: u16,
    pub destination: Address,
}

#[derive(Debug, Clone)]
pub struct Address {
    pub ip: net::IpAddr,
    pub raw: String,
}

impl Address {
    pub fn parse(host: &str) -> anyhow::Result<Address> {
        let raw = String::from(host);
        let opt = host.parse::<net::IpAddr>().ok();
        match opt {
            Some(ip) => Ok(Address { ip: ip, raw: raw }),
            None => {
                let new = format!("{}:{}", host, 0);
                let mut addrs = new.to_socket_addrs()?;
                if let Some(addr) = addrs.next() {
                    Ok(Address {
                        ip: addr.ip(),
                        raw: raw,
                    })
                } else {
                    bail!(RingError::InvalidConfig(String::from("address")))
                }
            }
        }
    }    
}
