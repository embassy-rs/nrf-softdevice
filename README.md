# `nrf-softdevice`

Rust bindings for Nordic Semiconductor nRF series SoftDevices.

SoftDevices are a closed source C binary written by Nordic for their microcontrollers that sits at the bottom of flash and is called first on startup. The softdevice then calls your application or bootloader or whatever is sitting directly after it in flash.

They are full featured, battle tested, and pre qualified for bluetooth certification and thus make valuable bluetooth stacks when bindgened to Rust -- at least until we get a Rust bluetooth stack certified to be shipped commercially. Different SoftDevices support specific chips as well as certain features, like working only as a peripheral, or both a peripheral and central, or even offer alternate radio configuration like ant.

Besides the handicap of being closed source, the cost of SoftDevices is they steal away resources like ram and flash as well as timer peripherals and several priorities of interrupts from your application.

## High-level bindings

The `nrf-softdevice` crate contains high-level easy-to-use Rust async/await bindings for the Softdevice.

Working:

- Safe interrupt management
- Async flash API
- Bluetooth central (scanning and connecting)
- Bluetooth peripheral (advertising, connectable-only for now)
- GATT client
- GATT server
- L2CAP Connection-Oriented channels
- Data length extension
- ATT MTU extension
- Get/set own BLE address

To use it you must specify the following Cargo features:

- exactly one softdevice model, for example feature `s140`.
- exactly one supported nRF chip model, for example feature `nrf52840`.

The following softdevices are supported.

- S112 (peripheral only)
- S113 (peripheral only)
- S122 (central only)
- S132 (central and peripheral)
- S140 v7.x.x (central and peripheral)

The following nRF chips are supported

- nRF52810
- nRF52832
- nRF52833
- nRF52840

Some softdevices support only some chips, check Nordic's documentation for details.

## Setting up your build environment

This project uses new toolchain features, often only to be found in nightly. Please ensure that your toolchains are up to date:

```
rustup update
```

You will also need [`probe-run`](https://ferrous-systems.com/blog/probe-run/) - a utility to enable `cargo run` to run embedded applications on a device:

```
cargo install probe-run
```

## Running examples

The following instructions are for the S140 and nRF52840-DK. You may have to adjust accordingly and can do so by modifying the `cargo.toml` of the examples folder - 
please check out the `nrf-softdevice` and `nrf-softdevice-s140` dependency declarations.

Flashing the softdevice is required. It is NOT part of the built binary. You only need to do it once at the beginning, or after doing full chip erases.

- Download SoftDevice S140 from Nordic's website [here](https://www.nordicsemi.com/Software-and-tools/Software/S140/Download). Supported versions are 7.x.x
- Unzip
- As a debug client, if you are using 
  - probe-rs:
    - Erase the flash with `probe-rs-cli erase --chip nrf52840`
    - Flash the SoftDevice with `probe-rs-cli download --chip nrf52840 --format hex s140_nrf52_7.X.X_softdevice.hex`
  - nrfjprog:
    - Flash the SoftDevice with `nrfjprog --family NRF52 --chiperase --verify --program s140_nrf52_7.0.1_softdevice.hex`

To run an example, simply use `cargo run` from the `examples` folder:

- `cd examples && cargo run --bin ble_bas_peripheral`

## Configuring a SoftDevice

The first thing to do is find out how much flash the SoftDevice you've chosen uses. Look in the release notes, or google for your SoftDevice version and "memory map". For an s132 v7.3 its listed as 0x26000, or in human readable numbers 152K (0x26000 in hex is 155648 in decimal / 1024 bytes = 152K)

Set the memory.x to move your applications flash start to after the SoftDevice size and subtract it from the total avialable size:

```bash
MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* These values correspond to the NRF52832 with SoftDevices S132 7.3.0 */
  FLASH : ORIGIN = 0x00000000 + 152K, LENGTH = 512K - 152K
  RAM : ORIGIN = 0x20000000 + 44K, LENGTH = 64K - 44K
}
```

You can pick mostly anything for ram right now as if you have defmt logging enabled, the SoftDevice will tell you what the right number is when you call enable:

```bash
1 INFO  softdevice RAM: 41600 bytes
└─ nrf_softdevice::softdevice::{impl#0}::enable @ /home/jacob/.cargo/git/checkouts/nrf-softdevice-03ef4aef10e777e4/fa369be/nrf-softdevice/src/fmt.rs:138
2 ERROR panicked at 'too little RAM for softdevice. Change your app's RAM start address to 2000a280'
```

You have some control over that number by tweaking the SoftDevice configuration parameters. See especially the concurrent connection parameters. If you dont need to support multiple connections these can really decrease your ram size:

- conn_gap.conn_count The number of concurrent connections the application can create with this configuration
- periph_role_count Maximum number of connections concurrently acting as a peripheral
- central_role_count Maximum number of connections concurrently acting as a central

Next you need to find out if your board has an external oscillator (which provides better battery life) But if in doubt just assume it doesnt and set the SoftDevice to use an internal clock. A common no external crystal configuration for nRF52 might be

```rust
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_250_PPM as u8,
        }),
```

The SoftDevice steals interrupt priority levels 0, 1, and 3 from your application. So make sure route around those. If you're using embassy and you have the `gpiote` and the `time-driver-rtc1` features enabled for instance you'll need to edit your embassy_config to move those priorities:

```rust
/ 0 is Highest. Lower prio number can preempt higher prio number
// Softdevice has reserved priorities 0, 1 and 3
pub fn embassy_config() -> embassy_nrf::config::Config {
    let mut config = embassy_nrf::config::Config::default();
    config.hfclk_source = embassy_nrf::config::HfclkSource::Internal;
    config.lfclk_source = embassy_nrf::config::LfclkSource::InternalRC;
    config.time_interrupt_priority = interrupt::Priority::P2;
    // if we see button misses lower this
    config.gpiote_interrupt_priority = interrupt::Priority::P7;
    config
}
```

## Troubleshooting

If your SoftDevice is hardfaulting on enable and you think you have everything right, mak sure to go back and do a full chip erase or recover, and reflash the SoftDevice again. A few bytes of empty space after the SoftDevice are required to be 0xFF, but might not be if the softdevice was flashed over an existing binary.

## Low-level raw bindings

The `nrf-softdevice-s1xx` crates contain low-level bindings, matching 1-1 with the softdevice C headers.

They are generated with `bindgen`, with extra post-processing to correctly generate the `svc`-based softdevice calls.

Generated code consists of inline functions using inline ASM, ensuring the lowest possible
overhead. Most of the times you'll see them inlined as a single `svc` instruction in the
calling function. Here is an example:

```rust
#[inline(always)]
pub unsafe fn sd_ble_gap_connect(
      p_peer_addr: *const ble_gap_addr_t,
      p_scan_params: *const ble_gap_scan_params_t,
      p_conn_params: *const ble_gap_conn_params_t,
      conn_cfg_tag: u8,
) -> u32 {
    let ret: u32;
    core::arch::asm!("svc 140",
        inout("r0") p_peer_addr => res,
        inout("r1") p_scan_params => _,
        inout("r2") p_conn_params => _,
        inout("r3") conn_cfg_tag => _,
        lateout("r12") _,
    );
    ret
}
```

### Generating

The bindings are generated from the headers with the `gen.sh` script.

## License

This repo includes the softdevice headers, which are licensed under [Nordic's proprietary license](LICENSE-NORDIC).
Generated `binding.rs` files are a derived work of the headers, so they are also subject to Nordic's license.

The high level bindings ([nrf-softdevice](nrf-softdevice)) and the generator
code ([nrf-softdevice-gen](nrf-softdevice-gen)) are licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
