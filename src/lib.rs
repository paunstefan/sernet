use std::io;
use std::{
    io::{Read, Write},
    sync::Arc,
    thread,
};

use tun_tap::{Iface, Mode};

mod errors;

use errors::SernetError;

const IP_MAX_LEN: usize = 65535;

#[derive(Debug, PartialEq)]
enum EtherType {
    IpV4,
    IpV6,
    Unsupported,
}

impl From<u16> for EtherType {
    fn from(ethertype: u16) -> Self {
        match ethertype {
            0x0800 => EtherType::IpV4,
            0x86DD => EtherType::IpV6,
            _ => EtherType::Unsupported,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IpTun {
    ifname: String,
    iface: Arc<Iface>,
    // TODO: add options
}

impl IpTun {
    /// Create a new IP TUN interface
    pub fn new(ifname: &str) -> Result<Self, SernetError> {
        let iface = Iface::new(ifname, Mode::Tun)?;

        Ok(IpTun {
            ifname: ifname.to_string(),
            iface: Arc::new(iface),
        })
    }

    /// Start the IP forwarding
    /// The 2 directions are handled by different threads
    pub fn start_forwarding(&self, ser_reader: impl Read + Send + 'static, ser_writer: impl Write) {
        let reader = (*self).clone();
        thread::spawn(move || reader.fwd_ser_ip(ser_reader));
        self.fwd_ip_ser(ser_writer);
    }

    /// Loop that forwards packets from IP tun to serial
    fn fwd_ip_ser(&self, mut writer: impl Write) {
        let mut buffer = vec![0; IP_MAX_LEN];
        loop {
            let size = match self.iface.recv(&mut buffer) {
                Ok(size) => size,
                Err(e) => {
                    eprintln!("{:?}", e);
                    continue;
                }
            };

            // TUN packet flags and protocol size
            if size < 4 {
                continue;
            }

            let _flags = u16::from_be_bytes([buffer[0], buffer[1]]);
            let proto = EtherType::from(u16::from_be_bytes([buffer[2], buffer[3]]));
            if proto != EtherType::IpV4 {
                continue;
            }

            if let Err(e) = send_serial_frame(&buffer[..size], &mut writer) {
                eprintln!("{:?}", e);
            }
        }
    }

    /// Loop that forwards packets from serial to IP tun
    fn fwd_ser_ip(&self, mut reader: impl Read) {
        let mut buffer = vec![0; IP_MAX_LEN];
        loop {
            let size = match read_ip_packet(&mut buffer, &mut reader) {
                Ok(size) => size,
                Err(e) => {
                    eprintln!("{:?}", e);
                    continue;
                }
            };

            if size < 4 {
                continue;
            }

            let _flags = u16::from_be_bytes([buffer[0], buffer[1]]);
            let proto = EtherType::from(u16::from_be_bytes([buffer[2], buffer[3]]));
            if proto != EtherType::IpV4 {
                continue;
            }

            let _sent = match self.iface.send(&buffer[..size]) {
                Ok(size) => size,
                Err(e) => {
                    eprintln!("{:?}", e);
                    continue;
                }
            };
        }
    }
}

/// Sends IP packet over serial. Prepends packet size.
fn send_serial_frame(buffer: &[u8], writer: &mut impl Write) -> Result<(), SernetError> {
    let size = buffer.len();
    if size > u16::MAX as usize {
        return Err(SernetError::OtherError);
    }
    let size_buf = (size as u16).to_be_bytes();

    writer.write_all(&size_buf)?;
    writer.write_all(buffer)?;

    Ok(())
}

/// Read IP packet from serial
fn read_ip_packet(buffer: &mut [u8], reader: &mut impl Read) -> Result<usize, SernetError> {
    let mut size_buf = [0u8; 2];

    read_exact_no_timeout(&mut size_buf, reader)?;
    let size = u16::from_be_bytes(size_buf) as usize;

    read_exact_no_timeout(&mut buffer[0..size], reader)?;

    Ok(size)
}

/// read_exact but ignores TimedOut errors
/// Workaround as the serial library doesn't yet support unlimited blocking
fn read_exact_no_timeout(buf: &mut [u8], reader: &mut impl Read) -> io::Result<()> {
    let to_read = buf.len();
    let mut read = 0;

    loop {
        match reader.read(&mut buf[read..to_read]) {
            Ok(size) => {
                read += size;
                if read == to_read {
                    return Ok(());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
            Err(e) => return Err(e),
        }
    }
}
