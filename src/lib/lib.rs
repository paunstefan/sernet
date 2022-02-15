use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
    thread,
};

use tun_tap::{Iface, Mode};

#[derive(Debug, Clone)]
pub struct IpTun<S>
where
    S: Read + Write + Clone + Send + Sync,
{
    ifname: String,
    iface: Arc<Iface>,
    ext_port: Arc<Mutex<S>>,
}

impl<S> IpTun<S>
where
    S: Read + Write + Clone + Send + Sync,
    S: 'static,
{
    pub fn new(ifname: String, port: S) -> Self {
        let iface = Iface::new(&ifname, Mode::Tun).unwrap();

        IpTun {
            ifname,
            iface: Arc::new(iface),
            ext_port: Arc::new(Mutex::new(port)),
        }
    }

    pub fn start_forwarding(&self) {
        let reader = (*self).clone();
        thread::spawn(move || reader.fwd_ser_ip());
        self.fwd_ip_ser();
    }

    fn fwd_ip_ser(&self) {
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
                &buffer[4..size].len(),
                &buffer[4..size]
            );

            let mut port = self.ext_port.lock().unwrap();
            let _written = port.write(&buffer[4..size]).unwrap();
        }
    }

    fn fwd_ser_ip(&self) {
        let mut buffer = vec![0; 9100];
        loop {
            {
                let mut port = self.ext_port.lock().unwrap();
                let _size = port.read(&mut buffer).unwrap();
            }

            println!(
                "Packet received from serial port:({} bytes) {:?}",
                &buffer[..].len(),
                &buffer[..]
            );

            let _written = self.iface.send(&buffer[..]).unwrap();
        }
    }
}
