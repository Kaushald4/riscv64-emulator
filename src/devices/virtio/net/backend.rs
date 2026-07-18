#[cfg(target_os = "linux")]
use std::os::fd::OwnedFd;

use std::io;

pub enum NetworkError {
    Io,
    WouldBlock,
}

pub trait NetworkBackend: Send {
    fn mac_address(&self) -> [u8; 6];
    fn send(&mut self, frame: &[u8]) -> Result<(), NetworkError>;
    fn receive(&mut self, buffer: &mut [u8]) -> Result<Option<usize>, NetworkError>;
}

pub struct DummyBackend {
    mac: [u8; 6],
}

impl DummyBackend {
    pub fn new() -> Self {
        Self { mac: [0x52, 0x54, 0x00, 0x12, 0x34, 0x56] }
    }
}

impl NetworkBackend for DummyBackend {
    fn mac_address(&self) -> [u8; 6] {
        self.mac
    }

    fn send(&mut self, _frame: &[u8]) -> Result<(), NetworkError> {
        Ok(())
    }

    fn receive(&mut self, _buffer: &mut [u8]) -> Result<Option<usize>, NetworkError> {
        Ok(None)
    }
}

// ── Linux TAP backend ──

#[cfg(target_os = "linux")]
#[repr(C)]
struct IfReq {
    name: [u8; 16],
    flags: u16,
    _pad: [u8; 22],
}

#[cfg(target_os = "linux")]
const TUNSETIFF: u64 = 0x4004_54ca;
#[cfg(target_os = "linux")]
const IFF_TAP: u16 = 0x0002;
#[cfg(target_os = "linux")]
const IFF_NO_PI: u16 = 0x1000;

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn open(path: *const u8, flags: i32) -> i32;
    fn close(fd: i32) -> i32;
    fn ioctl(fd: i32, request: u64, arg: *const IfReq) -> i32;
    fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
}

#[cfg(target_os = "linux")]
pub struct TapBackend {
    fd: OwnedFd,
    mac: [u8; 6],
}

#[cfg(not(target_os = "linux"))]
pub struct TapBackend {
    mac: [u8; 6],
}

#[cfg(target_os = "linux")]
impl TapBackend {
    /// Creates a TAP device. Requires CAP_NET_ADMIN or root.
    /// Returns `Err` if `/dev/net/tun` is not accessible.
    pub fn new(name: &str) -> io::Result<Self> {
        use std::os::unix::io::FromRawFd;

        let fd = unsafe { open(b"/dev/net/tun\0".as_ptr(), 0o2 | 0o4000) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let mut ifr = IfReq { name: [0u8; 16], flags: IFF_TAP | IFF_NO_PI, _pad: [0u8; 22] };
        let name_bytes = name.as_bytes();
        let copy_len = name_bytes.len().min(15);
        ifr.name[..copy_len].copy_from_slice(&name_bytes[..copy_len]);

        let ret = unsafe { ioctl(fd, TUNSETIFF, &ifr) };
        if ret < 0 {
            unsafe {
                close(fd);
            }
            return Err(io::Error::last_os_error());
        }

        Ok(Self {
            fd: unsafe { OwnedFd::from_raw_fd(fd) },
            mac: [0x52, 0x54, 0x00, 0x12, 0x34, 0x56],
        })
    }
}

#[cfg(target_os = "linux")]
impl NetworkBackend for TapBackend {
    fn mac_address(&self) -> [u8; 6] {
        self.mac
    }

    fn send(&mut self, frame: &[u8]) -> Result<(), NetworkError> {
        use std::os::unix::io::AsRawFd;
        let ret = unsafe { write(self.fd.as_raw_fd(), frame.as_ptr(), frame.len()) };
        if ret < 0 {
            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::WouldBlock {
                return Ok(());
            }
            return Err(NetworkError::Io);
        }
        Ok(())
    }

    fn receive(&mut self, buffer: &mut [u8]) -> Result<Option<usize>, NetworkError> {
        use std::os::unix::io::AsRawFd;
        let ret = unsafe { read(self.fd.as_raw_fd(), buffer.as_mut_ptr(), buffer.len()) };
        if ret < 0 {
            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::WouldBlock {
                return Ok(None);
            }
            return Err(NetworkError::Io);
        }
        if ret == 0 {
            return Ok(None);
        }
        Ok(Some(ret as usize))
    }
}

#[cfg(not(target_os = "linux"))]
impl TapBackend {
    pub fn new(_name: &str) -> io::Result<Self> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "TAP backend requires Linux"))
    }
}

#[cfg(not(target_os = "linux"))]
impl NetworkBackend for TapBackend {
    fn mac_address(&self) -> [u8; 6] {
        self.mac
    }

    fn send(&mut self, _frame: &[u8]) -> Result<(), NetworkError> {
        Ok(())
    }

    fn receive(&mut self, _buffer: &mut [u8]) -> Result<Option<usize>, NetworkError> {
        Ok(None)
    }
}
