#!/bin/bash
cargo b --release
ext=$?
if [[ $ext -ne 0 ]]; then
	exit $ext
fi
sudo setcap cap_net_admin=eip ./release/sernet
./release/sernet &
pid=$!
sudo ip addr add 10.10.0.2/24 dev testtun0
sudo ip link set up dev testtun0
trap "kill $pid" INT TERM
wait $pid