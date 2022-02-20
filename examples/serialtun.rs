use serialport::SerialPort;
use std::path::PathBuf;
use std::{
    io::{Read, Write},
    time::Duration,
};
use structopt::StructOpt;

use sernet::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "serialtun")]
struct Options {
    /// Serial device file
    #[structopt(short, long, parse(from_os_str))]
    serial: PathBuf,

    /// Baud rate
    #[structopt(short, long, default_value = "57600")]
    baud: u32,

    /// Name of the TUN interface
    #[structopt(short, long, default_value = "tun0")]
    tun: String,
}

fn main() {
    let opt = Options::from_args();

    println!("--- Starting serialtun ---");
    println!(
        "Serial port: {}, baud rate: {}",
        opt.serial.to_str().unwrap(),
        opt.baud
    );
    println!("TUN interface: {}", opt.tun);

    let (read, write) = new_serial(opt.serial.to_str().unwrap(), opt.baud);

    let intf = IpTun::new(&opt.tun).unwrap();

    intf.start_forwarding(read, write);
}

pub fn new_serial(path: &str, baud: u32) -> (impl Read, impl Write) {
    let reader = serialport::new(path, baud)
        .timeout(Duration::from_millis(1000))
        .open_native()
        .expect("Failed to open port");

    let writer = reader.try_clone().expect("Failed to clone port");

    (reader, writer)
}
