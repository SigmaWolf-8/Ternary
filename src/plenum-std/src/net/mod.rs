use crate::io::{self, Error, ErrorKind, Read, Write};
use alloc::string::String;

pub struct TcpStream;

impl TcpStream {
    pub fn connect<A>(_addr: A) -> io::Result<TcpStream> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    pub fn shutdown(&self, _how: Shutdown) -> io::Result<()> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    pub fn try_clone(&self) -> io::Result<TcpStream> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    pub fn set_nonblocking(&self, _nonblocking: bool) -> io::Result<()> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    pub fn set_nodelay(&self, _nodelay: bool) -> io::Result<()> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    pub fn set_ttl(&self, _ttl: u32) -> io::Result<()> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    pub fn ttl(&self) -> io::Result<u32> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }
}

impl Read for TcpStream {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }
}

impl Write for TcpStream {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }
}

pub struct TcpListener;

impl TcpListener {
    pub fn bind<A>(_addr: A) -> io::Result<TcpListener> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    pub fn set_nonblocking(&self, _nonblocking: bool) -> io::Result<()> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }
}

pub struct UdpSocket;

impl UdpSocket {
    pub fn bind<A>(_addr: A) -> io::Result<UdpSocket> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    pub fn send_to<A>(&self, _buf: &[u8], _addr: A) -> io::Result<usize> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    pub fn recv_from(&self, _buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        Err(Error::new(ErrorKind::Unsupported, "use z=0 distributor"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shutdown {
    Read,
    Write,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ipv4Addr {
    octets: [u8; 4],
}

impl Ipv4Addr {
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self { octets: [a, b, c, d] }
    }

    pub const LOCALHOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
    pub const UNSPECIFIED: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
    pub const BROADCAST: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 255);

    pub fn octets(&self) -> [u8; 4] {
        self.octets
    }

    pub fn is_loopback(&self) -> bool {
        self.octets[0] == 127
    }

    pub fn is_unspecified(&self) -> bool {
        self.octets == [0, 0, 0, 0]
    }
}

impl core::fmt::Display for Ipv4Addr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.octets[0], self.octets[1], self.octets[2], self.octets[3])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ipv6Addr {
    segments: [u16; 8],
}

impl Ipv6Addr {
    pub const fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> Self {
        Self { segments: [a, b, c, d, e, f, g, h] }
    }

    pub const LOCALHOST: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
    pub const UNSPECIFIED: Ipv6Addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0);

    pub fn segments(&self) -> [u16; 8] {
        self.segments
    }

    pub fn is_loopback(&self) -> bool {
        self.segments == [0, 0, 0, 0, 0, 0, 0, 1]
    }

    pub fn is_unspecified(&self) -> bool {
        self.segments == [0; 8]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IpAddr {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}

impl IpAddr {
    pub fn is_loopback(&self) -> bool {
        match self {
            IpAddr::V4(v4) => v4.is_loopback(),
            IpAddr::V6(v6) => v6.is_loopback(),
        }
    }

    pub fn is_unspecified(&self) -> bool {
        match self {
            IpAddr::V4(v4) => v4.is_unspecified(),
            IpAddr::V6(v6) => v6.is_unspecified(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SocketAddr {
    ip: IpAddr,
    port: u16,
}

impl SocketAddr {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self { ip, port }
    }

    pub fn ip(&self) -> &IpAddr {
        &self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl core::fmt::Display for SocketAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.ip {
            IpAddr::V4(v4) => write!(f, "{}:{}", v4, self.port),
            IpAddr::V6(_v6) => write!(f, "[::1]:{}", self.port),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SocketAddrV4 {
    ip: Ipv4Addr,
    port: u16,
}

impl SocketAddrV4 {
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        Self { ip, port }
    }

    pub fn ip(&self) -> &Ipv4Addr {
        &self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

pub trait ToSocketAddrs {
    type Iter: Iterator<Item = SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter>;
}

impl ToSocketAddrs for SocketAddr {
    type Iter = core::iter::Once<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<core::iter::Once<SocketAddr>> {
        Ok(core::iter::once(*self))
    }
}

impl ToSocketAddrs for &str {
    type Iter = core::iter::Empty<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<core::iter::Empty<SocketAddr>> {
        Err(Error::new(ErrorKind::Unsupported, "DNS resolution unsupported in kernel"))
    }
}

impl ToSocketAddrs for (String, u16) {
    type Iter = core::iter::Empty<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<core::iter::Empty<SocketAddr>> {
        Err(Error::new(ErrorKind::Unsupported, "DNS resolution unsupported in kernel"))
    }
}

impl ToSocketAddrs for (&str, u16) {
    type Iter = core::iter::Empty<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<core::iter::Empty<SocketAddr>> {
        Err(Error::new(ErrorKind::Unsupported, "DNS resolution unsupported in kernel"))
    }
}
