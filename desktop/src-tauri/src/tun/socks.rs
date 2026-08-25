//! Minimal SOCKS5 client (RFC 1928) used to hand tunnelled flows to Aether.
//!
//! Aether's local listener needs no authentication, so this only implements the
//! `NO AUTH` handshake plus CONNECT and UDP ASSOCIATE. Writing it here instead of
//! pulling a crate keeps the dependency surface small and, more importantly, lets
//! UDP relaying work exactly the way the TUN path needs it: one relay socket per
//! flow, with the SOCKS5 datagram header added and stripped in place.

use std::io;
use std::net::{IpAddr, SocketAddr};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};

const VERSION: u8 = 0x05;
const NO_AUTH: u8 = 0x00;
const CMD_CONNECT: u8 = 0x01;
const CMD_UDP_ASSOCIATE: u8 = 0x03;
const ATYP_V4: u8 = 0x01;
const ATYP_DOMAIN: u8 = 0x03;
const ATYP_V6: u8 = 0x04;
const RESERVED: u8 = 0x00;

fn reply_message(code: u8) -> &'static str {
    match code {
        0x01 => "general SOCKS server failure",
        0x02 => "connection not allowed by ruleset",
        0x03 => "network unreachable",
        0x04 => "host unreachable",
        0x05 => "connection refused",
        0x06 => "TTL expired",
        0x07 => "command not supported",
        0x08 => "address type not supported",
        _ => "unknown SOCKS5 error",
    }
}

/// Complete the greeting and assert the server accepted `NO AUTH`.
async fn greet(stream: &mut TcpStream) -> io::Result<()> {
    stream.write_all(&[VERSION, 0x01, NO_AUTH]).await?;
    let mut reply = [0u8; 2];
    stream.read_exact(&mut reply).await?;
    if reply[0] != VERSION {
        return Err(io::Error::other(format!(
            "unexpected SOCKS version {:#x} from the proxy",
            reply[0]
        )));
    }
    if reply[1] != NO_AUTH {
        return Err(io::Error::other(
            "the SOCKS5 proxy demanded authentication, which Aether's listener never does",
        ));
    }
    Ok(())
}

fn encode_addr(target: SocketAddr, out: &mut Vec<u8>) {
    match target.ip() {
        IpAddr::V4(v4) => {
            out.push(ATYP_V4);
            out.extend_from_slice(&v4.octets());
        }
        IpAddr::V6(v6) => {
            out.push(ATYP_V6);
            out.extend_from_slice(&v6.octets());
        }
    }
    out.extend_from_slice(&target.port().to_be_bytes());
}

/// Read a reply's bound address, which we must consume even when unused.
async fn read_bound_addr(stream: &mut TcpStream) -> io::Result<SocketAddr> {
    let mut head = [0u8; 4];
    stream.read_exact(&mut head).await?;

    if head[0] != VERSION {
        return Err(io::Error::other("malformed SOCKS5 reply"));
    }
    if head[1] != 0x00 {
        return Err(io::Error::other(format!(
            "SOCKS5 request rejected: {}",
            reply_message(head[1])
        )));
    }

    let ip = match head[3] {
        ATYP_V4 => {
            let mut octets = [0u8; 4];
            stream.read_exact(&mut octets).await?;
            IpAddr::from(octets)
        }
        ATYP_V6 => {
            let mut octets = [0u8; 16];
            stream.read_exact(&mut octets).await?;
            IpAddr::from(octets)
        }
        ATYP_DOMAIN => {
            let len = stream.read_u8().await? as usize;
            let mut name = vec![0u8; len];
            stream.read_exact(&mut name).await?;
            // A domain in a BND.ADDR is legal but useless to us; fall back to
            // the proxy's own address, which the caller already knows.
            IpAddr::from([0, 0, 0, 0])
        }
        other => {
            return Err(io::Error::other(format!(
                "unsupported SOCKS5 address type {other:#x}"
            )))
        }
    };

    let port = stream.read_u16().await?;
    Ok(SocketAddr::new(ip, port))
}

/// Open a tunnelled TCP connection to `target` through `proxy`.
pub async fn connect(proxy: SocketAddr, target: SocketAddr) -> io::Result<TcpStream> {
    let mut stream = TcpStream::connect(proxy).await?;
    stream.set_nodelay(true)?;
    greet(&mut stream).await?;

    let mut request = vec![VERSION, CMD_CONNECT, RESERVED];
    encode_addr(target, &mut request);
    stream.write_all(&request).await?;

    read_bound_addr(&mut stream).await?;
    Ok(stream)
}

/// A UDP flow relayed through the proxy.
///
/// The TCP control stream must stay open for the association's whole life, so it
/// is kept here even though nothing is ever read from it again.
pub struct UdpRelay {
    _control: TcpStream,
    socket: UdpSocket,
    relay: SocketAddr,
    target: SocketAddr,
}

impl UdpRelay {
    /// Establish a UDP association for datagrams addressed to `target`.
    pub async fn open(proxy: SocketAddr, target: SocketAddr) -> io::Result<Self> {
        let mut control = TcpStream::connect(proxy).await?;
        control.set_nodelay(true)?;
        greet(&mut control).await?;

        // All-zero DST tells the proxy we do not know our source port yet, which
        // is the normal case for a client behind its own NAT.
        let mut request = vec![VERSION, CMD_UDP_ASSOCIATE, RESERVED];
        let unspecified = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 0);
        encode_addr(unspecified, &mut request);
        control.write_all(&request).await?;

        let bound = read_bound_addr(&mut control).await?;

        // Servers commonly reply with 0.0.0.0 meaning "same host as the control
        // connection". Substitute the proxy address so the socket is usable.
        let relay = if bound.ip().is_unspecified() {
            SocketAddr::new(proxy.ip(), bound.port())
        } else {
            bound
        };

        let bind: SocketAddr = if relay.is_ipv4() {
            "0.0.0.0:0".parse().unwrap()
        } else {
            "[::]:0".parse().unwrap()
        };
        let socket = UdpSocket::bind(bind).await?;
        socket.connect(relay).await?;

        Ok(Self {
            _control: control,
            socket,
            relay,
            target,
        })
    }

    /// Wrap `payload` in a SOCKS5 datagram header and send it.
    pub async fn send(&self, payload: &[u8]) -> io::Result<()> {
        let mut datagram = Vec::with_capacity(payload.len() + 22);
        datagram.extend_from_slice(&[RESERVED, RESERVED, 0x00]); // RSV RSV FRAG
        encode_addr(self.target, &mut datagram);
        datagram.extend_from_slice(payload);
        self.socket.send(&datagram).await?;
        Ok(())
    }

    /// Receive one datagram and strip its SOCKS5 header.
    pub async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut raw = vec![0u8; buf.len() + 262];
        let n = self.socket.recv(&mut raw).await?;
        let body = strip_udp_header(&raw[..n])?;
        let len = body.len().min(buf.len());
        buf[..len].copy_from_slice(&body[..len]);
        Ok(len)
    }

    pub fn relay_addr(&self) -> SocketAddr {
        self.relay
    }
}

/// Remove the RSV/FRAG/ADDR prefix from a relayed datagram.
fn strip_udp_header(datagram: &[u8]) -> io::Result<&[u8]> {
    if datagram.len() < 5 {
        return Err(io::Error::other("truncated SOCKS5 UDP datagram"));
    }
    if datagram[2] != 0x00 {
        // Fragmented datagrams are legal in the RFC and universally unsupported.
        return Err(io::Error::other("fragmented SOCKS5 UDP datagram"));
    }

    let offset = match datagram[3] {
        ATYP_V4 => 4 + 4 + 2,
        ATYP_V6 => 4 + 16 + 2,
        ATYP_DOMAIN => 4 + 1 + datagram[4] as usize + 2,
        other => {
            return Err(io::Error::other(format!(
                "unsupported address type {other:#x} in a UDP datagram"
            )))
        }
    };

    datagram
        .get(offset..)
        .ok_or_else(|| io::Error::other("SOCKS5 UDP datagram shorter than its header"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ipv4_target_is_encoded_as_atyp_1() {
        let mut out = Vec::new();
        encode_addr("1.2.3.4:443".parse().unwrap(), &mut out);
        assert_eq!(out, vec![ATYP_V4, 1, 2, 3, 4, 0x01, 0xBB]);
    }

    #[test]
    fn a_v4_udp_header_is_ten_bytes_long() {
        let mut datagram = vec![0, 0, 0, ATYP_V4, 8, 8, 8, 8, 0, 53];
        datagram.extend_from_slice(b"payload");
        assert_eq!(strip_udp_header(&datagram).unwrap(), b"payload");
    }

    #[test]
    fn a_fragmented_datagram_is_refused() {
        let datagram = vec![0, 0, 0x01, ATYP_V4, 8, 8, 8, 8, 0, 53, b'x'];
        assert!(strip_udp_header(&datagram).is_err());
    }

    #[test]
    fn a_domain_udp_header_accounts_for_the_length_byte() {
        let mut datagram = vec![0, 0, 0, ATYP_DOMAIN, 3, b'a', b'b', b'c', 0, 53];
        datagram.extend_from_slice(b"ok");
        assert_eq!(strip_udp_header(&datagram).unwrap(), b"ok");
    }
}
