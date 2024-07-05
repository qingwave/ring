use std::net::{self, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use pnet::packet::icmp::echo_reply::EchoReplyPacket;
use pnet::packet::icmp::echo_request::{IcmpCodes, MutableEchoRequestPacket};
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::{util, MutablePacket, Packet};

use signal_hook::consts::{SIGINT, SIGTERM};

use socket2::{Domain, Protocol, Socket, Type};

use crossbeam_channel::{self, bounded, select, Receiver};

use crate::config::Config;
use crate::error::RingError;

#[derive(Clone)]
pub struct Pinger {
    config: Config,
    dest: SocketAddr,
    socket: Arc<Socket>,
}

impl Pinger {
    pub fn new(config: Config) -> std::io::Result<Self> {
        // Type::DGRAW with ICMP only support on linux
        let socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?;
        let src = SocketAddr::new(net::IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
        let dest = SocketAddr::new(config.destination.ip, 0);
        socket.bind(&src.into())?;
        socket.set_ttl(config.ttl)?;
        socket.set_read_timeout(Some(Duration::from_secs(config.timeout)))?;
        socket.set_write_timeout(Some(Duration::from_secs(config.timeout)))?;
        Ok(Self {
            config: config,
            dest: dest,
            socket: Arc::new(socket),
        })
    }

    pub fn run(&self) -> std::io::Result<()> {
        println!(
            "PING {}({})",
            self.config.destination.raw, self.config.destination.ip
        );
        let now = Instant::now();

        let send = Arc::new(AtomicU64::new(0));
        let _send = send.clone();
        let this = Arc::new(self.clone());
        let (sx, rx) = bounded(this.config.count as usize);
        thread::spawn(move || {
            for i in 0..this.config.count {
                let _this = this.clone();
                sx.send(thread::spawn(move || _this.ping(i))).unwrap();

                _send.fetch_add(1, Ordering::SeqCst);

                if i < this.config.count - 1 {
                    thread::sleep(Duration::from_millis(this.config.interval));
                }
            }
            drop(sx);
        });

        let success = Arc::new(AtomicU64::new(0));
        let _success = success.clone();
        let (summary_s, summary_r) = bounded(1);
        thread::spawn(move || {
            for handle in rx.iter() {
                if let Some(res) = handle.join().ok() {
                    if res.is_ok() {
                        _success.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
            summary_s.send(()).unwrap();
        });

        let stop = signal_notify()?;
        select!(
            recv(stop) -> sig => {
                if let Some(s) = sig.ok() {
                    println!("Receive signal {:?}", s);
                }
            },
            recv(summary_r) -> summary => {
                if let Some(e) = summary.err() {
                    println!("Error on summary: {}", e);
                }
            },
        );

        let total = now.elapsed().as_micros() as f64 / 1000.0;
        let send = send.load(Ordering::SeqCst);
        let success = success.load(Ordering::SeqCst);
        let loss_rate = if send > 0 {
            (send - success) * 100 / send
        } else {
            0
        };
        println!("\n--- {} ping statistics ---", self.config.destination.raw);
        println!(
            "{} packets transmitted, {} received, {}% packet loss, time {}ms",
            send, success, loss_rate, total,
        );
        Ok(())
    }

    pub fn ping(&self, seq_offset: u16) -> anyhow::Result<()> {
        // create icmp request packet
        let mut buf = vec![0; self.config.packet_size];
        let mut icmp =
            MutableEchoRequestPacket::new(&mut buf[..]).ok_or(RingError::InvalidBufferSize)?;
        icmp.set_icmp_type(IcmpTypes::EchoRequest);
        icmp.set_icmp_code(IcmpCodes::NoCode);
        icmp.set_sequence_number(self.config.sequence + seq_offset);
        icmp.set_identifier(self.config.id);
        icmp.set_checksum(util::checksum(icmp.packet(), 1));

        let start = Instant::now();

        // send request
        self.socket.send_to(icmp.packet_mut(), &self.dest.into())?;

        // handle recv
        let mut mem_buf =
            unsafe { &mut *(buf.as_mut_slice() as *mut [u8] as *mut [std::mem::MaybeUninit<u8>]) };
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

fn signal_notify() -> std::io::Result<Receiver<i32>> {
    let (s, r) = bounded(1);

    let mut signals = signal_hook::iterator::Signals::new(&[SIGINT, SIGTERM])?;

    thread::spawn(move || {
        for signal in signals.forever() {
            s.send(signal).unwrap();
            break;
        }
    });

    Ok(r)
}
