# Ring

Ring is the ping but with Rust, rust + ping -> ring, implement by `pnet`, `socket2` and `signal-hook`.

## Build

```bash
cargo build --release
cargo install --path .
sudo setcap cap_net_raw=+eip $(which ring) 
```

The `ring` need network privileges to create raw sockets.

If you want use `ring` as non-root mode in linux, you need change [socket type](./src/ping.rs) from `Type::RAW` to `Type::DGRAW` 

## Usage

```
Usage: ring [OPTIONS] <DESTINATION>

Arguments:
  <DESTINATION>  Ping destination, ip or domain

Options:
  -c <COUNT>            Count of ping times [default: 4]
  -s <PACKET_SIZE>      Ping packet size [default: 64]
  -t <TTL>              Ping ttl [default: 64]
  -w <TIMEOUT>          Ping timeout seconds [default: 1]
  -i <INTERVAL>         Ping interval duration milliseconds [default: 1000]
  -h, --help            Print help information
  -V, --version         Print version information
```

ping a ip address.

```bash
cargo run 8.8.8.8
```

ping a domain
```bash
cargo run www.github.com
```

ping and interrupt by Crtl+C
```bash
cargo run 8.8.8.8 -c 10

PING 8.8.8.8(8.8.8.8)
64 bytes from 8.8.8.8: icmp_seq=1 ttl=64 time=4.32ms
64 bytes from 8.8.8.8: icmp_seq=2 ttl=64 time=3.02ms
64 bytes from 8.8.8.8: icmp_seq=3 ttl=64 time=3.24ms
^CReceive signal 2

--- 8.8.8.8 ping statistics ---
3 packets transmitted, 3 received, 0% packet loss, time 2365.104ms
```