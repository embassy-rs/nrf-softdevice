use uuid::uuid;

use super::advertisement_builder::*;

#[test]
fn basic() {
    let adv_data = AdvertisementData::new()
        .flags([Flag::GeneralDiscovery, Flag::LE_Only])
        .services(Complete16([BasicService::HealthThermometer]))
        .name(FullName("Full Name"));

    #[rustfmt::skip]
    assert_eq!(
        adv_data.as_slice(),
        &[
            0x02, 0x01, 0x06,
            0x03, 0x03, 0x09, 0x18,
            0x0a, 0x09, b'F', b'u', b'l', b'l', b' ', b'N', b'a', b'm', b'e'
        ]
    );
}

#[test]
fn custom_service() {
    let test_service = CustomService(uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").as_bytes().clone());

    let adv_data = AdvertisementData::new()
        .flags([Flag::GeneralDiscovery, Flag::LE_Only])
        .services(Complete128([test_service]))
        .name(ShortName("ShrtNm"));

    #[rustfmt::skip]
    assert_eq!(
        adv_data.as_slice(),
        &[
            0x02, 0x01, 0x06,
            0x11, 0x07, 0xc8, 0xe0, 0x5f, 0x0e, 0x68, 0xbb, 0x47, 0x92, 0x6f, 0x42, 0xb1, 0x10, 0x44, 0x50, 0xe5, 0x67,
            0x07, 0x08, b'S', b'h', b'r', b't', b'N', b'm'
        ]
    );
}

#[test]
fn raw() {
    let adv_data = AdvertisementData::new()
        .flags([Flag::GeneralDiscovery, Flag::LE_Only])
        .raw(ADType::URI, &[0xde, 0xad, 0xbe, 0xef])
        .name(ShortName("ShrtNm"));

    #[rustfmt::skip]
    assert_eq!(
        adv_data.as_slice(),
        &[
            0x02, 0x01, 0x06,
            0x05, 0x24, 0xde, 0xad, 0xbe, 0xef,
            0x07, 0x08, b'S', b'h', b'r', b't', b'N', b'm'
        ]
    );
}
