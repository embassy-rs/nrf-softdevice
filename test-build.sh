#!/bin/bash

set -euxo pipefail

# Check that examples build

cargo build --target thumbv7em-none-eabihf -p nrf-softdevice-examples --bins

# Check that build works with all supported combinations.

cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s112,nrf52810,ble-peripheral
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s112,nrf52832,ble-peripheral

cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s113,nrf52810,ble-peripheral
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s113,nrf52810,ble-peripheral,ble-l2cap
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s113,nrf52832,ble-peripheral
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s113,nrf52832,ble-peripheral,ble-l2cap
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s113,nrf52833,ble-peripheral
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s113,nrf52833,ble-peripheral,ble-l2cap
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s113,nrf52840,ble-peripheral
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s113,nrf52840,ble-peripheral,ble-l2cap

cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s122,nrf52833,ble-central

cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s132,nrf52810,ble-central
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s132,nrf52810,ble-central,ble-l2cap
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s132,nrf52810,ble-peripheral
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s132,nrf52810,ble-peripheral,ble-l2cap
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s132,nrf52810,ble-central,ble-peripheral
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s132,nrf52810,ble-central,ble-peripheral,ble-l2cap
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s132,nrf52832,ble-central
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s132,nrf52832,ble-central,ble-l2cap
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s132,nrf52832,ble-peripheral
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s132,nrf52832,ble-peripheral,ble-l2cap
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s132,nrf52832,ble-central,ble-peripheral
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s132,nrf52832,ble-central,ble-peripheral,ble-l2cap

cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s140,nrf52833,ble-central
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s140,nrf52833,ble-central,ble-l2cap
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s140,nrf52833,ble-peripheral
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s140,nrf52833,ble-peripheral,ble-l2cap
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s140,nrf52833,ble-central,ble-peripheral
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s140,nrf52833,ble-central,ble-peripheral,ble-l2cap
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s140,nrf52840,ble-central
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s140,nrf52840,ble-central,ble-l2cap
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s140,nrf52840,ble-peripheral
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s140,nrf52840,ble-peripheral,ble-l2cap
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s140,nrf52840,ble-central,ble-peripheral
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s140,nrf52840,ble-central,ble-peripheral,ble-l2cap
cargo build --target thumbv7em-none-eabihf -p nrf-softdevice --features s140,nrf52840,ble-central,ble-peripheral,ble-l2cap

# https://www.nordicsemi.com/Software-and-tools/Software/Bluetooth-Software

#      | Central  Peripheral | nrf52805  nrf52810  nrf52811  nrf52820  nrf52832  nrf52833, nrf52840
# -----|---------------------|--------------------------------------------------------------------------
# s112 |              X      |    X         X         X         X         X
# s113 |              X      |    X         X         X         X         X         X         X
# s122 |    X                |                                  X                   X
# s132 |    X         X      |              X                             X  
# s140 |    X         X      |                        X         X                   X         X

# s112 nrf52805
# s112 nrf52810
# s112 nrf52811
# s112 nrf52820
# s112 nrf52832 
# 
# s113 nrf52805
# s113 nrf52810
# s113 nrf52811
# s113 nrf52820
# s113 nrf52832
# s113 nrf52833
# s113 nrf52840 
# 
# s122 nrf52820
# s122 nrf52833 
# 
# s132 nrf52810
# s132 nrf52832 
# 
# s140 nrf52811
# s140 nrf52820
# s140 nrf52833
# s140 nrf52840