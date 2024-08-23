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

- nRF52805
- nRF52810
- nRF52811
- nRF52820
- nRF52832
- nRF52833
- nRF52840

Some softdevices support only some chips, check Nordic's documentation for details.

## Setting up your build environment

This project used to require nightly toolchain features, which have been recently stabilized.
Therefore please ensure that your toolchains are up to date, by fetching latest stable toolchain:

```
rustup update
```

You will also need [`probe-rs`](https://probe.rs/) - a utility to enable `cargo run` to run embedded applications on a device. Install it following the instructions on [`the probe-rs website`](https://probe.rs/).

## Running examples

The following instructions are for the S140 and nRF52840-DK. You may have to adjust accordingly and can do so by modifying the `cargo.toml` of the examples folder -
please check out the `nrf-softdevice` and `nrf-softdevice-s140` dependency declarations.

Flashing the softdevice is required. It is NOT part of the built binary. You only need to do it once at the beginning, or after doing full chip erases.

- Download SoftDevice S140 from Nordic's website [here](https://www.nordicsemi.com/Software-and-tools/Software/S140/Download). Supported versions are 7.x.x
- Unzip
- As a debug client, if you are using
  - probe-rs:
    - Erase the flash with `probe-rs erase --chip nrf52840_xxAA` (You may have to supply additional `--allow-erase-all` argument).
    - Flash the SoftDevice with `probe-rs download --verify --binary-format hex --chip nRF52840_xxAA s140_nrf52_7.X.X_softdevice.hex`
  - nrfjprog:
    - Flash the SoftDevice with `nrfjprog --family NRF52 --chiperase --verify --program s140_nrf52_7.0.1_softdevice.hex`

To run an example, simply use `cargo run` from the `examples` folder:

- `cd examples && cargo run --bin ble_bas_peripheral --features nrf52840-dk`

Examples can also built for nrf52832 development kit targeting S132 softdevice (feature flag `nrf52832-dk`), or for nrf52833 targeting S140 softdevice on the BBC micro:bit v2 (feature flag `microbit-v2`). In these cases, edit `.cargo/config.toml` as needed.

## Configuring a SoftDevice

The first thing to do is find out how much flash the SoftDevice you've chosen uses. Look in the release notes, or google for your SoftDevice version and "memory map". For an s132 v7.3 its listed as 0x26000, or in human readable numbers 152K (0x26000 in hex is 155648 in decimal / 1024 bytes = 152K)

Set the memory.x to move your applications flash start to after the SoftDevice size and subtract it from the total available size:

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

Next you need to find out if your board has an external oscillator (which provides better battery life) But if in doubt just assume it doesn't and set the SoftDevice to use an internal clock. A common no external crystal configuration for nRF52 might be

```rust
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
```

## Interrupts

The SoftDevice does time-critical radio processing at high priorities. If its timing is disrupted, it will raise "assertion failed" errors. There's two common mistakes to avoid: (temporarily) disabling the softdevice's interrupts, and running your interrupts at too high priority.

These mistakes WILL cause "assertion failed" errors, 100% guaranteed. If you do these only "a little bit", such as disabling all interrupts but for very short periods of time only, things may appear to work, but you will get "assertion failed" errors after hours
of running. Make sure to follow them to the letter.

The Softdevice Driver (e.g. `Softdevice::run()`) cannot be used from interrupts by default. However, the `usable-from-interrupts` feature enables this functionality. To use this feature, a `critical-section` implementation is required.
This crate's internal implementation (`critical-section-impl` feature) is recommended, but other Softdevice-compatible implementations should also work.

### Critical sections

Interrupts for certain peripherals and SWI/EGUs are [reserved for the SoftDevice](https://infocenter.nordicsemi.com/topic/sds_s140/SDS/s1xx/sd_resource_reqs/hw_block_interrupt_vector.html?cp=4_7_4_0_6_0). Interrupt handlers for them are reserved by the softdevice, the handlers in your application won't be called.

DO NOT disable the softdevice's interrupts. You MUST NOT use the widely-used `cortex_m::interrupt::free` for "disable all interrupts" critical sections. Instead, use the [`critical-section`](https://crates.io/crates/critical-section) crate, which allows custom critical-section implementations:

- Make sure the `critical-section-impl` Cargo feature is enabled for `nrf-softdevice`. This makes `nrf-softdevice` emit a custom critical section implementation that disables only non-softdevice interrupts.
- Use `critical_section::with` instead of `cortex_m::interrupt::free`. This uses the custom critical-section impl.
- Use `embassy_sync::blocking_mutex::CriticalSectionMutex` instead of `cortex_m::interrupt::Mutex`.

Make sure you're not using any library that internally uses `cortex_m::interrupt::free` as well.

### Interrupt priority

Interrupt priority levels 0, 1, and 4 are [reserved for the SoftDevice](https://infocenter.nordicsemi.com/topic/sds_s140/SDS/s1xx/processor_avail_interrupt_latency/exception_mgmt_sd.html?cp=4_7_4_0_15_1). Make sure to not use them.

The default priority level for interrupts is 0, so for _every single interrupt_ you enable, make sure to set the priority level explicitly. For example:

```rust
use embassy_nrf::interrupt::{self, InterruptExt};

interrupt::SPIM3.set_priority(interrupt::Priority::P3);
let mut spim = spim::Spim::new(p.SPI3, Irqs, p.P0_13, p.P0_16, p.P0_15, config);
```

If you're using `embassy-nrf` with the `gpiote` or `time-driver-rtc1` features enabled, you'll need to edit your embassy_config to move those priorities:

```rust
// 0 is Highest. Lower prio number can preempt higher prio number
// Softdevice has reserved priorities 0, 1 and 4
let mut config = embassy_nrf::config::Config::default();
config.gpiote_interrupt_priority = Priority::P2;
config.time_interrupt_priority = Priority::P2;
let peripherals = embassy_nrf::init(config);
```

## Troubleshooting

### Interrupt priorities

If you are sure you have set interrupts correctly, but are still getting an error like below:

```
[ERROR]Location<lib.rs:104>panicked at 'sd_softdevice_enable err SdmIncorrectInterruptConfiguration'
```

Make sure the `defmt` feature is enabled on `embassy_nrf`.

You can then use this code to print whether an interrupt is enabled, and its priority:

```rust
// NB! MAX_IRQ depends on chip used, for example: nRF52840 has 48 IRQs, nRF52832 has 38.
const MAX_IRQ: u16 = ...;

use embassy_nrf::interrupt::{Interrupt, InterruptExt};
for num in 0..=MAX_IRQ {
    let interrupt = unsafe { core::mem::transmute::<u16, Interrupt>(num) };
    let is_enabled = InterruptExt::is_enabled(interrupt);
    let priority = InterruptExt::get_priority(interrupt);

    defmt::println!("Interrupt {}: Enabled = {}, Priority = {}", num, is_enabled, priority);
}
```

Interrupt numbers map to what they are in the [`Interrupt` enum](https://docs.embassy.dev/embassy-nrf/git/nrf52832/interrupt/enum.Interrupt.html).

If your SoftDevice is hardfaulting on enable and you think you have everything right, make sure to go back and do a full chip erase or recover, and reflash the SoftDevice again. A few bytes of empty space after the SoftDevice are required to be 0xFF, but might not be if the softdevice was flashed over an existing binary.

### Peripheral conflicts

If the following runtime error occurs

```
Softdevice memory access violation. Your program accessed registers for a peripheral reserved to the softdevice. PC=2a644 PREGION=8192
```

check which peripherals are used by application.

Softdevice uses number of peripherals for its functionality when its enabled (and even disabled), and therefore
enforces certain limits to [availability of peripherals](https://infocenter.nordicsemi.com/topic/sds_s132/SDS/s1xx/sd_resource_reqs/hw_block_interrupt_vector.html):

1. Open - peripheral is not used by SoftDevice and application has full access.
2. Blocked - peripheral is used by SoftDevice, and all application access is disabled. Though, certain peripherals (RADIO, TIMER0, CCM, and AAR) could be accessed via the Softdevice Radio Timeslot API.
3. Restricted - peripheral is used by SoftDevice, but it can have limited access via SoftDevice API. For example
   [`FLASH`](https://github.com/embassy-rs/nrf-softdevice/blob/master/nrf-softdevice/src/flash.rs),
   [`RNG`](https://github.com/embassy-rs/nrf-softdevice/blob/master/nrf-softdevice/src/random.rs) and
   [`TEMP`](https://github.com/embassy-rs/nrf-softdevice/blob/master/nrf-softdevice/src/temperature.rs) peripherals.

### Linking issues

If the following linking error occurs

```
rust-lld: error: undefined symbol: _critical_section_release
```

make sure the feature `critical-section-impl` is enabled and also that the softdevice is included in the code, e.g. `use nrf_softdevice as _;`.

If running the firmware timeouts after flashing, make sure the size and location of the RAM and FLASH region in the linker script is correct.

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
