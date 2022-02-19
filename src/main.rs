use sernet_lib::*;

mod serial;

fn main() {
    // // Create the tun interface.
    // let iface = Iface::new("testtun%d", Mode::Tun).unwrap();
    // eprintln!("Iface: {:?}", iface);

    // println!(
    //     "Created interface {}. Send some packets into it and see they're printed here",
    //     iface.name()
    // );
    // // That 1500 is a guess for the IFace's MTU (we probably could configure it explicitly). 4 more
    // // for TUN's „header“.
    // let mut buffer = vec![0; 1504];
    // loop {
    //     // Every read is one packet. If the buffer is too small, bad luck, it gets truncated.
    //     let size = iface.recv(&mut buffer).unwrap();
    //     assert!(size >= 4);
    //     let _flags = u16::from_be_bytes([buffer[0], buffer[1]]);
    //     let proto = u16::from_be_bytes([buffer[2], buffer[3]]);
    //     if proto != 0x0800 {
    //         continue;
    //     }
    //     println!("Packet: {:?}", &buffer[4..size]);
    // }
    println!("Started program");

    let (read, write) = serial::new_serial("/dev/ttyS0", 9600);

    let intf = IpTun::new("tun0");

    intf.start_forwarding(read, write);

    // let output = "This is a test. This is only a test.\n".as_bytes();
    // let _ = port.write(output).expect("Write failed!");
}
