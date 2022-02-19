use std::{
    io::{Read, Write},
    time::Duration,
};

use serialport::SerialPort;

pub fn new_serial(path: &str, baud: u32) -> (impl Read, impl Write) {
    let reader = serialport::new(path, baud)
        .timeout(Duration::from_millis(5))
        .open_native()
        .expect("Failed to open port");

    let writer = reader.try_clone().unwrap();

    (reader, writer)
}
