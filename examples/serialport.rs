use sernet::*;

use std::{
    io::{Read, Write},
    time::Duration,
};

use serialport::SerialPort;

fn main() {
    println!("Started program");

    let (read, write) = new_serial("/dev/ttyS0", 9600);

    let intf = IpTun::new("tun0");

    intf.start_forwarding(read, write);
}

pub fn new_serial(path: &str, baud: u32) -> (impl Read, impl Write) {
    let reader = serialport::new(path, baud)
        .timeout(Duration::from_millis(5))
        .open_native()
        .expect("Failed to open port");

    let writer = reader.try_clone().unwrap();

    (reader, writer)
}
