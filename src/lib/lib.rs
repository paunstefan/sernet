use std::io;
use std::{
    io::{Read, Write},
    sync::Arc,
    thread,
};

use tun_tap::{Iface, Mode};

#[derive(Debug, Clone)]
pub struct IpTun {
    ifname: String,
    iface: Arc<Iface>,
}

impl IpTun {
    pub fn new(ifname: &str) -> Self {
        let iface = Iface::new(ifname, Mode::Tun).unwrap();

        IpTun {
            ifname: ifname.to_string(),
            iface: Arc::new(iface),
        }
    }

    pub fn start_forwarding(&self, ser_reader: impl Read + Send + 'static, ser_writer: impl Write) {
        let reader = (*self).clone();
        thread::spawn(move || reader.fwd_ser_ip(ser_reader));
        self.fwd_ip_ser(ser_writer);
    }

    fn fwd_ip_ser(&self, mut writer: impl Write) {
        let mut buffer = vec![0; 9100];
        loop {
            let size = self.iface.recv(&mut buffer).unwrap();
            assert!(size >= 4);
            let _flags = u16::from_be_bytes([buffer[0], buffer[1]]);
            let proto = u16::from_be_bytes([buffer[2], buffer[3]]);
            if proto != 0x0800 {
                continue;
            }
            println!(
                "Packet received from TUN:({} bytes) {:?}",
                &buffer[..size].len(),
                &buffer[..size]
            );

            send_serial_frame(&buffer[..size], &mut writer);
        }
    }

    fn fwd_ser_ip(&self, mut reader: impl Read) {
        let mut buffer = vec![0; 9100];
        loop {
            let size = read_ip_packet(&mut buffer, &mut reader);

            println!(
                "Packet received from serial port:({} bytes) {:?}",
                &buffer[..size].len(),
                &buffer[..size]
            );

            let _written = self.iface.send(&buffer[..size]).unwrap();
            println!("Written if: {}", _written);
        }
    }
}

fn send_serial_frame(buffer: &[u8], writer: &mut impl Write) {
    let size = buffer.len();
    assert!(size < u16::MAX as usize);
    let size = (size as u16).to_be_bytes();

    let mut frame = size.to_vec();
    frame.extend_from_slice(buffer);

    writer.write_all(&frame).unwrap()
}

fn read_ip_packet(buffer: &mut [u8], reader: &mut impl Read) -> usize {
    let mut size_buf = [0u8; 2];

    read_exact_no_timeout(&mut size_buf, reader).unwrap();
    let size = u16::from_be_bytes(size_buf) as usize;

    read_exact_no_timeout(&mut buffer[0..size], reader).unwrap();

    size
}

/// Workaround as the serial implementation has no infinite blocking mode
fn read_exact_no_timeout(buf: &mut [u8], reader: &mut impl Read) -> io::Result<()> {
    loop {
        match reader.read_exact(buf) {
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
            Err(e) => return Err(e),
        }
    }
}
