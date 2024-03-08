#!/bin/bash

NRFXLIB=${NRFXLIB:-../sdk-nrfxlib}

set -euxo pipefail

(cd nrf-softdevice-controller-gen; cargo build --release)

for s in soft-float; do 
    ./nrf-softdevice-controller-gen/target/release/nrf-softdevice-controller-gen ${NRFXLIB}/softdevice_controller/include ./nrf-softdevice-controller-$s/src/bindings.rs
    rustfmt ./nrf-softdevice-controller-$s/src/bindings.rs
    (cd nrf-softdevice-controller-$s; cargo build --target thumbv7em-none-eabihf)
done
