#!/bin/bash

set -euxo pipefail

(cd nrf-softdevice-gen; cargo build --release)

for s in mbr s112 s113 s122 s132 s140; do 
    ./nrf-softdevice-gen/target/release/nrf-softdevice-gen ./softdevice/$s/headers ./nrf-softdevice-$s/src/bindings.rs
    rustfmt ./nrf-softdevice-$s/src/bindings.rs
    (cd nrf-softdevice-$s; cargo build --target thumbv7em-none-eabihf)
done