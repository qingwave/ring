# Ring

Ring is the ping but with Rust, rust + ping -> ring, implement by `pnet` and `socket2`.

## Build

```bash
cargo build
```

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

```
cargo run 8.8.8.8
```

ping a domain
```
cargo run www.github.com
```