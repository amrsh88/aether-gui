//! Byte-counting TCP relay used in System Proxy mode.
//!
//! In Full Tunnel mode every byte crosses the TUN adapter, so throughput can be
//! measured there. Proxy mode has no such choke point — applications talk to
//! Aether's SOCKS5 listener directly and the GUI never sees the traffic.
//!
//! Rather than inventing numbers or leaving the meter dead, this module inserts a
//! transparent relay in front of Aether: the system proxy points at *our* port, we
//! forward raw bytes to Aether's real listener, and count them on the way past.
//! SOCKS5 is a protocol carried inside the TCP stream, so a byte-for-byte copy
//! passes the handshake through untouched — no parsing, no interference.
//!
//! One caveat worth stating plainly: a client using UDP ASSOCIATE receives Aether's
//! own relay address in the reply and will send its datagrams straight there,
//! bypassing this relay. Those bytes go uncounted. They still work, and in proxy
//! mode almost nothing uses SOCKS5 UDP anyway.

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;

use crate::tun::adapter::Counters;

/// A running relay. Cancel the token to stop it.
pub struct Relay {
    counters: Arc<Counters>,
    cancel: CancellationToken,
    task: tokio::task::JoinHandle<()>,
    listen: SocketAddr,
}

impl Relay {
    /// Listen on `listen` and forward everything to `upstream`.
    pub async fn start(listen: SocketAddr, upstream: SocketAddr) -> io::Result<Self> {
        let listener = TcpListener::bind(listen).await.map_err(|e| {
            io::Error::other(format!(
                "could not listen on {listen}: {e}. Is another copy of Aether already running?"
            ))
        })?;

        // Binding to port 0 is legal, so report what we actually got.
        let listen = listener.local_addr()?;

        let counters = Arc::new(Counters::default());
        let cancel = CancellationToken::new();

        let task = tokio::spawn(accept_loop(
            listener,
            upstream,
            counters.clone(),
            cancel.clone(),
        ));

        log::info!("[+] proxy relay listening on {listen} -> {upstream}");

        Ok(Self {
            counters,
            cancel,
            task,
            listen,
        })
    }

    pub fn counters(&self) -> Arc<Counters> {
        self.counters.clone()
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.listen
    }

    pub async fn shutdown(self) {
        self.cancel.cancel();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(2), self.task).await;
        log::info!("[+] proxy relay stopped");
    }
}

async fn accept_loop(
    listener: TcpListener,
    upstream: SocketAddr,
    counters: Arc<Counters>,
    cancel: CancellationToken,
) {
    loop {
        let (client, _peer) = tokio::select! {
            biased;
            _ = cancel.cancelled() => break,
            accepted = listener.accept() => match accepted {
                Ok(pair) => pair,
                Err(e) => {
                    log::warn!("[!] relay accept failed: {e}");
                    continue;
                }
            },
        };

        let counters = counters.clone();
        let cancel = cancel.clone();
        tokio::spawn(async move {
            if let Err(e) = splice(client, upstream, counters, cancel).await {
                log::trace!("[.] relay connection closed: {e}");
            }
        });
    }
}

/// Copy one connection in both directions, tallying bytes as they pass.
async fn splice(
    client: TcpStream,
    upstream: SocketAddr,
    counters: Arc<Counters>,
    cancel: CancellationToken,
) -> io::Result<()> {
    let server = TcpStream::connect(upstream).await?;
    client.set_nodelay(true)?;
    server.set_nodelay(true)?;

    let (mut client_read, mut client_write) = client.into_split();
    let (mut server_read, mut server_write) = server.into_split();

    let up_counter = counters.clone();
    let up = async move {
        // Client -> Aether: outbound, so it lands in the "up" counter.
        copy_counting(&mut client_read, &mut server_write, &up_counter.up).await
    };

    let down_counter = counters.clone();
    let down = async move {
        // Aether -> client: inbound.
        copy_counting(&mut server_read, &mut client_write, &down_counter.down).await
    };

    tokio::select! {
        _ = cancel.cancelled() => Ok(()),
        // Either direction finishing ends the connection, which is what a plain
        // TCP proxy should do — a half-open SOCKS stream is never useful.
        result = up => result.map(|_| ()),
        result = down => result.map(|_| ()),
    }
}

/// `tokio::io::copy` with a counter bumped on every chunk.
async fn copy_counting<R, W>(
    reader: &mut R,
    writer: &mut W,
    counter: &std::sync::atomic::AtomicU64,
) -> io::Result<u64>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    use std::sync::atomic::Ordering;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buffer = vec![0u8; 32 * 1024];
    let mut total = 0u64;

    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        writer.write_all(&buffer[..read]).await?;
        counter.fetch_add(read as u64, Ordering::Relaxed);
        total += read as u64;
    }

    let _ = writer.shutdown().await;
    Ok(total)
}
