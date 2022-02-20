# sernet

Library for creating network interfaces over a serial port.

It can be useful for creating a network interface over a standard serial link between PCs. But the main use case and the reason I made it is so it can be used with different radio transmitters that connect to UART ports (such as the HC-12 module).

## Usage

The library works by first initializing the TUN interface, then giving it the 2 ends of a serial connection (that implement Read and Write). The same port can be cloned and used as both ends.

The `tun_tap` library works only on Linux, `sernet` is also incompatible with other OSes.

## Example

In the `examples` directory there is a `serialtun.rs` file that implements a simple usecase.

You can run it with:

```bash
cargo run --example serialtun -- --serial [serial device file]
```

It will create a `tun0` interface on the given serial port, with a baud rate of 57600.

The full options for it are:

```
-b, --baud <baud>        Baud rate [default: 57600]
-s, --serial <serial>    Serial device file
-t, --tun <tun>          Name of the TUN interface [default: tun0]
```

The interface is created but right now it doesn't have an address assigned.

```bash
sudo ip addr add [address]/[mask] dev tun0
sudo ip link set up dev tun0
```

Now the `tun0` interface can be used as any other network interface.