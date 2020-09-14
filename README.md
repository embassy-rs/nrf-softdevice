# `nrf-softdevice`

Rust bindings for Nordic Semiconductor nRF series SoftDevices.

## High-level bindings

The `nrf-softdevice` crate contains high-level easy-to-use Rust async/await bindings for the Softdevice.

Working:

- Safe interrupt management
- Async flash API
- Bluetooth central (scanning)
- Bluetooth peripheral (advertising, connectable-only for now)
- GATT client

Work in progress:

- GATT servers
- Data length extension
- ATT MTU extension

To use it you must specify the following Cargo features:

- exactly one softdevice model, for example feature `s140`.
- exactly one supported nRF chip model, for example feature `nrf52840`.

The following softdevices are supported.

- S112 (peripheral only)
- S113 (peripheral only)
- S122 (central only)
- S132 (central and peripheral)
- S140 (central and peripheral)

The following nRF chips are supported

- nRF52810
- nRF52832
- nRF52832
- nRF52840

Some softdevices support only some chips, check Nordic's documentation for details.

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
    asm!("svc 140",
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
