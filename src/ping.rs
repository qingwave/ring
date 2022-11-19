use std::net::{self, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use pnet::packet::icmp::echo_reply::EchoReplyPacket;
use pnet::packet::icmp::echo_request::{IcmpCodes, MutableEchoRequestPacket};
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::{util, MutablePacket, Packet};

use crate::config::Config;
use crate::error::RingError;

use socket2::{Domain, Protocol, Socket, Type};

#[derive(Clone)]
pub struct Pinger {
    config: Config,
    dest: SocketAddr,
    socket: Arc<Socket>,
}

impl Pinger {
    pub fn new(config: Config) -> std::io::Result<Self> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4))?;
        let src = SocketAddr::new(net::IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
        let dest = SocketAddr::new(config.destination.ip, 0);
        socket.bind(&src.into())?;
        socket.set_ttl(config.ttl)?;
        socket.set_read_timeout(Some(Duration::new(config.timeout, 0)))?;
        socket.set_write_timeout(Some(Duration::new(config.timeout, 0)))?;
        Ok(Self {
            config: config,
            dest: dest,
            socket: Arc::new(socket),
        })
    }

    pub fn run(&self) -> std::io::Result<()> {
        println!("PING {}({})", self.config.destination.raw, self.config.destination.ip);
        let now = Instant::now();
        let mut send: u64 = 0;
        let mut success: u64 = 0;
        let this = Arc::new(self.clone());
        let mut handles = Vec::new();
        for i in 0..self.config.count {
            let this = this.clone();
            handles.push(std::thread::spawn(move || {
                this.ping(i)
            }));
            
            send += 1;
            if i < self.config.count - 1 {
                thread::sleep(Duration::from_millis(self.config.interval));
            }
        }

        for handle in handles {
            if let Some(res) = handle.join().ok() {
                if res.is_ok() {
                    success += 1;
                }
            }
        }

        let total = now.elapsed().as_micros() as f64 / 1000.0;
        let loss_rate = if send > 0 {(send - success) * 100 / send} else {0};
        println!("\n--- {} ping statistics ---", self.config.destination.raw);
        println!(
            "{} packets transmitted, {} received, {}% packet loss, time {}ms",
            send,
            success,
            loss_rate,
            total,
        );
        Ok(())
    }

    pub fn ping(&self, seq_offset: u16) -> anyhow::Result<()> {
        // create icmp request packet
        let mut buf = vec![0; self.config.packet_size];
        let mut icmp = MutableEchoRequestPacket::new(&mut buf[..])
            .ok_or(RingError::InvalidBufferSize)?;
        icmp.set_icmp_type(IcmpTypes::EchoRequest);
        icmp.set_icmp_code(IcmpCodes::NoCode);
        icmp.set_sequence_number(self.config.sequence + seq_offset);
        icmp.set_identifier(self.config.id);
        icmp.set_checksum(util::checksum(icmp.packet(), 1));

        let start = Instant::now();

        // send request
        self.socket.send_to(icmp.packet_mut(), &self.dest.into())?;

        // handle recv
        let mut mem_buf = unsafe { &mut *(buf.as_mut_slice() as *mut [u8] as *mut [std::mem::MaybeUninit<u8>]) };
        let (size, _) = self.socket.recv_from(&mut mem_buf)?;

        let duration = start.elapsed().as_micros() as f64 / 1000.0;
        let reply = EchoReplyPacket::new(&buf).ok_or(RingError::InvalidPacket)?;
        println!(
            "{} bytes from {}: icmp_seq={} ttl={} time={:.2}ms",
            size,
            self.config.destination.ip,
            reply.get_sequence_number(),
            self.config.ttl,
            duration
        );
        Ok(())
    }
}
