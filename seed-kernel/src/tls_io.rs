use core::fmt;
use core::hint::spin_loop;

use embedded_io::{Error, ErrorKind, ErrorType, Read, Write};

use crate::{net, time};

const DEFAULT_READ_TIMEOUT_MS: u64 = 60_000;
const DEFAULT_WRITE_TIMEOUT_MS: u64 = 15_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TcpStreamError {
    TimedOut,
    NotConnected,
    Closed,
    LeaseDenied,
}

impl fmt::Display for TcpStreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for TcpStreamError {}

impl Error for TcpStreamError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::TimedOut => ErrorKind::TimedOut,
            Self::NotConnected => ErrorKind::NotConnected,
            Self::Closed => ErrorKind::ConnectionAborted,
            Self::LeaseDenied => ErrorKind::PermissionDenied,
        }
    }
}

#[derive(Clone, Copy)]
pub struct KernelTcpStream {
    lease: net::TransportLeaseToken,
    read_timeout_ms: u64,
    write_timeout_ms: u64,
}

impl KernelTcpStream {
    pub const fn new(lease: net::TransportLeaseToken) -> Self {
        Self {
            lease,
            read_timeout_ms: DEFAULT_READ_TIMEOUT_MS,
            write_timeout_ms: DEFAULT_WRITE_TIMEOUT_MS,
        }
    }

    fn wait_for<F>(&self, timeout_ms: u64, mut action: F) -> Result<usize, TcpStreamError>
    where
        F: FnMut(
            net::TransportLeaseToken,
            u64,
        ) -> Result<net::TcpIoResult, net::TransportLeaseError>,
    {
        let start = now_ms();
        loop {
            net::poll();
            match action(self.lease, now_ms()) {
                Ok(net::TcpIoResult::Ready(count)) => return Ok(count),
                Ok(net::TcpIoResult::WouldBlock) => {}
                Ok(net::TcpIoResult::Closed) => return Err(TcpStreamError::Closed),
                Ok(net::TcpIoResult::Unavailable) => return Err(TcpStreamError::NotConnected),
                Err(net::TransportLeaseError::LeaseTimedOut) => {
                    return Err(TcpStreamError::TimedOut)
                }
                Err(_) => return Err(TcpStreamError::LeaseDenied),
            }

            if now_ms().saturating_sub(start) >= timeout_ms {
                return Err(TcpStreamError::TimedOut);
            }
            spin_loop();
        }
    }
}

impl ErrorType for KernelTcpStream {
    type Error = TcpStreamError;
}

impl Read for KernelTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        self.wait_for(self.read_timeout_ms, |lease, now| {
            net::tcp_recv_step(lease, buf, now)
        })
    }
}

impl Write for KernelTcpStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        self.wait_for(self.write_timeout_ms, |lease, now| {
            net::tcp_send_step(lease, buf, now)
        })
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // NET-3 debt: embedded-tls still requires this bounded blocking adapter;
        // NET-4 must not use it as evidence of main-loop responsiveness.
        for _ in 0..8 {
            net::poll();
            net::tcp_receive_inspect_step(self.lease, now_ms())
                .map_err(|_| TcpStreamError::LeaseDenied)?;
            spin_loop();
        }
        Ok(())
    }
}

fn now_ms() -> u64 {
    let per_ms = time::tsc_per_ms().max(1);
    time::rdtsc() / per_ms
}
